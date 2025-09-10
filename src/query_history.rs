use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

/// 查询历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistoryEntry {
    /// 查询内容
    pub query: String,
    /// 查询时间
    pub timestamp: DateTime<Local>,
    /// 查询类型（可选）
    pub query_type: Option<String>,
    /// 查询结果数量（可选）
    pub result_count: Option<usize>,
    /// 是否成功
    pub success: bool,
}

/// 查询历史管理器
pub struct QueryHistory {
    /// 历史记录列表
    entries: VecDeque<QueryHistoryEntry>,
    /// 最大历史记录数
    max_entries: usize,
    /// 历史文件路径
    history_file: PathBuf,
}

impl QueryHistory {
    /// 创建新的查询历史管理器
    pub fn new(max_entries: usize) -> anyhow::Result<Self> {
        let history_file = Self::get_history_file_path()?;
        let mut history = Self {
            entries: VecDeque::new(),
            max_entries,
            history_file,
        };
        
        // 加载现有历史记录
        history.load_history()?;
        
        Ok(history)
    }

    /// 获取历史文件路径
    fn get_history_file_path() -> anyhow::Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        
        let config_dir = home_dir.join(".ai-commit");
        
        // 确保目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        Ok(config_dir.join("query_history.json"))
    }

    /// 加载历史记录
    fn load_history(&mut self) -> anyhow::Result<()> {
        if !self.history_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.history_file)?;
        if content.trim().is_empty() {
            return Ok(());
        }

        let entries: Vec<QueryHistoryEntry> = serde_json::from_str(&content)?;
        self.entries = VecDeque::from(entries);
        
        // 确保不超过最大数量
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        Ok(())
    }

    /// 保存历史记录到文件
    pub fn save_history(&self) -> anyhow::Result<()> {
        let entries: Vec<_> = self.entries.iter().cloned().collect();
        let json = serde_json::to_string_pretty(&entries)?;
        fs::write(&self.history_file, json)?;
        Ok(())
    }

    /// 添加新的查询记录
    pub fn add_entry(&mut self, query: String, query_type: Option<String>, result_count: Option<usize>, success: bool) -> anyhow::Result<()> {
        let entry = QueryHistoryEntry {
            query,
            timestamp: Local::now(),
            query_type,
            result_count,
            success,
        };

        // 添加到历史记录
        self.entries.push_back(entry);

        // 保持历史记录在限制内
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        // 自动保存
        self.save_history()?;

        Ok(())
    }

    /// 获取最近的历史记录
    pub fn get_recent(&self, count: usize) -> Vec<&QueryHistoryEntry> {
        self.entries
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    /// 搜索历史记录
    pub fn search(&self, pattern: &str) -> Vec<&QueryHistoryEntry> {
        let pattern_lower = pattern.to_lowercase();
        self.entries
            .iter()
            .filter(|entry| entry.query.to_lowercase().contains(&pattern_lower))
            .collect()
    }

    /// 清空历史记录
    pub fn clear(&mut self) -> anyhow::Result<()> {
        self.entries.clear();
        self.save_history()?;
        Ok(())
    }

    /// 获取历史记录统计信息
    pub fn get_stats(&self) -> QueryHistoryStats {
        let total_queries = self.entries.len();
        let successful_queries = self.entries.iter().filter(|e| e.success).count();
        let failed_queries = total_queries - successful_queries;
        
        let mut query_types = std::collections::HashMap::new();
        for entry in &self.entries {
            if let Some(ref query_type) = entry.query_type {
                *query_types.entry(query_type.clone()).or_insert(0) += 1;
            }
        }

        QueryHistoryStats {
            total_queries,
            successful_queries,
            failed_queries,
            query_types,
        }
    }

    /// 显示历史记录
    pub fn display_history(&self, count: Option<usize>) {
        let entries_to_show = count.unwrap_or(10);
        let recent = self.get_recent(entries_to_show);

        if recent.is_empty() {
            println!("No query history available.");
            return;
        }

        println!("📜 Query History (showing last {} entries):", entries_to_show);
        println!("{}", "─".repeat(60));

        for (i, entry) in recent.iter().enumerate() {
            let status_icon = if entry.success { "✅" } else { "❌" };
            let query_type = entry.query_type.as_deref().unwrap_or("query");
            let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
            
            println!("{} {} [{}] {}", 
                status_icon,
                timestamp,
                query_type,
                entry.query
            );
            
            if let Some(count) = entry.result_count {
                println!("   └─ Results: {}", count);
            }
            
            if i < recent.len() - 1 {
                println!();
            }
        }
        
        println!("{}", "─".repeat(60));
        println!("Total queries in history: {}", self.entries.len());
    }

    /// 交互式历史浏览
    pub fn interactive_browse(&self) -> anyhow::Result<Option<String>> {
        if self.entries.is_empty() {
            println!("No query history available.");
            return Ok(None);
        }

        let recent: Vec<_> = self.get_recent(20).into_iter().cloned().collect();
        
        println!("📜 Select a query from history:");
        println!("{}", "─".repeat(60));
        
        for (i, entry) in recent.iter().enumerate() {
            let status_icon = if entry.success { "✅" } else { "❌" };
            println!("{:2}. {} {}", i + 1, status_icon, entry.query);
        }
        
        println!("{}", "─".repeat(60));
        print!("Enter number (1-{}) or 'q' to quit: ", recent.len());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input == "q" || input == "quit" {
            return Ok(None);
        }
        
        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= recent.len() {
                return Ok(Some(recent[num - 1].query.clone()));
            }
        }
        
        println!("Invalid selection.");
        Ok(None)
    }
}

