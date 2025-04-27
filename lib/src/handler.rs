use crate::{
    engines::{get_engines, Engine},
    errors::{EngineError, EngineErrorType},
    network::NetworkHandler,
    Query, SearchResult,
};

use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use tokio::task::{Id, JoinSet};
use tracing::instrument;

static ENGINES: OnceLock<Vec<Box<dyn Engine>>> = OnceLock::new();

#[derive(Debug)]
pub struct EngineHandler {
    query_client: Arc<NetworkHandler>,
}

impl EngineHandler {
    /// Create a new engine handler based on provided list of engines.
    pub fn new(engines: &[String], network_handler: NetworkHandler) -> Result<EngineHandler> {
        let activated_engines: Vec<Box<dyn Engine>> = get_engines()
            .into_iter()
            .filter(|engine| engines.contains(&engine.get_name()))
            .collect();

        if activated_engines.is_empty() {
            tracing::warn!(
                "No engines were initialised. This might be unintentional, recheck config."
            )
        }

        tracing::info!(
            "Initialized {} engines: {}",
            activated_engines.len(),
            activated_engines
                .iter()
                .map(|e| e.get_name())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let _ = ENGINES.set(activated_engines);

        Ok(EngineHandler {
            query_client: Arc::new(network_handler),
        })
    }

    /// Concurrently search the query with all the selected engines.
    ///
    /// An async task is spun up for every engine and is executed concurrently. The tasks are
    /// waited until the last engine returns.
    #[instrument(level = "TRACE", skip_all)]
    pub async fn search(&self, query: Query) -> (Vec<Vec<SearchResult>>, Vec<EngineError>) {
        let mut tasks = JoinSet::new();
        let mut task_ids: HashMap<Id, String> = HashMap::new();

        for engine in ENGINES.get().unwrap().iter() {
            let engine_name = engine.get_name();
            let qclient = self.query_client.clone();
            let query = query.clone();

            let handle = tasks.spawn(async move { engine.search(qclient, query).await });
            task_ids.insert(handle.id(), engine_name);
        }

        let mut search_results = Vec::with_capacity(tasks.len());
        let mut engine_errors = Vec::with_capacity(tasks.len());

        while let Some(task_status) = tasks.join_next_with_id().await {
            if let Ok((id, task_result)) = task_status {
                let engine = task_ids.get(&id).unwrap();
                match task_result {
                    Ok(results) => {
                        if results.is_empty() {
                            engine_errors.push(EngineError {
                                engine: engine.to_string(),
                                source: EngineErrorType::NoResults,
                            });

                            tracing::warn!(
                                "{} has returned 0 results but did not trigger no results page. This engine could \
                                possibly be broken", 
                                engine);
                            continue;
                        }
                        search_results.push(results)
                    }
                    Err(error) => engine_errors.push(EngineError {
                        engine: engine.to_string(),
                        source: error,
                    }),
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
