use crate::llm::generate_commit_message;

mod git;
mod llm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let diff = git::get_staged_diff()?;
    if diff.is_empty() {
        println!("There are no files in the staging area. Did you forget to run ‘git add’?");
        return Ok(());
    }

    let result = generate_commit_message(diff).await?;
    println!("{}", result);
    Ok(())
}
