use crate::error::{UnifiedInputError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokenizers::{Tokenizer, AddedToken};
use async_trait::async_trait;

/// Configuration for the unified tokenizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizerConfig {
    pub vocab_size: usize,
    pub max_length: usize,
    pub pad_token: String,
    pub unk_token: String,
    pub bos_token: String,
    pub eos_token: String,
    pub mask_token: String,
    pub model_name: Option<String>,
    pub cache_dir: Option<String>,
    pub use_fast: bool,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            vocab_size: 50_257,
            max_length: 2048,
            pad_token: "[PAD]".to_string(),
            unk_token: "[UNK]".to_string(),
            bos_token: "[BOS]".to_string(),
            eos_token: "[EOS]".to_string(),
            mask_token: "[MASK]".to_string(),
            model_name: None,
            cache_dir: None,
            use_fast: true,
        }
    }
}

/// Unified tokenizer that handles both local and HuggingFace tokenizers
#[derive(Debug)]
pub struct UnifiedTokenizer {
    tokenizer: Tokenizer,
    config: TokenizerConfig,
    special_tokens: HashMap<String, u32>,
}

impl UnifiedTokenizer {
    /// Create a new unified tokenizer
    pub async fn new(config: TokenizerConfig) -> Result<Self> {
        let tokenizer = if let Some(model_name) = &config.model_name {
            // Load from HuggingFace Hub
            Self::load_from_hub(model_name, config.cache_dir.as_deref()).await?
        } else {
            // Create a basic tokenizer
            Self::create_basic_tokenizer(&config)?
        };

        let special_tokens = Self::extract_special_tokens(&tokenizer, &config)?;

        Ok(Self {
            tokenizer,
            config,
            special_tokens,
        })
    }

    /// Load tokenizer from HuggingFace Hub
    async fn load_from_hub(model_name: &str, cache_dir: Option<&str>) -> Result<Tokenizer> {
        let api = if let Some(cache_dir) = cache_dir {
            hf_hub::api::tokio::Api::new()?.with_cache_dir(cache_dir.into())
        } else {
            hf_hub::api::tokio::Api::new()?
        };

        let repo = api.model(model_name.to_string());
        let tokenizer_path = repo.get("tokenizer.json").await
            .map_err(|e| UnifiedInputError::HuggingFaceHub(e.to_string()))?;

        Tokenizer::from_file(tokenizer_path)
            .map_err(|e| UnifiedInputError::Tokenizer(e.to_string()))
    }

    /// Create a basic BPE tokenizer
    fn create_basic_tokenizer(config: &TokenizerConfig) -> Result<Tokenizer> {
        use tokenizers::models::bpe::BPE;
        use tokenizers::normalizers::Sequence;
        use tokenizers::pre_tokenizers::byte_level::ByteLevel;
        use tokenizers::processors::template::TemplateProcessing;

        let mut tokenizer = Tokenizer::new(
            BPE::builder()
                .unk_token(config.unk_token.clone())
                .build()
                .map_err(|e| UnifiedInputError::Tokenizer(e.to_string()))?
        );

        // Set up normalizer
        tokenizer.with_normalizer(Some(Sequence::new(vec![])));

        // Set up pre-tokenizer
        tokenizer.with_pre_tokenizer(Some(ByteLevel::default()));

        // Set up post-processor
        tokenizer.with_post_processor(Some(
            TemplateProcessing::builder()
                .try_single(format!("{} $A {}", config.bos_token, config.eos_token))
                .map_err(|e| UnifiedInputError::Tokenizer(e.to_string()))?
                .special_tokens(vec![
                    (config.bos_token.clone(), 1),
                    (config.eos_token.clone(), 2),
                ])
                .build()
                .map_err(|e| UnifiedInputError::Tokenizer(e.to_string()))?
        ));

        // Add special tokens
        let special_tokens = vec![
            AddedToken::from(config.pad_token.clone(), true),
            AddedToken::from(config.unk_token.clone(), true),
            AddedToken::from(config.bos_token.clone(), true),
            AddedToken::from(config.eos_token.clone(), true),
            AddedToken::from(config.mask_token.clone(), true),
        ];

        tokenizer.add_special_tokens(&special_tokens);

        Ok(tokenizer)
    }

