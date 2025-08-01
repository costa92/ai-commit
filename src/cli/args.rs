use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "ai-commit",
    version,
    about = "Generate commit messages using Ollama or Deepseek"
)]
pub struct Args {
    /// AI provider to use (ollama or deepseek)
    #[arg(short = 'P', long, default_value = "")] // 空字符串表示未指定
    pub provider: String,

    /// Model to use (default: mistral)
    #[arg(short, long, default_value = "")] // 空字符串表示未指定
    pub model: String,

    /// 不自动执行 git add .
    #[arg(short = 'n', long, default_value_t = false)]
    pub no_add: bool,

    /// commit 后是否自动 push
    #[arg(short = 'p', long, default_value_t = false)]
    pub push: bool,

    /// 创建新的 tag（可指定版本号，如 --new-tag v1.2.0）
    #[arg(short = 't', long = "new-tag", value_name = "VERSION", num_args = 0..=1, action = clap::ArgAction::Set)]
    pub new_tag: Option<String>,

    /// tag 备注内容（如 --tag-note "发布说明"），如不指定则用 AI 生成
    #[arg(long = "tag-note", value_name = "NOTE", default_value = "")]
    pub tag_note: String,

    /// 是否显示最新的 tag 信息
    #[arg(short = 's', long = "show-tag", default_value_t = false)]
    pub show_tag: bool,

    /// 推送 tag 时是否同时推送 master develop main 分支
    #[arg(short = 'b', long = "push-branches", default_value_t = false)]
    pub push_branches: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_args_default_values() {
        // 测试默认参数解析
        let args = Args::try_parse_from(&["ai-commit"]).unwrap();
        
        assert_eq!(args.provider, "");
        assert_eq!(args.model, "");
        assert_eq!(args.no_add, false);
        assert_eq!(args.push, false);
        assert_eq!(args.new_tag, None);
        assert_eq!(args.tag_note, "");
        assert_eq!(args.show_tag, false);
        assert_eq!(args.push_branches, false);
    }

    #[test]
    fn test_args_short_flags() {
        // 测试短参数
        let args = Args::try_parse_from(&[
            "ai-commit",
            "-P", "deepseek",
            "-m", "gpt-4",
            "-n",
            "-p",
            "-t", "v1.2.3",
            "-s",
            "-b",
        ]).unwrap();
        
        assert_eq!(args.provider, "deepseek");
        assert_eq!(args.model, "gpt-4");
        assert_eq!(args.no_add, true);
        assert_eq!(args.push, true);
        assert_eq!(args.new_tag, Some("v1.2.3".to_string()));
        assert_eq!(args.show_tag, true);
        assert_eq!(args.push_branches, true);
    }

    #[test]
    fn test_args_long_flags() {
        // 测试长参数
        let args = Args::try_parse_from(&[
            "ai-commit",
            "--provider", "ollama",
            "--model", "mistral",
            "--no-add",
            "--push",
            "--new-tag", "v2.0.0",
            "--tag-note", "Release version 2.0.0",
            "--show-tag",
            "--push-branches",
        ]).unwrap();
        
        assert_eq!(args.provider, "ollama");
        assert_eq!(args.model, "mistral");
        assert_eq!(args.no_add, true);
        assert_eq!(args.push, true);
        assert_eq!(args.new_tag, Some("v2.0.0".to_string()));
        assert_eq!(args.tag_note, "Release version 2.0.0");
        assert_eq!(args.show_tag, true);
        assert_eq!(args.push_branches, true);
    }

    #[test]
    fn test_args_mixed_flags() {
        // 测试混合使用短参数和长参数
        let args = Args::try_parse_from(&[
            "ai-commit",
            "-P", "siliconflow",
            "--model", "qwen-plus",
            "-p",
            "--new-tag",
            "--tag-note", "Mixed flags test",
        ]).unwrap();
        
        assert_eq!(args.provider, "siliconflow");
        assert_eq!(args.model, "qwen-plus");
        assert_eq!(args.push, true);
        assert_eq!(args.new_tag, Some("".to_string())); // --new-tag without value
        assert_eq!(args.tag_note, "Mixed flags test");
    }

