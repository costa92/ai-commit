/// TypeScript 语言特定的代码模式和最佳实践检测
use regex::Regex;
use std::collections::HashMap;

/// TypeScript 代码模式检测器
pub struct TypeScriptPatternDetector {
    patterns: HashMap<String, TypeScriptPattern>,
}

impl TypeScriptPatternDetector {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // 类型注解使用
        patterns.insert("type_annotations".to_string(), TypeScriptPattern {
            name: "Type Annotations".to_string(),
            regex: Regex::new(r":\s*\w+").unwrap(),
            description: "使用类型注解提高代码类型安全性".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // any 类型使用（不推荐）
        patterns.insert("any_type_usage".to_string(), TypeScriptPattern {
            name: "Any Type Usage".to_string(),
            regex: Regex::new(r":\s*any\b").unwrap(),
            description: "使用 any 类型会失去类型安全性，应该使用具体类型".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Warning,
        });

        // 接口定义
        patterns.insert("interface_definition".to_string(), TypeScriptPattern {
            name: "Interface Definition".to_string(),
            regex: Regex::new(r"(?m)^(?:export\s+)?interface\s+\w+").unwrap(),
            description: "使用接口定义类型契约".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 泛型使用
        patterns.insert("generic_types".to_string(), TypeScriptPattern {
            name: "Generic Types".to_string(),
            regex: Regex::new(r"<[A-Z]\w*(?:\s*,\s*[A-Z]\w*)*>").unwrap(),
            description: "使用泛型提高代码复用性和类型安全性".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 联合类型
        patterns.insert("union_types".to_string(), TypeScriptPattern {
            name: "Union Types".to_string(),
            regex: Regex::new(r"\w+\s*\|\s*\w+").unwrap(),
            description: "使用联合类型表示多种可能的类型".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 可选属性
        patterns.insert("optional_properties".to_string(), TypeScriptPattern {
            name: "Optional Properties".to_string(),
            regex: Regex::new(r"\w+\?\s*:").unwrap(),
            description: "使用可选属性标记非必需字段".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 严格空值检查
        patterns.insert("null_undefined_handling".to_string(), TypeScriptPattern {
            name: "Null/Undefined Handling".to_string(),
            regex: Regex::new(r"\|\s*null\s*\|\s*undefined|\|\s*undefined\s*\|\s*null").unwrap(),
            description: "明确处理 null 和 undefined 类型".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 类型断言（需谨慎使用）
        patterns.insert("type_assertions".to_string(), TypeScriptPattern {
            name: "Type Assertions".to_string(),
            regex: Regex::new(r"<\w+>|\bas\s+\w+").unwrap(),
            description: "类型断言绕过类型检查，应谨慎使用".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Info,
        });

        // @ts-ignore 使用（不推荐）
        patterns.insert("ts_ignore".to_string(), TypeScriptPattern {
            name: "TypeScript Ignore".to_string(),
            regex: Regex::new(r"@ts-ignore").unwrap(),
            description: "使用 @ts-ignore 忽略类型错误，应该修复根本问题".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Warning,
        });

        // 异步函数使用
        patterns.insert("async_functions".to_string(), TypeScriptPattern {
            name: "Async Functions".to_string(),
            regex: Regex::new(r"\basync\s+function|\basync\s+\w+\s*=>").unwrap(),
            description: "使用异步函数处理异步操作".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // Promise 类型
        patterns.insert("promise_types".to_string(), TypeScriptPattern {
            name: "Promise Types".to_string(),
            regex: Regex::new(r"\bPromise<\w+>").unwrap(),
            description: "使用 Promise 类型标注异步返回值".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 装饰器使用
        patterns.insert("decorators".to_string(), TypeScriptPattern {
            name: "Decorators".to_string(),
            regex: Regex::new(r"@\w+").unwrap(),
            description: "使用装饰器进行元编程".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 枚举定义
        patterns.insert("enum_definition".to_string(), TypeScriptPattern {
            name: "Enum Definition".to_string(),
            regex: Regex::new(r"(?m)^(?:export\s+)?enum\s+\w+").unwrap(),
            description: "使用枚举定义常量集合".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 访问修饰符
        patterns.insert("access_modifiers".to_string(), TypeScriptPattern {
            name: "Access Modifiers".to_string(),
            regex: Regex::new(r"\b(?:public|private|protected)\s+").unwrap(),
            description: "使用访问修饰符控制成员可见性".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 只读属性
        patterns.insert("readonly_properties".to_string(), TypeScriptPattern {
            name: "Readonly Properties".to_string(),
            regex: Regex::new(r"\breadonly\s+\w+").unwrap(),
            description: "使用 readonly 标记不可变属性".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 工具类型使用
        patterns.insert("utility_types".to_string(), TypeScriptPattern {
            name: "Utility Types".to_string(),
            regex: Regex::new(r"\b(?:Partial|Required|Readonly|Record|Pick|Omit|Exclude|Extract|NonNullable|ReturnType|InstanceType)<").unwrap(),
            description: "使用内置工具类型进行类型转换".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 映射类型
        patterns.insert("mapped_types".to_string(), TypeScriptPattern {
            name: "Mapped Types".to_string(),
            regex: Regex::new(r"\{\s*\[\s*\w+\s+in\s+\w+\s*\]").unwrap(),
            description: "使用映射类型创建新的类型".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 条件类型
        patterns.insert("conditional_types".to_string(), TypeScriptPattern {
            name: "Conditional Types".to_string(),
            regex: Regex::new(r"\w+\s+extends\s+\w+\s*\?\s*\w+\s*:\s*\w+").unwrap(),
            description: "使用条件类型进行类型选择".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 可选链操作符
        patterns.insert("optional_chaining".to_string(), TypeScriptPattern {
            name: "Optional Chaining".to_string(),
            regex: Regex::new(r"\?\.\w+").unwrap(),
            description: "使用可选链操作符安全访问嵌套属性".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 空值合并操作符
        patterns.insert("nullish_coalescing".to_string(), TypeScriptPattern {
            name: "Nullish Coalescing".to_string(),
            regex: Regex::new(r"\?\?").unwrap(),
            description: "使用空值合并操作符处理 null/undefined".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 类型守卫
        patterns.insert("type_guards".to_string(), TypeScriptPattern {
            name: "Type Guards".to_string(),
            regex: Regex::new(r"is\s+\w+").unwrap(),
            description: "使用类型守卫进行运行时类型检查".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 命名空间使用（不推荐）
        patterns.insert("namespace_usage".to_string(), TypeScriptPattern {
            name: "Namespace Usage".to_string(),
            regex: Regex::new(r"(?m)^namespace\s+\w+").unwrap(),
            description: "命名空间已过时，推荐使用 ES6 模块".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Info,
        });

        Self { patterns }
    }

    /// 检测代码中的所有模式
    pub fn detect_patterns(&self, content: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        for (pattern_id, pattern) in &self.patterns {
            for regex_match in pattern.regex.find_iter(content) {
                let line_number = content[..regex_match.start()].lines().count() + 1;

                matches.push(PatternMatch {
                    pattern_id: pattern_id.clone(),
                    pattern_name: pattern.name.clone(),
                    description: pattern.description.clone(),
                    line_number,
                    matched_text: regex_match.as_str().to_string(),
                    is_good_practice: pattern.is_good_practice,
                    severity: pattern.severity.clone(),
                });
            }
        }

        matches
    }

    /// 检测特定类型的模式
    pub fn detect_pattern_type(&self, content: &str, pattern_type: TypeScriptPatternType) -> Vec<PatternMatch> {
        let pattern_ids = match pattern_type {
            TypeScriptPatternType::TypeSafety => vec![
                "type_annotations", "any_type_usage", "interface_definition",
                "generic_types", "union_types", "type_assertions", "ts_ignore"
            ],
            TypeScriptPatternType::ModernFeatures => vec![
                "optional_chaining", "nullish_coalescing", "utility_types",
                "mapped_types", "conditional_types", "optional_properties"
            ],
            TypeScriptPatternType::AsyncProgramming => vec![
                "async_functions", "promise_types"
            ],
            TypeScriptPatternType::ObjectOriented => vec![
                "access_modifiers", "readonly_properties", "decorators", "enum_definition"
            ],
            TypeScriptPatternType::CodeQuality => vec![
                "type_guards", "null_undefined_handling", "namespace_usage"
            ],
        };

        let mut matches = Vec::new();
        for pattern_id in pattern_ids {
            if let Some(pattern) = self.patterns.get(pattern_id) {
                for regex_match in pattern.regex.find_iter(content) {
                    let line_number = content[..regex_match.start()].lines().count() + 1;

                    matches.push(PatternMatch {
                        pattern_id: pattern_id.to_string(),
                        pattern_name: pattern.name.clone(),
                        description: pattern.description.clone(),
                        line_number,
                        matched_text: regex_match.as_str().to_string(),
                        is_good_practice: pattern.is_good_practice,
                        severity: pattern.severity.clone(),
                    });
                }
            }
        }

        matches
    }

    /// 生成模式检测报告
    pub fn generate_report(&self, content: &str) -> TypeScriptPatternReport {
        let all_matches = self.detect_patterns(content);

        let good_practices = all_matches.iter().filter(|m| m.is_good_practice).count();
        let warnings = all_matches.iter().filter(|m| matches!(m.severity, PatternSeverity::Warning)).count();
        let infos = all_matches.iter().filter(|m| matches!(m.severity, PatternSeverity::Info)).count();

        TypeScriptPatternReport {
            total_patterns: all_matches.len(),
            good_practices,
            warnings,
            infos,
            matches: all_matches,
        }
    }
}

/// TypeScript 代码模式定义
#[derive(Debug, Clone)]
pub struct TypeScriptPattern {
    pub name: String,
    pub regex: Regex,
    pub description: String,
    pub is_good_practice: bool,
    pub severity: PatternSeverity,
}

/// 模式匹配结果
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub pattern_name: String,
    pub description: String,
    pub line_number: usize,
    pub matched_text: String,
    pub is_good_practice: bool,
    pub severity: PatternSeverity,
}

/// 模式严重程度
#[derive(Debug, Clone, PartialEq)]
pub enum PatternSeverity {
    Info,
    Warning,
    Error,
}

/// TypeScript 模式类型
#[derive(Debug, Clone)]
pub enum TypeScriptPatternType {
    TypeSafety,
    ModernFeatures,
    AsyncProgramming,
    ObjectOriented,
    CodeQuality,
}

/// TypeScript 模式检测报告
#[derive(Debug, Clone)]
pub struct TypeScriptPatternReport {
    pub total_patterns: usize,
    pub good_practices: usize,
    pub warnings: usize,
    pub infos: usize,
    pub matches: Vec<PatternMatch>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_annotation_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
function greet(name: string): string {
    return `Hello, ${name}!`;
}

const age: number = 25;
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::TypeSafety);
        let type_annotations = matches.iter()
            .find(|m| m.pattern_id == "type_annotations")
            .expect("Should find type annotations");

        assert!(type_annotations.is_good_practice);
    }

    #[test]
    fn test_any_type_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
function processData(data: any): void {
    console.log(data);
}
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::TypeSafety);
        let any_usage = matches.iter()
            .find(|m| m.pattern_id == "any_type_usage")
            .expect("Should find any type usage");

        assert!(!any_usage.is_good_practice);
        assert_eq!(any_usage.severity, PatternSeverity::Warning);
    }

    #[test]
    fn test_interface_definition_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
interface User {
    name: string;
    age: number;
}

export interface ApiResponse<T> {
    data: T;
    success: boolean;
}
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::TypeSafety);
        let interface_def = matches.iter()
            .find(|m| m.pattern_id == "interface_definition")
            .expect("Should find interface definition");

        assert!(interface_def.is_good_practice);
    }

    #[test]
    fn test_generic_types_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
function identity<T>(arg: T): T {
    return arg;
}

class Container<T, U> {
    private value: T;
    private metadata: U;
}
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::TypeSafety);
        let generics = matches.iter()
            .find(|m| m.pattern_id == "generic_types")
            .expect("Should find generic types");

        assert!(generics.is_good_practice);
    }

    #[test]
    fn test_union_types_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
type Status = 'loading' | 'success' | 'error';
type ID = string | number;

function processValue(value: string | number): void {
    console.log(value);
}
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::TypeSafety);
        let union_types = matches.iter()
            .find(|m| m.pattern_id == "union_types")
            .expect("Should find union types");

        assert!(union_types.is_good_practice);
    }

    #[test]
    fn test_optional_chaining_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
const user = {
    profile: {
        name: 'John'
    }
};

const name = user?.profile?.name;
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::ModernFeatures);
        let optional_chaining = matches.iter()
            .find(|m| m.pattern_id == "optional_chaining")
            .expect("Should find optional chaining");

        assert!(optional_chaining.is_good_practice);
    }

    #[test]
    fn test_nullish_coalescing_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
const config = {
    timeout: null
};

const timeout = config.timeout ?? 5000;
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::ModernFeatures);
        let nullish_coalescing = matches.iter()
            .find(|m| m.pattern_id == "nullish_coalescing")
            .expect("Should find nullish coalescing");

        assert!(nullish_coalescing.is_good_practice);
    }

    #[test]
    fn test_async_functions_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
async function fetchData(): Promise<string> {
    const response = await fetch('/api/data');
    return response.text();
}

const asyncArrow = async () => {
    return 'result';
};
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::AsyncProgramming);
        let async_functions = matches.iter()
            .find(|m| m.pattern_id == "async_functions")
            .expect("Should find async functions");

        assert!(async_functions.is_good_practice);
    }

    #[test]
    fn test_promise_types_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
function getData(): Promise<User> {
    return fetch('/api/user').then(r => r.json());
}

const userPromise: Promise<User> = getData();
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::AsyncProgramming);
        let promise_types = matches.iter()
            .find(|m| m.pattern_id == "promise_types")
            .expect("Should find promise types");

        assert!(promise_types.is_good_practice);
    }

    #[test]
    fn test_access_modifiers_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
class UserService {
    private apiKey: string;
    protected baseUrl: string;
    public timeout: number;

    private authenticate(): void {
        // private method
    }
}
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::ObjectOriented);
        let access_modifiers = matches.iter()
            .find(|m| m.pattern_id == "access_modifiers")
            .expect("Should find access modifiers");

        assert!(access_modifiers.is_good_practice);
    }

    #[test]
    fn test_decorators_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
class ApiController {
    @Get('/users')
    @UseGuards(AuthGuard)
    getUsers(): User[] {
        return [];
    }
}
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::ObjectOriented);
        let decorators = matches.iter()
            .find(|m| m.pattern_id == "decorators")
            .expect("Should find decorators");

        assert!(decorators.is_good_practice);
    }

    #[test]
    fn test_utility_types_detection() {
        let detector = TypeScriptPatternDetector::new();
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

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::ModernFeatures);
        let utility_types = matches.iter()
            .find(|m| m.pattern_id == "utility_types")
            .expect("Should find utility types");

        assert!(utility_types.is_good_practice);
    }

    #[test]
    fn test_ts_ignore_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
function problematicFunction() {
    // @ts-ignore
    const result = someUntypedLibrary.method();
    return result;
}
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::TypeSafety);
        let ts_ignore = matches.iter()
            .find(|m| m.pattern_id == "ts_ignore")
            .expect("Should find ts-ignore");

        assert!(!ts_ignore.is_good_practice);
        assert_eq!(ts_ignore.severity, PatternSeverity::Warning);
    }

    #[test]
    fn test_generate_report() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
interface User {
    name: string;
    age?: number;
}

class UserService {
    private users: User[] = [];

    async getUser(id: string): Promise<User | null> {
        const user = this.users.find(u => u.name === id);
        return user ?? null;
    }
}

enum Status {
    Active = "active",
    Inactive = "inactive"
}
"#;

        let report = detector.generate_report(code);

        assert!(report.total_patterns > 0);
        assert!(report.good_practices > 0);
        assert!(!report.matches.is_empty());
    }

    #[test]
    fn test_type_guards_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
function isString(value: unknown): value is string {
    return typeof value === 'string';
}

function isUser(obj: any): obj is User {
    return obj && typeof obj.name === 'string';
}
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::CodeQuality);
        let type_guards = matches.iter()
            .find(|m| m.pattern_id == "type_guards")
            .expect("Should find type guards");

        assert!(type_guards.is_good_practice);
    }

    #[test]
    fn test_readonly_properties_detection() {
        let detector = TypeScriptPatternDetector::new();
        let code = r#"
interface Config {
    readonly apiUrl: string;
    readonly timeout: number;
}

class ImmutableData {
    readonly createdAt: Date;
    readonly id: string;
}
"#;

        let matches = detector.detect_pattern_type(code, TypeScriptPatternType::ObjectOriented);
        let readonly_props = matches.iter()
            .find(|m| m.pattern_id == "readonly_properties")
            .expect("Should find readonly properties");

        assert!(readonly_props.is_good_practice);
    }
}