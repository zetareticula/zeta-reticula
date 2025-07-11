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




