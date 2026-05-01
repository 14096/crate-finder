use std::process::Command;

pub fn add_crate(
    name: &str,
    features: &[String],
    no_default_features: bool,
) -> Result<String, String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("add").arg(name);

    if no_default_features {
        cmd.arg("--no-default-features");
    }

    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run cargo add: {e}"))?;

    if output.status.success() {
        let msg = if features.is_empty() {
            format!("Added {name}")
        } else {
            format!("Added {name} with features: {}", features.join(", "))
        };
        Ok(msg)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}
