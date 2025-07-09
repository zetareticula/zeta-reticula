// Minimal stub for YoungTableau to allow compilation. Replace with actual implementation as needed.
#[derive(Debug, Clone)]
pub struct YoungTableau {
    pub dimensions: usize,
    pub threshold: f32,
}

impl YoungTableau {
    pub fn new(dimensions: usize, threshold: f32) -> Self {
        YoungTableau { dimensions, threshold }
    }
}
