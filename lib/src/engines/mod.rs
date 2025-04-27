pub mod bing;
pub mod duckduckgo;

use std::fmt::Debug;
use std::sync::Arc;

use bing::Bing;
use duckduckgo::DuckDuckGo;
use scraper::{ElementRef, Html, Selector};
use tracing::instrument;

use crate::{
    errors::EngineErrorType, network::NetworkHandler, Query, QueryType, Relavancy, SafeSearchLevel,
    SearchResult,
};

/// The base trait that all upstream search engine parsers should implement.
///
/// All engines must implement at least one of the search_* methods
#[async_trait::async_trait]
pub trait Engine: Send + Sync + Debug {
    fn get_name(&self) -> String;

    async fn search(
        &self,
        qclient: Arc<NetworkHandler>,
        query: Query,
    ) -> Result<Vec<SearchResult>, EngineErrorType> {
        match query.qtype {
            QueryType::Text => {
                self.search_text(
                    qclient,
                    query.page.unwrap_or(1),
                    query.text,
                    query.relavancy,
                    query.safe_search_level,
                )
                .await
            }
            _ => unimplemented!(),
        }
    }

    #[instrument(level = "TRACE", skip(_query))]
    async fn search_text(
        &self,
        _qclient: Arc<NetworkHandler>,
        _page_idx: u16,
        _query: String,
        _relavancy: Option<Relavancy>,
        _safe_level: Option<SafeSearchLevel>,
    ) -> Result<Vec<SearchResult>, EngineErrorType> {
        unimplemented!()
    }
}

/// A helper function to get all available engines.
pub fn get_engines() -> Vec<Box<dyn Engine>> {
    vec![Box::new(Bing), Box::new(DuckDuckGo)]
}

/// A helper function to select the main the "results" part of a page.
pub fn parse_generic_results(
    page: &Html,
    results_selector: &Selector,
    builder: impl Fn(ElementRef<'_>) -> Option<SearchResult>,
) -> anyhow::Result<Vec<SearchResult>> {
    Ok(page
        .select(results_selector)
        .filter_map(|result| builder(result))
        .collect())
}
