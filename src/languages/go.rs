use super::{Language, LanguageAnalyzer, LanguageFeature};
use once_cell::sync::Lazy;
use regex::Regex;

// Go 语言特定的正则表达式
static GO_PACKAGE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*package\s+(\w+)").unwrap());
static GO_FUNC_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*func\s+(\w*\s*)?\(").unwrap());
static GO_STRUCT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*type\s+(\w+)\s+struct").unwrap());
static GO_INTERFACE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*type\s+(\w+)\s+interface").unwrap());
static GO_CONST_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*const\s+(\w+)").unwrap());
static GO_VAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*var\s+(\w+)").unwrap());
static GO_IMPORT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^\s*import\s+(?:"([^"]+)"|(\w+)\s+"([^"]+)")"#).unwrap());
static GO_METHOD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*func\s+\([^)]+\)\s+(\w+)").unwrap());

pub struct GoAnalyzer;

impl Default for GoAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GoAnalyzer {
    pub fn new() -> Self {
        GoAnalyzer
    }

    /// 提取函数名
    fn extract_function_name(&self, line: &str) -> Option<String> {
        if let Some(_caps) = GO_FUNC_REGEX.captures(line) {
            // 提取完整的函数声明
            let func_part = line.split('(').next().unwrap_or(line);
            Some(func_part.trim().to_string())
        } else {
            None
        }
    }

    /// 提取方法名
    fn extract_method_name(&self, line: &str) -> Option<String> {
        if let Some(caps) = GO_METHOD_REGEX.captures(line) {
            caps.get(1).map(|m| format!("method {}", m.as_str()))
        } else {
            None
        }
    }

    /// 分析 Go 项目结构
    fn analyze_project_structure(&self, file_path: &str) -> Vec<String> {
        let path_parts: Vec<&str> = file_path.split('/').collect();
        let mut suggestions = Vec::new();

        match path_parts.first() {
            Some(&"cmd") => {
                suggestions.push("cli".to_string());
                if let Some(app_name) = path_parts.get(1) {
                    suggestions.push(app_name.to_string());
                }
            }
            Some(&"pkg") => {
                suggestions.push("library".to_string());
                if let Some(pkg_name) = path_parts.get(1) {
                    suggestions.push(pkg_name.to_string());
                }
            }
            Some(&"internal") => {
                suggestions.push("internal".to_string());
                if let Some(module_name) = path_parts.get(1) {
                    suggestions.push(module_name.to_string());
                }
            }
            Some(&"api") => suggestions.push("api".to_string()),
            Some(&"web") => suggestions.push("web".to_string()),
            Some(&"test") | Some(&"tests") => suggestions.push("test".to_string()),
            Some(&"docs") => suggestions.push("docs".to_string()),
            _ => {
                // 从文件名推断
                if let Some(filename) = path_parts.last() {
                    let name_without_ext = filename.split('.').next().unwrap_or(filename);
                    suggestions.push(name_without_ext.to_string());
                }
            }
        }

        suggestions
    }
}

impl LanguageAnalyzer for GoAnalyzer {
    fn language(&self) -> Language {
        Language::Go
    }