/// 查询历史统计信息
#[derive(Debug)]
pub struct QueryHistoryStats {
    pub total_queries: usize,
    pub successful_queries: usize,
    pub failed_queries: usize,
    pub query_types: std::collections::HashMap<String, usize>,
}

impl QueryHistoryStats {
    /// 显示统计信息
    pub fn display(&self) {
        println!("📊 Query History Statistics:");
        println!("{}", "─".repeat(40));
        println!("Total queries:      {}", self.total_queries);
        println!("Successful queries: {} ({:.1}%)", 
            self.successful_queries,
            if self.total_queries > 0 {
                (self.successful_queries as f64 / self.total_queries as f64) * 100.0
            } else {
                0.0
            }
        );
        println!("Failed queries:     {} ({:.1}%)",
            self.failed_queries,
            if self.total_queries > 0 {
                (self.failed_queries as f64 / self.total_queries as f64) * 100.0
            } else {
                0.0
            }
        );
        
        if !self.query_types.is_empty() {
            println!("\nQuery types:");
            for (query_type, count) in &self.query_types {
                println!("  {}: {}", query_type, count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_history_creation() {
        let history = QueryHistory::new(100);
        assert!(history.is_ok());
    }

    #[test]
    fn test_add_entry() {
        let mut history = QueryHistory::new(100).unwrap();
        let result = history.add_entry(
            "author:john".to_string(),
            Some("filter".to_string()),
            Some(10),
            true
        );
        assert!(result.is_ok());
        assert_eq!(history.entries.len(), 1);
    }

    #[test]
    fn test_max_entries_limit() {
        let mut history = QueryHistory::new(3).unwrap();
        
        for i in 0..5 {
            history.add_entry(
                format!("query {}", i),
                None,
                None,
                true
            ).unwrap();
        }
        
        // Should only keep last 3 entries
        assert_eq!(history.entries.len(), 3);
        assert_eq!(history.entries[0].query, "query 2");
        assert_eq!(history.entries[2].query, "query 4");
    }

    #[test]
    fn test_search_history() {
        let mut history = QueryHistory::new(100).unwrap();
        
        history.add_entry("author:john".to_string(), None, None, true).unwrap();
        history.add_entry("message:feat".to_string(), None, None, true).unwrap();
        history.add_entry("author:jane".to_string(), None, None, true).unwrap();
        
        let results = history.search("author");
        assert_eq!(results.len(), 2);
        
        let results = history.search("john");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].query, "author:john");
    }

    #[test]
    fn test_get_recent() {
        let mut history = QueryHistory::new(100).unwrap();
        
        for i in 0..5 {
            history.add_entry(
                format!("query {}", i),
                None,
                None,
                true
            ).unwrap();
        }
        
        let recent = history.get_recent(3);
        assert_eq!(recent.len(), 3);
        // Recent should be in reverse order (newest first)
        assert_eq!(recent[0].query, "query 4");
        assert_eq!(recent[1].query, "query 3");
        assert_eq!(recent[2].query, "query 2");
    }

    #[test]
    fn test_stats() {
        let mut history = QueryHistory::new(100).unwrap();
        
        history.add_entry("query1".to_string(), Some("filter".to_string()), None, true).unwrap();
        history.add_entry("query2".to_string(), Some("filter".to_string()), None, true).unwrap();
        history.add_entry("query3".to_string(), Some("search".to_string()), None, false).unwrap();
        
        let stats = history.get_stats();
        assert_eq!(stats.total_queries, 3);
        assert_eq!(stats.successful_queries, 2);
        assert_eq!(stats.failed_queries, 1);
        assert_eq!(stats.query_types.get("filter"), Some(&2));
        assert_eq!(stats.query_types.get("search"), Some(&1));
    }

    #[test]
    fn test_clear_history() {
        let mut history = QueryHistory::new(100).unwrap();
        
        history.add_entry("query1".to_string(), None, None, true).unwrap();
        history.add_entry("query2".to_string(), None, None, true).unwrap();
        
        assert_eq!(history.entries.len(), 2);
        
        history.clear().unwrap();
        assert_eq!(history.entries.len(), 0);
    }
}