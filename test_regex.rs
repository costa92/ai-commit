use regex::Regex;

fn main() {
    let re = Regex::new(r"^(feat|fix|docs|style|refactor|test|chore)(\([^)]+\))?:\s*.+$").unwrap();
    let test_msg = "test(git): 重构并增强Git工作树和标签测试覆盖率";
    println!("Message: {}", test_msg);
    println!("Matches: {}", re.is_match(test_msg));
    println!("Length: {} chars", test_msg.chars().count());
    
    // 测试一些可能的问题情况
    let test_cases = vec![
        "test(git): 重构并增强Git工作树和标签测试覆盖率",
        "feat(api): 添加用户认证功能",
        "fix(ui): 修复按钮显示问题",
        "refactor(core): 重构数据处理逻辑",
        "根据提供的变更信息和格式要求，以下是符合规范的提交消息：", // 不符合格式
        "", // 空字符串
    ];
    
    for (i, msg) in test_cases.iter().enumerate() {
        println!("Test case {}: '{}' -> {}", i + 1, msg, re.is_match(msg));
    }
}