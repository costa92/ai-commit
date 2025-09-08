use clap::Parser;

#[derive(Parser, Debug, Default)]
#[command(
    name = "ai-commit",
    version,
    about = "智能 Git 工具 - 使用 AI 生成提交消息，支持 Git Flow、历史查看和提交编辑",
    long_about = "ai-commit 是一个功能丰富的 Git 工具，集成 AI 生成提交消息、Git Flow 工作流、历史日志查看、提交编辑等功能。支持多种 AI 提供商和完整的 Git 工作流管理。支持自动解决推送冲突。",
)]
pub struct Args {
    /// AI provider to use (ollama, deepseek, siliconflow, or kimi)
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
    #[arg(short = 't', long = "new-tag", value_name = "VERSION", num_args = 0..=1, default_missing_value = "", action = clap::ArgAction::Set)]
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

    /// 创建新的 Git worktree（指定分支名，如 --worktree-create feature/new-ui）
    #[arg(long = "worktree-create", value_name = "BRANCH")]
    pub worktree_create: Option<String>,

    /// 切换到指定的 worktree（指定worktree名称或路径）
    #[arg(long = "worktree-switch", value_name = "NAME")]
    pub worktree_switch: Option<String>,

    /// 列出所有可用的 worktrees
    #[arg(long = "worktree-list", default_value_t = false)]
    pub worktree_list: bool,

    /// worktree list 详细模式 (等同于 git worktree list -v)
    #[arg(long = "worktree-verbose", short = 'v', default_value_t = false)]
    pub worktree_verbose: bool,

    /// worktree list 机器可读输出 (等同于 git worktree list --porcelain)
    #[arg(long = "worktree-porcelain", default_value_t = false)]
    pub worktree_porcelain: bool,

    /// worktree list 使用NUL字符终止记录 (等同于 git worktree list -z)
    #[arg(long = "worktree-z", short = 'z', default_value_t = false)]
    pub worktree_z: bool,

    /// worktree list 显示过期时间 (等同于 git worktree list --expire)
    #[arg(long = "worktree-expire", value_name = "TIME")]
    pub worktree_expire: Option<String>,

    /// 删除指定的 worktree（指定worktree名称或路径）
    #[arg(long = "worktree-remove", value_name = "NAME")]
    pub worktree_remove: Option<String>,

    /// 指定 worktree 创建的自定义路径
    #[arg(long = "worktree-path", value_name = "PATH")]
    pub worktree_path: Option<String>,

    /// 清空除当前外的所有其他 worktrees
    #[arg(long = "worktree-clear", default_value_t = false)]
    pub worktree_clear: bool,

    // =============== Tag 管理相关参数 ===============
    /// 列出所有 tags
    #[arg(long = "tag-list", default_value_t = false)]
    pub tag_list: bool,

    /// 删除指定的 tag（本地和远程）
    #[arg(long = "tag-delete", value_name = "TAG")]
    pub tag_delete: Option<String>,

    /// 显示指定 tag 的详细信息
    #[arg(long = "tag-info", value_name = "TAG")]
    pub tag_info: Option<String>,

    /// 比较两个 tags 之间的差异
    #[arg(long = "tag-compare", value_name = "TAG1..TAG2")]
    pub tag_compare: Option<String>,

    // =============== Git Flow 相关参数 ===============
    /// 开始新的 feature 分支
    #[arg(long = "flow-feature-start", value_name = "NAME")]
    pub flow_feature_start: Option<String>,

    /// 完成 feature 分支（合并到 develop）
    #[arg(long = "flow-feature-finish", value_name = "NAME")]
    pub flow_feature_finish: Option<String>,

    /// 开始新的 hotfix 分支
    #[arg(long = "flow-hotfix-start", value_name = "NAME")]
    pub flow_hotfix_start: Option<String>,

    /// 完成 hotfix 分支（合并到 main 和 develop）
    #[arg(long = "flow-hotfix-finish", value_name = "NAME")]
    pub flow_hotfix_finish: Option<String>,

    /// 开始新的 release 分支
    #[arg(long = "flow-release-start", value_name = "VERSION")]
    pub flow_release_start: Option<String>,

    /// 完成 release 分支（合并到 main 和 develop，创建 tag）
    #[arg(long = "flow-release-finish", value_name = "VERSION")]
    pub flow_release_finish: Option<String>,

