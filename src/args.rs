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

    /// 是否创建新的 tag
    #[arg(long, default_value_t = false)]
    pub new_tag: bool,

    /// 是否显示最新的 tag 信息
    #[arg(long, default_value_t = false)]
    pub show_tag: bool,
}
