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

//! Error types for the quantize-cli

use thiserror::Error;

/// Main error type for the application
#[derive(Error, Debug, PartialEq)]
pub enum CliError {
    /// IO error
    #[error("IO error: {0}")]
    Io(String),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Model loading/saving error
    #[error("Model error: {0}")]
    Model(String),
    
    /// Quantization error
    #[error("Quantization error: {0}")]
    Quantization(String),
    
    /// Inference error
    #[error("Inference error: {0}")]
    Inference(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

/// Result type for the application
pub type Result<T> = std::result::Result<T, CliError>;

/// Convert from other error types to our error type
impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::Serialization(err.to_string())
    }
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        CliError::Config(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let cli_err: CliError = io_err.into();
        assert_eq!(
            cli_err,
            CliError::Io("file not found".to_string())
        );
    }

    #[test]
    fn test_serde_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("{invalid}").unwrap_err();
        let cli_err: CliError = json_err.into();
        assert!(matches!(cli_err, CliError::Serialization(_)));
    }

    #[test]
    fn test_anyhow_error_conversion() {
        let anyhow_err = anyhow::anyhow!("test error");
        let cli_err: CliError = anyhow_err.into();
        assert_eq!(
            cli_err,
            CliError::Config("test error".to_string())
        );
    }

    #[test]
    fn test_error_display() {
        let io_err = CliError::Io("test io error".to_string());
        assert_eq!(io_err.to_string(), "IO error: test io error");

        let model_err = CliError::Model("test model error".to_string());
        assert_eq!(model_err.to_string(), "Model error: test model error");
    }
}
