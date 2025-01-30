use std::sync::Arc;

use axum::{extract::{Query, State}, Json};
use lib::{errors::EngineError, Handler, Relavancy, SafeSearchLevel, SearchResult};
use serde::{Deserialize, Serialize};

use crate::templates::{IndexTemplate, SearchTemplate};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    query: String,
    page: Option<u16>,
    relavancy: Option<Relavancy>,
    safe_level: Option<SafeSearchLevel>,
}

#[derive(Debug, Serialize)]
pub struct QueryResults {
    results: Vec<SearchResult>,
    errors: Vec<EngineError>,
}

pub async fn index_handler() -> IndexTemplate {
    IndexTemplate
}

pub async fn search_handler(
    Query(params): Query<SearchParams>,
    State(backend): State<Arc<Handler>>,
) -> SearchTemplate {
    let (results, errors) = backend.search(
        params.query,
        params.page.unwrap_or(0),
        params.relavancy,
        params.safe_level,
    ).await;

    SearchTemplate{results, errors}
}

pub async fn search_api_handler(
    Query(params): Query<SearchParams>,
    State(backend): State<Arc<Handler>>,
) -> Json<QueryResults> {
    let (results, errors) = backend.search(
        params.query,
        params.page.unwrap_or(0),
        params.relavancy,
        params.safe_level,
    ).await;

    Json(QueryResults{results, errors})
}
