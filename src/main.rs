use ai_commit::ai;
use ai_commit::ai::prompt;
use ai_commit::cli::args::Args;
use ai_commit::config::Config;
use ai_commit::git;
use clap::Parser;
use std::time::Instant;

async fn handle_tag_creation(args: &Args, _config: &Config, diff: &str) -> anyhow::Result<()> {
    // 先生成下一个 tag 名字
    let tag_name = git::get_next_tag_name(args.new_tag.as_deref()).await?;
    // note 优先用 tag_note，否则用 tag_name
    let note = if !args.tag_note.is_empty() {
        args.tag_note.clone()
    } else {
        tag_name.clone()
    };

    if !diff.trim().is_empty() {
        git::git_commit(&note).await?;
    } else {
        git::git_commit_allow_empty(&note).await?;
    }

    // 创建 tag，tag 名和 note 都用上面生成的
    git::create_tag_with_note(&tag_name, &note).await?;

    println!("Created new tag: {}", &tag_name);
    if args.push {
        git::push_tag(&tag_name, args.push_branches).await?;
        println!("Pushed tag {} to remote", &tag_name);
    }
    Ok(())
}

async fn handle_commit(args: &Args, config: &Config, diff: &str) -> anyhow::Result<()> {
    let prompt = prompt::get_prompt(diff);
    let start_time = Instant::now();
    let message = ai::generate_commit_message(diff, config, &prompt).await?;
    let elapsed_time = start_time.elapsed();
    println!("AI 生成 commit message 耗时: {:.2?}", elapsed_time);

    if elapsed_time.as_secs() > 30 {
        println!("警告: AI 模型 '{}' 生成 commit message 耗时较长，建议更换更快的模型或优化网络环境。", config.model);
    }

    if message.is_empty() {
        eprintln!("AI 生成 commit message 为空，请检查 AI 服务。");
        std::process::exit(1);
    }

    git::git_commit(&message).await?;
    if args.push {
        git::git_push().await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut config = Config::new();

    config.update_from_args(&args);
    config.validate()?;
    // 显示最新 tag
    if args.show_tag {
        if let Some((tag, note)) = git::get_latest_tag().await {
            println!("Latest tag: {}", tag);
            println!("Tag note: {}", note);
        } else {
            println!("No tags found in the repository");
        }
        return Ok(());
    }

    // git add
    if !args.no_add {
        git::git_add_all().await?;
    }

    let diff = git::get_git_diff().await?;

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
// 测试大文件修改场景
// 验证逻辑测试
