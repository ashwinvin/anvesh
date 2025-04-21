use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[error("`engine` failed to fetch results")]
pub struct EngineError {
    pub engine: String,
    pub source: EngineErrorType,
}

#[derive(Debug, Error, Serialize)]
pub enum EngineErrorType {
    #[error("Failed to parse")]
    ParseFailed,
    #[error("Ratelimited by upstream engine")]
    Ratelimited,
    #[error("No results returned for query")]
    NoResults,
    #[error("Failed to spawn search task")]
    ExecFailed,
    #[error("Unknown error occured: {0}")]
    Unknown(String),
    #[error("Network error occured")]
    Network(#[from] NetworkError),
}

#[derive(Debug, Error, Serialize)]
pub enum NetworkError {
    /// Raised when proxy is misconfigured or connection to proxy has been broken
    #[error("Could not connect to proxy: {0}")]
    ProxyError(String),
    /// Raised when connection to upstream search engine timesout.
    #[error("Request to {0} has timed out.")]
    ConnectionTimeout(String),
    /// Raised when there are problems while parsing or something else happens.
    #[error("Unknown error occured: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for NetworkError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_timeout() {
            // All the domains we request will have a host and we really shouldn't be seeing the query due to privacy
            // concerns
            NetworkError::ConnectionTimeout(value.url().unwrap().host_str().unwrap().to_string())
        } else {
            // These types of error should occur very less.
            NetworkError::Unknown(value.to_string())
        }
    }
}
