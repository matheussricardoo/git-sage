use anyhow::bail;
use std::process::Command;

pub fn get_staged_diff() -> anyhow::Result<String> {
    let output = Command::new("git").args(["diff", "--staged"]).output()?;
    let diff = String::from_utf8(output.stdout)?;
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        bail!("Git error: {}", error_message.trim());
    }
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
    let diff = String::from_utf8(output.stdout)?;
    let split_archive_names_vec = diff.lines().map(|s| s.to_string()).collect();

    Ok(split_archive_names_vec)
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