    #[test]
    fn test_args_new_tag_variations() {
        // 测试 new-tag 参数的不同用法
        
        // 不带值的 --new-tag
        let args = Args::try_parse_from(&["ai-commit", "--new-tag"]).unwrap();
        assert_eq!(args.new_tag, Some("".to_string()));
        
        // 带值的 --new-tag
        let args = Args::try_parse_from(&["ai-commit", "--new-tag", "v1.0.0"]).unwrap();
        assert_eq!(args.new_tag, Some("v1.0.0".to_string()));
        
        // 短参数不带值
        let args = Args::try_parse_from(&["ai-commit", "-t"]).unwrap();
        assert_eq!(args.new_tag, Some("".to_string()));
        
        // 短参数带值
        let args = Args::try_parse_from(&["ai-commit", "-t", "v2.1.0"]).unwrap();
        assert_eq!(args.new_tag, Some("v2.1.0".to_string()));
    }

    #[test]
    fn test_args_tag_note_variations() {
        // 测试 tag-note 参数的不同用法
        
        // 空 tag note
        let args = Args::try_parse_from(&["ai-commit", "--tag-note", ""]).unwrap();
        assert_eq!(args.tag_note, "");
        
        // 简单 tag note
        let args = Args::try_parse_from(&["ai-commit", "--tag-note", "Simple note"]).unwrap();
        assert_eq!(args.tag_note, "Simple note");
        
        // 包含特殊字符的 tag note
        let args = Args::try_parse_from(&[
            "ai-commit", 
            "--tag-note", 
            "Version 1.0.0 - Bug fixes & improvements"
        ]).unwrap();
        assert_eq!(args.tag_note, "Version 1.0.0 - Bug fixes & improvements");
        
        // 中文 tag note
        let args = Args::try_parse_from(&[
            "ai-commit", 
            "--tag-note", 
            "发布版本 1.0.0"
        ]).unwrap();
        assert_eq!(args.tag_note, "发布版本 1.0.0");
    }

    #[test]
    fn test_args_provider_variations() {
        // 测试不同的 provider 参数
        let providers = vec!["ollama", "deepseek", "siliconflow", "custom"];
        
        for provider in providers {
            let args = Args::try_parse_from(&["ai-commit", "-P", provider]).unwrap();
            assert_eq!(args.provider, provider);
            
            let args = Args::try_parse_from(&["ai-commit", "--provider", provider]).unwrap();
            assert_eq!(args.provider, provider);
        }
    }

    #[test]
    fn test_args_model_variations() {
        // 测试不同的 model 参数
        let models = vec!["mistral", "gpt-4", "qwen-plus", "deepseek-chat", "custom-model"];
        
        for model in models {
            let args = Args::try_parse_from(&["ai-commit", "-m", model]).unwrap();
            assert_eq!(args.model, model);
            
            let args = Args::try_parse_from(&["ai-commit", "--model", model]).unwrap();
            assert_eq!(args.model, model);
        }
    }

    #[test]
    fn test_args_boolean_flags() {
        // 测试所有布尔标志
        
        // 单独测试每个布尔标志
        let args = Args::try_parse_from(&["ai-commit", "--no-add"]).unwrap();
        assert_eq!(args.no_add, true);
        
        let args = Args::try_parse_from(&["ai-commit", "--push"]).unwrap();
        assert_eq!(args.push, true);
        
        let args = Args::try_parse_from(&["ai-commit", "--show-tag"]).unwrap();
        assert_eq!(args.show_tag, true);
        
        let args = Args::try_parse_from(&["ai-commit", "--push-branches"]).unwrap();
        assert_eq!(args.push_branches, true);
        
        // 组合测试
        let args = Args::try_parse_from(&[
            "ai-commit", 
            "--no-add", 
            "--push", 
            "--show-tag", 
            "--push-branches"
        ]).unwrap();
        assert_eq!(args.no_add, true);
        assert_eq!(args.push, true);
        assert_eq!(args.show_tag, true);
        assert_eq!(args.push_branches, true);
    }

