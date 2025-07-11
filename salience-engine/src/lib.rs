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


use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use log::info;

// ---- Core Data Structures ----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenFeatures {
    pub token_id: u32,
    pub frequency: f32,
    pub sentiment_score: f32,
    pub context_relevance: f32,
    pub role: String,
}

#[derive(Deserialize, Serialize)]
pub struct SalienceRequest {
    pub text: String,
    pub user_id: String,
}

#[derive(Serialize)]
pub struct SalienceResponse {
    pub salient_phrases: Vec<String>,
    pub upgrade_prompt: Option<String>,
}

lazy_static::lazy_static! {
    static ref USAGE_TRACKER: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
}

async fn process_salience(
    req: web::Json<SalienceRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = &req.user_id;
    let mut tracker = USAGE_TRACKER.lock().unwrap();
    let usage = tracker.entry(user_id.clone()).and_modify(|e| *e += 1).or_insert(1);

    let upgrade_prompt = if *usage > 30 && !cfg!(feature = "enterprise") {
        Some("Upgrade to Enterprise for more salience processing!".to_string())
    } else {
        None
    };

    // Simple salience extraction (splits text into words and takes first 3 as "salient")
    let salient_phrases: Vec<String> = req.text
        .split_whitespace()
        .take(3)
        .map(|s| s.to_string())
        .collect();

    info!("Processed salience request for user: {}", user_id);

    Ok(HttpResponse::Ok().json(SalienceResponse { 
        salient_phrases, 
        upgrade_prompt 
    }))
}

#[actix_web::main]
pub async fn start_server(port: u16) -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    info!("Starting salience-engine server on port {}", port);
    
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/salience")
                            .route(web::post().to(process_salience))
                    )
            )
    })
    .bind(("0.0.0.0", port))?
    .workers(2)
    .run()
    .await
}

#[cfg(not(test))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    start_server(8083).await
}

// ---- Core Data Structures ----



impl PrecisionLevel {
    fn to_string(&self) -> String {
        match self {
            PrecisionLevel::Bit4 => "Bit4".to_string(),
            PrecisionLevel::Bit8 => "Bit8".to_string(),
            PrecisionLevel::Bit16 => "Bit16".to_string(),
        }
    }
}



impl YoungTableau {
    pub fn new(rows: usize, threshold: f32) -> Self {
        YoungTableau {
            rows: vec![Vec::with_capacity(10); rows],
            dimensions: (rows, 10),
            threshold,
        }
    }

    pub fn insert(&mut self, result: QuantizationResult) {
        if result.row < self.dimensions.0 {
            self.rows[result.row].push(result);
        }
    }
}

// ---- Frame-Based Convolution with Gaussian Weighting ----


impl<'a> Frame<'a> {
    pub fn new(frame_id: u32, tokens: &'a [TokenFeatures]) -> Self {
        Frame {
            tokens,
            aggregated_salience: 0.0,
            frame_id,
        }
    }

    pub fn compute_salience(&mut self, threshold: f32, bump: &Bump) {
        let normal = Normal::new(0.0, 1.0).unwrap(); // Gaussian with mean=0, std=1
        let mut weights = bumpalo::vec![in bump; 0.0; self.tokens.len()];
        
        // Compute Gaussian weights based on token position
        for (i, weight) in weights.iter_mut().enumerate() {
            let position = i as f32 / self.tokens.len() as f32; // Normalize position
            *weight = normal.pdf(position);
        }

        // Normalize weights to sum to 1
        let weight_sum: f32 = weights.iter().sum();
        if weight_sum > 0.0 {
            weights.iter_mut().for_each(|w| *w /= weight_sum);
        }

        // Compute weighted salience
        self.aggregated_salience = self.tokens.iter()
            .zip(weights.iter())
            .filter(|(t, _)| t.frequency >= threshold)
            .map(|(t, w)| w * t.frequency * t.sentiment_score * t.context_relevance)
            .sum();
    }
}

pub struct SalienceQuantizer {
    threshold: f32,
    frames: Arc<RwLock<Vec<Frame<'static>>>>, // Lifetime managed by bump allocator
}

impl SalienceQuantizer {
    pub fn new(threshold: f32) -> Self {
        SalienceQuantizer {
            threshold,
            frames: Arc::new(RwLock::new(Vec::with_capacity(100))),
        }
    }

