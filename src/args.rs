use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "ai-commit",
    version,
    about = "Generate commit messages using Ollama or Deepseek"
)]
pub struct Args {
    /// AI provider to use (ollama or deepseek)
    #[arg(long, default_value = "")] // 空字符串表示未指定
    pub provider: String,

    /// Model to use (default: mistral)
    #[arg(short, long, default_value = "")] // 空字符串表示未指定
    pub model: String,

    /// 不自动执行 git add .
    #[arg(long, default_value_t = false)]
    pub no_add: bool,

    /// commit 后是否自动 push
    #[arg(long, default_value_t = false)]
    pub push: bool,

    /// 创建新的 tag（可指定版本号，如 --new-tag v1.2.0）
    #[arg(long, value_name = "VERSION", num_args = 0..=1, default_value = None)]
    pub new_tag: Option<String>,

    /// 是否显示最新的 tag 信息
    #[arg(long, default_value_t = false)]
    pub show_tag: bool,

    /// 推送 tag 时是否同时推送 master develop main 分支
    #[arg(long, default_value_t = false)]
    pub push_branches: bool,
}
