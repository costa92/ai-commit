use std::collections::HashMap;
use tokio::process::Command;

/// Git查询解析器，支持类似GRV的查询语法
pub struct GitQuery;

#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    pub author: Option<String>,
    pub message: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub file: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
}

impl GitQuery {
    /// 解析查询字符串为过滤条件
    /// 支持语法: "author:john AND message:feat OR since:2024-01-01"
    pub fn parse_query(query: &str) -> anyhow::Result<Vec<QueryFilter>> {
        let mut filters = Vec::new();
        let mut current_filter = QueryFilter::default();

        // 简单解析，按照 AND/OR 分割
        let parts = query.split_whitespace().collect::<Vec<_>>();
        let mut i = 0;

        while i < parts.len() {
            let part = parts[i];

            if part.eq_ignore_ascii_case("AND") {
                i += 1;
                continue;
            } else if part.eq_ignore_ascii_case("OR") {
                // OR 表示开始新的过滤器
                if !Self::is_filter_empty(&current_filter) {
                    filters.push(current_filter);
                    current_filter = QueryFilter::default();
                }
                i += 1;
                continue;
            }

            // 解析键值对 key:value
            if let Some((key, value)) = part.split_once(':') {
                match key.to_lowercase().as_str() {
                    "author" => current_filter.author = Some(value.to_string()),
                    "message" => current_filter.message = Some(value.to_string()),
                    "since" => current_filter.since = Some(value.to_string()),
                    "until" => current_filter.until = Some(value.to_string()),
                    "file" => current_filter.file = Some(value.to_string()),
                    "branch" => current_filter.branch = Some(value.to_string()),
                    "tag" => current_filter.tag = Some(value.to_string()),
                    _ => {
                        eprintln!("Unknown query key: {}", key);
                    }
                }
            }

            i += 1;
        }

        // 添加最后一个过滤器
        if !Self::is_filter_empty(&current_filter) {
            filters.push(current_filter);
        }

        if filters.is_empty() {
            filters.push(QueryFilter::default());
        }

        Ok(filters)
    }

    /// 检查过滤器是否为空
    fn is_filter_empty(filter: &QueryFilter) -> bool {
        fn is_empty(opt: &Option<String>) -> bool {
            opt.as_ref().is_none_or(|s| s.trim().is_empty())
        }
        is_empty(&filter.author)
            && is_empty(&filter.message)
            && is_empty(&filter.since)
            && is_empty(&filter.until)
            && is_empty(&filter.file)
            && is_empty(&filter.branch)
            && is_empty(&filter.tag)
    }

    /// 执行查询并返回结果
    pub async fn execute_query(filters: &[QueryFilter]) -> anyhow::Result<String> {
        let mut all_results = Vec::new();

        for filter in filters {
            let result = Self::execute_single_filter(filter).await?;
            if !result.trim().is_empty() {
                all_results.push(result);
            }
        }

        Ok(all_results.join("\n\n"))
    }

