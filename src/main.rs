use axum::{Json, Router, routing::post};
use llm_cache_service::{
    cache::{CacheLayer, sqlite_cache::SqliteCache},
    config::Config,
    prompts::{
        loader::load_prompt_profile, prompt_provider::PromptProvider,
        provider_factory::ProviderFactory,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Deserialize)]
struct LLMRequest {
    content: String,
}
#[derive(Serialize)]
struct LLMResponse {
    response: String,
}

async fn llm_handler<L1, L2>(
    Json(payload): Json<LLMRequest>,
    param_provider: Arc<PromptProvider<L1, L2>>,
) -> Result<Json<LLMResponse>, String>
where
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

    info!(
        "initialising SQLite cache (L1) at {}",
        loaded_config.l1_cache_url
    );
    let l1_cache =
        SqliteCache::with_builder(&loaded_config.l1_cache_url, loaded_config.cache_ttl, |b| b)
            .await
            .map_err(|e| {
                error!("Failed to initialize L1 cache: {}", e);
                e
            })?;
    let l1_cache = Arc::new(l1_cache);

    info!(
        "initialising SQLite cache (L2) at {}",
        loaded_config.l2_cache_url
    );
    let l2_cache =
        SqliteCache::with_builder(&loaded_config.l2_cache_url, loaded_config.cache_ttl, |b| b)
            .await
            .map_err(|e| {
                error!("Failed to initialize L2 cache: {}", e);
                e
            })?;
    let l2_cache = Arc::new(l2_cache);

    let profiles = load_prompt_profile("prompts")?;
    let provider_factory = ProviderFactory {
        default_provider: loaded_config.llm_provider,
        default_model: loaded_config.llm_name,
        api_key: loaded_config.api_key,
        l1_cache: l1_cache.clone(),
        l2_cache: l2_cache.clone(),
    };

    let mut app = Router::new();

    for profile in profiles {
        let provider = provider_factory.build_provider(&profile);
        let route_path = format!("/llm/{}", profile.name);
        app = app.route(
            &route_path,
            post({
                let provider = provider.clone();
                move |payload| llm_handler(payload, provider.clone())
            }),
        );
    }

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
