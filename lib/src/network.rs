use std::time::Duration;

use crate::errors::NetworkError;
use anyhow::{Context, Result};
use reqwest::{
    header::{HeaderMap, USER_AGENT},
    Client, Proxy,
};

#[derive(Debug)]
pub struct NetworkHandler {
    pub client: Client,
    user_agents: Vec<String>,
}

impl NetworkHandler {
    // TODO: Replace `anyhow::Result` with `Result<NetworkHandler, NetworkError>`
    pub async fn new(
        timeout: u16,
        proxy_url: Option<&str>,
        is_tor: Option<bool>,
        user_agents: Vec<String>,
    ) -> Result<NetworkHandler> {
        let client = Client::builder().connect_timeout(Duration::from_secs(timeout.into()));

        // TODO: Clean this up
        let client = if let Some(proxy_url) = proxy_url {
            let proxy = Proxy::all(proxy_url)?;
            let client = client.proxy(proxy);

            let client = client.build().with_context(|| {
                format!("Failed to initialise web query engine. Check for configuration mistakes")
            })?;

            // TODO: Check tor connection every x seconds to ensure integrity.
            if is_tor.unwrap_or(false) && !(NetworkHandler::check_tor(&client).await?) {
                return Err(NetworkError::ProxyError(proxy_url.to_string()).into());
            }
            client
        } else {
            client.build().with_context(|| {
                format!("Failed to initialise web query engine. Check for configuration mistakes")
            })?
        };

        Ok(NetworkHandler {
            client,
            user_agents,
        })
    }

    /// Sends a request to `https://check.torproject.org/api/ip` to determine whether the connection uses tor.
    async fn check_tor(client: &Client) -> anyhow::Result<bool> {
        let data = client
            .get("https://check.torproject.org/api/ip")
            .send()
            .await?
            .text()
            .await?;

        Ok(data.contains("\"IsTor\":true"))
    }

    /// Fetches a url with `GET` method.
    ///
    /// The useragent can be overriden by setting the useragent header.
    pub async fn get_data(
        &self,
        url: &str,
        mut headers: HeaderMap,
        is_json: bool,
    ) -> Result<String, NetworkError> {
        let user_agent = &self.user_agents[fastrand::usize(..self.user_agents.len())];
        headers.insert(USER_AGENT, user_agent.parse().unwrap());

        let data = self.client.get(url).headers(headers).send().await?;

        tracing::trace!("Request to {url} returned {}", data.status());

        if is_json {
            Ok(data.json().await?)
        } else {
            Ok(data.text().await?)
        }
    }
}
