use crate::app::{CrateDetail, Feature};
use std::process::Command;

pub fn get_info(name: &str) -> Result<CrateDetail, String> {
    let output = Command::new("cargo")
        .args(["info", name])
        .output()
        .map_err(|e| format!("Failed to run cargo info: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cargo info failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_info(name, &stdout)
}

fn parse_info(name: &str, output: &str) -> Result<CrateDetail, String> {
    let mut lines = output.lines();
    lines.next();
    let description = lines.next().unwrap_or("").trim().to_string();

    let mut version = String::new();
    let mut rust_version = None;
    let mut repository = None;
    let mut features: Vec<Feature> = Vec::new();
    let mut in_features = false;

    for line in lines {
        if line.starts_with("version: ") {
            version = line["version: ".len()..].trim().to_string();
            in_features = false;
        } else if line.starts_with("rust-version: ") {
            rust_version = Some(line["rust-version: ".len()..].trim().to_string());
            in_features = false;
        } else if line.starts_with("repository: ") {
            repository = Some(line["repository: ".len()..].trim().to_string());
            in_features = false;
        } else if line == "features:" {
            in_features = true;
        } else if line.starts_with("note:") {
            in_features = false;
        } else if in_features {
            if let Some(feature) = parse_feature(line) {
                features.push(feature);
            }
        }
    }

    Ok(CrateDetail {
        name: name.to_string(),
        version,
        description,
        rust_version,
        repository,
        features,
    })
}

fn parse_feature(line: &str) -> Option<Feature> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let enabled = trimmed.starts_with('+');
    let rest = if enabled { &trimmed[1..] } else { trimmed };

    let (name, deps) = match rest.split_once('=') {
        Some((n, d)) => (n.trim().to_string(), d.trim().to_string()),
        None => (rest.trim().to_string(), String::new()),
    };

    if name.is_empty() {
        return None;
    }

    Some(Feature {
        name,
        enabled,
        deps,
    })
}
