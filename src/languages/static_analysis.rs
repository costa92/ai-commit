use crate::languages::Language;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::process::Command;

/// 静态分析工具枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StaticAnalysisTool {
    // Go 工具
    GoFmt,
    GoVet,
    GoLint,
    // 可扩展的其他语言工具
    // RustFmt,
    // RustClippy,
    // ESLint,
    // Prettier,
}

/// 静态分析问题严重级别
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
    Style,
}

/// 静态分析发现的问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisIssue {
    pub tool: StaticAnalysisTool,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub column: Option<usize>,
    pub severity: IssueSeverity,
    pub message: String,
    pub rule: Option<String>,
    pub suggestion: Option<String>,
}

/// 静态分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisResult {
    pub tool: StaticAnalysisTool,
    pub issues: Vec<StaticAnalysisIssue>,
    pub execution_time: std::time::Duration,
    pub success: bool,
    pub error_message: Option<String>,
}

/// 静态分析服务
pub struct StaticAnalysisService {
    enabled_tools: HashMap<Language, Vec<StaticAnalysisTool>>,
}

impl Default for StaticAnalysisService {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticAnalysisService {
    pub fn new() -> Self {
        let mut enabled_tools = HashMap::new();

        // 默认为 Go 语言启用所有工具
        enabled_tools.insert(
            Language::Go,
            vec![
                StaticAnalysisTool::GoFmt,
                StaticAnalysisTool::GoVet,
                StaticAnalysisTool::GoLint,
            ],
        );

        Self { enabled_tools }
    }

