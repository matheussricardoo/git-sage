use crate::llm::{LlmConfig, generate_commit_message};
use clap::Parser;
use inquire::{Select, Text};
use owo_colors::OwoColorize;
use std::path::Path;
use tokio::process::Command;

mod git;
mod llm;

#[derive(Parser, Debug)]
#[command(author, version, about = "AI-powered git commit generator")]
struct Args {
    #[arg(short, long, help = "Generate one commit per staged file")]
    split: bool,

    #[arg(
        short,
        long,
        default_value_t = 0.0,
        help = "Model temperature (0.0 = deterministic, 1.0 = creative)"
    )]
    temp: f32,

    #[arg(
        long,
        default_value = "qwen2.5-coder:3b",
        help = "Ollama model to use for commit generation"
    )]
    model: String,

    #[arg(
        long,
        default_value_t = 4,
        help = "Number of CPU threads to use (set to your core count)"
    )]
    threads: u32,

    #[arg(
        long,
        default_value_t = 99,
        help = "Number of model layers to offload to GPU (0 = CPU only, 99 = all layers)"
    )]
    gpu: u32,

    #[arg(
        long,
        default_value_t = 2048,
        help = "Context window size in tokens (increase for large diffs, requires more VRAM)"
    )]
    ctx: u32,

    #[arg(
        long,
        help = "Prompt to push commits to remote after all commits are done"
    )]
    push: bool,
}

async fn git_push() -> anyhow::Result<()> {
    let answer = Select::new("Push commits to remote?", vec!["Yes", "No"]).prompt()?;
    if answer == "Yes" {
        let status = Command::new("git").arg("push").status().await?;
        if status.success() {
            println!("{}", "Successfully pushed!".green().bold());
        } else {
            eprintln!("{}", "Failed to push.".red().bold());
        }
    }
    Ok(())
}

async fn commit_workflow(
    message: String,
    filename: Option<&str>,
    lock_files: &[String],
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
                for lock in lock_files {
                    cmd.arg(lock);
                }
            }

            let status = cmd.status().await?;

            if status.success() {
                println!("{}", "Successfully committed!".green().bold());
                return Ok(true);
            } else {
                eprintln!("{}", "Failed to commit.".red().bold());
            }
        }
        _ => println!("{}", "Skipping...".dimmed()),
    }
    Ok(false)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let config = LlmConfig {
        model: args.model,
        num_gpu: args.gpu,
        num_thread: args.threads,
        num_ctx: args.ctx,
        temp: args.temp,
    };

    let files = git::get_staged_diff_name_only()?;
    if files.is_empty() {
        println!("{}", "No files in staging area...".yellow());
        return Ok(());
    }

    if args.split {
        // detect staged lock files generically so this works outside Rust projects too
        let lock_files = git::get_staged_lock_files()?;
        let mut lock_pending = !lock_files.is_empty();
        let mut any_committed = false;

        for filename in files {
            println!("\n{} {}", "---".dimmed(), filename.cyan().bold());

            let diff = git::get_staged_diff_for_file(&filename)?;
            let stem = Path::new(&filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&filename);

            let result = generate_commit_message(diff, Some(stem), &config).await?;

            if result.truncated {
                println!(
                    "{} Diff truncated ({}/{} bytes). Use {} for full analysis.",
                    "⚠".yellow(),
                    result.used_bytes.yellow(),
                    result.original_bytes.yellow(),
                    format!("--ctx {}", result.original_bytes / 3).cyan()
                );
            }

            println!("Suggestion: {}\n", result.message.green());

            let locks_to_attach = if lock_pending {
                lock_files.clone()
            } else {
                vec![]
            };
            let committed =
                commit_workflow(result.message, Some(&filename), &locks_to_attach).await?;

            if committed {
                any_committed = true;
                lock_pending = false;
            }
        }

        if any_committed && args.push {
            git_push().await?;
        }
    } else {
        let diff = git::get_staged_diff()?;
        let result = generate_commit_message(diff, None, &config).await?;

        if result.truncated {
            println!(
                "{} Diff truncated ({}/{} bytes). Use {} for full analysis.",
                "⚠".yellow(),
                result.used_bytes.yellow(),
                result.original_bytes.yellow(),
                format!("--ctx {}", result.original_bytes / 3).cyan()
            );
        }

        println!("General Suggestion: {}\n", result.message.green());
        let committed = commit_workflow(result.message, None, &[]).await?;
        if committed && args.push {
            git_push().await?;
        }
    }

    Ok(())
}
