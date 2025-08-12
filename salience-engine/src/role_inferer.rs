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
