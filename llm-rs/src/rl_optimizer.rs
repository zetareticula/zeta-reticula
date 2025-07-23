//! RL-based optimization for KV cache and quantization parameters

use ndarray::{Array2, ArrayView2};
use tch::{Device, Kind, Tensor, nn};
use std::collections::VecDeque;
use rand::Rng;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Instant, Duration};

#[derive(Error, Debug)]
pub enum RLOptimizerError {
    #[error("CUDA not available")]
    CudaUnavailable,
    #[error("Model error: {0}")]
    ModelError(String),
    #[error("Invalid state dimensions")]
    InvalidStateDimensions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLOptimizerConfig {
    pub state_dim: usize,
    pub action_dim: usize,
    pub batch_size: usize,
    pub memory_capacity: usize,
    pub gamma: f32,
    pub tau: f32,
    pub lr: f64,
    pub update_every: usize,
    pub device: String,
}

impl Default for RLOptimizerConfig {
    fn default() -> Self {
        Self {
            state_dim: 128,
            action_dim: 8,  // 8 possible bit-depths (1-8 bits)
            batch_size: 64,
            memory_capacity: 10000,
            gamma: 0.99,
            tau: 1e-3,
            lr: 1e-4,
            update_every: 100,
            device: "cuda".to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Transition {
    state: Vec<f32>,
    action: usize,
    reward: f32,
    next_state: Vec<f32>,
    done: bool,
}

pub struct RLOptimizer {
    config: RLOptimizerConfig,
    policy_net: nn::Sequential,
    target_net: nn::Sequential,
    optimizer: tch::nn::Optimizer,
    replay_buffer: VecDeque<Transition>,
    step_count: usize,
    device: Device,
}

impl RLOptimizer {
    pub fn new(config: RLOptimizerConfig) -> Result<Self, RLOptimizerError> {
        let device = if config.device == "cuda" && tch::Cuda::is_available() {
            Device::Cuda(0)
        } else if config.device == "cuda" {
            warn!("CUDA not available, falling back to CPU");
            Device::Cpu
        } else {
            Device::Cpu
        };

        let vs = nn::VarStore::new(device);
        
        // Define policy network
        let policy_net = nn::seq()
            .add(nn::linear(
                &vs.root() / "fc1",
                config.state_dim as i64,
                256,
                Default::default(),
            ))
            .add_fn(|x| x.relu())
            .add(nn::linear(
                &vs.root() / "fc2",
                256,
                128,
                Default::default(),
            ))
            .add_fn(|x| x.relu())
            .add(nn::linear(
                &vs.root() / "out",
                128,
                config.action_dim as i64,
                Default::default(),
            ));

        // Clone for target network
        let target_net = policy_net.deep_clone();
        
        // Create optimizer
        let mut optimizer = nn::Adam::default().build(&vs, config.lr)
            .map_err(|e| RLOptimizerError::ModelError(e.to_string()))?;

        Ok(Self {
            config,
            policy_net,
            target_net,
            optimizer,
            replay_buffer: VecDeque::with_capacity(10000),
            step_count: 0,
            device,
        })
    }

    pub fn select_action(&self, state: &[f32], epsilon: f32) -> Result<usize, RLOptimizerError> {
        let mut rng = rand::thread_rng();
        
        // Epsilon-greedy action selection
        if rng.gen::<f32>() < epsilon {
            return Ok(rng.gen_range(0..self.config.action_dim));
        }
        
        // Convert state to tensor
        let state_tensor = Tensor::of_slice(state)
            .to(self.device)
            .to_kind(Kind::Float);
            
        // Forward pass
        let q_values = self.policy_net.forward_t(&state_tensor, false);
        
        // Get action with highest Q-value
        match q_values.argmax(-1, false).try_into() {
            Ok(action) => Ok(action),
            Err(_) => Err(RLOptimizerError::ModelError("Failed to select action".to_string())),
        }
    }

    pub fn store_transition(&mut self, transition: Transition) {
        if self.replay_buffer.len() >= self.config.memory_capacity {
            self.replay_buffer.pop_front();
        }
        self.replay_buffer.push_back(transition);
    }

    pub fn optimize(&mut self) -> Result<f32, RLOptimizerError> {
        if self.replay_buffer.len() < self.config.batch_size {
            return Ok(0.0);
        }

        // Sample batch from replay buffer
        let mut rng = rand::thread_rng();
        let batch: Vec<_> = self.replay_buffer
            .iter()
            .choose_multiple(&mut rng, self.config.batch_size)
            .cloned()
            .collect();

        // Prepare batch tensors
        let states: Vec<f32> = batch.iter().flat_map(|t| t.state.clone()).collect();
        let actions: Vec<i64> = batch.iter().map(|t| t.action as i64).collect();
        let rewards: Vec<f32> = batch.iter().map(|t| t.reward).collect();
        let next_states: Vec<f32> = batch.iter().flat_map(|t| t.next_state.clone()).collect();
        let dones: Vec<f32> = batch.iter().map(|t| if t.done { 0.0 } else { 1.0 }).collect();

        // Convert to tensors
        let states_tensor = Tensor::of_slice(&states)
            .view([self.config.batch_size as i64, self.config.state_dim as i64])
            .to(self.device)
            .to_kind(Kind::Float);
            
        let actions_tensor = Tensor::of_slice(&actions)
            .to(self.device);
            
        let rewards_tensor = Tensor::of_slice(&rewards)
            .to(self.device)
            .to_kind(Kind::Float);
            
        let next_states_tensor = Tensor::of_slice(&next_states)
            .view([self.config.batch_size as i64, self.config.state_dim as i64])
            .to(self.device)
            .to_kind(Kind::Float);
            
        let dones_tensor = Tensor::of_slice(&dones)
            .to(self.device)
            .to_kind(Kind::Float);

        // Compute Q(s_t, a) - the model computes Q(s_t), then we select the columns of actions taken
        let state_action_values = self.policy_net.forward_t(&states_tensor, false)
            .gather(1, &actions_tensor.unsqueeze(-1), false)
            .squeeze();

        // Compute V(s_{t+1}) for all next states
        let next_state_values = self.target_net.forward_t(&next_states_tensor, false)
            .max_dim(1, false)
            .0
            .detach();
            
        // Compute the expected Q values
        let expected_state_action_values = (next_state_values * dones_tensor) * self.config.gamma + rewards_tensor;

        // Compute Huber loss
        let loss = nn::smooth_l1_loss(
            &state_action_values,
            &expected_state_action_values,
            tch::Reduction::Mean,
            1.0,
        );

        // Optimize the model
        self.optimizer.zero_grad();
        loss.backward();
        
        // Clip gradients
        for param in self.optimizer.variables() {
            let _ = param.grad().clamp_(-1.0, 1.0);
        }
        
        self.optimizer.step();

        // Update target network
        if self.step_count % self.config.update_every == 0 {
            self.update_target_network();
        }
        
        self.step_count += 1;
        
        Ok(loss.into())
    }

    fn update_target_network(&mut self) {
        // Soft update of the target network's weights
        // θ′ ← τθ + (1 −τ)θ′
        let policy_vars = self.policy_net.variables();
        let target_vars = self.target_net.variables();
        
        for (i, param) in policy_vars.iter().enumerate() {
            let target_param = &target_vars[i];
            let new_value = param.data() * self.config.tau + target_param.data() * (1.0 - self.config.tau);
            target_param.copy_(&new_value);
        }
    }

    pub fn save(&self, path: &str) -> Result<(), RLOptimizerError> {
        self.policy_net.save(path)
            .map_err(|e| RLOptimizerError::ModelError(e.to_string()))
    }

    pub fn load(&mut self, path: &str) -> Result<(), RLOptimizerError> {
        self.policy_net.load(path)
            .map_err(|e| RLOptimizerError::ModelError(e.to_string()))?;
        self.target_net = self.policy_net.deep_clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rl_optimizer_creation() {
        let config = RLOptimizerConfig::default();
        let optimizer = RLOptimizer::new(config);
        assert!(optimizer.is_ok());
    }
    
    #[test]
    fn test_action_selection() {
        let config = RLOptimizerConfig::default();
        let optimizer = RLOptimizer::new(config).unwrap();
        let state = vec![0.0; 128];
        let action = optimizer.select_action(&state, 0.1).unwrap();
        assert!(action < 8);
    }
}
