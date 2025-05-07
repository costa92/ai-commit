use clap::Parser;
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "ai-commit",
    version,
    about = "Generate commit messages using Ollama or Deepseek"
)]
struct Args {
    /// AI provider to use (ollama or deepseek)
    #[arg(long, default_value = "")] // 空字符串表示未指定
    provider: String,

    /// Model to use (default: mistral)
    #[arg(short, long, default_value = "")] // 空字符串表示未指定
    model: String,

    /// 不自动执行 git add .
    #[arg(long, default_value_t = false)]
    no_add: bool,

    /// commit 后是否自动 push
    #[arg(long, default_value_t = false)]
    push: bool,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Serialize)]
struct DeepseekRequest<'a> {
    model: &'a str,
    messages: Vec<DeepseekMessage<'a>>,
    stream: bool,
}

#[derive(Serialize)]
struct DeepseekMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

#[derive(Deserialize)]
struct DeepseekResponse {
    choices: Vec<DeepseekChoice>,
}

#[derive(Deserialize)]
struct DeepseekChoice {
    message: DeepseekChoiceMessage,
}

#[derive(Deserialize)]
struct DeepseekChoiceMessage {
    content: String,
}

fn get_git_diff() -> String {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .expect("Failed to run git diff");

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_git_commit(message: &str) {
    Command::new("git")
        .args(["commit", "-m", message])
        .status()
        .expect("Git commit failed");
}

fn git_add_all() {
    Command::new("git")
        .args(["add", "."])
        .status()
        .expect("Git add failed");
}

fn git_push() {
    Command::new("git")
        .args(["push"])
        .status()
        .expect("Git push failed");
}

async fn generate_commit_message(
    diff: &str,
    provider: &str,
    model: &str,
) -> anyhow::Result<String> {
    // 优先从环境变量读取提示词文件路径，否则用默认
    let prompt_path =
        env::var("AI_COMMIT_PROMPT_PATH").unwrap_or_else(|_| "commit-prompt.txt".to_string());
    let prompt_template = fs::read_to_string(&prompt_path)
        .unwrap_or_else(|_| panic!("无法读取提示词文件: {}", prompt_path));
    let prompt = prompt_template.replace("{{git_diff}}", diff);
    let client = Client::new();

    match provider {
        "deepseek" => {
            let url = env::var("AI_COMMIT_DEEPSEEK_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com/v1/chat/completions".to_string());
            let api_key = env::var("AI_COMMIT_DEEPSEEK_API_KEY").unwrap_or_default();
            let request = DeepseekRequest {
                model,
                messages: vec![DeepseekMessage {
                    role: "user",
                    content: &prompt,
                }],
                stream: false,
            };
            let res = client
                .post(url)
                .bearer_auth(api_key)
                .json(&request)
                .send()
                .await?;
            let body: DeepseekResponse = res.json().await?;
            let content = body
                .choices
                .get(0)
                .map(|c| c.message.content.trim())
                .unwrap_or("");
            Ok(content.to_string())
        }
        _ => {
            // 默认 ollama
            let url = env::var("AI_COMMIT_OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string());
            let request = OllamaRequest {
                model,
                prompt: &prompt,
                stream: false,
            };
            let res = client.post(url).json(&request).send().await?;
            let body: OllamaResponse = res.json().await?;
            Ok(body.response.trim().to_string())
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok(); // 加载 .env 文件
    let args = Args::parse();

    // provider 优先级：命令行 > 环境变量 > 默认值 ollama
    let provider = if !args.provider.is_empty() {
        args.provider.clone()
    } else if let Ok(env_provider) = env::var("AI_COMMIT_PROVIDER") {
        env_provider
    } else {
        "ollama".to_string()
    };

    // model 优先级：命令行 > 环境变量 > 默认值 mistral
    let model = if !args.model.is_empty() {
        args.model.clone()
    } else if let Ok(env_model) = env::var("AI_COMMIT_MODEL") {
        env_model
    } else {
        "mistral".to_string()
    };

    // 如果需要，先执行 git add .
    if !args.no_add {
        git_add_all();
    }

    let diff = get_git_diff();
    if diff.trim().is_empty() {
        println!("No staged changes.");
        return Ok(());
    }

    let message = generate_commit_message(&diff, &provider, &model).await?;
    println!("Suggested commit message:\n\n{}\n", message);

    run_git_commit(&message);
    if args.push {
        git_push();
    }

    Ok(())
}
