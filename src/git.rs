use std::process::Command;

pub fn get_staged_diff() -> anyhow::Result<String> {
    let output = Command::new("git").args(["diff", "--staged"]).output()?;
    let diff = String::from_utf8(output.stdout)?;
    Ok(diff)
}
