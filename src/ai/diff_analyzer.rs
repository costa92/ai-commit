use std::collections::HashMap;
use regex::Regex;
use once_cell::sync::Lazy;

// 大文件阈值 (字符数)
const LARGE_DIFF_THRESHOLD: usize = 10000;
// 多文件阈值 (文件数量)
const MULTI_FILE_THRESHOLD: usize = 5;

static FILE_CHANGE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^diff --git a/(.+?) b/(.+?)$").unwrap()
});

static ADDITION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\+").unwrap()
});

static DELETION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^-").unwrap()
});

#[derive(Debug, Clone)]
pub struct FileChange {
    pub file_path: String,
    pub additions: usize,
    pub deletions: usize,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug)]
pub struct DiffAnalysis {
    pub total_files: usize,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub file_changes: Vec<FileChange>,
    pub is_large_diff: bool,
    pub is_multi_file: bool,
    pub primary_change_type: String,
    pub dominant_scope: Option<String>,
}

impl DiffAnalysis {
    pub fn analyze_diff(diff: &str) -> Self {
        let mut file_changes = Vec::new();
        let mut total_additions = 0;
        let mut total_deletions = 0;
        let mut current_file: Option<String> = None;
        let mut current_additions = 0;
        let mut current_deletions = 0;

        for line in diff.lines() {
            // 检测文件变更
            if let Some(captures) = FILE_CHANGE_REGEX.captures(line) {
                // 保存前一个文件的统计
                if let Some(file_path) = current_file.take() {
                    file_changes.push(FileChange {
                        file_path,
                        additions: current_additions,
                        deletions: current_deletions,
                        change_type: Self::determine_change_type(current_additions, current_deletions),
                    });
                    current_additions = 0;
                    current_deletions = 0;
                }
                
                current_file = Some(captures.get(2).unwrap().as_str().to_string());
            }
            // 统计添加行
            else if ADDITION_REGEX.is_match(line) && !line.starts_with("+++") {
                current_additions += 1;
                total_additions += 1;
            }
            // 统计删除行
            else if DELETION_REGEX.is_match(line) && !line.starts_with("---") {
                current_deletions += 1;
                total_deletions += 1;
            }
        }

        // 处理最后一个文件
        if let Some(file_path) = current_file {
            file_changes.push(FileChange {
                file_path,
                additions: current_additions,
                deletions: current_deletions,
                change_type: Self::determine_change_type(current_additions, current_deletions),
            });
        }

        let total_files = file_changes.len();
        let is_large_diff = diff.len() > LARGE_DIFF_THRESHOLD;
        let is_multi_file = total_files > MULTI_FILE_THRESHOLD;

        let primary_change_type = Self::determine_primary_change_type(&file_changes);
        let dominant_scope = Self::determine_dominant_scope(&file_changes);

        DiffAnalysis {
            total_files,
            total_additions,
            total_deletions,
            file_changes,
            is_large_diff,
            is_multi_file,
            primary_change_type,
            dominant_scope,
        }
    }

    fn determine_change_type(additions: usize, deletions: usize) -> ChangeType {
        match (additions, deletions) {
            (0, 0) => ChangeType::Modified, // 可能是权限变更等
            (a, 0) if a > 0 => ChangeType::Added,
            (0, d) if d > 0 => ChangeType::Deleted,
            _ => ChangeType::Modified,
        }
    }

    fn determine_primary_change_type(file_changes: &[FileChange]) -> String {
        let mut type_counts = HashMap::new();
        
        for change in file_changes {
            let count = type_counts.entry(&change.change_type).or_insert(0);
            *count += 1;
        }

        let dominant_type = if let Some((change_type, _)) = type_counts
            .iter()
            .max_by_key(|(_, count)| *count)
        {
            change_type
        } else {
            &ChangeType::Modified
        };

        match dominant_type {
            ChangeType::Added => "feat".to_string(),
            ChangeType::Deleted => "refactor".to_string(),
            ChangeType::Modified => {
                // 基于文件路径推断类型
                if file_changes.iter().any(|f| f.file_path.contains("test")) {
                    "test".to_string()
                } else if file_changes.iter().any(|f| f.file_path.ends_with(".md") || f.file_path.contains("doc")) {
                    "docs".to_string()
                } else if file_changes.iter().any(|f| f.file_path.contains("fix") || f.file_path.contains("bug")) {
                    "fix".to_string()
                } else {
                    "refactor".to_string()
                }
            }
            ChangeType::Renamed => "refactor".to_string(),
        }
    }

