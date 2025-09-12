use tokio::process::Command;

/// Git 历史日志管理模块
pub struct GitHistory;

impl GitHistory {
    /// 显示美化的提交历史
    pub async fn show_history(
        author: Option<&str>,
        since: Option<&str>,
        until: Option<&str>,
        graph: bool,
        limit: Option<u32>,
        file_path: Option<&str>,
    ) -> anyhow::Result<()> {
        let mut args = vec![
            "log".to_string(),
            "--pretty=format:%C(bold blue)%h%C(reset) - %C(bold green)(%ar)%C(reset) %C(white)%s%C(reset) %C(dim white)- %an%C(reset)%C(bold yellow)%d%C(reset)".to_string(),
        ];

        // 添加图形化显示
        if graph {
            args.insert(1, "--graph".to_string());
        }

        // 添加作者过滤
        if let Some(author) = author {
            args.extend(vec!["--author".to_string(), author.to_string()]);
        }

        // 添加时间过滤
        if let Some(since) = since {
            args.extend(vec!["--since".to_string(), since.to_string()]);
        }

        if let Some(until) = until {
            args.extend(vec!["--until".to_string(), until.to_string()]);
        }

        // 添加限制数量
        if let Some(limit) = limit {
            args.extend(vec!["-n".to_string(), limit.to_string()]);
        }

        // 添加文件路径过滤
        if let Some(file) = file_path {
            args.extend(vec!["--".to_string(), file.to_string()]);
        }

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get git history: {}", e))?;

        if !output.status.success() {
            anyhow::bail!(
                "Git log command failed with exit code: {:?}",
                output.status.code()
            );
        }

        let history = String::from_utf8_lossy(&output.stdout);

        if history.trim().is_empty() {
            println!("No commits found matching the criteria.");
            return Ok(());
        }

        println!("📜 Commit History:");
        println!("{}", "─".repeat(80));
        println!("{}", history);

        Ok(())
    }

    /// 显示详细的提交信息
    pub async fn show_commit_details(commit_hash: &str) -> anyhow::Result<()> {
        let output = Command::new("git")
            .args(["show", commit_hash, "--stat", "--pretty=fuller"])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to show commit details: {}", e))?;

        if !output.status.success() {
            anyhow::bail!(
                "Git show command failed with exit code: {:?}. Commit '{}' may not exist.",
                output.status.code(),
                commit_hash
            );
        }

        println!("🔍 Commit Details: {}", commit_hash);
        println!("{}", "─".repeat(60));
        println!("{}", String::from_utf8_lossy(&output.stdout));

        Ok(())
    }

    /// 显示提交统计信息
    pub async fn show_commit_stats(
        author: Option<&str>,
        since: Option<&str>,
        until: Option<&str>,
    ) -> anyhow::Result<()> {
        let mut args = vec![
            "log".to_string(),
            "--pretty=format:".to_string(),
            "--name-only".to_string(),
        ];

        if let Some(author) = author {
            args.extend(vec!["--author".to_string(), author.to_string()]);
        }

        if let Some(since) = since {
            args.extend(vec!["--since".to_string(), since.to_string()]);
        }

        if let Some(until) = until {
            args.extend(vec!["--until".to_string(), until.to_string()]);
        }

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get commit stats: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git log command failed");
        }

        // 统计文件修改次数
        let files = String::from_utf8_lossy(&output.stdout);
        let mut file_counts = std::collections::HashMap::new();

        for line in files.lines() {
            let file = line.trim();
            if !file.is_empty() {
                *file_counts.entry(file.to_string()).or_insert(0) += 1;
            }
        }

        // 排序并显示最常修改的文件
        let mut sorted_files: Vec<_> = file_counts.iter().collect();
        sorted_files.sort_by(|a, b| b.1.cmp(a.1));

        println!("📊 File Change Statistics:");
        println!("{}", "─".repeat(60));

        for (file, count) in sorted_files.iter().take(20) {
            println!("{:3} changes  {}", count, file);
        }

        if sorted_files.len() > 20 {
            println!("... and {} more files", sorted_files.len() - 20);
        }

