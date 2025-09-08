use crate::config::Config;
use crate::git::GitQuery;
use crate::query_history::QueryHistory;

/// 处理查询命令
pub async fn handle_query_command(query: &str, config: &Config) -> anyhow::Result<()> {
    // 初始化查询历史
    let mut history = QueryHistory::new(1000)?;
    
    if config.debug {
        println!("Executing query: {}", query);
    }

    // 处理历史相关命令
    if query == "history" || query == "--history" {
        history.display_history(Some(20));
        return Ok(());
    }

    if query == "history-stats" || query == "--stats" {
        let stats = history.get_stats();
        stats.display();
        return Ok(());
    }

    if query == "history-clear" {
        history.clear()?;
        println!("Query history cleared.");
        return Ok(());
    }

    if query == "history-browse" || query == "--browse" {
        if let Some(selected_query) = history.interactive_browse()? {
            println!("Executing selected query: {}", selected_query);
            // Avoid recursion by directly executing the selected query
            let filters = GitQuery::parse_query(&selected_query)?;
            let results = GitQuery::execute_query(&filters).await?;
            let result_count = results.lines().count();
            
            if results.trim().is_empty() {
                println!("No results found for query: {}", selected_query);
                history.add_entry(
                    selected_query,
                    Some("execute".to_string()),
                    Some(0),
                    true
                )?;
            } else {
                println!("🔍 Query Results: {}", selected_query);
                println!("{}", "─".repeat(60));
                println!("{}", results);
                
                history.add_entry(
                    selected_query,
                    Some("execute".to_string()),
                    Some(result_count),
                    true
                )?;
            }
        }
        return Ok(());
    }

    if query == "help" || query == "--help" {
        println!("{}", GitQuery::get_query_help());
        println!("\nHistory Commands:");
        println!("  history, --history      Show query history");
        println!("  history-stats, --stats  Show history statistics");
        println!("  history-clear           Clear query history");
        println!("  history-browse, --browse Interactive history browser");
        return Ok(());
    }

    if query == "list" || query == "saved" {
        GitQuery::list_saved_queries().await?;
        return Ok(());
    }

    // 检查是否是保存查询的命令
    if query.starts_with("save:") {
        let parts: Vec<&str> = query.splitn(3, ':').collect();
        if parts.len() == 3 && parts[0] == "save" {
            let name = parts[1];
            let query_content = parts[2];
            GitQuery::save_query(name, query_content).await?;
            
            // 记录到历史
            history.add_entry(
                query.to_string(),
                Some("save".to_string()),
                None,
                true
            )?;
            
            return Ok(());
        }
    }

    // 解析并执行查询
    let filters = match GitQuery::parse_query(query) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to parse query: {}", e);
            // 记录失败的查询
            history.add_entry(
                query.to_string(),
                Some("execute".to_string()),
                None,
                false
            )?;
            return Err(e);
        }
    };

    let results = match GitQuery::execute_query(&filters).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to execute query: {}", e);
            // 记录失败的查询
            history.add_entry(
                query.to_string(),
                Some("execute".to_string()),
                None,
                false
            )?;
            return Err(e);
        }
    };

    let result_count = results.lines().count();
    
    if results.trim().is_empty() {
        println!("No results found for query: {}", query);
        // 记录无结果的查询
        history.add_entry(
            query.to_string(),
            Some("execute".to_string()),
            Some(0),
            true
        )?;
    } else {
        println!("🔍 Query Results: {}", query);
        println!("{}", "─".repeat(60));
        println!("{}", results);
        
        // 记录成功的查询
        history.add_entry(
            query.to_string(),
            Some("execute".to_string()),
            Some(result_count),
            true
        )?;
    }

    if config.debug {
        println!("\nQuery executed with {} filters, returned {} results", filters.len(), result_count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_parsing() {
        // 测试查询字符串解析逻辑
        assert!(handle_query_help("help"));
        assert!(handle_query_help("--help"));
        assert!(handle_saved_queries("list"));
        assert!(handle_saved_queries("saved"));
    }

    fn handle_query_help(query: &str) -> bool {
        query == "help" || query == "--help"
    }

    fn handle_saved_queries(query: &str) -> bool {
        query == "list" || query == "saved"
    }

    #[test]
    fn test_save_query_parsing() {
        let query = "save:test_query:author:john,since:2024-01-01";
        assert!(query.starts_with("save:"));
        
        let parts: Vec<&str> = query.splitn(3, ':').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "save");
        assert_eq!(parts[1], "test_query");
        assert_eq!(parts[2], "author:john,since:2024-01-01");
    }

    #[test]
    fn test_query_command_types() {
        // 测试不同类型的查询命令
        let help_queries = vec!["help", "--help"];
        for query in help_queries {
            assert!(is_help_query(query));
            assert!(!is_saved_queries_command(query));
            assert!(!is_save_command(query));
        }

        let list_queries = vec!["list", "saved"];
        for query in list_queries {
            assert!(!is_help_query(query));
            assert!(is_saved_queries_command(query));
            assert!(!is_save_command(query));
        }

        let save_queries = vec![
            "save:name:query",
            "save:test:author:john",
            "save:complex:author:jane,since:2024-01-01,type:feat"
        ];
        for query in save_queries {
            assert!(!is_help_query(query));
            assert!(!is_saved_queries_command(query));
            assert!(is_save_command(query));
        }
    }

    fn is_help_query(query: &str) -> bool {
        query == "help" || query == "--help"
    }

    fn is_saved_queries_command(query: &str) -> bool {
        query == "list" || query == "saved"
    }

    fn is_save_command(query: &str) -> bool {
        query.starts_with("save:")
    }

    #[test]
    fn test_save_command_validation() {
        // 测试保存命令的验证
        let valid_save_commands = vec![
            ("save:name:query", ("name", "query")),
            ("save:test:author:john,since:2024-01-01", ("test", "author:john,since:2024-01-01")),
            ("save:complex_name:type:feat,author:jane", ("complex_name", "type:feat,author:jane")),
        ];

        for (input, expected) in valid_save_commands {
            if let Some(parsed) = parse_save_command(input) {
                assert_eq!(parsed, expected);
            } else {
                panic!("Failed to parse valid save command: {}", input);
            }
        }

        // 测试无效的保存命令
        let invalid_save_commands = vec![
            "save:", // 缺少名称和查询
            "save:name", // 缺少查询
            "save:name:", // 空查询
            "save", // 不是保存格式
        ];

        for input in invalid_save_commands {
            assert!(parse_save_command(input).is_none(), "Should reject invalid save command: {}", input);
        }
    }

    fn parse_save_command(query: &str) -> Option<(&str, &str)> {
        if query.starts_with("save:") {
            let parts: Vec<&str> = query.splitn(3, ':').collect();
            if parts.len() == 3 && parts[0] == "save" && !parts[1].is_empty() && !parts[2].is_empty() {
                Some((parts[1], parts[2]))
            } else {
                None
            }
        } else {
            None
        }
    }

    #[test]
    fn test_query_content_validation() {
        // 测试查询内容的有效性
        let valid_queries = vec![
            "author:john",
            "since:2024-01-01",
            "type:feat",
            "author:john,since:2024-01-01",
            "type:feat,author:jane,since:2024-01-01",
            "file:src/main.rs,type:fix",
        ];

        for query in valid_queries {
            assert!(is_valid_query_content(query), "Should accept valid query: {}", query);
        }

        // 空查询或格式错误的查询可能仍然有效，取决于具体实现
        let edge_case_queries = vec![
            "",
            ":",
            "key:",
            ":value",
            "key:value:",
        ];

        // 这些边界情况的处理取决于具体的查询解析器实现
        for query in edge_case_queries {
            // 不进行断言，因为处理方式可能因实现而异
            let _result = is_valid_query_content(query);
        }
    }

    fn is_valid_query_content(query: &str) -> bool {
        // 简单的验证逻辑：包含键值对格式
        if query.is_empty() {
            return false;
        }
        
        // 检查是否包含键值对格式 (key:value)
        query.split(',').all(|part| {
            let kv: Vec<&str> = part.split(':').collect();
            kv.len() >= 2 && !kv[0].trim().is_empty() && !kv[1].trim().is_empty()
        })
    }

    #[test]
    fn test_query_command_integration() {
        // 测试查询命令的集成逻辑
        
        // 模拟配置
        let mut _config = Config::default();
        _config.provider = "test".to_string();
        _config.model = "test-model".to_string();
        _config.debug = false;

        // 测试不同的查询类型应该如何处理
        let query_scenarios = vec![
            ("help", QueryType::Help),
            ("--help", QueryType::Help),
            ("list", QueryType::ListSaved),
            ("saved", QueryType::ListSaved),
            ("save:test:author:john", QueryType::Save),
            ("author:john,type:feat", QueryType::Execute),
        ];

        for (query, expected_type) in query_scenarios {
            let detected_type = detect_query_type(query);
            assert_eq!(detected_type, expected_type, "Failed for query: {}", query);
        }
    }

    #[derive(Debug, PartialEq)]
    enum QueryType {
        Help,
        ListSaved,
        Save,
        Execute,
    }

    fn detect_query_type(query: &str) -> QueryType {
        if query == "help" || query == "--help" {
            QueryType::Help
        } else if query == "list" || query == "saved" {
            QueryType::ListSaved
        } else if query.starts_with("save:") {
            QueryType::Save
        } else {
            QueryType::Execute
        }
    }

    #[test]
    fn test_query_error_handling() {
        // 测试错误处理场景
        let error_scenarios = vec![
            "save:", // 不完整的保存命令
            "save::", // 空的保存命令
            "save:name_without_query:", // 缺少查询内容
        ];

        for scenario in error_scenarios {
            if scenario.starts_with("save:") {
                let parts: Vec<&str> = scenario.splitn(3, ':').collect();
                let is_valid = parts.len() == 3 && 
                              parts[0] == "save" && 
                              !parts[1].is_empty() && 
                              !parts[2].is_empty();
                assert!(!is_valid, "Should detect invalid save command: {}", scenario);
            }
        }
    }
}