use regex::Regex;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BpeError {
    #[error("BPE not initialized")]
    NotInitialized,
    
    #[error("Invalid merge operation: {0}")]
    InvalidMerge(String),
    
    #[error("Vocabulary not loaded")]
    VocabularyNotLoaded,
}

/// Simplified BPE tokenizer for quantization engine
#[derive(Debug, Clone)]
pub struct BPE {
    vocab: HashMap<String, u32>,
    pattern: Regex,
    special_tokens: HashSet<String>,
}

impl BPE {
    pub fn new() -> Result<Self, BpeError> {
        let pattern = Regex::new(r"'s|'t|'re|'ve|'m|'ll|'d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+\S|\s+")
            .map_err(|e| BpeError::InvalidMerge(e.to_string()))?;
        
        Ok(Self {
            vocab: HashMap::new(),
            pattern,
            special_tokens: HashSet::new(),
        })
    }
    
    pub fn tokenize(&self, text: &str) -> Result<Vec<String>, BpeError> {
        let mut tokens = Vec::new();
        
        for token in self.pattern.find_iter(text) {
            tokens.push(token.as_str().to_string());
        }
        
        if tokens.is_empty() {
            tokens = text.split_whitespace().map(|s| s.to_string()).collect();
        }
        
        Ok(tokens)
    }
}

use crate::error::{QuantizationError, Result};
use std::collections::HashMap;

/// Tokenizer integration for KV cache prefill
pub struct TokenizerIntegration {
    bpe: BPE,
    vocab_cache: HashMap<String, Vec<u32>>,
}

impl TokenizerIntegration {
    pub fn new() -> Result<Self> {
        let bpe = BPE::new()
            .map_err(|e| QuantizationError::config(format!("Failed to initialize BPE: {:?}", e)))?;
        
        Ok(Self {
            bpe,
            vocab_cache: HashMap::new(),
        })
    }

    /// Tokenize text for KV cache prefill
    pub fn tokenize_for_prefill(&mut self, text: &str) -> Result<Vec<String>> {
        if let Some(cached_tokens) = self.vocab_cache.get(text) {
            return Ok(cached_tokens.iter().map(|&id| format!("token_{}", id)).collect());
        }

        let tokens = self.bpe.tokenize(text)
            .map_err(|e| QuantizationError::config(format!("Tokenization failed: {:?}", e)))?;

        // Cache the result
        let token_ids: Vec<u32> = tokens.iter().enumerate().map(|(i, _)| i as u32).collect();
        self.vocab_cache.insert(text.to_string(), token_ids);

        Ok(tokens)
    }

    /// Get token embeddings for KV cache
    pub fn get_token_embeddings(&self, tokens: &[String]) -> Vec<Vec<f32>> {
        tokens.iter().map(|token| {
            // Generate synthetic embeddings based on token hash
            let hash = token.chars().map(|c| c as u32).sum::<u32>();
            (0..768).map(|i| ((hash + i) as f32 / 1000.0).sin()).collect()
        }).collect()
    }
}
