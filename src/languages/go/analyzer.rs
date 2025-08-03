use crate::languages::{Language, LanguageAnalyzer, LanguageFeature, LanguageAnalysisResult};
use super::{extract_go_feature, GoFeatureType};

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

    /// 从文件路径提取 Go 特定的作用域建议
    fn extract_go_scope_from_path(&self, file_path: &str) -> Vec<String> {
        let mut scopes = Vec::new();

        // 基于路径结构的作用域建议
        if file_path.contains("main.go") {
            scopes.push("main".to_string());
        } else if file_path.contains("cmd/") {
            scopes.push("cmd".to_string());
        } else if file_path.contains("internal/") {
            scopes.push("internal".to_string());
        } else if file_path.contains("pkg/") {
            scopes.push("pkg".to_string());
        } else if file_path.contains("test/") || file_path.contains("_test.go") {
            scopes.push("test".to_string());
        } else if file_path.contains("examples/") {
            scopes.push("example".to_string());
        }

        // 基于包名的作用域建议
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            if let Some(package_name) = parent.file_name() {
                let name = package_name.to_string_lossy().to_string();
                if !["cmd", "internal", "pkg", "test", "examples"].contains(&name.as_str()) {
                    scopes.push(name);
                }
            }
        }

        // 如果是在子目录中，添加目录名作为作用域
        let path_parts: Vec<&str> = file_path.split('/').collect();
        if path_parts.len() > 1 {
            for part in &path_parts[..path_parts.len()-1] {
                if !["src", "cmd", "pkg", "internal"].contains(part) && !part.is_empty() {
                    scopes.push(part.to_string());
                }
            }
        }

        scopes
    }

    /// 分析 Go 特定的变更模式
    fn analyze_go_change_patterns(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut patterns = Vec::new();

        let function_count = features.iter().filter(|f| f.feature_type == "function").count();
        let method_count = features.iter().filter(|f| f.feature_type == "method").count();
        let struct_count = features.iter().filter(|f| f.feature_type == "struct").count();
        let interface_count = features.iter().filter(|f| f.feature_type == "interface").count();
        let import_count = features.iter().filter(|f| f.feature_type == "import").count();
        let package_count = features.iter().filter(|f| f.feature_type == "package").count();

        if function_count > 0 {
            patterns.push(format!("新增 {} 个函数", function_count));
        }
        if method_count > 0 {
            patterns.push(format!("新增 {} 个方法", method_count));
        }
        if struct_count > 0 {
            patterns.push(format!("新增 {} 个结构体", struct_count));
        }
        if interface_count > 0 {
            patterns.push(format!("新增 {} 个接口", interface_count));
        }
        if import_count > 3 {
            patterns.push("大量依赖导入变更".to_string());
        }
        if package_count > 0 {
            patterns.push(format!("包声明变更 ({} 个)", package_count));
        }

        // 检测测试相关变更
        let test_functions = features.iter().filter(|f| {
            f.feature_type == "function" && (
                f.name.starts_with("Test") || 
                f.name.starts_with("Benchmark") ||
                f.name.starts_with("Example")
            )
        }).count();
        
        if test_functions > 0 {
            patterns.push(format!("新增 {} 个测试函数", test_functions));
        }

        // 检测并发相关变更
        let goroutine_usage = features.iter().filter(|f| {
            f.description.contains("go ") || f.description.contains("chan ")
        }).count();
        
        if goroutine_usage > 0 {
            patterns.push("涉及并发编程变更".to_string());
        }

        // 检测错误处理变更
        let error_handling = features.iter().filter(|f| {
            f.description.contains("error") || f.description.contains("Error")
        }).count();
        
        if error_handling > 0 {
            patterns.push("错误处理相关变更".to_string());
        }

        patterns
    }

    /// 生成 Go 特定的测试建议
    fn generate_go_test_suggestions(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut suggestions = Vec::new();

        let has_functions = features.iter().any(|f| f.feature_type == "function");
        let has_methods = features.iter().any(|f| f.feature_type == "method");
        let has_structs = features.iter().any(|f| f.feature_type == "struct");
        let has_interfaces = features.iter().any(|f| f.feature_type == "interface");
        let has_concurrency = features.iter().any(|f| 
            f.description.contains("go ") || f.description.contains("chan ")
        );

        if has_functions {
            suggestions.push("为新增函数编写单元测试".to_string());
            suggestions.push("使用表驱动测试模式".to_string());
        }
        if has_methods {
            suggestions.push("为结构体方法编写测试".to_string());
        }
        if has_structs {
            suggestions.push("测试结构体的序列化和反序列化".to_string());
        }
        if has_interfaces {
            suggestions.push("为接口实现编写集成测试".to_string());
            suggestions.push("使用 mock 对象测试接口依赖".to_string());
        }
        if has_concurrency {
            suggestions.push("编写并发安全测试".to_string());
            suggestions.push("使用 race detector 检测数据竞争".to_string());
        }

        // 基于 Go 最佳实践的建议
        suggestions.push("运行 go test -v 执行详细测试".to_string());
        suggestions.push("使用 go test -race 检测竞态条件".to_string());
        suggestions.push("运行 go test -cover 检查测试覆盖率".to_string());
        suggestions.push("使用 go vet 进行静态分析".to_string());

        // 检查是否需要基准测试
        let performance_critical = features.iter().any(|f|
            f.description.contains("benchmark") || 
            f.description.contains("performance") ||
            f.name.contains("process") ||
            f.name.contains("parse")
        );
        
        if performance_critical {
            suggestions.push("编写基准测试衡量性能".to_string());
        }

        suggestions
    }

    /// 生成 Go 特定的风险评估
    fn assess_go_risks(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut risks = Vec::new();

        let has_goroutines = features.iter().any(|f| f.description.contains("go "));
        let has_channels = features.iter().any(|f| f.description.contains("chan "));
        let has_many_interfaces = features.iter().filter(|f| f.feature_type == "interface").count() > 2;
        let has_many_structs = features.iter().filter(|f| f.feature_type == "struct").count() > 3;

        if has_goroutines {
            risks.push("使用 goroutine，需要检查是否存在泄漏风险".to_string());
        }
        if has_channels {
            risks.push("使用 channel，需要验证是否存在死锁风险".to_string());
        }
        if has_many_interfaces {
            risks.push("多个新接口可能影响 API 兼容性".to_string());
        }
        if has_many_structs {
            risks.push("大量结构体变更可能增加内存使用".to_string());
        }

        // 检查指针使用
        let has_pointers = features.iter().any(|f| 
            f.description.contains("*") && !f.description.contains("import")
        );
        if has_pointers {
            risks.push("使用指针，需要注意 nil 指针引用".to_string());
        }

        // 检查反射使用
        let has_reflection = features.iter().any(|f| 
            f.description.contains("reflect")
        );
        if has_reflection {
            risks.push("使用反射，可能影响性能和类型安全".to_string());
        }

        // 检查 unsafe 包使用
        let has_unsafe = features.iter().any(|f| 
            f.description.contains("unsafe")
        );
        if has_unsafe {
            risks.push("使用 unsafe 包，需要额外的安全性审查".to_string());
        }

        risks
    }
}

