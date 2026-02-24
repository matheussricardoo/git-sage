use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct Request {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    stream: bool,
}

#[derive(Deserialize)]
struct Answer {
    message: Message,
}

pub async fn generate_commit_message(diff: String) -> anyhow::Result<String> {
    let prompt = include_str!("instructions.txt");
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: prompt.to_string(),
        },
        Message {
            role: "user".to_string(),
            content: diff,
        },
    ];

    let request = Request {
        model: "llama3.2".to_string(),
        messages: messages,
        temperature: 0.0,
        stream: false,
    };
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&request)
        .send()
        .await?;

    let answer: Answer = response.json().await?;

    Ok(answer.message.content)
}
