use crate::languages::{LanguageAnalyzer, LanguageFeature};
use regex::Regex;
use std::collections::HashSet;

/// TypeScript 语言分析器
pub struct TypeScriptAnalyzer {
    patterns: TypeScriptPatterns,
}

impl TypeScriptAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: TypeScriptPatterns::new(),
        }
    }

    /// 提取模块声明
    fn extract_module_declarations(&self, content: &str) -> Vec<LanguageFeature> {
        let mut modules = Vec::new();

        for captures in self.patterns.module_regex.captures_iter(content) {
            if let Some(module_name) = captures.get(1) {
                modules.push(LanguageFeature::Module(module_name.as_str().to_string()));
            }
        }

        modules
    }

    /// 提取函数定义
    fn extract_functions(&self, content: &str) -> Vec<LanguageFeature> {
        let mut functions = Vec::new();

        // 普通函数
        for captures in self.patterns.function_regex.captures_iter(content) {
            if let Some(func_name) = captures.get(1) {
                functions.push(LanguageFeature::Function(func_name.as_str().to_string()));
            }
        }

        // 箭头函数
        for captures in self.patterns.arrow_function_regex.captures_iter(content) {
            if let Some(func_name) = captures.get(1) {
                functions.push(LanguageFeature::Function(func_name.as_str().to_string()));
            }
        }

        // 方法定义
        for captures in self.patterns.method_regex.captures_iter(content) {
            if let Some(method_name) = captures.get(1) {
                functions.push(LanguageFeature::Function(method_name.as_str().to_string()));
            }
        }

        functions
    }

    /// 提取类定义
    fn extract_classes(&self, content: &str) -> Vec<LanguageFeature> {
        let mut classes = Vec::new();

        for captures in self.patterns.class_regex.captures_iter(content) {
            if let Some(class_name) = captures.get(1) {
                classes.push(LanguageFeature::Class(class_name.as_str().to_string()));
            }
        }

        classes
    }

    /// 提取接口定义
    fn extract_interfaces(&self, content: &str) -> Vec<LanguageFeature> {
        let mut interfaces = Vec::new();

        for captures in self.patterns.interface_regex.captures_iter(content) {
            if let Some(interface_name) = captures.get(1) {
                interfaces.push(LanguageFeature::Interface(interface_name.as_str().to_string()));
            }
        }

        interfaces
    }

    /// 检测 TypeScript 特定的代码模式
    pub fn detect_typescript_patterns(&self, content: &str) -> TypeScriptCodePatterns {
        let mut patterns = TypeScriptCodePatterns::default();

        // 检测类型系统使用
        patterns.has_type_annotations = self.patterns.type_annotation_regex.is_match(content);
        patterns.has_generic_types = self.patterns.generic_regex.is_match(content);
        patterns.has_union_types = self.patterns.union_type_regex.is_match(content);
        patterns.has_intersection_types = self.patterns.intersection_type_regex.is_match(content);

        // 检测高级类型特性
        patterns.has_mapped_types = self.patterns.mapped_type_regex.is_match(content);
        patterns.has_conditional_types = self.patterns.conditional_type_regex.is_match(content);
        patterns.has_utility_types = self.patterns.utility_type_regex.is_match(content);

        // 检测异步编程
        patterns.has_async_await = self.patterns.async_await_regex.is_match(content);
        patterns.has_promises = self.patterns.promise_regex.is_match(content);

        // 检测装饰器
        patterns.has_decorators = self.patterns.decorator_regex.is_match(content);

        // 检测模块系统
        patterns.has_es6_imports = self.patterns.es6_import_regex.is_match(content);
        patterns.has_es6_exports = self.patterns.es6_export_regex.is_match(content);
        patterns.has_namespace = self.patterns.namespace_regex.is_match(content);

        // 检测类特性
        patterns.has_access_modifiers = self.patterns.access_modifier_regex.is_match(content);
        patterns.has_abstract_classes = self.patterns.abstract_class_regex.is_match(content);

        // 检测枚举
        patterns.has_enums = self.patterns.enum_regex.is_match(content);

        // 检测类型断言
        patterns.has_type_assertions = self.patterns.type_assertion_regex.is_match(content);

        // 检测可选链和空值合并
        patterns.has_optional_chaining = self.patterns.optional_chaining_regex.is_match(content);
        patterns.has_nullish_coalescing = self.patterns.nullish_coalescing_regex.is_match(content);

        patterns
    }

    /// 分析导入和导出
    pub fn analyze_imports_exports(&self, content: &str) -> ImportExportAnalysis {
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        // ES6 导入
        for captures in self.patterns.es6_import_regex.captures_iter(content) {
            if let Some(import_path) = captures.get(1) {
                imports.push(ImportInfo {
                    module_path: import_path.as_str().trim_matches('"').trim_matches('\'').to_string(),
                    import_type: ImportType::ES6,
                    imported_items: self.extract_imported_items(&captures[0]),
                });
            }
        }

        // CommonJS require
        for captures in self.patterns.require_regex.captures_iter(content) {
            if let Some(require_path) = captures.get(1) {
                imports.push(ImportInfo {
                    module_path: require_path.as_str().trim_matches('"').trim_matches('\'').to_string(),
                    import_type: ImportType::CommonJS,
                    imported_items: vec![],
                });
            }
        }

        // ES6 导出
        for captures in self.patterns.es6_export_regex.captures_iter(content) {
            exports.push(ExportInfo {
                export_type: ExportType::ES6,
                exported_item: captures.get(1).map(|m| m.as_str().to_string()),
            });
        }

        // CommonJS 导出
        for captures in self.patterns.commonjs_export_regex.captures_iter(content) {
            exports.push(ExportInfo {
                export_type: ExportType::CommonJS,
                exported_item: captures.get(1).map(|m| m.as_str().to_string()),
            });
        }

        ImportExportAnalysis {
            imports,
            exports,
        }
    }

    fn extract_imported_items(&self, import_statement: &str) -> Vec<String> {
        let mut items = Vec::new();

        // 提取 { item1, item2 } 形式的导入
        if let Some(captures) = self.patterns.named_import_regex.captures(import_statement) {
            if let Some(named_imports) = captures.get(1) {
                for item in named_imports.as_str().split(',') {
                    let item = item.trim();
                    if !item.is_empty() {
                        items.push(item.to_string());
                    }
                }
            }
        }

        // 提取默认导入
        if let Some(captures) = self.patterns.default_import_regex.captures(import_statement) {
            if let Some(default_import) = captures.get(1) {
                items.push(default_import.as_str().to_string());
            }
        }

        items
    }

    /// 分析函数复杂度
    pub fn analyze_function_complexity(&self, content: &str) -> Vec<FunctionComplexity> {
        let mut complexities = Vec::new();

        // 分析普通函数
        for captures in self.patterns.function_regex.captures_iter(content) {
            if let Some(func_name) = captures.get(1) {
                let func_name = func_name.as_str().to_string();
                let func_start = captures.get(0).unwrap().end();

                if let Some(function_body) = self.extract_function_body(content, func_start) {
                    let complexity = self.calculate_simple_complexity(&function_body);
                    let line_count = function_body.lines().count();

                    complexities.push(FunctionComplexity {
                        name: func_name,
                        cyclomatic_complexity: complexity,
                        line_count,
                        is_async: function_body.contains("async"),
                    });
                }
            }
        }

        // 分析箭头函数
        for captures in self.patterns.arrow_function_regex.captures_iter(content) {
            if let Some(func_name) = captures.get(1) {
                let func_name = func_name.as_str().to_string();
                let func_start = captures.get(0).unwrap().end();

                if let Some(function_body) = self.extract_arrow_function_body(content, func_start) {
                    let complexity = self.calculate_simple_complexity(&function_body);
                    let line_count = function_body.lines().count();

                    complexities.push(FunctionComplexity {
                        name: func_name,
                        cyclomatic_complexity: complexity,
                        line_count,
                        is_async: function_body.contains("async"),
                    });
                }
            }
        }

        complexities
    }

    /// 提取函数体内容
    fn extract_function_body(&self, content: &str, start_pos: usize) -> Option<String> {
        let remaining_content = &content[start_pos..];

        // 找到第一个 {
        if let Some(brace_start) = remaining_content.find('{') {
            let mut brace_count = 0;
            let mut body_end = brace_start;
            let chars: Vec<char> = remaining_content.chars().collect();

            for (i, &ch) in chars.iter().enumerate().skip(brace_start) {
                match ch {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            body_end = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if brace_count == 0 {
                let body: String = chars[brace_start..body_end].iter().collect();
                return Some(body);
            }
        }

        None
    }

    /// 提取箭头函数体内容
    fn extract_arrow_function_body(&self, content: &str, start_pos: usize) -> Option<String> {
        let remaining_content = &content[start_pos..];

        // 箭头函数可能有 { } 包围的函数体，也可能是单表达式
        if let Some(arrow_pos) = remaining_content.find("=>") {
            let after_arrow = &remaining_content[arrow_pos + 2..].trim_start();

            if after_arrow.starts_with('{') {
                // 有大括号的函数体
                return self.extract_function_body(content, start_pos + arrow_pos + 2);
            } else {
                // 单表达式，找到行尾或分号
                let end_pos = after_arrow.find(';')
                    .or_else(|| after_arrow.find('\n'))
                    .unwrap_or(after_arrow.len());

                return Some(after_arrow[..end_pos].to_string());
            }
        }

        None
    }

    fn calculate_simple_complexity(&self, function_body: &str) -> u32 {
        let mut complexity = 1; // 基础复杂度

        // 计算控制流语句
        complexity += self.patterns.if_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.for_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.while_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.switch_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.case_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.catch_regex.find_iter(function_body).count() as u32;

        complexity
    }

    /// 检测类型安全问题
    pub fn detect_type_safety_issues(&self, content: &str) -> Vec<TypeSafetyIssue> {
        let mut issues = Vec::new();

        // 检测 any 类型使用
        for regex_match in self.patterns.any_type_regex.find_iter(content) {
            let line_number = content[..regex_match.start()].lines().count() + 1;
            issues.push(TypeSafetyIssue {
                issue_type: TypeSafetyIssueType::AnyType,
                line_number,
                description: "使用了 any 类型，失去了类型安全性".to_string(),
                severity: TypeSafetySeverity::Medium,
            });
        }

        // 检测类型断言
        for regex_match in self.patterns.type_assertion_regex.find_iter(content) {
            let line_number = content[..regex_match.start()].lines().count() + 1;
            issues.push(TypeSafetyIssue {
                issue_type: TypeSafetyIssueType::TypeAssertion,
                line_number,
                description: "使用了类型断言，可能绕过类型检查".to_string(),
                severity: TypeSafetySeverity::Low,
            });
        }

        // 检测 @ts-ignore 注释
        for regex_match in self.patterns.ts_ignore_regex.find_iter(content) {
            let line_number = content[..regex_match.start()].lines().count() + 1;
            issues.push(TypeSafetyIssue {
                issue_type: TypeSafetyIssueType::TsIgnore,
                line_number,
                description: "使用了 @ts-ignore 忽略类型错误".to_string(),
                severity: TypeSafetySeverity::High,
            });
        }

        issues
    }
}

impl LanguageAnalyzer for TypeScriptAnalyzer {
    fn analyze_features(&self, content: &str) -> Vec<LanguageFeature> {
        let mut features = Vec::new();

        // 提取模块
        features.extend(self.extract_module_declarations(content));

        // 提取函数
        features.extend(self.extract_functions(content));

        // 提取类
        features.extend(self.extract_classes(content));

        // 提取接口
        features.extend(self.extract_interfaces(content));

        features
    }
}

/// TypeScript 语言正则表达式模式
struct TypeScriptPatterns {
    module_regex: Regex,
    function_regex: Regex,
    arrow_function_regex: Regex,
    method_regex: Regex,
    class_regex: Regex,
    interface_regex: Regex,
    type_annotation_regex: Regex,
    generic_regex: Regex,
    union_type_regex: Regex,
    intersection_type_regex: Regex,
    mapped_type_regex: Regex,
    conditional_type_regex: Regex,
    utility_type_regex: Regex,
    async_await_regex: Regex,
    promise_regex: Regex,
    decorator_regex: Regex,
    es6_import_regex: Regex,
    es6_export_regex: Regex,
    namespace_regex: Regex,
    access_modifier_regex: Regex,
    abstract_class_regex: Regex,
    enum_regex: Regex,
    type_assertion_regex: Regex,
    optional_chaining_regex: Regex,
    nullish_coalescing_regex: Regex,
    require_regex: Regex,
    commonjs_export_regex: Regex,
    named_import_regex: Regex,
    default_import_regex: Regex,
    if_regex: Regex,
    for_regex: Regex,
    while_regex: Regex,
    switch_regex: Regex,
    case_regex: Regex,
    catch_regex: Regex,
    any_type_regex: Regex,
    ts_ignore_regex: Regex,
}

impl TypeScriptPatterns {
    fn new() -> Self {
        Self {
            module_regex: Regex::new(r"(?m)^module\s+(\w+)").unwrap(),
            function_regex: Regex::new(r"(?m)function\s+(\w+)\s*\(").unwrap(),
            arrow_function_regex: Regex::new(r"(?m)(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?(?:\([^)]*\)\s*)?=>").unwrap(),
            method_regex: Regex::new(r"(?m)(\w+)\s*\([^)]*\)\s*\{").unwrap(),
            class_regex: Regex::new(r"(?m)^(?:export\s+)?(?:abstract\s+)?class\s+(\w+)").unwrap(),
            interface_regex: Regex::new(r"(?m)^(?:export\s+)?interface\s+(\w+)").unwrap(),
            type_annotation_regex: Regex::new(r":\s*\w+").unwrap(),
            generic_regex: Regex::new(r"<[A-Z]\w*(?:\s*,\s*[A-Z]\w*)*>").unwrap(),
            union_type_regex: Regex::new(r"\w+\s*\|\s*\w+").unwrap(),
            intersection_type_regex: Regex::new(r"\w+\s*&\s*\w+").unwrap(),
            mapped_type_regex: Regex::new(r"\{\s*\[\s*\w+\s+in\s+\w+\s*\]").unwrap(),
            conditional_type_regex: Regex::new(r"\w+\s+extends\s+\w+\s*\?\s*\w+\s*:\s*\w+").unwrap(),
            utility_type_regex: Regex::new(r"\b(?:Partial|Required|Readonly|Record|Pick|Omit|Exclude|Extract|NonNullable|ReturnType|InstanceType)<").unwrap(),
            async_await_regex: Regex::new(r"\basync\s+|\bawait\s+").unwrap(),
            promise_regex: Regex::new(r"\bPromise<").unwrap(),
            decorator_regex: Regex::new(r"@\w+").unwrap(),
            es6_import_regex: Regex::new(r#"(?m)^import\s+.*from\s+['"]([^'"]+)['"]"#).unwrap(),
            es6_export_regex: Regex::new(r"(?m)^export\s+(?:default\s+)?(?:class|function|interface|type|const|let|var)?\s*(\w+)?").unwrap(),
            namespace_regex: Regex::new(r"(?m)^namespace\s+(\w+)").unwrap(),
            access_modifier_regex: Regex::new(r"\b(?:public|private|protected)\s+").unwrap(),
            abstract_class_regex: Regex::new(r"(?m)^abstract\s+class\s+(\w+)").unwrap(),
            enum_regex: Regex::new(r"(?m)^(?:export\s+)?enum\s+(\w+)").unwrap(),
            type_assertion_regex: Regex::new(r"<\w+>|\bas\s+\w+").unwrap(),
            optional_chaining_regex: Regex::new(r"\?\.\w+").unwrap(),
            nullish_coalescing_regex: Regex::new(r"\?\?").unwrap(),
            require_regex: Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap(),
            commonjs_export_regex: Regex::new(r"(?:module\.exports|exports)\.(\w+)").unwrap(),
            named_import_regex: Regex::new(r"import\s*\{\s*([^}]+)\s*\}").unwrap(),
            default_import_regex: Regex::new(r"import\s+(\w+)\s+from").unwrap(),
            if_regex: Regex::new(r"\bif\s*\(").unwrap(),
            for_regex: Regex::new(r"\bfor\s*\(").unwrap(),
            while_regex: Regex::new(r"\bwhile\s*\(").unwrap(),
            switch_regex: Regex::new(r"\bswitch\s*\(").unwrap(),
            case_regex: Regex::new(r"\bcase\s+").unwrap(),
            catch_regex: Regex::new(r"\bcatch\s*\(").unwrap(),
            any_type_regex: Regex::new(r":\s*any\b").unwrap(),
            ts_ignore_regex: Regex::new(r"@ts-ignore").unwrap(),
        }
    }
}

/// TypeScript 代码模式检测结果
#[derive(Debug, Default, Clone)]
pub struct TypeScriptCodePatterns {
    pub has_type_annotations: bool,
    pub has_generic_types: bool,
    pub has_union_types: bool,
    pub has_intersection_types: bool,
    pub has_mapped_types: bool,
    pub has_conditional_types: bool,
    pub has_utility_types: bool,
    pub has_async_await: bool,
    pub has_promises: bool,
    pub has_decorators: bool,
    pub has_es6_imports: bool,
    pub has_es6_exports: bool,
    pub has_namespace: bool,
    pub has_access_modifiers: bool,
    pub has_abstract_classes: bool,
    pub has_enums: bool,
    pub has_type_assertions: bool,
    pub has_optional_chaining: bool,
    pub has_nullish_coalescing: bool,
}

/// 导入导出分析结果
#[derive(Debug, Clone)]
pub struct ImportExportAnalysis {
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<ExportInfo>,
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub module_path: String,
    pub import_type: ImportType,
    pub imported_items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExportInfo {
    pub export_type: ExportType,
    pub exported_item: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ImportType {
    ES6,
    CommonJS,
}

#[derive(Debug, Clone)]
pub enum ExportType {
    ES6,
    CommonJS,
}

/// 函数复杂度信息
#[derive(Debug, Clone)]
pub struct FunctionComplexity {
    pub name: String,
    pub cyclomatic_complexity: u32,
    pub line_count: usize,
    pub is_async: bool,
}

/// 类型安全问题
#[derive(Debug, Clone)]
pub struct TypeSafetyIssue {
    pub issue_type: TypeSafetyIssueType,
    pub line_number: usize,
    pub description: String,
    pub severity: TypeSafetySeverity,
}

#[derive(Debug, Clone)]
pub enum TypeSafetyIssueType {
    AnyType,
    TypeAssertion,
    TsIgnore,
}

#[derive(Debug, Clone)]
pub enum TypeSafetySeverity {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_analyzer_creation() {
        let analyzer = TypeScriptAnalyzer::new();
        // 验证分析器创建成功
        assert!(analyzer.patterns.function_regex.is_match("function test() {"));
    }

    #[test]
    fn test_function_extraction() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
function regularFunction() {
    console.log("regular");
}

const arrowFunction = () => {
    console.log("arrow");
};

const asyncArrow = async () => {
    await somePromise();
};

class MyClass {
    methodFunction() {
        console.log("method");
    }
}
"#;

        let functions = analyzer.extract_functions(code);
        assert!(functions.len() >= 3);

        let function_names: Vec<String> = functions.iter()
            .filter_map(|f| if let LanguageFeature::Function(name) = f { Some(name.clone()) } else { None })
            .collect();

        assert!(function_names.contains(&"regularFunction".to_string()));
        assert!(function_names.contains(&"arrowFunction".to_string()));
        assert!(function_names.contains(&"asyncArrow".to_string()));
    }

    #[test]
    fn test_class_extraction() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
class User {
    name: string;
    age: number;
}

export class ApiClient {
    baseUrl: string;
}

abstract class BaseEntity {
    id: number;
}
"#;

        let classes = analyzer.extract_classes(code);
        assert_eq!(classes.len(), 3);

        let class_names: Vec<String> = classes.iter()
            .filter_map(|c| if let LanguageFeature::Class(name) = c { Some(name.clone()) } else { None })
            .collect();

        assert!(class_names.contains(&"User".to_string()));
        assert!(class_names.contains(&"ApiClient".to_string()));
        assert!(class_names.contains(&"BaseEntity".to_string()));
    }

    #[test]
    fn test_interface_extraction() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
interface User {
    name: string;
    age: number;
}

export interface ApiResponse<T> {
    data: T;
    status: number;
}
"#;

        let interfaces = analyzer.extract_interfaces(code);
        assert_eq!(interfaces.len(), 2);

        let interface_names: Vec<String> = interfaces.iter()
            .filter_map(|i| if let LanguageFeature::Interface(name) = i { Some(name.clone()) } else { None })
            .collect();

        assert!(interface_names.contains(&"User".to_string()));
        assert!(interface_names.contains(&"ApiResponse".to_string()));
    }

    #[test]
    fn test_typescript_pattern_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
interface User {
    name: string;
    age: number;
    email?: string;
}

type UserKeys = keyof User;
type PartialUser = Partial<User>;

class UserService {
    private users: User[] = [];

    async getUser(id: number): Promise<User | null> {
        const user = this.users.find(u => u.id === id);
        const name = user?.name;
        return user ?? null;
    }

    @deprecated
    oldMethod() {
        // deprecated method
    }
}

enum Status {
    Active = "active",
    Inactive = "inactive"
}

const processUser = <T extends User>(user: T): T => {
    return { ...user, processed: true } as T;
};
"#;

        let patterns = analyzer.detect_typescript_patterns(code);

        assert!(patterns.has_type_annotations);
        assert!(patterns.has_generic_types);
        assert!(patterns.has_union_types);
        assert!(patterns.has_utility_types);
        assert!(patterns.has_async_await);
        assert!(patterns.has_promises);
        assert!(patterns.has_decorators);
        assert!(patterns.has_access_modifiers);
        assert!(patterns.has_enums);
        assert!(patterns.has_optional_chaining);
        assert!(patterns.has_nullish_coalescing);
        assert!(patterns.has_type_assertions);
    }

    #[test]
    fn test_import_export_analysis() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { ApiClient } from './api-client';

const fs = require('fs');

export class UserService {
    // class implementation
}

export default UserService;

module.exports = { UserService };
"#;

        let analysis = analyzer.analyze_imports_exports(code);

        assert_eq!(analysis.imports.len(), 4);
        assert!(analysis.imports.iter().any(|i| i.module_path == "react"));
        assert!(analysis.imports.iter().any(|i| i.module_path == "axios"));
        assert!(analysis.imports.iter().any(|i| i.module_path == "./api-client"));
        assert!(analysis.imports.iter().any(|i| i.module_path == "fs"));

        assert!(!analysis.exports.is_empty());
    }

    #[test]
    fn test_function_complexity() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