    /// 初始化 git flow 仓库结构
    #[arg(long = "flow-init", default_value_t = false)]
    pub flow_init: bool,

    // =============== Git 初始化相关参数 ===============
    /// 初始化新的 Git 仓库
    #[arg(long = "git-init", default_value_t = false)]
    pub git_init: bool,

    // =============== 历史日志相关参数 ===============
    /// 显示提交历史（美化格式）
    #[arg(long = "history", default_value_t = false)]
    pub history: bool,

    /// 按作者过滤历史记录
    #[arg(long = "log-author", value_name = "AUTHOR")]
    pub log_author: Option<String>,

    /// 显示指定时间之后的历史记录
    #[arg(long = "log-since", value_name = "DATE")]
    pub log_since: Option<String>,

    /// 显示指定时间之前的历史记录
    #[arg(long = "log-until", value_name = "DATE")]
    pub log_until: Option<String>,

    /// 显示图形化分支历史
    #[arg(long = "log-graph", default_value_t = false)]
    pub log_graph: bool,

    /// 限制显示的提交数量
    #[arg(long = "log-limit", value_name = "N")]
    pub log_limit: Option<u32>,

    /// 按文件路径过滤历史记录
    #[arg(long = "log-file", value_name = "PATH")]
    pub log_file: Option<String>,

    /// 显示提交统计信息
    #[arg(long = "log-stats", default_value_t = false)]
    pub log_stats: bool,

    /// 显示贡献者统计
    #[arg(long = "log-contributors", default_value_t = false)]
    pub log_contributors: bool,

    /// 搜索提交消息中的关键词
    #[arg(long = "log-search", value_name = "TERM")]
    pub log_search: Option<String>,

    /// 显示所有分支的历史图
    #[arg(long = "log-branches", default_value_t = false)]
    pub log_branches: bool,

    /// 查询过滤器（支持复合条件）
    #[arg(long = "query", value_name = "QUERY")]
    pub query: Option<String>,

    /// 显示查询历史记录
    #[arg(long = "query-history", default_value_t = false)]
    pub query_history: bool,

    /// 显示查询历史统计信息
    #[arg(long = "query-stats", default_value_t = false)]
    pub query_stats: bool,

    /// 清空查询历史记录
    #[arg(long = "query-clear", default_value_t = false)]
    pub query_clear: bool,

    /// 交互式浏览查询历史
    #[arg(long = "query-browse", default_value_t = false)]
    pub query_browse: bool,

    /// 启动TUI界面查看查询历史
    #[arg(long = "query-tui", default_value_t = false)]
    pub query_tui: bool,

    /// 启动增强版TUI界面（GRV风格）
    #[arg(long = "query-tui-pro", default_value_t = false)]
    pub query_tui_pro: bool,

    /// 监控仓库变化
    #[arg(long = "watch", default_value_t = false)]
    pub watch: bool,

    /// 显示增强的差异查看
    #[arg(long = "diff-view", value_name = "COMMIT")]
    pub diff_view: Option<String>,

    /// 交互式历史浏览
    #[arg(long = "interactive-history", default_value_t = false)]
    pub interactive_history: bool,

    // =============== Commit 修改相关参数 ===============
    /// 修改最后一次提交
    #[arg(long = "amend", default_value_t = false)]
    pub amend: bool,

    /// 交互式修改指定的提交（使用 rebase）
    #[arg(long = "edit-commit", value_name = "COMMIT_HASH")]
    pub edit_commit: Option<String>,

    /// 交互式 rebase 修改多个提交
    #[arg(long = "rebase-edit", value_name = "BASE_COMMIT")]
    pub rebase_edit: Option<String>,

    /// 重写提交消息（不改变内容）
    #[arg(long = "reword-commit", value_name = "COMMIT_HASH")]
    pub reword_commit: Option<String>,

    /// 撤销最后一次提交（保留文件修改）
    #[arg(long = "undo-commit", default_value_t = false)]
    pub undo_commit: bool,

    // =============== Push 冲突解决相关参数 ===============
    /// 强制解决推送冲突（自动执行 pull + push）
    #[arg(long = "force-push", default_value_t = false)]
    pub force_push: bool,

