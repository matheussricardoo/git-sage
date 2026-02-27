use anyhow::bail;
use std::process::Command;

/// Known lock files that should be attached to the first commit
/// but excluded from diff analysis.
const LOCK_FILES: &[&str] = &[
    "Cargo.lock",        // Rust
    "package-lock.json", // Node.js (npm)
    "yarn.lock",         // Node.js (yarn)
    "pnpm-lock.yaml",    // Node.js (pnpm)
    "poetry.lock",       // Python (poetry)
    "Pipfile.lock",      // Python (pipenv)
    "Gemfile.lock",      // Ruby
    "go.sum",            // Go
    "composer.lock",     // PHP
    "flake.lock",        // Nix
];

fn is_lock_file(filename: &str) -> bool {
    let base = std::path::Path::new(filename)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);
    LOCK_FILES.contains(&base)
}

pub fn get_staged_diff() -> anyhow::Result<String> {
    let excludes: Vec<String> = LOCK_FILES.iter().map(|f| format!(":!{}", f)).collect();
    let exclude_refs: Vec<&str> = excludes.iter().map(|s| s.as_str()).collect();

    let output = Command::new("git")
        .arg("diff")
        .arg("--staged")
        .arg("--")
        .arg(".")
        .args(&exclude_refs)
        .output()?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        bail!("Git error: {}", error_message.trim());
    }
    let diff = String::from_utf8(output.stdout)?;
    Ok(diff)
}

pub fn get_staged_diff_name_only() -> anyhow::Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--staged", "--name-only"])
        .output()?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        bail!("Git error: {}", error_message.trim());
    }
    let output_str = String::from_utf8(output.stdout)?;
    let files = output_str
        .lines()
        .filter(|line| !is_lock_file(line))
        .map(|s| s.to_string())
        .collect();

    Ok(files)
}

pub fn get_staged_diff_for_file(filename: &str) -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(["diff", "--staged", "--", filename])
        .output()?;
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        bail!("Git error file: {}", error_message.trim());
    }
    let diff = String::from_utf8(output.stdout)?;
    Ok(diff)
}

/// Returns the list of lock files that are currently staged.
pub fn get_staged_lock_files() -> anyhow::Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--staged", "--name-only"])
        .output()?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        bail!("Git error: {}", error_message.trim());
    }
    let output_str = String::from_utf8(output.stdout)?;
    let locks = output_str
        .lines()
        .filter(|line| is_lock_file(line))
        .map(|s| s.to_string())
        .collect();

    Ok(locks)
}
