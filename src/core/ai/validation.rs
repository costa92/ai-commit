use once_cell::sync::Lazy;
use regex::Regex;

/// 所有支持的 Conventional Commits 类型
pub const COMMIT_TYPES: &[&str] = &[
    "feat", "fix", "docs", "style", "refactor", "test", "chore", "perf", "ci", "build", "revert",
];

/// Conventional Commits 格式验证正则表达式
pub static COMMIT_FORMAT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\([^)]+\))?:\s+\S+.*$",
    )
    .unwrap()
});

/// 无效 AI 响应检测正则表达式（20+ 种英文描述模式）
///
/// 检测 AI 返回分析性文本而非 commit 消息的情况。
pub static INVALID_RESPONSE_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\{\{git_diff\}\}|输出格式|git diff:|these are|here's a|the changes|overall assessment|breakdown|suggestions|\*\*|good changes|clean|helpful|address|improve|significant changes|i don't have|represent good|contribute to|robust codebase|^the |^i |^1\.|\*)").unwrap()
});

/// 检查消息是否符合 Conventional Commits 格式
pub fn is_valid_commit_format(message: &str) -> bool {
    let first_line = message.lines().next().unwrap_or("");
    COMMIT_FORMAT_REGEX.is_match(first_line)
}

/// 检查 AI 响应是否为无效的描述性文本
pub fn is_invalid_response(message: &str) -> bool {
    message.trim().is_empty() || INVALID_RESPONSE_PATTERNS.is_match(message)
}

/// 验证提交消息格式并返回错误信息
pub fn validate_commit_message(message: &str) -> anyhow::Result<()> {
    let first_line = message.lines().next().unwrap_or("");

    if !COMMIT_FORMAT_REGEX.is_match(first_line) {
        anyhow::bail!(
            "提交消息格式不正确。期望格式：<type>(<scope>): <subject>\n实际：{}",
            first_line
        );
    }

    if first_line.chars().count() > 100 {
        anyhow::bail!(
            "提交消息过长（{}字符），应不超过100字符",
            first_line.chars().count()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_types_complete() {
        assert_eq!(COMMIT_TYPES.len(), 11);
        assert!(COMMIT_TYPES.contains(&"feat"));
        assert!(COMMIT_TYPES.contains(&"perf"));
        assert!(COMMIT_TYPES.contains(&"ci"));
        assert!(COMMIT_TYPES.contains(&"build"));
        assert!(COMMIT_TYPES.contains(&"revert"));
    }

    #[test]
    fn test_is_valid_commit_format() {
        assert!(is_valid_commit_format("feat(api): 添加用户认证"));
        assert!(is_valid_commit_format("fix: 修复登录问题"));
        assert!(is_valid_commit_format("docs(readme): 更新文档"));
        assert!(is_valid_commit_format("test: a"));

        assert!(is_valid_commit_format("perf(core): 优化查询性能"));
        assert!(is_valid_commit_format("ci: 更新CI配置"));
        assert!(is_valid_commit_format("build: 升级依赖版本"));
        assert!(is_valid_commit_format("revert: 撤销上次变更"));

        assert!(!is_valid_commit_format("invalid message"));
        assert!(!is_valid_commit_format("feat 缺少冒号"));
        assert!(!is_valid_commit_format(""));
        assert!(!is_valid_commit_format("test:"));
        assert!(!is_valid_commit_format("test: "));
    }

    #[test]
    fn test_validate_commit_message() {
        assert!(validate_commit_message("feat(api): 添加功能").is_ok());
        assert!(validate_commit_message("invalid").is_err());
        assert!(validate_commit_message("").is_err());

        let long_msg = format!("feat: {}", "很长的描述".repeat(30));
        assert!(validate_commit_message(&long_msg).is_err());
    }

    #[test]
    fn test_multiline_message_validates_first_line() {
        assert!(is_valid_commit_format("feat(api): 添加功能\n\n详细描述"));
        assert!(!is_valid_commit_format("描述\nfeat(api): 添加功能"));
    }

    #[test]
    fn test_is_invalid_response() {
        assert!(is_invalid_response(""));
        assert!(is_invalid_response("   "));
        assert!(is_invalid_response("These are good changes"));
        assert!(is_invalid_response("Here's a breakdown of the changes"));
        assert!(is_invalid_response("The changes include..."));
        assert!(is_invalid_response("Overall Assessment: good"));
        assert!(is_invalid_response("**bold text**"));
        assert!(is_invalid_response("{{git_diff}}"));

        assert!(!is_invalid_response("feat(api): 添加用户认证功能"));
        assert!(!is_invalid_response("fix(ui): 修复按钮显示问题"));
    }
}
