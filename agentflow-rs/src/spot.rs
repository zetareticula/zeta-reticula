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


/// Bit precision modulation with mantissa
///
/// # Arguments
///
/// * `value` - The value to be modulated
/// * `precision` - The desired precision of the output
/// * `mantissa` - The maximum value of the mantissa
///
/// # Returns
///
/// The modulated value
pub fn modulate_bit_precision(value: f32, precision: u8, mantissa: u32) -> f32 {
    let (integer_part, decimal_part) = modf(value);
    let integer_part = integer_part as i32;
    let decimal_part = decimal_part.abs() * (10f32.powi(precision as i32) as f32);
    let mantissa = ((decimal_part * mantissa as f32) as u32) % mantissa as u32;
    let modulated_value = integer_part << precision as i32 | mantissa as i32;
    modulated_value as f32
}

use std::mem::transmute;

/// Returns the integer part of a number and the decimal part.
///
/// # Arguments
///
/// * `number` - The number to be decomposed.
///
/// # Returns
///
/// A tuple containing the integer part and the decimal part of the number.
pub fn modf(number: f32) -> (i32, f32) {
    let bits = unsafe { transmute::<f32, u32>(number) };
    let exponent = ((bits >> 23) & 0xff) as i32;
    let mantissa = if exponent == 0 {
        (bits & 0x7fffff) as i32
    } else {
        (bits & 0x7fffff) | 0x800000
    };
    let integer_part = (number / 2f32.powi(exponent - 23)).trunc() as i32;
    let decimal_part = f32::from_bits(mantissa << 23) - integer_part as f32 * 2f32.powi(exponent - 23);
    (integer_part, decimal_part)
}
