use egg::{EGraph, Runner, SymbolLang, Pattern, Searcher, Applier, Id, Rewrite};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::collections::HashMap;
use crate::rewrite_wrapper::{SerializableRewrite, RewriteError};

/// Errors that can occur during symbolic reasoning
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum SymbolicError {
    #[error("Failed to parse constraint: {0}")]
    ParseError(String),
    
    #[error("Failed to apply rewrite rules: {0}")]
    RewriteError(String),
    
    #[error("Invalid symbolic expression: {0}")]
    InvalidExpression(String),
}

/// A symbolic reasoner that uses e-graphs for rewriting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolicReasoner {
    #[serde(skip_serializing, skip_deserializing)]
    egraph: EGraph<SymbolLang, ()>,
    rules: HashMap<String, SerializableRewrite>,
    next_var_id: usize,
}

impl SymbolicReasoner {
    /// Create a new SymbolicReasoner with default rules
    pub fn new() -> Self {
        let mut reasoner = SymbolicReasoner {
            egraph: EGraph::new(()),
            rules: HashMap::new(),
            next_var_id: 0,
        };
        
        // Add some default rewrite rules
        reasoner.add_rule("commutativity", "(+ ?a ?b)", "(+ ?b ?a)").unwrap();
        reasoner.add_rule("associativity", "(+ (+ ?a ?b) ?c)", "(+ ?a (+ ?b ?c))").unwrap();
        reasoner.add_rule("identity", "(+ ?a 0)", "?a").unwrap();
        
        reasoner
    }
    
    /// Add a new rewrite rule to the reasoner
    pub fn add_rule(&mut self, name: &str, left: &str, right: &str) -> Result<(), SymbolicError> {
        let rule = SerializableRewrite::new(name, left, right)
            .map_err(|e| SymbolicError::ParseError(e.to_string()))?;
        
        self.rules.insert(name.to_string(), rule);
        Ok(())
    }
    
    /// Apply constraints and return derived facts
    pub fn apply_constraints(
        &mut self, 
        constraints: &[String],
        salience_profile: &[QuantizationResult]
    ) -> Result<Vec<String>, SymbolicError> {
        // Clear previous state
        self.egraph = EGraph::new(());
        
        // Parse constraints into the e-graph
        for constraint in constraints {
            let expr: Pattern<SymbolLang> = constraint.parse()
                .map_err(|e| SymbolicError::ParseError(e.to_string()))?;
            
            // Add the constraint to the e-graph
            let id = self.egraph.add_expr(&expr.ast);
            self.egraph.union(id, id); // Ensure the constraint is in its own equivalence class
        }
        
        // Prepare rewrite rules
        let rules: Result<Vec<Rewrite<SymbolLang, ()>>, _> = self.rules.values_mut()
            .map(|r| r.get_rewrite()
                .map_err(|e| SymbolicError::RewriteError(e.to_string()))
                .map(|r| r.clone())
            )
            .collect();
            
        let rules = rules?;
        
        // Apply rewrite rules
        let runner = Runner::default()
            .with_egraph(self.egraph.clone())
            .run(&rules);
            
        // Extract results
        let mut results = Vec::new();
        for class in runner.egraph.classes() {
            results.push(class.nodes[0].to_string());
        }
        
        // Filter based on salience profile
        self.filter_by_salience(&mut results, salience_profile);
        
        Ok(results)
    }
    
    /// Filter results based on salience profile
    fn filter_by_salience(&self, results: &mut Vec<String>, salience_profile: &[QuantizationResult]) {
        if salience_profile.is_empty() {
            return;
        }
        
        // Simple implementation: just limit the number of results based on salience
        let avg_salience: f32 = salience_profile.iter()
            .map(|r| r.salience_score)
            .sum::<f32>() / salience_profile.len() as f32;
            
        let max_results = (avg_salience * 10.0) as usize;
        if max_results < results.len() {
            results.truncate(max_results);
        }
    }
    
    /// Generate a new unique variable name
    fn new_var(&mut self) -> String {
        let var = format!("?v{}", self.next_var_id);
        self.next_var_id += 1;
        var
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::QuantizationResult;
    
    #[test]
    fn test_basic_reasoning() {
        let mut reasoner = SymbolicReasoner::new();
        let constraints = vec![
            "(+ a b)".to_string(),
            "(+ b c)".to_string(),
        ];
        
        let results = reasoner.apply_constraints(&constraints, &[]).unwrap();
        assert!(results.contains(&"(+ b a)".to_string()));
        assert!(results.contains(&"(+ c b)".to_string()));
    }
    
    #[test]
    fn test_salience_filtering() {
        let mut reasoner = SymbolicReasoner::new();
        let constraints = (0..20)
            .map(|i| format!("expr_{}", i))
            .collect::<Vec<_>>();
            
        let salience = vec![QuantizationResult {
            salience_score: 0.5,
            ..Default::default()
        }];
        
        let results = reasoner.apply_constraints(&constraints, &salience).unwrap();
        assert!(results.len() <= 5); // 0.5 * 10 = 5 max results
    }
    
    #[test]
    fn test_invalid_constraint() {
        let mut reasoner = SymbolicReasoner::new();
        let result = reasoner.apply_constraints(&["invalid ( expression".to_string()], &[]);
        assert!(matches!(result, Err(SymbolicError::ParseError(_))));
    }
}