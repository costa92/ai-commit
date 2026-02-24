/// Diff行类型
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    Context,  // 上下文行
    Added,    // 添加的行
    Removed,  // 删除的行
    Header,   // 文件头
    Hunk,     // 代码块头
    FileTree, // 文件树结构
    Binary,   // 二进制文件
}

/// Diff显示模式
#[derive(Debug, Clone, PartialEq)]
pub enum DiffDisplayMode {
    Unified,    // 统一diff模式（默认）
    SideBySide, // 并排对比模式
    FileTree,   // 文件树形diff
}

/// 文件信息
#[derive(Debug, Clone)]
pub struct DiffFile {
    pub path: String,
    pub old_path: Option<String>,
    pub is_binary: bool,
    pub is_image: bool,
    pub additions: u32,
    pub deletions: u32,
    pub lines: Vec<DiffLine>,
}

/// Diff行数据
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
}

/// 文件树节点类型
#[derive(Debug, Clone)]
pub(super) enum FileTreeNode {
    Directory(std::collections::BTreeMap<String, FileTreeNode>),
    File(usize), // 文件索引
}

/// 根据文件扩展名获取图标
pub(super) fn get_file_icon(path: &str) -> Option<&'static str> {
    let extension = path.split('.').next_back()?.to_lowercase();
    match extension.as_str() {
        "rs" => Some("🦀 "),
        "py" => Some("🐍 "),
        "js" | "ts" => Some("⚡ "),
        "html" | "htm" => Some("🌐 "),
        "css" | "scss" | "sass" => Some("🎨 "),
        "json" => Some("📋 "),
        "xml" => Some("📰 "),
        "md" | "markdown" => Some("📝 "),
        "txt" => Some("📄 "),
        "toml" | "yaml" | "yml" => Some("⚙️ "),
        "sh" | "bash" => Some("🐚 "),
        "dockerfile" => Some("🐳 "),
        "go" => Some("🔷 "),
        "java" | "class" => Some("☕ "),
        "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" => Some("⚡ "),
        "rb" => Some("💎 "),
        "php" => Some("🐘 "),
        "sql" => Some("🗄️ "),
        _ => None,
    }
}
