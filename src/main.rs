use crate::llm::generate_commit_message;
use clap::Parser;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let files = git::get_staged_diff_name_only()?;
    if files.is_empty() {
        println!("No files in staging area...");
        return Ok(());
    }
    if args.split {
        for filename in files {
            let diff = git::get_staged_diff_for_file(&filename)?;
            println!("--- File: {} ---", filename);
            let message = generate_commit_message(diff, args.temp).await?;
            println!("Suggestion: {}\n", message);
        }
    } else {
        let diff = git::get_staged_diff()?;
        let result = generate_commit_message(diff, args.temp).await?;
        println!("{}", result);
    }
    Ok(())
}
