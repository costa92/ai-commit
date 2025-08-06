use std::process::Command;
use async_trait::async_trait;
use regex::Regex;
use serde_json::Value;
use crate::languages::Language;
use crate::analysis::static_analysis::{StaticAnalysisTool, Issue, Severity, IssueCategory};

/// TSLint 工具 (已弃用，但仍可能在一些项目中使用)
pub struct TSLintTool;

impl TSLintTool {
    pub fn new() -> Self {
        Self
    }

    fn parse_tslint_output(&self, output: &str, file_path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        // TSLint 输出格式: ERROR: filename[line, column]: message (rule-name)
        let re = Regex::new(r"(ERROR|WARNING):\s*([^\[]+)\[(\d+),\s*(\d+)\]:\s*(.+?)\s*\(([^)]+)\)").unwrap();

        for line in output.lines() {
            if let Some(captures) = re.captures(line) {
                let level = captures.get(1).unwrap().as_str();
                let reported_file = captures.get(2).unwrap().as_str().trim();
                let line_num: usize = captures.get(3).unwrap().as_str().parse().unwrap_or(0);
                let col_num: usize = captures.get(4).unwrap().as_str().parse().unwrap_or(0);
                let message = captures.get(5).unwrap().as_str();
                let rule_name = captures.get(6).unwrap().as_str();

                // 只处理当前文件的问题
                if reported_file.ends_with(file_path) || file_path.ends_with(reported_file) {
                    let severity = self.determine_severity(level, message);
                    let category = self.determine_category(rule_name, message);
                    let suggestion = self.get_suggestion(rule_name, message);

                    issues.push(
                        Issue::new(
                            "tslint".to_string(),
                            file_path.to_string(),
                            severity,
                            category,
                            message.to_string(),
                        )
                        .with_location(line_num, Some(col_num))
                        .with_rule_id(rule_name.to_string())
                        .with_suggestion(suggestion)
                    );
                }
            }
        }

        issues
    }

    fn determine_severity(&self, level: &str, message: &str) -> Severity {
        match level {
            "ERROR" => {
                let message_lower = message.to_lowercase();
                if message_lower.contains("syntax") ||
                   message_lower.contains("parse") {
                    Severity::Critical
                } else {
                    Severity::High
                }
            }
            "WARNING" => Severity::Medium,
            _ => Severity::Low,
        }
    }

    fn determine_category(&self, rule_name: &str, message: &str) -> IssueCategory {
        let rule_lower = rule_name.to_lowercase();
        let message_lower = message.to_lowercase();

        if rule_lower.contains("security") ||
           message_lower.contains("security") {
            IssueCategory::Security
        } else if rule_lower.contains("performance") ||
                  message_lower.contains("performance") {
            IssueCategory::Performance
        } else if rule_lower.contains("complexity") ||
                  message_lower.contains("complexity") {
            IssueCategory::Complexity
        } else if rule_lower.contains("unused") ||
                  rule_lower.contains("no-unused") {
            IssueCategory::Maintainability
        } else {
            IssueCategory::Style
        }
    }

    fn get_suggestion(&self, rule_name: &str, message: &str) -> String {
        let rule_lower = rule_name.to_lowercase();

        if rule_lower.contains("no-unused") {
            "移除未使用的变量或导入".to_string()
        } else if rule_lower.contains("no-any") {
            "使用具体类型替代 any".to_string()
        } else if rule_lower.contains("prefer-const") {
            "使用 const 替代 let".to_string()
        } else if rule_lower.contains("semicolon") {
            "添加或移除分号".to_string()
        } else {
            format!("请查看 TSLint 规则 {} 的文档", rule_name)
        }
    }
}

#[async_trait]
impl StaticAnalysisTool for TSLintTool {
    fn name(&self) -> &str {
        "tslint"
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::TypeScript]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        let output = tokio::process::Command::new("tslint")
            .args([file_path])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let issues = self.parse_tslint_output(&stdout, file_path);

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        Command::new("tslint")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// ESLint 工具
pub struct ESLintTool;

impl ESLintTool {
    pub fn new() -> Self {
        Self
    }

