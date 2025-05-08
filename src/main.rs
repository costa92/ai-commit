mod ai;
mod args;
mod git;
mod prompt;

use args::Args;
use clap::Parser;
use dotenvy;
use std::path::PathBuf;

fn load_env() {
    let home_env = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let user_env_path = PathBuf::from(format!("{}/.ai-commit/.env", home_env));
    if user_env_path.exists() {
        dotenvy::from_path(user_env_path).ok();
    } else {
        dotenvy::dotenv().ok();
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_env();
    let args = Args::parse();

    // 如果需要创建新的 tag
    if std::env::args().any(|arg| arg == "--new-tag") {
        let new_tag = if let Some(ref ver) = args.new_tag {
            if !ver.is_empty() {
                git::create_new_tag(Some(ver))?
            } else {
                git::create_new_tag(None)?
            }
        } else {
            git::create_new_tag(None)?
        };
        println!("Created new tag: {}", new_tag);
        if args.push {
            git::push_tag(&new_tag, args.push_branches)?;
            println!("Pushed tag {} to remote", new_tag);
        }
        return Ok(());
    }

    // 如果指定了显示 tag，则显示并退出
    if args.show_tag {
        if let Some((tag, note)) = git::get_latest_tag() {
            println!("Latest tag: {}", tag);
            println!("Tag note: {}", note);
        } else {
            println!("No tags found in the repository");
        }
        return Ok(());
    }

    // provider 优先级：命令行 > 环境变量 > 默认值 ollama
    let provider = if !args.provider.is_empty() {
        args.provider.clone()
    } else if let Ok(env_provider) = std::env::var("AI_COMMIT_PROVIDER") {
        env_provider
    } else {
        "ollama".to_string()
    };

    // model 优先级：命令行 > 环境变量 > 默认值 mistral
    let model = if !args.model.is_empty() {
        args.model.clone()
    } else if let Ok(env_model) = std::env::var("AI_COMMIT_MODEL") {
        env_model
    } else {
        "mistral".to_string()
    };

    // 如果需要，先执行 git add .
    if !args.no_add {
        git::git_add_all();
    }

    let diff = git::get_git_diff();
    if diff.trim().is_empty() {
        println!("No staged changes.");
        return Ok(());
    }

    let prompt = prompt::get_prompt(&diff);
    // println!("prompt: {}", prompt);
    let message = ai::generate_commit_message(&diff, &provider, &model, &prompt).await?;
    // println!("Suggested commit message:\n\n{}\n", message);

    git::git_commit(&message);
    if args.push {
        git::git_push();
    }

    Ok(())
}
