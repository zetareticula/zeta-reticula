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

use serde::{Serialize, Deserialize};

use crate::role_inference::RoleInferenceResult;
use crate::role_inference::TokenFeatures;

/// Trait for role inference in the salience engine
pub trait RoleInferer: Send + Sync + std::fmt::Debug {
    /// Infer roles for a set of token features
    fn infer_roles(&self, features: Vec<TokenFeatures>, theory_key: &str) -> Vec<RoleInferenceResult>;
}

/// A boxed trait object for dynamic dispatch of role inferers
pub type BoxedRoleInferer = Box<dyn RoleInferer + Send + Sync + 'static>;

/// Helper function to create a boxed role inferer
pub fn boxed_role_inferer<T: RoleInferer + 'static>(inferer: T) -> BoxedRoleInferer {
    Box::new(inferer)
}
