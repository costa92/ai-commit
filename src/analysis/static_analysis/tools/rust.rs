use std::process::Command;
use async_trait::async_trait;
use serde_json::Value;
use crate::languages::Language;
use crate::analysis::static_analysis::{StaticAnalysisTool, Issue, Severity, IssueCategory};

/// Rust 格式化工具
pub struct RustFmtTool;

impl RustFmtTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl StaticAnalysisTool for RustFmtTool {
    fn name(&self) -> &str {
        "rustfmt"
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::Rust]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        let output = tokio::process::Command::new("rustfmt")
            .args(["--check", "--edition", "2021", file_path])
            .output()
            .await?;

        let mut issues = Vec::new();

        // rustfmt --check 如果文件需要格式化会返回非零退出码
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            issues.push(
                Issue::new(
                    "rustfmt".to_string(),
                    file_path.to_string(),
                    Severity::Low,
                    IssueCategory::Style,
                    "代码格式不符合 Rust 标准格式".to_string(),
                )
                .with_suggestion("运行 'rustfmt filename.rs' 自动格式化代码".to_string())
                .with_rule_id("rustfmt-format".to_string())
                .with_code_snippet(stderr.to_string())
            );
        }

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        Command::new("rustfmt")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Clippy 工具
pub struct ClippyTool;

impl ClippyTool {
    pub fn new() -> Self {
        Self
    }

    fn parse_clippy_json(&self, json_output: &str, file_path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        for line in json_output.lines() {
            if let Ok(json) = serde_json::from_str::<Value>(line) {
                if let Some(message) = json.get("message") {
                    if let Some(spans) = message.get("spans").and_then(|s| s.as_array()) {
                        for span in spans {
                            if let Some(span_file) = span.get("file_name").and_then(|f| f.as_str()) {
                                // 只处理当前文件的问题
                                if span_file.ends_with(file_path) || file_path.ends_with(span_file) {
                                    let line_start = span.get("line_start")
                                        .and_then(|l| l.as_u64())
                                        .unwrap_or(0) as usize;
                                    let column_start = span.get("column_start")
                                        .and_then(|c| c.as_u64())
                                        .unwrap_or(0) as usize;

                                    let msg = message.get("message")
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("Clippy warning");

                                    let level = message.get("level")
                                        .and_then(|l| l.as_str())
                                        .unwrap_or("warning");

                                    let code = message.get("code")
                                        .and_then(|c| c.get("code"))
                                        .and_then(|c| c.as_str());

                                    let severity = self.determine_severity(level, msg);
                                    let category = self.determine_category(msg);
                                    let suggestion = self.get_suggestion(msg);

                                    let mut issue = Issue::new(
                                        "clippy".to_string(),
                                        file_path.to_string(),
                                        severity,
                                        category,
                                        msg.to_string(),
                                    )
                                    .with_location(line_start, Some(column_start))
                                    .with_suggestion(suggestion);

                                    if let Some(code) = code {
                                        issue = issue.with_rule_id(code.to_string());
                                    }

                                    issues.push(issue);
                                }
                            }
                        }
                    }
                }
            }
        }

        issues
    }

    fn determine_severity(&self, level: &str, message: &str) -> Severity {
        match level {
            "error" => Severity::High,
            "warning" => {
                let message_lower = message.to_lowercase();
                if message_lower.contains("unsafe") ||
                   message_lower.contains("panic") ||
                   message_lower.contains("unwrap") {
                    Severity::Medium
                } else {
                    Severity::Low
                }
            }
            _ => Severity::Info,
        }
    }

    fn determine_category(&self, message: &str) -> IssueCategory {
        let message_lower = message.to_lowercase();

        if message_lower.contains("unsafe") ||
           message_lower.contains("panic") ||
           message_lower.contains("unwrap") {
            IssueCategory::Bug
        } else if message_lower.contains("performance") ||
                  message_lower.contains("inefficient") {
            IssueCategory::Performance
        } else if message_lower.contains("complexity") {
            IssueCategory::Complexity
        } else if message_lower.contains("unused") ||
                  message_lower.contains("dead_code") {
            IssueCategory::Maintainability
        } else {
            IssueCategory::Style
        }
    }

    fn get_suggestion(&self, message: &str) -> String {
        let message_lower = message.to_lowercase();

        if message_lower.contains("unwrap") {
            "使用 match 或 if let 替代 unwrap()".to_string()
        } else if message_lower.contains("clone") {
            "考虑使用引用而不是克隆".to_string()
        } else if message_lower.contains("unused") {
            "移除未使用的代码或添加 #[allow(dead_code)]".to_string()
        } else if message_lower.contains("complexity") {
            "简化复杂的表达式或函数".to_string()
        } else {
            "请查看 Clippy 文档了解详细信息".to_string()
        }
    }
}

