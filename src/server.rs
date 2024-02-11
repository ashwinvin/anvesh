use std::sync::Arc;

use axum::{extract::{Query, State}, Json};
use backend::{Handler, Relavancy, SafeSearchLevel};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::templates::{IndexTemplate, SearchTemplate};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    query: String,
    page: Option<u16>,
    relavancy: Option<Relavancy>,
    safe_level: Option<SafeSearchLevel>,
}

pub async fn index_handler() -> IndexTemplate {
    IndexTemplate
}

pub async fn search_handler(
    Query(params): Query<SearchParams>,
    State(backend): State<Arc<Handler>>,
) -> Json<Value> {
    let results = backend.search(
        params.query,
        params.page.unwrap_or(1),
        params.relavancy,
        params.safe_level,
    ).await;

    Json(json!(results))
}
