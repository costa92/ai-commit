use crate::cli::args::Args;
use crate::config::Config;
use tokio::process::Command;

/// 处理所有 tag 相关命令
pub async fn handle_tag_commands(args: &Args, config: &Config) -> anyhow::Result<()> {
    if args.tag_list {
        list_tags(config).await?;
    }

    if let Some(tag) = &args.tag_delete {
        delete_tag(tag, config).await?;
    }

    if let Some(tag) = &args.tag_info {
        show_tag_info(tag, config).await?;
    }

    if let Some(comparison) = &args.tag_compare {
        compare_tags(comparison, config).await?;
    }

    Ok(())
}

/// 列出所有标签（增强版）
async fn list_tags(config: &Config) -> anyhow::Result<()> {
    let output = Command::new("git")
        .args([
            "tag",
            "-l",
            "--sort=-version:refname",
            "--format=%(refname:short) %(objectname:short) %(subject) %(authordate:short)",
        ])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list tags: {}", e))?;

    if !output.status.success() {
        anyhow::bail!(
            "Git tag list failed with exit code: {:?}",
            output.status.code()
        );
    }

    let tag_list = String::from_utf8_lossy(&output.stdout);

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
            let tag = parts[0];
            let commit = parts[1];
            let message = if parts[2].chars().count() > 47 {
                let truncated: String = parts[2].chars().take(47).collect();
                format!("{}...", truncated)
            } else {
                parts[2].to_string()
            };
            let date = parts[3];

            println!("{:<20} {:<12} {:<50} {:<12}", tag, commit, message, date);
        }
    }

    if config.debug {
        println!("\nTotal tags found: {}", tag_list.lines().count());
    }

    Ok(())
}

/// 删除指定标签（本地和远程）
async fn delete_tag(tag: &str, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Attempting to delete tag: {}", tag);
    }

    // 首先检查标签是否存在
    let tag_exists = Command::new("git")
        .args(["tag", "-l", tag])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to check tag existence: {}", e))?;

    if tag_exists.stdout.is_empty() {
        anyhow::bail!("Tag '{}' does not exist", tag);
    }

    // 删除本地标签
    let status = Command::new("git")
        .args(["tag", "-d", tag])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to delete local tag: {}", e))?;

    if !status.success() {
        anyhow::bail!(
            "Failed to delete local tag '{}' with exit code: {:?}",
            tag,
            status.code()
        );
    }

    println!("✓ Deleted local tag: {}", tag);

    // 尝试删除远程标签
    let remote_delete_status = Command::new("git")
        .args(["push", "origin", &format!(":refs/tags/{}", tag)])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to delete remote tag: {}", e))?;

    if remote_delete_status.success() {
        println!("✓ Deleted remote tag: {}", tag);
    } else if config.debug {
        println!(
            "⚠ Warning: Failed to delete remote tag '{}' (it might not exist on remote)",
            tag
        );
    }

    Ok(())
}

/// 显示标签详细信息
async fn show_tag_info(tag: &str, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Showing info for tag: {}", tag);
    }

    // 检查标签是否存在
    let tag_exists = Command::new("git")
        .args(["tag", "-l", tag])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to check tag existence: {}", e))?;

    if tag_exists.stdout.is_empty() {
        anyhow::bail!("Tag '{}' does not exist", tag);
    }

    // 获取标签信息
    let show_output = Command::new("git")
        .args(["show", tag, "--no-patch", "--format=fuller"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to show tag info: {}", e))?;

    if !show_output.status.success() {
        anyhow::bail!(
            "Git show command failed with exit code: {:?}",
            show_output.status.code()
        );
    }

    println!("📌 Tag Information: {}", tag);
    println!("{}", "─".repeat(50));
    println!("{}", String::from_utf8_lossy(&show_output.stdout));

    // 获取标签的注释信息（如果是 annotated tag）
    let tag_message_output = Command::new("git")
        .args(["tag", "-l", "-n99", tag])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get tag message: {}", e))?;

    if tag_message_output.status.success() {
        let tag_message = String::from_utf8_lossy(&tag_message_output.stdout);
        if !tag_message.trim().is_empty() {
            println!("\n📝 Tag Message:");
            println!("{}", "─".repeat(50));
            // 跳过第一个单词（标签名）显示消息
            let message = tag_message
                .split_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ");
            if !message.is_empty() {
                println!("{}", message);
            }
        }
    }

    Ok(())
}

/// 比较两个标签之间的差异
async fn compare_tags(comparison: &str, config: &Config) -> anyhow::Result<()> {
    // 解析比较格式: tag1..tag2
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
    for tag in [tag1, tag2] {
        let tag_exists = Command::new("git")
            .args(["tag", "-l", tag])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check tag existence: {}", e))?;

        if tag_exists.stdout.is_empty() {
            anyhow::bail!("Tag '{}' does not exist", tag);
        }
    }

    println!("🔍 Comparing {} → {}", tag1, tag2);
    println!("{}", "─".repeat(60));

    // 显示提交差异统计
    let stat_output = Command::new("git")
        .args(["diff", "--stat", &format!("{}..{}", tag1, tag2)])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get diff stats: {}", e))?;

    if stat_output.status.success() && !stat_output.stdout.is_empty() {
        println!("📊 Changes Summary:");
        println!("{}", String::from_utf8_lossy(&stat_output.stdout));
    }

    // 显示提交日志
    let log_output = Command::new("git")
        .args([
            "log",
            "--oneline",
            "--graph",
            "--decorate",
            &format!("{}..{}", tag1, tag2),
        ])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get commit log: {}", e))?;

    if log_output.status.success() && !log_output.stdout.is_empty() {
        println!("\n📝 Commits between {} and {}:", tag1, tag2);
        println!("{}", String::from_utf8_lossy(&log_output.stdout));
    } else {
        println!("No commits found between {} and {}", tag1, tag2);
    }

    // 如果用户想要详细差异，可以提示如何查看
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
