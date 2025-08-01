use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
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