    /// Extract special token IDs
    fn extract_special_tokens(tokenizer: &Tokenizer, config: &TokenizerConfig) -> Result<HashMap<String, u32>> {
        let mut special_tokens = HashMap::new();
        
        if let Some(vocab) = tokenizer.get_vocab(false) {
            for token in &[&config.pad_token, &config.unk_token, &config.bos_token, &config.eos_token, &config.mask_token] {
                if let Some(&id) = vocab.get(*token) {
                    special_tokens.insert(token.to_string(), id);
                }
            }
        }

        Ok(special_tokens)
    }

    /// Encode text to token IDs
    pub fn encode(&self, text: &str, add_special_tokens: bool) -> Result<Vec<u32>> {
        let encoding = self.tokenizer
            .encode(text, add_special_tokens)
            .map_err(|e| UnifiedInputError::TokenizationFailed(e.to_string()))?;

        Ok(encoding.get_ids().to_vec())
    }

    /// Encode batch of texts
    pub fn encode_batch(&self, texts: &[&str], add_special_tokens: bool) -> Result<Vec<Vec<u32>>> {
        let encodings = self.tokenizer
            .encode_batch(texts.to_vec(), add_special_tokens)
            .map_err(|e| UnifiedInputError::TokenizationFailed(e.to_string()))?;

        Ok(encodings.into_iter().map(|enc| enc.get_ids().to_vec()).collect())
    }

    /// Decode token IDs to text
    pub fn decode(&self, token_ids: &[u32], skip_special_tokens: bool) -> Result<String> {
        self.tokenizer
            .decode(token_ids, skip_special_tokens)
            .map_err(|e| UnifiedInputError::TokenizationFailed(e.to_string()))
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(false)
    }

    /// Get special token ID
    pub fn get_special_token_id(&self, token: &str) -> Option<u32> {
        self.special_tokens.get(token).copied()
    }

    /// Get pad token ID
    pub fn pad_token_id(&self) -> Option<u32> {
        self.get_special_token_id(&self.config.pad_token)
    }

    /// Get UNK token ID
    pub fn unk_token_id(&self) -> Option<u32> {
        self.get_special_token_id(&self.config.unk_token)
    }

    /// Get BOS token ID
    pub fn bos_token_id(&self) -> Option<u32> {
        self.get_special_token_id(&self.config.bos_token)
    }

    /// Get EOS token ID
    pub fn eos_token_id(&self) -> Option<u32> {
        self.get_special_token_id(&self.config.eos_token)
    }

    /// Save tokenizer to file
    pub fn save(&self, path: &Path) -> Result<()> {
        self.tokenizer
            .save(path, false)
            .map_err(|e| UnifiedInputError::Tokenizer(e.to_string()))
    }

    /// Load tokenizer from file
    pub async fn from_file(path: &Path, config: TokenizerConfig) -> Result<Self> {
        let tokenizer = Tokenizer::from_file(path)
            .map_err(|e| UnifiedInputError::Tokenizer(e.to_string()))?;

        let special_tokens = Self::extract_special_tokens(&tokenizer, &config)?;

        Ok(Self {
            tokenizer,
            config,
            special_tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_tokenizer() {
        let config = TokenizerConfig::default();
        let tokenizer = UnifiedTokenizer::new(config).await.unwrap();

        let text = "Hello, world!";
        let token_ids = tokenizer.encode(text, true).unwrap();
        let decoded = tokenizer.decode(&token_ids, false).unwrap();

        assert!(!token_ids.is_empty());
        assert!(decoded.contains("Hello"));
    }

    #[tokio::test]
    async fn test_batch_encoding() {
        let config = TokenizerConfig::default();
        let tokenizer = UnifiedTokenizer::new(config).await.unwrap();

        let texts = vec!["Hello", "World", "Test"];
        let batch_ids = tokenizer.encode_batch(&texts, true).unwrap();

        assert_eq!(batch_ids.len(), 3);
        for ids in batch_ids {
            assert!(!ids.is_empty());
        }
    }
}
