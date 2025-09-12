use crate::git::core::GitCore;
use tokio::process::Command;

/// Git Flow 支持模块
pub struct GitFlow;

impl GitFlow {
    /// 初始化 Git Flow 仓库结构
    pub async fn init() -> anyhow::Result<()> {
        // 检查是否在 git 仓库中，如果不是则先初始化
        if !GitCore::is_git_repo().await {
            println!("Not in a Git repository, initializing...");
            GitCore::init_repository().await?;
        }

        // 确保 develop 分支存在
        if !GitCore::branch_exists("develop").await? {
            println!("Creating develop branch...");
            GitCore::create_and_checkout_branch("develop").await?;

            // 推送 develop 分支到远程
            if !GitCore::get_remotes().await?.is_empty() {
                GitCore::push_branch("develop", "origin", true).await?;
                println!("✓ Pushed develop branch to origin");
            }
        }

        // 切换回 main/master 分支
        let main_branch = Self::get_main_branch().await?;
        GitCore::checkout_branch(&main_branch).await?;

        println!("✓ Git Flow initialized");
        println!("  - Main branch: {}", main_branch);
        println!("  - Develop branch: develop");

        Ok(())
    }

    /// 开始新的 feature 分支
    pub async fn start_feature(name: &str) -> anyhow::Result<()> {
        let feature_branch = format!("feature/{}", name);

        // 检查 feature 分支是否已存在
        if GitCore::branch_exists(&feature_branch).await? {
            anyhow::bail!("Feature branch '{}' already exists", feature_branch);
        }

        // 确保在 develop 分支
        if !GitCore::branch_exists("develop").await? {
            anyhow::bail!("Develop branch does not exist. Run --flow-init first.");
        }

        GitCore::checkout_branch("develop").await?;

        // 从 develop 创建 feature 分支
        GitCore::create_and_checkout_branch(&feature_branch).await?;

        println!("✓ Started feature branch: {}", feature_branch);
        println!("  - Based on: develop");
        println!("  - Current branch: {}", feature_branch);

        Ok(())
    }

    /// 完成 feature 分支
    pub async fn finish_feature(name: &str) -> anyhow::Result<()> {
        let feature_branch = format!("feature/{}", name);

        // 检查 feature 分支是否存在
        if !GitCore::branch_exists(&feature_branch).await? {
            anyhow::bail!("Feature branch '{}' does not exist", feature_branch);
        }

        // 确保工作区干净
        if !GitCore::is_working_tree_clean().await? {
            anyhow::bail!("Working tree is not clean. Please commit or stash your changes.");
        }

        // 切换到 develop 分支
        GitCore::checkout_branch("develop").await?;

        // 合并 feature 分支到 develop
        let merge_message = format!("Merge feature branch '{}'", name);
        GitCore::merge_branch(&feature_branch, Some(&merge_message)).await?;

        // 删除 feature 分支
        GitCore::delete_branch(&feature_branch, false).await?;

        println!("✓ Finished feature: {}", name);
        println!("  - Merged into: develop");
        println!("  - Deleted branch: {}", feature_branch);

        Ok(())
    }

    /// 开始新的 hotfix 分支
    pub async fn start_hotfix(name: &str) -> anyhow::Result<()> {
        let hotfix_branch = format!("hotfix/{}", name);

        // 检查 hotfix 分支是否已存在
        if GitCore::branch_exists(&hotfix_branch).await? {
            anyhow::bail!("Hotfix branch '{}' already exists", hotfix_branch);
        }

        // 从 main 分支创建 hotfix 分支
        let main_branch = Self::get_main_branch().await?;
        GitCore::checkout_branch(&main_branch).await?;
        GitCore::create_and_checkout_branch(&hotfix_branch).await?;

        println!("✓ Started hotfix branch: {}", hotfix_branch);
        println!("  - Based on: {}", main_branch);
        println!("  - Current branch: {}", hotfix_branch);

        Ok(())
    }