#[cfg(test)]
mod tests {
    include!("analyzer_tests.rs");
}

impl LanguageAnalyzer for GoAnalyzer {
    fn language(&self) -> Language {
        Language::Go
    }

    fn analyze_line(&self, line: &str, line_number: usize) -> Vec<LanguageFeature> {
        let mut features = Vec::new();

        if let Some(feature) = extract_go_feature(line, line_number) {
            features.push(feature);
        }

        // 检测测试函数
        if line.contains("func Test") || line.contains("func Benchmark") || line.contains("func Example") {
            features.push(LanguageFeature {
                feature_type: GoFeatureType::Test.as_str().to_string(),
                name: "test_function".to_string(),
                line_number: Some(line_number),
                description: format!("Go test: {}", line.trim()),
            });
        }

        // 检测 goroutine 启动
        if line.contains("go ") && (line.contains("func") || line.contains("()")) {
            features.push(LanguageFeature {
                feature_type: GoFeatureType::Goroutine.as_str().to_string(),
                name: "goroutine_start".to_string(),
                line_number: Some(line_number),
                description: format!("Goroutine start: {}", line.trim()),
            });
        }

        // 检测 channel 操作
        if line.contains("make(chan") || line.contains("chan ") {
            features.push(LanguageFeature {
                feature_type: GoFeatureType::Channel.as_str().to_string(),
                name: "channel_operation".to_string(),
                line_number: Some(line_number),
                description: format!("Channel operation: {}", line.trim()),
            });
        }

        features
    }

    fn extract_scope_suggestions(&self, file_path: &str) -> Vec<String> {
        self.extract_go_scope_from_path(file_path)
    }

    fn analyze_change_patterns(&self, features: &[LanguageFeature]) -> Vec<String> {
        self.analyze_go_change_patterns(features)
    }

    fn generate_test_suggestions(&self, features: &[LanguageFeature]) -> Vec<String> {
        self.generate_go_test_suggestions(features)
    }

    fn assess_risks(&self, features: &[LanguageFeature]) -> Vec<String> {
        self.assess_go_risks(features)
    }
}

