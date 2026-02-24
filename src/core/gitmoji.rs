/// Gitmoji 支持模块
/// 将 Conventional Commits 类型映射到对应的 emoji

/// 获取 commit type 对应的 gitmoji
pub fn get_emoji(commit_type: &str) -> Option<&'static str> {
    match commit_type {
        "feat" => Some("\u{2728}"),     // ✨
        "fix" => Some("\u{1F41B}"),     // 🐛
        "docs" => Some("\u{1F4DD}"),    // 📝
        "style" => Some("\u{1F484}"),   // 💄
        "refactor" => Some("\u{267B}\u{FE0F}"), // ♻️
        "test" => Some("\u{2705}"),     // ✅
        "chore" => Some("\u{1F527}"),   // 🔧
        "perf" => Some("\u{26A1}"),     // ⚡
        "ci" => Some("\u{1F477}"),      // 👷
        "build" => Some("\u{1F4E6}"),   // 📦
        "revert" => Some("\u{23EA}"),   // ⏪
        _ => None,
    }
}

/// 为 commit message 添加 gitmoji 前缀
///
/// 输入: `feat(api): 添加用户认证功能`
/// 输出: `✨ feat(api): 添加用户认证功能`
pub fn add_emoji(message: &str) -> String {
    let commit_type = extract_commit_type(message);
    match commit_type.and_then(|t| get_emoji(t).map(|e| (t, e))) {
        Some((_type, emoji)) => format!("{} {}", emoji, message),
        None => message.to_string(),
    }
}

/// 从 conventional commit message 中提取 type
///
/// 支持格式:
/// - `feat(scope): message`
/// - `feat: message`
fn extract_commit_type(message: &str) -> Option<&str> {
    let trimmed = message.trim();
    // 查找第一个 '(' 或 ':'
    let type_end = trimmed.find(|c: char| c == '(' || c == ':')?;
    let commit_type = &trimmed[..type_end];

    // 验证 type 是合法的
    if commit_type.chars().all(|c| c.is_ascii_alphanumeric()) && !commit_type.is_empty() {
        Some(commit_type)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_emoji_known_types() {
        assert_eq!(get_emoji("feat"), Some("\u{2728}"));
        assert_eq!(get_emoji("fix"), Some("\u{1F41B}"));
        assert_eq!(get_emoji("docs"), Some("\u{1F4DD}"));
        assert_eq!(get_emoji("style"), Some("\u{1F484}"));
        assert_eq!(get_emoji("refactor"), Some("\u{267B}\u{FE0F}"));
        assert_eq!(get_emoji("test"), Some("\u{2705}"));
        assert_eq!(get_emoji("chore"), Some("\u{1F527}"));
        assert_eq!(get_emoji("perf"), Some("\u{26A1}"));
        assert_eq!(get_emoji("ci"), Some("\u{1F477}"));
        assert_eq!(get_emoji("build"), Some("\u{1F4E6}"));
        assert_eq!(get_emoji("revert"), Some("\u{23EA}"));
    }

    #[test]
    fn test_get_emoji_unknown_type() {
        assert_eq!(get_emoji("unknown"), None);
        assert_eq!(get_emoji(""), None);
    }

    #[test]
    fn test_extract_commit_type_with_scope() {
        assert_eq!(extract_commit_type("feat(api): 添加用户认证"), Some("feat"));
        assert_eq!(extract_commit_type("fix(ui): 修复按钮显示"), Some("fix"));
        assert_eq!(
            extract_commit_type("refactor(core): 重构数据处理"),
            Some("refactor")
        );
    }

    #[test]
    fn test_extract_commit_type_without_scope() {
        assert_eq!(extract_commit_type("feat: 添加新功能"), Some("feat"));
        assert_eq!(extract_commit_type("fix: 修复问题"), Some("fix"));
    }

    #[test]
    fn test_extract_commit_type_invalid() {
        assert_eq!(extract_commit_type(""), None);
        assert_eq!(extract_commit_type("no type here"), None);
    }

    #[test]
    fn test_add_emoji_with_scope() {
        let result = add_emoji("feat(api): 添加用户认证功能");
        assert!(result.starts_with('\u{2728}'));
        assert!(result.contains("feat(api): 添加用户认证功能"));
    }

    #[test]
    fn test_add_emoji_without_scope() {
        let result = add_emoji("fix: 修复登录问题");
        assert!(result.starts_with('\u{1F41B}'));
        assert!(result.contains("fix: 修复登录问题"));
    }

    #[test]
    fn test_add_emoji_unknown_type() {
        let msg = "unknown: 未知类型";
        let result = add_emoji(msg);
        assert_eq!(result, msg);
    }

    #[test]
    fn test_add_emoji_all_types() {
        let cases = vec![
            ("feat(x): msg", "\u{2728}"),
            ("fix(x): msg", "\u{1F41B}"),
            ("docs(x): msg", "\u{1F4DD}"),
            ("style(x): msg", "\u{1F484}"),
            ("refactor(x): msg", "\u{267B}\u{FE0F}"),
            ("test(x): msg", "\u{2705}"),
            ("chore(x): msg", "\u{1F527}"),
            ("perf(x): msg", "\u{26A1}"),
            ("ci(x): msg", "\u{1F477}"),
            ("build(x): msg", "\u{1F4E6}"),
            ("revert(x): msg", "\u{23EA}"),
        ];

        for (input, expected_emoji) in cases {
            let result = add_emoji(input);
            assert!(
                result.starts_with(expected_emoji),
                "Expected '{}' to start with emoji for '{}'",
                result,
                input
            );
        }
    }

    #[test]
    fn test_add_emoji_preserves_message() {
        let msg = "feat(auth): 实现 JWT 令牌认证\n\n详细的提交说明";
        let result = add_emoji(msg);
        assert!(result.contains("feat(auth): 实现 JWT 令牌认证\n\n详细的提交说明"));
    }

    #[test]
    fn test_add_emoji_idempotent_format() {
        // Verify format is "emoji space message"
        let result = add_emoji("feat: test");
        assert_eq!(result, "\u{2728} feat: test");
    }
}
