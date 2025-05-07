use std::process::Command;

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

pub fn get_latest_tag() -> Option<(String, String)> {
    // 获取最新的 tag
    let tag_output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
        .ok()?;

    if tag_output.stdout.is_empty() {
        return None;
    }

    let tag = String::from_utf8_lossy(&tag_output.stdout)
        .trim()
        .to_string();

    // 获取 tag 的备注信息
    let note_output = Command::new("git")
        .args(["tag", "-l", "-n", &tag])
        .output()
        .ok()?;

    let note = String::from_utf8_lossy(&note_output.stdout)
        .trim()
        .to_string();

    Some((tag, note))
}
