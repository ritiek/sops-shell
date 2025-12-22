use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

pub fn sops_decrypt(filepath: &Path) -> Result<String> {
    run_sops_command(vec!["--decrypt", &filepath.to_string_lossy()])
}

pub fn sops_set(filepath: &Path, key: &str, value: &str) -> Result<()> {
    let json_value = format_value_for_sops(value)?;
    let path = if filepath.extension().and_then(|s| s.to_str()) == Some("ini") {
        // For ini files, assume keys are in [config] section
        &format!(r#"["config"]["{}"] {}"#, key, json_value)
    } else {
        // For other formats, use direct key path
        &format!(r#"["{}"] {}"#, key, json_value)
    };

    run_sops_command(vec![
        "--set",
        path,
        &filepath.to_string_lossy()
    ])?;
    Ok(())
}

fn run_sops_command(args: Vec<&str>) -> Result<String> {
    if Command::new("sops").arg("--version").output().is_err() {
        return Err(anyhow!("SOPS command not found. Please install SOPS or ensure it's in PATH"));
    }

    let output = Command::new("sops")
        .args(args)
        .output()
        .context("Failed to execute sops command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("SOPS command failed: {}", stderr));
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn format_value_for_sops(value: &str) -> Result<String> {
    match serde_json::from_str::<Value>(value) {
        Ok(parsed) => Ok(json!(parsed).to_string()),
        Err(_) => Ok(json!(value).to_string()),
    }
}
