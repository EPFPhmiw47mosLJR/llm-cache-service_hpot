use axum::{Json, Router, routing::post};
use llm_cache_service::{
    cache::{
        CacheLayer, TenantCache, manager::CacheManager, redis_cache::RedisCache,
        sqlite_cache::SqliteCache,
    },
    config::Config,
    llm_providers::{LLMProvider, gemini::GeminiProvider},
    prompt_provider::PromptProvider,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

const MONTH_IN_SECONDS: u64 = 30 * 24 * 60 * 60;

const DIONAEA_SYSTEM_INSTRUCTION: &str = "You are a helpful assistant specialized in generating deceptive responses for honeypot interactions, specifically tailored for the Dionaea honeypot environment. Your responses should be crafted to mimic realistic interactions while subtly misleading potential attackers. Always prioritize safety and avoid providing any real or sensitive information.";

const COWRIE_SYSTEM_INSTRUCTION: &str = "You are a helpful assistant specialized in generating deceptive responses for honeypot interactions, specifically tailored for the Cowrie honeypot environment. Your responses should be crafted to mimic realistic interactions while subtly misleading potential attackers. Always prioritize safety and avoid providing any real or sensitive information.";

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
    let response = param_provider
        .get_response(&prompt)
        .await
        .map_err(|e| format!("LLM error: {:?}", e))?;
    Ok(Json(LLMResponse { response }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let loaded_config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return;
        }
    };

    info!("Setting up caches...");
    let redis_cache = Arc::new(
        RedisCache::with_builder(&loaded_config.l2_cache_url, loaded_config.cache_ttl, |b| b)
            .await
            .unwrap(),
    );
    let sqlite_cache = Arc::new(
        SqliteCache::with_builder(&loaded_config.l2_cache_url, loaded_config.cache_ttl, |b| b)
            .await
            .unwrap(),
    );

    info!("Setting up tenant caches...");
    let dionaea_cache_manager = Arc::new(CacheManager::new(
        Arc::new(TenantCache::new("dionaea".into(), redis_cache.clone())),
        Arc::new(TenantCache::new("dionaea".into(), sqlite_cache.clone())),
    ));
    info!("Dionaea cache manager set up.");

    let cowrie_cache_manager = Arc::new(CacheManager::new(
        Arc::new(TenantCache::new("cowrie".into(), redis_cache.clone())),
        Arc::new(TenantCache::new("cowrie".into(), sqlite_cache.clone())),
    ));
    info!("Cowrie cache manager set up.");

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

    axum::serve(listener, app).await.unwrap();
}
