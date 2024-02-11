use std::{collections::HashMap, sync::Arc};

use reqwest::header::HeaderMap;
use scraper::{Html, Selector};

use crate::{
    errors::EngineError, network::NetworkHandler, Relavancy, SafeSearchLevel, SearchResult,
};

use super::{parse_generic_results, Engine};

#[derive(Debug)]
pub struct DuckDuckGo {
    no_results_selector: Selector,
    text_results_selector: Selector,
    text_result_url_selector: Selector,
    text_result_title_selector: Selector,
    text_result_desc_selector: Selector,
}

impl Default for DuckDuckGo {
    fn default() -> Self {
        Self {
            no_results_selector: Selector::parse(".no-results").unwrap(),
            text_results_selector: Selector::parse(".results>.result").unwrap(),
            text_result_url_selector: Selector::parse(".result__url").unwrap(),
            text_result_title_selector: Selector::parse(".result__title>.result__a").unwrap(),
            text_result_desc_selector: Selector::parse(".result__snippet").unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl Engine for DuckDuckGo {
    async fn search_text(
        &self,
        qclient: Arc<NetworkHandler>,
        page_idx: u16,
        query: String,
        _relavancy: Option<Relavancy>,
        _safe_level: Option<SafeSearchLevel>,
    ) -> Result<Vec<SearchResult>, EngineError> {

        let url: String = match page_idx {
            0 => {
                format!("https://html.duckduckgo.com/html/?q={query}&s=&dc=&v=1&o=json&api=/d.js")
            }
            _ => {
                format!(
                    "https://duckduckgo.com/html/?q={query}&s={}&dc={}&v=1&o=json&api=/d.js",
                    page_idx * 30,
                    page_idx * 30 + 1
                )
            }
        };

        let headers = HeaderMap::try_from(&HashMap::from([
            ("REFERER".to_string(), "https://google.com/".to_string()),
            (
                "CONTENT_TYPE".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
            ("COOKIE".to_string(), "kl=wt-wt".to_string()),
        ]))
        .unwrap();

        let page = qclient
            .get_data(&url, headers, crate::network::SourceType::String)
            .await?;

        let page = Html::parse_document(&page);

        if let Some(no_result_msg) = page.select(&self.no_results_selector).nth(0) {
            if no_result_msg
                .value()
                .attr("class")
                .map(|classes| classes.contains("b_algo"))
                .unwrap_or(false)
            {
                return Err(EngineError::NoResults);
            }
        }

        let results = parse_generic_results(&page, &self.text_results_selector, |result| {
            let title = result.select(&self.text_result_title_selector).next();
            let url = result.select(&self.text_result_url_selector).next();
            let desc = result.select(&self.text_result_desc_selector).next();

            if let (Some(title), Some(url), Some(desc)) = (title, url, desc) {
                Some(SearchResult::new(
                    &format!("https://{}", url.inner_html().trim()),
                    title.inner_html().trim(),
                    desc.inner_html().trim(),
                    "DuckDuckGo",
                ))
            } else {
                None
            }
        })
        .map_err(|_| EngineError::ParseFailed)?;

        Ok(results)
    }
}
