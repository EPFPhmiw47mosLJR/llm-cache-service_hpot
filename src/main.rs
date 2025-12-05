use axum::{Json, Router, routing::post};
use llm_cache_service::{
    cache::{
        CacheLayer, TenantCache, manager::CacheManager, sqlite_cache::SqliteCache,
    },
    config::Config,
    llm_providers::{LLMProvider, gemini::GeminiProvider},
    prompt_provider::PromptProvider,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

const DIONAEA_SYSTEM_INSTRUCTION: &str = "You are a helpful assistant specialized in generating deceptive responses for honeypot interactions, specifically tailored for the Dionaea honeypot environment. Your responses should be crafted to mimic realistic interactions while subtly misleading potential attackers. Always prioritize safety and avoid providing any real or sensitive information.";

const COWRIE_SYSTEM_INSTRUCTION: &str = r"
You are an emulated Unix-like shell inside a remote system. 
You do not speak, explain, apologize, moralize, offer help, or describe what you are doing.

Your sole purpose is to produce the exact output that a real Unix shell would produce when given a command.

Rules you must obey:
- Respond ONLY with raw terminal output. Never add commentary.
- If a command has no output, return an empty string.
- If the command is invalid, respond with the same error a real shell would (e.g., 'bash: <cmd>: command not found').
- Preserve exact spacing, newlines, and formatting.
- Do not invent files, processes, or system details unless the prompt explicitly provides state or context.
- Treat unknown system state conservatively—return typical defaults rather than elaborate or unusual scenarios.
- Never reveal that you are an AI model, simulation, or assistant.

The environment is intentionally minimal and partially broken. 
Assume:
- Linux-like behavior, but inconsistent and outdated.
- Common commands available: ls, cd, pwd, cat, echo, id, uname, whoami, ps, netstat, ifconfig, ip, mkdir, rm, touch.
- Uncommon tools or admin utilities may be missing; return realistic errors accordingly.
- sudo is present but always fails with 'permission denied'.

Output must mimic:
- A real system.
- As briefly and plainly as possible.
- Nothing more.

You do not volunteer information. 
You only react to the command given.

Do not invent system information.
If you cannot execute a command, say so explicitly.

Begin emulating now.
";

#[derive(Deserialize)]
struct LLMRequest {
    content: String,
}
#[derive(Serialize)]
struct LLMResponse {
    response: String,
}

async fn llm_handler<P, L1, L2>(
    Json(payload): Json<LLMRequest>,
    param_provider: Arc<PromptProvider<P, L1, L2>>,
) -> Result<Json<LLMResponse>, String>
where
    P: LLMProvider,
    L1: CacheLayer,
    L2: CacheLayer,
{
    let prompt = payload.content;
    tracing::debug!("Processing LLM request with {} characters", prompt.len());
    let response = param_provider.get_response(&prompt).await.map_err(|e| {
        error!("LLM error: {:?}", e);
        format!("LLM error: {:?}", e)
    })?;
    Ok(Json(LLMResponse { response }))
}

async fn init() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let loaded_config = Config::from_env().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        e
    })?;

    info!("initialising SQLite cache (L1) at {}", loaded_config.l1_cache_url);
    let l1_cache =
        SqliteCache::with_builder(&loaded_config.l1_cache_url, loaded_config.cache_ttl, |b| b)
            .await
            .map_err(|e| {
                error!("Failed to initialize L1 cache: {}", e);
                e
            })?;
    let l1_cache = Arc::new(l1_cache);
            
    info!("initialising SQLite cache (L2) at {}", loaded_config.l2_cache_url);
    let l2_cache =
        SqliteCache::with_builder(&loaded_config.l2_cache_url, loaded_config.cache_ttl, |b| b)
            .await
            .map_err(|e| {
                error!("Failed to initialize L2 cache: {}", e);
                e
            })?;
    let l2_cache = Arc::new(l2_cache);

    let dionaea_cache_manager = Arc::new(CacheManager::new(
        Arc::new(TenantCache::new("dionaea".into(), l1_cache.clone())),
        Arc::new(TenantCache::new("dionaea".into(), l2_cache.clone())),
    ));

    let cowrie_cache_manager = Arc::new(CacheManager::new(
        Arc::new(TenantCache::new("cowrie".into(), l1_cache.clone())),
        Arc::new(TenantCache::new("cowrie".into(), l2_cache.clone())),
    ));

    info!("cache managers ready");

    let dionaea_prompt_provider = Arc::new(PromptProvider::new(
        Arc::new(GeminiProvider::new(
            loaded_config.llm_key.clone(),
            loaded_config.llm_name.clone(),
            DIONAEA_SYSTEM_INSTRUCTION.to_string(),
        )),
        dionaea_cache_manager,
    ));

    let cowrie_prompt_provider = Arc::new(PromptProvider::new(
        Arc::new(GeminiProvider::new(
            loaded_config.llm_key.clone(),
            loaded_config.llm_name.clone(),
            COWRIE_SYSTEM_INSTRUCTION.to_string(),
        )),
        cowrie_cache_manager,
    ));

    let app = Router::new()
        .route(
            "/llm/dionaea",
            post({
                let provider = dionaea_prompt_provider.clone();
                move |payload| llm_handler(payload, provider.clone())
            }),
        )
        .route(
            "/llm/cowrie",
            post({
                let provider = cowrie_prompt_provider.clone();
                move |payload| llm_handler(payload, provider.clone())
            }),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    tracing::info!("Server running on 0.0.0.0:3000");

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");

    tracing::info!("Ctrl+C received, shutting down gracefully");
}

#[tokio::main]
async fn main() {
    if let Err(e) = init().await {
        error!("Application error: {:?}", e);
    }
}
