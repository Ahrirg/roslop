use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io::Write;

use crate::api::roblox::{AppState};

pub async fn ask_ai(api_key: String, question: String, filetree: String, system_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemma-4-31b-it:generateContent?key={}",
        // "https://generativelanguage.googleapis.com/v1beta/models/ListModels?key={}",
        api_key
    );

    let body = json!({
        "system_instruction": {
            "parts": { "text": system_prompt }
        },
        "contents": [{
            "parts": [
                { "text": format!("Filesystem tree:\n{}", filetree.replace("<|NL|>", "\n")) },
                { "text": format!("Question: {}", question.replace("<|NL|>", "\n")) }
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

    let text = res["candidates"][0]["content"]["parts"]
        .as_array()
        .map(|parts| {
            parts.iter()
                .filter(|p| p["thought"].as_bool() != Some(true))
                .filter_map(|p| p["text"].as_str())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();


    let log_entry = format!(
        "--- NEW REQUEST ---\nREQUEST BODY:\n{}\n\nRESPONSE BODY:\n{}\n--- END ---\n\n",
        serde_json::to_string_pretty(&body)?,
        serde_json::to_string_pretty(&res)?
    );

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("app.log")?;

    file.write_all(log_entry.as_bytes())?;


    Ok(text)
}


pub async fn full_ai_pipeline(api_key: String, question: String, filetree: String, state: &AppState) -> Result<String, Box<dyn std::error::Error>> {
    let system_prompt_get_file = "You are a path selection assistant. Your ONLY job is to identify the single correct file path from the tree based on the user's request. 

    STRICT OUTPUT FORMAT:
    Line 1: The word 'Content'
    Line 2: The full file path only (e.g., Folder.Subfolder.Script)

    DO NOT generate code. DO NOT provide explanations.";

    let system_prompt_override = "You are a roblox coding assistant. Use the provided file tree and the provided file contents to contextalize your answers. Respond only with word OVERRIDE (first line) then filename (second line) and fixed or new code for the whole file. You need to override one script";

    let response_path = ask_ai(api_key.clone(), question.clone(), filetree.clone(), system_prompt_get_file).await;
    match response_path {
        Ok( file_path_unparsed) => {
            let mut file_path = "";
            let mut lines = file_path_unparsed.lines();
            if let Some(first_line) = lines.next() {
                if first_line == "Content" {
                    if let Some(second_line) = lines.next() {
                        file_path = second_line;
                    }
                }
            }

            if file_path.is_empty() { 
                return Err(format!("No file was found: {}", file_path_unparsed).into());
            }

            let filecontent = state.send(format!("{} {}", "Content", file_path)).await.ok_or("Not content")?;
            let question_with_file_content = format!("{}\n\nFile contents:\n```{}\n{}\n```", question, file_path, filecontent);


            let response_new_contents = ask_ai(api_key.clone(), question_with_file_content, filetree.clone(), system_prompt_override).await;
            match  response_new_contents {
                Ok( updated_file_contents ) => {

                    Ok(updated_file_contents.replace("\n", "<|NL|>"))
                }
                Err(e) => {
                    eprintln!("Error did happen {}", e);
                    Err(e)
                }
            }
        },
        Err( e ) => {
            eprintln!("Error did happen {}", e);
            Err(e)
        }
    }
}