    #[test]
    fn test_args_help_and_version() {
        // 测试 help 和 version 标志（这些会导致程序退出，所以测试失败是预期的）
        
        let result = Args::try_parse_from(&["ai-commit", "--help"]);
        assert!(result.is_err());
        
        let result = Args::try_parse_from(&["ai-commit", "--version"]);
        assert!(result.is_err());
        
        let result = Args::try_parse_from(&["ai-commit", "-h"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_args_invalid_arguments() {
        // 测试无效参数
        
        let result = Args::try_parse_from(&["ai-commit", "--invalid-flag"]);
        assert!(result.is_err());
        
        let result = Args::try_parse_from(&["ai-commit", "-x"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_args_real_world_scenarios() {
        // 测试真实世界的使用场景
        
        // 场景1: 快速提交
        let args = Args::try_parse_from(&["ai-commit"]).unwrap();
        assert_eq!(args.provider, "");
        assert_eq!(args.push, false);
        
        // 场景2: 使用 Deepseek 并推送
        let args = Args::try_parse_from(&[
            "ai-commit", 
            "--provider", "deepseek", 
            "--model", "deepseek-chat",
            "--push"
        ]).unwrap();
        assert_eq!(args.provider, "deepseek");
        assert_eq!(args.model, "deepseek-chat");
        assert_eq!(args.push, true);
        
        // 场景3: 创建标签并推送
        let args = Args::try_parse_from(&[
            "ai-commit",
            "--new-tag", "v1.0.0",
            "--tag-note", "First stable release",
            "--push",
            "--push-branches"
        ]).unwrap();
        assert_eq!(args.new_tag, Some("v1.0.0".to_string()));
        assert_eq!(args.tag_note, "First stable release");
        assert_eq!(args.push, true);
        assert_eq!(args.push_branches, true);
        
        // 场景4: 查看标签信息
        let args = Args::try_parse_from(&["ai-commit", "--show-tag"]).unwrap();
        assert_eq!(args.show_tag, true);
        
        // 场景5: 跳过 git add
        let args = Args::try_parse_from(&["ai-commit", "--no-add"]).unwrap();
        assert_eq!(args.no_add, true);
    }

    #[test]
    fn test_args_empty_values() {
        // 测试空值处理
        let args = Args::try_parse_from(&[
            "ai-commit",
            "--provider", "",
            "--model", "",
            "--tag-note", "",
        ]).unwrap();
        
        assert_eq!(args.provider, "");
        assert_eq!(args.model, "");
        assert_eq!(args.tag_note, "");
    }

    #[test]
    fn test_args_debug_format() {
        // 测试 Debug trait
        let args = Args::try_parse_from(&["ai-commit"]).unwrap();
        let debug_str = format!("{:?}", args);
        
        assert!(debug_str.contains("Args"));
        assert!(debug_str.contains("provider"));
        assert!(debug_str.contains("model"));
    }

    #[test]
    fn test_args_complex_scenarios() {
        // 测试复杂场景组合
        
        // 复杂场景1: 全参数
        let args = Args::try_parse_from(&[
            "ai-commit",
            "--provider", "siliconflow",
            "--model", "qwen-turbo",
            "--no-add",
            "--push",
            "--new-tag", "v2.1.0-beta",
            "--tag-note", "Beta release with new features",
            "--push-branches",
        ]).unwrap();
        
        assert_eq!(args.provider, "siliconflow");
        assert_eq!(args.model, "qwen-turbo");
        assert_eq!(args.no_add, true);
        assert_eq!(args.push, true);
        assert_eq!(args.new_tag, Some("v2.1.0-beta".to_string()));
        assert_eq!(args.tag_note, "Beta release with new features");
        assert_eq!(args.push_branches, true);
        assert_eq!(args.show_tag, false); // 未设置的保持默认值
    }
}
// CLI参数修改
