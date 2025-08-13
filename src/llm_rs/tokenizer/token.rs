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

//! Token representation and related types

use std::fmt;
use serde::{Serialize, Deserialize};

/// Represents a single token with its position and attributes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// The token text
    pub text: String,
    /// The token ID in the vocabulary
    pub id: u32,
    /// The start byte offset in the original text
    pub offset_begin: usize,
    /// The end byte offset in the original text
    pub offset_end: usize,
    /// Whether this is a special token
    pub special: bool,
}

impl Token {
    /// Create a new token
    pub fn new(text: String, id: u32, offset_begin: usize, offset_end: usize, special: bool) -> Self {
        Self {
            text,
            id,
            offset_begin,
            offset_end,
            special,
        }
    }
    
    /// Create a special token
    pub fn special(text: &str, id: u32) -> Self {
        Self {
            text: text.to_string(),
            id,
            offset_begin: 0,
            offset_end: 0,
            special: true,
        }
    }
    
    /// Get the length of the token in characters
    pub fn len_chars(&self) -> usize {
        self.text.chars().count()
    }
    
    /// Check if the token is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token(text='{}', id={}, offsets=({},{}), special={})",
            self.text, self.id, self.offset_begin, self.offset_end, self.special
        )
    }
}

/// Represents the result of tokenization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenizedInput {
    /// The original text
    pub original: String,
    /// The tokens
    pub tokens: Vec<Token>,
    /// The token IDs
    pub ids: Vec<u32>,
    /// The attention mask (1 for real tokens, 0 for padding)
    pub attention_mask: Vec<u32>,
    /// The token type IDs (for sentence pairs)
    pub token_type_ids: Vec<u32>,
    /// The offsets for each token in the original text
    pub offsets: Vec<(usize, usize)>,
    /// The special tokens mask (1 for special tokens, 0 for regular tokens)
    pub special_tokens_mask: Vec<u32>,
}

impl TokenizedInput {
    /// Create a new TokenizedInput
    pub fn new(original: String) -> Self {
        Self {
            original,
            tokens: Vec::new(),
            ids: Vec::new(),
            attention_mask: Vec::new(),
            token_type_ids: Vec::new(),
            offsets: Vec::new(),
            special_tokens_mask: Vec::new(),
        }
    }
    
    /// Add a token to the tokenized input
    pub fn add_token(&mut self, token: Token) {
        self.tokens.push(token.clone());
        self.ids.push(token.id);
        self.attention_mask.push(1);
        self.token_type_ids.push(0);
        self.offsets.push((token.offset_begin, token.offset_end));
        self.special_tokens_mask.push(if token.special { 1 } else { 0 });
    }
    
    /// Add special tokens to the tokenized input
    pub fn add_special_token(&mut self, token: &Token) {
        self.tokens.push(token.clone());
        self.ids.push(token.id);
        self.attention_mask.push(1);
        self.token_type_ids.push(0);
        self.offsets.push((token.offset_begin, token.offset_end));
        self.special_tokens_mask.push(1);
    }
    
    /// Pad the tokenized input to the specified length
    pub fn pad_to(&mut self, max_length: usize, pad_token_id: u32) {
        while self.ids.len() < max_length {
            self.ids.push(pad_token_id);
            self.attention_mask.push(0);
            self.token_type_ids.push(0);
            self.offsets.push((0, 0));
            self.special_tokens_mask.push(1);
        }
    }
    
    /// Truncate the tokenized input to the specified length
    pub fn truncate(&mut self, max_length: usize) {
        if self.ids.len() > max_length {
            self.ids.truncate(max_length);
            self.attention_mask.truncate(max_length);
            self.token_type_ids.truncate(max_length);
            self.offsets.truncate(max_length);
            self.special_tokens_mask.truncate(max_length);
            self.tokens.truncate(max_length);
        }
    }
    
    /// Get the length of the tokenized input
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    
    /// Check if the tokenized input is empty
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_creation() {
        let token = Token::new("hello".to_string(), 1, 0, 5, false);
        assert_eq!(token.text, "hello");
        assert_eq!(token.id, 1);
        assert_eq!(token.offset_begin, 0);
        assert_eq!(token.offset_end, 5);
        assert!(!token.special);
    }
    
    #[test]
    fn test_special_token() {
        let token = Token::special("[CLS]", 101);
        assert_eq!(token.text, "[CLS]");
        assert_eq!(token.id, 101);
        assert!(token.special);
    }
    
    #[test]
    fn test_tokenized_input() {
        let mut tokenized = TokenizedInput::new("Hello, world!".to_string());
        let token1 = Token::new("Hello".to_string(), 1, 0, 5, false);
        let token2 = Token::new(",".to_string(), 2, 5, 6, false);
        let token3 = Token::new("world".to_string(), 3, 7, 12, false);
        let token4 = Token::new("!".to_string(), 4, 12, 13, false);
        
        tokenized.add_token(token1);
        tokenized.add_token(token2);
        tokenized.add_token(token3);
        tokenized.add_token(token4);
        
        assert_eq!(tokenized.len(), 4);
        assert_eq!(tokenized.ids, vec![1, 2, 3, 4]);
        assert_eq!(tokenized.attention_mask, vec![1, 1, 1, 1]);
    }
    
    #[test]
    fn test_pad_and_truncate() {
        let mut tokenized = TokenizedInput::new("Hello".to_string());
        tokenized.add_token(Token::new("Hello".to_string(), 1, 0, 5, false));
        
        // Test padding
        tokenized.pad_to(5, 0);
        assert_eq!(tokenized.len(), 5);
        assert_eq!(tokenized.ids, vec![1, 0, 0, 0, 0]);
        
        // Test truncation
        tokenized.truncate(3);
        assert_eq!(tokenized.len(), 3);
        assert_eq!(tokenized.ids, vec![1, 0, 0]);
    }
}
