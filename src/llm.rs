use serde::{Deserialize, Serialize};
use std::io::Write;

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

#[derive(Deserialize, Default)]
struct StreamMessage {
    #[serde(default)]
    content: String,
    #[serde(default)]
    thinking: String,
}

#[derive(Deserialize)]
struct StreamAnswer {
    #[serde(default)]
    message: StreamMessage,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    error: Option<String>,
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
        stream: true,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                anyhow::anyhow!(
                    "Could not connect to Ollama at http://localhost:11434\n  \
                     Make sure Ollama is running: ollama serve"
                )
            } else {
                anyhow::anyhow!("Request failed: {}", e)
            }
        })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let details = body.trim();
        if details.is_empty() {
            return Err(anyhow::anyhow!("Ollama returned an error: HTTP {}", status));
        }
        return Err(anyhow::anyhow!(
            "Ollama returned an error: HTTP {} - {}",
            status,
            details
        ));
    }

    let mut response = response;

    let mut streamed_message = String::new();
    let mut pending = Vec::<u8>::new();
    let mut hinted_generation = false;

    // Parse NDJSON lines and print each token immediately
    while let Some(chunk) = response.chunk().await? {
        pending.extend_from_slice(&chunk);

        while let Some(pos) = pending.iter().position(|b| *b == b'\n') {
            let mut line = pending.drain(..=pos).collect::<Vec<u8>>();

            // Remove line endings
            if matches!(line.last(), Some(b'\n')) {
                line.pop();
            }
            if matches!(line.last(), Some(b'\r')) {
                line.pop();
            }
            if line.is_empty() {
                continue;
            }

            let text = std::str::from_utf8(&line)
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in stream: {}", e))?;
            let answer: StreamAnswer = serde_json::from_str(text)
                .map_err(|e| anyhow::anyhow!("Invalid JSON in stream: {}", e))?;

            if let Some(error) = answer.error {
                return Err(anyhow::anyhow!("Ollama stream error: {}", error));
            }

            if !answer.message.content.is_empty() {
                print!("{}", answer.message.content);
                std::io::stdout().flush()?;
                streamed_message.push_str(&answer.message.content);
            } else if !answer.message.thinking.is_empty() && !hinted_generation {
                // On models that emit long reasoning first, print a minimal hint
                // so the terminal never appears frozen.
                print!("...");
                std::io::stdout().flush()?;
                hinted_generation = true;
            }

            if answer.done {
                break;
            }
        }
    }

    Ok(CommitResult {
        message: streamed_message.trim().to_string(),
        truncated,
        original_bytes,
        used_bytes,
    })
}
