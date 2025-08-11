// Copyright 2025 ZETA RETICULA
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

//! Byte Pair Encoding (BPE) implementation

use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::{Ordering, Reverse};
use std::fmt;
use regex::Regex;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BpePair {
    left: String,
    right: String,
}

impl BpePair {
    fn new(left: &str, right: &str) -> Self {
        Self {
            left: left.to_string(),
            right: right.to_string(),
        }
    }
}

impl fmt::Display for BpePair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.left, self.right)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BpeMerge {
    pair: BpePair,
    count: usize,
    pos: usize,
}

impl Ord for BpeMerge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count
            .cmp(&other.count)
            .then_with(|| self.pos.cmp(&other.pos).reverse())
    }
}

impl PartialOrd for BpeMerge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct BPE {
    vocab: HashMap<String, u32>,
    merges: HashMap<BpePair, (u32, u32)>,
    cache: HashMap<String, Vec<String>>,
    dropout: Option<f32>,
    unk_token: Option<String>,
    continuing_subword_prefix: String,
    end_of_word_suffix: String,
    byte_encoder: HashMap<u8, String>,
    byte_decoder: HashMap<String, u8>,
    pattern: Regex,
    special_tokens: HashSet<String>,
    special_tokens_encoder: HashMap<String, u32>,
    special_tokens_decoder: HashMap<u32, String>,
    added_tokens_encoder: HashMap<String, u32>,
    added_tokens_decoder: HashMap<u32, String>,
}

impl Default for BPE {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl BPE {
    pub fn new() -> Result<Self, BpeError> {
        // Initialize byte-level encoder/decoder
        let mut byte_encoder = HashMap::new();
        let mut byte_decoder = HashMap::new();
        
        // Bytes 0-255 to their unicode string representation
        for byte in 0..=255u8 {
            let char_str = format!("<0x{:02X}>", byte);
            byte_encoder.insert(byte, char_str.clone());
            byte_decoder.insert(char_str, byte);
        }
        
        // Default pattern: splits on whitespace and punctuation
        let pattern = Regex::new(r"'s|'t|'re|'ve|'m|'ll|'d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+\S|\s+")
            .map_err(|e| BpeError::InvalidMerge(e.to_string()))?;
        
        Ok(Self {
            vocab: HashMap::new(),
            merges: HashMap::new(),
            cache: HashMap::new(),
            dropout: None,
            unk_token: None,
            continuing_subword_prefix: "##".to_string(),
            end_of_word_suffix: "</w>".to_string(),
            byte_encoder,
            byte_decoder,
            pattern,
            special_tokens: HashSet::new(),
            special_tokens_encoder: HashMap::new(),
            special_tokens_decoder: HashMap::new(),
            added_tokens_encoder: HashMap::new(),
            added_tokens_decoder: HashMap::new(),
        })
    }
    
    pub fn tokenize(&self, text: &str) -> Result<Vec<String>, BpeError> {
        if self.vocab.is_empty() {
            return Err(BpeError::VocabularyNotLoaded);
        }
        
        let mut tokens = Vec::new();
        
        // First, check for special tokens
        if !self.special_tokens.is_empty() {
            // In a real implementation, you would split on special tokens first
            // This is a simplified version
            tokens.push(text.to_string());
        } else {
            // Normal tokenization
            for token in self.pattern.find_iter(text) {
                let token_str = token.as_str();
                
                // Convert to bytes and then to string representation
                let mut byte_tokens = Vec::new();
                for &byte in token_str.as_bytes() {
                    if let Some(token) = self.byte_encoder.get(&byte) {
                        byte_tokens.push(token.clone());
                    }
                }
                
                // Apply BPE
                let bpe_tokens = self.bpe(byte_tokens.join(""));
                tokens.extend(bpe_tokens);
            }
        }
        
        Ok(tokens)
    }
    
    pub fn decode(&self, text: &str) -> String {
        // In a real implementation, you would convert the token IDs back to text
        // This is a simplified version that just returns the input
        text.to_string()
    }
    
    fn bpe(&self, token: String) -> Vec<String> {
        if let Some(cached) = self.cache.get(&token) {
            return cached.clone();
        }
        
        let mut word: Vec<String> = token.chars().map(|c| c.to_string()).collect();
        let mut pairs = self.get_pairs(&word);
        
        if pairs.is_empty() {
            return vec![token];
        }
        
        while !pairs.is_empty() {
            let bigram = self.find_most_frequent_pair(&pairs);
            
            if !self.merges.contains_key(&bigram) {
                break;
            }
            
            let (first, second) = self.merges[&bigram];
            let mut new_word = Vec::new();
            let mut i = 0;
            
            while i < word.len() {
                if let Some(j) = (i..word.len()).find(|&j| word[j] == bigram.left) {
                    new_word.extend_from_slice(&word[i..j]);
                    i = j;
                    
                    if i < word.len() - 1 && word[i] == bigram.left && word[i + 1] == bigram.right {
                        new_word.push(format!("{}{}", bigram.left, bigram.right));
                        i += 2;
                    } else {
                        new_word.push(word[i].clone());
                        i += 1;
                    }
                } else {
                    new_word.extend_from_slice(&word[i..]);
                    break;
                }
            }
            
            word = new_word;
            if word.len() == 1 {
                break;
            } else {
                pairs = self.get_pairs(&word);
            }
        }
        
        self.cache.insert(token, word.clone());
        word
    }
    
    fn get_pairs(&self, word: &[String]) -> HashSet<BpePair> {
        let mut pairs = HashSet::new();
        
        for i in 0..word.len() - 1 {
            let pair = BpePair::new(&word[i], &word[i + 1]);
            pairs.insert(pair);
        }
        
        pairs
    }
    
    fn find_most_frequent_pair(&self, pairs: &HashSet<BpePair>) -> BpePair {
        // In a real implementation, you would find the most frequent pair based on the merge rules
        // This is a simplified version that just returns the first pair
        pairs.iter().next().cloned().unwrap_or_else(|| BpePair::new("", ""))
    }
    
    pub fn add_special_tokens(&mut self, tokens: &[String]) {
        for token in tokens {
            self.special_tokens.insert(token.clone());
        }
    }
    
    pub fn set_dropout(&mut self, dropout: Option<f32>) -> Result<(), BpeError> {
        if let Some(dropout) = dropout {
            if !(0.0..=1.0).contains(&dropout) {
                return Err(BpeError::InvalidMerge("Dropout must be between 0 and 1".to_string()));
            }
        }
        self.dropout = dropout;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bpe_tokenize() {
        let bpe = BPE::new().unwrap();
        let text = "Hello, world!";
        let tokens = bpe.tokenize(text).unwrap();
        
        assert!(!tokens.is_empty());
        assert!(tokens.len() <= text.len());
    }
    
    #[test]
    fn test_bpe_decode() {
        let bpe = BPE::new().unwrap();
        let text = "Hello, world!";
        let tokens = bpe.tokenize(text).unwrap();
        let decoded = bpe.decode(&tokens.join(""));
        
        assert_eq!(decoded, text);
    }
}