#[async_trait]
impl StaticAnalysisTool for ClippyTool {
    fn name(&self) -> &str {
        "clippy"
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::Rust]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        let output = tokio::process::Command::new("cargo")
            .args([
                "clippy",
                "--message-format=json",
                "--",
                "-W", "clippy::all",
                "-W", "clippy::pedantic",
                "-W", "clippy::nursery"
            ])
            .current_dir(std::path::Path::new(file_path).parent().unwrap_or(std::path::Path::new(".")))
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let issues = self.parse_clippy_json(&stdout, file_path);

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        Command::new("cargo")
            .args(["clippy", "--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Cargo Check 工具
pub struct CargoCheckTool;

impl CargoCheckTool {
    pub fn new() -> Self {
        Self
    }

    fn parse_cargo_check_json(&self, json_output: &str, file_path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        for line in json_output.lines() {
            if let Ok(json) = serde_json::from_str::<Value>(line) {
                if let Some(message) = json.get("message") {
                    if let Some(spans) = message.get("spans").and_then(|s| s.as_array()) {
                        for span in spans {
                            if let Some(span_file) = span.get("file_name").and_then(|f| f.as_str()) {
                                // 只处理当前文件的问题
                                if span_file.ends_with(file_path) || file_path.ends_with(span_file) {
                                    let line_start = span.get("line_start")
                                        .and_then(|l| l.as_u64())
                                        .unwrap_or(0) as usize;
                                    let column_start = span.get("column_start")
                                        .and_then(|c| c.as_u64())
                                        .unwrap_or(0) as usize;

                                    let msg = message.get("message")
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("Compilation error");

                                    let level = message.get("level")
                                        .and_then(|l| l.as_str())
                                        .unwrap_or("error");

                                    let code = message.get("code")
                                        .and_then(|c| c.get("code"))
                                        .and_then(|c| c.as_str());

                                    let severity = self.determine_severity(level, msg);
                                    let category = self.determine_category(msg);
                                    let suggestion = self.get_suggestion(msg);

                                    let mut issue = Issue::new(
                                        "cargo check".to_string(),
                                        file_path.to_string(),
                                        severity,
                                        category,
                                        msg.to_string(),
                                    )
                                    .with_location(line_start, Some(column_start))
                                    .with_suggestion(suggestion);

                                    if let Some(code) = code {
                                        issue = issue.with_rule_id(code.to_string());
                                    }

                                    issues.push(issue);
                                }
                            }
                        }
                    }
                }
            }
        }

        issues
    }

    fn determine_severity(&self, level: &str, message: &str) -> Severity {
        match level {
            "error" => {
                let message_lower = message.to_lowercase();
                if message_lower.contains("cannot find") ||
                   message_lower.contains("undefined") ||
                   message_lower.contains("mismatched types") {
                    Severity::Critical
                } else {
                    Severity::High
                }
            }
            "warning" => {
                let message_lower = message.to_lowercase();
                if message_lower.contains("unused") ||
                   message_lower.contains("dead_code") {
                    Severity::Medium
                } else {
                    Severity::Low
                }
            }
            _ => Severity::Info,
        }
    }

    fn determine_category(&self, message: &str) -> IssueCategory {
        let message_lower = message.to_lowercase();

        if message_lower.contains("cannot find") ||
           message_lower.contains("undefined") ||
           message_lower.contains("mismatched types") ||
           message_lower.contains("borrow checker") {
            IssueCategory::Bug
        } else if message_lower.contains("unused") ||
                  message_lower.contains("dead_code") {
            IssueCategory::Maintainability
        } else {
            IssueCategory::Style
        }
    }

    fn get_suggestion(&self, message: &str) -> String {
        let message_lower = message.to_lowercase();

        if message_lower.contains("cannot find") {
            "检查模块导入和依赖声明".to_string()
        } else if message_lower.contains("mismatched types") {
            "检查类型匹配和类型转换".to_string()
        } else if message_lower.contains("borrow checker") {
            "检查所有权和借用规则".to_string()
        } else if message_lower.contains("unused") {
            "移除未使用的代码或添加 #[allow(dead_code)]".to_string()
        } else {
            "请查看编译错误详细信息".to_string()
        }
    }
}

#[async_trait]
impl StaticAnalysisTool for CargoCheckTool {
    fn name(&self) -> &str {
        "cargo check"
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::Rust]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        let output = tokio::process::Command::new("cargo")
            .args(["check", "--message-format=json"])
            .current_dir(std::path::Path::new(file_path).parent().unwrap_or(std::path::Path::new(".")))
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let issues = self.parse_cargo_check_json(&stdout, file_path);

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        Command::new("cargo")
            .args(["--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::static_analysis::{Severity, IssueCategory};

    #[tokio::test]
    async fn test_rustfmt_tool_creation() {
        let tool = RustFmtTool::new();
        assert_eq!(tool.name(), "rustfmt");
        assert_eq!(tool.supported_languages(), vec![Language::Rust]);
    }

    #[tokio::test]
    async fn test_clippy_tool_creation() {
        let tool = ClippyTool::new();
        assert_eq!(tool.name(), "clippy");
        assert_eq!(tool.supported_languages(), vec![Language::Rust]);
    }

    #[tokio::test]
    async fn test_cargo_check_tool_creation() {
        let tool = CargoCheckTool::new();
        assert_eq!(tool.name(), "cargo check");
        assert_eq!(tool.supported_languages(), vec![Language::Rust]);
    }

    #[test]
    fn test_clippy_json_parsing() {
        let tool = ClippyTool::new();
        let json_output = r#"{"message":{"message":"this function has too many arguments (8/7)","code":{"code":"clippy::too_many_arguments","explanation":null},"level":"warning","spans":[{"file_name":"src/main.rs","byte_start":123,"byte_end":456,"line_start":10,"line_end":10,"column_start":5,"column_end":15,"is_primary":true,"text":[{"text":"fn complex_function(","highlight_start":5,"highlight_end":15}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"warning: this function has too many arguments (8/7)\n"},"target":{"kind":["bin"],"crate_types":["bin"],"name":"test","src_path":"src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true}}"#;
        let issues = tool.parse_clippy_json(json_output, "src/main.rs");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line_number, Some(10));
        assert_eq!(issues[0].column_number, Some(5));
        assert_eq!(issues[0].severity, Severity::Low);
        assert_eq!(issues[0].rule_id, Some("clippy::too_many_arguments".to_string()));
    }

    #[test]
    fn test_cargo_check_json_parsing() {
        let tool = CargoCheckTool::new();
        let json_output = r#"{"message":{"message":"cannot find value `undefined_var` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used."},"level":"error","spans":[{"file_name":"src/main.rs","byte_start":200,"byte_end":213,"line_start":15,"line_end":15,"column_start":10,"column_end":23,"is_primary":true,"text":[{"text":"    println!(\"{}\", undefined_var);","highlight_start":10,"highlight_end":23}],"label":"not found in this scope","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0425]: cannot find value `undefined_var` in this scope\n"},"target":{"kind":["bin"],"crate_types":["bin"],"name":"test","src_path":"src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true}}"#;
        let issues = tool.parse_cargo_check_json(json_output, "src/main.rs");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line_number, Some(15));
        assert_eq!(issues[0].column_number, Some(10));
        assert_eq!(issues[0].severity, Severity::Critical);
        assert_eq!(issues[0].category, IssueCategory::Bug);
        assert_eq!(issues[0].rule_id, Some("E0425".to_string()));
    }

    #[test]
    fn test_clippy_severity_determination() {
        let tool = ClippyTool::new();

        assert_eq!(tool.determine_severity("error", "compilation error"), Severity::High);
        assert_eq!(tool.determine_severity("warning", "unsafe block"), Severity::Medium);
        assert_eq!(tool.determine_severity("warning", "style issue"), Severity::Low);
        assert_eq!(tool.determine_severity("note", "info"), Severity::Info);
    }

    #[test]
    fn test_cargo_check_severity_determination() {
        let tool = CargoCheckTool::new();

        assert_eq!(tool.determine_severity("error", "cannot find value"), Severity::Critical);
        assert_eq!(tool.determine_severity("error", "compilation error"), Severity::High);
        assert_eq!(tool.determine_severity("warning", "unused variable"), Severity::Medium);
        assert_eq!(tool.determine_severity("warning", "style warning"), Severity::Low);
    }

    #[test]
    fn test_clippy_category_determination() {
        let tool = ClippyTool::new();

        assert_eq!(tool.determine_category("unsafe block"), IssueCategory::Bug);
        assert_eq!(tool.determine_category("performance issue"), IssueCategory::Performance);
        assert_eq!(tool.determine_category("complexity warning"), IssueCategory::Complexity);
        assert_eq!(tool.determine_category("unused variable"), IssueCategory::Maintainability);
        assert_eq!(tool.determine_category("style issue"), IssueCategory::Style);
    }

    #[test]
    fn test_cargo_check_category_determination() {
        let tool = CargoCheckTool::new();

        assert_eq!(tool.determine_category("cannot find value"), IssueCategory::Bug);
        assert_eq!(tool.determine_category("mismatched types"), IssueCategory::Bug);
        assert_eq!(tool.determine_category("borrow checker error"), IssueCategory::Bug);
        assert_eq!(tool.determine_category("unused variable"), IssueCategory::Maintainability);
        assert_eq!(tool.determine_category("style warning"), IssueCategory::Style);
    }
}