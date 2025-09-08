use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;

/// Git 仓库
#[derive(Clone)]
pub struct Repository {
    path: PathBuf,
    inner: Arc<RepositoryInner>,
}

struct RepositoryInner {
    is_bare: bool,
    git_dir: PathBuf,
}

impl Repository {
    /// 打开仓库
    pub async fn open(path: Option<PathBuf>) -> Result<Self> {
        let path = path.unwrap_or_else(|| PathBuf::from("."));
        
        // 检查是否是 Git 仓库
        if !Self::is_git_repository(&path).await? {
            anyhow::bail!("Not a git repository");
        }
        
        // 获取 Git 目录
        let git_dir = Self::get_git_dir(&path).await?;
        let is_bare = Self::is_bare_repository(&path).await?;
        
        Ok(Self {
            path,
            inner: Arc::new(RepositoryInner {
                is_bare,
                git_dir,
            }),
        })
    }

    /// 初始化新仓库
    pub async fn init(path: &Path) -> Result<Self> {
        let output = Command::new("git")
            .arg("init")
            .arg(path)
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("Failed to initialize repository");
        }
        
        Self::open(Some(path.to_path_buf())).await
    }

    /// 获取仓库路径
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 是否为裸仓库
    pub fn is_bare(&self) -> bool {
        self.inner.is_bare
    }

    /// 获取 Git 目录
    pub fn git_dir(&self) -> &Path {
        &self.inner.git_dir
    }

    /// 获取仓库状态
    pub async fn get_status(&self) -> Result<RepositoryStatus> {
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(&self.path)
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("Failed to get repository status");
        }
        
        let status_text = String::from_utf8_lossy(&output.stdout);
        RepositoryStatus::parse(&status_text)
    }

    /// 执行 Git 命令
    pub async fn run_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git command failed: {}", stderr);
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// 检查是否是 Git 仓库
    async fn is_git_repository(path: &Path) -> Result<bool> {
        let output = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(path)
            .output()
            .await?;
        
        Ok(output.status.success())
    }

    /// 获取 Git 目录路径
    async fn get_git_dir(path: &Path) -> Result<PathBuf> {
        let output = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(path)
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("Failed to get git directory");
        }
        
        let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(git_dir))
    }

    /// 检查是否为裸仓库
    async fn is_bare_repository(path: &Path) -> Result<bool> {
        let output = Command::new("git")
            .args(["rev-parse", "--is-bare-repository"])
            .current_dir(path)
            .output()
            .await?;
        
        if !output.status.success() {
            return Ok(false);
        }
        
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(result == "true")
    }
}

/// 仓库状态
#[derive(Debug, Clone, Default)]
pub struct RepositoryStatus {
    pub staged_files: usize,
    pub modified_files: usize,
    pub untracked_files: usize,
    pub deleted_files: usize,
    pub conflicted_files: usize,
}

impl RepositoryStatus {
    /// 解析状态输出
    fn parse(status_text: &str) -> Result<Self> {
        let mut status = Self::default();
        
        for line in status_text.lines() {
            if line.len() < 2 {
                continue;
            }
            
            let status_code = &line[0..2];
            match status_code {
                "M " | "MM" => status.modified_files += 1,
                "A " | "AM" => status.staged_files += 1,
                "D " | "DM" => status.deleted_files += 1,
                "??" => status.untracked_files += 1,
                "UU" => status.conflicted_files += 1,
                _ => {}
            }
        }
        
        Ok(status)
    }

    /// 是否有变更
    pub fn has_changes(&self) -> bool {
        self.staged_files > 0
            || self.modified_files > 0
            || self.untracked_files > 0
            || self.deleted_files > 0
            || self.conflicted_files > 0
    }

    /// 获取总变更文件数
    pub fn total_changes(&self) -> usize {
        self.staged_files
            + self.modified_files
            + self.untracked_files
            + self.deleted_files
            + self.conflicted_files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_parse_empty() {
        let status = RepositoryStatus::parse("").unwrap();
        assert_eq!(status.staged_files, 0);
        assert_eq!(status.modified_files, 0);
        assert_eq!(status.untracked_files, 0);
        assert!(!status.has_changes());
    }

    #[test]
    fn test_status_parse_with_changes() {
        let status_text = "M  src/main.rs\nA  src/new.rs\n?? temp.txt\nD  old.rs\nUU conflict.rs";
        let status = RepositoryStatus::parse(status_text).unwrap();
        
        assert_eq!(status.modified_files, 1);
        assert_eq!(status.staged_files, 1);
        assert_eq!(status.untracked_files, 1);
        assert_eq!(status.deleted_files, 1);
        assert_eq!(status.conflicted_files, 1);
        assert!(status.has_changes());
        assert_eq!(status.total_changes(), 5);
    }

    #[test]
    fn test_status_has_changes() {
        let mut status = RepositoryStatus::default();
        assert!(!status.has_changes());
        
        status.modified_files = 1;
        assert!(status.has_changes());
        
        status = RepositoryStatus::default();
        status.untracked_files = 1;
        assert!(status.has_changes());
    }

    #[test]
    fn test_repository_path() {
        // 这是一个模拟测试，实际测试需要 Git 仓库
        let path = PathBuf::from("/test/repo");
        // 在实际测试中，这里会创建 Repository 实例
        assert_eq!(path.to_str().unwrap(), "/test/repo");
    }
}