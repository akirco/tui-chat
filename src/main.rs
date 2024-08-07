mod skin;
use dirs;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};

#[derive(Serialize, Clone, Debug)]
struct Prompt {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct RequestData<'a> {
    model: &'a str,
    messages: Vec<Prompt>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

#[derive(Deserialize)]
struct Delta {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Content,
}

#[derive(Deserialize)]
struct Content {
    content: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    sd_apikey: String,
    sd_apisecret: String,
}

async fn send_request(
    prompts: &Vec<Prompt>,
    api_url: &str,
    api_key: &str,
    api_secret: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let skin = skin::get_skin();
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}:{}", api_key, api_secret))?,
    );

    let request_data = RequestData {
        model: "generalv3.5",
        messages: prompts.clone(),
        temperature: 0.0,
        max_tokens: 4096,
        stream: true,
    };

    let mut response = client
        .post(api_url)
        .headers(headers)
        .json(&request_data)
        .send()
        .await?;

    let mut session_response = String::new();

    while let Some(chunk) = response.chunk().await? {
        if let Ok(text) = std::str::from_utf8(&chunk) {
            if text.starts_with("data: ") {
                let event_data = &text[6..];
                if event_data != "[DONE]" {
                    if let Ok(content) = serde_json::from_str::<Delta>(event_data) {
                        if let Some(delta_content) = content.choices[0].delta.content.clone() {
                            session_response.push_str(&delta_content);
                            print!("{}", skin.inline(&delta_content));
                            io::stdout().flush().unwrap();
                        }
                    }
                }
            }
        }
    }

    Ok(session_response)
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let skin = skin::get_skin();
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let config_path = home_dir.join(".config/scoop/config.json");

    if !config_path.exists() {
        println!("Please notice: {} is available!", config_path.display());
        return;
    }

    let config_data = fs::read_to_string(config_path).expect("Could not read config file");
    let config: Config = serde_json::from_str(&config_data).expect("Could not parse config file");

    let api_url = "https://spark-api-open.xf-yun.com/v1/chat/completions";
    let mut prompts: Vec<Prompt> = Vec::new();

    loop {
        print!("🐼: ");
        io::stdout().flush().unwrap();

        let mut prompt = String::new();
        io::stdin()
            .read_line(&mut prompt)
            .expect("Failed to read line");
        let prompt = prompt.trim();

        if prompt == "q" {
            break;
        } else if prompt == "cls" {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            continue;
        } else if prompt.is_empty() || prompt == "h" {
            println!("{}", skin.inline("\t **sparkdesk**"));
            println!("\t tips:");
            println!("\t ● q    quit prompt");
            println!("\t ● cls  clear host");
            println!("\t ● n    new conversation");
        } else if prompt == "n" {
            prompts.clear();
        } else {
            prompts.push(Prompt {
                role: "user".to_string(),
                content: prompt.to_string(),
            });

            if !prompts.is_empty() {
                print!("🤖: ");
                io::stdout().flush().unwrap();

                match send_request(&prompts, api_url, &config.sd_apikey, &config.sd_apisecret).await
                {
                    Ok(response) => {
                        prompts.push(Prompt {
                            role: "assistant".to_string(),
                            content: response,
                        });
                        println!();
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
    }
}
