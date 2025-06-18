use serde::{Serialize, Deserialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use crate::server::AgentFlowServer;
use rayon::prelude::*;
use futures::future::join_all;
use dashmap::DashMap;


// This module defines the DistributedIO struct which provides parallel read functionality
// for distributed clients in the AgentFlowServer.

#[derive(Serialize, Deserialize)]
pub struct DistributedIO;

impl DistributedIO {
    pub async fn parallel_read(&self, server: &AgentFlowServer, chunk_size: usize) -> Vec<u8> {
        let futures: Vec<_> = server.clients.par_iter()
            .map(|entry| {
                let client = entry.value();
                let path = client.local_data.clone();
                async move {
                    let file = File::open(&path).await.unwrap();
                    let mut reader = BufReader::new(file);
                    let mut buffer = vec![0u8; chunk_size];
                    reader.read_exact(&mut buffer).await.unwrap();
                    buffer
                }
            })
            .collect();

        let results = futures::join_all(futures).await;
        results.into_iter().flatten().collect()
    }
}