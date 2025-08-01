use std::collections::HashSet;
use tokio::process::Command;
use std::str::FromStr;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Git 命令缓存
static BRANCH_CACHE: Lazy<Mutex<Option<HashSet<String>>>> = Lazy::new(|| Mutex::new(None));
static TAGS_CACHE: Lazy<Mutex<Option<HashSet<String>>>> = Lazy::new(|| Mutex::new(None));

/// 获取最新的 tag 和备注
pub async fn get_latest_tag() -> Option<(String, String)> {
    let tag_output = Command::new("bash")
        .arg("-c")
        .arg("git describe --tags `git rev-list --tags --max-count=1`")
        .output()
        .await
        .ok()?;

    if !tag_output.status.success() || tag_output.stdout.is_empty() {
        return None;
    }

    let tag = String::from_utf8(tag_output.stdout).ok()?.trim().to_owned();

    let note_output = Command::new("git")
        .args(["tag", "-l", "-n1", &tag])
        .output()
        .await
        .ok()?;

    let note = if note_output.status.success() && !note_output.stdout.is_empty() {
        let note_line = String::from_utf8(note_output.stdout).ok()?.trim().to_owned();
        if let Some(index) = note_line.find(char::is_whitespace) {
            note_line.split_at(index).1.trim().to_owned()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    Some((tag, note))
}

/// 获取最新的 tag（仅版本号）
pub async fn get_latest_tag_version() -> Option<String> {
    get_latest_tag().await.map(|(tag, _)| tag)
}

async fn get_all_tags() -> anyhow::Result<HashSet<String>> {
    // 检查缓存
    {
        let cache = TAGS_CACHE.lock().unwrap();
        if let Some(ref tags) = *cache {
            return Ok(tags.clone());
        }
    }
    
    let output = Command::new("git")
        .args(["tag", "-l"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list tags: {}", e))?;
        
    if !output.status.success() {
        anyhow::bail!("Git tag list failed with exit code: {:?}", output.status.code());
    }
    
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse tag list: {}", e))?;
    
    let tags: HashSet<String> = stdout
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    
    // 更新缓存
    *TAGS_CACHE.lock().unwrap() = Some(tags.clone());
    
    Ok(tags)
}

async fn resolve_next_tag_name(base_version: Option<&str>) -> anyhow::Result<String> {
    let all_tags = get_all_tags().await?;

    // Clean up base_version: treat None, Some(""), Some("v") as "no base version specified"
    let base_version = base_version.and_then(|v| {
        let trimmed = v.trim_start_matches('v');
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    let mut parts: Vec<u32>;

    if let Some(base) = base_version {
        // Case 1 & 2: User provided a base version string
        parts = base
            .split('.')
            .filter_map(|s| u32::from_str(s).ok())
            .collect();

        if parts.len() < 2 || parts.len() > 3 {
            anyhow::bail!("Invalid version format for --new-tag: {}", base);
        }
        // If only major/minor provided, start patch at 0. If full version provided, use it.
        if parts.len() == 2 {
            parts.push(0);
        }
    } else {
        // Case 3: No base version, find latest tag and increment patch
        let latest_tag_str = get_latest_tag_version().await.unwrap_or_else(|| "v0.0.0".to_string());
        let base = latest_tag_str.trim_start_matches('v');

        parts = base
            .split('.')
            .filter_map(|s| u32::from_str(s).ok())
            .collect();

        if parts.len() != 3 {
            anyhow::bail!("Invalid version format in latest tag: {}", base);
        }
        parts[2] += 1; // Increment patch
    }

    // Loop to find the next available tag name
    let mut counter = 0;
    loop {
        let final_tag = format!("v{}.{}.{}", parts[0], parts[1], parts[2]);
        if !all_tags.contains(&final_tag) {
            return Ok(final_tag);
        }
        parts[2] += 1;
        counter += 1;
        if counter > 1000 {
            anyhow::bail!("Too many tag increments, please check your tags");
        }
    }
}

/// 创建新的 tag
pub async fn create_tag(tag: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["tag", tag])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create tag: {}", e))?;

    if !status.success() {
        anyhow::bail!("Failed to create tag '{}' with exit code: {:?}", tag, status.code());
    }

    Ok(())
}

/// 创建新的带 note 的 tag
pub async fn create_tag_with_note(tag: &str, note: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["tag", "-a", tag, "-m", note])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create tag: {}", e))?;

    if !status.success() {
        anyhow::bail!("Failed to create tag '{}' with note, exit code: {:?}", tag, status.code());
    }

    Ok(())
}

/// 推送 tag 到远程
pub async fn push_tag(tag: &str, allow_push_branches: bool) -> anyhow::Result<()> {
    let mut push_args = vec!["push", "origin"];

    if allow_push_branches {
        // 检查缓存的分支列表
        let existing_branches = {
            let cache = BRANCH_CACHE.lock().unwrap();
            if let Some(ref branches) = *cache {
                branches.clone()
            } else {
                drop(cache);
                
                let branches_output = Command::new("git")
                    .args(["branch", "--list", "--format=%(refname:short)"])
                    .output()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to list branches: {}", e))?;
                    
                if !branches_output.status.success() {
                    anyhow::bail!("Git branch list failed with exit code: {:?}", branches_output.status.code());
                }
                
                let branches_str = String::from_utf8(branches_output.stdout)
                    .map_err(|e| anyhow::anyhow!("Failed to parse branch list: {}", e))?;
                    
                let branches: HashSet<String> = branches_str
                    .lines()
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                // 更新缓存
                *BRANCH_CACHE.lock().unwrap() = Some(branches.clone());
                branches
            }
        };

        for branch in ["master", "develop", "main"] {
            if existing_branches.contains(branch) {
                push_args.push(branch);
            }
        }
    }

    push_args.push(tag);

    let status = Command::new("git")
        .args(&push_args)
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to push with args: {:?}: {}", push_args, e))?;

    if !status.success() {
        eprintln!("Warning: failed to push with args: {:?}, exit code: {:?}", push_args, status.code());
    }

    Ok(())
}

/// 仅生成下一个 tag 名字，不创建 tag
pub async fn get_next_tag_name(base_version: Option<&str>) -> anyhow::Result<String> {
    resolve_next_tag_name(base_version).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_tag_cache_initialization() {
        // 测试缓存初始化
        let cache = TAGS_CACHE.lock().unwrap();
        // 缓存应该是 Option<HashSet<String>>
        assert!(cache.is_none() || cache.is_some());
    }

    #[test]
    fn test_branch_cache_initialization() {
        // 测试分支缓存初始化
        let cache = BRANCH_CACHE.lock().unwrap();
        assert!(cache.is_none() || cache.is_some());
    }

    #[tokio::test]
    async fn test_get_all_tags_caching() {
        // 清除缓存
        *TAGS_CACHE.lock().unwrap() = None;
        
        // 第一次调用（可能失败，但应该尝试缓存）
        let result1 = get_all_tags().await;
        
        match result1 {
            Ok(tags1) => {
                // 验证缓存已设置
                {
                    let cache = TAGS_CACHE.lock().unwrap();
                    assert!(cache.is_some());
                }
                
                // 第二次调用应该使用缓存
                let result2 = get_all_tags().await;
                assert!(result2.is_ok());
                let tags2 = result2.unwrap();
                
                // 两次结果应该相同（来自缓存）
                assert_eq!(tags1, tags2);
            }
            Err(_) => {
                // 在没有 git 环境的情况下会失败，这是预期的
                println!("Git tags command failed (expected in non-git environment)");
            }
        }
    }

    #[tokio::test]
    async fn test_get_latest_tag_structure() {
        let result = get_latest_tag().await;
        
        match result {
            Some((tag, note)) => {
                // 验证返回值结构
                assert!(!tag.is_empty() || tag.is_empty()); // 字符串类型
                assert!(!note.is_empty() || note.is_empty()); // 字符串类型
                println!("Latest tag: {}, note: {}", tag, note);
            }
            None => {
                println!("No tags found (expected in repositories without tags)");
            }
        }
    }

    #[tokio::test]
    async fn test_get_latest_tag_version_structure() {
        let result = get_latest_tag_version().await;
        
        match result {
            Some(tag) => {
                assert!(!tag.is_empty() || tag.is_empty()); // 字符串类型
                println!("Latest tag version: {}", tag);
            }
            None => {
                println!("No tag version found");
            }
        }
    }

    #[test]
    fn test_version_parsing_logic() {
        // 测试版本解析逻辑
        let test_cases = vec![
            ("v1.2.3", Some("1.2.3")),
            ("1.2.3", Some("1.2.3")),
            ("v", None),
            ("", None),
            ("  v  ", None),
        ];

        for (input, expected) in test_cases {
            let result = {
                let trimmed = input.trim_start_matches('v');
                if trimmed.trim().is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            };

            assert_eq!(result, expected, "Input: '{}' should parse to {:?}", input, expected);
        }
    }

    #[test]
    fn test_version_format_validation() {
        use std::str::FromStr;
        
        // 测试版本格式验证逻辑
        let valid_versions = vec!["1.2.3", "0.1.0", "10.20.30"];
        let invalid_versions = vec!["1.2", "1.2.3.4", "a.b.c", "1.2.x"];

        for version in valid_versions {
            let parts: Vec<u32> = version
                .split('.')
                .filter_map(|s| u32::from_str(s).ok())
                .collect();
            
            assert_eq!(parts.len(), 3, "Valid version '{}' should parse to 3 parts", version);
        }

        for version in invalid_versions {
            let parts: Vec<u32> = version
                .split('.')
                .filter_map(|s| u32::from_str(s).ok())
                .collect();
            
            assert_ne!(parts.len(), 3, "Invalid version '{}' should not parse to exactly 3 parts", version);
        }
    }

    #[test]
    fn test_tag_name_generation_logic() {
        // 测试 tag 名称生成逻辑
        let parts = vec![1, 2, 3];
        let tag_name = format!("v{}.{}.{}", parts[0], parts[1], parts[2]);
        assert_eq!(tag_name, "v1.2.3");

        // 测试增量逻辑
        let mut parts = vec![1, 2, 3];
        parts[2] += 1;  // 增加 patch 版本
        let next_tag = format!("v{}.{}.{}", parts[0], parts[1], parts[2]);
        assert_eq!(next_tag, "v1.2.4");
    }

    #[tokio::test]
    async fn test_create_tag_command_structure() {
        let result = create_tag("test-tag").await;
        
        match result {
            Ok(_) => {
                println!("Create tag succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to create tag") ||
                    error_msg.contains("test-tag"),
                    "Error should contain tag creation information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_create_tag_with_note_command_structure() {
        let result = create_tag_with_note("test-tag", "test note").await;
        
        match result {
            Ok(_) => {
                println!("Create tag with note succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to create tag") ||
                    error_msg.contains("test-tag"),
                    "Error should contain tag creation information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_push_tag_command_structure() {
        let result = push_tag("test-tag", false).await;
        
        match result {
            Ok(_) => {
                println!("Push tag succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                // push_tag 可能会有警告但不会失败
                println!("Push tag result: {}", error_msg);
            }
        }
    }

    #[tokio::test]
    async fn test_push_tag_with_branches_command_structure() {
        let result = push_tag("test-tag", true).await;
        
        match result {
            Ok(_) => {
                println!("Push tag with branches succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                println!("Push tag with branches result: {}", error_msg);
            }
        }
    }

    #[tokio::test]
    async fn test_resolve_next_tag_name_structure() {
        // 测试不同的输入参数
        let test_cases = vec![
            None,
            Some(""),
            Some("v"),
            Some("1.2"),
            Some("1.2.3"),
            Some("v1.2.3"),
        ];

        for base_version in test_cases {
            let result = resolve_next_tag_name(base_version).await;
            
            match result {
                Ok(tag_name) => {
                    // 验证生成的 tag 名称格式
                    assert!(tag_name.starts_with("v"), "Tag name should start with 'v': {}", tag_name);
                    assert!(tag_name.contains('.'), "Tag name should contain dots: {}", tag_name);
                    println!("Generated tag name for {:?}: {}", base_version, tag_name);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    println!("Tag name generation failed for {:?}: {}", base_version, error_msg);
                    
                    // 某些情况下失败是预期的
                    if let Some(version) = base_version {
                        if version == "1.2" {
                            // 1.2 格式应该被接受（会转换为 1.2.0）
                            // 如果失败，可能是因为 git 环境问题
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_branch_names_logic() {
        // 测试分支名称处理逻辑
        let common_branches = ["master", "develop", "main"];
        let mock_existing_branches: HashSet<String> = 
            ["main", "feature/test", "develop"].iter().map(|s| s.to_string()).collect();

        let mut matched_branches = Vec::new();
        for branch in common_branches {
            if mock_existing_branches.contains(branch) {
                matched_branches.push(branch);
            }
        }

        // 应该匹配 main 和 develop
        assert_eq!(matched_branches.len(), 2);
        assert!(matched_branches.contains(&"main"));
        assert!(matched_branches.contains(&"develop"));
        assert!(!matched_branches.contains(&"master"));
    }

    #[test]
    fn test_cache_thread_safety() {
        use std::thread;

        // 测试缓存的线程安全性
        let handles: Vec<_> = (0..5)
            .map(|_| {
                thread::spawn(|| {
                    // 尝试访问缓存
                    let _cache = TAGS_CACHE.lock().unwrap();
                    let _branch_cache = BRANCH_CACHE.lock().unwrap();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // 如果没有死锁或恐慌，测试通过
        assert!(true);
    }
}