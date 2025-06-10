use neon::prelude::*;
use crate::quantizer::{SalienceQuantizer, TokenFeatures};
use serde_json;

pub mod quantizer;
pub mod tableaux;

#[derive(Serialize, Deserialize)]
pub struct TokenFeatures {
    pub token_id: u32,
    pub frequency: f32,
    pub sentiment_score: f32,
    pub context_relevance: f32,
    pub role: String,  // e.g., "subject", "modifier"
}

fn quantize_tokens(mut cx: FunctionContext) -> JsResult<JsString> {
    let input = cx.argument::<JsString>(0)?.value(&mut cx);
    let theory_key = cx.argument::<JsString>(1)?.value(&mut cx);
    let token_features: Vec<TokenFeatures> = serde_json::from_str(&input)
        .or_else(|_| cx.throw_error("Invalid input format"))?;

    let quantizer = SalienceQuantizer::new(0.7);
    let (results, _tableau) = quantizer.quantize_tokens(token_features, &theory_key);

    let output = serde_json::to_string(&results)
        .or_else(|_| cx.throw_error("Failed to serialize result"))?;
    Ok(cx.string(output))
}

//----// Main entry point for the Neon module
//----//
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("quantizeTokens", quantize_tokens)?;
    Ok(())
}

#[derive(Serialize, Deserialize)]   
pub struct QuantizationResult {
    pub token_id: u32,
    pub precision: String,  // e.g., "Bit4", "Bit8", "Bit16"
    pub salience_score: f32,
    pub row: usize,
    pub role: String,
    pub role_confidence: f32,
}

//----// Tests for the quantization function
//----//
use neon::test::TestContext;
use neon::prelude::*;
use neon::result::NeonResult;
use crate::quantizer::{SalienceQuantizer, TokenFeatures};
use serde_json::json;
use serde_json::Value;
use serde::{Serialize, Deserialize};

// Define the quantization function for testing
pub fn quantize_tokens_test(input: &str, theory_key: &str) -> Result<Vec<QuantizationResult>, String> {
    let token_features: Vec<TokenFeatures> = serde_json::from_str(input)
        .map_err(|_| "Invalid input format".to_string())?;

    let quantizer = SalienceQuantizer::new(0.7);
    let (results, _tableau) = quantizer.quantize_tokens(token_features, theory_key);

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use neon::prelude::*;

    #[test]
    fn test_quantize_tokens() {
        let mut cx = TestContext::new();
        let input = r#"[{"token_id": 1, "frequency": 0.5, "sentiment_score": 0.0, "context_relevance": 0.5, "role": "subject"}]"#;
        let theory_key = "test_theory";

        let js_input = cx.string(input);
        let js_theory_key = cx.string(theory_key);

        let result = quantize_tokens(cx, js_input, js_theory_key).unwrap();
        assert!(result.value(&mut cx).contains("token_id"));
    }
}
