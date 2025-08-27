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


use llm_rs::models::Model;
use llm_rs::models::ModelConfig;

pub struct Model {
    pub model: Model,
    pub config: ModelConfig,
}


impl Model {
    pub fn new(model: Model, config: ModelConfig) -> Self {
        Model { model, config }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model: String,
    pub config: ModelConfig,
}

pub fn model_config(model: String, config: ModelConfig) -> ModelConfig {
    ModelConfig { model, config }
}

pub fn model(model: Model, config: ModelConfig) -> Model {
    Model { model, config }
}




