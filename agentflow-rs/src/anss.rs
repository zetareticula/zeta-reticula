use serde::{Serialize, Deserialize};
use ndarray::Array1;
use rayon::prelude::*;
use crate::server::AgentFlowServer;

#[derive(Serialize, Deserialize)]
pub struct FederatedANSS;

impl FederatedANSS {
    pub async fn search(&self, server: &AgentFlowServer, query: &Array1<f32>, top_m: usize) -> Vec<u32> {
        let results: Vec<_> = server.clients.par_iter()
            .map(|entry| {
                let client = entry.value();
                client.fusion_anns.collaborative_filter(query, top_m / server.clients.len())
            })
            .collect();

        let mut candidates: Vec<u32> = results.into_iter().flatten().collect();
        candidates.sort();
        candidates.dedup();
        candidates.truncate(top_m);

        let ranked: Vec<_> = server.clients.par_iter()
            .map(|entry| {
                let client = entry.value();
                rt.block_on(client.fusion_anns.heuristic_rerank(query, candidates.clone())).unwrap()
            })
            .collect();

        ranked.into_iter().flatten().collect()
    }
}