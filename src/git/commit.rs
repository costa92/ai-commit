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

pub fn git_commit_allow_empty(message: &str) {
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", message])
        .status()
        .expect("Git commit (allow-empty) failed");
}
