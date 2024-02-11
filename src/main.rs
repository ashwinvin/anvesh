pub mod cli;
pub mod config;
pub mod server;
pub mod templates;

use std::sync::Arc;

use backend::Handler;

use clap::Parser;

use axum::{routing::get, Router};
use config::parse_config;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter, FmtSubscriber};

use crate::server::{index_handler, search_handler};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let parsed_args = cli::Cli::parse();
    let pconfig = parse_config(parsed_args.config_path).unwrap();

    let log_level = match pconfig.log_level {
        0 => LevelFilter::TRACE,
        1 => LevelFilter::DEBUG,
        2 => LevelFilter::INFO,
        3 => LevelFilter::WARN,
        4 => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };
    let logger = FmtSubscriber::builder()
        .with_env_filter(&format!("hyper=warn,backend={0},anvesh={0}", log_level))
        // .with_max_level(log_level)
        .with_span_events(FmtSpan::FULL)
        .finish();

    tracing::subscriber::set_global_default(logger).expect("Setting default logger failed");

    // TODO: THIS IS UGLY!! CLEANUP CONFIG INITIALISATION
    let score_multiplers = pconfig
        .upstream_search_engines
        .iter()
        .map(|(key, conf)| (key.clone(), conf.score_multiplier))
        .collect();

    let engines = pconfig
        .upstream_search_engines
        .keys()
        .map(|s| s.clone())
        .collect::<Vec<String>>();

    let backend_handler = match pconfig.proxy {
        Some(ref proxy) => {
            Handler::new(
                score_multiplers,
                pconfig.request_timeout,
                Some(&proxy.connection_url),
                Some(proxy.is_tor),
                &engines,
            )
            .await?
        }
        None => {
            Handler::new(
                score_multiplers,
                pconfig.request_timeout,
                None,
                None,
                &engines,
            )
            .await?
        }
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/search", get(search_handler))
        .with_state(Arc::new(backend_handler));
    let listener = tokio::net::TcpListener::bind((pconfig.bind_ip.clone(), pconfig.port))
        .await
        .unwrap();

    tracing::info!(
        "Web server started at interface {}:{}",
        pconfig.bind_ip,
        pconfig.port
    );
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
