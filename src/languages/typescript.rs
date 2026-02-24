use super::{Language, LanguageAnalyzer, LanguageFeature};
use once_cell::sync::Lazy;
use regex::Regex;

// TypeScript 语言特定的正则表达式
static TS_INTERFACE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:export\s+)?interface\s+(\w+)").unwrap());
static TS_CLASS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:export\s+)?(?:abstract\s+)?class\s+(\w+)").unwrap());
static TS_FUNCTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)").unwrap());
static TS_ARROW_FUNCTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?:export\s+)?(?:const|let|var)\s+(\w+)\s*=\s*(?:\([^)]*\)\s*)?=>").unwrap()
});
static TS_METHOD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?:public|private|protected|static)?\s*(?:async\s+)?(\w+)\s*\(").unwrap()
});
static TS_TYPE_ALIAS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:export\s+)?type\s+(\w+)\s*=").unwrap());
static TS_ENUM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:export\s+)?enum\s+(\w+)").unwrap());
static TS_IMPORT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^\s*import\s+(?:\{[^}]+\}|\w+|\*\s+as\s+\w+)\s+from\s+['"]([^'"]+)['"]"#).unwrap()
});
static TS_EXPORT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*export\s+(?:\{[^}]+\}|(?:default\s+)?(?:class|function|interface|type|enum|const|let|var)\s+(\w+))").unwrap()
});

pub struct TypeScriptAnalyzer;

impl Default for TypeScriptAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptAnalyzer {
    pub fn new() -> Self {
        TypeScriptAnalyzer
    }

    /// 检测是否为 React 组件
    fn is_react_component(&self, line: &str) -> bool {
        line.contains("React.")
            || line.contains("JSX.")
            || line.contains("ReactNode")
            || line.contains("Component")
            || line.contains("useState")
            || line.contains("useEffect")
    }

    /// 提取函数名（包括箭头函数）
    fn extract_function_name(&self, line: &str) -> Option<String> {
        if let Some(caps) = TS_FUNCTION_REGEX.captures(line) {
            caps.get(1).map(|m| m.as_str().to_string())
        } else if let Some(caps) = TS_ARROW_FUNCTION_REGEX.captures(line) {
            caps.get(1)
                .map(|m| format!("{} (arrow function)", m.as_str()))
        } else {
            None
        }
    }

    /// 分析 TypeScript 项目结构
    fn analyze_project_structure(&self, file_path: &str) -> Vec<String> {
        let path_parts: Vec<&str> = file_path.split('/').collect();
        let mut suggestions = Vec::new();

        // 分析目录结构
        for (i, part) in path_parts.iter().enumerate() {
            match *part {
                "src" => {
                    if let Some(next_part) = path_parts.get(i + 1) {
                        suggestions.push(next_part.to_string());
                    }
                }
                "components" => suggestions.push("ui".to_string()),
                "pages" => suggestions.push("page".to_string()),
                "hooks" => suggestions.push("hook".to_string()),
                "utils" | "helpers" => suggestions.push("utils".to_string()),
                "services" => suggestions.push("service".to_string()),
                "api" => suggestions.push("api".to_string()),
                "types" => suggestions.push("types".to_string()),
                "store" | "state" => suggestions.push("state".to_string()),
                "test" | "tests" | "__tests__" => suggestions.push("test".to_string()),
                "stories" => suggestions.push("storybook".to_string()),
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

            match name_without_ext.as_str() {
                name if name.ends_with("component") => suggestions.push("component".to_string()),
                name if name.ends_with("hook") => suggestions.push("hook".to_string()),
                name if name.ends_with("service") => suggestions.push("service".to_string()),
                name if name.ends_with("util") || name.ends_with("utils") => {
                    suggestions.push("utils".to_string())
                }
                name if name.ends_with("type") || name.ends_with("types") => {
                    suggestions.push("types".to_string())
                }
                name if name.contains("test") || name.contains("spec") => {
                    suggestions.push("test".to_string())
                }
                _ => {
                    if !suggestions.contains(&name_without_ext) {
                        suggestions.push(name_without_ext);
                    }
                }
            }
        }

        // 去重并返回
        suggestions.sort();
        suggestions.dedup();
        suggestions
    }
}

impl LanguageAnalyzer for TypeScriptAnalyzer {
    fn language(&self) -> Language {
        Language::TypeScript
    }

