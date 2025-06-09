use neon::prelude::*;
use crate::quantizer::{SalienceQuantizer, TokenFeatures};
use serde_json;

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

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("quantizeTokens", quantize_tokens)?;
    Ok(())
}