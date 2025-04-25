use std::{collections::HashMap, sync::Arc};

use regex::Regex;

use reqwest::header::HeaderMap;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    errors::EngineErrorType, network::NetworkHandler, Relavancy, SafeSearchLevel, SearchResult,
};

use super::{parse_generic_results, Engine};

const COOKIE_PARAMS: &str =
    "_EDGE_V=1;SRCHD=AF=NOFORM;_Rwho=u=d;bngps=s=0;_UR=QS=0&TQS=0;_UR=QS=0&TQS=0;";
/// Get the actual url of the website by decoding query param.
fn get_og_url(url: &str) -> Result<String, EngineErrorType> {
    Url::parse(url)
        .map_err(|_| EngineErrorType::ParseFailed)?
        .query_pairs()
        .find_map(|(k, v)| {
            k.eq("u").then_some(
                base64_url::decode(v.as_bytes())
                    .ok()
                    .and_then(|val| String::from_utf8(val).ok()),
            )
        })
        .flatten()
        .ok_or(EngineErrorType::ParseFailed)
}
#[derive(Debug)]
pub struct Bing {
    no_results_selector: Selector,
    text_results_selector: Selector,
    text_result_url_selector: Selector,
    text_result_title_selector: Selector,
    text_result_desc_selector: Vec<Selector>,
    re_strong: Regex,
    re_span: Regex,
}

impl Bing {
    pub fn new() -> Arc<Box<dyn Engine>> {
        Arc::new(Box::new(Self {
            no_results_selector: Selector::parse(".b_results").unwrap(),
            text_results_selector: Selector::parse(".b_algo").unwrap(),
            text_result_url_selector: Selector::parse(".b_tpcn a.tilk").unwrap(),
            text_result_title_selector: Selector::parse("h2 a").unwrap(),
            text_result_desc_selector: [
                Selector::parse(".b_caption p").unwrap(),
                Selector::parse(".b_lineclamp3 p").unwrap(),
            ]
            .to_vec(),

            re_span: Regex::new(r#"<span.*?>.*?(?:</span>&nbsp;·|</span>)"#).unwrap(),
            re_strong: Regex::new(r#"(<strong>|</strong>)"#).unwrap(),
        }))
    }
}

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

        if let Some(no_result_msg) = page.select(&self.no_results_selector).nth(0) {
            if no_result_msg
                .value()
                .attr("class")
                .map(|classes| classes.contains("b_algo"))
                .unwrap_or(false)
            {
                return Err(EngineErrorType::NoResults);
            }
        }

        let results = parse_generic_results(&page, &self.text_results_selector, |result| {
            let title = result.select(&self.text_result_title_selector).next();
            let url = result.select(&self.text_result_url_selector).next();
            let desc = &self
                .text_result_desc_selector
                .iter()
                .find_map(|slctr| result.select(slctr).next());

            if let (Some(title), Some(url), Some(desc)) = (title, url, desc) {
                SearchResult::new(
                    &get_og_url(&url.value().attr("href").unwrap()).unwrap(),
                    &self.re_strong.replace_all(title.inner_html().trim(), ""),
                    &self.re_span.replace_all(desc.inner_html().trim(), ""),
                    "Bing",
                )
                .ok()
            } else {
                None
            }
        })
        .map_err(|_| EngineErrorType::ParseFailed)?;

        tracing::trace!("Bing returned {} results.", results.len());
        Ok(results)
    }
}