    // =============== Commit 确认相关参数 ===============
    /// 跳过 AI 生成 commit message 的二次确认（默认需要确认）
    #[arg(long = "yes", short = 'y', default_value_t = false)]
    pub skip_confirm: bool,
}


#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_args_default_values() {
        // 测试默认参数解析
        let args = Args::try_parse_from(["ai-commit"]).unwrap();

        assert_eq!(args.provider, "");
        assert_eq!(args.model, "");
        assert!(!args.no_add);
        assert!(!args.push);
        assert_eq!(args.new_tag, None);
        assert_eq!(args.tag_note, "");
        assert!(!args.show_tag);
        assert!(!args.push_branches);
        assert_eq!(args.worktree_create, None);
        assert_eq!(args.worktree_switch, None);
        assert!(!args.worktree_list);
        assert!(!args.worktree_verbose);
        assert!(!args.worktree_porcelain);
        assert!(!args.worktree_z);
        assert_eq!(args.worktree_expire, None);
        assert_eq!(args.worktree_remove, None);
        assert_eq!(args.worktree_path, None);
        assert!(!args.worktree_clear);
        assert!(!args.force_push);
        assert!(!args.skip_confirm);
    }

    #[test]
    fn test_args_short_flags() {
        // 测试短参数
        let args = Args::try_parse_from([
            "ai-commit",
            "-P",
            "deepseek",
            "-m",
            "gpt-4",
            "-n",
            "-p",
            "-t",
            "v1.2.3",
            "-s",
            "-b",
        ])
        .unwrap();

        assert_eq!(args.provider, "deepseek");
        assert_eq!(args.model, "gpt-4");
        assert!(args.no_add);
        assert!(args.push);
        assert_eq!(args.new_tag, Some("v1.2.3".to_string()));
        assert!(args.show_tag);
        assert!(args.push_branches);
    }

    #[test]
    fn test_args_long_flags() {
        // 测试长参数
        let args = Args::try_parse_from([
            "ai-commit",
            "--provider",
            "ollama",
            "--model",
            "mistral",
            "--no-add",
            "--push",
            "--new-tag",
            "v2.0.0",
            "--tag-note",
            "Release version 2.0.0",
            "--show-tag",
            "--push-branches",
        ])
        .unwrap();

        assert_eq!(args.provider, "ollama");
        assert_eq!(args.model, "mistral");
        assert!(args.no_add);
        assert!(args.push);
        assert_eq!(args.new_tag, Some("v2.0.0".to_string()));
        assert_eq!(args.tag_note, "Release version 2.0.0");
        assert!(args.show_tag);
        assert!(args.push_branches);
    }

    #[test]
    fn test_args_mixed_flags() {
        // 测试混合使用短参数和长参数
        let args = Args::try_parse_from([
            "ai-commit",
            "-P",
            "siliconflow",
            "--model",
            "qwen-plus",
            "-p",
            "--new-tag",
            "--tag-note",
            "Mixed flags test",
        ])
        .unwrap();

        assert_eq!(args.provider, "siliconflow");
        assert_eq!(args.model, "qwen-plus");
        assert!(args.push);
        assert_eq!(args.new_tag, Some("".to_string())); // --new-tag without value
        assert_eq!(args.tag_note, "Mixed flags test");
    }

    #[test]
    fn test_args_new_tag_variations() {
        // 测试 new-tag 参数的不同用法

        // 不带值的 --new-tag
        let args = Args::try_parse_from(["ai-commit", "--new-tag"]).unwrap();
        assert_eq!(args.new_tag, Some("".to_string()));

        // 带值的 --new-tag
        let args = Args::try_parse_from(["ai-commit", "--new-tag", "v1.0.0"]).unwrap();
        assert_eq!(args.new_tag, Some("v1.0.0".to_string()));

        // 短参数不带值
        let args = Args::try_parse_from(["ai-commit", "-t"]).unwrap();
        assert_eq!(args.new_tag, Some("".to_string()));

        // 短参数带值
        let args = Args::try_parse_from(["ai-commit", "-t", "v2.1.0"]).unwrap();
        assert_eq!(args.new_tag, Some("v2.1.0".to_string()));
    }

    #[test]
    fn test_args_tag_note_variations() {
        // 测试 tag-note 参数的不同用法

        // 空 tag note
        let args = Args::try_parse_from(["ai-commit", "--tag-note", ""]).unwrap();
        assert_eq!(args.tag_note, "");

        // 简单 tag note
        let args = Args::try_parse_from(["ai-commit", "--tag-note", "Simple note"]).unwrap();
        assert_eq!(args.tag_note, "Simple note");

        // 包含特殊字符的 tag note
        let args = Args::try_parse_from([
            "ai-commit",
            "--tag-note",
            "Version 1.0.0 - Bug fixes & improvements",
        ])
        .unwrap();
        assert_eq!(args.tag_note, "Version 1.0.0 - Bug fixes & improvements");

        // 中文 tag note
        let args = Args::try_parse_from(["ai-commit", "--tag-note", "发布版本 1.0.0"]).unwrap();
        assert_eq!(args.tag_note, "发布版本 1.0.0");
    }

    #[test]
    fn test_args_provider_variations() {
        // 测试不同的 provider 参数
        let providers = vec!["ollama", "deepseek", "siliconflow", "custom"];

        for provider in providers {
            let args = Args::try_parse_from(["ai-commit", "-P", provider]).unwrap();
            assert_eq!(args.provider, provider);

            let args = Args::try_parse_from(["ai-commit", "--provider", provider]).unwrap();
            assert_eq!(args.provider, provider);
        }
    }

    #[test]
    fn test_args_model_variations() {
        // 测试不同的 model 参数
        let models = vec![
            "mistral",
            "gpt-4",
            "qwen-plus",
            "deepseek-chat",
            "custom-model",
        ];

        for model in models {
            let args = Args::try_parse_from(["ai-commit", "-m", model]).unwrap();
            assert_eq!(args.model, model);

            let args = Args::try_parse_from(["ai-commit", "--model", model]).unwrap();
            assert_eq!(args.model, model);
        }
    }

    #[test]
    fn test_args_boolean_flags() {
        // 测试所有布尔标志

        // 单独测试每个布尔标志
        let args = Args::try_parse_from(["ai-commit", "--no-add"]).unwrap();
        assert!(args.no_add);

        let args = Args::try_parse_from(["ai-commit", "--push"]).unwrap();
        assert!(args.push);

        let args = Args::try_parse_from(["ai-commit", "--show-tag"]).unwrap();
        assert!(args.show_tag);

        let args = Args::try_parse_from(["ai-commit", "--push-branches"]).unwrap();
        assert!(args.push_branches);

        // 组合测试
        let args = Args::try_parse_from([
            "ai-commit",
            "--no-add",
            "--push",
            "--show-tag",
            "--push-branches",
        ])
        .unwrap();
        assert!(args.no_add);
        assert!(args.push);
        assert!(args.show_tag);
        assert!(args.push_branches);
    }

    #[test]
    fn test_args_help_and_version() {
        // 测试 help 和 version 标志（这些会导致程序退出，所以测试失败是预期的）

        let result = Args::try_parse_from(["ai-commit", "--help"]);
        assert!(result.is_err());

        let result = Args::try_parse_from(["ai-commit", "--version"]);
        assert!(result.is_err());

        let result = Args::try_parse_from(["ai-commit", "-h"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_args_invalid_arguments() {
        // 测试无效参数

        let result = Args::try_parse_from(["ai-commit", "--invalid-flag"]);
        assert!(result.is_err());

        let result = Args::try_parse_from(["ai-commit", "-x"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_args_real_world_scenarios() {
        // 测试真实世界的使用场景

        // 场景1: 快速提交
        let args = Args::try_parse_from(["ai-commit"]).unwrap();
        assert_eq!(args.provider, "");
        assert!(!args.push);

        // 场景2: 使用 Deepseek 并推送
        let args = Args::try_parse_from([
            "ai-commit",
            "--provider",
            "deepseek",
            "--model",
            "deepseek-chat",
            "--push",
        ])
        .unwrap();
        assert_eq!(args.provider, "deepseek");
        assert_eq!(args.model, "deepseek-chat");
        assert!(args.push);

        // 场景3: 创建标签并推送
        let args = Args::try_parse_from([
            "ai-commit",
            "--new-tag",
            "v1.0.0",
            "--tag-note",
            "First stable release",
            "--push",
            "--push-branches",
        ])
        .unwrap();
        assert_eq!(args.new_tag, Some("v1.0.0".to_string()));
        assert_eq!(args.tag_note, "First stable release");
        assert!(args.push);
        assert!(args.push_branches);

        // 场景4: 查看标签信息
        let args = Args::try_parse_from(["ai-commit", "--show-tag"]).unwrap();
        assert!(args.show_tag);

        // 场景5: 跳过 git add
        let args = Args::try_parse_from(["ai-commit", "--no-add"]).unwrap();
        assert!(args.no_add);
    }

    #[test]
    fn test_args_empty_values() {
        // 测试空值处理
        let args = Args::try_parse_from([
            "ai-commit",
            "--provider",
            "",
            "--model",
            "",
            "--tag-note",
            "",
        ])
        .unwrap();

        assert_eq!(args.provider, "");
        assert_eq!(args.model, "");
        assert_eq!(args.tag_note, "");
    }

    #[test]
    fn test_args_debug_format() {
        // 测试 Debug trait
        let args = Args::try_parse_from(["ai-commit"]).unwrap();
        let debug_str = format!("{:?}", args);

        assert!(debug_str.contains("Args"));
        assert!(debug_str.contains("provider"));
        assert!(debug_str.contains("model"));
    }

    #[test]
    fn test_args_git_init() {
        // 测试 git init 参数
        let args = Args::try_parse_from(["ai-commit", "--git-init"]).unwrap();
        assert!(args.git_init);

        // 测试默认值
        let args = Args::try_parse_from(["ai-commit"]).unwrap();
        assert!(!args.git_init);
    }

    #[test]
    fn test_args_git_init_with_other_flags() {
        // 测试 git init 与其他参数组合
        let args = Args::try_parse_from([
            "ai-commit",
            "--git-init",
            "--provider", "ollama",
            "--model", "mistral",
        ]).unwrap();
        
        assert!(args.git_init);
        assert_eq!(args.provider, "ollama");
        assert_eq!(args.model, "mistral");
    }

    #[test]
    fn test_args_complex_scenarios() {
        // 测试复杂场景组合

        // 复杂场景1: 全参数
        let args = Args::try_parse_from([
            "ai-commit",
            "--provider",
            "siliconflow",
            "--model",
            "qwen-turbo",
            "--no-add",
            "--push",
            "--new-tag",
            "v2.1.0-beta",
            "--tag-note",
            "Beta release with new features",
            "--push-branches",
        ])
        .unwrap();

        assert_eq!(args.provider, "siliconflow");
        assert_eq!(args.model, "qwen-turbo");
        assert!(args.no_add);
        assert!(args.push);
        assert_eq!(args.new_tag, Some("v2.1.0-beta".to_string()));
        assert_eq!(args.tag_note, "Beta release with new features");
        assert!(args.push_branches);
        assert!(!args.show_tag); // 未设置的保持默认值
    }

    #[test]
    fn test_args_worktree_create() {
        // 测试 worktree-create 参数
        let args =
            Args::try_parse_from(["ai-commit", "--worktree-create", "feature/new-ui"]).unwrap();

        assert_eq!(args.worktree_create, Some("feature/new-ui".to_string()));
        assert_eq!(args.worktree_switch, None);
        assert!(!args.worktree_list);
        assert!(!args.worktree_verbose);
        assert!(!args.worktree_porcelain);
        assert!(!args.worktree_z);
        assert_eq!(args.worktree_expire, None);
        assert_eq!(args.worktree_remove, None);
        assert_eq!(args.worktree_path, None);
        assert!(!args.worktree_clear);
    }

    #[test]
    fn test_args_worktree_create_with_path() {
        // 测试 worktree-create 和 worktree-path 组合
        let args = Args::try_parse_from([
            "ai-commit",
            "--worktree-create",
            "feature/auth",
            "--worktree-path",
            "../worktrees/auth",
        ])
        .unwrap();

        assert_eq!(args.worktree_create, Some("feature/auth".to_string()));
        assert_eq!(args.worktree_path, Some("../worktrees/auth".to_string()));
    }

    #[test]
    fn test_args_worktree_switch() {
        // 测试 worktree-switch 参数
        let args = Args::try_parse_from(["ai-commit", "--worktree-switch", "feature/ui"]).unwrap();

        assert_eq!(args.worktree_switch, Some("feature/ui".to_string()));
        assert_eq!(args.worktree_create, None);
    }

    #[test]
    fn test_args_worktree_list() {
        // 测试 worktree-list 参数
        let args = Args::try_parse_from(["ai-commit", "--worktree-list"]).unwrap();

        assert!(args.worktree_list);
        assert_eq!(args.worktree_create, None);
        assert_eq!(args.worktree_switch, None);
    }

    #[test]
    fn test_args_worktree_remove() {
        // 测试 worktree-remove 参数
        let args = Args::try_parse_from(["ai-commit", "--worktree-remove", "feature/old-feature"])
            .unwrap();

        assert_eq!(
            args.worktree_remove,
            Some("feature/old-feature".to_string())
        );
        assert_eq!(args.worktree_create, None);
    }

    #[test]
    fn test_args_worktree_combined_with_commit() {
        // 测试 worktree 参数与提交参数组合
        let args = Args::try_parse_from([
            "ai-commit",
            "--worktree-switch",
            "feature/api",
            "--provider",
            "deepseek",
            "--push",
            "--new-tag",
            "v1.1.0",
        ])
        .unwrap();

        assert_eq!(args.worktree_switch, Some("feature/api".to_string()));
        assert_eq!(args.provider, "deepseek");
        assert!(args.push);
        assert_eq!(args.new_tag, Some("v1.1.0".to_string()));
    }

    #[test]
    fn test_args_worktree_all_options() {
        // 测试所有 worktree 相关选项的默认值
        let args = Args::try_parse_from(["ai-commit"]).unwrap();

        assert_eq!(args.worktree_create, None);
        assert_eq!(args.worktree_switch, None);
        assert!(!args.worktree_list);
        assert!(!args.worktree_verbose);
        assert!(!args.worktree_porcelain);
        assert!(!args.worktree_z);
        assert_eq!(args.worktree_expire, None);
        assert_eq!(args.worktree_remove, None);
        assert_eq!(args.worktree_path, None);
        assert!(!args.worktree_clear);
    }

    #[test]
    fn test_args_worktree_invalid_combinations() {
        // 虽然 clap 不会验证语义冲突，但确保解析正常
        let args = Args::try_parse_from([
            "ai-commit",
            "--worktree-create",
            "branch1",
            "--worktree-switch",
            "branch2",
        ])
        .unwrap();

        // 两个参数都应该被正确解析
        assert_eq!(args.worktree_create, Some("branch1".to_string()));
        assert_eq!(args.worktree_switch, Some("branch2".to_string()));
    }

    #[test]
    fn test_args_worktree_clear() {
        // 测试 worktree-clear 参数
        let args = Args::try_parse_from(["ai-commit", "--worktree-clear"]).unwrap();

        assert!(args.worktree_clear);
        assert_eq!(args.worktree_create, None);
        assert_eq!(args.worktree_remove, None);
    }

    #[test]
    fn test_args_worktree_clear_with_debug() {
        // 测试 worktree-clear 与其他参数组合
        let args = Args::try_parse_from(["ai-commit", "--worktree-clear", "--provider", "ollama"])
            .unwrap();

        assert!(args.worktree_clear);
        assert_eq!(args.provider, "ollama");
    }

    #[test]
    fn test_args_worktree_clear_default() {
        // 测试 worktree-clear 默认值
        let args = Args::try_parse_from(["ai-commit"]).unwrap();
        assert!(!args.worktree_clear);
    }

    #[test]
    fn test_args_worktree_list_options() {
        // 测试 worktree list 的各种选项
        let args = Args::try_parse_from([
            "ai-commit",
            "--worktree-list",
            "--worktree-verbose",
            "--worktree-porcelain",
            "--worktree-z",
            "--worktree-expire",
            "2weeks",
        ])
        .unwrap();

        assert!(args.worktree_list);
        assert!(args.worktree_verbose);
        assert!(args.worktree_porcelain);
        assert!(args.worktree_z);
        assert_eq!(args.worktree_expire, Some("2weeks".to_string()));
    }

    #[test]
    fn test_args_worktree_list_short_options() {
        // 测试 worktree list 的短选项
        let args = Args::try_parse_from(["ai-commit", "--worktree-list", "-v", "-z"]).unwrap();

        assert!(args.worktree_list);
        assert!(args.worktree_verbose);
        assert!(args.worktree_z);
        assert!(!args.worktree_porcelain);
    }

    #[test]
    fn test_args_worktree_list_expire_formats() {
        // 测试不同的过期时间格式
        let test_cases = vec!["1week", "2weeks", "1month", "2023-01-01", "yesterday"];

        for expire_time in test_cases {
            let args = Args::try_parse_from([
                "ai-commit",
                "--worktree-list",
                "--worktree-expire",
                expire_time,
            ])
            .unwrap();

            assert!(args.worktree_list);
            assert_eq!(args.worktree_expire, Some(expire_time.to_string()));
        }
    }

    #[test]
    fn test_args_worktree_list_combinations() {
        // 测试 worktree list 选项组合
        let args = Args::try_parse_from([
            "ai-commit",
            "--worktree-list",
            "--worktree-porcelain",
            "--worktree-z",
        ])
        .unwrap();

        assert!(args.worktree_list);
        assert!(args.worktree_porcelain);
        assert!(args.worktree_z);
        assert!(!args.worktree_verbose); // 不应该同时使用 verbose 和 porcelain
    }

    #[test]
    fn test_args_force_push() {
        // 测试 force-push 参数
        let args = Args::try_parse_from(["ai-commit", "--force-push"]).unwrap();
        assert!(args.force_push);

        // 测试默认值
        let args = Args::try_parse_from(["ai-commit"]).unwrap();
        assert!(!args.force_push);
    }

    #[test]
    fn test_args_force_push_with_push() {
        // 测试 force-push 与 push 参数组合
        let args = Args::try_parse_from([
            "ai-commit",
            "--force-push",
            "--push",
            "--provider", "ollama",
        ]).unwrap();
        
        assert!(args.force_push);
        assert!(args.push);
        assert_eq!(args.provider, "ollama");
    }

    #[test]
    fn test_args_force_push_with_tag() {
        // 测试 force-push 与 tag 创建参数组合
        let args = Args::try_parse_from([
            "ai-commit",
            "--force-push",
            "--push",
            "--new-tag", "v1.0.0",
        ]).unwrap();
        
        assert!(args.force_push);
        assert!(args.push);
        assert_eq!(args.new_tag, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_args_skip_confirm() {
        // 测试 skip_confirm 参数
        let args = Args::try_parse_from(["ai-commit", "--yes"]).unwrap();
        assert!(args.skip_confirm);

        let args = Args::try_parse_from(["ai-commit", "-y"]).unwrap();
        assert!(args.skip_confirm);

        // 测试默认值
        let args = Args::try_parse_from(["ai-commit"]).unwrap();
        assert!(!args.skip_confirm);
    }

    #[test]
    fn test_args_skip_confirm_with_other_flags() {
        // 测试 skip_confirm 与其他参数组合
        let args = Args::try_parse_from([
            "ai-commit",
            "--yes",
            "--push",
            "--provider", "ollama",
        ]).unwrap();
        
        assert!(args.skip_confirm);
        assert!(args.push);
        assert_eq!(args.provider, "ollama");
    }

    #[test]
    fn test_args_skip_confirm_with_force_push() {
        // 测试 skip_confirm 与 force_push 参数组合
        let args = Args::try_parse_from([
            "ai-commit",
            "-y",
            "--force-push",
            "--push",
        ]).unwrap();
        
        assert!(args.skip_confirm);
        assert!(args.force_push);
        assert!(args.push);
    }

    #[test]
    fn test_args_all_new_features() {
        // 测试所有新功能参数组合
        let args = Args::try_parse_from([
            "ai-commit",
            "--force-push",
            "--yes",
            "--push",
            "--new-tag", "v1.2.0",
            "--provider", "deepseek",
        ]).unwrap();
        
        assert!(args.force_push);
        assert!(args.skip_confirm);
        assert!(args.push);
        assert_eq!(args.new_tag, Some("v1.2.0".to_string()));
        assert_eq!(args.provider, "deepseek");
    }
}
// CLI参数修改
