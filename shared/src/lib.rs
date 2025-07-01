//! Shared types and utilities for the zeta-reticula project.

pub mod quantization;
pub use quantization::{QuantizationResult, PrecisionLevel};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