    /// 检查工具是否可用
    pub async fn check_tool_availability(&self, tool: &StaticAnalysisTool) -> bool {
        let command = match tool {
            StaticAnalysisTool::GoFmt => "gofmt",
            StaticAnalysisTool::GoVet => "go",
            StaticAnalysisTool::GoLint => "golint",
        };

        let result = Command::new(command).arg("--help").output().await;

        match result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// 运行 gofmt 格式检查
    pub async fn run_gofmt(&self, file_path: &str) -> anyhow::Result<StaticAnalysisResult> {
        let start_time = std::time::Instant::now();

        let output = Command::new("gofmt")
            .args(["-d", file_path]) // -d 参数显示格式差异
            .output()
            .await?;

        let execution_time = start_time.elapsed();
        let success = output.status.success();

        if !success {
            return Ok(StaticAnalysisResult {
                tool: StaticAnalysisTool::GoFmt,
                issues: vec![],
                execution_time,
                success: false,
                error_message: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut issues = Vec::new();

        // 解析 gofmt 输出，如果有差异说明格式不符合标准
        if !stdout.trim().is_empty() {
            issues.push(StaticAnalysisIssue {
                tool: StaticAnalysisTool::GoFmt,
                file_path: file_path.to_string(),
                line_number: None,
                column: None,
                severity: IssueSeverity::Style,
                message: "代码格式不符合 Go 标准格式".to_string(),
                rule: Some("gofmt".to_string()),
                suggestion: Some("运行 'gofmt -w filename.go' 自动格式化代码".to_string()),
            });
        }

        Ok(StaticAnalysisResult {
            tool: StaticAnalysisTool::GoFmt,
            issues,
            execution_time,
            success: true,
            error_message: None,
        })
    }

    /// 运行 go vet 静态分析
    pub async fn run_go_vet(&self, file_path: &str) -> anyhow::Result<StaticAnalysisResult> {
        let start_time = std::time::Instant::now();

        // go vet 需要在包目录中运行
        let dir = Path::new(file_path).parent().unwrap_or(Path::new("."));

        let output = Command::new("go")
            .args(["vet", file_path])
            .current_dir(dir)
            .output()
            .await?;

        let execution_time = start_time.elapsed();
        let success = output.status.success();

        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut issues = Vec::new();

        // 解析 go vet 输出
        for line in stderr.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // go vet 输出格式通常是: filename:line:column: message
            if let Some(issue) = parse_go_vet_line(line, file_path) {
                issues.push(issue);
            }
        }

        Ok(StaticAnalysisResult {
            tool: StaticAnalysisTool::GoVet,
            issues,
            execution_time,
            success,
            error_message: if success {
                None
            } else {
                Some(stderr.to_string())
            },
        })
    }

    /// 运行 golint 代码规范检查
    pub async fn run_golint(&self, file_path: &str) -> anyhow::Result<StaticAnalysisResult> {
        let start_time = std::time::Instant::now();

        let output = Command::new("golint").arg(file_path).output().await?;

        let execution_time = start_time.elapsed();
        let success = output.status.success();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut issues = Vec::new();

        // 解析 golint 输出
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // golint 输出格式: filename:line:column: message
            if let Some(issue) = parse_golint_line(line, file_path) {
                issues.push(issue);
            }
        }

        Ok(StaticAnalysisResult {
            tool: StaticAnalysisTool::GoLint,
            issues,
            execution_time,
            success,
            error_message: if success {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        })
    }

    /// 对指定文件运行所有适用的静态分析工具
    pub async fn analyze_file(
        &self,
        file_path: &str,
        language: &Language,
    ) -> Vec<StaticAnalysisResult> {
        let mut results = Vec::new();

        if let Some(tools) = self.enabled_tools.get(language) {
            for tool in tools {
                // 首先检查工具是否可用
                if !self.check_tool_availability(tool).await {
                    results.push(StaticAnalysisResult {
                        tool: tool.clone(),
                        issues: vec![],
                        execution_time: std::time::Duration::from_millis(0),
                        success: false,
                        error_message: Some(format!("{:?} 工具未安装或不可用", tool)),
                    });
                    continue;
                }

                let result = match tool {
                    StaticAnalysisTool::GoFmt => self.run_gofmt(file_path).await,
                    StaticAnalysisTool::GoVet => self.run_go_vet(file_path).await,
                    StaticAnalysisTool::GoLint => self.run_golint(file_path).await,
                };

                match result {
                    Ok(analysis_result) => results.push(analysis_result),
                    Err(e) => {
                        results.push(StaticAnalysisResult {
                            tool: tool.clone(),
                            issues: vec![],
                            execution_time: std::time::Duration::from_millis(0),
                            success: false,
                            error_message: Some(format!("执行 {:?} 时出错: {}", tool, e)),
                        });
                    }
                }
            }
        }

        results
    }

    /// 获取所有问题的统计信息
    pub fn get_issue_statistics(
        &self,
        results: &[StaticAnalysisResult],
    ) -> HashMap<IssueSeverity, usize> {
        let mut stats = HashMap::new();

        for result in results {
            for issue in &result.issues {
                *stats.entry(issue.severity.clone()).or_insert(0) += 1;
            }
        }

        stats
    }
}

/// 解析 go vet 输出行
fn parse_go_vet_line(line: &str, file_path: &str) -> Option<StaticAnalysisIssue> {
    // go vet 输出格式: filename:line:column: message
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    if parts.len() >= 3 {
        let line_num = parts[1].parse::<usize>().ok();
        let column = if parts.len() >= 4 {
            parts[2].parse::<usize>().ok()
        } else {
            None
        };
        let message = if parts.len() >= 4 {
            parts[3].trim().to_string()
        } else {
            parts[2].trim().to_string()
        };

        Some(StaticAnalysisIssue {
            tool: StaticAnalysisTool::GoVet,
            file_path: file_path.to_string(),
            line_number: line_num,
            column,
            severity: IssueSeverity::Warning,
            message,
            rule: Some("go-vet".to_string()),
            suggestion: None,
        })
    } else {
        None
    }
}

/// 解析 golint 输出行
fn parse_golint_line(line: &str, file_path: &str) -> Option<StaticAnalysisIssue> {
    // golint 输出格式: filename:line:column: message
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    if parts.len() >= 3 {
        let line_num = parts[1].parse::<usize>().ok();
        let column = if parts.len() >= 4 {
            parts[2].parse::<usize>().ok()
        } else {
            None
        };
        let message = if parts.len() >= 4 {
            parts[3].trim().to_string()
        } else {
            parts[2].trim().to_string()
        };

        Some(StaticAnalysisIssue {
            tool: StaticAnalysisTool::GoLint,
            file_path: file_path.to_string(),
            line_number: line_num,
            column,
            severity: IssueSeverity::Style,
            message,
            rule: Some("golint".to_string()),
            suggestion: None,
        })
    } else {
        None
    }
}

impl StaticAnalysisTool {
    pub fn name(&self) -> &str {
        match self {
            StaticAnalysisTool::GoFmt => "gofmt",
            StaticAnalysisTool::GoVet => "go vet",
            StaticAnalysisTool::GoLint => "golint",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            StaticAnalysisTool::GoFmt => "Go 官方代码格式化工具，检查代码格式是否符合标准",
            StaticAnalysisTool::GoVet => "Go 官方静态分析工具，检查常见的编程错误",
            StaticAnalysisTool::GoLint => "Go 代码风格检查工具，检查代码是否符合 Go 编码规范",
        }
    }
}

impl IssueSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            IssueSeverity::Error => "错误",
            IssueSeverity::Warning => "警告",
            IssueSeverity::Info => "信息",
            IssueSeverity::Style => "格式",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_analysis_service_creation() {
        let service = StaticAnalysisService::new();

        // 验证默认配置
        assert!(service.enabled_tools.contains_key(&Language::Go));
        let go_tools = service.enabled_tools.get(&Language::Go).unwrap();
        assert_eq!(go_tools.len(), 3);
        assert!(go_tools.contains(&StaticAnalysisTool::GoFmt));
        assert!(go_tools.contains(&StaticAnalysisTool::GoVet));
        assert!(go_tools.contains(&StaticAnalysisTool::GoLint));
    }

    #[test]
    fn test_parse_go_vet_line() {
        let line = "main.go:10:2: suspicious condition in if statement";
        let issue = parse_go_vet_line(line, "main.go").unwrap();

        assert_eq!(issue.tool, StaticAnalysisTool::GoVet);
        assert_eq!(issue.file_path, "main.go");
        assert_eq!(issue.line_number, Some(10));
        assert_eq!(issue.column, Some(2));
        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert!(issue.message.contains("suspicious condition"));
    }

    #[test]
    fn test_parse_golint_line() {
        let line = "main.go:5:1: exported function Foo should have comment";
        let issue = parse_golint_line(line, "main.go").unwrap();

        assert_eq!(issue.tool, StaticAnalysisTool::GoLint);
        assert_eq!(issue.file_path, "main.go");
        assert_eq!(issue.line_number, Some(5));
        assert_eq!(issue.column, Some(1));
        assert_eq!(issue.severity, IssueSeverity::Style);
        assert!(issue.message.contains("should have comment"));
    }

    #[test]
    fn test_tool_names_and_descriptions() {
        assert_eq!(StaticAnalysisTool::GoFmt.name(), "gofmt");
        assert_eq!(StaticAnalysisTool::GoVet.name(), "go vet");
        assert_eq!(StaticAnalysisTool::GoLint.name(), "golint");

        assert!(!StaticAnalysisTool::GoFmt.description().is_empty());
        assert!(!StaticAnalysisTool::GoVet.description().is_empty());
        assert!(!StaticAnalysisTool::GoLint.description().is_empty());
    }

    #[test]
    fn test_issue_severity_display() {
        assert_eq!(IssueSeverity::Error.as_str(), "错误");
        assert_eq!(IssueSeverity::Warning.as_str(), "警告");
        assert_eq!(IssueSeverity::Info.as_str(), "信息");
        assert_eq!(IssueSeverity::Style.as_str(), "格式");
    }

    #[test]
    fn test_issue_statistics() {
        let service = StaticAnalysisService::new();
        let results = vec![StaticAnalysisResult {
            tool: StaticAnalysisTool::GoFmt,
            issues: vec![StaticAnalysisIssue {
                tool: StaticAnalysisTool::GoFmt,
                file_path: "test.go".to_string(),
                line_number: None,
                column: None,
                severity: IssueSeverity::Style,
                message: "format issue".to_string(),
                rule: Some("gofmt".to_string()),
                suggestion: None,
            }],
            execution_time: std::time::Duration::from_millis(100),
            success: true,
            error_message: None,
        }];

        let stats = service.get_issue_statistics(&results);
        assert_eq!(stats.get(&IssueSeverity::Style), Some(&1));
        assert_eq!(stats.get(&IssueSeverity::Error), None);
    }
}
