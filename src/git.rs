use std::process::Command;
use std::str::FromStr;

pub fn git_add_all() {
    Command::new("git")
        .args(["add", "."])
        .status()
        .expect("Git add failed");
}

pub fn git_commit(message: &str) {
    Command::new("git")
        .args(["commit", "-m", message])
        .status()
        .expect("Git commit failed");
}

pub fn git_push() {
    Command::new("git")
        .args(["push"])
        .status()
        .expect("Git push failed");
}

pub fn get_git_diff() -> String {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .expect("Failed to run git diff");

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// 获取最新的 tag 和备注
pub fn get_latest_tag() -> Option<(String, String)> {
    // 获取最新的 tag
    let tag_output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
        .ok()?;

    if tag_output.stdout.is_empty() {
        return None;
    }

    let tag = String::from_utf8(tag_output.stdout)
        .ok()?
        .trim()
        .to_string();

    // 获取 tag 的备注信息
    let note_output = Command::new("git")
        .args(["tag", "-l", "-n", &tag])
        .output()
        .ok()?;

    let note = String::from_utf8(note_output.stdout)
        .ok()?
        .trim()
        .to_string();

    Some((tag, note))
}

/// 获取最新的 tag（仅版本号）
pub fn get_latest_tag_version() -> Option<String> {
    get_latest_tag().map(|(tag, _)| tag)
}

/// 创建新的 tag
pub fn create_new_tag() -> anyhow::Result<String> {
    let latest_tag = get_latest_tag_version();

    // 如果没有 tag，从 v0.0.1 开始
    let new_tag = if let Some(tag) = latest_tag {
        // 移除 'v' 前缀
        let version = tag.trim_start_matches('v');

        // 解析版本号
        let mut parts: Vec<u32> = version
            .split('.')
            .filter_map(|s| u32::from_str(s).ok())
            .collect();

        if parts.len() != 3 {
            anyhow::bail!("Invalid version format: {}", version);
        }

        // 增加补丁版本号
        parts[2] += 1;

        // 重新组合版本号
        format!("v{}.{}.{}", parts[0], parts[1], parts[2])
    } else {
        "v0.0.1".to_string()
    };

    // 检查 tag 是否已存在，如果存在则继续递增版本号
    let mut final_tag = new_tag.clone();
    let mut counter = 1;

    while {
        let tag_exists = Command::new("git")
            .args(["tag", "-l", &final_tag])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to check tag existence: {}", e))?;
        !tag_exists.stdout.is_empty()
    } {
        // 如果 tag 存在，继续递增补丁版本号
        let version = final_tag.trim_start_matches('v');
        let mut parts: Vec<u32> = version
            .split('.')
            .filter_map(|s| u32::from_str(s).ok())
            .collect();

        if parts.len() != 3 {
            anyhow::bail!("Invalid version format: {}", version);
        }

        parts[2] += 1;
        final_tag = format!("v{}.{}.{}", parts[0], parts[1], parts[2]);

        // 防止无限循环
        counter += 1;
        if counter > 1000 {
            anyhow::bail!("Too many tag increments, please check your tags");
        }
    }

    // 创建新的 tag
    Command::new("git")
        .args(["tag", &final_tag])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to create tag: {}", e))?;

    Ok(final_tag)
}

/// 推送 tag 到远程
pub fn push_tag(tag: &str) -> anyhow::Result<()> {
    Command::new("git")
        .args(["push", "origin", tag])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to push tag: {}", e))?;
    Ok(())
}
