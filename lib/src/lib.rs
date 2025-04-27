use std::collections::HashMap;

use aggregator::Aggregator;
use anyhow::Result;
use errors::{EngineError, EngineErrorType};
use handler::EngineHandler;
use network::NetworkHandler;

mod aggregator;
mod engines;
pub mod errors;
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

/// Document type to search for
#[derive(Debug, Copy, Clone, Deserialize)]
pub enum QueryType {
    Text,
    Image,
    File,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Query {
    pub text: String,
    pub qtype: QueryType,
    pub page: Option<u16>,
    // If safe search level is left `None`, the engine will use the provided default level
    pub safe_search_level: Option<SafeSearchLevel>,
    pub relavancy: Option<Relavancy>,
}

// Search result returned by an engine
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
    pub fn new(
        url: &str,
        title: &str,
        description: &str,
        source: &str,
    ) -> Result<Self, EngineErrorType> {
        Ok(Self {
            url: Url::parse(url).map_err(|_| EngineErrorType::ParseFailed)?,
            title: title.to_string(),
            description: description.to_string(),
            score: 0.0,
            sources: vec![source.to_string()],
        })
    }
}

#[derive(Serialize, Debug)]
pub struct QueryResult {
    pub results: Vec<SearchResult>,
    pub errors: Vec<EngineError>,
}

pub struct Handler {
    aggregator: Aggregator,
    engine_handler: EngineHandler,
    /// Default safe search level
    safe_search_level: SafeSearchLevel,
}

impl Handler {
    pub async fn new(
        engine_score_multipliers: HashMap<String, f32>,
        timeout: u16,
        proxy_url: Option<&str>,
        is_tor: Option<bool>,
        engines: &[String],
        user_agents: Vec<String>,
        safe_search_level: SafeSearchLevel,
    ) -> Result<Self> {
        let aggregator = Aggregator::new(engine_score_multipliers);
        let network_handler = NetworkHandler::new(timeout, proxy_url, is_tor, user_agents).await?;
        let engine_handler = EngineHandler::new(engines, network_handler)?;

        Ok(Self {
            aggregator,
            engine_handler,
            safe_search_level,
        })
    }

    pub async fn search(&self, mut query: Query) -> QueryResult {
        query
            .safe_search_level
            .get_or_insert(self.safe_search_level.clone());

        let (raw_results, errors) = self.engine_handler.search(query).await;

        let results = self.aggregator.process(raw_results);

        QueryResult { results, errors }
    }
}
