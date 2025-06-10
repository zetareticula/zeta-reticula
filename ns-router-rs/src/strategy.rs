use crate::{ModelConfig, KVCacheConfig, PrecisionLevel};
use rayon::prelude::*;
use crate::symbolic::SymbolicReasoner;

#[derive(Serialize, Deserialize)]
pub enum ExecutionStrategy {
    Local,
    Federated,
    Distributed,
}

pub struct NSStrategySelector {
    reasoner: SymbolicReasoner,
}

impl NSStrategySelector {
    pub fn new() -> Self {
        NSStrategySelector {
            reasoner: SymbolicReasoner::new(),
        }
    }

    pub fn select_strategy(&self, context: &NSContextAnalysis) -> (ModelConfig, ExecutionStrategy, KVCacheConfig, Vec<String>) {
        let symbolic_rules = self.reasoner.apply_constraints(&context.symbolic_constraints, &context.salience_profile);

        let sparsity_threshold = 0.5;
        let priority_tokens: Vec<u32> = context.salience_profile.iter()
            .filter(|r| r.salience_score > sparsity_threshold)
            .map(|r| r.token_id)
            .collect();

        let inactive_neurons: Vec<usize> = context.salience_profile.iter()
            .enumerate()
            .filter(|(_, r)| r.salience_score < 0.3)
            .map(|(i, _)| i)
            .collect();

        // Mock re-ranking feedback
        let re_rank_accuracy_improvement = 0.05; // Example feedback

        rayon::scope(|s| {
            let (model_config, strategy, kv_cache) = rayon::join(
                || Self::choose_model_config(context, &symbolic_rules),
                || Self::choose_execution_strategy(context),
                || Self::choose_kv_cache_config(context, &symbolic_rules, priority_tokens, inactive_neurons, re_rank_accuracy_improvement),
            );

            (model_config, strategy, kv_cache, symbolic_rules)
        })
    }

    fn choose_model_config(context: &NSContextAnalysis, rules: &[String]) -> ModelConfig {
        let size = if context.theory_complexity > 0.7 { 7_000_000_000 } else { 3_000_000_000 };
        let mut precision = context.salience_profile.iter()
            .map(|r| r.precision.clone())
            .collect::<Vec<_>>();

        for (i, result) in context.salience_profile.iter().enumerate() {
            if result.role == "negation" && rules.contains(&"negations require Bit16 precision".to_string()) {
                precision[i] = PrecisionLevel::Bit16;
            }
        }

        ModelConfig { size, precision }
    }

    fn choose_execution_strategy(context: &NSContextAnalysis) -> ExecutionStrategy {
        if context.token_count > 1000 { ExecutionStrategy::Distributed } else { ExecutionStrategy::Local }
    }

    fn choose_kv_cache_config(context: &NSContextAnalysis, rules: &[String], priority_tokens: Vec<u32>, inactive_neurons: Vec<usize>, re_rank_accuracy: f32) -> KVCacheConfig {
        let sparsity = if re_rank_accuracy > 0.01 { 0.3 } else { 0.7 }; // Adjust sparsity based on feedback
        let mut final_priority_tokens = priority_tokens;

        if rules.contains(&"subjects > modifiers in salience".to_string()) {
            final_priority_tokens.extend(context.salience_profile.iter()
                .filter(|r| r.role == "subject")
                .map(|r| r.token_id));
            final_priority_tokens.sort();
            final_priority_tokens.dedup();
        }

        KVCacheConfig {
            sparsity,
            priority_tokens: final_priority_tokens,
            inactive_neurons,
            re_rank_accuracy,
        }
    }
}