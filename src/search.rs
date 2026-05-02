use crate::app::CrateInfo;
use std::process::Command;

pub fn search(query: &str) -> Result<Vec<CrateInfo>, String> {
    let output = Command::new("cargo")
        .args(["search", query, "--limit", "30"])
        .output()
        .map_err(|e| format!("Failed to run cargo search: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cargo search failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_output(&stdout))
}

fn parse_output(output: &str) -> Vec<CrateInfo> {
    output.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<CrateInfo> {
    let (lhs, description) = line.split_once('#')?;
    let (name, version_part) = lhs.split_once('=')?;

    Some(CrateInfo {
        name: name.trim().to_string(),
        version: version_part.trim().trim_matches('"').to_string(),
        description: description.trim().to_string(),
    })
}
