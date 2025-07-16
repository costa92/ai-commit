use std::collections::HashSet;
use std::process::Command;
use std::str::FromStr;

/// 获取最新的 tag 和备注
pub fn get_latest_tag() -> Option<(String, String)> {
    let tag_output = Command::new("bash")
        .arg("-c")
        .arg("git describe --tags `git rev-list --tags --max-count=1`")
        .output()
        .ok()?;

    if !tag_output.status.success() || tag_output.stdout.is_empty() {
        return None;
    }

    let tag = String::from_utf8(tag_output.stdout).ok()?.trim().to_string();

    let note_output = Command::new("git")
        .args(["tag", "-l", "-n1", &tag])
        .output()
        .ok()?;

    let note = if note_output.status.success() && !note_output.stdout.is_empty() {
        let note_line = String::from_utf8(note_output.stdout).ok()?.trim().to_string();
        if let Some(index) = note_line.find(char::is_whitespace) {
            note_line.split_at(index).1.trim().to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    Some((tag, note))
}

/// 获取最新的 tag（仅版本号）
pub fn get_latest_tag_version() -> Option<String> {
    get_latest_tag().map(|(tag, _)| tag)
}

fn get_all_tags() -> anyhow::Result<HashSet<String>> {
    let output = Command::new("git")
        .args(["tag", "-l"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to list tags: {}", e))?;
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse tag list: {}", e))?;
    Ok(stdout.lines().map(|s| s.trim().to_string()).collect())
}

fn resolve_next_tag_name(base_version: Option<&str>) -> anyhow::Result<String> {
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

    let all_tags = get_all_tags()?;
    let mut counter = 1;

    loop {
        let final_tag = format!("v{}.{}.{}", parts[0], parts[1], parts[2]);
        if !all_tags.contains(&final_tag) {
            return Ok(final_tag);
        }
        parts[2] += 1;
        counter += 1;
        if counter > 1000 {
            anyhow::bail!("Too many tag increments, please check your tags");
        }
    }
}

/// 创建新的 tag
pub fn create_tag(tag: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["tag", tag])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to create tag: {}", e))?;

    if !status.success() {
        anyhow::bail!("Failed to create tag '{}'", tag);
    }

    Ok(())
}

/// 创建新的带 note 的 tag
pub fn create_tag_with_note(tag: &str, note: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["tag", "-a", tag, "-m", note])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to create tag: {}", e))?;

    if !status.success() {
        anyhow::bail!("Failed to create tag '{}' with note", tag);
    }

    Ok(())
}

/// 推送 tag 到远程
pub fn push_tag(tag: &str, allow_push_branches: bool) -> anyhow::Result<()> {
    let mut push_args = vec!["push", "origin"];

    if allow_push_branches {
        let branches_output = Command::new("git")
            .args(["branch", "--list", "--format=%(refname:short)"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to list branches: {}", e))?;
        let branches_str = String::from_utf8(branches_output.stdout)
            .map_err(|e| anyhow::anyhow!("Failed to parse branch list: {}", e))?;
        let existing_branches: HashSet<&str> = branches_str.lines().collect();

        for b in ["master", "develop", "main"] {
            if existing_branches.contains(b) {
                push_args.push(b);
            }
        }
    }

    push_args.push(tag);

    let status = Command::new("git")
        .args(&push_args)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to push with args: {:?}: {}", push_args, e))?;

    if !status.success() {
        eprintln!("Warning: failed to push with args: {:?}", push_args);
    }

    Ok(())
}

/// 仅生成下一个 tag 名字，不创建 tag
pub fn get_next_tag_name(base_version: Option<&str>) -> anyhow::Result<String> {
    resolve_next_tag_name(base_version)
}