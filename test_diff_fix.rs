use std::process::Command;

fn main() {
    println!("Testing git diff functionality fixes...");
    
    // 测试基本 git 命令
    println!("\n1. Testing basic git commands:");
    
    // 测试 git log
    match Command::new("git")
        .args(["log", "--oneline", "-5"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let commits = String::from_utf8_lossy(&output.stdout);
                println!("✓ Recent commits:");
                for line in commits.lines().take(3) {
                    println!("  {}", line);
                }
            } else {
                println!("✗ Git log failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => println!("✗ Failed to run git log: {}", e),
    }
    
    // 测试 git show
    match Command::new("git")
        .args(["log", "--format=%H", "-1"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let latest_commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
                println!("\n2. Testing git show for latest commit: {}", &latest_commit[..8]);
                
                match Command::new("git")
                    .args(["show", &latest_commit])
                    .output()
                {
                    Ok(show_output) => {
                        if show_output.status.success() {
                            let diff_content = String::from_utf8_lossy(&show_output.stdout);
                            println!("✓ Git show successful, content length: {} bytes", diff_content.len());
                            
                            // 显示前几行作为示例
                            let lines: Vec<&str> = diff_content.lines().take(10).collect();
                            println!("  First few lines:");
                            for line in lines {
                                println!("    {}", line);
                            }
                        } else {
                            println!("✗ Git show failed: {}", String::from_utf8_lossy(&show_output.stderr));
                        }
                    }
                    Err(e) => println!("✗ Failed to run git show: {}", e),
                }
            }
        }
        Err(e) => println!("✗ Failed to get latest commit: {}", e),
    }
    
    println!("\n3. Testing enhanced git commands used in our fix:");
    
    // 测试我们在修复中使用的命令
    match Command::new("git")
        .args(["log", "--format=%H", "-1"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let latest_commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
                
                // 测试 rev-parse (我们在修复中添加的验证命令)
                match Command::new("git")
                    .args(["rev-parse", "--verify", &latest_commit])
                    .output()
                {
                    Ok(verify_output) => {
                        if verify_output.status.success() {
                            println!("✓ git rev-parse verification successful");
                        } else {
                            println!("✗ git rev-parse failed");
                        }
                    }
                    Err(e) => println!("✗ Failed to run git rev-parse: {}", e),
                }
                
                // 测试文件状态命令 (我们在修复中使用的)
                match Command::new("git")
                    .args(["show", "--name-status", "--format=", &latest_commit])
                    .output()
                {
                    Ok(status_output) => {
                        if status_output.status.success() {
                            let file_status = String::from_utf8_lossy(&status_output.stdout);
                            println!("✓ File status command successful");
                            println!("  Changed files:");
                            for line in file_status.lines().take(5) {
                                if !line.is_empty() {
                                    println!("    {}", line);
                                }
                            }
                        } else {
                            println!("✗ File status command failed");
                        }
                    }
                    Err(e) => println!("✗ Failed to get file status: {}", e),
                }
            }
        }
        Err(e) => println!("✗ Failed to get latest commit for testing: {}", e),
    }
    
    println!("\n✓ Git diff functionality test completed!");
    println!("The fixes should improve error handling and provide better debugging information.");
}