    fn determine_dominant_scope(file_changes: &[FileChange]) -> Option<String> {
        let mut scope_counts = HashMap::new();
        
        for change in file_changes {
            let scope = Self::extract_scope_from_path(&change.file_path);
            if let Some(scope) = scope {
                let count = scope_counts.entry(scope).or_insert(0);
                *count += 1;
            }
        }

        scope_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(scope, _)| scope.clone())
    }

    fn extract_scope_from_path(file_path: &str) -> Option<String> {
        // 从文件路径中提取作用域
        let path_parts: Vec<&str> = file_path.split('/').collect();
        
        // 优先级规则：src/ > 具体模块 > 顶级目录
        if path_parts.len() >= 2 {
            match path_parts[0] {
                "src" if path_parts.len() >= 3 => Some(path_parts[1].to_string()),
                "src" if path_parts.len() >= 2 => Some(path_parts[1].split('.').next().unwrap_or("core").to_string()),
                "tests" => Some("test".to_string()),
                "docs" => Some("docs".to_string()),
                "examples" => Some("example".to_string()),
                _ => Some(path_parts[0].to_string()),
            }
        } else if let Some(filename) = path_parts.last() {
            // 从文件名推断作用域
            if filename.contains("test") {
                Some("test".to_string())
            } else if filename.ends_with(".md") {
                Some("docs".to_string())
            } else if filename.starts_with("Cargo") {
                Some("cargo".to_string())
            } else {
                Some("core".to_string())
            }
        } else {
            None
        }
    }

    /// 生成大文件场景的摘要
    pub fn generate_summary(&self) -> String {
        if !self.is_large_diff && !self.is_multi_file {
            return String::new();
        }

        let mut summary_parts = Vec::new();

        // 文件数量摘要
        if self.is_multi_file {
            summary_parts.push(format!("涉及{}个文件", self.total_files));
        }

        // 变更摘要  
        if self.total_additions > 0 && self.total_deletions > 0 {
            summary_parts.push(format!("新增{}行，删除{}行", self.total_additions, self.total_deletions));
        } else if self.total_additions > 0 {
            summary_parts.push(format!("新增{}行", self.total_additions));
        } else if self.total_deletions > 0 {
            summary_parts.push(format!("删除{}行", self.total_deletions));
        }

        // 主要变更类型
        let change_summary = match self.primary_change_type.as_str() {
            "feat" => "添加新功能",
            "fix" => "修复问题", 
            "refactor" => "重构代码",
            "test" => "更新测试",
            "docs" => "更新文档",
            _ => "优化代码",
        };
        
        summary_parts.push(change_summary.to_string());

        summary_parts.join("，")
    }

    /// 为大型或多文件diff生成优化的prompt
    pub fn create_optimized_prompt(&self, original_diff: &str) -> String {
        if !self.is_large_diff && !self.is_multi_file {
            return original_diff.to_string();
        }

        let mut optimized_prompt = String::new();
        
        // 添加摘要信息
        optimized_prompt.push_str(&format!("变更摘要：{}\n\n", self.generate_summary()));
        
        // 文件列表
        optimized_prompt.push_str("主要变更文件：\n");
        for (i, change) in self.file_changes.iter().take(10).enumerate() {
            let change_type_desc = match change.change_type {
                ChangeType::Added => "新增",
                ChangeType::Modified => "修改", 
                ChangeType::Deleted => "删除",
                ChangeType::Renamed => "重命名",
            };
            optimized_prompt.push_str(&format!("{}. {} {} (+{} -{} lines)\n", 
                i + 1, change_type_desc, change.file_path, change.additions, change.deletions));
        }
        
        if self.file_changes.len() > 10 {
            optimized_prompt.push_str(&format!("...还有{}个文件\n", self.file_changes.len() - 10));
        }
        
        // 如果diff过大，只包含关键部分
        if self.is_large_diff {
            optimized_prompt.push_str("\n核心变更片段：\n");
            optimized_prompt.push_str(&Self::extract_key_changes(original_diff));
        } else {
            optimized_prompt.push_str("\n完整diff：\n");
            optimized_prompt.push_str(original_diff);
        }
        
        optimized_prompt
    }

    /// 提取关键变更片段
    fn extract_key_changes(diff: &str) -> String {
        let lines: Vec<&str> = diff.lines().collect();
        let mut key_changes = Vec::new();
        let mut current_file = String::new();
        let mut change_lines = Vec::new();
        
        for line in lines {
            if line.starts_with("diff --git") {
                // 保存前一个文件的关键变更
                if !change_lines.is_empty() {
                    key_changes.push(format!("File: {}\n{}", current_file, change_lines.join("\n")));
                    change_lines.clear();
                }
                current_file = line.to_string();
            } else if line.starts_with("@@") {
                // 保存上下文信息
                change_lines.push(line.to_string());
            } else if line.starts_with('+') || line.starts_with('-') {
                // 保存变更行
                if change_lines.len() < 20 { // 限制每个文件最多20行关键变更
                    change_lines.push(line.to_string());
                }
            }
        }
        
        // 保存最后一个文件
        if !change_lines.is_empty() {
            key_changes.push(format!("File: {}\n{}", current_file, change_lines.join("\n")));
        }
        
        key_changes.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_single_file_diff() {
        let diff = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
     println!("Goodbye");
 }"#;

        let analysis = DiffAnalysis::analyze_diff(diff);
        assert_eq!(analysis.total_files, 1);
        assert_eq!(analysis.total_additions, 1);
        assert_eq!(analysis.total_deletions, 0);
        assert!(!analysis.is_large_diff);
        assert!(!analysis.is_multi_file);
        assert_eq!(analysis.primary_change_type, "feat");
    }

    #[test]
    fn test_multi_file_diff() {
        let diff = r#"diff --git a/src/lib.rs b/src/lib.rs
index 1234567..abcdefg 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,4 @@
+pub mod new_module;
 pub mod existing;

diff --git a/src/new_module.rs b/src/new_module.rs
new file mode 100644
index 0000000..1234567
--- /dev/null
+++ b/src/new_module.rs
@@ -0,0 +1,10 @@
+pub fn new_function() {
+    println!("New function");
+}

diff --git a/tests/integration.rs b/tests/integration.rs
index 1234567..abcdefg 100644
--- a/tests/integration.rs
+++ b/tests/integration.rs
@@ -1,3 +1,8 @@
+#[test]
+fn test_new_function() {
+    assert!(true);
+}
+
 #[test]
 fn existing_test() {
     assert!(true);
"#;

        let analysis = DiffAnalysis::analyze_diff(diff);
        assert_eq!(analysis.total_files, 3);
        assert!(analysis.total_additions > 0);
        assert!(!analysis.is_large_diff);
        assert!(!analysis.is_multi_file); // 3 files < MULTI_FILE_THRESHOLD (5)
        
        // Check file changes
        assert_eq!(analysis.file_changes.len(), 3);
        assert!(analysis.file_changes.iter().any(|f| f.file_path == "src/lib.rs"));
        assert!(analysis.file_changes.iter().any(|f| f.file_path == "src/new_module.rs"));
        assert!(analysis.file_changes.iter().any(|f| f.file_path == "tests/integration.rs"));
    }

    #[test]
    fn test_scope_extraction() {
        assert_eq!(DiffAnalysis::extract_scope_from_path("src/ai/mod.rs"), Some("ai".to_string()));
        assert_eq!(DiffAnalysis::extract_scope_from_path("src/config.rs"), Some("config".to_string()));
        assert_eq!(DiffAnalysis::extract_scope_from_path("tests/unit.rs"), Some("test".to_string()));
        assert_eq!(DiffAnalysis::extract_scope_from_path("docs/README.md"), Some("docs".to_string()));
        assert_eq!(DiffAnalysis::extract_scope_from_path("Cargo.toml"), Some("cargo".to_string()));
    }

    #[test]
    fn test_change_type_determination() {
        assert_eq!(DiffAnalysis::determine_change_type(5, 0), ChangeType::Added);
        assert_eq!(DiffAnalysis::determine_change_type(0, 3), ChangeType::Deleted);
        assert_eq!(DiffAnalysis::determine_change_type(2, 4), ChangeType::Modified);
        assert_eq!(DiffAnalysis::determine_change_type(0, 0), ChangeType::Modified);
    }

    #[test]
    fn test_large_diff_detection() {
        let large_content = "a".repeat(LARGE_DIFF_THRESHOLD + 1000);
        let analysis = DiffAnalysis::analyze_diff(&large_content);
        assert!(analysis.is_large_diff);
    }

    #[test]
    fn test_summary_generation() {
        let diff = r#"diff --git a/src/main.rs b/src/main.rs
+    println!("test");
+    println!("test2");
-    old_code();
"#;
        
        let mut analysis = DiffAnalysis::analyze_diff(diff);
        analysis.is_multi_file = true; // Force multi-file for testing
        analysis.total_files = 6;
        
        let summary = analysis.generate_summary();
        assert!(summary.contains("6个文件"));
        assert!(summary.contains("新增2行"));
        assert!(summary.contains("删除1行"));
    }
}