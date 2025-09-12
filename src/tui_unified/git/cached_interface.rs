// Cached Git interface implementation - Task 2.3: Caching Integration

use super::interface::GitRepositoryAPI;
use super::models::*;
use crate::tui_unified::cache::CacheManager;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

pub struct CachedGitInterface {
    git_impl: Box<dyn GitRepositoryAPI + Send + Sync>,
    cache_manager: Arc<CacheManager>,
}

impl CachedGitInterface {
    pub fn new(
        git_impl: Box<dyn GitRepositoryAPI + Send + Sync>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        Self {
            git_impl,
            cache_manager,
        }
    }

    pub async fn start_background_refresh(&self) {
        self.cache_manager
            .start_background_refresh(Duration::from_secs(60))
            .await;
    }

    // Helper method to generate cache key
    fn generate_cache_key(prefix: &str, params: &[&str]) -> String {
        format!("{}:{}", prefix, params.join(":"))
    }
}

#[async_trait]
impl GitRepositoryAPI for CachedGitInterface {
    async fn get_commits(
        &self,
        limit: Option<u32>,
    ) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
        let cache_key = Self::generate_cache_key("commits", &[&limit.unwrap_or(50).to_string()]);
        let git_cache = self.cache_manager.get_git_cache().await;

        // Try to get from cache first
        {
            let mut cache = git_cache.write().await;
            if let Some(cached_commits) = cache.get_commits(&cache_key) {
                return Ok(cached_commits.clone());
            }
        }

        // If not in cache, fetch from git and cache the result
        let commits = self.git_impl.get_commits(limit).await?;

        {
            let mut cache = git_cache.write().await;
            cache.cache_commits(cache_key, commits.clone());
        }

