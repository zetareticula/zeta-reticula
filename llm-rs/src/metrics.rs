// Copyright 2025 ZETA RETICULA INC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


//! Metrics collection and reporting for the KV cache

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{error};

#[derive(Debug, Clone)]
pub struct MetricsRecorder {
    namespace: String,
    counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    histograms: Arc<RwLock<HashMap<String, HistogramStats>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
}

#[derive(Debug)]
struct HistogramStats {
    sum: AtomicU64,
    count: AtomicU64,
    min: AtomicU64,
    max: AtomicU64,
    buckets: Vec<(f64, AtomicU64)>,
}

impl MetricsRecorder {
    pub fn new(namespace: &str) -> Self {
        // External metrics exporters are disabled in this build.
        // We keep an internal recorder for lightweight tracking.
        Self {
            namespace: namespace.to_string(),
            counters: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn record_latency(&self, operation: &str, duration: Duration) {
        let name = format!("{}_latency_seconds", operation);
        let micros = duration.as_micros() as f64 / 1_000_000.0;
        
        // Record to local histogram
        self.record_histogram(&name, micros);
    }
    
    pub fn increment_counter(&self, name: &str) {
        let full_name = format!("{}_{}", self.namespace, name);
        
        // Record to local counter
        let counters = self.counters.blocking_read();
        if let Some(counter) = counters.get(name) {
            counter.fetch_add(1, Ordering::Relaxed);
        } else {
            drop(counters);
            let mut counters = self.counters.blocking_write();
            counters.insert(name.to_string(), AtomicU64::new(1));
        }
    }
    
    pub fn record_histogram(&self, name: &str, value: f64) {
        let full_name = format!("{}_{}", self.namespace, name);
        
        // Record to local histogram
        let histograms = self.histograms.blocking_read();
        if let Some(hist) = histograms.get(name) {
            let value_micros = (value * 1_000_000.0) as u64; // Convert to microseconds for storage
            hist.record(value_micros);
        } else {
            drop(histograms);
            let mut histograms = self.histograms.blocking_write();
            let hist = HistogramStats::new();
            hist.record((value * 1_000_000.0) as u64);
            histograms.insert(name.to_string(), hist);
        }
    }
    
    pub fn set_gauge(&self, name: &str, value: f64) {
        let full_name = format!("{}_{}", self.namespace, name);
        
        // Record to local gauge
        let mut gauges = self.gauges.blocking_write();
        gauges.insert(name.to_string(), value);
    }
    
    pub fn get_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        // Collect counter metrics
        let counters = self.counters.blocking_read();
        for (name, counter) in counters.iter() {
            metrics.insert(format!("counter_{}", name), counter.load(Ordering::Relaxed) as f64);
        }
        
        // Collect histogram metrics
        let histograms = self.histograms.blocking_read();
        for (name, hist) in histograms.iter() {
            let stats = hist.get_stats();
            metrics.insert(format!("histogram_{}_count", name), stats.count as f64);
            metrics.insert(format!("histogram_{}_sum", name), stats.sum as f64 / 1_000_000.0); // Convert back to seconds
            metrics.insert(format!("histogram_{}_min", name), stats.min as f64 / 1_000_000.0);
            metrics.insert(format!("histogram_{}_max", name), stats.max as f64 / 1_000_000.0);
            
            // Add percentiles
            for (le, count) in &stats.buckets {
                metrics.insert(
                    format!("histogram_{}_bucket_le_{}", name, le),
                    *count as f64,
                );
            }
        }
        
        // Collect gauge metrics
        let gauges = self.gauges.blocking_read();
        for (name, value) in gauges.iter() {
            metrics.insert(format!("gauge_{}", name), *value);
        }
        
        metrics
    }
}

impl HistogramStats {
    fn new() -> Self {
        // Define bucket boundaries in microseconds
        let buckets = vec![
            1_000,      // 1ms
            5_000,      // 5ms
            10_000,     // 10ms
            25_000,     // 25ms
            50_000,     // 50ms
            100_000,    // 100ms
            250_000,    // 250ms
            500_000,    // 500ms
            1_000_000,  // 1s
            2_500_000,  // 2.5s
            5_000_000,  // 5s
            10_000_000, // 10s
        ];
        
        Self {
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
            buckets: buckets.into_iter()
                .map(|le| (le as f64 / 1_000_000.0, AtomicU64::new(0))) // Convert to seconds for the label
                .collect(),
        }
    }
    