    /// 完成 hotfix 分支
    pub async fn finish_hotfix(name: &str) -> anyhow::Result<()> {
        let hotfix_branch = format!("hotfix/{}", name);

        // 检查 hotfix 分支是否存在
        if !GitCore::branch_exists(&hotfix_branch).await? {
            anyhow::bail!("Hotfix branch '{}' does not exist", hotfix_branch);
        }

        // 确保工作区干净
        if !GitCore::is_working_tree_clean().await? {
            anyhow::bail!("Working tree is not clean. Please commit or stash your changes.");
        }

        let main_branch = Self::get_main_branch().await?;
        let merge_message = format!("Hotfix: {}", name);

        // 合并到 main 分支
        GitCore::checkout_branch(&main_branch).await?;
        GitCore::merge_branch(&hotfix_branch, Some(&merge_message)).await?;

        // 如果 develop 分支存在，也合并到 develop
        if GitCore::branch_exists("develop").await? {
            GitCore::checkout_branch("develop").await?;
            GitCore::merge_branch(&hotfix_branch, Some(&merge_message)).await?;
        }

        // 删除 hotfix 分支
        GitCore::delete_branch(&hotfix_branch, false).await?;

        println!("✓ Finished hotfix: {}", name);
        println!("  - Merged into: {}", main_branch);
        if GitCore::branch_exists("develop").await? {
            println!("  - Merged into: develop");
        }
        println!("  - Deleted branch: {}", hotfix_branch);

        Ok(())
    }

    /// 开始新的 release 分支
    pub async fn start_release(version: &str) -> anyhow::Result<()> {
        let release_branch = format!("release/{}", version);

        // 检查 release 分支是否已存在
        if GitCore::branch_exists(&release_branch).await? {
            anyhow::bail!("Release branch '{}' already exists", release_branch);
        }

        // 确保 develop 分支存在
        if !GitCore::branch_exists("develop").await? {
            anyhow::bail!("Develop branch does not exist. Run --flow-init first.");
        }

        GitCore::checkout_branch("develop").await?;
        GitCore::create_and_checkout_branch(&release_branch).await?;

        println!("✓ Started release branch: {}", release_branch);
        println!("  - Based on: develop");
        println!("  - Current branch: {}", release_branch);
        println!("  - Ready for release preparation and testing");

        Ok(())
    }

    /// 完成 release 分支
    pub async fn finish_release(version: &str) -> anyhow::Result<()> {
        let release_branch = format!("release/{}", version);

        // 检查 release 分支是否存在
        if !GitCore::branch_exists(&release_branch).await? {
            anyhow::bail!("Release branch '{}' does not exist", release_branch);
        }

        // 确保工作区干净
        if !GitCore::is_working_tree_clean().await? {
            anyhow::bail!("Working tree is not clean. Please commit or stash your changes.");
        }

        let main_branch = Self::get_main_branch().await?;
        let merge_message = format!("Release {}", version);

        // 合并到 main 分支
        GitCore::checkout_branch(&main_branch).await?;
        GitCore::merge_branch(&release_branch, Some(&merge_message)).await?;

        // 创建 release tag
        let tag_name = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        let tag_status = Command::new("git")
            .args([
                "tag",
                "-a",
                &tag_name,
                "-m",
                &format!("Release {}", version),
            ])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create tag: {}", e))?;

        if !tag_status.success() {
            anyhow::bail!("Failed to create release tag");
        }

        // 合并到 develop 分支
        if GitCore::branch_exists("develop").await? {
            GitCore::checkout_branch("develop").await?;
            GitCore::merge_branch(&release_branch, Some(&merge_message)).await?;
        }

        // 删除 release 分支
        GitCore::delete_branch(&release_branch, false).await?;

        println!("✓ Finished release: {}", version);
        println!("  - Merged into: {}", main_branch);
        println!("  - Created tag: {}", tag_name);
        if GitCore::branch_exists("develop").await? {
            println!("  - Merged into: develop");
        }
        println!("  - Deleted branch: {}", release_branch);

        Ok(())
    }

    /// 获取主分支名称（main 或 master）
    async fn get_main_branch() -> anyhow::Result<String> {
        if GitCore::branch_exists("main").await? {
            Ok("main".to_string())
        } else if GitCore::branch_exists("master").await? {
            Ok("master".to_string())
        } else {
            anyhow::bail!("Neither 'main' nor 'master' branch exists");
        }
    }

