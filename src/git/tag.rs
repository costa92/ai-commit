use std::process::Command;
use std::str::FromStr;

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
pub fn create_new_tag(base_version: Option<&str>) -> anyhow::Result<String> {
    // 1. 选择基础版本号
    let base_owned;
    let base: &str = if let Some(ver) = base_version {
        ver.trim_start_matches('v')
    } else if let Some(tag) = get_latest_tag_version() {
        base_owned = tag.trim_start_matches('v').to_string();
        base_owned.as_str()
    } else {
        "0.0.0"
    };

    // 2. 解析主次补丁
    let mut parts: Vec<u32> = base
        .split('.')
        .filter_map(|s| u32::from_str(s).ok())
        .collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid version format: {}", base);
    }

    // 3. 如果是指定大版本，补丁号归零，否则递增补丁号
    if base_version.is_some() {
        parts[2] = 0;
    } else {
        parts[2] += 1;
    }

    let mut final_tag = format!("v{}.{}.{}", parts[0], parts[1], parts[2]);
    let mut counter = 1;

    // 4. 检查 tag 是否已存在，已存在则递增补丁号
    while {
        let tag_exists = Command::new("git")
            .args(["tag", "-l", &final_tag])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to check tag existence: {}", e))?;
        !tag_exists.stdout.is_empty()
    } {
        parts[2] += 1;
        final_tag = format!("v{}.{}.{}", parts[0], parts[1], parts[2]);
        counter += 1;
        if counter > 1000 {
            anyhow::bail!("Too many tag increments, please check your tags");
        }
    }

    // 5. 创建新的 tag
    Command::new("git")
        .args(["tag", &final_tag])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to create tag: {}", e))?;

    Ok(final_tag)
}

/// 创建新的带 note 的 tag
pub fn create_new_tag_with_note(base_version: Option<&str>, note: &str) -> anyhow::Result<String> {
    // 1. 选择基础版本号
    let base_owned;
    let base: &str = if let Some(ver) = base_version {
        ver.trim_start_matches('v')
    } else if let Some(tag) = get_latest_tag_version() {
        base_owned = tag.trim_start_matches('v').to_string();
        base_owned.as_str()
    } else {
        "0.0.0"
    };

    // 2. 解析主次补丁
    let mut parts: Vec<u32> = base
        .split('.')
        .filter_map(|s| u32::from_str(s).ok())
        .collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid version format: {}", base);
    }

    // 3. 如果是指定大版本，补丁号归零，否则递增补丁号
    if base_version.is_some() {
        parts[2] = 0;
    } else {
        parts[2] += 1;
    }

    let mut final_tag = format!("v{}.{}.{}", parts[0], parts[1], parts[2]);
    let mut counter = 1;

    // 4. 检查 tag 是否已存在，已存在则递增补丁号
    while {
        let tag_exists = Command::new("git")
            .args(["tag", "-l", &final_tag])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to check tag existence: {}", e))?;
        !tag_exists.stdout.is_empty()
    } {
        parts[2] += 1;
        final_tag = format!("v{}.{}.{}", parts[0], parts[1], parts[2]);
        counter += 1;
        if counter > 1000 {
            anyhow::bail!("Too many tag increments, please check your tags");
        }
    }

    // 5. 创建新的带 note 的 tag
    Command::new("git")
        .args(["tag", "-a", &final_tag, "-m", note])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to create tag: {}", e))?;

    Ok(final_tag)
}

/// 推送 tag 到远程
pub fn push_tag(tag: &str, allow_push_branches: bool) -> anyhow::Result<()> {
    if allow_push_branches {
        // 检查本地 develop、master、main 分支是否存在
        let branches_output = Command::new("git")
            .args(["branch", "--list"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to list branches: {}", e))?;
        let branches_str = String::from_utf8(branches_output.stdout)
            .map_err(|e| anyhow::anyhow!("Failed to parse branch list: {}", e))?;
        let mut push_args = vec!["push", "origin"];
        for b in ["master", "develop", "main"] {
            if branches_str
                .lines()
                .any(|line| line.trim_end().ends_with(b))
            {
                push_args.push(b);
            }
        }
        push_args.push(tag);
        let status = Command::new("git")
            .args(&push_args)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to push tag and branches: {}", e))?;
        if !status.success() {
            eprintln!("Warning: failed to push tag and branches to origin");
        }
    } else {
        let status = Command::new("git")
            .args(["push", "origin", tag])
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to push tag: {}", e))?;
        if !status.success() {
            eprintln!("Warning: failed to push tag to origin");
        }
    }
    Ok(())
}
