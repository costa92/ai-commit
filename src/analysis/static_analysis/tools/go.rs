use std::process::Command;
use async_trait::async_trait;
use regex::Regex;
use crate::languages::Language;
use crate::analysis::static_analysis::{StaticAnalysisTool, Issue, Severity, IssueCategory};

/// Go 格式化工具
pub struct GoFmtTool;

impl GoFmtTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl StaticAnalysisTool for GoFmtTool {
    fn name(&self) -> &str {
        "gofmt"
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::Go]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        let output = tokio::process::Command::new("gofmt")
            .args(["-d", file_path])
            .output()
            .await?;

        let mut issues = Vec::new();

        if !output.stdout.is_empty() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            issues.push(
                Issue::new(
                    "gofmt".to_string(),
                    file_path.to_string(),
                    Severity::Low,
                    IssueCategory::Style,
                    "代码格式不符合 Go 标准格式".to_string(),
                )
                .with_suggestion("运行 'gofmt -w filename.go' 自动格式化代码".to_string())
                .with_rule_id("gofmt-format".to_string())
                .with_code_snippet(stdout.to_string())
            );
        }

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        Command::new("gofmt")
            .arg("--help")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Go Vet 工具
pub struct GoVetTool;

impl GoVetTool {
    pub fn new() -> Self {
        Self
    }

    fn parse_vet_output(&self, output: &str, file_path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        // go vet 输出格式: filename:line:column: message
        let re = Regex::new(r"([^:]+):(\d+):(\d+):\s*(.+)").unwrap();

        for line in output.lines() {
            if let Some(captures) = re.captures(line) {
                let reported_file = captures.get(1).unwrap().as_str();
                let line_num: usize = captures.get(2).unwrap().as_str().parse().unwrap_or(0);
                let col_num: usize = captures.get(3).unwrap().as_str().parse().unwrap_or(0);
                let message = captures.get(4).unwrap().as_str();

                // 只处理当前文件的问题
                if reported_file.ends_with(file_path) || file_path.ends_with(reported_file) {
                    let severity = self.determine_severity(message);
                    let category = self.determine_category(message);

                    issues.push(
                        Issue::new(
                            "go vet".to_string(),
                            file_path.to_string(),
                            severity,
                            category,
                            message.to_string(),
                        )
                        .with_location(line_num, Some(col_num))
                        .with_rule_id("go-vet".to_string())
                        .with_suggestion(self.get_suggestion(message))
                    );
                }
            }
        }

        issues
    }

    fn determine_severity(&self, message: &str) -> Severity {
        let message_lower = message.to_lowercase();

        if message_lower.contains("unreachable") ||
           message_lower.contains("nil pointer") ||
           message_lower.contains("panic") {
            Severity::High
        } else if message_lower.contains("unused") ||
                  message_lower.contains("shadow") {
            Severity::Medium
        } else {
            Severity::Low
        }
    }

    fn determine_category(&self, message: &str) -> IssueCategory {
        let message_lower = message.to_lowercase();

        if message_lower.contains("unreachable") ||
           message_lower.contains("nil pointer") ||
           message_lower.contains("panic") {
            IssueCategory::Bug
        } else if message_lower.contains("unused") {
            IssueCategory::Maintainability
        } else {
            IssueCategory::Style
        }
    }

    fn get_suggestion(&self, message: &str) -> String {
        let message_lower = message.to_lowercase();

        if message_lower.contains("unused") {
            "移除未使用的变量或函数".to_string()
        } else if message_lower.contains("unreachable") {
            "移除不可达的代码".to_string()
        } else if message_lower.contains("nil pointer") {
            "添加 nil 检查".to_string()
        } else {
            "请查看 Go vet 文档了解详细信息".to_string()
        }
    }
}

