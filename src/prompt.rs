use std::env;
use std::fs;

pub fn get_prompt(diff: &str) -> String {
    let prompt_path =
        env::var("AI_COMMIT_PROMPT_PATH").unwrap_or_else(|_| "commit-prompt.txt".to_string());
    let prompt_template = fs::read_to_string(&prompt_path)
        .unwrap_or_else(|_| panic!("无法读取提示词文件: {}", prompt_path));
    prompt_template.replace("{{git_diff}}", diff)
}
