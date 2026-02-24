use crate::cli::args::Args;
use crate::config::Config;
use crate::git::tag;

/// 处理所有 tag 相关命令
pub async fn handle_tag_commands(args: &Args, config: &Config) -> anyhow::Result<()> {
    if args.tag_list {
        list_tags(config).await?;
    }

    if let Some(tag_name) = &args.tag_delete {
        delete_tag(tag_name, config).await?;
    }

    if let Some(tag_name) = &args.tag_info {
        show_tag_info(tag_name, config).await?;
    }

    if let Some(comparison) = &args.tag_compare {
        compare_tags(comparison, config).await?;
    }

    Ok(())
}

/// 列出所有标签（增强版）
async fn list_tags(config: &Config) -> anyhow::Result<()> {
    let tag_list = tag::list_tags_formatted().await?;

    if tag_list.trim().is_empty() {
        println!("No tags found in this repository.");
        return Ok(());
    }

    println!("📋 Tags (sorted by version):");
    println!(
        "{:<20} {:<12} {:<50} {:<12}",
        "Tag", "Commit", "Message", "Date"
    );
    println!("{}", "─".repeat(100));

    for line in tag_list.lines() {
        let parts: Vec<&str> = line.trim().splitn(4, ' ').collect();
        if parts.len() >= 4 {
            let tag_name = parts[0];
            let commit = parts[1];
            let message = if parts[2].chars().count() > 47 {
                let truncated: String = parts[2].chars().take(47).collect();
                format!("{}...", truncated)
            } else {
                parts[2].to_string()
            };
            let date = parts[3];

            println!(
                "{:<20} {:<12} {:<50} {:<12}",
                tag_name, commit, message, date
            );
        }
    }

    if config.debug {
        println!("\nTotal tags found: {}", tag_list.lines().count());
    }

    Ok(())
}

/// 删除指定标签（本地和远程）
async fn delete_tag(tag_name: &str, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Attempting to delete tag: {}", tag_name);
    }

    if !tag::tag_exists(tag_name).await? {
        anyhow::bail!("Tag '{}' does not exist", tag_name);
    }

    tag::delete_tag_local(tag_name).await?;
    println!("✓ Deleted local tag: {}", tag_name);

    if tag::delete_tag_remote(tag_name).await? {
        println!("✓ Deleted remote tag: {}", tag_name);
    } else if config.debug {
        println!(
            "⚠ Warning: Failed to delete remote tag '{}' (it might not exist on remote)",
            tag_name
        );
    }

    Ok(())
}

/// 显示标签详细信息
async fn show_tag_info(tag_name: &str, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Showing info for tag: {}", tag_name);
    }

    if !tag::tag_exists(tag_name).await? {
        anyhow::bail!("Tag '{}' does not exist", tag_name);
    }

    let info = tag::show_tag_info(tag_name).await?;
    println!("📌 Tag Information: {}", tag_name);
    println!("{}", "─".repeat(50));
    println!("{}", info);

    if let Ok(Some(message)) = tag::get_tag_message(tag_name).await {
        println!("\n📝 Tag Message:");
        println!("{}", "─".repeat(50));
        println!("{}", message);
    }

    Ok(())
}

/// 比较两个标签之间的差异
async fn compare_tags(comparison: &str, config: &Config) -> anyhow::Result<()> {
    let parts: Vec<&str> = comparison.split("..").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid comparison format. Use: TAG1..TAG2");
    }

    let tag1 = parts[0].trim();
    let tag2 = parts[1].trim();

    if config.debug {
        println!("Comparing tags: {} -> {}", tag1, tag2);
    }

    // 检查两个标签是否都存在
    for t in [tag1, tag2] {
        if !tag::tag_exists(t).await? {
            anyhow::bail!("Tag '{}' does not exist", t);
        }
    }

    println!("🔍 Comparing {} → {}", tag1, tag2);
    println!("{}", "─".repeat(60));

    // 显示提交差异统计
    let stat_output = tag::compare_tags_stat(tag1, tag2).await?;
    if !stat_output.trim().is_empty() {
        println!("📊 Changes Summary:");
        println!("{}", stat_output);
    }

    // 显示提交日志
    let log_output = tag::compare_tags_log(tag1, tag2).await?;
    if !log_output.trim().is_empty() {
        println!("\n📝 Commits between {} and {}:", tag1, tag2);
        println!("{}", log_output);
    } else {
        println!("No commits found between {} and {}", tag1, tag2);
    }

    println!("\n💡 To see detailed file changes, run:");
    println!("   git diff {}..{}", tag1, tag2);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_list_tags_command_structure() {
        let config = Config::new();
        let result = list_tags(&config).await;

        match result {
            Ok(_) => {
                println!("List tags succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                println!(
                    "List tags failed (expected in non-git environment): {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_show_tag_info_command_structure() {
        let config = Config::new();
        let result = show_tag_info("nonexistent-tag", &config).await;

        // 应该失败，因为标签不存在
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("does not exist"));
    }

    #[tokio::test]
    async fn test_delete_tag_command_structure() {
        let config = Config::new();
        let result = delete_tag("nonexistent-tag", &config).await;

        // 应该失败，因为标签不存在
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("does not exist"));
    }

    #[tokio::test]
    async fn test_compare_tags_format_validation() {
        let config = Config::new();

        // 测试无效格式
        let result = compare_tags("invalid-format", &config).await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid comparison format"));

        // 测试正确格式但不存在的标签
        let result = compare_tags("tag1..tag2", &config).await;
        // 应该在检查标签存在性时失败
        if let Err(e) = result {
            println!("Expected failure for nonexistent tags: {}", e);
        }
    }

    #[test]
    fn test_tag_comparison_parsing() {
        let test_cases = vec![
            ("tag1..tag2", Some(("tag1", "tag2"))),
            ("v1.0.0..v1.1.0", Some(("v1.0.0", "v1.1.0"))),
            ("invalid", None),
            ("tag1...tag2", None), // 三个点不支持
            ("", None),
        ];

        for (input, expected) in test_cases {
            let parts: Vec<&str> = input.split("..").collect();
            let result = if parts.len() == 2 {
                Some((parts[0].trim(), parts[1].trim()))
            } else {
                None
            };

            assert_eq!(
                result, expected,
                "Input '{}' should parse to {:?}",
                input, expected
            );
        }
    }
}
