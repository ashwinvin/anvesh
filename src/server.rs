use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use lib::Handler;

use crate::templates::{IndexTemplate, SearchTemplate};

pub async fn index_handler() -> IndexTemplate {
    IndexTemplate
}

pub async fn search_handler(
    Query(params): Query<lib::Query>,
    State(backend): State<Arc<Handler>>,
) -> Response {
    let q_text = params.text.clone();

    let result = backend.search(params).await;
    SearchTemplate::new(q_text, result).into_response()
}
