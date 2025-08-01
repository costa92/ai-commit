use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    SimplifiedChinese,
    TraditionalChinese,
    English,
}

impl Language {
    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "zh-cn" | "zh_cn" | "chs" => Language::SimplifiedChinese,
            "zh-tw" | "zh_tw" | "cht" => Language::TraditionalChinese,
            _ => Language::English,
        }
    }

    pub fn to_code(&self) -> &'static str {
        match self {
            Language::SimplifiedChinese => "zh-CN",
            Language::TraditionalChinese => "zh-TW",
            Language::English => "en-US",
        }
    }
}

pub struct I18n {
    strings: HashMap<String, HashMap<Language, String>>,
    current_language: Language,
}

impl I18n {
    pub fn new() -> Self {
        let mut i18n = I18n {
            strings: HashMap::new(),
            current_language: Language::SimplifiedChinese,
        };
        
        i18n.load_default_strings();
        i18n
    }

    pub fn set_language(&mut self, lang: Language) {
        self.current_language = lang;
    }

    pub fn get(&self, key: &str) -> String {
        self.strings
            .get(key)
            .and_then(|langs| langs.get(&self.current_language))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    fn load_default_strings(&mut self) {
        let mut messages = HashMap::new();
        
        // Git messages
        messages.insert("git_commit_failed".to_string(), {
            let mut m = HashMap::new();
            m.insert(Language::SimplifiedChinese, "Git提交失败".to_string());
            m.insert(Language::TraditionalChinese, "Git提交失敗".to_string());
            m.insert(Language::English, "Git commit failed".to_string());
            m
        });

        messages.insert("no_staged_changes".to_string(), {
            let mut m = HashMap::new();
            m.insert(Language::SimplifiedChinese, "没有暂存的变更".to_string());
            m.insert(Language::TraditionalChinese, "沒有暫存的變更".to_string());
            m.insert(Language::English, "No staged changes".to_string());
            m
        });

        messages.insert("commit_message_generated".to_string(), {
            let mut m = HashMap::new();
            m.insert(Language::SimplifiedChinese, "AI生成commit消息耗时".to_string());
            m.insert(Language::TraditionalChinese, "AI生成commit消息耗時".to_string());
            m.insert(Language::English, "AI generated commit message duration".to_string());
            m
        });

        self.strings.extend(messages);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_code() {
        let test_cases = vec![
            ("zh-cn", Language::SimplifiedChinese),
            ("ZH-CN", Language::SimplifiedChinese),
            ("zh_cn", Language::SimplifiedChinese),
            ("chs", Language::SimplifiedChinese),
            ("zh-tw", Language::TraditionalChinese),
            ("ZH-TW", Language::TraditionalChinese),
            ("zh_tw", Language::TraditionalChinese),
            ("cht", Language::TraditionalChinese),
            ("en", Language::English),
            ("en-us", Language::English),
            ("EN-US", Language::English),
            ("fr", Language::English), // 默认回退到英语
            ("", Language::English),    // 空字符串回退到英语
            ("unknown", Language::English), // 未知语言回退到英语
        ];

        for (code, expected) in test_cases {
            let result = Language::from_code(code);
            assert_eq!(result, expected, "Language code '{}' should map to {:?}", code, expected);
        }
    }

    #[test]
    fn test_language_to_code() {
        assert_eq!(Language::SimplifiedChinese.to_code(), "zh-CN");
        assert_eq!(Language::TraditionalChinese.to_code(), "zh-TW");
        assert_eq!(Language::English.to_code(), "en-US");
    }

    #[test]
    fn test_language_round_trip() {
        // 测试往返转换
        let languages = vec![
            Language::SimplifiedChinese,
            Language::TraditionalChinese,
            Language::English,
        ];

        for lang in languages {
            let code = lang.to_code();
            let converted = Language::from_code(code);
            assert_eq!(lang, converted, "Round trip conversion failed for {:?}", lang);
        }
    }

    #[test]
    fn test_i18n_new() {
        let i18n = I18n::new();
        
        // 验证默认语言
        assert_eq!(i18n.current_language, Language::SimplifiedChinese);
        
        // 验证默认消息已加载
        assert!(!i18n.strings.is_empty());
        
        // 验证包含预期的消息键
        assert!(i18n.strings.contains_key("git_commit_failed"));
        assert!(i18n.strings.contains_key("no_staged_changes"));
        assert!(i18n.strings.contains_key("commit_message_generated"));
    }

    #[test]
    fn test_i18n_set_language() {
        let mut i18n = I18n::new();
        
        // 初始语言应该是简体中文
        assert_eq!(i18n.current_language, Language::SimplifiedChinese);
        
        // 设置为繁体中文
        i18n.set_language(Language::TraditionalChinese);
        assert_eq!(i18n.current_language, Language::TraditionalChinese);
        
        // 设置为英语
        i18n.set_language(Language::English);
        assert_eq!(i18n.current_language, Language::English);
    }

    #[test]
    fn test_i18n_get_messages() {
        let mut i18n = I18n::new();
        
        // 测试简体中文
        i18n.set_language(Language::SimplifiedChinese);
        assert_eq!(i18n.get("git_commit_failed"), "Git提交失败");
        assert_eq!(i18n.get("no_staged_changes"), "没有暂存的变更");
        assert_eq!(i18n.get("commit_message_generated"), "AI生成commit消息耗时");
        
        // 测试繁体中文
        i18n.set_language(Language::TraditionalChinese);
        assert_eq!(i18n.get("git_commit_failed"), "Git提交失敗");
        assert_eq!(i18n.get("no_staged_changes"), "沒有暫存的變更");
        assert_eq!(i18n.get("commit_message_generated"), "AI生成commit消息耗時");
        
        // 测试英语
        i18n.set_language(Language::English);
        assert_eq!(i18n.get("git_commit_failed"), "Git commit failed");
        assert_eq!(i18n.get("no_staged_changes"), "No staged changes");
        assert_eq!(i18n.get("commit_message_generated"), "AI generated commit message duration");
    }

    #[test]
    fn test_i18n_get_unknown_key() {
        let i18n = I18n::new();
        
        // 未知键应该返回键本身
        let unknown_key = "unknown_message_key";
        assert_eq!(i18n.get(unknown_key), unknown_key);
        
        // 空键
        assert_eq!(i18n.get(""), "");
    }

    #[test]
    fn test_language_traits() {
        let lang1 = Language::SimplifiedChinese;
        let lang2 = Language::SimplifiedChinese;
        let lang3 = Language::TraditionalChinese;
        
        // 测试 PartialEq
        assert_eq!(lang1, lang2);
        assert_ne!(lang1, lang3);
        
        // 测试 Clone
        let lang1_cloned = lang1.clone();
        assert_eq!(lang1, lang1_cloned);
        
        // 测试 Debug
        let debug_str = format!("{:?}", lang1);
        assert!(debug_str.contains("SimplifiedChinese"));
    }

    #[test]
    fn test_language_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        set.insert(Language::SimplifiedChinese);
        set.insert(Language::TraditionalChinese);
        set.insert(Language::English);
        
        // 应该包含所有三种语言
        assert_eq!(set.len(), 3);
        assert!(set.contains(&Language::SimplifiedChinese));
        assert!(set.contains(&Language::TraditionalChinese));
        assert!(set.contains(&Language::English));
        
        // 添加重复项不应该增加大小
        set.insert(Language::SimplifiedChinese);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_i18n_message_coverage() {
        let i18n = I18n::new();
        
        // 验证所有消息在所有语言中都有定义
        let expected_keys = vec![
            "git_commit_failed",
            "no_staged_changes", 
            "commit_message_generated",
        ];
        
        let languages = vec![
            Language::SimplifiedChinese,
            Language::TraditionalChinese,
            Language::English,
        ];
        
        for key in expected_keys {
            for lang in &languages {
                let message_map = i18n.strings.get(key).expect(&format!("Key '{}' should exist", key));
                assert!(message_map.contains_key(lang), "Language {:?} should have message for key '{}'", lang, key);
                let message = message_map.get(lang).unwrap();
                assert!(!message.is_empty(), "Message for key '{}' in language {:?} should not be empty", key, lang);
            }
        }
    }

    #[test]
    fn test_i18n_concurrent_access() {
        use std::sync::Arc;
        use std::thread;
        
        let i18n = Arc::new(I18n::new());
        
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let i18n_clone = Arc::clone(&i18n);
                thread::spawn(move || {
                    // 并发访问消息
                    let message = i18n_clone.get("git_commit_failed");
                    assert!(!message.is_empty());
                    
                    // 返回线程ID用于验证
                    i
                })
            })
            .collect();
        
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_language_case_insensitive() {
        // 测试语言代码的大小写不敏感性
        let test_cases = vec![
            ("zh-cn", "ZH-CN"),
            ("zh-tw", "ZH-TW"),
            ("chs", "CHS"),
            ("cht", "CHT"),
        ];
        
        for (lower, upper) in test_cases {
            let lang_lower = Language::from_code(lower);
            let lang_upper = Language::from_code(upper);
            assert_eq!(lang_lower, lang_upper, "Case sensitivity test failed for '{}' vs '{}'", lower, upper);
        }
    }

    #[test]
    fn test_message_content_validity() {
        let i18n = I18n::new();
        
        // 验证消息内容的有效性
        let test_cases = vec![
            ("git_commit_failed", vec!["Git", "提交", "失败"]),
            ("no_staged_changes", vec!["暂存", "变更", "沒有"]),
            ("commit_message_generated", vec!["AI", "commit", "消息"]),
        ];
        
        for (key, expected_substrings) in test_cases {
            // 检查至少有一种语言包含预期的子字符串
            let message_map = i18n.strings.get(key).unwrap();
            let mut found_expected = false;
            
            for (_lang, message) in message_map {
                for substring in &expected_substrings {
                    if message.contains(substring) {
                        found_expected = true;
                        break;
                    }
                }
                if found_expected {
                    break;
                }
            }
            
            assert!(found_expected, "Key '{}' should contain at least one of {:?} in some language", key, expected_substrings);
        }
    }
}// 国际化修改