    fn record(&self, value_micros: u64) {
        // Update sum and count
        self.sum.fetch_add(value_micros, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
        
        // Update min and max
        let mut current_min = self.min.load(Ordering::Relaxed);
        while value_micros < current_min {
            match self.min.compare_exchange_weak(
                current_min,
                value_micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(min) => current_min = min,
            }
        }
        
        let mut current_max = self.max.load(Ordering::Relaxed);
        while value_micros > current_max {
            match self.max.compare_exchange_weak(
                current_max,
                value_micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(max) => current_max = max,
            }
        }
        
        // Update buckets
        for (le, count) in &self.buckets {
            let le_micros = (le * 1_000_000.0) as u64;
            if value_micros <= le_micros {
                count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    
    fn get_stats(&self) -> HistogramStatsSnapshot {
        let sum = self.sum.load(Ordering::Relaxed);
        let count = self.count.load(Ordering::Relaxed);
        let min = self.min.load(Ordering::Relaxed);
        let max = self.max.load(Ordering::Relaxed);
        
        let buckets = self.buckets
            .iter()
            .map(|(le, count)| (*le, count.load(Ordering::Relaxed)))
            .collect();
        
        HistogramStatsSnapshot {
            sum,
            count,
            min: if count > 0 { min } else { 0 },
            max,
            buckets,
        }
    }
}

#[derive(Debug, Clone)]
struct HistogramStatsSnapshot {
    sum: u64,
    count: u64,
    min: u64,
    max: u64,
    buckets: Vec<(f64, u64)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_metrics_recorder() {
        let recorder = MetricsRecorder::new("test");
        
        // Test counter
        recorder.increment_counter("test_counter");
        recorder.increment_counter("test_counter");
        
        // Test histogram
        recorder.record_latency("test_op", Duration::from_millis(100));
        recorder.record_latency("test_op", Duration::from_millis(200));
        
        // Test gauge
        recorder.set_gauge("test_gauge", 42.0);
        
        // Verify metrics
        let metrics = recorder.get_metrics();
        
        assert_eq!(metrics.get("counter_test_counter"), Some(&2.0));
        assert_eq!(metrics.get("gauge_test_gauge"), Some(&42.0));
        
        // Check histogram stats
        assert_eq!(metrics.get("histogram_test_op_latency_seconds_count"), Some(&2.0));
        assert!(metrics.get("histogram_test_op_latency_seconds_sum").unwrap() > &0.0);
        assert!(metrics.get("histogram_test_op_latency_seconds_min").unwrap() >= &0.1);
        assert!(metrics.get("histogram_test_op_latency_seconds_max").unwrap() <= &0.3);
    }
    
    #[test]
    fn test_histogram_stats() {
        let hist = HistogramStats::new();
        
        // Record some values
        hist.record(500_000);  // 0.5s
        hist.record(1_000_000); // 1.0s
        hist.record(1_500_000); // 1.5s
        
        let stats = hist.get_stats();
        
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 3_000_000);
        assert_eq!(stats.min, 500_000);
        assert_eq!(stats.max, 1_500_000);
        
        // Check bucket counts
        let bucket_1s = stats.buckets.iter().find(|(le, _)| *le == 1.0).unwrap();
        assert_eq!(bucket_1s.1, 2);  // 2 values <= 1s (0.5s and 1.0s)
        
        let bucket_2s = stats.buckets.iter().find(|(le, _)| *le == 2.0).unwrap();
        assert_eq!(bucket_2s.1, 3);  // All 3 values <= 2s
    }
}