        Ok(commits)
    }

    async fn get_branches(&self) -> Result<Vec<Branch>, Box<dyn std::error::Error>> {
        let git_cache = self.cache_manager.get_git_cache().await;

        // Try to get from cache first
        {
            let cache = git_cache.read().await;
            if let Some(cached_branches) = cache.get_branches() {
                return Ok(cached_branches.clone());
            }
        }

        // If not in cache, fetch from git and cache the result
        let branches = self.git_impl.get_branches().await?;

        {
            let mut cache = git_cache.write().await;
            cache.cache_branches(branches.clone());
        }

        Ok(branches)
    }

    async fn get_current_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Current branch changes frequently, so don't cache it
        self.git_impl.get_current_branch().await
    }

    async fn switch_branch(&self, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.git_impl.switch_branch(branch).await?;

        // Clear relevant caches when switching branches
        let git_cache = self.cache_manager.get_git_cache().await;
        let mut cache = git_cache.write().await;
        cache.clear();

        Ok(())
    }

    async fn get_status(&self) -> Result<String, Box<dyn std::error::Error>> {
        let git_cache = self.cache_manager.get_git_cache().await;

        // Try to get from cache first
        {
            let cache = git_cache.read().await;
            if let Some(cached_status) = cache.get_status() {
                return Ok(cached_status.clone());
            }
        }

        // If not in cache, fetch from git and cache the result
        let status = self.git_impl.get_status().await?;

        {
            let mut cache = git_cache.write().await;
            cache.cache_status(status.clone());
        }

        Ok(status)
    }

    async fn get_diff(
        &self,
        commit_hash: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let cache_key = Self::generate_cache_key("diff", &[commit_hash.unwrap_or("HEAD")]);
        let file_cache = self.cache_manager.get_file_cache().await;

        // Try to get from cache first
        {
            let mut cache = file_cache.write().await;
            if let Some(cached_diff) = cache.get_diff(&cache_key) {
                return Ok(cached_diff.clone());
            }
        }

        // If not in cache, fetch from git and cache the result
        let diff = self.git_impl.get_diff(commit_hash).await?;

        {
            let mut cache = file_cache.write().await;
            cache.cache_diff(cache_key, diff.clone());
        }

        Ok(diff)
    }

    async fn get_commit_diff(
        &self,
        commit_hash: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let cache_key = Self::generate_cache_key("commit_diff", &[commit_hash]);
        let file_cache = self.cache_manager.get_file_cache().await;

        // Try to get from cache first
        {
            let mut cache = file_cache.write().await;
            if let Some(cached_diff) = cache.get_diff(&cache_key) {
                return Ok(cached_diff.clone());
            }
        }

        // If not in cache, fetch from git and cache the result
        let diff = self.git_impl.get_commit_diff(commit_hash).await?;

        {
            let mut cache = file_cache.write().await;
            cache.cache_diff(cache_key, diff.clone());
        }

        Ok(diff)
    }

    async fn get_file_diff(
        &self,
        file_path: &str,
        commit_hash: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let cache_key =
            Self::generate_cache_key("file_diff", &[file_path, commit_hash.unwrap_or("HEAD")]);
        let file_cache = self.cache_manager.get_file_cache().await;

        // Try to get from cache first
        {
            let mut cache = file_cache.write().await;
            if let Some(cached_diff) = cache.get_diff(&cache_key) {
                return Ok(cached_diff.clone());
            }
        }

        // If not in cache, fetch from git and cache the result
        let diff = self.git_impl.get_file_diff(file_path, commit_hash).await?;

        {
            let mut cache = file_cache.write().await;
            cache.cache_diff(cache_key, diff.clone());
        }

        Ok(diff)
    }

    async fn get_tags(&self) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
        // Tags don't change frequently, so this is a good candidate for caching
        self.git_impl.get_tags().await
    }

    async fn get_remotes(&self) -> Result<Vec<Remote>, Box<dyn std::error::Error>> {
        // Remotes don't change frequently, so this is a good candidate for caching
        self.git_impl.get_remotes().await
    }

    async fn get_stashes(&self) -> Result<Vec<Stash>, Box<dyn std::error::Error>> {
        // Stashes can change but not too frequently
        self.git_impl.get_stashes().await
    }

    // Task 2.1: Git Operations - just pass through, clearing cache when needed
    async fn create_branch(
        &self,
        branch_name: &str,
        from_branch: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.git_impl
            .create_branch(branch_name, from_branch)
            .await?;

        // Clear branches cache when creating a new branch
        let git_cache = self.cache_manager.get_git_cache().await;
        let mut cache = git_cache.write().await;
        cache.cache_branches(vec![]); // This will force a refresh next time

        Ok(())
    }

    async fn delete_branch(
        &self,
        branch_name: &str,
        force: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.git_impl.delete_branch(branch_name, force).await?;

        // Clear branches cache when deleting a branch
        let git_cache = self.cache_manager.get_git_cache().await;
        let mut cache = git_cache.write().await;
        cache.cache_branches(vec![]); // This will force a refresh next time

        Ok(())
    }

    async fn stage_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.git_impl.stage_file(file_path).await?;

        // Clear status cache when staging files
        let git_cache = self.cache_manager.get_git_cache().await;
        let mut cache = git_cache.write().await;
        cache.cache_status("".to_string()); // This will force a refresh next time

        Ok(())
    }

    async fn unstage_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.git_impl.unstage_file(file_path).await?;

        // Clear status cache when unstaging files
        let git_cache = self.cache_manager.get_git_cache().await;
        let mut cache = git_cache.write().await;
        cache.cache_status("".to_string()); // This will force a refresh next time

        Ok(())
    }

    async fn stage_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.git_impl.stage_all().await?;

        // Clear status cache when staging all files
        let git_cache = self.cache_manager.get_git_cache().await;
        let mut cache = git_cache.write().await;
        cache.cache_status("".to_string()); // This will force a refresh next time

        Ok(())
    }

    async fn get_working_tree_status(&self) -> Result<Vec<FileStatus>, Box<dyn std::error::Error>> {
        // File status changes frequently, so we might cache it for a short time
        self.git_impl.get_working_tree_status().await
    }

    // Task 2.2: Search and Filtering - with caching
    async fn search_commits(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
        let git_cache = self.cache_manager.get_git_cache().await;

        // Try to get from cache first
        {
            let mut cache = git_cache.write().await;
            if let Some(cached_results) = cache.get_search_result(query) {
                return Ok(cached_results.clone());
            }
        }

        // If not in cache, fetch from git and cache the result
        let results = self.git_impl.search_commits(query, limit).await?;

        {
            let mut cache = git_cache.write().await;
            cache.cache_search_result(query.to_string(), results.clone());
        }

        Ok(results)
    }

    async fn filter_commits_by_author(
        &self,
        author: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
        let git_cache = self.cache_manager.get_git_cache().await;

        // Try to get from cache first
        {
            let mut cache = git_cache.write().await;
            if let Some(cached_results) = cache.get_author_filter_result(author) {
                return Ok(cached_results.clone());
            }
        }

        // If not in cache, fetch from git and cache the result
        let results = self
            .git_impl
            .filter_commits_by_author(author, limit)
            .await?;

        {
            let mut cache = git_cache.write().await;
            cache.cache_author_filter_result(author.to_string(), results.clone());
        }

        Ok(results)
    }

    async fn filter_commits_by_date_range(
        &self,
        since: &str,
        until: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
        // Date range filters are usually not repeated often, so we just pass through
        self.git_impl
            .filter_commits_by_date_range(since, until, limit)
            .await
    }

    async fn search_branches(
        &self,
        query: &str,
    ) -> Result<Vec<Branch>, Box<dyn std::error::Error>> {
        // Branch search is implemented locally, so it uses the cached branches
        let branches = self.get_branches().await?;
        let filtered_branches = branches
            .into_iter()
            .filter(|branch| branch.name.to_lowercase().contains(&query.to_lowercase()))
            .collect();
        Ok(filtered_branches)
    }
}