function complexFunction(x: number): number {
    if (x > 0) {
        for (let i = 0; i < x; i++) {
            switch (i) {
                case 1:
                    return 1;
                case 2:
                    return 2;
                default:
                    continue;
            }
        }
    }

    while (x > 10) {
        try {
            x = processValue(x);
        } catch (error) {
            console.error(error);
            break;
        }
    }

    return 0;
}
"#;

        let complexities = analyzer.analyze_function_complexity(code);
        assert_eq!(complexities.len(), 1);

        let complexity = &complexities[0];
        assert_eq!(complexity.name, "complexFunction");
        assert!(complexity.cyclomatic_complexity > 1);
    }

    #[test]
    fn test_type_safety_issues() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
function unsafeFunction(data: any) {
    // @ts-ignore
    const result = data.someProperty;

    const typed = data as User;
    return typed;
}
"#;

        let issues = analyzer.detect_type_safety_issues(code);
        assert!(!issues.is_empty());

        let has_any_type = issues.iter().any(|issue| matches!(issue.issue_type, TypeSafetyIssueType::AnyType));
        let has_ts_ignore = issues.iter().any(|issue| matches!(issue.issue_type, TypeSafetyIssueType::TsIgnore));
        let has_type_assertion = issues.iter().any(|issue| matches!(issue.issue_type, TypeSafetyIssueType::TypeAssertion));

        assert!(has_any_type);
        assert!(has_ts_ignore);
        assert!(has_type_assertion);
    }

    #[test]
    fn test_language_analyzer_trait() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
