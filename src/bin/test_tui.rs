use ai_commit::tui_enhanced;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Testing Git commits loading...");
    
    // 直接测试 App 的创建
    match tui_enhanced::test_git_commits_loading().await {
        Ok(_) => println!("Git commits loaded successfully!"),
        Err(e) => println!("Error loading git commits: {}", e),
    }
    
    Ok(())
}