use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub llm_name: String,
    pub llm_key: String,
    pub l1_cache_url: String,
    pub l2_cache_url: String,
    pub cache_ttl: u64,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let llm_name = env::var("LCS_LLM_NAME").expect("Missing required env var: LCS_LLM_NAME");

        let llm_key = env::var("LCS_LLM_KEY").expect("Missing required env var: LCS_LLM_KEY");

        let l1_cache_url =
            env::var("LCS_L1_CACHE_URL").expect("Missing required env var: LCS_L1_CACHE_URL");

        let l2_cache_url =
            env::var("LCS_L2_CACHE_URL").expect("Missing required env var: LCS_L2_CACHE_URL");

        let cache_ttl = env::var("LCS_CACHE_TTL")
            .expect("Missing required env var: LCS_CACHE_TTL")
            .parse()
            .map_err(|e| format!("Invalid LCS_CACHE_TTL: {}", e))?;

        let log_level = env::var("LCS_LOG_LEVEL").expect("Missing required env var: LCS_LOG_LEVEL");

        Ok(Config {
            llm_name,
            llm_key,
            l1_cache_url,
            l2_cache_url,
            cache_ttl,
            log_level,
        })
    }
}

/*
#[cfg(test)]
mod tests {
    #[test]
    fn default_config() {
        unimplemented!();
    }

    #[test]
    fn override_from_env() {
        unimplemented!();
    }

    #[test]
    fn invalid_port_returns_error() {
        unimplemented!();
    }
}
*/