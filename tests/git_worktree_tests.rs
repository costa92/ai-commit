use ai_commit::git::worktree::*;
use std::path::PathBuf;

/// Git Worktree 模块集成测试
/// 
/// 这些测试验证 worktree 模块各个子模块之间的协作
/// 以及整体功能的正确性

#[cfg(test)]
mod worktree_integration_tests {
    use super::*;

    #[test]
    fn test_worktree_info_serialization() {
        let info = WorktreeInfo::new(
            PathBuf::from("/test/path"),
            "main".to_string(),
            "abc123".to_string(),
            false,
            false,
        );

        // 测试序列化
        let json = serde_json::to_string(&info);
        assert!(json.is_ok());

        // 测试反序列化
        let deserialized: Result<WorktreeInfo, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());

        let restored = deserialized.unwrap();
        assert_eq!(restored.path, info.path);
        assert_eq!(restored.branch, info.branch);
        assert_eq!(restored.commit, info.commit);
        assert_eq!(restored.is_bare, info.is_bare);
        assert_eq!(restored.is_detached, info.is_detached);
    }

    #[test]
    fn test_worktree_list_options_completeness() {
        let mut options = WorktreeListOptions::default();
        
        // 测试默认值
        assert!(!options.verbose);
        assert!(!options.porcelain);
        assert!(!options.z);
        assert!(options.expire.is_none());

        // 测试所有选项组合
        let test_combinations = vec![
            (true, false, false, None),
            (false, true, false, None),
            (false, false, true, None),
            (false, false, false, Some("1week".to_string())),
            (true, false, true, Some("2weeks".to_string())),
        ];

        for (verbose, porcelain, z, expire) in test_combinations {
            options.verbose = verbose;
            options.porcelain = porcelain;
            options.z = z;
            options.expire = expire.clone();

            assert_eq!(options.verbose, verbose);
            assert_eq!(options.porcelain, porcelain);
            assert_eq!(options.z, z);
            assert_eq!(options.expire, expire);
        }
    }

    #[test]
    fn test_worktree_data_consistency() {
        // 测试不同状态的 worktree 数据一致性
        let test_cases = vec![
            // 正常分支
            (PathBuf::from("/repo/main"), "main", "abc123", false, false),
            // Bare worktree
            (PathBuf::from("/repo/bare"), "bare", "def456", true, false),
            // Detached HEAD
            (PathBuf::from("/repo/detached"), "detached", "ghi789", false, true),
            // 复杂分支名
            (PathBuf::from("/repo/feature"), "feature/ui/new-design", "jkl012", false, false),
        ];

        for (path, branch, commit, is_bare, is_detached) in test_cases {
            let info = WorktreeInfo::new(
                path.clone(),
                branch.to_string(),
                commit.to_string(),
                is_bare,
                is_detached,
            );

            assert_eq!(info.path, path);
            assert_eq!(info.branch, branch);
            assert_eq!(info.commit, commit);
            assert_eq!(info.is_bare, is_bare);
            assert_eq!(info.is_detached, is_detached);

            // 测试状态逻辑一致性
            if is_bare {
                assert!(!is_detached, "Bare worktree should not be detached");
            }
        }
    }

    #[test]
    fn test_worktree_path_handling() {
        // 测试各种路径格式的处理
        let path_formats = vec![
            "/absolute/path/worktree",
            "relative/path/worktree",
            "./current/path/worktree",
            "../parent/path/worktree",
            "~/home/path/worktree",
            "/path with spaces/worktree",
            "/path-with-dashes/worktree",
            "/path_with_underscores/worktree",
        ];

        for path_str in path_formats {
            let path = PathBuf::from(path_str);
            let info = WorktreeInfo::new(
                path.clone(),
                "test".to_string(),
                "abc123".to_string(),
                false,
                false,
            );

            assert_eq!(info.path, path);
            assert_eq!(info.path.to_string_lossy(), path_str);
        }
    }

    #[test]
    fn test_worktree_branch_name_handling() {
        // 测试各种分支名格式的处理
        let branch_formats = vec![
            "main",
            "develop",
            "feature/new-ui",
            "feature/ui/component",
            "bugfix/issue-123",
            "hotfix/critical-fix",
            "release/v1.2.3",
            "support/legacy-feature",
            "feat/UI-123_special@chars",
        ];

        for branch in branch_formats {
            let info = WorktreeInfo::new(
                PathBuf::from("/test"),
                branch.to_string(),
                "abc123".to_string(),
                false,
                false,
            );

            assert_eq!(info.branch, branch);
            assert!(!info.branch.is_empty());
        }
    }

    #[test]
    fn test_worktree_commit_hash_handling() {
        // 测试各种提交哈希格式的处理
        let commit_formats = vec![
            "abc123",
            "abcd1234",
            "a1b2c3d4e5f6789",
            "1234567890abcdef1234567890abcdef12345678",
            "0000000", // 特殊情况
            "",         // 空提交哈希
        ];

        for commit in commit_formats {
            let info = WorktreeInfo::new(
                PathBuf::from("/test"),
                "main".to_string(),
                commit.to_string(),
                false,
                false,
            );

            assert_eq!(info.commit, commit);
        }
    }

    #[test]
    fn test_worktree_status_combinations() {
        // 测试 worktree 状态组合的有效性
        let status_combinations = vec![
            (false, false), // 正常 worktree
            (true, false),  // bare worktree
            (false, true),  // detached HEAD
            // 注意：(true, true) 理论上不应该存在，但我们不在数据结构层面限制
        ];

        for (is_bare, is_detached) in status_combinations {
            let info = WorktreeInfo::new(
                PathBuf::from("/test"),
                "test".to_string(),
                "abc123".to_string(),
                is_bare,
                is_detached,
            );

            assert_eq!(info.is_bare, is_bare);
            assert_eq!(info.is_detached, is_detached);
        }
    }

    #[test]
    fn test_worktree_clone_and_debug() {
        let original = WorktreeInfo::new(
            PathBuf::from("/test"),
            "main".to_string(),
            "abc123".to_string(),
            false,
            false,
        );

        // 测试克隆
        let cloned = original.clone();
        assert_eq!(original.path, cloned.path);
        assert_eq!(original.branch, cloned.branch);
        assert_eq!(original.commit, cloned.commit);
        assert_eq!(original.is_bare, cloned.is_bare);
        assert_eq!(original.is_detached, cloned.is_detached);

        // 测试 Debug trait
        let debug_str = format!("{:?}", original);
        assert!(debug_str.contains("WorktreeInfo"));
        assert!(debug_str.contains("/test"));
        assert!(debug_str.contains("main"));
        assert!(debug_str.contains("abc123"));
    }

    #[test]
    fn test_worktree_options_debug() {
        let options = WorktreeListOptions {
            verbose: true,
            porcelain: false,
            z: true,
            expire: Some("1week".to_string()),
        };

        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("WorktreeListOptions"));
        assert!(debug_str.contains("verbose: true"));
        assert!(debug_str.contains("porcelain: false"));
        assert!(debug_str.contains("z: true"));
        assert!(debug_str.contains("1week"));
    }

    #[test]
    fn test_module_public_interface() {
        // 测试模块公共接口的可访问性
        
        // 类型应该可访问
        let _info: WorktreeInfo = WorktreeInfo::new(
            PathBuf::from("/test"),
            "main".to_string(),
            "abc123".to_string(),
            false,
            false,
        );

        let _options: WorktreeListOptions = WorktreeListOptions::default();

        // 验证函数可访问性（编译时检查）
        // 注意：这些是 async 函数，这里只验证它们能被调用
        fn _verify_function_accessibility() {
            // 这些函数的存在性通过编译验证，无需运行时检查
            // 实际使用需要 await，这里只是编译时可见性验证
            async fn _example_usage() {
                let _result1 = create_worktree("test", None);
                let _result2 = create_worktree_with_new_branch("test", None);
                let _result3 = list_worktrees();
                let options = WorktreeListOptions::default();
                let _result4 = list_worktrees_with_options(&options);
                let _result5 = list_worktrees_raw(&options);
                let _result6 = remove_worktree("test");
                let _result7 = prune_worktrees();
                let _result8 = clear_other_worktrees();
                let _result9 = switch_to_worktree("test");
                let _result10 = get_current_worktree();
            }
        }
    }
}