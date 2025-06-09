use crate::{NSRoutingPlan, ModelConfig, KVCacheConfig, PrecisionLevel, context::NSContextAnalyzer, strategy::NSStrategySelector};
use salience_engine::quantizer::{SalienceQuantizer, TokenFeatures};
use llm_rs::InferenceEngine;
use log;

pub struct NSRouter {
    analyzer: NSContextAnalyzer,
    selector: NSStrategySelector,
    quantizer: SalienceQuantizer,
}

impl NSRouter {
    pub fn new() -> Self {
        NSRouter {
            analyzer: NSContextAnalyzer::new(),
            selector: NSStrategySelector::new(),
            quantizer: SalienceQuantizer::new(0.7),
        }
    }

    pub fn route_inference(&self, input: &str, user_id: &str) -> Result<NSRoutingPlan, String> {
        log::info!("Routing inference with neurosymbolic capabilities for input: {}", input);

        let token_features: Vec<TokenFeatures> = input.split_whitespace()
            .enumerate()
            .map(|(idx, word)| TokenFeatures {
                token_id: idx as u32,
                frequency: 0.5,
                sentiment_score: 0.0,
                context_relevance: 0.5,
                role: if idx % 2 == 0 { "subject".to_string() } else { "modifier".to_string() },
            })
            .collect();

        let (quantization_results, _tableau) = self.quantizer.quantize_tokens(token_features);
        let context = self.analyzer.analyze(input, quantization_results);

        let (model_config, execution_strategy, kv_cache_config, symbolic_rules) = self.selector.select_strategy(&context);

        let engine = InferenceEngine::new(model_config.size);
        let _output = engine.infer(input, &kv_cache_config);

        Ok(NSRoutingPlan {
            model_config,
            execution_strategy: format!("{:?}", execution_strategy),
            kv_cache_config,
            symbolic_rules,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ns_route_inference() {
        let router = NSRouter::new();
        let plan = router.route_inference("Hello world", "user123").unwrap();
        assert!(plan.model_config.size == 3_000_000_000 || plan.model_config.size == 7_000_000_000);
        assert!(matches!(plan.execution_strategy.as_str(), "Local" | "Distributed"));
        assert!(!plan.kv_cache_config.priority_tokens.is_empty());
        assert!(plan.symbolic_rules.contains(&"subjects > modifiers in salience".to_string()));
    }
}