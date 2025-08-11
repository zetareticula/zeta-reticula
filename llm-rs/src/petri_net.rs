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


//! Petri net implementation for request tracing and auditing

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use uuid::Uuid;
use thiserror::Error;
use tracing::{info, debug, error};

#[derive(Error, Debug)]
pub enum PetriNetError {
    #[error("Transition not enabled")]
    TransitionNotEnabled,
    #[error("Invalid transition: {0}")]
    InvalidTransition(String),
    #[error("Place not found: {0}")]
    PlaceNotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub id: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

impl Token {
    pub fn new(data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub capacity: Option<usize>,
    pub tokens: VecDeque<Token>,
}

impl Place {
    pub fn new(id: impl Into<String>, name: impl Into<String>, capacity: Option<usize>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            capacity,
            tokens: VecDeque::new(),
        }
    }
    
    pub fn add_token(&mut self, token: Token) -> Result<(), PetriNetError> {
        if let Some(cap) = self.capacity {
            if self.tokens.len() >= cap {
                return Err(PetriNetError::InvalidTransition(
                    format!("Place {} is at capacity", self.id)
                ));
            }
        }
        
        self.tokens.push_back(token);
        Ok(())
    }
    
    pub fn take_token(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }
    
    pub fn has_tokens(&self) -> bool {
        !self.tokens.is_empty()
    }
    
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
    pub name: String,
    pub input_arcs: Vec<ArcDef>,
    pub output_arcs: Vec<ArcDef>,
    pub guard: Option<Arc<dyn Fn(&[&Token]) -> bool + Send + Sync>>,
    pub action: Option<Arc<dyn Fn(Vec<Token>) -> Vec<Token> + Send + Sync>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcDef {
    pub place_id: String,
    pub weight: usize,
}

impl Transition {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        input_arcs: Vec<ArcDef>,
        output_arcs: Vec<ArcDef>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input_arcs,
            output_arcs,
            guard: None,
            action: None,
        }
    }
    
    pub fn with_guard<F>(mut self, guard: F) -> Self
    where
        F: Fn(&[&Token]) -> bool + Send + Sync + 'static,
    {
        self.guard = Some(Arc::new(guard));
        self
    }
    
    pub fn with_action<F>(mut self, action: F) -> Self
    where
        F: Fn(Vec<Token>) -> Vec<Token> + Send + Sync + 'static,
    {
        self.action = Some(Arc::new(action));
        self
    }
}