    fn analyze_line(&self, line: &str, line_number: usize) -> Vec<LanguageFeature> {
        let mut features = Vec::new();
        let trimmed_line = line.trim();

        // 跳过注释行
        if trimmed_line.starts_with("//")
            || trimmed_line.starts_with("/*")
            || trimmed_line.starts_with("*")
        {
            return features;
        }

        // Import 声明
        if let Some(caps) = TS_IMPORT_REGEX.captures(trimmed_line) {
            let import_path = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown");
            features.push(LanguageFeature {
                feature_type: "import".to_string(),
                name: import_path.to_string(),
                line_number: Some(line_number),
                description: "TypeScript import statement for module dependencies".to_string(),
            });
        }

        // Interface 定义
        if let Some(caps) = TS_INTERFACE_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "interface".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "TypeScript interface definition for type contracts".to_string(),
            });
        }

        // Class 定义
        if let Some(caps) = TS_CLASS_REGEX.captures(trimmed_line) {
            let class_name = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown");
            let feature_type = if self.is_react_component(trimmed_line) {
                "react_component"
            } else {
                "class"
            };

            features.push(LanguageFeature {
                feature_type: feature_type.to_string(),
                name: class_name.to_string(),
                line_number: Some(line_number),
                description: "TypeScript class definition with methods and properties".to_string(),
            });
        }

        // Function 定义（包括箭头函数）
        if let Some(func_name) = self.extract_function_name(trimmed_line) {
            let feature_type = if self.is_react_component(trimmed_line) {
                "react_component"
            } else {
                "function"
            };

            features.push(LanguageFeature {
                feature_type: feature_type.to_string(),
                name: func_name,
                line_number: Some(line_number),
                description: "TypeScript function definition with type annotations".to_string(),
            });
        }

        // Method 定义
        if let Some(caps) = TS_METHOD_REGEX.captures(trimmed_line) {
            if !trimmed_line.contains("function") && !trimmed_line.contains("class") {
                features.push(LanguageFeature {
                    feature_type: "method".to_string(),
                    name: caps
                        .get(1)
                        .map(|m| m.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    line_number: Some(line_number),
                    description: "TypeScript class method with access modifiers".to_string(),
                });
            }
        }

        // Type Alias 定义
        if let Some(caps) = TS_TYPE_ALIAS_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "type_alias".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "TypeScript type alias for complex type definitions".to_string(),
            });
        }

        // Enum 定义
        if let Some(caps) = TS_ENUM_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "enum".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "TypeScript enum definition for named constants".to_string(),
            });
        }

        // Export 声明
        if let Some(caps) = TS_EXPORT_REGEX.captures(trimmed_line) {
            if let Some(export_name) = caps.get(1) {
                features.push(LanguageFeature {
                    feature_type: "export".to_string(),
                    name: export_name.as_str().to_string(),
                    line_number: Some(line_number),
                    description: "TypeScript export statement for module API".to_string(),
                });
            }
        }

        features
    }

    fn extract_scope_suggestions(&self, file_path: &str) -> Vec<String> {
        self.analyze_project_structure(file_path)
    }

    fn analyze_change_patterns(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut patterns = Vec::new();

        let has_interfaces = features.iter().any(|f| f.feature_type == "interface");
        let has_types = features.iter().any(|f| f.feature_type == "type_alias");
        let has_classes = features.iter().any(|f| f.feature_type == "class");
        let has_functions = features.iter().any(|f| f.feature_type == "function");
        let has_react_components = features.iter().any(|f| f.feature_type == "react_component");
        let has_exports = features.iter().any(|f| f.feature_type == "export");
        let has_imports = features.iter().any(|f| f.feature_type == "import");

        if has_react_components {
            patterns.push("React组件变更，可能影响UI渲染和用户交互".to_string());
        }

        if has_interfaces || has_types {
            patterns.push("类型定义变更，可能影响类型检查和API契约".to_string());
        }

        if has_classes {
            patterns.push("类定义变更，可能影响继承关系和实例化".to_string());
        }

        if has_functions {
            patterns.push("函数逻辑变更，需要验证参数类型和返回值".to_string());
        }

        if has_exports {
            patterns.push("模块导出变更，可能影响外部模块的导入和使用".to_string());
        }

        if has_imports {
            patterns.push("依赖关系变更，需要检查版本兼容性和类型声明".to_string());
        }

        if patterns.is_empty() {
            patterns.push("代码细节调整".to_string());
        }

        patterns
    }

    fn generate_test_suggestions(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 基础测试建议
        suggestions.push("创建对应的 .test.ts 或 .spec.ts 文件".to_string());
        suggestions.push("使用 Jest 或 Vitest 进行单元测试".to_string());

        // 基于特征的特定建议
        for feature in features {
            match feature.feature_type.as_str() {
                "react_component" => {
                    suggestions.push(format!(
                        "为 {} 组件添加 React Testing Library 测试",
                        feature.name
                    ));
                    suggestions.push("测试组件的渲染、交互和状态变化".to_string());
                    suggestions.push("添加快照测试确保UI一致性".to_string());
                }
                "function" => {
                    suggestions.push(format!(
                        "为 {} 函数添加单元测试，覆盖各种输入场景",
                        feature.name
                    ));
                    suggestions.push("测试函数的类型安全和边界条件".to_string());
                }
                "class" => {
                    suggestions.push(format!(
                        "测试 {} 类的实例化、方法调用和状态管理",
                        feature.name
                    ));
                    suggestions.push("验证类的继承关系和多态性".to_string());
                }
                "interface" | "type_alias" => {
                    suggestions.push(format!(
                        "为 {} 类型创建类型测试，确保类型安全",
                        feature.name
                    ));
                    suggestions.push("验证类型约束和类型推断的正确性".to_string());
                }
                _ => {}
            }
        }

        // TypeScript 特定的测试建议
        suggestions.push("运行 tsc --noEmit 检查类型错误".to_string());
        suggestions.push("使用 ESLint 和 Prettier 保持代码质量".to_string());
        suggestions.push("确保测试覆盖率达到 80% 以上".to_string());
        suggestions.push("添加集成测试验证模块间的交互".to_string());

        // 去重
        suggestions.sort();
        suggestions.dedup();
        suggestions
    }

    fn assess_risks(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut risks = Vec::new();

        // React 组件风险
        if features.iter().any(|f| f.feature_type == "react_component") {
            risks.push("React组件变更可能影响页面渲染和用户体验".to_string());
            risks.push("需要检查组件的props类型和依赖的context变化".to_string());
        }

        // 类型定义风险
        if features
            .iter()
            .any(|f| f.feature_type == "interface" || f.feature_type == "type_alias")
        {
            risks.push("类型定义变更可能导致现有代码的类型检查失败".to_string());
        }

        // 公共API风险
        for feature in features {
            if feature.feature_type == "export" {
                risks.push(format!(
                    "导出的 {} 变更可能影响依赖此模块的其他代码",
                    feature.name
                ));
            }
        }

        // 导入变更风险
        if features.iter().any(|f| f.feature_type == "import") {
            risks.push("新增或修改导入可能引入版本冲突或运行时错误".to_string());
        }

        // 异步代码风险
        let has_async = features.iter().any(|f| {
            f.name.to_lowercase().contains("async")
                || f.description.to_lowercase().contains("async")
        });
        if has_async {
            risks.push("异步代码变更需要特别关注错误处理和竞态条件".to_string());
        }

        // 状态管理风险
        let has_state = features.iter().any(|f| {
            f.name.to_lowercase().contains("state")
                || f.name.to_lowercase().contains("store")
                || f.name.to_lowercase().contains("reducer")
        });
        if has_state {
            risks.push("状态管理变更可能影响应用的数据流和一致性".to_string());
        }

        risks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_analyzer_basic() {
        let analyzer = TypeScriptAnalyzer::new();
        assert_eq!(analyzer.language(), Language::TypeScript);
    }

    #[test]
    fn test_default_implementation() {
        // 测试 Default trait 实现
        let analyzer = TypeScriptAnalyzer;
        assert_eq!(analyzer.language(), Language::TypeScript);

        // 确保 Default 和 new() 创建的实例功能相同
        let new_analyzer = TypeScriptAnalyzer::new();
        assert_eq!(analyzer.language(), new_analyzer.language());

        // 测试默认实例能正常工作
        let line = "function test(): void {}";
        let features_default = analyzer.analyze_line(line, 1);
        let features_new = new_analyzer.analyze_line(line, 1);
        assert_eq!(features_default.len(), features_new.len());
    }

    #[test]
    fn test_interface_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let line = "export interface User {";
        let features = analyzer.analyze_line(line, 1);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "interface");
        assert_eq!(features[0].name, "User");
    }

    #[test]
    fn test_class_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let line = "export class UserService {";
        let features = analyzer.analyze_line(line, 5);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "class");
        assert_eq!(features[0].name, "UserService");
    }

    #[test]
    fn test_function_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let line = "export function processData(input: string): Promise<Result> {";
        let features = analyzer.analyze_line(line, 10);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "function");
        assert_eq!(features[0].name, "processData");
    }

    #[test]
    fn test_arrow_function_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let line = "const handleClick = (event: MouseEvent) => {";
        let features = analyzer.analyze_line(line, 15);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "function");
        assert!(features[0].name.contains("handleClick"));
        assert!(features[0].name.contains("arrow function"));
    }

    #[test]
    fn test_react_component_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let line = "const UserProfile: React.FC<Props> = ({ user }) => {";
        let features = analyzer.analyze_line(line, 20);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "react_component");
        assert!(features[0].name.contains("UserProfile"));
    }

    #[test]
    fn test_type_alias_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let line = "export type ApiResponse<T> = {";
        let features = analyzer.analyze_line(line, 25);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "type_alias");
        assert_eq!(features[0].name, "ApiResponse");
    }

    #[test]
    fn test_enum_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let line = "enum UserRole {";
        let features = analyzer.analyze_line(line, 30);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "enum");
        assert_eq!(features[0].name, "UserRole");
    }

    #[test]
    fn test_import_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let line = "import { useState, useEffect } from 'react';";
        let features = analyzer.analyze_line(line, 1);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "import");
        assert_eq!(features[0].name, "react");
    }

    #[test]
    fn test_scope_suggestions() {
        let analyzer = TypeScriptAnalyzer::new();

        // React 组件
        let suggestions = analyzer.extract_scope_suggestions("src/components/UserProfile.tsx");
        assert!(
            suggestions.contains(&"ui".to_string())
                || suggestions.contains(&"components".to_string())
        );

        // 服务层
        let suggestions = analyzer.extract_scope_suggestions("src/services/api.ts");
        assert!(suggestions.contains(&"service".to_string()));

        // Hooks
        let suggestions = analyzer.extract_scope_suggestions("src/hooks/useAuth.ts");
        assert!(suggestions.contains(&"hook".to_string()));
    }

    #[test]
    fn test_change_patterns() {
        let analyzer = TypeScriptAnalyzer::new();
        let features = vec![
            LanguageFeature {
                feature_type: "react_component".to_string(),
                name: "UserProfile".to_string(),
                line_number: Some(1),
                description: "test".to_string(),
            },
            LanguageFeature {
                feature_type: "interface".to_string(),
                name: "User".to_string(),
                line_number: Some(2),
                description: "test".to_string(),
            },
        ];

        let patterns = analyzer.analyze_change_patterns(&features);
        assert!(patterns.iter().any(|p| p.contains("React组件变更")));
        assert!(patterns.iter().any(|p| p.contains("类型定义变更")));
    }

    #[test]
    fn test_test_suggestions() {
        let analyzer = TypeScriptAnalyzer::new();
        let features = vec![LanguageFeature {
            feature_type: "react_component".to_string(),
            name: "Button".to_string(),
            line_number: Some(1),
            description: "test".to_string(),
        }];

        let suggestions = analyzer.generate_test_suggestions(&features);
        assert!(suggestions
            .iter()
            .any(|s| s.contains("React Testing Library")));
        assert!(suggestions
            .iter()
            .any(|s| s.contains(".test.ts") || s.contains(".spec.ts")));
    }

    #[test]
    fn test_risk_assessment() {
        let analyzer = TypeScriptAnalyzer::new();
        let features = vec![LanguageFeature {
            feature_type: "export".to_string(),
            name: "publicApi".to_string(),
            line_number: Some(1),
            description: "test".to_string(),
        }];

        let risks = analyzer.assess_risks(&features);
        assert!(risks
            .iter()
            .any(|r| r.contains("导出的") && r.contains("可能影响")));
    }
}
