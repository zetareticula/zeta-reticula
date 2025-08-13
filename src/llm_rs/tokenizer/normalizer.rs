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

//! Text normalization utilities for tokenization

use unicode_normalization::UnicodeNormalization;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Whitespace normalization
    static ref WHITESPACE_RE: Regex = Regex::new(r"\s+").unwrap();
    
    // Control characters
    static ref CONTROL_CHARS_RE: Regex = Regex::new(r"[\u0000-\u001F\u007F-\u009F]").unwrap();
    
    // Punctuation normalization
    static ref PUNCTUATION_RE: Regex = Regex::new(r"[\u{2000}-\u{206F}\u{2E00}-\u{2E7F}\"'`\-]").unwrap();
}

/// Text normalizer that handles various text preprocessing steps
#[derive(Debug, Clone, Default)]
pub struct Normalizer {
    lowercase: bool,
    strip_accents: bool,
    clean_text: bool,
    handle_chinese_chars: bool,
    strip_whitespace: bool,
    remove_control_chars: bool,
}

impl Normalizer {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set whether to convert text to lowercase
    pub fn with_lowercase(mut self, lowercase: bool) -> Self {
        self.lowercase = lowercase;
        self
    }
    
    /// Set whether to strip accents
    pub fn with_strip_accents(mut self, strip_accents: bool) -> Self {
        self.strip_accents = strip_accents;
        self
    }
    
    /// Set whether to clean text (remove control characters, etc.)
    pub fn with_clean_text(mut self, clean_text: bool) -> Self {
        self.clean_text = clean_text;
        self
    }
    
    /// Set whether to handle Chinese characters specially
    pub fn with_chinese_chars(mut self, handle: bool) -> Self {
        self.handle_chinese_chars = handle;
        self
    }
    
    /// Set whether to strip whitespace
    pub fn with_strip_whitespace(mut self, strip: bool) -> Self {
        self.strip_whitespace = strip;
        self
    }
    
    /// Set whether to remove control characters
    pub fn with_remove_control_chars(mut self, remove: bool) -> Self {
        self.remove_control_chars = remove;
        self
    }
    
    /// Normalize the input text according to the specified options
    pub fn normalize(&self, text: &str) -> String {
        let mut text = text.to_string();
        
        if self.clean_text {
            text = Self::clean_text(&text);
        }
        
        if self.handle_chinese_chars {
            text = Self::add_whitespace_around_chinese_chars(&text);
        }
        
        if self.strip_accents {
            text = Self::strip_accents(&text);
        }
        
        if self.lowercase {
            text = text.to_lowercase();
        }
        
        if self.remove_control_chars {
            text = CONTROL_CHARS_RE.replace_all(&text, "").to_string();
        }
        
        if self.strip_whitespace {
            text = WHITESPACE_RE.replace_all(&text, " ").trim().to_string();
        }
        
        text
    }
    
    /// Clean text by removing control characters and normalizing whitespace
    fn clean_text(text: &str) -> String {
        let text = text.replace("\r\n", "\n");
        let text = text.replace('\', "");
        text
    }
    
    /// Add whitespace around Chinese characters to handle them as separate tokens
    fn add_whitespace_around_chinese_chars(text: &str) -> String {
        let mut result = String::new();
        let mut last_char_was_chinese = false;
        
        for c in text.chars() {
            let is_chinese = is_chinese_char(c);
            
            if is_chinese && !last_char_was_chinese && !result.is_empty() {
                result.push(' ');
            } else if !is_chinese && last_char_was_chinese && !result.is_empty() {
                result.push(' ');
            }
            
            result.push(c);
            last_char_was_chinese = is_chinese;
        }
        
        result
    }
    
    /// Strip accents from text using Unicode normalization
    fn strip_accents(text: &str) -> String {
        text.nfd()
            .filter(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c.is_whitespace())
            .collect::<String>()
            .nfc()
            .collect()
    }
}

/// Check if a character is a Chinese character
fn is_chinese_char(c: char) -> bool {
    match c as u32 {
        0x4E00..=0x9FFF |
        0x3400..=0x4DBF |
        0x20000..=0x2A6DF |
        0x2A700..=0x2B73F |
        0x2B740..=0x2B81F |
        0x2B820..=0x2CEAF |
        0xF900..=0xFAFF |
        0x2F800..=0x2FA1F => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_lowercase() {
        let normalizer = Normalizer::new().with_lowercase(true);
        assert_eq!(normalizer.normalize("Hello WORLD"), "hello world");
    }
    
    #[test]
    fn test_normalize_strip_accents() {
        let normalizer = Normalizer::new().with_strip_accents(true);
        assert_eq!(normalizer.normalize("déjà vu"), "deja vu");
    }
    
    #[test]
    fn test_chinese_chars() {
        let normalizer = Normalizer::new().with_chinese_chars(true);
        let text = "你好hello世界";
        let normalized = normalizer.normalize(text);
        assert!(normalized.contains(' '));
    }
    
    #[test]
    fn test_clean_text() {
        let normalizer = Normalizer::new().with_clean_text(true);
        let text = "Hello\r\nWorld\n";
        assert_eq!(normalizer.normalize(text), "Hello\nWorld\n");
    }
}