    fn analyze_line(&self, line: &str, line_number: usize) -> Vec<LanguageFeature> {
        let mut features = Vec::new();
        let trimmed_line = line.trim();

        // 跳过注释行
        if trimmed_line.starts_with("//") || trimmed_line.starts_with("/*") {
            return features;
        }

        // Package 声明
        if let Some(caps) = GO_PACKAGE_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "package".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Go package declaration defining module scope and namespace"
                    .to_string(),
            });
        }

        // Import 声明
        if let Some(caps) = GO_IMPORT_REGEX.captures(trimmed_line) {
            let import_path = caps
                .get(1)
                .or(caps.get(3))
                .map(|m| m.as_str())
                .unwrap_or("unknown");
            features.push(LanguageFeature {
                feature_type: "import".to_string(),
                name: import_path.to_string(),
                line_number: Some(line_number),
                description: "Go package import for external dependencies".to_string(),
            });
        }

        // Method 定义 (必须在 function 之前检查)
        if let Some(method_name) = self.extract_method_name(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "method".to_string(),
                name: method_name,
                line_number: Some(line_number),
                description: "Go method definition with receiver type".to_string(),
            });
        }
        // Function 定义
        else if let Some(func_name) = self.extract_function_name(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "function".to_string(),
                name: func_name,
                line_number: Some(line_number),
                description: "Go function definition with type signature".to_string(),
            });
        }

        // Struct 定义
        if let Some(caps) = GO_STRUCT_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "struct".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Go struct type definition for data modeling and encapsulation"
                    .to_string(),
            });
        }

        // Interface 定义
        if let Some(caps) = GO_INTERFACE_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "interface".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Go interface definition for behavior contracts and polymorphism"
                    .to_string(),
            });
        }

        // Const 定义
        if let Some(caps) = GO_CONST_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "const".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Go constant declaration for immutable values".to_string(),
            });
        }

        // Var 定义
        if let Some(caps) = GO_VAR_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "variable".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Go variable declaration with optional initialization".to_string(),
            });
        }

        features
    }

    fn extract_scope_suggestions(&self, file_path: &str) -> Vec<String> {
        self.analyze_project_structure(file_path)
    }

    fn analyze_change_patterns(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut patterns = Vec::new();

        let has_package = features.iter().any(|f| f.feature_type == "package");
        let has_functions = features
            .iter()
            .any(|f| f.feature_type == "function" || f.feature_type == "method");
        let has_structs = features.iter().any(|f| f.feature_type == "struct");
        let has_interfaces = features.iter().any(|f| f.feature_type == "interface");
        let has_imports = features.iter().any(|f| f.feature_type == "import");

        if has_package {
            patterns.push("新建Go模块或包结构调整".to_string());
        }

        if has_interfaces {
            patterns.push("接口定义变更，可能影响实现类型".to_string());
        }

        if has_structs {
            patterns.push("数据结构定义变更，可能影响序列化和兼容性".to_string());
        }

        if has_functions {
            patterns.push("业务逻辑实现变更".to_string());
        }

        if has_imports {
            patterns.push("依赖关系变更，需要检查版本兼容性".to_string());
        }

        if patterns.is_empty() {
            patterns.push("代码细节调整".to_string());
        }

        patterns
    }

    fn generate_test_suggestions(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 基础测试建议
        suggestions.push("创建对应的 *_test.go 文件".to_string());
        suggestions.push("使用表驱动测试模式进行全面测试".to_string());

        // 基于特征的特定建议
        for feature in features {
            match feature.feature_type.as_str() {
                "function" | "method" => {
                    suggestions.push(format!(
                        "为 {} 添加单元测试，覆盖正常和异常情况",
                        feature.name
                    ));
                    suggestions.push("测试函数的输入验证和错误处理".to_string());
                }
                "struct" => {
                    suggestions.push(format!(
                        "测试 {} 结构体的创建、序列化和反序列化",
                        feature.name
                    ));
                    suggestions.push("验证结构体字段的约束和验证逻辑".to_string());
                }
                "interface" => {
                    suggestions.push(format!("为 {} 接口的所有实现创建测试用例", feature.name));
                    suggestions.push("测试接口的多态性和类型断言".to_string());
                }
                _ => {}
            }
        }

        // Go 特定的测试建议
        suggestions.push("运行 go test -race 检查并发安全性".to_string());
        suggestions.push("使用 go test -bench 进行性能基准测试".to_string());
        suggestions.push("确保测试覆盖率达到 80% 以上".to_string());

        // 去重
        suggestions.sort();
        suggestions.dedup();
        suggestions
    }

    fn assess_risks(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut risks = Vec::new();

        // 接口变更风险
        if features.iter().any(|f| f.feature_type == "interface") {
            risks.push("接口变更可能导致现有实现失效，需要检查所有实现类型".to_string());
        }

        // 公共API变更风险
        for feature in features {
            if feature
                .name
                .chars()
                .next()
                .is_some_and(|c| c.is_uppercase())
            {
                match feature.feature_type.as_str() {
                    "function" | "struct" | "interface" => {
                        risks.push(format!(
                            "公共 {} {} 的变更可能影响外部调用者",
                            feature.feature_type, feature.name
                        ));
                    }
                    _ => {}
                }
            }
        }

        // 包级别变更风险
        if features.iter().any(|f| f.feature_type == "package") {
            risks.push("包声明变更可能影响导入路径和模块依赖".to_string());
        }

        // 导入变更风险
        if features.iter().any(|f| f.feature_type == "import") {
            risks.push("新增依赖需要检查版本兼容性和安全性".to_string());
        }

        // 并发安全风险
        if features.iter().any(|f| {
            f.name.to_lowercase().contains("goroutine")
                || f.name.to_lowercase().contains("channel")
                || f.name.to_lowercase().contains("mutex")
        }) {
            risks.push("涉及并发的代码变更需要特别关注竞态条件和死锁问题".to_string());
        }

        risks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_analyzer_basic() {
        let analyzer = GoAnalyzer::new();
        assert_eq!(analyzer.language(), Language::Go);
    }

    #[test]
    fn test_default_implementation() {
        // 测试 Default trait 实现
        let analyzer = GoAnalyzer;
        assert_eq!(analyzer.language(), Language::Go);

        // 确保 Default 和 new() 创建的实例功能相同
        let new_analyzer = GoAnalyzer::new();
        assert_eq!(analyzer.language(), new_analyzer.language());

        // 测试默认实例能正常工作
        let line = "func test() {}";
        let features_default = analyzer.analyze_line(line, 1);
        let features_new = new_analyzer.analyze_line(line, 1);
        assert_eq!(features_default.len(), features_new.len());
    }

    #[test]
    fn test_package_detection() {
        let analyzer = GoAnalyzer::new();
        let line = "package main";
        let features = analyzer.analyze_line(line, 1);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "package");
        assert_eq!(features[0].name, "main");
    }

    #[test]
    fn test_function_detection() {
        let analyzer = GoAnalyzer::new();
        let line = "func NewHandler(db *sql.DB) *Handler {";
        let features = analyzer.analyze_line(line, 10);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "function");
        assert!(features[0].name.contains("NewHandler"));
    }

    #[test]
    fn test_struct_detection() {
        let analyzer = GoAnalyzer::new();
        let line = "type User struct {";
        let features = analyzer.analyze_line(line, 15);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "struct");
        assert_eq!(features[0].name, "User");
    }

    #[test]
    fn test_interface_detection() {
        let analyzer = GoAnalyzer::new();
        let line = "type Repository interface {";
        let features = analyzer.analyze_line(line, 20);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "interface");
        assert_eq!(features[0].name, "Repository");
    }

    #[test]
    fn test_method_detection() {
        let analyzer = GoAnalyzer::new();
        let line = "func (u *User) GetName() string {";
        let features = analyzer.analyze_line(line, 25);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "method");
        assert!(features[0].name.contains("GetName"));
    }

    #[test]
    fn test_scope_suggestions() {
        let analyzer = GoAnalyzer::new();

        // cmd 目录
        let suggestions = analyzer.extract_scope_suggestions("cmd/server/main.go");
        assert!(suggestions.contains(&"cli".to_string()));
        assert!(suggestions.contains(&"server".to_string()));

        // pkg 目录
        let suggestions = analyzer.extract_scope_suggestions("pkg/auth/handler.go");
        assert!(suggestions.contains(&"library".to_string()));
        assert!(suggestions.contains(&"auth".to_string()));

        // internal 目录
        let suggestions = analyzer.extract_scope_suggestions("internal/config/config.go");
        assert!(suggestions.contains(&"internal".to_string()));
        assert!(suggestions.contains(&"config".to_string()));
    }

    #[test]
    fn test_change_patterns() {
        let analyzer = GoAnalyzer::new();
        let features = vec![
            LanguageFeature {
                feature_type: "interface".to_string(),
                name: "UserService".to_string(),
                line_number: Some(1),
                description: "test".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "User".to_string(),
                line_number: Some(2),
                description: "test".to_string(),
            },
        ];

        let patterns = analyzer.analyze_change_patterns(&features);
        assert!(patterns.iter().any(|p| p.contains("接口定义变更")));
        assert!(patterns.iter().any(|p| p.contains("数据结构定义变更")));
    }

    #[test]
    fn test_test_suggestions() {
        let analyzer = GoAnalyzer::new();
        let features = vec![LanguageFeature {
            feature_type: "function".to_string(),
            name: "ProcessData".to_string(),
            line_number: Some(1),
            description: "test".to_string(),
        }];

        let suggestions = analyzer.generate_test_suggestions(&features);
        assert!(suggestions.iter().any(|s| s.contains("*_test.go")));
        assert!(suggestions.iter().any(|s| s.contains("表驱动测试")));
    }

    #[test]
    fn test_risk_assessment() {
        let analyzer = GoAnalyzer::new();
        let features = vec![LanguageFeature {
            feature_type: "interface".to_string(),
            name: "PublicInterface".to_string(),
            line_number: Some(1),
            description: "test".to_string(),
        }];

        let risks = analyzer.assess_risks(&features);
        assert!(risks.iter().any(|r| r.contains("接口变更")));
    }
}
