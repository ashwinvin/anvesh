use askama_axum::Template;


#[derive(Template)]
#[template(path="base.html")]
pub struct IndexTemplate;

#[derive(Template)]
#[template(path="search.html")]
pub struct SearchTemplate;