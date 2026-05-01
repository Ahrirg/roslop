use serde_json::{json, Value};

pub async fn ask_ai(api_key: String, question: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemma-4-31b-it:generateContent?key={}",
        // "https://generativelanguage.googleapis.com/v1beta/models/ListModels?key={}",
        api_key
    );

    let body = json!({
        "contents": [{
            "parts": [{"text": question}]
        }]
    });

    let client = reqwest::Client::new();

    let res = client
        .post(url)
        .json(&body)
        .send()
        .await?
        .json::<Value>() 
        .await?;

    let text = res["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(text.replace("\n", "<|NL|>"))
}