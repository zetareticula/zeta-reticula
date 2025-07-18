use std::process::Command;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Zeta Reticula Model Quantization Tool"));
}

#[test]
fn test_cli_quantize_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("quantize")
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Quantize a model"));
}

#[test]
fn test_cli_infer_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("infer")
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Run inference with the model"));
}

#[test]
fn test_cli_optimize_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("optimize")
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Optimize a model"));
}

#[test]
fn test_cli_convert_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("convert")
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Convert between model formats"));
}
