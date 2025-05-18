use ai_commit::ai;
use ai_commit::ai::prompt;
use ai_commit::cli::args::Args;
use ai_commit::config::Config;
use ai_commit::git;
use clap::Parser;
use dotenvy;
use std::path::PathBuf;

fn load_env() {
    let home_env = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    print!("home_env: {}", home_env);
    let user_env_path = PathBuf::from(format!("{}/.ai-commit/.env", home_env));
    if user_env_path.exists() {
        dotenvy::from_path(user_env_path).ok();
    } else {
        dotenvy::dotenv().ok();
    }
}

async fn handle_tag_creation(args: &Args, _config: &Config, diff: &str) -> anyhow::Result<()> {
    // 先生成下一个 tag 名字
    let tag_name = git::get_next_tag_name(args.new_tag.as_deref())?;
    // note 优先用 tag_note，否则用 tag_name
    let note = if !args.tag_note.is_empty() {
        args.tag_note.clone()
    } else {
        tag_name.clone()
    };

    if !diff.trim().is_empty() {
        git::git_commit(&note);
    } else {
        git::git_commit_allow_empty(&note);
    }

    // 创建 tag，tag 名和 note 都用上面生成的
    let new_tag = git::create_new_tag_with_note(Some(&tag_name), &note)?;

    println!("Created new tag: {}", new_tag);
    if args.push {
        git::push_tag(&new_tag, args.push_branches)?;
        println!("Pushed tag {} to remote", new_tag);
    }
    Ok(())
}

async fn handle_commit(args: &Args, config: &Config, diff: &str) -> anyhow::Result<()> {
    let prompt = prompt::get_prompt(diff);
    let message = ai::generate_commit_message(diff, config, &prompt).await?;

    if message.contains("{{git_diff}}") || message.contains("Conventional Commits") {
        eprintln!("AI 生成 commit message 失败，返回了提示词模板。请检查 AI 服务。");
        std::process::exit(1);
    }

    git::git_commit(&message);
    if args.push {
        git::git_push();
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_env();
    let args = Args::parse();
    let mut config = Config::new();
    config.update_from_args(&args);
    config.validate()?;

    // 显示最新 tag
    if args.show_tag {
        if let Some((tag, note)) = git::get_latest_tag() {
            println!("Latest tag: {}", tag);
            println!("Tag note: {}", note);
        } else {
            println!("No tags found in the repository");
        }
        return Ok(());
    }

    // git add
    if !args.no_add {
        git::git_add_all();
    }

    let diff = git::get_git_diff();

    // 处理 tag 或 commit
    if matches!(args.new_tag, Some(_))
        || std::env::args().any(|arg| arg == "-t" || arg == "--new-tag")
    {
        // tag 流程允许 diff 为空
        handle_tag_creation(&args, &config, &diff).await?;
    } else {
        if diff.trim().is_empty() {
            println!("No staged changes.");
            return Ok(());
        }
        handle_commit(&args, &config, &diff).await?;
    }

    Ok(())
}
