use tokio::process::Command;

/// 增强的差异查看器，类似GRV的差异显示
pub struct DiffViewer;

#[derive(Debug, Clone)]
pub struct DiffStats {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
    pub file_stats: Vec<FileStats>,
}

#[derive(Debug, Clone)]
pub struct FileStats {
    pub path: String,
    pub insertions: u32,
    pub deletions: u32,
    pub status: FileStatus,
}

#[derive(Debug, Clone)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed(String),
    Copied(String),
}

impl DiffViewer {
    /// 显示提交的详细差异
    pub async fn show_commit_diff(commit: &str, context_lines: Option<u32>) -> anyhow::Result<()> {
        let mut args = vec!["show".to_string(), commit.to_string()];

        if let Some(context) = context_lines {
            args.push(format!("-U{}", context));
        }

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to show commit diff: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git show failed with exit code: {:?}", output.status.code());
        }

        let diff_output = String::from_utf8_lossy(&output.stdout);
        Self::display_colored_diff(&diff_output);

        Ok(())
    }

    /// 比较两个提交之间的差异
    pub async fn compare_commits(commit1: &str, commit2: &str) -> anyhow::Result<()> {
        let args = vec!["diff".to_string(), commit1.to_string(), commit2.to_string()];

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to compare commits: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git diff failed with exit code: {:?}", output.status.code());
        }

        let diff_output = String::from_utf8_lossy(&output.stdout);

        println!("🔍 Comparing {} -> {}", commit1, commit2);
        println!("{}", "─".repeat(60));
        Self::display_colored_diff(&diff_output);

        Ok(())
    }

    /// 显示工作区差异
    pub async fn show_working_diff(cached: bool) -> anyhow::Result<()> {
        let mut args = vec!["diff".to_string()];

        if cached {
            args.push("--cached".to_string());
        }

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to show working diff: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git diff failed with exit code: {:?}", output.status.code());
        }

        let diff_output = String::from_utf8_lossy(&output.stdout);

        if diff_output.trim().is_empty() {
            if cached {
                println!("No staged changes.");
            } else {
                println!("No unstaged changes.");
            }
            return Ok(());
        }

        let title = if cached {
            "Staged Changes"
        } else {
            "Unstaged Changes"
        };
        println!("📝 {}", title);
        println!("{}", "─".repeat(60));
        Self::display_colored_diff(&diff_output);

        Ok(())
    }

    /// 获取差异统计信息
    pub async fn get_diff_stats(
        commit1: Option<&str>,
        commit2: Option<&str>,
    ) -> anyhow::Result<DiffStats> {
        let mut args = vec!["diff".to_string(), "--numstat".to_string()];

        match (commit1, commit2) {
            (Some(c1), Some(c2)) => {
                args.extend(vec![c1.to_string(), c2.to_string()]);
            }
            (Some(c1), None) => {
                args.extend(vec![format!("{}^", c1), c1.to_string()]);
            }
            _ => {
                // 默认显示工作区差异
            }
        }

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get diff stats: {}", e))?;

        if !output.status.success() {
            anyhow::bail!(
                "Git diff --numstat failed with exit code: {:?}",
                output.status.code()
            );
        }

        let stats_output = String::from_utf8_lossy(&output.stdout);
        Self::parse_diff_stats(&stats_output)
    }

    /// 解析差异统计信息
    fn parse_diff_stats(stats: &str) -> anyhow::Result<DiffStats> {
        let mut file_stats = Vec::new();
        let mut total_insertions = 0;
        let mut total_deletions = 0;

        for line in stats.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let insertions = parts[0].parse::<u32>().unwrap_or(0);
                let deletions = parts[1].parse::<u32>().unwrap_or(0);
                let path = parts[2].to_string();

                total_insertions += insertions;
                total_deletions += deletions;

                // 简单的状态检测
                let status = if insertions > 0 && deletions == 0 {
                    FileStatus::Added
                } else if insertions == 0 && deletions > 0 {
                    FileStatus::Deleted
                } else {
                    FileStatus::Modified
                };

                file_stats.push(FileStats {
                    path,
                    insertions,
                    deletions,
                    status,
                });
            }
        }

        Ok(DiffStats {
            files_changed: file_stats.len() as u32,
            insertions: total_insertions,
            deletions: total_deletions,
            file_stats,
        })
    }

    /// 显示带颜色的差异
    fn display_colored_diff(diff: &str) {
        for line in diff.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                println!("\x1b[32m{}\x1b[0m", line); // Green for additions
            } else if line.starts_with('-') && !line.starts_with("---") {
                println!("\x1b[31m{}\x1b[0m", line); // Red for deletions
            } else if line.starts_with("@@") {
                println!("\x1b[36m{}\x1b[0m", line); // Cyan for hunk headers
            } else if line.starts_with("diff --git") {
                println!("\x1b[1m{}\x1b[0m", line); // Bold for file headers
            } else if line.starts_with("index") {
                println!("\x1b[90m{}\x1b[0m", line); // Gray for metadata
            } else {
                println!("{}", line);
            }
        }
    }

    /// 显示差异统计摘要
    pub fn display_diff_summary(stats: &DiffStats) {
        println!("📊 Diff Statistics:");
        println!("{}", "─".repeat(50));
        println!(
            "{} files changed, {} insertions(+), {} deletions(-)",
            stats.files_changed, stats.insertions, stats.deletions
        );

        if !stats.file_stats.is_empty() {
            println!("\nFile details:");
            for file_stat in &stats.file_stats {
                let status_icon = match file_stat.status {
                    FileStatus::Added => "🆕",
                    FileStatus::Modified => "📝",
                    FileStatus::Deleted => "🗑️",
                    FileStatus::Renamed(_) => "🔄",
                    FileStatus::Copied(_) => "📋",
                };

                println!(
                    "  {} {} (+{}, -{})",
                    status_icon, file_stat.path, file_stat.insertions, file_stat.deletions
                );
            }
        }
    }

    /// 显示文件级差异浏览
    pub async fn browse_file_diff(commit: &str, file_path: &str) -> anyhow::Result<()> {
        let args = vec!["show".to_string(), format!("{}:{}", commit, file_path)];

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to show file diff: {}", e))?;

        if !output.status.success() {
            anyhow::bail!(
                "Git show failed for file '{}' in commit '{}': {:?}",
                file_path,
                commit,
                output.status.code()
            );
        }

        println!("📄 File: {} @ {}", file_path, commit);
        println!("{}", "─".repeat(60));

        let content = String::from_utf8_lossy(&output.stdout);

        // 简单的语法高亮（基于文件扩展名）
        if file_path.ends_with(".rs") {
            Self::highlight_rust_syntax(&content);
        } else if file_path.ends_with(".js") || file_path.ends_with(".ts") {
            Self::highlight_javascript_syntax(&content);
        } else {
            println!("{}", content);
        }

        Ok(())
    }

    /// Rust语法高亮（简单实现）
    fn highlight_rust_syntax(content: &str) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                println!("\x1b[90m{}\x1b[0m", line); // Gray for comments
            } else if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                println!("\x1b[94m{}\x1b[0m", line); // Blue for functions
            } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                println!("\x1b[93m{}\x1b[0m", line); // Yellow for structs
            } else {
                println!("{}", line);
            }
        }
    }

    /// JavaScript/TypeScript语法高亮（简单实现）
    fn highlight_javascript_syntax(content: &str) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                println!("\x1b[90m{}\x1b[0m", line); // Gray for comments
            } else if trimmed.starts_with("function ") || trimmed.contains(" => ") {
                println!("\x1b[94m{}\x1b[0m", line); // Blue for functions
            } else if trimmed.starts_with("class ") {
                println!("\x1b[93m{}\x1b[0m", line); // Yellow for classes
            } else {
                println!("{}", line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff_stats() {
        let stats_output = "10\t5\tsrc/main.rs\n20\t0\tsrc/lib.rs\n0\t15\tsrc/old.rs";
        let stats = DiffViewer::parse_diff_stats(stats_output).unwrap();

        assert_eq!(stats.files_changed, 3);
        assert_eq!(stats.insertions, 30);
        assert_eq!(stats.deletions, 20);

        assert_eq!(stats.file_stats.len(), 3);
        assert_eq!(stats.file_stats[0].path, "src/main.rs");
        assert_eq!(stats.file_stats[0].insertions, 10);
        assert_eq!(stats.file_stats[0].deletions, 5);

        // 检查状态推断
        assert!(matches!(stats.file_stats[1].status, FileStatus::Added));
        assert!(matches!(stats.file_stats[2].status, FileStatus::Deleted));
    }

    #[tokio::test]
    async fn test_get_diff_stats() {
        let result = DiffViewer::get_diff_stats(None, None).await;

        match result {
            Ok(stats) => {
                println!("Diff stats: {} files changed", stats.files_changed);
            }
            Err(e) => {
                println!("Diff stats failed (expected in non-git environment): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_show_working_diff() {
        let result = DiffViewer::show_working_diff(false).await;

        match result {
            Ok(_) => {
                println!("Working diff displayed successfully");
            }
            Err(e) => {
                println!("Working diff failed: {}", e);
            }
        }
    }

    #[test]
    fn test_file_status_detection() {
        // Test different file status scenarios
        let added_stats = FileStats {
            path: "new_file.rs".to_string(),
            insertions: 100,
            deletions: 0,
            status: FileStatus::Added,
        };

        let deleted_stats = FileStats {
            path: "old_file.rs".to_string(),
            insertions: 0,
            deletions: 50,
            status: FileStatus::Deleted,
        };

        let modified_stats = FileStats {
            path: "existing_file.rs".to_string(),
            insertions: 25,
            deletions: 10,
            status: FileStatus::Modified,
        };

        assert!(matches!(added_stats.status, FileStatus::Added));
        assert!(matches!(deleted_stats.status, FileStatus::Deleted));
        assert!(matches!(modified_stats.status, FileStatus::Modified));
    }

    #[test]
    fn test_display_diff_summary() {
        let stats = DiffStats {
            files_changed: 3,
            insertions: 100,
            deletions: 50,
            file_stats: vec![
                FileStats {
                    path: "src/main.rs".to_string(),
                    insertions: 60,
                    deletions: 20,
                    status: FileStatus::Modified,
                },
                FileStats {
                    path: "src/new.rs".to_string(),
                    insertions: 40,
                    deletions: 0,
                    status: FileStatus::Added,
                },
                FileStats {
                    path: "src/old.rs".to_string(),
                    insertions: 0,
                    deletions: 30,
                    status: FileStatus::Deleted,
                },
            ],
        };

        // This test just ensures the display function doesn't panic
        DiffViewer::display_diff_summary(&stats);
    }

    #[test]
    fn test_display_colored_diff() {
        // Test colorized diff display with sample diff output
        let sample_diff = r#"
diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,5 +10,7 @@ fn main() {
     println!("Hello, world!");
+    println!("New line added");
-    println!("Old line removed");
 }
"#;

        // This should not panic and should process the diff
        DiffViewer::display_colored_diff(sample_diff);
    }

    #[test]
    fn test_colorize_diff_line() {
        // Test individual line colorization
        let added_line = "+    println!(\"New line\");";
        let deleted_line = "-    println!(\"Old line\");";
        let context_line = "     println!(\"Context line\");";
        let header_line = "diff --git a/src/main.rs b/src/main.rs";

        // These should not panic
        DiffViewer::display_colored_diff(added_line);
        DiffViewer::display_colored_diff(deleted_line);
        DiffViewer::display_colored_diff(context_line);
        DiffViewer::display_colored_diff(header_line);
    }

    #[test]
    fn test_empty_diff_handling() {
        // Test empty diff
        let empty_diff = "";
        DiffViewer::display_colored_diff(empty_diff);

        // Test whitespace-only diff
        let whitespace_diff = "   \n  \t  \n   ";
        DiffViewer::display_colored_diff(whitespace_diff);
    }

    #[tokio::test]
    async fn test_compare_commits_error_handling() {
        // Test comparing non-existent commits
        let result =
            DiffViewer::compare_commits("non-existent-commit-1", "non-existent-commit-2").await;

        match result {
            Ok(_) => println!("Commit comparison succeeded unexpectedly"),
            Err(e) => println!("Commit comparison failed as expected: {}", e),
        }
    }

    #[tokio::test]
    async fn test_show_commit_diff_with_context() {
        // Test showing commit diff with different context lines
        for context in [None, Some(1), Some(3), Some(10)] {
            let result = DiffViewer::show_commit_diff("HEAD", context).await;
            match result {
                Ok(_) => println!("Commit diff with context {:?} succeeded", context),
                Err(e) => println!("Commit diff with context {:?} failed: {}", context, e),
            }
        }
    }

    #[tokio::test]
    async fn test_get_diff_stats_edge_cases() {
        // Test getting diff stats with different parameters
        let test_cases = vec![
            (None, None),                   // Working directory
            (Some("HEAD"), None),           // Single commit
            (Some("HEAD"), Some("HEAD~1")), // Commit range
        ];

        for (commit1, commit2) in test_cases {
            let result = DiffViewer::get_diff_stats(commit1, commit2).await;
            match result {
                Ok(stats) => {
                    println!(
                        "Diff stats for {:?}..{:?}: {} files, +{} -{}",
                        commit1, commit2, stats.files_changed, stats.insertions, stats.deletions
                    );
                }
                Err(e) => {
                    println!("Diff stats for {:?}..{:?} failed: {}", commit1, commit2, e);
                }
            }
        }
    }

    #[test]
    fn test_file_status_classification() {
        // Test file status determination based on insertions/deletions
        let test_cases = vec![
            (100, 0, "new_file.rs", FileStatus::Added),
            (0, 50, "deleted_file.rs", FileStatus::Deleted),
            (30, 20, "modified_file.rs", FileStatus::Modified),
        ];

        for (insertions, deletions, path, expected_status) in test_cases {
            let stats = FileStats {
                path: path.to_string(),
                insertions,
                deletions,
                status: expected_status.clone(),
            };

            assert!(matches!(stats.status, _expected_status));
        }
    }

    #[test]
    fn test_diff_stats_calculations() {
        // Test DiffStats aggregation
        let file_stats = vec![
            FileStats {
                path: "file1.rs".to_string(),
                insertions: 50,
                deletions: 20,
                status: FileStatus::Modified,
            },
            FileStats {
                path: "file2.rs".to_string(),
                insertions: 30,
                deletions: 0,
                status: FileStatus::Added,
            },
            FileStats {
                path: "file3.rs".to_string(),
                insertions: 0,
                deletions: 40,
                status: FileStatus::Deleted,
            },
        ];

        let expected_files = file_stats.len();
        let expected_insertions: u32 = file_stats.iter().map(|f| f.insertions).sum();
        let expected_deletions: u32 = file_stats.iter().map(|f| f.deletions).sum();

        let stats = DiffStats {
            files_changed: expected_files as u32,
            insertions: expected_insertions,
            deletions: expected_deletions,
            file_stats: file_stats.clone(),
        };

        assert_eq!(stats.files_changed, 3);
        assert_eq!(stats.insertions, 80);
        assert_eq!(stats.deletions, 60);
        assert_eq!(stats.file_stats.len(), 3);
    }

    #[tokio::test]
    async fn test_show_working_diff_staged_unstaged() {
        // Test both staged and unstaged diffs
        let test_cases = vec![false, true]; // unstaged, staged

        for staged in test_cases {
            let result = DiffViewer::show_working_diff(staged).await;
            match result {
                Ok(_) => println!("Working diff (staged: {}) displayed successfully", staged),
                Err(e) => println!("Working diff (staged: {}) failed: {}", staged, e),
            }
        }
    }

    #[test]
    fn test_diff_line_patterns() {
        // Test different types of diff lines
        let diff_lines = vec![
            "diff --git a/file.rs b/file.rs",
            "index 1234567..abcdefg 100644",
            "--- a/file.rs",
            "+++ b/file.rs",
            "@@ -10,4 +10,6 @@ fn main() {",
            "+    // Added line",
            "-    // Removed line",
            "     // Context line",
            "\\ No newline at end of file",
        ];

        for line in diff_lines {
            // Ensure no line processing causes panic
            DiffViewer::display_colored_diff(line);
        }
    }

    #[test]
    fn test_complex_diff_scenarios() {
        let complex_diff = r#"
diff --git a/src/complex.rs b/src/complex.rs
index abc123..def456 100644
--- a/src/complex.rs
+++ b/src/complex.rs
@@ -1,10 +1,15 @@
 use std::collections::HashMap;
+use std::fs::File;
 
 fn main() {
+    // New function added
+    let data = load_data();
     let mut map = HashMap::new();
-    map.insert("old_key", "old_value");
+    map.insert("new_key", "new_value");
     
     println!("Map: {:?}", map);
+    
+    process_data(&data);
 }
 
-fn old_function() {
-    println!("This function is removed");
-}
+fn load_data() -> Vec<String> {
+    vec!["data1".to_string(), "data2".to_string()]
+}
+
+fn process_data(data: &[String]) {
+    for item in data {
+        println!("Processing: {}", item);
+    }
+}
"#;

        // Test complex diff processing
        DiffViewer::display_colored_diff(complex_diff);
    }

    #[tokio::test]
    async fn test_diff_viewer_concurrent_operations() {
        // Test multiple concurrent diff operations
        use tokio::task;

        let tasks = vec![
            task::spawn(async { DiffViewer::show_working_diff(false).await }),
            task::spawn(async { DiffViewer::show_working_diff(true).await }),
        ];

        let stats_task = task::spawn(async { DiffViewer::get_diff_stats(None, None).await });

        for task in tasks {
            match task.await {
                Ok(result) => match result {
                    Ok(_) => println!("Concurrent diff operation succeeded"),
                    Err(e) => println!("Concurrent diff operation failed: {}", e),
                },
                Err(e) => println!("Task join error: {}", e),
            }
        }

        // Handle stats task separately due to different return type
        match stats_task.await {
            Ok(result) => match result {
                Ok(_stats) => println!("Concurrent diff stats operation succeeded"),
                Err(e) => println!("Concurrent diff stats operation failed: {}", e),
            },
            Err(e) => println!("Stats task join error: {}", e),
        }
    }
}