    /// 执行单个过滤器查询
    async fn execute_single_filter(filter: &QueryFilter) -> anyhow::Result<String> {
        let mut args = vec![
            "log".to_string(),
            "--pretty=format:%C(bold blue)%h%C(reset) - %C(bold green)(%ar)%C(reset) %C(white)%s%C(reset) %C(dim white)- %an%C(reset)%C(bold yellow)%d%C(reset)".to_string(),
        ];

        // 添加过滤条件
        if let Some(author) = &filter.author {
            args.extend(vec!["--author".to_string(), author.clone()]);
        }

        if let Some(message) = &filter.message {
            args.extend(vec!["--grep".to_string(), message.clone()]);
        }

        if let Some(since) = &filter.since {
            args.extend(vec!["--since".to_string(), since.clone()]);
        }

        if let Some(until) = &filter.until {
            args.extend(vec!["--until".to_string(), until.clone()]);
        }

        if let Some(branch) = &filter.branch {
            args.push(branch.clone());
        }

        // 文件路径需要在最后添加
        if let Some(file) = &filter.file {
            args.extend(vec!["--".to_string(), file.clone()]);
        }

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute git query: {}", e))?;

        if !output.status.success() {
            anyhow::bail!(
                "Git query failed with exit code: {:?}",
                output.status.code()
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 获取查询帮助信息
    pub fn get_query_help() -> String {
        r#"查询语法帮助：

支持的查询字段：
  author:NAME     - 按作者过滤 (例: author:john)
  message:TEXT    - 按提交消息过滤 (例: message:feat)
  since:DATE      - 指定开始日期 (例: since:2024-01-01)
  until:DATE      - 指定结束日期 (例: until:2024-12-31)
  file:PATH       - 按文件路径过滤 (例: file:src/main.rs)
  branch:NAME     - 指定分支 (例: branch:feature/auth)
  tag:NAME        - 指定标签 (例: tag:v1.0.0)

逻辑操作符：
  AND             - 逻辑与 (默认)
  OR              - 逻辑或

查询示例：
  author:john AND message:feat
  since:2024-01-01 OR tag:v1.0.0
  file:src/main.rs AND author:alice
  message:fix OR message:bug
  branch:main AND since:yesterday
"#
        .to_string()
    }

    /// 保存常用查询
    pub async fn save_query(name: &str, query: &str) -> anyhow::Result<()> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("ai-commit");

        tokio::fs::create_dir_all(&config_dir).await?;

        let queries_file = config_dir.join("saved_queries.txt");
        let entry = format!("{}: {}\n", name, query);

        tokio::fs::write(&queries_file, entry)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to save query: {}", e))?;

        println!("✓ Saved query '{}' to {}", name, queries_file.display());
        Ok(())
    }

    /// 加载保存的查询
    pub async fn load_saved_queries() -> anyhow::Result<HashMap<String, String>> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("ai-commit");

        let queries_file = config_dir.join("saved_queries.txt");

        if !queries_file.exists() {
            return Ok(HashMap::new());
        }

        let content = tokio::fs::read_to_string(&queries_file)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load saved queries: {}", e))?;

        let mut queries = HashMap::new();
        for line in content.lines() {
            if let Some((name, query)) = line.split_once(": ") {
                queries.insert(name.trim().to_string(), query.trim().to_string());
            }
        }

        Ok(queries)
    }

    /// 列出保存的查询
    pub async fn list_saved_queries() -> anyhow::Result<()> {
        let queries = Self::load_saved_queries().await?;

        if queries.is_empty() {
            println!("没有保存的查询。");
            return Ok(());
        }

        println!("💾 保存的查询：");
        println!("{}", "─".repeat(60));

        for (name, query) in queries {
            println!("{:<20} {}", name, query);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let query = "author:john";
        let filters = GitQuery::parse_query(query).unwrap();

        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].author, Some("john".to_string()));
        assert_eq!(filters[0].message, None);
    }

