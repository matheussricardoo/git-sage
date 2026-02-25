use crate::llm::generate_commit_message;
use clap::Parser;
use inquire::{Select, Text};
use std::path::Path;
use tokio::process::Command;

mod git;
mod llm;

#[derive(Parser, Debug)]
#[command(author, version, about = "AI-powered git commit generator")]
struct Args {
    #[arg(short, long)]
    split: bool,

    #[arg(short, long, default_value_t = 0.0)]
    temp: f32,
}

async fn commit_workflow(
    message: String,
    filename: Option<&str>,
    with_cargo_lock: bool,
) -> anyhow::Result<bool> {
    let options = vec!["Yes", "No", "Edit"];
    let answer = Select::new("Commit this file?", options).prompt()?;
    match answer {
        "Yes" | "Edit" => {
            let final_message = if answer == "Edit" {
                Text::new("Edit message:")
                    .with_initial_value(&message)
                    .prompt()?
            } else {
                message
            };

            let mut cmd = Command::new("git");
            cmd.args(["commit", "-m", &final_message]);

            if let Some(file) = filename {
                cmd.arg("--").arg(file);
                if with_cargo_lock {
                    cmd.arg("Cargo.lock");
                }
            }

            let status = cmd.status().await?;

            if status.success() {
                println!("Successfully committed!");
                return Ok(true);
            } else {
                eprintln!("Failed to commit.");
            }
        }
        _ => println!("Skipping..."),
    }
    Ok(false)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let files = git::get_staged_diff_name_only()?;
    if files.is_empty() {
        println!("No files in staging area...");
        return Ok(());
    }

    if args.split {
        let mut cargo_lock_pending = git::is_cargo_lock_staged()?;

        for filename in files {
            let diff = git::get_staged_diff_for_file(&filename)?;
            println!("--- File: {} ---", filename);
            let stem = Path::new(&filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&filename);
            let message = generate_commit_message(diff, Some(stem), args.temp).await?;
            println!("Suggestion: {}\n", message);

            let committed = commit_workflow(message, Some(&filename), cargo_lock_pending).await?;
            if committed && cargo_lock_pending {
                cargo_lock_pending = false;
            }
        }
    } else {
        let diff = git::get_staged_diff()?;
        let result = generate_commit_message(diff, None, args.temp).await?;
        println!("General Suggestion: {}\n", result);
        commit_workflow(result, None, false).await?;
    }

    Ok(())
}