#[derive(Debug, Default)]
pub struct PetriNet {
    places: HashMap<String, Arc<tokio::sync::RwLock<Place>>>,
    transitions: HashMap<String, Arc<Transition>>,
    trace_log: Arc<Mutex<Vec<TransitionLog>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransitionLog {
    pub transition_id: String,
    pub timestamp: u64,
    pub input_tokens: Vec<serde_json::Value>,
    pub output_tokens: Vec<serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

impl PetriNet {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_place(&mut self, place: Place) -> &mut Self {
        let id = place.id.clone();
        self.places.insert(id, Arc::new(tokio::sync::RwLock::new(place)));
        self
    }
    
    pub fn add_transition(&mut self, transition: Transition) -> &mut Self {
        let id = transition.id.clone();
        self.transitions.insert(id, Arc::new(transition));
        self
    }
    
    pub async fn add_token(&self, place_id: &str, token: Token) -> Result<(), PetriNetError> {
        let place = self.places.get(place_id)
            .ok_or_else(|| PetriNetError::PlaceNotFound(place_id.to_string()))?;
            
        let mut place = place.write().await;
        place.add_token(token)
    }
    
    pub async fn fire_transition(&self, transition_id: &str) -> Result<Vec<Token>, PetriNetError> {
        let transition = self.transitions.get(transition_id)
            .ok_or_else(|| PetriNetError::InvalidTransition(transition_id.to_string()))?;
            
        // Check if transition is enabled
        let input_tokens = self.get_input_tokens(transition).await?;
        
        // Check guard condition if present
        if let Some(guard) = &transition.guard {
            if !guard(&input_tokens.iter().collect::<Vec<_>>()) {
                return Err(PetriNetError::TransitionNotEnabled);
            }
        }
        
        // Consume input tokens
        self.consume_input_tokens(transition, &input_tokens).await?;
        
        // Apply action if present
        let output_tokens = if let Some(action) = &transition.action {
            action(input_tokens.clone())
        } else {
            // Default action: pass through input tokens
            input_tokens.clone()
        };
        
        // Produce output tokens
        self.produce_output_tokens(transition, &output_tokens).await?;
        
        // Log the transition
        self.log_transition(transition, &input_tokens, &output_tokens).await;
        
        Ok(output_tokens)
    }
    
    async fn get_input_tokens(&self, transition: &Transition) -> Result<Vec<Token>, PetriNetError> {
        let mut input_tokens = Vec::new();
        
        for arc in &transition.input_arcs {
            let place = self.places.get(&arc.place_id)
                .ok_or_else(|| PetriNetError::PlaceNotFound(arc.place_id.clone()))?;
                
            let place = place.read().await;
            if place.token_count() < arc.weight {
                return Err(PetriNetError::TransitionNotEnabled);
            }
            
            // Only check the first arc for simplicity
            // In a full implementation, we would handle all arcs
            if let Some(token) = place.tokens.front() {
                input_tokens.push(token.clone());
            }
        }
        
        Ok(input_tokens)
    }
    
    async fn consume_input_tokens(
        &self,
        transition: &Transition,
        tokens: &[Token],
    ) -> Result<(), PetriNetError> {
        for arc in &transition.input_arcs {
            let place = self.places.get(&arc.place_id)
                .ok_or_else(|| PetriNetError::PlaceNotFound(arc.place_id.clone()))?;
                
            let mut place = place.write().await;
            for _ in 0..arc.weight {
                place.take_token();
            }
        }
        
        Ok(())
    }
    
    async fn produce_output_tokens(
        &self,
        transition: &Transition,
        tokens: &[Token],
    ) -> Result<(), PetriNetError> {
        for arc in &transition.output_arcs {
            let place = self.places.get(&arc.place_id)
                .ok_or_else(|| PetriNetError::PlaceNotFound(arc.place_id.clone()))?;
                
            let mut place = place.write().await;
            for token in tokens {
                place.add_token(token.clone())?;
            }
        }
        
        Ok(())
    }
    
    async fn log_transition(
        &self,
        transition: &Transition,
        input_tokens: &[Token],
        output_tokens: &[Token],
    ) {
        let log = TransitionLog {
            transition_id: transition.id.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            input_tokens: input_tokens.iter().map(|t| t.data.clone()).collect(),
            output_tokens: output_tokens.iter().map(|t| t.data.clone()).collect(),
            metadata: HashMap::new(),
        };
        
        let mut trace_log = self.trace_log.lock().await;
        trace_log.push(log);
    }
    
    pub async fn get_trace_logs(&self) -> Vec<TransitionLog> {
        self.trace_log.lock().await.clone()
    }
    
    pub async fn to_json(&self) -> Result<String, serde_json::Error> {
        let state = serde_json::json!({
            "places": self.places,
            "transitions": self.transitions.values().map(|t| t.as_ref()).collect::<Vec<_>>(),
        });
        
        serde_json::to_string_pretty(&state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_petri_net() {
        let mut net = PetriNet::new();
        
        // Add places
        net.add_place(Place::new("p1", "Input Place", Some(10)));
        net.add_place(Place::new("p2", "Output Place", Some(10)));
        
        // Add transition
        net.add_transition(
            Transition::new(
                "t1",
                "Process Data",
                vec![ArcDef { place_id: "p1".to_string(), weight: 1 }],
                vec![ArcDef { place_id: "p2".to_string(), weight: 1 }],
            )
            .with_guard(|tokens| {
                // Check if the input token has a "valid" field set to true
                tokens[0].data.get("valid")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .with_action(|mut tokens| {
                // Transform the token data
                if let Some(token) = tokens.get_mut(0) {
                    token.data = json!({ "processed": true, "original": token.data });
                }
                tokens
            })
        );
        
        // Add a valid token to p1
        let token = Token::new(json!({ "valid": true, "data": "test" }));
        net.add_token("p1", token).await.unwrap();
        
        // Fire the transition
        let result = net.fire_transition("t1").await;
        assert!(result.is_ok());
        
        // Check that p1 is empty and p2 has a token
        let p1 = net.places.get("p1").unwrap().read().await;
        assert_eq!(p1.token_count(), 0);
        
        let p2 = net.places.get("p2").unwrap().read().await;
        assert_eq!(p2.token_count(), 1);
        
        // Check the transformed token data
        if let Some(token) = p2.tokens.front() {
            assert_eq!(token.data["processed"], true);
            assert_eq!(token.data["original"]["valid"], true);
            assert_eq!(token.data["original"]["data"], "test");
        } else {
            panic!("No token in output place");
        }
        
        // Check the trace log
        let logs = net.get_trace_logs().await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].transition_id, "t1");
    }
    
    #[tokio::test]
    async fn test_transition_guard() {
        let mut net = PetriNet::new();
        
        // Add places
        net.add_place(Place::new("p1", "Input Place", Some(10)));
        net.add_place(Place::new("p2", "Output Place", Some(10)));
        
        // Add transition with guard
        net.add_transition(
            Transition::new(
                "t1",
                "Process Data",
                vec![ArcDef { place_id: "p1".to_string(), weight: 1 }],
                vec![ArcDef { place_id: "p2".to_string(), weight: 1 }],
            )
            .with_guard(|tokens| {
                tokens[0].data.get("valid")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
        );
        
        // Add an invalid token to p1
        let token = Token::new(json!({ "valid": false }));
        net.add_token("p1", token).await.unwrap();
        
        // Try to fire the transition (should fail due to guard)
        let result = net.fire_transition("t1").await;
        assert!(matches!(result, Err(PetriNetError::TransitionNotEnabled)));
        
        // Add a valid token
        let token = Token::new(json!({ "valid": true }));
        net.add_token("p1", token).await.unwrap();
        
        // Now the transition should fire
        let result = net.fire_transition("t1").await;
        assert!(result.is_ok());
    }
}