    fn parse_eslint_json(&self, json_output: &str, file_path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        if let Ok(json) = serde_json::from_str::<Value>(json_output) {
            if let Some(files) = json.as_array() {
                for file_result in files {
                    if let Some(file_path_json) = file_result.get("filePath").and_then(|f| f.as_str()) {
                        // 只处理当前文件的问题
                        if file_path_json.ends_with(file_path) || file_path.ends_with(file_path_json) {
                            if let Some(messages) = file_result.get("messages").and_then(|m| m.as_array()) {
                                for message in messages {
                                    let line_num = message.get("line")
                                        .and_then(|l| l.as_u64())
                                        .unwrap_or(0) as usize;
                                    let col_num = message.get("column")
                                        .and_then(|c| c.as_u64())
                                        .unwrap_or(0) as usize;

                                    let msg = message.get("message")
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("ESLint error");

                                    let severity_num = message.get("severity")
                                        .and_then(|s| s.as_u64())
                                        .unwrap_or(1);

                                    let rule_id = message.get("ruleId")
                                        .and_then(|r| r.as_str());

                                    let severity = self.determine_severity(severity_num, msg);
                                    let category = self.determine_category(rule_id.unwrap_or(""), msg);
                                    let suggestion = self.get_suggestion(rule_id.unwrap_or(""), msg);

                                    let mut issue = Issue::new(
                                        "eslint".to_string(),
                                        file_path.to_string(),
                                        severity,
                                        category,
                                        msg.to_string(),
                                    )
                                    .with_location(line_num, Some(col_num))
                                    .with_suggestion(suggestion);

                                    if let Some(rule) = rule_id {
                                        issue = issue.with_rule_id(rule.to_string());
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

    fn determine_severity(&self, severity_num: u64, message: &str) -> Severity {
        match severity_num {
            2 => {
                let message_lower = message.to_lowercase();
                if message_lower.contains("syntax") ||
                   message_lower.contains("parse") {
                    Severity::Critical
                } else {
                    Severity::High
                }
            }
            1 => Severity::Medium,
            _ => Severity::Low,
        }
    }

    fn determine_category(&self, rule_id: &str, message: &str) -> IssueCategory {
        let rule_lower = rule_id.to_lowercase();
        let message_lower = message.to_lowercase();

        if rule_lower.contains("security") ||
           message_lower.contains("security") {
            IssueCategory::Security
        } else if rule_lower.contains("performance") ||
                  message_lower.contains("performance") {
            IssueCategory::Performance
        } else if rule_lower.contains("complexity") ||
                  message_lower.contains("complexity") {
            IssueCategory::Complexity
        } else if rule_lower.contains("unused") ||
                  rule_lower.contains("no-unused") {
            IssueCategory::Maintainability
        } else {
            IssueCategory::Style
        }
    }

    fn get_suggestion(&self, rule_id: &str, message: &str) -> String {
        let rule_lower = rule_id.to_lowercase();

        if rule_lower.contains("no-unused") {
            "移除未使用的变量或导入".to_string()
        } else if rule_lower.contains("no-undef") {
            "定义变量或添加类型声明".to_string()
        } else if rule_lower.contains("prefer-const") {
            "使用 const 替代 let".to_string()
        } else if rule_lower.contains("semicolon") {
            "添加或移除分号".to_string()
        } else if rule_lower.contains("quotes") {
            "统一引号风格".to_string()
        } else {
            format!("请查看 ESLint 规则 {} 的文档", rule_id)
        }
    }
}

#[async_trait]
impl StaticAnalysisTool for ESLintTool {
    fn name(&self) -> &str {
        "eslint"
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::TypeScript]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        let output = tokio::process::Command::new("eslint")
            .args(["--format", "json", file_path])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let issues = self.parse_eslint_json(&stdout, file_path);

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        Command::new("eslint")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// TypeScript 编译器工具
pub struct TypeScriptCompilerTool;

impl TypeScriptCompilerTool {
    pub fn new() -> Self {
        Self
    }

    fn parse_tsc_output(&self, output: &str, file_path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        // TypeScript 编译器输出格式: filename(line,column): error TS####: message
        let re = Regex::new(r"([^(]+)\((\d+),(\d+)\):\s*(error|warning)\s*TS(\d+):\s*(.+)").unwrap();

        for line in output.lines() {
            if let Some(captures) = re.captures(line) {
                let reported_file = captures.get(1).unwrap().as_str().trim();
                let line_num: usize = captures.get(2).unwrap().as_str().parse().unwrap_or(0);
                let col_num: usize = captures.get(3).unwrap().as_str().parse().unwrap_or(0);
                let level = captures.get(4).unwrap().as_str();
                let error_code = captures.get(5).unwrap().as_str();
                let message = captures.get(6).unwrap().as_str();

                // 只处理当前文件的问题
                if reported_file.ends_with(file_path) || file_path.ends_with(reported_file) {
                    let severity = self.determine_severity(level, error_code, message);
                    let category = self.determine_category(error_code, message);
                    let suggestion = self.get_suggestion(error_code, message);

                    issues.push(
                        Issue::new(
                            "tsc".to_string(),
                            file_path.to_string(),
                            severity,
                            category,
                            message.to_string(),
                        )
                        .with_location(line_num, Some(col_num))
                        .with_rule_id(format!("TS{}", error_code))
                        .with_suggestion(suggestion)
                    );
                }
            }
        }

        issues
    }

    fn determine_severity(&self, level: &str, error_code: &str, message: &str) -> Severity {
        match level {
            "error" => {
                let message_lower = message.to_lowercase();
                if message_lower.contains("cannot find") ||
                   message_lower.contains("does not exist") ||
                   error_code == "2307" || // Cannot find module
                   error_code == "2304" {  // Cannot find name
                    Severity::Critical
                } else {
                    Severity::High
                }
            }
            "warning" => Severity::Medium,
            _ => Severity::Low,
        }
    }

    fn determine_category(&self, error_code: &str, message: &str) -> IssueCategory {
        let message_lower = message.to_lowercase();

        match error_code {
            "2307" | "2304" | "2339" => IssueCategory::Bug, // Module/name/property not found
            "2322" | "2345" => IssueCategory::Bug, // Type mismatch
            "6133" | "6196" => IssueCategory::Maintainability, // Unused variables
            "2377" | "2394" => IssueCategory::Complexity, // Overload/duplicate issues
            _ => {
                if message_lower.contains("type") {
                    IssueCategory::Bug
                } else if message_lower.contains("unused") {
                    IssueCategory::Maintainability
                } else {
                    IssueCategory::Style
                }
            }
        }
    }

    fn get_suggestion(&self, error_code: &str, message: &str) -> String {
        match error_code {
            "2307" => "检查模块路径和导入语句".to_string(),
            "2304" => "检查变量名拼写和作用域".to_string(),
            "2339" => "检查属性名和类型定义".to_string(),
            "2322" | "2345" => "检查类型匹配和类型转换".to_string(),
            "6133" | "6196" => "移除未使用的变量或添加下划线前缀".to_string(),
            _ => {
                let message_lower = message.to_lowercase();
                if message_lower.contains("cannot find") {
                    "检查导入路径和模块声明".to_string()
                } else if message_lower.contains("type") {
                    "检查类型定义和类型注解".to_string()
                } else {
                    "请查看 TypeScript 编译器错误文档".to_string()
                }
            }
        }
    }
}

#[async_trait]
impl StaticAnalysisTool for TypeScriptCompilerTool {
    fn name(&self) -> &str {
        "tsc"
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::TypeScript]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        let output = tokio::process::Command::new("tsc")
            .args(["--noEmit", "--pretty", "false", file_path])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // TypeScript 编译器可能将错误输出到 stdout 或 stderr
        let combined_output = format!("{}\n{}", stdout, stderr);
        let issues = self.parse_tsc_output(&combined_output, file_path);

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        Command::new("tsc")
            .arg("--version")
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
    async fn test_tslint_tool_creation() {
        let tool = TSLintTool::new();
        assert_eq!(tool.name(), "tslint");
        assert_eq!(tool.supported_languages(), vec![Language::TypeScript]);
    }

    #[tokio::test]
    async fn test_eslint_tool_creation() {
        let tool = ESLintTool::new();
        assert_eq!(tool.name(), "eslint");
        assert_eq!(tool.supported_languages(), vec![Language::TypeScript]);
    }

    #[tokio::test]
    async fn test_typescript_compiler_tool_creation() {
        let tool = TypeScriptCompilerTool::new();
        assert_eq!(tool.name(), "tsc");
        assert_eq!(tool.supported_languages(), vec![Language::TypeScript]);
    }

    #[test]
    fn test_tslint_output_parsing() {
        let tool = TSLintTool::new();
        let output = "ERROR: src/main.ts[10, 5]: Variable 'x' is declared but never used (no-unused-variable)";
        let issues = tool.parse_tslint_output(output, "src/main.ts");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line_number, Some(10));
        assert_eq!(issues[0].column_number, Some(5));
        assert_eq!(issues[0].severity, Severity::High);
        assert_eq!(issues[0].rule_id, Some("no-unused-variable".to_string()));
    }

    #[test]
    fn test_eslint_json_parsing() {
        let tool = ESLintTool::new();
        let json_output = r#"[{"filePath":"src/main.ts","messages":[{"ruleId":"no-unused-vars","severity":2,"message":"'x' is defined but never used.","line":10,"column":5,"nodeType":"Identifier","messageId":"unusedVar","endLine":10,"endColumn":6}],"errorCount":1,"warningCount":0,"fixableErrorCount":0,"fixableWarningCount":0,"source":"const x = 5;\nconsole.log('hello');"}]"#;
        let issues = tool.parse_eslint_json(json_output, "src/main.ts");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line_number, Some(10));
        assert_eq!(issues[0].column_number, Some(5));
        assert_eq!(issues[0].severity, Severity::High);
        assert_eq!(issues[0].rule_id, Some("no-unused-vars".to_string()));
    }

    #[test]
    fn test_tsc_output_parsing() {
        let tool = TypeScriptCompilerTool::new();
        let output = "src/main.ts(15,10): error TS2304: Cannot find name 'undefinedVar'.";
        let issues = tool.parse_tsc_output(output, "src/main.ts");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line_number, Some(15));
        assert_eq!(issues[0].column_number, Some(10));
        assert_eq!(issues[0].severity, Severity::Critical);
        assert_eq!(issues[0].category, IssueCategory::Bug);
        assert_eq!(issues[0].rule_id, Some("TS2304".to_string()));
    }

    #[test]
    fn test_tslint_severity_determination() {
        let tool = TSLintTool::new();

        assert_eq!(tool.determine_severity("ERROR", "syntax error"), Severity::Critical);
        assert_eq!(tool.determine_severity("ERROR", "type error"), Severity::High);
        assert_eq!(tool.determine_severity("WARNING", "style warning"), Severity::Medium);
    }

    #[test]
    fn test_eslint_severity_determination() {
        let tool = ESLintTool::new();

        assert_eq!(tool.determine_severity(2, "syntax error"), Severity::Critical);
        assert_eq!(tool.determine_severity(2, "type error"), Severity::High);
        assert_eq!(tool.determine_severity(1, "style warning"), Severity::Medium);
        assert_eq!(tool.determine_severity(0, "info"), Severity::Low);
    }

    #[test]
    fn test_tsc_severity_determination() {
        let tool = TypeScriptCompilerTool::new();

        assert_eq!(tool.determine_severity("error", "2307", "Cannot find module"), Severity::Critical);
        assert_eq!(tool.determine_severity("error", "2304", "Cannot find name"), Severity::Critical);
        assert_eq!(tool.determine_severity("error", "2322", "Type error"), Severity::High);
        assert_eq!(tool.determine_severity("warning", "6133", "Unused variable"), Severity::Medium);
    }

    #[test]
    fn test_tsc_category_determination() {
        let tool = TypeScriptCompilerTool::new();

        assert_eq!(tool.determine_category("2307", "Cannot find module"), IssueCategory::Bug);
        assert_eq!(tool.determine_category("2304", "Cannot find name"), IssueCategory::Bug);
        assert_eq!(tool.determine_category("6133", "Unused variable"), IssueCategory::Maintainability);
        assert_eq!(tool.determine_category("2377", "Overload signature"), IssueCategory::Complexity);
    }
}