    pub fn quantize_tokens(
        &self,
        features: Vec<TokenFeatures>,
        theory_key: &str,
        bump: &Bump,
    ) -> (Vec<QuantizationResult>, YoungTableau) {
        let mut results = Vec::with_capacity(features.len());
        let mut tableau = YoungTableau::new(10, self.threshold);
        let mut frames = self.frames.write().unwrap();

        // Group tokens into frames using bump allocator
        let frame_size = 10;
        let chunks: Vec<_> = features.chunks(frame_size).collect();
        let mut temp_frames = bumpalo::vec![in bump; Frame::new(0, &[]); chunks.len()];

        // Parallel frame creation
        temp_frames
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, frame)| {
                let chunk = chunks[i];
                *frame = Frame::new(i as u32, bump.alloc_slice_copy(chunk));
                frame.compute_salience(self.threshold, bump);
            });

        frames.extend(temp_frames.into_iter());

        // Parallel frame processing for quantization
        let frame_results: Vec<Vec<QuantizationResult>> = frames
            .par_iter()
            .map(|frame| {
                let mut frame_results = bumpalo::vec![in bump; QuantizationResult {
                    token_id: 0,
                    precision: String::new(),
                    salience_score: 0.0,
                    row: 0,
                    role: String::new(),
                    role_confidence: 0.0,
                }; frame.tokens.len()];

                for (i, (feature, result)) in frame.tokens.iter().zip(frame_results.iter_mut()).enumerate() {
                    if feature.frequency < self.threshold {
                        continue;
                    }

                    let precision = if feature.frequency < 0.5 {
                        PrecisionLevel::Bit4
                    } else if feature.frequency < 1.0 {
                        PrecisionLevel::Bit8
                    } else {
                        PrecisionLevel::Bit16
                    };

                    let salience_score = feature.frequency * feature.sentiment_score * feature.context_relevance;

                    *result = QuantizationResult {
                        token_id: feature.token_id,
                        precision: precision.to_string(),
                        salience_score,
                        row: frame.frame_id as usize,
                        role: feature.role.clone(),
                        role_confidence: 1.0,
                    };
                }

                frame_results.into_iter().filter(|r| r.salience_score > 0.0).collect()
            })
            .collect();

        // Flatten results and insert into tableau
        for frame_result in frame_results {
            results.extend(frame_result.iter().cloned());
            for result in frame_result {
                tableau.insert(result);
            }
        }

        // Convolution: Aggregate frame-level results with Gaussian weighting
        let aggregated_results: Vec<QuantizationResult> = frames
            .par_iter()
            .map(|frame| {
                QuantizationResult {
                    token_id: frame.frame_id,
                    precision: PrecisionLevel::Bit16.to_string(),
                    salience_score: frame.aggregated_salience,
                    row: frame.frame_id as usize,
                    role: "frame".to_string(),
                    role_confidence: 1.0,
                }
            })
            .collect();

        (aggregated_results, tableau)
    }
}

// ---- Neon Integration ----

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("quantizeTokens", quantize_tokens)?;
    Ok(())
}

fn quantize_tokens(cx: FunctionContext) -> JsResult<JsString> {
    let input = cx.argument::<JsString>(0)?.value(&mut cx);
    let theory_key = cx.argument::<JsString>(1)?.value(&mut cx);
    let token_features: Vec<TokenFeatures> = serde_json::from_str(&input)
        .or_else(|_| cx.throw_error("Invalid input format"))?;

    let quantizer = SalienceQuantizer::new(0.7);
    let bump = Bump::new();
    let (results, _tableau) = quantizer.quantize_tokens(token_features, &theory_key, &bump);

    let output = serde_json::to_string(&results)
        .or_else(|_| cx.throw_error("Failed to serialize result"))?;
    Ok(cx.string(output))
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_quantize_tokens() {
        let input = json!([{
            "token_id": 1,
            "frequency": 0.5,
            "sentiment_score": 0.8,
            "context_relevance": 0.9,
            "role": "subject"
        }]).to_string();
        let theory_key = "test_theory";
        let bump = Bump::new();

        let quantizer = SalienceQuantizer::new(0.7);
        let results = quantizer.quantize_tokens_test(&input, theory_key, &bump).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].token_id, 1);
        assert_eq!(results[0].precision, "Bit4");
    }
}

impl SalienceQuantizer {
    pub fn quantize_tokens_test(
        &self,
        input: &str,
        theory_key: &str,
        bump: &Bump,
    ) -> Result<Vec<QuantizationResult>, String> {
        let token_features: Vec<TokenFeatures> = serde_json::from_str(input)
            .map_err(|_| "Invalid input format".to_string())?;

        let (results, _tableau) = self.quantize_tokens(token_features, theory_key, bump);
        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QuantizationResult {
    token_id: u32,
    precision: String,
    salience_score: f32,
    row: usize,
    role: String,
    role_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct YoungTableau {
    rows: Vec<Vec<QuantizationResult>>,
    dimensions: (usize, usize),
    threshold: f32,
}

impl YoungTableau {
    pub fn new(rows: usize, threshold: f32) -> Self {
        YoungTableau {
            rows: vec![Vec::with_capacity(10); rows],
            dimensions: (rows, 10),
            threshold,
        }
    }

    pub fn insert(&mut self, result: QuantizationResult) {
        if result.row < self.dimensions.0 {
            self.rows[result.row].push(result);
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Frame<'a> {
    tokens: &'a [TokenFeatures],
    aggregated_salience: f32,
    frame_id: u32,
}


impl<'a> Frame<'a> {
    pub fn new(frame_id: u32, tokens: &'a [TokenFeatures]) -> Self {
        Frame {
            tokens,
            aggregated_salience: 0.0,
            frame_id,
        }
    }

    pub fn compute_salience(&mut self, threshold: f32, bump: &Bump) {
        let normal = Normal::new(0.0, 1.0).unwrap(); // Gaussian with mean=0, std=1
        let mut weights = bumpalo::vec![in bump; 0.0; self.tokens.len()];
        
        // Compute Gaussian weights based on token position
        for (i, weight) in weights.iter_mut().enumerate() {
            let position = i as f32 / self.tokens.len() as f32; // Normalize position
            *weight = normal.pdf(position);
        }

        // Normalize weights to sum to 1
        let weight_sum: f32 = weights.iter().sum();
        if weight_sum > 0.0 {
            weights.iter_mut().for_each(|w| *w /= weight_sum);
        }

        // Compute weighted salience
        self.aggregated_salience = self.tokens.iter()
            .zip(weights.iter())
            .filter(|(t, _)| t.frequency >= threshold)
            .map(|(t, w)| w * t.frequency * t.sentiment_score * t.context_relevance)
            .sum();
    }
}