mod ai;
mod args;
mod git;
mod prompt;

use args::Args;
use clap::Parser;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let args = Args::parse();

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
    let message = ai::generate_commit_message(&diff, &provider, &model, &prompt).await?;
    println!("Suggested commit message:\n\n{}\n", message);

    git::git_commit(&message);
    if args.push {
        git::git_push();
    }

    Ok(())
}