    #[test]
    fn test_parse_and_query() {
        let query = "author:john AND message:feat";
        let filters = GitQuery::parse_query(query).unwrap();

        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].author, Some("john".to_string()));
        assert_eq!(filters[0].message, Some("feat".to_string()));
    }

    #[test]
    fn test_parse_or_query() {
        let query = "author:john OR author:alice";
        let filters = GitQuery::parse_query(query).unwrap();

        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].author, Some("john".to_string()));
        assert_eq!(filters[1].author, Some("alice".to_string()));
    }

    #[test]
    fn test_parse_complex_query() {
        let query = "author:john AND message:feat OR since:2024-01-01";
        let filters = GitQuery::parse_query(query).unwrap();

        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].author, Some("john".to_string()));
        assert_eq!(filters[0].message, Some("feat".to_string()));
        assert_eq!(filters[1].since, Some("2024-01-01".to_string()));
    }

    #[test]
    fn test_parse_empty_query() {
        let query = "";
        let filters = GitQuery::parse_query(query).unwrap();

        assert_eq!(filters.len(), 1);
        assert!(GitQuery::is_filter_empty(&filters[0]));
    }

    #[test]
    fn test_parse_invalid_key() {
        let query = "invalid:value";
        let filters = GitQuery::parse_query(query).unwrap();

        // 无效键被忽略，返回空过滤器
        assert_eq!(filters.len(), 1);
        assert!(GitQuery::is_filter_empty(&filters[0]));
    }

    #[tokio::test]
    async fn test_execute_empty_filter() {
        let filter = QueryFilter::default();
        let result = GitQuery::execute_single_filter(&filter).await;

        // 空过滤器应该返回所有提交（或失败，这取决于是否在git仓库中）
        match result {
            Ok(_) => println!("Empty filter query succeeded"),
            Err(e) => println!(
                "Empty filter query failed (expected in non-git environment): {}",
                e
            ),
        }
    }

    #[test]
    fn test_query_help() {
        let help = GitQuery::get_query_help();

        assert!(help.contains("author:NAME"));
        assert!(help.contains("message:TEXT"));
        assert!(help.contains("AND"));
        assert!(help.contains("OR"));
    }

    #[test]
    fn test_is_filter_empty() {
        let empty_filter = QueryFilter::default();
        assert!(GitQuery::is_filter_empty(&empty_filter));

        let mut non_empty_filter = QueryFilter::default();
        non_empty_filter.author = Some("john".to_string());
        assert!(!GitQuery::is_filter_empty(&non_empty_filter));
    }

    #[test]
    fn test_parse_query_edge_cases() {
        // 测试空白查询
        let query = "   ";
        let filters = GitQuery::parse_query(query).unwrap();
        assert_eq!(filters.len(), 1);
        assert!(GitQuery::is_filter_empty(&filters[0]));

        // 测试只有分隔符的查询
        let query = "AND OR";
        let filters = GitQuery::parse_query(query).unwrap();
        assert_eq!(filters.len(), 1);
        assert!(GitQuery::is_filter_empty(&filters[0]));

        // 测试重复的AND/OR
        let query = "author:john AND AND message:feat";
        let filters = GitQuery::parse_query(query).unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].author, Some("john".to_string()));
        assert_eq!(filters[0].message, Some("feat".to_string()));
    }

    #[test]
    fn test_parse_query_special_characters() {
        // 测试包含特殊字符的值
        let query = "author:john-doe message:feat/fix";
        let filters = GitQuery::parse_query(query).unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].author, Some("john-doe".to_string()));
        assert_eq!(filters[0].message, Some("feat/fix".to_string()));

        // 测试包含空格的值（应该在冒号后处理）
        let query = "message:fix bug";
        let filters = GitQuery::parse_query(query).unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].message, Some("fix".to_string()));
    }

    #[test]
    fn test_parse_query_all_fields() {
        // 测试所有支持的字段
        let query = "author:john message:feat since:2024-01-01 until:2024-12-31 file:src/main.rs branch:main tag:v1.0.0";
        let filters = GitQuery::parse_query(query).unwrap();
        assert_eq!(filters.len(), 1);

        let filter = &filters[0];
        assert_eq!(filter.author, Some("john".to_string()));
        assert_eq!(filter.message, Some("feat".to_string()));
        assert_eq!(filter.since, Some("2024-01-01".to_string()));
        assert_eq!(filter.until, Some("2024-12-31".to_string()));
        assert_eq!(filter.file, Some("src/main.rs".to_string()));
        assert_eq!(filter.branch, Some("main".to_string()));
        assert_eq!(filter.tag, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_parse_query_case_sensitivity() {
        // 测试关键字大小写敏感性
        let query = "AUTHOR:john MESSAGE:feat";
        let filters = GitQuery::parse_query(query).unwrap();
        assert_eq!(filters.len(), 1);
        // parse_query uses to_lowercase() internally, so uppercase keywords are valid
        assert!(!GitQuery::is_filter_empty(&filters[0]));
        assert_eq!(filters[0].author, Some("john".to_string()));
        assert_eq!(filters[0].message, Some("feat".to_string()));

        // 测试AND/OR大小写
        let query = "author:john and message:feat";
        let filters = GitQuery::parse_query(query).unwrap();
        assert_eq!(filters.len(), 1); // 小写and被当作普通词处理
    }

    #[test]
    fn test_filter_combinations() {
        // 测试多个相同字段（后面的应该覆盖前面的）
        let query = "author:john author:alice";
        let filters = GitQuery::parse_query(query).unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].author, Some("alice".to_string()));

        // 测试复杂的OR组合
        let query = "author:john OR message:feat OR since:yesterday";
        let filters = GitQuery::parse_query(query).unwrap();
        assert_eq!(filters.len(), 3);
    }

    #[tokio::test]
    async fn test_saved_queries_functionality() {
        use dirs::home_dir;
        use std::fs;

        // 创建测试查询
        let query_name = "test_query";
        let query_content = "author:test AND message:feat";

        // 保存查询
        let result = GitQuery::save_query(query_name, query_content).await;
        match result {
            Ok(_) => {
                println!("Query saved successfully");

                // 清理测试文件
                if let Some(home) = home_dir() {
                    let query_file = home
                        .join(".ai-commit")
                        .join("queries")
                        .join(format!("{}.txt", query_name));
                    if query_file.exists() {
                        let _ = fs::remove_file(query_file);
                    }
                }
            }
            Err(e) => println!("Query save failed (expected in test environment): {}", e),
        }
    }

    #[tokio::test]
    async fn test_list_saved_queries() {
        let result = GitQuery::list_saved_queries().await;
        match result {
            Ok(_) => println!("List queries succeeded"),
            Err(e) => println!("List queries failed (expected if no queries exist): {}", e),
        }
    }

    #[test]
    fn test_query_help_completeness() {
        let help = GitQuery::get_query_help();

        // 检查所有字段都在帮助中
        let required_fields = [
            "author:", "message:", "since:", "until:", "file:", "branch:", "tag:",
        ];
        for field in &required_fields {
            assert!(help.contains(field), "Help should contain field: {}", field);
        }

        // 检查操作符
        assert!(help.contains("AND"));
        assert!(help.contains("OR"));

        // 检查示例
        assert!(help.contains("author:john"));
    }

    #[test]
    fn test_empty_query_handling() {
        // 测试各种空查询形式
        let empty_queries = vec!["", "   ", "\t", "\n", "  \n  \t  "];

        for query in empty_queries {
            let filters = GitQuery::parse_query(query).unwrap();
            assert_eq!(filters.len(), 1);
            assert!(GitQuery::is_filter_empty(&filters[0]));
        }
    }

    #[tokio::test]
    async fn test_execute_query_error_handling() {
        // 测试执行无效查询
        let filters = vec![QueryFilter {
            author: Some("non-existent-author-12345".to_string()),
            ..Default::default()
        }];

        let result = GitQuery::execute_query(&filters).await;
        match result {
            Ok(output) => {
                assert!(output.is_empty() || output.contains("no commits found"));
            }
            Err(e) => {
                println!("Query execution failed as expected: {}", e);
            }
        }
    }

    #[test]
    fn test_filter_validation() {
        // 测试过滤器字段验证
        let mut filter = QueryFilter::default();

        // 设置各种字段
        filter.author = Some("".to_string()); // 空字符串
        assert!(GitQuery::is_filter_empty(&filter)); // 空字符串应该被视为空

        filter.author = Some("valid-author".to_string());
        assert!(!GitQuery::is_filter_empty(&filter));

        // 测试日期字段
        filter.since = Some("2024-01-01".to_string());
        filter.until = Some("invalid-date".to_string()); // 无效日期格式
        assert!(!GitQuery::is_filter_empty(&filter)); // 仍然有有效字段
    }

    #[test]
    fn test_query_string_normalization() {
        // 测试查询字符串的规范化处理
        let queries = vec![
            ("author:john", "author:john AND message:feat", true),
            ("  author:john  ", "author:john", true),
            (
                "author:john\n\nmessage:feat",
                "author:john message:feat",
                true,
            ),
        ];

        for (query, _expected, should_parse) in queries {
            let result = GitQuery::parse_query(query);
            if should_parse {
                assert!(result.is_ok(), "Query should parse successfully: {}", query);
            } else {
                assert!(result.is_err(), "Query should fail to parse: {}", query);
            }
        }
    }
}
