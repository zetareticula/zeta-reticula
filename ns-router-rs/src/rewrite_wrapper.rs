use egg::{Rewrite, SymbolLang, Pattern};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RewriteError {
    #[error("Failed to parse pattern: {0}")]
    ParseError(String),
}

/// A serializable wrapper around Rewrite<SymbolLang, ()>
#[derive(Clone, Debug)]
pub struct SerializableRewrite {
    name: String,
    left: String,
    right: String,
    rewrite: Option<Rewrite<SymbolLang, ()>>,
}

impl SerializableRewrite {
    /// Create a new serializable rewrite
    pub fn new(
        name: impl Into<String>,
        left: impl Into<String>,
        right: impl Into<String>,
    ) -> Result<Self, RewriteError> {
        let name = name.into();
        let left = left.into();
        let right = right.into();
        
        // Parse patterns to validate them
        let _left_pat: Pattern<SymbolLang> = left.parse()
            .map_err(|e| RewriteError::ParseError(format!("Left pattern: {}", e)))?;
        let _right_pat: Pattern<SymbolLang> = right.parse()
            .map_err(|e| RewriteError::ParseError(format!("Right pattern: {}", e)))?;
        
        Ok(Self {
            name,
            left,
            right,
            rewrite: None,
        })
    }
    
    /// Get the inner rewrite, initializing it if necessary
    pub fn get_rewrite(&mut self) -> Result<&Rewrite<SymbolLang, ()>, RewriteError> {
        if self.rewrite.is_none() {
            let left_pat: Pattern<SymbolLang> = self.left.parse()
                .map_err(|e| RewriteError::ParseError(format!("Left pattern: {}", e)))?;
            let right_pat: Pattern<SymbolLang> = self.right.parse()
                .map_err(|e| RewriteError::ParseError(format!("Right pattern: {}", e)))?;
                
            self.rewrite = Some(Rewrite::new(
                self.name.clone(),
                left_pat,
                right_pat,
            ));
        }
        
        self.rewrite.as_ref().ok_or_else(|| 
            RewriteError::ParseError("Failed to initialize rewrite".to_string())
        )
    }
}

impl Serialize for SerializableRewrite {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        
        let mut state = serializer.serialize_struct("SerializableRewrite", 3)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("left", &self.left)?;
        state.serialize_field("right", &self.right)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for SerializableRewrite {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            name: String,
            left: String,
            right: String,
        }
        
        let helper = Helper::deserialize(deserializer)?;
        SerializableRewrite::new(helper.name, helper.left, helper.right)
            .map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for SerializableRewrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} => {}", self.name, self.left, self.right)
    }
}
