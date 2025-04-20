use askama_axum::Template;
use lib::{errors::EngineError, SearchResult};

#[derive(Template)]
#[template(path = "base.html")]
pub struct IndexTemplate;

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub errors: Vec<EngineError>,
}

