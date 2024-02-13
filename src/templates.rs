use askama_axum::Template;
use backend::{SearchResult, errors::EngineError};


#[derive(Template)]
#[template(path="base.html")]
pub struct IndexTemplate;

#[derive(Template)]
#[template(path="search.html")]
pub struct SearchTemplate{
    pub results: Vec<SearchResult>,
    pub errors: Vec<EngineError>
}