#[async_trait]
impl StaticAnalysisTool for GoVetTool {
    fn name(&self) -> &str {
        "go vet"
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::Go]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        let output = tokio::process::Command::new("go")
            .args(["vet", file_path])
            .output()
            .await?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let issues = self.parse_vet_output(&stderr, file_path);

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        Command::new("go")
            .args(["version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Go Lint 工具 (golint)
pub struct GoLintTool;

impl GoLintTool {
    pub fn new() -> Self {
        Self
    }

    fn parse_lint_output(&self, output: &str, file_path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        // golint 输出格式: filename:line:column: message
        let re = Regex::new(r"([^:]+):(\d+):(\d+):\s*(.+)").unwrap();

        for line in output.lines() {
            if let Some(captures) = re.captures(line) {
                let reported_file = captures.get(1).unwrap().as_str();
                let line_num: usize = captures.get(2).unwrap().as_str().parse().unwrap_or(0);
                let col_num: usize = captures.get(3).unwrap().as_str().parse().unwrap_or(0);
                let message = captures.get(4).unwrap().as_str();

                // 只处理当前文件的问题
                if reported_file.ends_with(file_path) || file_path.ends_with(reported_file) {
                    issues.push(
                        Issue::new(
                            "golint".to_string(),
                            file_path.to_string(),
                            Severity::Low, // golint 主要是风格问题
                            IssueCategory::Style,
                            message.to_string(),
                        )
                        .with_location(line_num, Some(col_num))
                        .with_rule_id("golint".to_string())
                        .with_suggestion("请遵循 Go 代码风格指南".to_string())
                    );
                }
            }
        }

        issues
    }
}

#[async_trait]
impl StaticAnalysisTool for GoLintTool {
    fn name(&self) -> &str {
        "golint"
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::Go]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        let output = tokio::process::Command::new("golint")
            .arg(file_path)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let issues = self.parse_lint_output(&stdout, file_path);

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        Command::new("golint")
            .arg("--help")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Go Build 工具
pub struct GoBuildTool;

impl GoBuildTool {
    pub fn new() -> Self {
        Self
    }

    fn parse_build_output(&self, output: &str, file_path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        // go build 输出格式: filename:line:column: message
        let re = Regex::new(r"([^:]+):(\d+):(\d+):\s*(.+)").unwrap();

        for line in output.lines() {
            if let Some(captures) = re.captures(line) {
                let reported_file = captures.get(1).unwrap().as_str();
                let line_num: usize = captures.get(2).unwrap().as_str().parse().unwrap_or(0);
                let col_num: usize = captures.get(3).unwrap().as_str().parse().unwrap_or(0);
                let message = captures.get(4).unwrap().as_str();

                // 只处理当前文件的问题
                if reported_file.ends_with(file_path) || file_path.ends_with(reported_file) {
                    let severity = self.determine_severity(message);
                    let category = self.determine_category(message);

                    issues.push(
                        Issue::new(
                            "go build".to_string(),
                            file_path.to_string(),
                            severity,
                            category,
                            message.to_string(),
                        )
                        .with_location(line_num, Some(col_num))
                        .with_rule_id("go-build".to_string())
                        .with_suggestion(self.get_suggestion(message))
                    );
                }
            }
        }

        issues
    }

    fn determine_severity(&self, message: &str) -> Severity {
        let message_lower = message.to_lowercase();

        if message_lower.contains("undefined") ||
           message_lower.contains("cannot use") ||
           message_lower.contains("type") {
            Severity::High
        } else if message_lower.contains("declared and not used") {
            Severity::Medium
        } else {
            Severity::Low
        }
    }

    fn determine_category(&self, message: &str) -> IssueCategory {
        let message_lower = message.to_lowercase();

        if message_lower.contains("undefined") ||
           message_lower.contains("cannot use") ||
           message_lower.contains("type") {
            IssueCategory::Bug
        } else if message_lower.contains("declared and not used") {
            IssueCategory::Maintainability
        } else {
            IssueCategory::Style
        }
    }

    fn get_suggestion(&self, message: &str) -> String {
        let message_lower = message.to_lowercase();

        if message_lower.contains("undefined") {
            "检查变量或函数是否正确定义和导入".to_string()
        } else if message_lower.contains("declared and not used") {
            "移除未使用的变量或使用下划线前缀".to_string()
        } else if message_lower.contains("cannot use") {
            "检查类型是否匹配".to_string()
        } else {
            "请查看编译错误详细信息".to_string()
        }
    }
}

#[async_trait]
impl StaticAnalysisTool for GoBuildTool {
    fn name(&self) -> &str {
        "go build"
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::Go]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        // 使用 go build -o /dev/null 来检查编译错误而不生成文件
        let output = tokio::process::Command::new("go")
            .args(["build", "-o", "/dev/null", file_path])
            .output()
            .await?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let issues = self.parse_build_output(&stderr, file_path);

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        Command::new("go")
            .args(["version"])
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
    async fn test_gofmt_tool_creation() {
        let tool = GoFmtTool::new();
        assert_eq!(tool.name(), "gofmt");
        assert_eq!(tool.supported_languages(), vec![Language::Go]);
    }

    #[tokio::test]
    async fn test_govet_tool_creation() {
        let tool = GoVetTool::new();
        assert_eq!(tool.name(), "go vet");
        assert_eq!(tool.supported_languages(), vec![Language::Go]);
    }

    #[tokio::test]
    async fn test_golint_tool_creation() {
        let tool = GoLintTool::new();
        assert_eq!(tool.name(), "golint");
        assert_eq!(tool.supported_languages(), vec![Language::Go]);
    }

    #[tokio::test]
    async fn test_gobuild_tool_creation() {
        let tool = GoBuildTool::new();
        assert_eq!(tool.name(), "go build");
        assert_eq!(tool.supported_languages(), vec![Language::Go]);
    }

    #[test]
    fn test_govet_output_parsing() {
        let tool = GoVetTool::new();
        let output = "main.go:10:5: unreachable code\nmain.go:15:10: unused variable x";
        let issues = tool.parse_vet_output(output, "main.go");

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].line_number, Some(10));
        assert_eq!(issues[0].column_number, Some(5));
        assert_eq!(issues[0].severity, Severity::High);
        assert_eq!(issues[1].line_number, Some(15));
        assert_eq!(issues[1].severity, Severity::Medium);
    }

    #[test]
    fn test_golint_output_parsing() {
        let tool = GoLintTool::new();
        let output = "main.go:5:1: exported function Foo should have comment";
        let issues = tool.parse_lint_output(output, "main.go");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line_number, Some(5));
        assert_eq!(issues[0].column_number, Some(1));
        assert_eq!(issues[0].severity, Severity::Low);
        assert_eq!(issues[0].category, IssueCategory::Style);
    }

    #[test]
    fn test_gobuild_output_parsing() {
        let tool = GoBuildTool::new();
        let output = "main.go:8:2: undefined: fmt.Printl\nmain.go:12:5: x declared and not used";
        let issues = tool.parse_build_output(output, "main.go");

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].line_number, Some(8));
        assert_eq!(issues[0].severity, Severity::High);
        assert_eq!(issues[0].category, IssueCategory::Bug);
        assert_eq!(issues[1].line_number, Some(12));
        assert_eq!(issues[1].severity, Severity::Medium);
        assert_eq!(issues[1].category, IssueCategory::Maintainability);
    }

    #[test]
    fn test_govet_severity_determination() {
        let tool = GoVetTool::new();

        assert_eq!(tool.determine_severity("unreachable code"), Severity::High);
        assert_eq!(tool.determine_severity("nil pointer dereference"), Severity::High);
        assert_eq!(tool.determine_severity("unused variable x"), Severity::Medium);
        assert_eq!(tool.determine_severity("missing return"), Severity::Low);
    }

    #[test]
    fn test_gobuild_severity_determination() {
        let tool = GoBuildTool::new();

        assert_eq!(tool.determine_severity("undefined: fmt.Printl"), Severity::High);
        assert_eq!(tool.determine_severity("cannot use string as int"), Severity::High);
        assert_eq!(tool.determine_severity("x declared and not used"), Severity::Medium);
    }
}