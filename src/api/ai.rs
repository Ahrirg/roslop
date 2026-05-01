use serde_json::{json, Value};

pub async fn ask_ai(api_key: String, question: String, filetree: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemma-4-31b-it:generateContent?key={}",
        // "https://generativelanguage.googleapis.com/v1beta/models/ListModels?key={}",
        api_key
    );

    let system_prompt = "You are a roblox coding assistant. Use the provided file tree to contextalize your answers. Respond only with word OVERRIDE (first line) then filename (second line) and code for the whole file. You need to override one script";
    let body = json!({
        "system_instruction": {
            "parts": { "text": system_prompt }
        },
        "contents": [{
            "parts": [
                { "text": format!("Filesystem tree:\n{}", filetree) },
                { "text": format!("Question: {}", question) }
            ]
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

    let text = res["candidates"][0]["content"]["parts"][1]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(text.replace("\n", "<|NL|>"))
}