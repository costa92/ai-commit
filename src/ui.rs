use std::io::{self, Write};

/// 用户交互界面模块
/// 处理命令行用户输入和确认操作

/// 用户确认的结果
#[derive(Debug, PartialEq)]
pub enum ConfirmResult {
    /// 用户确认使用指定的消息
    Confirmed(String),
    /// 用户拒绝
    Rejected,
}

/// 显示 AI 生成的 commit message 并请求用户确认
/// 
/// 支持三种操作：
/// - y/yes/回车: 确认使用 AI 生成的消息
/// - n/no: 拒绝并取消操作
/// - e/edit: 启动编辑器编辑消息（支持 vim、vi、nano 等）
pub fn confirm_commit_message(message: &str, skip_confirm: bool) -> anyhow::Result<ConfirmResult> {
    if skip_confirm {
        return Ok(ConfirmResult::Confirmed(message.to_string()));
    }

    // 显示生成的 commit message
    println!("🤖 AI 生成的 commit message:");
    println!("   {}", message);
    println!();
    
    loop {
        print!("确认使用此 commit message? (y)es/(n)o/(e)dit: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        match input.as_str() {
            "y" | "yes" | "" => return Ok(ConfirmResult::Confirmed(message.to_string())),  // 默认回车视为确认
            "n" | "no" => return Ok(ConfirmResult::Rejected),
            "e" | "edit" => {
                // 允许用户编辑 commit message
                return edit_commit_message(message);
            }
            _ => {
                println!("请输入 y/yes, n/no, 或 e/edit");
                continue;
            }
        }
    }
}

