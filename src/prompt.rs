use std::env;
use std::fs;

pub fn get_prompt(diff: &str) -> String {
    // 优先使用项目根目录下的 commit-prompt.txt
    let default_path = "commit-prompt.txt";
    let prompt_path = if std::path::Path::new(default_path).exists() {
        default_path.to_string()
    } else {
        // 如果项目中不存在,则检查环境变量配置
        env::var("AI_COMMIT_PROMPT_PATH").unwrap_or_else(|_| default_path.to_string())
    };

    // 打印
    // println!("prompt_path: {}", prompt_path);

    // 只要有 commit-prompt.txt 就用，否则用内置模板（编译时 include_str!）
    if std::path::Path::new(&prompt_path).exists() {
        match fs::read_to_string(&prompt_path) {
            Ok(prompt_template) => prompt_template.replace("{{git_diff}}", diff),
            Err(e) => {
                eprintln!("无法读取提示词文件 {}: {}", prompt_path, e);
                let default_prompt = include_str!("../commit-prompt.txt");
                default_prompt.replace("{{git_diff}}", diff)
            }
        }
    } else {
        // 内置默认模板，编译时读取 commit-prompt.txt
        let default_prompt = include_str!("../commit-prompt.txt");
        default_prompt.replace("{{git_diff}}", diff)
    }
}
