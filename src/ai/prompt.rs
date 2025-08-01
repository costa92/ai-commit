use std::env;
use std::fs;
use once_cell::sync::Lazy;
use std::sync::RwLock;

// 提示模板缓存
static PROMPT_CACHE: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

// 加载提示模板（仅执行一次）
fn load_prompt_template() -> String {
    let default_path = "commit-prompt.txt";
    let prompt_path = if std::path::Path::new(default_path).exists() {
        default_path
    } else {
        // 如果项目中不存在，则检查环境变量配置
        &env::var("AI_COMMIT_PROMPT_PATH").unwrap_or_else(|_| default_path.to_owned())
    };

    // 尝试读取外部文件，失败则使用内置模板
    if std::path::Path::new(prompt_path).exists() {
        match fs::read_to_string(prompt_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("无法读取提示词文件 {}: {}，使用内置模板", prompt_path, e);
                include_str!("../../commit-prompt.txt").to_owned()
            }
        }
    } else {
        // 内置默认模板，编译时读取 commit-prompt.txt
        include_str!("../../commit-prompt.txt").to_owned()
    }
}

pub fn get_prompt(diff: &str) -> String {
    // 检查缓存
    {
        let cache = PROMPT_CACHE.read().unwrap();
        if let Some(ref template) = *cache {
            return template.replace("{{git_diff}}", diff);
        }
    }
    
    // 加载并缓存模板
    let template = load_prompt_template();
    *PROMPT_CACHE.write().unwrap() = Some(template.clone());
    
    template.replace("{{git_diff}}", diff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_prompt_with_diff() {
        let diff = "diff --git a/test.txt b/test.txt\n+added line";
        let prompt = get_prompt(diff);
        
        // 验证 diff 已被正确替换
        assert!(prompt.contains("added line"));
        assert!(!prompt.contains("{{git_diff}}"));
    }

    #[test]
    fn test_get_prompt_empty_diff() {
        let prompt = get_prompt("");
        
        // 验证空 diff 不会导致错误
        assert!(!prompt.contains("{{git_diff}}"));
        assert!(!prompt.is_empty()); // 应该包含模板内容
    }

    #[test]
    fn test_get_prompt_multiple_calls_cached() {
        let diff1 = "first diff";
        let diff2 = "second diff";
        
        let prompt1 = get_prompt(diff1);
        let prompt2 = get_prompt(diff2);
        
        // 验证缓存工作正常
        assert!(prompt1.contains("first diff"));
        assert!(prompt2.contains("second diff"));
        assert!(!prompt1.contains("{{git_diff}}"));
        assert!(!prompt2.contains("{{git_diff}}"));
    }

    #[test]
    fn test_load_prompt_template_default() {
        let template = load_prompt_template();
        
        // 验证加载的模板包含预期内容（更新为实际模板内容）
        assert!(template.contains("{{git_diff}}"));
        assert!(template.contains("输出格式"));
        assert!(template.contains("feat|fix|docs"));
    }

    #[test]
    fn test_load_prompt_template_with_custom_file() {
        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();  
        let custom_content = "Custom template with {{git_diff}} placeholder";
        temp_file.write_all(custom_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // 设置环境变量（只有在没有本地 commit-prompt.txt 时才会生效）
        let original_path = std::env::var("AI_COMMIT_PROMPT_PATH").ok();
        std::env::set_var("AI_COMMIT_PROMPT_PATH", temp_file.path());

        // 清除缓存以便重新加载
        *PROMPT_CACHE.write().unwrap() = None;

        let template = load_prompt_template();
        
        // 验证模板内容 - 如果存在本地 commit-prompt.txt，则使用本地文件
        // 否则使用环境变量指定的文件
        if std::path::Path::new("commit-prompt.txt").exists() {
            // 如果存在本地文件，则应该使用本地文件内容
            assert!(template.contains("{{git_diff}}"));
            assert!(!template.is_empty());
        } else {
            // 如果没有本地文件，则应该使用环境变量指定的文件
            assert_eq!(template, custom_content);
        }

        // 恢复原始环境变量
        match original_path {
            Some(path) => std::env::set_var("AI_COMMIT_PROMPT_PATH", path),
            None => std::env::remove_var("AI_COMMIT_PROMPT_PATH"),
        }

        // 清除缓存
        *PROMPT_CACHE.write().unwrap() = None;
    }

    #[test]
    fn test_prompt_cache_singleton() {
        // 清除缓存
        *PROMPT_CACHE.write().unwrap() = None;
        
        let diff = "test diff";
        let prompt1 = get_prompt(diff);
        
        // 验证缓存已设置
        {
            let cache = PROMPT_CACHE.read().unwrap();
            assert!(cache.is_some());
        }
        
        let prompt2 = get_prompt(diff);
        
        // 两次调用应该返回相同结果
        assert_eq!(prompt1, prompt2);
    }

    #[test]
    fn test_template_placeholder_replacement() {
        let template = "Before {{git_diff}} After";
        let diff = "REPLACEMENT";
        let result = template.replace("{{git_diff}}", diff);
        
        assert_eq!(result, "Before REPLACEMENT After");
        assert!(!result.contains("{{git_diff}}"));
    }

    #[test]
    fn test_env_var_handling() {
        // 测试环境变量处理
        let original = std::env::var("AI_COMMIT_PROMPT_PATH").ok();
        
        // 设置不存在的路径
        std::env::set_var("AI_COMMIT_PROMPT_PATH", "/nonexistent/path.txt");
        
        // 清除缓存
        *PROMPT_CACHE.write().unwrap() = None;
        
        // 应该回退到默认模板
        let template = load_prompt_template();
        assert!(template.contains("输出格式"));
        
        // 恢复原始环境变量
        match original {
            Some(path) => std::env::set_var("AI_COMMIT_PROMPT_PATH", path),
            None => std::env::remove_var("AI_COMMIT_PROMPT_PATH"),
        }
        
        // 清除缓存
        *PROMPT_CACHE.write().unwrap() = None;
    }

    #[test]
    fn test_concurrent_cache_access() {
        use std::thread;
        
        // 清除缓存
        *PROMPT_CACHE.write().unwrap() = None;
        
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let diff = format!("diff content {}", i);
                    get_prompt(&diff)
                })
            })
            .collect();
        
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        
        // 所有结果都应该成功生成
        assert_eq!(results.len(), 10);
        for (i, result) in results.iter().enumerate() {
            assert!(result.contains(&format!("diff content {}", i)));
        }
    }
}