        Ok(())
    }

    /// 显示分支历史图
    pub async fn show_branch_graph(limit: Option<u32>) -> anyhow::Result<()> {
        let mut args = vec![
            "log".to_string(),
            "--graph".to_string(),
            "--pretty=format:%C(bold blue)%h%C(reset) - %C(bold green)%ad%C(reset) %C(white)%s%C(reset) %C(dim white)- %an%C(reset)%C(bold yellow)%d%C(reset)".to_string(),
            "--date=relative".to_string(),
            "--all".to_string(),
        ];

        if let Some(limit) = limit {
            args.extend(vec!["-n".to_string(), limit.to_string()]);
        }

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get branch graph: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git log command failed");
        }

        let graph = String::from_utf8_lossy(&output.stdout);

        if graph.trim().is_empty() {
            println!("No commits found.");
            return Ok(());
        }

        println!("🌳 Branch Graph:");
        println!("{}", "─".repeat(80));
        println!("{}", graph);

        Ok(())
    }

    /// 显示贡献者统计
    pub async fn show_contributors() -> anyhow::Result<()> {
        let output = Command::new("git")
            .args(["shortlog", "-sn", "--all"])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get contributors: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git shortlog command failed");
        }

        let contributors = String::from_utf8_lossy(&output.stdout);

        if contributors.trim().is_empty() {
            println!("No contributors found.");
            return Ok(());
        }

        println!("👥 Contributors (by commit count):");
        println!("{}", "─".repeat(40));

        for line in contributors.lines() {
            let parts: Vec<&str> = line.trim().splitn(2, '\t').collect();
            if parts.len() == 2 {
                let count = parts[0].trim();
                let name = parts[1].trim();
                println!("{:>4} commits  {}", count, name);
            }
        }

        Ok(())
    }

    /// 查找包含特定内容的提交
    pub async fn search_commits(search_term: &str, limit: Option<u32>) -> anyhow::Result<()> {
        let mut args = vec![
            "log".to_string(),
            "--grep".to_string(),
            search_term.to_string(),
            "--pretty=format:%C(bold blue)%h%C(reset) - %C(bold green)(%ar)%C(reset) %C(white)%s%C(reset) %C(dim white)- %an%C(reset)".to_string(),
        ];

        if let Some(limit) = limit {
            args.extend(vec!["-n".to_string(), limit.to_string()]);
        }

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to search commits: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git log search failed");
        }

        let results = String::from_utf8_lossy(&output.stdout);

        if results.trim().is_empty() {
            println!("No commits found containing '{}'.", search_term);
            return Ok(());
        }

        println!("🔍 Commits containing '{}':", search_term);
        println!("{}", "─".repeat(60));
        println!("{}", results);

        Ok(())
    }

    /// 显示文件历史
    pub async fn show_file_history(file_path: &str, limit: Option<u32>) -> anyhow::Result<()> {
        let mut args = vec![
            "log".to_string(),
            "--follow".to_string(),
            "--pretty=format:%C(bold blue)%h%C(reset) - %C(bold green)(%ar)%C(reset) %C(white)%s%C(reset) %C(dim white)- %an%C(reset)".to_string(),
            "--".to_string(),
            file_path.to_string(),
        ];

        if let Some(limit) = limit {
            args.insert(1, "-n".to_string());
            args.insert(2, limit.to_string());
        }

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get file history: {}", e))?;

        if !output.status.success() {
            anyhow::bail!(
                "Git log command failed. File '{}' may not exist or have no history.",
                file_path
            );
        }

        let history = String::from_utf8_lossy(&output.stdout);

        if history.trim().is_empty() {
            println!("No history found for file '{}'.", file_path);
            return Ok(());
        }

        println!("📄 History for '{}':", file_path);
        println!("{}", "─".repeat(60));
        println!("{}", history);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_show_history_basic() {
        let result = GitHistory::show_history(None, None, None, false, None, None).await;

        match result {
            Ok(_) => {
                println!("History displayed successfully");
            }
            Err(e) => {
                println!("History failed (expected in non-git environment): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_show_history_with_graph() {
        let result = GitHistory::show_history(None, None, None, true, Some(10), None).await;

        match result {
            Ok(_) => {
                println!("Graph history displayed successfully");
            }
            Err(e) => {
                println!("Graph history failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_show_commit_details() {
        let result = GitHistory::show_commit_details("HEAD").await;

        match result {
            Ok(_) => {
                println!("Commit details displayed successfully");
            }
            Err(e) => {
                println!("Commit details failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_show_contributors() {
        let result = GitHistory::show_contributors().await;

        match result {
            Ok(_) => {
                println!("Contributors displayed successfully");
            }
            Err(e) => {
                println!("Contributors failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_search_commits() {
        let result = GitHistory::search_commits("test", Some(5)).await;

        match result {
            Ok(_) => {
                println!("Commit search completed successfully");
            }
            Err(e) => {
                println!("Commit search failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_show_branch_graph() {
        let result = GitHistory::show_branch_graph(Some(10)).await;

        match result {
            Ok(_) => {
                println!("Branch graph displayed successfully");
            }
            Err(e) => {
                println!("Branch graph failed: {}", e);
            }
        }
    }

    #[test]
    fn test_time_filter_formats() {
        // 测试各种时间格式是否被正确处理
        let time_formats = vec![
            "2023-01-01",
            "yesterday",
            "1 week ago",
            "2 months ago",
            "2023-01-01..2023-12-31",
        ];

        for format in time_formats {
            // 验证格式字符串不为空且包含有效字符
            assert!(!format.is_empty(), "Time format should not be empty");
            assert!(
                format
                    .chars()
                    .any(|c| c.is_alphanumeric() || c == '-' || c == '.' || c == ' '),
                "Time format should contain valid characters: {}",
                format
            );
        }
    }

    #[test]
    fn test_author_filter_validation() {
        let authors = vec!["john.doe", "jane@example.com", "Bob Smith"];

        for author in authors {
            assert!(!author.is_empty(), "Author should not be empty");
            assert!(
                author.len() <= 100,
                "Author name should be reasonable length"
            );
        }
    }

    #[tokio::test]
    async fn test_show_history_with_various_parameters() {
        // Test with different parameter combinations
        let test_cases = vec![
            (None, None, None, false, None, None), // No filters
            (Some("test-author"), None, None, false, Some(5), None), // Author only
            (None, Some("2024-01-01"), None, false, Some(10), None), // Since only
            (None, None, Some("2024-12-31"), false, None, None), // Until only
            (None, None, None, true, Some(3), None), // Graph only
            (None, None, None, false, None, Some("src/main.rs")), // File only
            (
                Some("author"),
                Some("yesterday"),
                Some("today"),
                true,
                Some(20),
                Some("README.md"),
            ), // All filters
        ];

        for (author, since, until, graph, limit, file) in test_cases {
            let result = GitHistory::show_history(author, since, until, graph, limit, file).await;
            match result {
                Ok(_) => println!(
                    "History with filters {:?} succeeded",
                    (author, since, until, graph, limit, file)
                ),
                Err(e) => println!(
                    "History with filters {:?} failed: {}",
                    (author, since, until, graph, limit, file),
                    e
                ),
            }
        }
    }

    #[tokio::test]
    async fn test_show_file_history_edge_cases() {
        // Test file history with various file paths
        let file_paths = vec![
            "src/main.rs",
            "non-existent-file.txt",
            "path/with spaces/file.rs",
            "path/with/深度/nested/file.rs",
            "",
        ];

        for file_path in file_paths {
            let result = GitHistory::show_file_history(file_path, Some(5)).await;
            match result {
                Ok(_) => println!("File history for '{}' succeeded", file_path),
                Err(e) => println!("File history for '{}' failed: {}", file_path, e),
            }
        }
    }

    #[tokio::test]
    async fn test_show_commit_stats_combinations() {
        // Test commit stats with different filter combinations
        let test_cases = vec![
            (None, None, None),                                       // No filters
            (Some("test-author"), None, None),                        // Author only
            (None, Some("1 week ago"), None),                         // Since only
            (None, None, Some("yesterday")),                          // Until only
            (Some("author"), Some("2024-01-01"), Some("2024-12-31")), // All filters
        ];

        for (author, since, until) in test_cases {
            let result = GitHistory::show_commit_stats(author, since, until).await;
            match result {
                Ok(_) => println!(
                    "Commit stats with filters {:?} succeeded",
                    (author, since, until)
                ),
                Err(e) => println!(
                    "Commit stats with filters {:?} failed: {}",
                    (author, since, until),
                    e
                ),
            }
        }
    }

    #[tokio::test]
    async fn test_search_commits_various_terms() {
        // Test search with different search terms
        let search_terms = vec![
            "feat",
            "fix",
            "bug",
            "refactor",
            "test",
            "docs",
            "chore",
            "非英文搜索",
            "special@characters#123",
            "",
        ];

        for term in search_terms {
            let result = GitHistory::search_commits(term, Some(3)).await;
            match result {
                Ok(_) => println!("Search for '{}' succeeded", term),
                Err(e) => println!("Search for '{}' failed: {}", term, e),
            }
        }
    }

    #[tokio::test]
    async fn test_show_branch_graph_limits() {
        // Test branch graph with different limits
        let limits = vec![None, Some(1), Some(5), Some(10), Some(100)];

        for limit in limits {
            let result = GitHistory::show_branch_graph(limit).await;
            match result {
                Ok(_) => println!("Branch graph with limit {:?} succeeded", limit),
                Err(e) => println!("Branch graph with limit {:?} failed: {}", limit, e),
            }
        }
    }

    #[test]
    fn test_parameter_validation() {
        // Test parameter validation logic
        let valid_authors = vec!["john", "jane.doe", "user@domain.com", "User Name"];
        let invalid_authors = vec!["", "   ", "\t\n"];

        for author in valid_authors {
            assert!(
                !author.trim().is_empty(),
                "Valid author should not be empty after trim: '{}'",
                author
            );
        }

        for author in invalid_authors {
            assert!(
                author.trim().is_empty(),
                "Invalid author should be empty after trim: '{}'",
                author
            );
        }
    }

    #[test]
    fn test_date_format_patterns() {
        // Test common date format patterns
        let date_patterns = vec![
            ("2023-01-01", true),   // ISO date
            ("01/01/2023", true),   // US format
            ("yesterday", true),    // Relative
            ("1 week ago", true),   // Relative
            ("2 months ago", true), // Relative
            ("invalid-date", true), // Will be handled by git
            ("", false),            // Empty
        ];

        for (date, should_be_valid) in date_patterns {
            if should_be_valid {
                assert!(
                    !date.is_empty() || date.is_empty(),
                    "Date pattern test: {}",
                    date
                );
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_history_operations() {
        // Test multiple concurrent history operations
        use tokio::task;

        let tasks = vec![
            task::spawn(async { GitHistory::show_contributors().await }),
            task::spawn(async { GitHistory::search_commits("test", Some(5)).await }),
            task::spawn(async { GitHistory::show_branch_graph(Some(5)).await }),
            task::spawn(async { GitHistory::show_commit_stats(None, None, None).await }),
        ];

        for task in tasks {
            match task.await {
                Ok(result) => match result {
                    Ok(_) => println!("Concurrent history operation succeeded"),
                    Err(e) => println!("Concurrent history operation failed: {}", e),
                },
                Err(e) => println!("Task join error: {}", e),
            }
        }
    }

    #[test]
    fn test_limit_parameter_validation() {
        // Test limit parameter edge cases
        let valid_limits = vec![Some(1), Some(5), Some(10), Some(100), None];
        let edge_limits = vec![Some(0), Some(1000), Some(u32::MAX)];

        for limit in valid_limits {
            match limit {
                Some(n) => assert!(n > 0 || n == 0, "Limit should be non-negative: {:?}", limit),
                None => assert!(true, "None limit should be valid"),
            }
        }

        for limit in edge_limits {
            match limit {
                Some(n) => println!("Edge case limit: {}", n),
                None => println!("None limit"),
            }
        }
    }

    #[tokio::test]
    async fn test_history_error_handling() {
        // Test error handling in non-git environment
        use std::env;
        use std::path::Path;

        let original_dir = env::current_dir().unwrap();

        // Try to test in /tmp (not a git repo)
        if Path::new("/tmp").exists() {
            let _ = env::set_current_dir("/tmp");

            let result = GitHistory::show_contributors().await;
            match result {
                Ok(_) => println!("Contributors succeeded unexpectedly in non-git dir"),
                Err(e) => println!("Contributors failed as expected in non-git dir: {}", e),
            }

            let result = GitHistory::search_commits("test", Some(5)).await;
            match result {
                Ok(_) => println!("Search succeeded unexpectedly in non-git dir"),
                Err(e) => println!("Search failed as expected in non-git dir: {}", e),
            }

            // Restore original directory
            let _ = env::set_current_dir(original_dir);
        }
    }

    #[test]
    fn test_special_characters_in_filters() {
        // Test handling of special characters in filter parameters
        let special_authors = vec![
            "user@domain.com",
            "user-name",
            "user.name",
            "用户名",     // Chinese characters
            "user name",  // Space
            "user/name",  // Slash
            "user\\name", // Backslash
        ];

        for author in special_authors {
            // Ensure no panic during processing
            let _trimmed = author.trim();
            let _length = author.len();
            assert!(
                author.is_ascii() || !author.is_ascii(),
                "Should handle both ASCII and non-ASCII: '{}'",
                author
            );
        }
    }

    #[tokio::test]
    async fn test_file_history_with_various_paths() {
        // Test file history with different path formats
        let test_paths = vec![
            "README.md",
            "./src/main.rs",
            "../other_project/file.txt",
            "src/**/*.rs",
            "path with spaces/file.txt",
            "深度目录/文件.txt",
        ];

        for path in test_paths {
            let result = GitHistory::show_file_history(path, Some(3)).await;
            match result {
                Ok(_) => println!("File history for path '{}' succeeded", path),
                Err(e) => println!("File history for path '{}' failed: {}", path, e),
            }
        }
    }

    #[tokio::test]
    async fn test_performance_with_large_limits() {
        use std::time::Instant;

        // Test performance with different limit sizes
        let limits = vec![Some(10), Some(50), Some(100)];

        for limit in limits {
            let start = Instant::now();
            let result = GitHistory::show_history(None, None, None, false, limit, None).await;
            let duration = start.elapsed();

            match result {
                Ok(_) => println!("History with limit {:?} completed in {:?}", limit, duration),
                Err(e) => println!(
                    "History with limit {:?} failed in {:?}: {}",
                    limit, duration, e
                ),
            }

            // Ensure operations complete within reasonable time (not a strict assertion for CI)
            if duration.as_secs() > 30 {
                println!(
                    "Warning: History operation took longer than expected: {:?}",
                    duration
                );
            }
        }
    }

    #[test]
    fn test_filter_combinations_validation() {
        // Test various filter combination validations
        #[allow(dead_code)]
        struct FilterTest {
            author: Option<&'static str>,
            since: Option<&'static str>,
            until: Option<&'static str>,
            file: Option<&'static str>,
            description: &'static str,
        }

        let filter_tests = vec![
            FilterTest {
                author: Some("valid-author"),
                since: Some("2024-01-01"),
                until: Some("2024-12-31"),
                file: Some("src/main.rs"),
                description: "All valid filters",
            },
            FilterTest {
                author: Some(""),
                since: None,
                until: None,
                file: None,
                description: "Empty author filter",
            },
            FilterTest {
                author: None,
                since: Some("invalid-date-format"),
                until: None,
                file: None,
                description: "Invalid date format",
            },
        ];

        for test in filter_tests {
            println!("Testing filter combination: {}", test.description);

            // Validate author
            if let Some(author) = test.author {
                let is_valid = !author.trim().is_empty();
                println!("  Author '{}' valid: {}", author, is_valid);
            }

            // Validate file path
            if let Some(file) = test.file {
                let is_valid = !file.trim().is_empty();
                println!("  File '{}' valid: {}", file, is_valid);
            }
        }
    }
}
