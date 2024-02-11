use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{header::HeaderMap, Client, Proxy};

use crate::errors::NetworkError;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36";

pub enum SourceType {
    String,
    Json,
}

#[derive(Debug)]
pub struct NetworkHandler {
    pub client: Client,
}

impl NetworkHandler {
    // TODO: Replace `anyhow::Result` with `Result<NetworkHandler, NetworkError>`
    pub async fn new(
        timeout: u16,
        proxy_url: Option<&str>,
        is_tor: Option<bool>,
    ) -> Result<NetworkHandler> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(timeout.into()));

        // TODO: Clean this up
        let client = if let Some(proxy_url) = proxy_url {
            let proxy = Proxy::all(proxy_url)?;
            let client = client.proxy(proxy);

            let client = client.build().with_context(|| {
                format!("Failed to initialise web query engine. Check for configuration mistakes")
            })?;

            // TODO: Check tor connection every x seconds to ensure integrity.
            if is_tor.unwrap_or(false) {
                if !(NetworkHandler::check_tor(&client).await?) {
                    return Err(NetworkError::ProxyError(proxy_url.to_string()).into());
                }
            }
            client
        } else {
            client.build().with_context(|| {
                format!("Failed to initialise web query engine. Check for configuration mistakes")
            })?
        };

        Ok(NetworkHandler { client })
    }

    /// Sends a request to `https://check.torproject.org/api/ip` to determine whether the connection uses tor.
    async fn check_tor(client: &Client) -> anyhow::Result<bool> {
        let data = client
            .get("https://check.torproject.org/api/ip")
            .send()
            .await?
            .text()
            .await?;

        // this is just convenient.
        Ok(data.contains("\"IsTor\":true"))
    }

    /// Fetches a url with `GET` method.
    ///
    /// The useragent can be overriden by setting the useragent header.
    pub async fn get_data(
        &self,
        url: &str,
        headers: HeaderMap,
        source_type: SourceType,
    ) -> Result<String, NetworkError> {
        let data = self.client.get(url).headers(headers).send().await?;
        let parsed_data = match source_type {
            SourceType::String => data.text().await?,
            SourceType::Json => data.json().await?,
        };

        Ok(parsed_data)
    }
}
