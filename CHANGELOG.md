# Changelog

All notable changes to this repository will be documented in this file.

## 2025-08-12

### Fix: Rust workspace compilation and visibility issues

- ns-router-rs
  - `router.rs`: Replace undefined `RoutingCache` with `Arc<RwLock<LruCache<...>>>` as `decision_cache`. Fix imports to use `salience_engine::role_inference::SalienceResult`. Close unclosed `impl NSRouter` block. Adjust `ModelConfig` construction to use `RouterConfig` fields. Remove stray/duplicate lines and fix mismatched braces.
  - `context.rs`: Remove invalid `let` bindings inside struct; clean unresolved imports (`tonic`, `fusion_anns`, etc.). Use concrete `NSContextAnalyzer` with `Arc`.
  - `salience.rs`: Import `SalienceResult` and `TokenFeatures` from `salience_engine::role_inference` instead of private `mesolimbic` module.
  - `symbolic.rs`: Import `shared::QuantizationResult`; use `score` instead of `salience_score`.
  - `rewrite_wrapper.rs`: Remove invalid serde field attribute on non-derive struct.

- salience-engine
  - `role_inference.rs`: Publicly re-export `RoleInferer` and ensure `RoleInfererImpl::sample_role` implemented.

- agentflow-rs
  - `privacy.rs`: Import `SalienceResult`/`RoleInferer` from `salience_engine::role_inference`.

### Build

- Add a root `.gitignore` to exclude build artifacts (e.g., `target/`, `.DS_Store`, logs, OpenBLAS temp outputs) and common local files.

### Notes

- Workspace build currently blocked by system OpenBLAS linking (`openblas-src`) on macOS. Proposed next step: gate BLAS-dependent crates behind optional features or configure system OpenBLAS via Homebrew.
