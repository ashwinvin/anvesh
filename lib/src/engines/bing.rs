use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use regex::Regex;

use reqwest::header::HeaderMap;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    errors::EngineErrorType, network::NetworkHandler, Relavancy, SafeSearchLevel, SearchResult,
};

use super::Engine;

const COOKIE_PARAMS: &str =
    "_EDGE_V=1;SRCHD=AF=NOFORM;_Rwho=u=d;bngps=s=0;_UR=QS=0&TQS=0;_UR=QS=0&TQS=0;";

static NO_RESULTS_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".b_results").unwrap());
static TEXT_RESULTS_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".b_algo").unwrap());
static TEXT_RESULT_URL_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".b_tpcn a.tilk").unwrap());
static TEXT_RESULT_TITLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("h2 a").unwrap());
static TEXT_RESULT_DESC_SELECTOR: LazyLock<Vec<Selector>> = LazyLock::new(|| {
    [
        Selector::parse(".b_caption p").unwrap(),
        Selector::parse(".b_lineclamp3 p").unwrap(),
    ]
    .to_vec()
});

static RE_SPAN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<span.*?>.*?(?:</span>&nbsp;·|</span>)"#).unwrap());
static RE_STRONG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(<strong>|</strong>)"#).unwrap());

/// Get the actual url of the website by decoding query param.
fn get_og_url(url: &str) -> Result<String, EngineErrorType> {
    Url::parse(url)
        .map_err(|_| EngineErrorType::ParseFailed)?
        .query_pairs()
        .find_map(|(k, v)| {
            k.eq("u").then_some(
                base64_url::decode(v.trim_start_matches("a1").as_bytes())
                    .ok()
                    .and_then(|val| String::from_utf8(val).ok()),
            )
        })
        .flatten()
        .ok_or(EngineErrorType::ParseFailed)
}

#[derive(Debug)]
pub struct Bing;

#[async_trait::async_trait]
impl Engine for Bing {
    fn get_name(&self) -> String {
        "Bing".to_string()
    }
    async fn search_text(
        &self,
        qclient: Arc<NetworkHandler>,
        page_idx: u16,
        query: String,
        _relavancy: Option<Relavancy>,
        _safe_level: Option<SafeSearchLevel>,
    ) -> Result<Vec<SearchResult>, EngineErrorType> {
        let cont_result = 10 * page_idx + 1;

        let url = match page_idx {
            0 => format!("https://www.bing.com/search?q={query}"),
            _ => format!("https://www.bing.com/search?q={query}&first={cont_result}"),
        };

        let headers = HeaderMap::try_from(&HashMap::from([
            ("REFERER".to_string(), "https://google.com/".to_string()),
            (
                "CONTENT_TYPE".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
            ("COOKIE".to_string(), COOKIE_PARAMS.to_string()),
        ]))
        .unwrap();

        let page = qclient.get_data(&url, headers, false).await?;
        let page = Html::parse_document(&page);

        if let Some(no_result_msg) = page.select(&NO_RESULTS_SELECTOR).nth(0) {
            if no_result_msg
                .value()
                .attr("class")
                .map(|classes| classes.contains("b_algo"))
                .unwrap_or(false)
            {
                return Err(EngineErrorType::NoResults);
            }
        }

        let mut parsed_results: Vec<SearchResult> = Vec::new();

        for result in page.select(&TEXT_RESULTS_SELECTOR) {
            let title = result.select(&TEXT_RESULT_TITLE_SELECTOR).next();
            let url = result.select(&TEXT_RESULT_URL_SELECTOR).next();
            let desc = &TEXT_RESULT_DESC_SELECTOR
                .iter()
                .find_map(|slctr| result.select(slctr).next());

            if let (Some(title), Some(url), Some(desc)) = (title, url, desc) {
                let p_url = if let Some(url) = url.value().attr("href") {
                    if url.starts_with("https://www.bing.com/ck/a?") {
                        &get_og_url(url).unwrap()
                    } else {
                        url
                    }
                } else {
                    tracing::warn!("Could not parse search result url!");
                    continue;
                };

                parsed_results.push(SearchResult::new(
                    &p_url,
                    &RE_STRONG.replace_all(title.inner_html().trim(), ""),
                    &RE_SPAN.replace_all(desc.inner_html().trim(), ""),
                    "Bing",
                )?);
            } else {
                continue;
            }
        }

        tracing::trace!("Bing returned {} results.", parsed_results.len());
        Ok(parsed_results)
    }
}
