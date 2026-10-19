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
    let max_diff_bytes = (config.num_ctx as usize).saturating_mul(3); // ~3 bytes per token as a safe estimate
    let original_bytes = diff.len();

    let (diff, truncated) = truncate_diff(&diff, max_diff_bytes);

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
    let mut done = false;
    'stream: while let Some(chunk) = response.chunk().await? {
        pending.extend_from_slice(&chunk);
        while let Some(pos) = pending.iter().position(|b| *b == b'\n') {
            let line = pending.drain(..=pos).collect::<Vec<u8>>();
            let (token, finished) = parse_line(&line)?;
            print!("{}", token);
            std::io::stdout().flush()?;
            streamed_message.push_str(&token);
            if finished {
                done = true;
                break 'stream;
            }
        }
    }
    if !done && !pending.is_empty() {
        let (token, finished) = parse_line(&pending)?;
        print!("{}", token);
        std::io::stdout().flush()?;
        streamed_message.push_str(&token);
        done = finished;
    }
    anyhow::ensure!(
        done,
        "Ollama stream ended before completion; no commit was created"
    );
    anyhow::ensure!(
        !streamed_message.trim().is_empty(),
        "Ollama returned an empty message"
    );
    Ok(CommitResult {
        message: streamed_message.trim().to_string(),
        truncated,
        original_bytes,
        used_bytes,
    })
}

fn truncate_diff(diff: &str, max_bytes: usize) -> (&str, bool) {
    if diff.len() <= max_bytes {
        return (diff, false);
    }
    let mut end = max_bytes;
    while !diff.is_char_boundary(end) {
        end -= 1;
    }
    let end = diff[..end].rfind('\n').unwrap_or(end);
    (&diff[..end], true)
}

fn parse_line(line: &[u8]) -> anyhow::Result<(String, bool)> {
    if line.iter().all(u8::is_ascii_whitespace) {
        return Ok((String::new(), false));
    }
    let answer: StreamAnswer = serde_json::from_slice(line)?;
    if let Some(error) = answer.error {
        anyhow::bail!("Ollama stream error: {}", error);
    }
    Ok((answer.message.content, answer.done))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_preserves_utf8_at_every_boundary() {
        let diff = "é🙂 change";
        for limit in 0..diff.len() {
            let (text, truncated) = truncate_diff(diff, limit);
            assert!(truncated);
            assert!(text.len() <= limit);
            assert!(diff.starts_with(text));
        }
        assert_eq!(truncate_diff(diff, diff.len()), (diff, false));
    }

    #[test]
    fn truncation_prefers_complete_lines() {
        assert_eq!(truncate_diff("first\nsecond line", 10), ("first", true));
    }

    #[test]
    fn parses_final_line_without_newline() {
        assert_eq!(
            parse_line(br#"{"message":{"content":"fix: test"},"done":true}"#).unwrap(),
            ("fix: test".to_string(), true)
        );
    }

    #[test]
    fn rejects_upstream_error_and_invalid_json() {
        assert!(parse_line(br#"{"error":"missing model"}"#).is_err());
        assert!(parse_line(b"invalid").is_err());
    }
}
