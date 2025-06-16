use dashmap::DashMap;
use rayon::prelude::*;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use rand::Rng;
use rand_distr::{Distribution, Normal};

// SalienceOptimizer optimizes the computation of salience scores for tokens
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]  
#[serde(deny_unknown_fields)]
#[derive(Debug, Clone)]


pub struct SalienceOptimizer {
    cache: Arc<DashMap<u32, f32>>, // Cache token salience scores
}

impl SalienceOptimizer {
    pub fn new() -> Self {
        SalienceOptimizer {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn optimize_computation<T, F>(&self, items: Vec<T>, compute: F) -> Vec<(T, f32)>
    where
        F: Fn(&T) -> (u32, f32) + Send + Sync,
        T: Send + Sync,
    {
        items.into_par_iter().map(|item| {
            let (id, salience) = compute(&item);
            if let Some(cached) = self.cache.get(&id) {
                (item, *cached)
            } else {
                self.cache.insert(id, salience);
                (item, salience)
            }
        }).collect()
    }
}