//! Tokenizer implementation with BPE (Byte Pair Encoding) support

mod bpe;
mod normalizer;
mod token;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;
use regex::Regex;
use serde::{Serialize, Deserialize};

pub use token::Token;
pub use normalizer::Normalizer;
pub use bpe::BPE;

#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Tokenizer not initialized")]
    NotInitialized,
    
    #[error("Invalid vocabulary format")]
    InvalidVocabulary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizerConfig {
    pub vocab_size: usize,
    pub max_length: usize,
    pub pad_token: String,
    pub unk_token: String,
    pub bos_token: String,
    pub eos_token: String,
    pub mask_token: String,
    pub cls_token: String,
    pub sep_token: String,
    pub padding_side: String,
    pub truncation_side: String,
    pub model_max_length: usize,
    pub special_tokens: Vec<String>,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            vocab_size: 50_257,  // GPT-2 vocabulary size
            max_length: 1024,
            pad_token: "[PAD]".into(),
            unk_token: "[UNK]".into(),
            bos_token: "[BOS]".into(),
            eos_token: "[EOS]".into(),
            mask_token: "[MASK]".into(),
            cls_token: "[CLS]".into(),
            sep_token: "[SEP]".into(),
            padding_side: "right".into(),
            truncation_side: "right".into(),
            model_max_length: 1024,
            special_tokens: vec![
                "[PAD]".into(),
                "[UNK]".into(),
                "[BOS]".into(),
                "[EOS]".into(),
                "[MASK]".into(),
                "[CLS]".into(),
                "[SEP]".into(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tokenizer {
    bpe: BPE,
    normalizer: Normalizer,
    config: TokenizerConfig,
    vocab: HashMap<String, u32>,
    inv_vocab: HashMap<u32, String>,
    special_tokens: HashMap<String, u32>,
    special_tokens_regex: Option<Regex>,
}

impl Tokenizer {
    pub fn new(config: TokenizerConfig) -> Result<Self, TokenizerError> {
        let normalizer = Normalizer::new();
        let bpe = BPE::new()?;
        
        let mut tokenizer = Self {
            bpe,
            normalizer,
            config,
            vocab: HashMap::new(),
            inv_vocab: HashMap::new(),
            special_tokens: HashMap::new(),
            special_tokens_regex: None,
        };
        
        tokenizer.initialize_vocab()?;
        tokenizer.initialize_special_tokens()?;
        
        Ok(tokenizer)
    }
    
    fn initialize_vocab(&mut self) -> Result<(), TokenizerError> {
        // Initialize with byte-level tokens
        for i in 0..=255 {
            let token = format!("<0x{:02X}>", i);
            self.vocab.insert(token.clone(), i as u32);
            self.inv_vocab.insert(i as u32, token);
        }
        
        // Add special tokens to vocab
        for (i, token) in self.config.special_tokens.iter().enumerate() {
            let token_id = self.vocab.len() as u32;
            self.vocab.insert(token.clone(), token_id);
            self.inv_vocab.insert(token_id, token.clone());
        }
        
        Ok(())
    }
    
    fn initialize_special_tokens(&mut self) -> Result<(), TokenizerError> {
        // Create special tokens mapping
        for (i, token) in self.config.special_tokens.iter().enumerate() {
            if let Some(&token_id) = self.vocab.get(token) {
                self.special_tokens.insert(token.clone(), token_id);
            }
        }
        
        // Create regex for special tokens
        let pattern = self.config.special_tokens
            .iter()
            .map(|t| regex::escape(t))
            .collect::<Vec<_>>()
            .join("|");
            
        self.special_tokens_regex = Regex::new(&format!("({})", pattern)).ok();
        
        Ok(())
    }
    
    pub fn encode(&self, text: &str) -> Result<Vec<u32>, TokenizerError> {
        let normalized = self.normalizer.normalize(text);
        let tokens = self.bpe.tokenize(&normalized)?;
        
        let mut token_ids = Vec::with_capacity(tokens.len());
        for token in tokens {
            if let Some(&token_id) = self.vocab.get(&token) {
                token_ids.push(token_id);
            } else {
                // Handle unknown tokens
                if let Some(unk_id) = self.special_tokens.get(&self.config.unk_token) {
                    token_ids.push(*unk_id);
                } else {
                    return Err(TokenizerError::InvalidToken(token));
                }
            }
        }
        
        // Add special tokens if needed
        if let Some(bos_id) = self.special_tokens.get(&self.config.bos_token) {
            token_ids.insert(0, *bos_id);
        }
        
        if let Some(eos_id) = self.special_tokens.get(&self.config.eos_token) {
            token_ids.push(*eos_id);
        }
        
        // Truncate if necessary
        if token_ids.len() > self.config.max_length {
            match self.config.truncation_side.as_str() {
                "left" => {
                    let start = token_ids.len() - self.config.max_length;
                    token_ids = token_ids[start..].to_vec();
                }
                _ => {
                    token_ids.truncate(self.config.max_length);
                }
            }
        }
        
        // Pad if necessary
        if token_ids.len() < self.config.max_length {
            if let Some(pad_id) = self.special_tokens.get(&self.config.pad_token) {
                let padding = vec![*pad_id; self.config.max_length - token_ids.len()];
                match self.config.padding_side.as_str() {
                    "left" => {
                        let mut new_tokens = padding;
                        new_tokens.extend(token_ids);
                        token_ids = new_tokens;
                    }
                    _ => {
                        token_ids.extend(padding);
                    }
                }
            }
        }
        
        Ok(token_ids)
    }
    
    pub fn decode(&self, token_ids: &[u32], skip_special_tokens: bool) -> String {
        let mut tokens = Vec::with_capacity(token_ids.len());
        
        for &token_id in token_ids {
            if skip_special_tokens {
                if let Some(token) = self.inv_vocab.get(&token_id) {
                    if !self.config.special_tokens.contains(token) {
                        tokens.push(token.clone());
                    }
                }
            } else if let Some(token) = self.inv_vocab.get(&token_id) {
                tokens.push(token.clone());
            }
        }
        
        // Reconstruct text from BPE tokens
        let text = tokens.join("");
        self.bpe.decode(&text)
    }
    
    pub fn get_vocab_size(&self) -> usize {
        self.vocab.len()
    }
    
    pub fn get_pad_token_id(&self) -> Option<u32> {
        self.special_tokens.get(&self.config.pad_token).copied()
    }
    
    pub fn get_unk_token_id(&self) -> Option<u32> {
        self.special_tokens.get(&self.config.unk_token).copied()
    }
    
    pub fn get_bos_token_id(&self) -> Option<u32> {
        self.special_tokens.get(&self.config.bos_token).copied()
    }
    
    pub fn get_eos_token_id(&self) -> Option<u32> {
        self.special_tokens.get(&self.config.eos_token).copied()
    }
    
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, TokenizerError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        // In a real implementation, you would load the tokenizer from a JSON or similar format
        // This is a simplified version
        let config = TokenizerConfig::default();
        let tokenizer = Self::new(config)?;
        
        Ok(tokenizer)
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), TokenizerError> {
        // In a real implementation, you would save the tokenizer configuration and vocabulary
        // This is a simplified version
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenizer() {
        let config = TokenizerConfig::default();
        let tokenizer = Tokenizer::new(config).unwrap();
        
        let text = "Hello, world! This is a test.";
        let token_ids = tokenizer.encode(text).unwrap();
        let decoded = tokenizer.decode(&token_ids, true);
        
        assert!(!token_ids.is_empty());
        assert_eq!(decoded, text);
    }
}
