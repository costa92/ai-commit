use std::path::PathBuf;

/// prepare-commit-msg hook 脚本内容
const HOOK_SCRIPT: &str = r#"#!/bin/sh
# ai-commit prepare-commit-msg hook
# Installed by: ai-commit --hook-install
# This hook automatically generates commit messages using AI.
# To uninstall: ai-commit --hook-uninstall

COMMIT_MSG_FILE="$1"
COMMIT_SOURCE="$2"

# Only generate for normal commits (not merge, squash, amend, etc.)
if [ -n "$COMMIT_SOURCE" ]; then
    exit 0
fi

# Check if ai-commit is available
if ! command -v ai-commit >/dev/null 2>&1; then
    echo "Warning: ai-commit not found in PATH. Skipping AI commit message generation." >&2
    exit 0
fi

# Generate commit message using AI (--yes skips confirm, --no-add skips git add)
AI_MSG=$(ai-commit --yes --no-add 2>/dev/null)

if [ $? -eq 0 ] && [ -n "$AI_MSG" ]; then
    # Write AI-generated message, preserving any existing comments
    COMMENTS=$(grep '^#' "$COMMIT_MSG_FILE" 2>/dev/null || true)
    printf '%s\n' "$AI_MSG" > "$COMMIT_MSG_FILE"
    if [ -n "$COMMENTS" ]; then
        printf '\n%s\n' "$COMMENTS" >> "$COMMIT_MSG_FILE"
    fi
fi
"#;

/// ai-commit hook 标识符
const HOOK_MARKER: &str = "# Installed by: ai-commit --hook-install";

/// 获取 .git/hooks 目录路径
async fn get_hooks_dir() -> anyhow::Result<PathBuf> {
    let output = tokio::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Not a git repository");
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(git_dir).join("hooks"))
}

/// 获取 prepare-commit-msg hook 文件路径
async fn get_hook_path() -> anyhow::Result<PathBuf> {
    Ok(get_hooks_dir().await?.join("prepare-commit-msg"))
}

/// 检查 hook 是否已由 ai-commit 安装
fn is_ai_commit_hook(content: &str) -> bool {
    content.contains(HOOK_MARKER)
}

/// 安装 prepare-commit-msg hook
pub async fn install_hook() -> anyhow::Result<String> {
    let hook_path = get_hook_path().await?;
    let hooks_dir = hook_path.parent().unwrap();

    // 确保 hooks 目录存在
    if !hooks_dir.exists() {
        tokio::fs::create_dir_all(hooks_dir).await?;
    }

    // 检查已存在的 hook
    if hook_path.exists() {
        let existing = tokio::fs::read_to_string(&hook_path).await?;

        if is_ai_commit_hook(&existing) {
            // 已安装，覆盖更新
            tokio::fs::write(&hook_path, HOOK_SCRIPT).await?;
            return Ok(format!(
                "✓ Updated ai-commit hook at: {}",
                hook_path.display()
            ));
        }

        // 存在其他 hook，不覆盖
        anyhow::bail!(
            "A prepare-commit-msg hook already exists at: {}\n\
             To preserve your existing hook, please manually integrate ai-commit.\n\
             Or remove the existing hook first, then re-run --hook-install.",
            hook_path.display()
        );
    }

    // 写入 hook 脚本
    tokio::fs::write(&hook_path, HOOK_SCRIPT).await?;

    // 设置可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        tokio::fs::set_permissions(&hook_path, perms).await?;
    }

    Ok(format!(
        "✓ Installed ai-commit hook at: {}\n  \
         When you run `git commit`, AI will automatically generate the commit message.\n  \
         To uninstall: ai-commit --hook-uninstall",
        hook_path.display()
    ))
}

/// 卸载 prepare-commit-msg hook
pub async fn uninstall_hook() -> anyhow::Result<String> {
    let hook_path = get_hook_path().await?;

    if !hook_path.exists() {
        return Ok("No prepare-commit-msg hook found. Nothing to uninstall.".to_string());
    }

    let content = tokio::fs::read_to_string(&hook_path).await?;

    if !is_ai_commit_hook(&content) {
        anyhow::bail!(
            "The existing prepare-commit-msg hook was not installed by ai-commit.\n\
             Refusing to remove it. Please remove it manually if needed:\n  {}",
            hook_path.display()
        );
    }

    tokio::fs::remove_file(&hook_path).await?;

    Ok(format!(
        "✓ Uninstalled ai-commit hook from: {}",
        hook_path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_script_contains_marker() {
        assert!(HOOK_SCRIPT.contains(HOOK_MARKER));
    }

    #[test]
    fn test_hook_script_has_shebang() {
        assert!(HOOK_SCRIPT.starts_with("#!/bin/sh"));
    }

    #[test]
    fn test_hook_script_checks_commit_source() {
        assert!(HOOK_SCRIPT.contains("COMMIT_SOURCE"));
        assert!(HOOK_SCRIPT.contains("exit 0"));
    }

    #[test]
    fn test_hook_script_checks_ai_commit_availability() {
        assert!(HOOK_SCRIPT.contains("command -v ai-commit"));
    }

    #[test]
    fn test_hook_script_uses_no_add() {
        assert!(HOOK_SCRIPT.contains("--no-add"));
    }

    #[test]
    fn test_hook_script_uses_yes() {
        assert!(HOOK_SCRIPT.contains("--yes"));
    }

    #[test]
    fn test_is_ai_commit_hook_positive() {
        assert!(is_ai_commit_hook(HOOK_SCRIPT));
    }

    #[test]
    fn test_is_ai_commit_hook_negative() {
        assert!(!is_ai_commit_hook("#!/bin/sh\necho hello"));
    }

    #[test]
    fn test_is_ai_commit_hook_empty() {
        assert!(!is_ai_commit_hook(""));
    }

    #[tokio::test]
    async fn test_get_hooks_dir() {
        // Should succeed in a git repository
        let result = get_hooks_dir().await;
        assert!(result.is_ok());
        let dir = result.unwrap();
        assert!(dir.to_string_lossy().contains("hooks"));
    }

    #[tokio::test]
    async fn test_get_hook_path() {
        let result = get_hook_path().await;
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("prepare-commit-msg"));
    }
}
