use std::collections::HashMap;

use aggregator::Aggregator;
use anyhow::Result;
use errors::EngineError;
use handler::EngineHandler;
use network::NetworkHandler;

mod aggregator;
mod engines;
mod errors;
pub mod handler;
mod network;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum SafeSearchLevel {
    // No filtering
    Off,
    // The search engines will be requested to filter the results
    Medium,
    // `SafeSearchLevel::Medium` + internal allowlists and blocklists
    High,
}
/// Time Relavancy of query
#[derive(Debug, Copy, Clone, Deserialize)]
pub enum Relavancy {
    AnyTime,
    PastDay,
    PastWeek,
    PastMonth,
    PastYear,
}

// The definition for a search result returned by an engine
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub url: Url,
    pub title: String,
    pub description: String,
    pub score: f32,
    // List of search engines which suggested this result
    pub sources: Vec<String>,
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        // Ignore transport type as it always will http or https
        self.url.host().unwrap() == other.url.host().unwrap()
            && self.url.path() == other.url.path()
            && self.url.query() == other.url.query()
    }
}

impl SearchResult {
    pub fn new(url: &str, title: &str, description: &str, source: &str) -> Result<Self> {
        Ok(Self {
            url: Url::parse(url)?,
            title: title.to_string(),
            description: description.to_string(),
            score: 0.0,
            sources: vec![source.to_string()],
        })
    }
}

pub struct Handler {
    aggregator: Aggregator,
    engine_handler: EngineHandler,
}

impl Handler {
    pub async fn new(
        engine_score_multipliers: HashMap<String, f32>,
        timeout: u16,
        proxy_url: Option<&str>,
        is_tor: Option<bool>,
        engines: &[String],
    ) -> Result<Self> {
        let aggregator = Aggregator::new(engine_score_multipliers);
        let network_handler = NetworkHandler::new(timeout, proxy_url, is_tor).await?;
        let engine_handler = EngineHandler::new(engines, network_handler)?;

        Ok(Self {
            aggregator,
            engine_handler,
        })
    }

    pub async fn search(
        &self,
        query: String,
        page: u16,
        relavancy: Option<Relavancy>,
        safe_level: Option<SafeSearchLevel>,
    ) -> (Vec<SearchResult>, Vec<EngineError>) {
        let (raw_results, engine_errors) = self
            .engine_handler
            .search(query, page, relavancy, safe_level)
            .await;

        (self.aggregator.process(raw_results), engine_errors)
    }
}