/// 允许用户使用外部编辑器编辑 commit message
/// 
/// 功能特性：
/// - 自动检测可用编辑器：EDITOR 环境变量 -> VISUAL 环境变量 -> vim -> vi -> nano
/// - 预填充 AI 生成的内容到临时文件
/// - 支持格式验证和二次确认
/// - 编辑器不可用时自动回退到命令行输入模式
fn edit_commit_message(initial_message: &str) -> anyhow::Result<ConfirmResult> {
    use std::env;
    use std::fs;
    use std::process::Command;
    
    // 创建临时文件
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join("ai_commit_message.txt");
    
    // 将初始消息写入临时文件
    fs::write(&temp_file, initial_message)?;
    
    // 验证文件写入成功
    if !temp_file.exists() {
        return Err(anyhow::anyhow!("无法创建临时文件: {}", temp_file.display()));
    }
    
    // 调试信息：显示临时文件路径和内容
    if env::var("AI_COMMIT_DEBUG").is_ok() {
        println!("DEBUG: 临时文件路径: {}", temp_file.display());
        println!("DEBUG: 预填充内容: '{}'", initial_message);
        if let Ok(content) = fs::read_to_string(&temp_file) {
            println!("DEBUG: 文件实际内容: '{}'", content);
        }
    }
    
    // 获取编辑器命令，优先使用环境变量，然后尝试 vim、vi、nano
    let editor_result = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // 使用简单的 which 命令检查编辑器可用性
            let editors = ["vim", "vi", "nano"];
            for editor in &editors {
                if Command::new("which").arg(editor).output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
                {
                    return editor.to_string();
                }
            }
            // 如果 which 不可用，直接尝试常见编辑器
            for editor in &editors {
                if Command::new(editor).arg("--help").output().is_ok() ||
                   Command::new(editor).arg("--version").output().is_ok() {
                    return editor.to_string();
                }
            }
            // 无可用编辑器
            "".to_string()
        });
    
    // 如果没有找到编辑器，回退到命令行输入
    if editor_result.is_empty() {
        if env::var("AI_COMMIT_DEBUG").is_ok() {
            println!("DEBUG: 没有找到可用的编辑器，回退到命令行输入模式");
            println!("DEBUG: 环境变量 EDITOR: {:?}", env::var("EDITOR"));
            println!("DEBUG: 环境变量 VISUAL: {:?}", env::var("VISUAL"));
        }
        return edit_commit_message_fallback(initial_message);
    }
    
    if env::var("AI_COMMIT_DEBUG").is_ok() {
        println!("DEBUG: 选择的编辑器: {}", editor_result);
    }
    
    println!("正在启动编辑器 ({})...", editor_result);
    println!("提示: 保存并退出编辑器以确认提交消息");
    
    // 显示临时文件信息（仅在调试模式下显示完整信息）
    if env::var("AI_COMMIT_DEBUG").is_ok() {
        println!("临时文件路径: {}", temp_file.display());
        println!("预填充内容: {}", initial_message);
        
        print!("按回车继续启动编辑器，或输入 'show' 查看临时文件内容: ");
        io::stdout().flush().unwrap_or(());
        let mut debug_input = String::new();
        if io::stdin().read_line(&mut debug_input).is_ok() {
            if debug_input.trim() == "show" {
                if let Ok(content) = fs::read_to_string(&temp_file) {
                    println!("=== 临时文件内容 ===");
                    println!("{}", content);
                    println!("==================");
                }
            }
        }
    } else {
        println!("编辑器将打开预填充的提交消息，请编辑后保存退出");
    }
    
    // 启动编辑器前，再次确认文件存在且可读
    if let Ok(content) = fs::read_to_string(&temp_file) {
        if content != initial_message {
            println!("警告: 临时文件内容与预期不符!");
            println!("预期: {}", initial_message);
            println!("实际: {}", content);
        }
    } else {
        return Err(anyhow::anyhow!("无法读取临时文件: {}", temp_file.display()));
    }
    
    // 为不同编辑器准备特定参数
    let mut cmd = Command::new(&editor_result);
    cmd.arg(&temp_file);
    
    // 确保编辑器在正确的工作目录中运行
    if let Ok(current_dir) = env::current_dir() {
        cmd.current_dir(current_dir);
    }
    
    // 为 vim/vi 添加特定参数以确保正确显示
    if editor_result == "vim" || editor_result == "vi" {
        cmd.args(&["+set", "nobackup", "+set", "noswapfile", "+set", "nowritebackup"]);
    }
    
    // 启动编辑器
    let status = cmd.status();
    
    match status {
        Ok(status) if status.success() => {
            // 读取编辑后的内容
            let edited_content = fs::read_to_string(&temp_file)
                .map_err(|e| anyhow::anyhow!("无法读取编辑后的内容: {}", e))?;
            
            // 清理临时文件
            let _ = fs::remove_file(&temp_file);
            
            let edited_message = edited_content.trim().to_string();
            
            if edited_message.is_empty() {
                println!("Commit message 为空，操作已取消。");
                return Ok(ConfirmResult::Rejected);
            }
            
            // 验证编辑的消息格式
            validate_and_confirm_edited_message(&edited_message)
        }
        Ok(_) => {
            // 用户取消了编辑器操作
            let _ = fs::remove_file(&temp_file);
            println!("编辑器操作已取消。");
            Ok(ConfirmResult::Rejected)
        }
        Err(_) => {
            // 编辑器启动失败，回退到命令行输入
            let _ = fs::remove_file(&temp_file);
            println!("无法启动编辑器 '{}'，回退到命令行输入模式...", editor_result);
            edit_commit_message_fallback(initial_message)
        }
    }
}

/// 回退的命令行编辑模式
fn edit_commit_message_fallback(initial_message: &str) -> anyhow::Result<ConfirmResult> {
    println!("请输入您的 commit message:");
    println!("当前内容: {}", initial_message);
    print!("> ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let edited_message = input.trim().to_string();
    
    if edited_message.is_empty() {
        println!("Commit message 不能为空，操作已取消。");
        return Ok(ConfirmResult::Rejected);
    }
    
    validate_and_confirm_edited_message(&edited_message)
}

/// 验证并确认编辑后的消息
fn validate_and_confirm_edited_message(edited_message: &str) -> anyhow::Result<ConfirmResult> {
    // 验证编辑的消息格式
    if !is_valid_commit_message(edited_message) {
        println!("⚠️  警告: Commit message 格式可能不符合 Conventional Commits 规范");
        println!("   建议格式: type(scope): description");
        println!("   实际内容: {}", edited_message);
        println!();
        
        print!("是否仍要使用此消息? (y/n): ");
        io::stdout().flush()?;
        
        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;
        let confirm = confirm.trim().to_lowercase();
        
        if confirm != "y" && confirm != "yes" {
            return Ok(ConfirmResult::Rejected);
        }
    }
    
    println!("✓ 已使用编辑的 commit message: {}", edited_message);
    Ok(ConfirmResult::Confirmed(edited_message.to_string()))
}

/// 简单验证 commit message 格式
fn is_valid_commit_message(message: &str) -> bool {
    // 检查是否符合 Conventional Commits 格式
    let conventional_commit_regex = regex::Regex::new(r"^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .+").unwrap();
    conventional_commit_regex.is_match(message)
}

/// 显示选择菜单并获取用户选择
pub fn show_menu_and_get_choice(options: &[&str]) -> anyhow::Result<usize> {
    println!();
    for (i, option) in options.iter().enumerate() {
        println!("  {}: {}", i + 1, option);
    }
    println!();
    
    loop {
        print!("请选择 (1-{}): ", options.len());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice >= 1 && choice <= options.len() {
                return Ok(choice - 1);
            }
        }
        
        println!("无效选择，请输入 1 到 {} 之间的数字", options.len());
    }
}

