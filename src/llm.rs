use serde::{Deserialize, Serialize};
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

pub async fn generate_commit_message(
    diff: String,
    filename: Option<&str>,
    temp: f32,
) -> anyhow::Result<String> {
    let diff = if diff.len() > 6000 {
        &diff[..6000]
    } else {
        &diff
    };
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
        model: "qwen2.5-coder:3b".to_string(),
        messages,
        options: Options {
            temperature: temp,
            num_predict: 80,
            num_ctx: 2048,
            num_gpu: 99,
            num_thread: 4,
        },
        stream: false,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&request)
        .send()
        .await?;

    let answer: Answer = response.json().await?;

    Ok(answer.message.content.trim().to_string())
}
