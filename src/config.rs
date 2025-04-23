use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use serde_yaml::from_reader;

/// Defines all the configuration for anvesh
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Whole numbers from 0 to 5. 0 corresponds to Trace, 1 to Debug, etc
    pub log_level: u8,
    /// Maximum threads to be used by the internal tokio scheduler.
    pub threads: usize,
    /// Port to run the web server on.
    pub port: u16,
    /// The interface to bind the web server to.
    pub bind_ip: String,
    /// Configuration for rate limiter.
    pub rate_limiter: RateLimiter,
    /// Common request timeout (in seconds) for the requests made to upstream engines.
    pub request_timeout: u16,
    /// User agents to be used when searching the web.
    pub user_agents: Vec<String>,
    /// Whole numbers from 0 to 3. 0 corresponds to no filtering, 1 to low, etc.
    pub safe_search_level: u8,
    /// Connection URL to redis server.
    pub redis_url: Option<String>,
    /// TTL for search results in cache.
    pub cache_expiry_time: u64,
    /// Configuration for proxy
    pub proxy: Option<ProxyConfig>,
    /// Specific upstream engine settings.
    pub upstream_search_engines: HashMap<String, EngineConfig>,
}

#[derive(Debug, Deserialize)]
pub enum ProxyType {
    Socks5,
    Http,
}

#[derive(Debug, Deserialize)]
pub struct ProxyConfig {
    pub connection_url: String,
    pub is_tor: bool,
    pub proxy_type: ProxyType,
}

#[derive(Debug, Deserialize)]
pub struct RateLimiter {
    pub number_of_requests: usize,
    pub time_limit: u64,
}

#[derive(Debug, Deserialize)]
pub struct EngineConfig {
    pub enabled: bool,
    pub timeout: u128,
    pub score_multiplier: f32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            enabled: true,
            timeout: 5,
            score_multiplier: 1.0,
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self {
            number_of_requests: 10,
            time_limit: 10,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: 0,
            threads: num_cpus::get(),
            port: 2000,
            bind_ip: "0.0.0.0".to_string(),
            rate_limiter: Default::default(),
            request_timeout: 5,
            user_agents: [].to_vec(),
            safe_search_level: 1,
            redis_url: None,
            cache_expiry_time: 0,
            upstream_search_engines: HashMap::from([
                ("Bing".to_string(), EngineConfig::default()),
                ("DuckDuckGo".to_string(), EngineConfig::default()),
            ]),
            proxy: None,
        }
    }
}

pub fn parse_config(path: Option<impl AsRef<Path>>) -> Result<Config> {
    match path {
        Some(p) => {
            let file = File::open(p)?;
            let config: Config = from_reader(file)?;
            Ok(config)
        }
        None => Ok(Config::default()),
    }
}
