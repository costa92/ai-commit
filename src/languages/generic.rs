use super::{Language, LanguageAnalyzer, LanguageFeature};

/// 通用语言分析器，用于不支持的语言
pub struct GenericAnalyzer;

impl GenericAnalyzer {
    pub fn new() -> Self {
        GenericAnalyzer
    }

    /// 基本的通用代码特征检测
    fn detect_generic_features(&self, line: &str) -> Vec<String> {
        let mut features = Vec::new();
        let lower_line = line.to_lowercase();

        // 检测常见的代码模式
        if lower_line.contains("function") || lower_line.contains("def ") || lower_line.contains("fn ") {
            features.push("function".to_string());
        }
        if lower_line.contains("class") {
            features.push("class".to_string());
        }
        if lower_line.contains("import") || lower_line.contains("include") || lower_line.contains("require") {
            features.push("import".to_string());
        }
        if lower_line.contains("export") {
            features.push("export".to_string());
        }
        if lower_line.contains("const") || lower_line.contains("let") || lower_line.contains("var") {
            features.push("variable".to_string());
        }

        features
    }

    /// 从文件路径推断作用域
    fn analyze_generic_structure(&self, file_path: &str) -> Vec<String> {
        let path_parts: Vec<&str> = file_path.split('/').collect();
        let mut suggestions = Vec::new();

        // 通用目录模式
        for part in &path_parts {
            match *part {
                "test" | "tests" | "__tests__" | "spec" => suggestions.push("test".to_string()),
                "lib" | "libs" | "library" => suggestions.push("library".to_string()),
                "src" | "source" => suggestions.push("source".to_string()),
                "config" | "configuration" => suggestions.push("config".to_string()),
                "util" | "utils" | "utilities" => suggestions.push("utils".to_string()),
                "helper" | "helpers" => suggestions.push("helper".to_string()),
                "service" | "services" => suggestions.push("service".to_string()),
                "api" => suggestions.push("api".to_string()),
                "docs" | "doc" | "documentation" => suggestions.push("docs".to_string()),
                "example" | "examples" => suggestions.push("example".to_string()),
                _ => {}
            }
        }

        // 从文件名推断
        if let Some(filename) = path_parts.last() {
            let name_without_ext = filename
                .split('.')
                .next()
                .unwrap_or(filename)
                .to_lowercase();
            
            if !suggestions.contains(&name_without_ext) && !name_without_ext.is_empty() {
                suggestions.push(name_without_ext);
            }
        }

        // 去重并返回
        suggestions.sort();
        suggestions.dedup();
        suggestions
    }
}

impl LanguageAnalyzer for GenericAnalyzer {
    fn language(&self) -> Language {
        Language::Unknown
    }

    fn analyze_line(&self, line: &str, line_number: usize) -> Vec<LanguageFeature> {
        let mut features = Vec::new();
        let trimmed_line = line.trim();

        // 跳过空行和明显的注释行
        if trimmed_line.is_empty() || 
           trimmed_line.starts_with("//") || 
           trimmed_line.starts_with("#") || 
           trimmed_line.starts_with("/*") ||
           trimmed_line.starts_with("*") {
            return features;
        }

        // 检测通用特征
        let detected_features = self.detect_generic_features(trimmed_line);
        
        for feature_type in detected_features {
            features.push(LanguageFeature {
                feature_type: feature_type.clone(),
                name: format!("generic_{}", feature_type),
                line_number: Some(line_number),
                description: format!("Generic {} detected in unknown language", feature_type),
            });
        }

        // 如果没有检测到特定特征，标记为通用代码变更
        if features.is_empty() && !trimmed_line.is_empty() {
            features.push(LanguageFeature {
                feature_type: "code_change".to_string(),
                name: "generic_change".to_string(),
                line_number: Some(line_number),
                description: "Generic code change in unknown language".to_string(),
            });
        }

        features
    }

    fn extract_scope_suggestions(&self, file_path: &str) -> Vec<String> {
        self.analyze_generic_structure(file_path)
    }

    fn analyze_change_patterns(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut patterns = Vec::new();

        let has_functions = features.iter().any(|f| f.feature_type == "function");
        let has_classes = features.iter().any(|f| f.feature_type == "class");
        let has_imports = features.iter().any(|f| f.feature_type == "import");
        let has_exports = features.iter().any(|f| f.feature_type == "export");
        let has_variables = features.iter().any(|f| f.feature_type == "variable");

        if has_functions {
            patterns.push("函数或方法定义变更".to_string());
        }

        if has_classes {
            patterns.push("类或对象定义变更".to_string());
        }

        if has_imports {
            patterns.push("依赖导入变更".to_string());
        }

        if has_exports {
            patterns.push("模块导出变更".to_string());
        }

        if has_variables {
            patterns.push("变量声明变更".to_string());
        }

        if patterns.is_empty() {
            patterns.push("通用代码调整".to_string());
        }

        patterns
    }

