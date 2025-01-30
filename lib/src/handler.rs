use crate::{
    engines::{bing::Bing, duckduckgo::DuckDuckGo, Engine},
    errors::EngineError,
    network::NetworkHandler,
    SearchResult, {Relavancy, SafeSearchLevel},
};
use anyhow::Result;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::instrument;

#[derive(Debug)]
pub struct EngineHandler {
    engines: Vec<Arc<Box<dyn Engine>>>,
    query_client: Arc<NetworkHandler>,
}

impl EngineHandler {
    /// Create a new engine handler based on provided list of engines.
    #[instrument(level = "TRACE", skip(network_handler))]
    pub fn new(
        activated_engines: &[String],
        network_handler: NetworkHandler,
    ) -> Result<EngineHandler> {
        let mut engines: Vec<Arc<Box<dyn Engine>>> = vec![];

        for engine in activated_engines {
            // Add new engines here
            if engine.eq_ignore_ascii_case("bing") {
                engines.push(Bing::new())
            } if engine.eq_ignore_ascii_case("duckduckgo") {
                engines.push(DuckDuckGo::new())
            }
        }
        if engines.is_empty(){
            tracing::warn!("No engines were initialised. This might be unintentional, recheck config.")
        }

        tracing::info!("Initialized {} engines", engines.len());

        Ok(EngineHandler {
            engines,
            query_client: Arc::new(network_handler),
        })
    }

    /// Concurrently search the query with all the selected engines.
    /// 
    /// An async task is spun up for every engine and is executed concurrently. The tasks are
    /// waited until the last engine returns.
    #[instrument(level = "TRACE", skip_all)]
    pub async fn search(
        &self,
        query: String,
        page: u16,
        relavancy: Option<Relavancy>,
        safe_level: Option<SafeSearchLevel>,
    ) -> (Vec<Vec<SearchResult>>, Vec<EngineError>) {
        let mut tasks = JoinSet::new();

        for engine in &self.engines {
            let engine = engine.clone();
            let qclient = self.query_client.clone();
            let query = query.clone();

            tasks.spawn(async move {
                engine
                    .search_text(qclient, page, query, relavancy, safe_level)
                    .await
            });
        }

        let mut search_results = Vec::with_capacity(tasks.len());
        let mut engine_errors = Vec::with_capacity(tasks.len());

        while let Some(task_status) = tasks.join_next().await {
            if let Ok(task_result) = task_status {
                match task_result {
                    Ok(results) => search_results.push(results),
                    Err(errors) => engine_errors.push(errors),
                }
            } else {
                // We can't figure out which engine failed as spawning tasks with names is currently unstable.
                // https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html#method.build_task
                tracing::warn!(
                    "An engine has failed to execute due to: \n {}",
                    task_status.err().unwrap()
                );
            }
        }

        (search_results, engine_errors)
    }
}
