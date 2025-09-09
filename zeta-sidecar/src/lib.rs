//! Zeta Sidecar - gRPC service for model serving and caching

#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]
#![doc(
    html_logo_url = "https://github.com/zetareticula/.github/raw/main/logo.png",
    html_favicon_url = "https://github.com/zetareticula/.github/raw/main/favicon.ico"
)]

pub mod pb;
pub mod service;
pub mod error;
pub mod config;

/// Re-exports for common types
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

/// Prelude module containing commonly used items
pub mod prelude {
    pub use crate::pb::{
        sidecar::{
            sidecar_service_server::SidecarService,
            CacheRequest, CacheResponse, CacheUpdate, UpdateResponse,
        },
        zeta::*,
        policy::*,
        kvquant::*,
        nsrouter::*,
        salience::*,
    };
    
    pub use crate::service::SidecarServer;
    pub use crate::config::Config;
    pub use crate::Error;
}