    fn generate_test_suggestions(&self, _features: &[LanguageFeature]) -> Vec<String> {
        vec![
            "根据具体语言添加相应的测试文件".to_string(),
            "确保变更不会破坏现有功能".to_string(),
            "检查代码格式和风格一致性".to_string(),
            "验证变更是否符合项目规范".to_string(),
            "考虑添加适当的文档说明".to_string(),
        ]
    }

    fn assess_risks(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut risks = Vec::new();

        // 通用风险评估
        if features.iter().any(|f| f.feature_type == "export") {
            risks.push("导出内容变更可能影响其他模块的使用".to_string());
        }

        if features.iter().any(|f| f.feature_type == "import") {
            risks.push("依赖关系变更需要检查版本兼容性".to_string());
        }

        if features.iter().any(|f| f.feature_type == "function" || f.feature_type == "class") {
            risks.push("核心逻辑变更需要充分测试".to_string());
        }

        // 通用风险提醒
        risks.push("由于语言未知，建议进行额外的代码审查".to_string());
        risks.push("确保变更符合项目的编码标准和最佳实践".to_string());

        risks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_analyzer_basic() {
        let analyzer = GenericAnalyzer::new();
        assert_eq!(analyzer.language(), Language::Unknown);
    }

    #[test]
    fn test_function_detection() {
        let analyzer = GenericAnalyzer::new();
        let line = "function processData() {";
        let features = analyzer.analyze_line(line, 10);
        
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "function");
        assert_eq!(features[0].name, "generic_function");
    }

    #[test]
    fn test_class_detection() {
        let analyzer = GenericAnalyzer::new();
        let line = "class UserService {";
        let features = analyzer.analyze_line(line, 5);
        
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "class");
        assert_eq!(features[0].name, "generic_class");
    }

    #[test]
    fn test_import_detection() {
        let analyzer = GenericAnalyzer::new();
        let line = "import something from 'module';";
        let features = analyzer.analyze_line(line, 1);
        
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "import");
        assert_eq!(features[0].name, "generic_import");
    }

    #[test]
    fn test_generic_code_change() {
        let analyzer = GenericAnalyzer::new();
        let line = "some random code line";
        let features = analyzer.analyze_line(line, 15);
        
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "code_change");
        assert_eq!(features[0].name, "generic_change");
    }

    #[test]
    fn test_scope_suggestions() {
        let analyzer = GenericAnalyzer::new();
        
        // 测试目录
        let suggestions = analyzer.extract_scope_suggestions("tests/unit/helper.py");
        assert!(suggestions.contains(&"test".to_string()));
        
        // 服务目录
        let suggestions = analyzer.extract_scope_suggestions("src/services/api.php");
        assert!(suggestions.contains(&"service".to_string()));
        
        // 工具目录
        let suggestions = analyzer.extract_scope_suggestions("utils/helper.rb");
        assert!(suggestions.contains(&"utils".to_string()));
    }

    #[test]
    fn test_change_patterns() {
        let analyzer = GenericAnalyzer::new();
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "generic_function".to_string(),
                line_number: Some(1),
                description: "test".to_string(),
            },
            LanguageFeature {
                feature_type: "import".to_string(),
                name: "generic_import".to_string(),
                line_number: Some(2),
                description: "test".to_string(),
            },
        ];
        
        let patterns = analyzer.analyze_change_patterns(&features);
        assert!(patterns.iter().any(|p| p.contains("函数或方法定义变更")));
        assert!(patterns.iter().any(|p| p.contains("依赖导入变更")));
    }

    #[test]
    fn test_test_suggestions() {
        let analyzer = GenericAnalyzer::new();
        let features = vec![];
        
        let suggestions = analyzer.generate_test_suggestions(&features);
        assert!(suggestions.iter().any(|s| s.contains("测试文件")));
        assert!(suggestions.iter().any(|s| s.contains("不会破坏现有功能")));
    }

    #[test]
    fn test_risk_assessment() {
        let analyzer = GenericAnalyzer::new();
        let features = vec![
            LanguageFeature {
                feature_type: "export".to_string(),
                name: "generic_export".to_string(),
                line_number: Some(1),
                description: "test".to_string(),
            },
        ];
        
        let risks = analyzer.assess_risks(&features);
        assert!(risks.iter().any(|r| r.contains("导出内容变更")));
        assert!(risks.iter().any(|r| r.contains("语言未知")));
    }

    #[test]
    fn test_empty_line_handling() {
        let analyzer = GenericAnalyzer::new();
        let features = analyzer.analyze_line("", 1);
        assert_eq!(features.len(), 0);
        
        let features = analyzer.analyze_line("   ", 1);
        assert_eq!(features.len(), 0);
    }

    #[test]
    fn test_comment_line_handling() {
        let analyzer = GenericAnalyzer::new();
        
        // 不同风格的注释
        let features = analyzer.analyze_line("// This is a comment", 1);
        assert_eq!(features.len(), 0);
        
        let features = analyzer.analyze_line("# This is a comment", 1);
        assert_eq!(features.len(), 0);
        
        let features = analyzer.analyze_line("/* This is a comment */", 1);
        assert_eq!(features.len(), 0);
        
        let features = analyzer.analyze_line("* This is a comment", 1);
        assert_eq!(features.len(), 0);
    }
}