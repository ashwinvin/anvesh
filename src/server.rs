use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use lib::{errors::EngineError, Handler, Relavancy, SafeSearchLevel, SearchResult};
use serde::{Deserialize, Serialize};

use crate::templates::{IndexTemplate, SearchTemplate};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    query: String,
    page: Option<u16>,
    relavancy: Option<Relavancy>,
    safe_level: Option<SafeSearchLevel>,
    json: Option<bool>,
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
) -> Response {
    let (results, errors) = backend
        .search(
            params.query.clone(),
            params.page.unwrap_or(0),
            params.relavancy,
            params.safe_level,
        )
        .await;
    match params.json {
        Some(val) => {
            if val {
                return Json(QueryResults { results, errors }).into_response();
            }
            return SearchTemplate {
                results,
                errors,
                query: params.query,
            }
            .into_response();
        }
        None => {
            return SearchTemplate {
                results,
                errors,
                query: params.query,
            }
            .into_response()
        }
    }
}