module utils;

interface User {
    name: string;
}

class UserService {
    getUser(): User {
        return { name: "test" };
    }
}

function processUser(user: User): void {
    console.log(user.name);
}

const helper = () => {
    // helper function
};
"#;

        let features = analyzer.analyze_features(code);

        // 应该包含：1个模块，3个函数，1个类，1个接口
        assert!(features.len() >= 5);

        // 验证包含正确的特征类型
        let has_module = features.iter().any(|f| matches!(f, LanguageFeature::Module(_)));
        let has_function = features.iter().any(|f| matches!(f, LanguageFeature::Function(_)));
        let has_class = features.iter().any(|f| matches!(f, LanguageFeature::Class(_)));
        let has_interface = features.iter().any(|f| matches!(f, LanguageFeature::Interface(_)));

        assert!(has_module);
        assert!(has_function);
        assert!(has_class);
        assert!(has_interface);
    }

    #[test]
    fn test_async_pattern_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
async function fetchData(): Promise<string> {
    const response = await fetch('/api/data');
    return response.text();
}

const asyncArrow = async () => {
    await someAsyncOperation();
};
"#;

        let patterns = analyzer.detect_typescript_patterns(code);
        assert!(patterns.has_async_await);
        assert!(patterns.has_promises);
    }

    #[test]
    fn test_generic_type_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
interface Container<T> {
    value: T;
}

function identity<T>(arg: T): T {
    return arg;
}

class GenericClass<T, U> {
    process(input: T): U {
        // implementation
    }
}
"#;

        let patterns = analyzer.detect_typescript_patterns(code);
        assert!(patterns.has_generic_types);
    }

    #[test]
    fn test_utility_type_detection() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
interface User {
    name: string;
    age: number;
    email: string;
}

type PartialUser = Partial<User>;
type RequiredUser = Required<User>;
type UserName = Pick<User, 'name'>;
type UserWithoutEmail = Omit<User, 'email'>;
"#;

        let patterns = analyzer.detect_typescript_patterns(code);
        assert!(patterns.has_utility_types);
    }
}