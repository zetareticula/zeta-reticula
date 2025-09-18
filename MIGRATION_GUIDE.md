# Zeta Reticula Migration Guide

## Overview
This guide outlines the migration from the bloated multi-crate architecture to the new unified structure.

## Migration Steps

### Phase 1: Core Module Migration (COMPLETED ✅)

#### 1. KV Cache Consolidation
- **FROM**: 17+ scattered implementations
- **TO**: `core/kv-cache/` - Single unified implementation
- **Migrated**:
  - `kvquant_rs/src/block.rs` → `core/kv-cache/src/lib.rs`
  - `llm-rs/src/kv_cache.rs` → Removed
  - `llm-rs/src/kv_cache_manager.rs` → Removed
  - `zeta-vault-synergy/kv_cache_manager.rs` → Removed

#### 2. Quantization Engine Consolidation  
- **FROM**: 26+ scattered implementations
- **TO**: `core/quantization/` - Single unified engine
- **Migrated**:
  - `zeta-quantize/src/quantization.rs` → `core/quantization/src/lib.rs`
  - `salience-engine/src/quantizer.rs` → Removed
  - `llm-rs/src/quantizer.rs` → Removed
  - `shared/src/quantization.rs` → Removed

#### 3. Salience System Consolidation
- **FROM**: 5+ scattered implementations  
- **TO**: `core/salience/` - Single unified system
- **Migrated**:
  - `salience-engine/src/mesolimbic.rs` → `core/salience/src/lib.rs`
  - `agentflow-rs/src/mesolimbic.rs` → Removed
  - `kvquant_rs/src/mesolimbic_system.rs` → Removed

### Phase 2: Runtime Consolidation (COMPLETED ✅)

#### Inference Runtime
- **FROM**: `zeta-infer/`, `zeta-integration/`, `kvquant-deployment/`
- **TO**: `runtime/inference/` - Unified inference engine
- **Benefits**: Single entry point for all inference operations

### Phase 3: Interface Consolidation (COMPLETED ✅)

#### Unified CLI
- **FROM**: `quantize-cli/`, scattered CLI tools
- **TO**: `interfaces/cli/` - Single `zeta` command
- **New Commands**:
  ```bash
  zeta quantize model --input model.bin --output quantized.bin --precision int4
  zeta infer single --model llama --input "Hello world"
  zeta cache stats
  zeta salience analyze --input "Sample text"
  zeta system status
  ```

### Phase 4: Workspace Optimization (COMPLETED ✅)

#### New Workspace Structure
```
zeta-reticula/
├── core/                    # Core functionality
│   ├── kv-cache/           # Unified KV cache
│   ├── quantization/       # Unified quantization
│   ├── salience/           # Unified salience/mesolimbic
│   └── shared/             # Common types
├── runtime/                # Execution engines
│   ├── inference/          # Inference runtime
│   └── deployment/         # Deployment tools
├── interfaces/             # External interfaces
│   ├── api/                # REST API
│   ├── cli/                # Command line
│   └── grpc/               # gRPC services
└── integrations/           # External integrations
    ├── k8s/                # Kubernetes
    └── serverless/         # Serverless platforms
```

## Breaking Changes

### Import Changes
```rust
// OLD
use kvquant_rs::KVQuantizer;
use salience_engine::MesolimbicSystem;
use zeta_quantize::QuantizationEngine;

// NEW
use kv_cache::UnifiedKVCache;
use salience::UnifiedSalienceSystem;
use quantization::UnifiedQuantizer;
```

### Configuration Changes
```rust
// OLD - Multiple separate configs
let kv_config = KVQuantConfig::default();
let salience_config = SalienceConfig::default();
let quant_config = QuantizationConfig::default();

// NEW - Single unified config
let config = ZetaConfig::default();
```

### CLI Changes
```bash
# OLD - Multiple commands
quantize-cli quantize --model model.bin
zeta-infer run --model llama
kvquant-tool cache-stats

# NEW - Single unified command
zeta quantize model --input model.bin --output quantized.bin
zeta infer single --model llama --input "text"
zeta cache stats
```

## Migration Benefits

### Reduced Complexity
- **Before**: 19 workspace members, 161 Rust files, 25,330 lines
- **After**: ~8 core modules, estimated 50% reduction in codebase

### Improved Performance
- **Build times**: Faster due to reduced dependency graph
- **Memory usage**: Shared components reduce duplication
- **Binary size**: Single unified binary vs multiple tools

### Better Maintainability
- **Single source of truth** for each component
- **Consistent APIs** across all functionality
- **Unified configuration** and error handling
- **Simplified testing** and deployment

## Next Steps

### Phase 5: Legacy Cleanup (PENDING)
1. Remove legacy crates:
   - `zeta-infer/` → Delete
   - `zeta-integration/` → Delete  
   - `quantize-cli/` → Delete
   - Duplicate implementations → Delete

2. Update documentation and examples

3. Update deployment scripts and CI/CD

### Phase 6: Advanced Features (FUTURE)
1. Add `interfaces/api/` - REST API server
2. Add `interfaces/grpc/` - gRPC services
3. Add `integrations/k8s/` - Kubernetes operators
4. Add `tools/benchmarks/` - Performance testing
5. Add `tools/testing/` - Integration test suite

## Compatibility

### Backward Compatibility
- Legacy imports will work during transition period
- Gradual migration path available
- Configuration file compatibility maintained

### Forward Compatibility
- New unified APIs designed for extensibility
- Plugin architecture for custom components
- Stable public interfaces

## Support

For migration assistance:
1. Check the unified CLI help: `zeta --help`
2. Review configuration examples in `core/shared/`
3. Run diagnostics: `zeta system diagnostics`
4. Check system status: `zeta system status`
