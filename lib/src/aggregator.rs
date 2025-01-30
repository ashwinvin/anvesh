use std::collections::HashMap;
use tracing::instrument;
use url::Url;

use crate::SearchResult;

#[derive(Debug)]
pub struct Aggregator {
    score_multipliers: HashMap<String, f32>,
}

/// Handles the filtering, scoring and sorting of results
///
/// The scores are calculated by summing the scores given by each search engine.
/// The scores given by each engine = position of result from last * score multiplier of search engine
/// The scoring is done on the assumption that results are parsed in the right order
impl Aggregator {
    pub fn new(score_multipliers: HashMap<String, f32>) -> Self {
        Aggregator { score_multipliers }
    }

    /// Deduplicate the search results and rank it based on its position and no of occurences
    #[instrument(level = "TRACE", skip_all)]
    pub fn process(&self, raw_results: Vec<Vec<SearchResult>>) -> Vec<SearchResult> {
        let mut deduped_results: HashMap<Url, SearchResult> = HashMap::new();

        for results in raw_results {
            let total_results = results.len() as f32;

            for (pos, mut result) in results.into_iter().rev().enumerate() {
                match deduped_results.get_mut(&result.url) {
                    Some(existing_result) => {
                        tracing::debug!("Found duplicate result: {}", existing_result.url);

                        existing_result.score +=
                            self.score_result(&result, (pos + 1) as f32, total_results);
                        existing_result.sources.extend(result.sources);
                    }
                    None => {
                        result.score = self.score_result(&result, (pos + 1) as f32, total_results);
                        deduped_results.insert(result.url.clone(), result);
                    }
                };
            }
        }

        let mut agg_results: Vec<SearchResult> = deduped_results.into_values().collect();
        // sort in descending order
        agg_results.sort_by(|b, a| a.score.partial_cmp(&b.score).unwrap());

        agg_results
    }

    #[inline]
    fn score_result(&self, result: &SearchResult, pos: f32, total_results: f32) -> f32 {
        // The search result is guaranteed to have at least one element in the source field.
        let score_multiplier = self
            .score_multipliers
            .get(result.sources.last().unwrap())
            .unwrap_or(&1.0); // This will never panic as engines not configured in config file will be loaded with defaults.
        score_multiplier * (pos / total_results) as f32
    }
}