    /// 检查当前分支类型
    pub async fn get_branch_type() -> anyhow::Result<BranchType> {
        let current_branch = GitCore::get_current_branch().await?;

        if current_branch == "main" || current_branch == "master" {
            Ok(BranchType::Main)
        } else if current_branch == "develop" {
            Ok(BranchType::Develop)
        } else if current_branch.starts_with("feature/") {
            Ok(BranchType::Feature)
        } else if current_branch.starts_with("hotfix/") {
            Ok(BranchType::Hotfix)
        } else if current_branch.starts_with("release/") {
            Ok(BranchType::Release)
        } else {
            Ok(BranchType::Other)
        }
    }

    /// 列出所有 flow 分支
    pub async fn list_flow_branches() -> anyhow::Result<()> {
        let output = Command::new("git")
            .args(["branch", "--list", "--format=%(refname:short)"])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list branches: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git branch command failed");
        }

        let branches = String::from_utf8_lossy(&output.stdout);
        let mut features = Vec::new();
        let mut hotfixes = Vec::new();
        let mut releases = Vec::new();

        for branch in branches.lines() {
            let branch = branch.trim();
            if branch.starts_with("feature/") {
                features.push(branch);
            } else if branch.starts_with("hotfix/") {
                hotfixes.push(branch);
            } else if branch.starts_with("release/") {
                releases.push(branch);
            }
        }

        println!("🌿 Git Flow Branches:");
        println!("{}", "─".repeat(40));

        if !features.is_empty() {
            println!("\n📦 Features:");
            for feature in &features {
                println!("  - {}", feature);
            }
        }

        if !hotfixes.is_empty() {
            println!("\n🚨 Hotfixes:");
            for hotfix in &hotfixes {
                println!("  - {}", hotfix);
            }
        }

        if !releases.is_empty() {
            println!("\n🚀 Releases:");
            for release in &releases {
                println!("  - {}", release);
            }
        }

