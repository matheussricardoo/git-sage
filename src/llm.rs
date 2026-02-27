use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct Options {
    temperature: f32,
    num_predict: u32,
    num_ctx: u32,
    num_gpu: u32,
    num_thread: u32,
}

#[derive(Serialize)]
struct Request {
    model: String,
    messages: Vec<Message>,
    options: Options,
    stream: bool,
}

#[derive(Deserialize)]
struct Answer {
    message: Message,
}

pub struct LlmConfig {
    pub model: String,
    pub num_gpu: u32,
    pub num_thread: u32,
    pub num_ctx: u32,
    pub temp: f32,
}

pub struct CommitResult {
    pub message: String,
    pub truncated: bool,
    pub original_bytes: usize,
    pub used_bytes: usize,
}

pub async fn generate_commit_message(
    diff: String,
    filename: Option<&str>,
    config: &LlmConfig,
) -> anyhow::Result<CommitResult> {
    let max_diff_bytes = (config.num_ctx * 3) as usize; // ~3 bytes per token as a safe estimate
    let original_bytes = diff.len();

    let (diff, truncated) = if diff.len() > max_diff_bytes {
        // truncate at the last newline before the limit
        // to avoid sending a malformed partial line to the model
        let boundary = diff[..max_diff_bytes]
            .rfind('\n')
            .unwrap_or(max_diff_bytes);
        (&diff[..boundary], true)
    } else {
        (diff.as_str(), false)
    };

    let used_bytes = diff.len();
    let prompt = include_str!("instructions.txt");

    let user_content = match filename {
        Some(name) => format!(
            "File being committed: {name}\n\
             The scope in your commit message MUST be exactly \"{name}\" — do not use the path or extension from the diff header.\n\n\
             Diff to analyze:\n{diff}"
        ),
        None => format!("Diff to analyze:\n{diff}"),
    };

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: prompt.to_string(),
        },
        Message {
            role: "user".to_string(),
            content: user_content,
        },
    ];

    let request = Request {
        model: config.model.clone(),
        messages,
        options: Options {
            temperature: config.temp,
            num_predict: 80,
            num_ctx: config.num_ctx,
            num_gpu: config.num_gpu,
            num_thread: config.num_thread,
        },
        stream: false,
    };

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} Generating commit message...")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(80));

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            spinner.finish_and_clear();
            if e.is_connect() {
                anyhow::anyhow!(
                    "Could not connect to Ollama at http://localhost:11434\n  \
                     Make sure Ollama is running: ollama serve"
                )
            } else {
                anyhow::anyhow!("Request failed: {}", e)
            }
        })?;

    let answer: Answer = response.json().await?;
    spinner.finish_and_clear();

    Ok(CommitResult {
        message: answer.message.content.trim().to_string(),
        truncated,
        original_bytes,
        used_bytes,
    })
}
