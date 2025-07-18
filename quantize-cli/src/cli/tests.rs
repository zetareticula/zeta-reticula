use super::*;
use std::path::PathBuf;

#[test]
fn test_quantize_command_parsing() {
    let args = ["quantize-cli", "quantize", "--input", "model.gguf", "--output", "output.gguf", "--bits", "4", "--use-salience"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    
    let cli = Cli::parse_from(args);
    
    assert!(matches!(cli.command, Commands::Quantize(args) => {
        args.input == PathBuf::from("model.gguf") &&
        args.output == PathBuf::from("output.gguf") &&
        args.bits == 4 &&
        args.use_salience
    }));
}

#[test]
fn test_infer_command_parsing() {
    let args = ["quantize-cli", "infer", "--model", "model.gguf", "--input", "Hello world", "--max-tokens", "50", "--use-ns-router"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    
    let cli = Cli::parse_from(args);
    
    if let Commands::Infer(args) = cli.command {
        assert_eq!(args.model, PathBuf::from("model.gguf"));
        assert_eq!(args.input, "Hello world");
        assert_eq!(args.max_tokens, 50);
        assert!(args.use_ns_router);
    } else {
        panic!("Expected Infer command");
    }
}

#[test]
fn test_optimize_command_parsing() {
    let args = ["quantize-cli", "optimize", "--model", "model.gguf", "--output", "optimized.gguf", "--use-kv-cache"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    
    let cli = Cli::parse_from(args);
    
    if let Commands::Optimize(args) = cli.command {
        assert_eq!(args.model, PathBuf::from("model.gguf"));
        assert_eq!(args.output, PathBuf::from("optimized.gguf"));
        assert!(args.use_kv_cache);
    } else {
        panic!("Expected Optimize command");
    }
}

#[test]
fn test_convert_command_parsing() {
    let args = ["quantize-cli", "convert", "--input", "model.bin", "--output", "model.gguf", "--format", "gguf"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    
    let cli = Cli::parse_from(args);
    
    if let Commands::Convert(args) = cli.command {
        assert_eq!(args.input, PathBuf::from("model.bin"));
        assert_eq!(args.output, PathBuf::from("model.gguf"));
        assert_eq!(args.format, "gguf");
    } else {
        panic!("Expected Convert command");
    }
}

#[test]
fn test_global_flags() {
    let args = ["quantize-cli", "--verbose", "--format", "yaml", "quantize", "--input", "model.gguf", "--output", "output.gguf"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    
    let cli = Cli::parse_from(args);
    
    assert!(cli.verbose);
    assert_eq!(cli.format, "yaml");
    assert!(matches!(cli.command, Commands::Quantize(_)));
}