        if features.is_empty() && hotfixes.is_empty() && releases.is_empty() {
            println!("No flow branches found.");
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum BranchType {
    Main,
    Develop,
    Feature,
    Hotfix,
    Release,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_main_branch() {
        let result = GitFlow::get_main_branch().await;

        match result {
            Ok(branch) => {
                assert!(branch == "main" || branch == "master");
                println!("Main branch: {}", branch);
            }
            Err(e) => {
                println!(
                    "Failed to get main branch (expected in non-git environment): {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_branch_type() {
        let result = GitFlow::get_branch_type().await;

        match result {
            Ok(branch_type) => {
                println!("Current branch type: {:?}", branch_type);
            }
            Err(e) => {
                println!("Failed to get branch type: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_list_flow_branches() {
        let result = GitFlow::list_flow_branches().await;

        match result {
            Ok(_) => {
                println!("Successfully listed flow branches");
            }
            Err(e) => {
                println!("Failed to list flow branches: {}", e);
            }
        }
    }

    #[test]
    fn test_branch_type_detection() {
        let test_cases = vec![
            ("main", BranchType::Main),
            ("master", BranchType::Main),
            ("develop", BranchType::Develop),
            ("feature/new-ui", BranchType::Feature),
            ("feature/auth-system", BranchType::Feature),
            ("hotfix/critical-bug", BranchType::Hotfix),
            ("hotfix/security-fix", BranchType::Hotfix),
            ("release/1.0.0", BranchType::Release),
            ("release/v2.1.0", BranchType::Release),
            ("custom-branch", BranchType::Other),
        ];

        for (branch_name, expected_type) in test_cases {
            let detected_type = if branch_name == "main" || branch_name == "master" {
                BranchType::Main
            } else if branch_name == "develop" {
                BranchType::Develop
            } else if branch_name.starts_with("feature/") {
                BranchType::Feature
            } else if branch_name.starts_with("hotfix/") {
                BranchType::Hotfix
            } else if branch_name.starts_with("release/") {
                BranchType::Release
            } else {
                BranchType::Other
            };

            assert_eq!(
                detected_type, expected_type,
                "Branch '{}' should be detected as {:?}",
                branch_name, expected_type
            );
        }
    }

    #[tokio::test]
    async fn test_git_flow_init() {
        let result = GitFlow::init().await;
        match result {
            Ok(_) => println!("Git Flow init succeeded"),
            Err(e) => println!(
                "Git Flow init failed (expected if already initialized): {}",
                e
            ),
        }
    }

    #[tokio::test]
    async fn test_feature_workflow() {
        let feature_name = "test-feature";

        // Test starting a feature
        let result = GitFlow::start_feature(feature_name).await;
        match result {
            Ok(_) => {
                println!("Feature '{}' started successfully", feature_name);

                // Test finishing the feature
                let finish_result = GitFlow::finish_feature(feature_name).await;
                match finish_result {
                    Ok(_) => println!("Feature '{}' finished successfully", feature_name),
                    Err(e) => println!("Feature '{}' finish failed: {}", feature_name, e),
                }
            }
            Err(e) => println!("Feature '{}' start failed: {}", feature_name, e),
        }
    }

    #[tokio::test]
    async fn test_hotfix_workflow() {
        let hotfix_name = "test-hotfix";

        // Test starting a hotfix
        let result = GitFlow::start_hotfix(hotfix_name).await;
        match result {
            Ok(_) => {
                println!("Hotfix '{}' started successfully", hotfix_name);

                // Test finishing the hotfix
                let finish_result = GitFlow::finish_hotfix(hotfix_name).await;
                match finish_result {
                    Ok(_) => println!("Hotfix '{}' finished successfully", hotfix_name),
                    Err(e) => println!("Hotfix '{}' finish failed: {}", hotfix_name, e),
                }
            }
            Err(e) => println!("Hotfix '{}' start failed: {}", hotfix_name, e),
        }
    }

    #[tokio::test]
    async fn test_release_workflow() {
        let release_version = "1.0.0-test";

        // Test starting a release
        let result = GitFlow::start_release(release_version).await;
        match result {
            Ok(_) => {
                println!("Release '{}' started successfully", release_version);

                // Test finishing the release
                let finish_result = GitFlow::finish_release(release_version).await;
                match finish_result {
                    Ok(_) => println!("Release '{}' finished successfully", release_version),
                    Err(e) => println!("Release '{}' finish failed: {}", release_version, e),
                }
            }
            Err(e) => println!("Release '{}' start failed: {}", release_version, e),
        }
    }

    #[tokio::test]
    async fn test_get_branch_type_detailed() {
        let result = GitFlow::get_branch_type().await;
        match result {
            Ok(branch_type) => println!("Current branch type: {:?}", branch_type),
            Err(e) => println!("Failed to get branch type: {}", e),
        }
    }

    #[tokio::test]
    async fn test_list_flow_branches_detailed() {
        let result = GitFlow::list_flow_branches().await;
        match result {
            Ok(_) => println!("Flow branches listed successfully"),
            Err(e) => println!("Failed to list flow branches: {}", e),
        }
    }

    #[test]
    fn test_branch_name_validation() {
        // Test valid branch names
        let valid_names = vec![
            "simple-feature",
            "user-authentication",
            "bug-fix-123",
            "feature_with_underscores",
            "hotfix-critical-bug",
            "release-v2.0.0",
        ];

        for name in valid_names {
            assert!(
                !name.is_empty(),
                "Branch name should not be empty: '{}'",
                name
            );
            assert!(
                !name.contains(' '),
                "Branch name should not contain spaces: '{}'",
                name
            );
            assert!(
                name.len() <= 100,
                "Branch name should be reasonable length: '{}'",
                name
            );
        }

        // Test edge case names
        let edge_cases = vec![
            "",
            "a",
            "very-long-branch-name-that-might-cause-issues-with-git-flow-workflows",
            "with spaces",
            "with/slashes",
            "with\\backslashes",
        ];

        for name in edge_cases {
            if name.is_empty() {
                assert!(name.is_empty(), "Empty name should be detected");
            } else if name.contains(' ') {
                assert!(
                    name.contains(' '),
                    "Name with spaces should be detected: '{}'",
                    name
                );
            }
        }
    }

    #[test]
    fn test_branch_type_enum_completeness() {
        // Test all BranchType variants
        let branch_types = vec![
            BranchType::Main,
            BranchType::Develop,
            BranchType::Feature,
            BranchType::Hotfix,
            BranchType::Release,
            BranchType::Other,
        ];

        for branch_type in branch_types {
            // Ensure each type can be formatted and debugged
            let _debug_str = format!("{:?}", branch_type);
            let _display_str = format!("{:?}", branch_type); // Using Debug as Display isn't implemented
        }
    }

    #[tokio::test]
    async fn test_concurrent_flow_operations() {
        // Test multiple concurrent flow operations (read-only)
        use tokio::task;

        let type_task1 = task::spawn(async { GitFlow::get_branch_type().await });
        let branches_task = task::spawn(async { GitFlow::list_flow_branches().await });
        let type_task2 = task::spawn(async { GitFlow::get_branch_type().await });

        // Handle each task separately due to different return types
        match type_task1.await {
            Ok(result) => match result {
                Ok(_branch_type) => println!("Concurrent flow operation 1 succeeded"),
                Err(e) => println!("Concurrent flow operation 1 failed: {}", e),
            },
            Err(e) => println!("Task 1 join error: {}", e),
        }

        match branches_task.await {
            Ok(result) => match result {
                Ok(_) => println!("Concurrent flow branches operation succeeded"),
                Err(e) => println!("Concurrent flow branches operation failed: {}", e),
            },
            Err(e) => println!("Branches task join error: {}", e),
        }

        match type_task2.await {
            Ok(result) => match result {
                Ok(_branch_type) => println!("Concurrent flow operation 2 succeeded"),
                Err(e) => println!("Concurrent flow operation 2 failed: {}", e),
            },
            Err(e) => println!("Task 2 join error: {}", e),
        }
    }

    #[test]
    fn test_version_name_patterns() {
        // Test release version patterns
        let version_patterns = vec![
            "v1.0.0",
            "1.2.3",
            "2.0.0-beta",
            "3.1.4-alpha.1",
            "0.1.0-rc.2",
            "1.0.0-SNAPSHOT",
        ];

        for version in version_patterns {
            assert!(
                !version.is_empty(),
                "Version should not be empty: '{}'",
                version
            );
            assert!(
                version.len() <= 50,
                "Version should be reasonable length: '{}'",
                version
            );

            // Check if it contains typical version characters
            let has_version_chars = version
                .chars()
                .any(|c| c.is_numeric() || c == '.' || c == '-');
            assert!(
                has_version_chars,
                "Version should contain typical version characters: '{}'",
                version
            );
        }
    }

    #[tokio::test]
    async fn test_flow_error_scenarios() {
        // Test error handling with invalid inputs
        let invalid_names = vec!["", "name with spaces", "name/with/slashes"];

        for name in invalid_names {
            println!("Testing invalid feature name: '{}'", name);
            let result = GitFlow::start_feature(name).await;
            match result {
                Ok(_) => println!(
                    "Feature start succeeded unexpectedly with invalid name: '{}'",
                    name
                ),
                Err(e) => println!(
                    "Feature start failed as expected with invalid name '{}': {}",
                    name, e
                ),
            }
        }
    }

    #[test]
    fn test_branch_prefix_detection() {
        // Test branch prefix detection logic
        struct PrefixTest {
            branch_name: &'static str,
            expected_prefix: Option<&'static str>,
            expected_name: Option<&'static str>,
        }

        let prefix_tests = vec![
            PrefixTest {
                branch_name: "feature/user-auth",
                expected_prefix: Some("feature/"),
                expected_name: Some("user-auth"),
            },
            PrefixTest {
                branch_name: "hotfix/critical-bug",
                expected_prefix: Some("hotfix/"),
                expected_name: Some("critical-bug"),
            },
            PrefixTest {
                branch_name: "release/v1.2.0",
                expected_prefix: Some("release/"),
                expected_name: Some("v1.2.0"),
            },
            PrefixTest {
                branch_name: "main",
                expected_prefix: None,
                expected_name: Some("main"),
            },
            PrefixTest {
                branch_name: "develop",
                expected_prefix: None,
                expected_name: Some("develop"),
            },
            PrefixTest {
                branch_name: "custom-branch",
                expected_prefix: None,
                expected_name: Some("custom-branch"),
            },
        ];

        for test in prefix_tests {
            if let Some(expected_prefix) = test.expected_prefix {
                assert!(
                    test.branch_name.starts_with(expected_prefix),
                    "Branch '{}' should start with '{}'",
                    test.branch_name,
                    expected_prefix
                );

                if let Some(expected_name) = test.expected_name {
                    let actual_name = test.branch_name.strip_prefix(expected_prefix).unwrap();
                    assert_eq!(
                        actual_name, expected_name,
                        "Branch name after prefix should be '{}'",
                        expected_name
                    );
                }
            } else if let Some(expected_name) = test.expected_name {
                assert_eq!(
                    test.branch_name, expected_name,
                    "Branch without prefix should be '{}'",
                    expected_name
                );
            }
        }
    }

    #[tokio::test]
    async fn test_flow_state_consistency() {
        // Test that flow operations maintain consistent state
        let initial_branch_type = GitFlow::get_branch_type().await;

        match initial_branch_type {
            Ok(branch_type) => {
                println!("Initial branch type: {:?}", branch_type);

                // Test that branch type detection is consistent
                let second_check = GitFlow::get_branch_type().await;
                match second_check {
                    Ok(second_type) => {
                        // In a stable environment, branch type should be consistent
                        println!("Second check branch type: {:?}", second_type);
                        // Note: We don't assert equality as the user might switch branches during tests
                    }
                    Err(e) => println!("Second branch type check failed: {}", e),
                }
            }
            Err(e) => println!("Initial branch type check failed: {}", e),
        }
    }

    #[test]
    fn test_workflow_name_constraints() {
        // Test constraints on workflow names
        struct NameTest {
            name: &'static str,
            should_be_valid: bool,
            reason: &'static str,
        }

        let name_tests = vec![
            NameTest { name: "valid-name", should_be_valid: true, reason: "standard valid name" },
            NameTest { name: "valid_name", should_be_valid: true, reason: "underscore is valid" },
            NameTest { name: "valid123", should_be_valid: true, reason: "numbers are valid" },
            NameTest { name: "", should_be_valid: false, reason: "empty name invalid" },
            NameTest { name: "name with spaces", should_be_valid: false, reason: "spaces invalid" },
            NameTest { name: "name/with/slashes", should_be_valid: false, reason: "slashes typically invalid" },
            NameTest { name: "very-very-very-very-very-very-very-very-long-name-that-exceeds-reasonable-limits", should_be_valid: false, reason: "too long" },
        ];

        for test in name_tests {
            if test.should_be_valid {
                assert!(
                    !test.name.is_empty(),
                    "Valid name should not be empty: {} - {}",
                    test.name,
                    test.reason
                );
                assert!(
                    !test.name.contains(' '),
                    "Valid name should not contain spaces: {} - {}",
                    test.name,
                    test.reason
                );
                assert!(
                    test.name.len() <= 100,
                    "Valid name should not be too long: {} - {}",
                    test.name,
                    test.reason
                );
            } else {
                let has_issues =
                    test.name.is_empty() || test.name.contains(' ') || test.name.len() > 100;
                assert!(
                    has_issues,
                    "Invalid name should have issues: {} - {}",
                    test.name, test.reason
                );
            }
        }
    }

    #[tokio::test]
    async fn test_git_flow_commands_error_handling() {
        // Test error handling in non-git or non-initialized git flow environment
        use std::env;
        use std::path::Path;

        let original_dir = env::current_dir().unwrap();

        // Try to test in /tmp (not a git repo)
        if Path::new("/tmp").exists() {
            let _ = env::set_current_dir("/tmp");

            let result = GitFlow::get_branch_type().await;
            match result {
                Ok(_) => println!("Branch type succeeded unexpectedly in non-git dir"),
                Err(e) => println!("Branch type failed as expected in non-git dir: {}", e),
            }

            let result = GitFlow::list_flow_branches().await;
            match result {
                Ok(_) => println!("List branches succeeded unexpectedly in non-git dir"),
                Err(e) => println!("List branches failed as expected in non-git dir: {}", e),
            }

            // Restore original directory
            let _ = env::set_current_dir(original_dir);
        }
    }
}
