use serde::{Serialize, Deserialize};
use rand::distributions::{Distribution, Normal};

#[derive(Serialize, Deserialize)]
pub struct PrivacyGuard {
    epsilon: f32,
}

impl PrivacyGuard {
    pub fn new(epsilon: f32) -> Self {
        PrivacyGuard { epsilon }
    }

    pub fn add_noise(&self, data: &mut [f32]) {
        let normal = Normal::new(0.0, 1.0 / self.epsilon as f64);
        let mut rng = rand::thread_rng();
        for val in data.iter_mut() {
            *val += normal.sample(&mut rng) as f32;
        }
    }
}