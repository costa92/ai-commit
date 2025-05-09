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
    let mut config = Config::new();
    config.update_from_args(&args);
    config.validate()?;

    // 兼容 -t/--new-tag 无参和有参
    if matches!(args.new_tag, Some(_))
        || std::env::args().any(|arg| arg == "-t" || arg == "--new-tag")
    {
        // 1. 获取本次 diff
        let diff = git::get_git_diff();
        let prompt = prompt::get_prompt(&diff);
        let summary = ai::generate_commit_message(&diff, &config, &prompt).await?;
        // 校验 summary，防止 prompt 被用作 commit message
        if summary.contains("{{git_diff}}") || summary.contains("Conventional Commits") {
            eprintln!("AI 生成 commit message 失败，返回了提示词模板。请检查 AI 服务。");
            std::process::exit(1);
        }
        // 2. 生成并提交 AI 总结 commit
        git::git_commit(&summary);
        // 3. 创建 tag，tag note 也用 summary
        let new_tag = if let Some(ref ver) = args.new_tag {
            if !ver.is_empty() {
                git::create_new_tag_with_note(Some(ver), &summary)?
            } else {
                git::create_new_tag_with_note(None, &summary)?
            }
        } else {
            git::create_new_tag_with_note(None, &summary)?
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
    let message = ai::generate_commit_message(&diff, &config, &prompt).await?;
    // 校验 message，防止 prompt 被用作 commit message
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
