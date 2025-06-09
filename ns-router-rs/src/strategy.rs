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
        // Apply symbolic constraints
        let symbolic_rules = self.reasoner.apply_constraints(&context.symbolic_constraints, &context.salience_profile);

        rayon::scope(|s| {
            let (model_config, strategy, kv_cache) = rayon::join(
                || Self::choose_model_config(context, &symbolic_rules),
                || Self::choose_execution_strategy(context),
                || Self::choose_kv_cache_config(context, &symbolic_rules),
            );

            (model_config, strategy, kv_cache, symbolic_rules)
        })
    }

    fn choose_model_config(context: &NSContextAnalysis, rules: &[String]) -> ModelConfig {
        let size = if context.theory_complexity > 0.7 { 7_000_000_000 } else { 3_000_000_000 };
        let mut precision = context.salience_profile.iter()
            .map(|r| r.precision.clone())
            .collect::<Vec<_>>();

        // Apply symbolic rule: e.g., "negations require Bit16 precision"
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

    fn choose_kv_cache_config(context: &NSContextAnalysis, rules: &[String]) -> KVCacheConfig {
        let sparsity = if context.theory_complexity < 0.5 { 0.3 } else { 0.7 };
        let mut priority_tokens: Vec<u32> = context.salience_profile.iter()
            .filter(|r| matches!(r.precision, PrecisionLevel::Bit16))
            .map(|r| r.token_id)
            .collect();

        // Apply symbolic rule: prioritize subjects
        if rules.contains(&"subjects > modifiers in salience".to_string()) {
            priority_tokens.extend(context.salience_profile.iter()
                .filter(|r| r.role == "subject")
                .map(|r| r.token_id));
            priority_tokens.sort();
            priority_tokens.dedup();
        }

        KVCacheConfig { sparsity, priority_tokens }
    }
}