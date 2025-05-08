use std::env;
use std::fs;

pub fn get_prompt(diff: &str) -> String {
    // 首先检查项目中是否存在 commit-prompt.txt
    let default_path = "commit-prompt.txt";
    let prompt_path = if std::path::Path::new(default_path).exists() {
        default_path.to_string()
    } else {
        // 如果项目中不存在,则检查环境变量配置
        env::var("AI_COMMIT_PROMPT_PATH").unwrap_or_else(|_| default_path.to_string())
    };

    // 打印
    println!("prompt_path: {}", prompt_path);

    // 检查最终路径文件是否存在
    if !std::path::Path::new(&prompt_path).exists() {
        // 如果文件不存在,创建默认的提示词模板
        let default_prompt = r#"请根据以下 git diff 内容生成一个符合 Conventional Commits 规范的提交信息。要求:
1. 使用中文
2. 标题不超过 50 个字符
3. 正文详细说明改动内容

git diff 内容如下:
{{git_diff}}

请按以下格式输出:
type(scope): 标题

正文"#;
        if let Err(e) = fs::write(&prompt_path, default_prompt) {
            eprintln!("无法创建提示词模板文件: {}", e);
            return default_prompt.replace("{{git_diff}}", diff);
        }
    }

    match fs::read_to_string(&prompt_path) {
        Ok(prompt_template) => prompt_template.replace("{{git_diff}}", diff),
        Err(e) => {
            eprintln!("无法读取提示词文件 {}: {}", prompt_path, e);
            let default_prompt = "请根据 git diff 生成提交信息:\n{{git_diff}}";
            default_prompt.replace("{{git_diff}}", diff)
        }
    }
}
