use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use lib::{Handler, Relavancy, SafeSearchLevel};
use serde::Deserialize;

use crate::templates::{IndexTemplate, SearchTemplate};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    query: String,
    page: Option<u16>,
    relavancy: Option<Relavancy>,
    safe_level: Option<SafeSearchLevel>,
    json: Option<bool>,
}

pub async fn index_handler() -> IndexTemplate {
    IndexTemplate
}

pub async fn search_handler(
    Query(params): Query<SearchParams>,
    State(backend): State<Arc<Handler>>,
) -> Response {
    let result = backend
        .search(
            params.query,
            params.page.unwrap_or(0),
            params.relavancy,
            params.safe_level,
        )
        .await;

    if params.json.unwrap_or(false) {
        Json(result).into_response()
    } else {
        SearchTemplate::new(result).into_response()
    }
}