/// 简单的 yes/no 确认
pub fn confirm_action(prompt: &str) -> anyhow::Result<bool> {
    loop {
        print!("{} (y/n): ", prompt);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        match input.as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                println!("请输入 y/yes 或 n/no");
                continue;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_commit_message() {
        // 有效的 commit messages
        assert!(is_valid_commit_message("feat(api): 添加用户认证功能"));
        assert!(is_valid_commit_message("fix: 修复按钮显示问题"));
        assert!(is_valid_commit_message("docs(readme): 更新安装说明"));
        assert!(is_valid_commit_message("refactor(core): 重构数据处理逻辑"));
        assert!(is_valid_commit_message("test: 添加单元测试"));
        assert!(is_valid_commit_message("chore: 更新依赖包"));
        assert!(is_valid_commit_message("style(ui): 优化界面样式"));

        // 无效的 commit messages
        assert!(!is_valid_commit_message("添加新功能"));
        assert!(!is_valid_commit_message("update readme"));
        assert!(!is_valid_commit_message("feat 添加功能"));
        assert!(!is_valid_commit_message("feat():"));
        assert!(!is_valid_commit_message(""));
    }

    #[test]
    fn test_commit_message_validation_edge_cases() {
        // 边界情况测试
        assert!(is_valid_commit_message("feat: a"));  // 最短有效消息
        assert!(is_valid_commit_message("fix(component): 这是一个很长的提交消息，用来测试长消息的处理情况"));
        assert!(!is_valid_commit_message("feat: "));  // 只有空格
        assert!(!is_valid_commit_message("FEAT: 添加功能"));  // 大写类型
    }

    #[test] 
    fn test_commit_message_types() {
        let types = ["feat", "fix", "docs", "style", "refactor", "test", "chore"];
        
        for commit_type in &types {
            let message = format!("{}: 测试消息", commit_type);
            assert!(is_valid_commit_message(&message), "Type {} should be valid", commit_type);
            
            let message_with_scope = format!("{}(scope): 测试消息", commit_type);
            assert!(is_valid_commit_message(&message_with_scope), "Type {} with scope should be valid", commit_type);
        }
    }

    #[test]
    fn test_commit_message_with_special_characters() {
        // 测试包含特殊字符的消息
        assert!(is_valid_commit_message("feat: 添加API接口/用户管理"));
        assert!(is_valid_commit_message("fix(ui): 修复按钮点击事件#123"));
        assert!(is_valid_commit_message("docs: 更新README.md文档"));
        
        // 包含 emoji 的消息
        assert!(is_valid_commit_message("feat: 🎉 添加新功能"));
        assert!(is_valid_commit_message("fix: 🐛 修复bug"));
    }

    #[test]
    fn test_commit_message_scope_variations() {
        // 测试不同的 scope 格式
        assert!(is_valid_commit_message("feat(api): 添加功能"));
        assert!(is_valid_commit_message("feat(user-auth): 添加功能"));
        assert!(is_valid_commit_message("feat(ui/components): 添加功能"));
        assert!(is_valid_commit_message("feat(123): 添加功能"));
    }
}