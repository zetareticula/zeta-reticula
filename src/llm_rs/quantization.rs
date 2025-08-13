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


//! Quantization support for LLM weights

use std::ops::{Add, Mul};
use ndarray::{Array2, ArrayView2};
use num_traits::{AsPrimitive, FromPrimitive, Zero};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuantizationError {
    #[error("Unsupported bit width: {0}")]
    UnsupportedBitWidth(u8),
    
    #[error("Dequantization error: {0}")]
    DequantizationError(String),
}

/// Quantization parameters for a tensor
#[derive(Debug, Clone)]
pub struct QuantizationParams<T> {
    pub scale: f32,
    pub zero_point: T,
    pub min: f32,
    pub max: f32,
}

impl<T> Default for QuantizationParams<T> 
where
    T: Default + FromPrimitive + Copy,
{
    fn default() -> Self {
        Self {
            scale: 1.0,
            zero_point: T::zero(),
            min: 0.0,
            max: 1.0,
        }
    }
}

/// Quantize a float tensor to integer representation
pub fn quantize<T, U>(
    input: &ArrayView2<f32>,
    bit_width: u8,
) -> Result<(Array2<T>, QuantizationParams<U>), QuantizationError>
where
    T: num_traits::int::PrimInt + num_traits::FromPrimitive + num_traits::ToPrimitive + Send + Sync + 'static,
    U: num_traits::int::PrimInt + num_traits::FromPrimitive + num_traits::ToPrimitive + Send + Sync + 'static,
    f32: AsPrimitive<T>,
    T: AsPrimitive<f32>,
    U: AsPrimitive<f32>,
    f32: AsPrimitive<U>,
{
    // Determine range based on bit width
    let max_val = match bit_width {
        8 => i8::MAX as f32,
        16 => i16::MAX as f32,
        32 => i32::MAX as f32,
        _ => return Err(QuantizationError::UnsupportedBitWidth(bit_width)),
    };
    
    let min_val = -max_val;
    
    // Find min/max in input
    let input_min = *input.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let input_max = *input.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    
    // Calculate scale and zero point
    let scale = (input_max - input_min) / (max_val - min_val);
    let zero_point = U::from_f32(-input_min / scale + min_val).unwrap_or_else(U::zero);
    
    // Quantize the tensor
    let quantized = input.mapv(|x| {
        let val = (x / scale + zero_point.as_()).round();
        T::from_f32(val).unwrap_or_else(|| {
            if val < 0.0 {
                T::min_value()
            } else {
                T::max_value()
            }
        })
    });
    
    let params = QuantizationParams {
        scale,
        zero_point,
        min: input_min,
        max: input_max,
    };
    
    Ok((quantized, params))
}

/// Dequantize an integer tensor back to float
pub fn dequantize<T, U>(
    input: &ArrayView2<T>,
    params: &QuantizationParams<U>,
) -> Array2<f32>
where
    T: num_traits::int::PrimInt + num_traits::ToPrimitive + Copy,
    U: num_traits::int::PrimInt + num_traits::ToPrimitive + Copy,
{
    input.mapv(|x| {
        let x_f32 = x.to_f32().unwrap_or(0.0);
        let zp_f32 = params.zero_point.to_f32().unwrap_or(0.0);
        (x_f32 - zp_f32) * params.scale
    })
}

/// Quantized matrix multiplication with dequantization
pub fn quantized_matmul<T, U>(
    a: &ArrayView2<T>,
    a_params: &QuantizationParams<U>,
    b: &ArrayView2<T>,
    b_params: &QuantizationParams<U>,
) -> Array2<f32>
where
    T: num_traits::int::PrimInt + num_traits::ToPrimitive + Copy + Send + Sync + 'static,
    U: num_traits::int::PrimInt + num_traits::ToPrimitive + Copy,
    f32: Add<Output = f32> + Mul<Output = f32> + Zero + Send + Sync + 'static,
{
    // Perform integer matrix multiplication
    let mut result = Array2::zeros((a.shape()[0], b.shape()[1]));
    
    // Convert to f32 and apply dequantization during multiplication
    for i in 0..a.shape()[0] {
        for j in 0..b.shape()[1] {
            let mut sum = 0.0;
            for k in 0..a.shape()[1] {
                let a_val = a[[i, k]].to_f32().unwrap_or(0.0) - a_params.zero_point.to_f32().unwrap_or(0.0);
                let b_val = b[[k, j]].to_f32().unwrap_or(0.0) - b_params.zero_point.to_f32().unwrap_or(0.0);
                sum += a_val * b_val;
            }
            result[[i, j]] = sum * a_params.scale * b_params.scale;
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_quantization() {
        let input = array![[1.0, 2.0], [3.0, 4.0]];
        let (quantized, params) = quantize::<i8, i8>(&input.view(), 8).unwrap();
        let dequantized = dequantize(&quantized.view(), &params);
        
        // Check if dequantized values are close to original
        for (orig, deq) in input.iter().zip(dequantized.iter()) {
            assert_relative_eq!(orig, deq, epsilon = 0.1);
        }
    }
    
    #[test]
    fn test_quantized_matmul() {
        let a = array![[1.0, 2.0], [3.0, 4.0]];
        let b = array![[5.0, 6.0], [7.0, 8.0]];
        
        let (a_quant, a_params) = quantize::<i8, i8>(&a.view(), 8).unwrap();
        let (b_quant, b_params) = quantize::<i8, i8>(&b.view(), 8).unwrap();
        
        let result = quantized_matmul(
            &a_quant.view(),
            &a_params,
            &b_quant.t().view(),  // Transpose b for matmul
            &b_params,
        );
        
        // Expected result: a * b^T
        let expected = a.dot(&b.t());
        
        // Check if results are close
        for (exp, res) in expected.iter().zip(result.iter()) {
            assert_relative_eq!(exp, res, epsilon = 0.5);
        }
    }
}
