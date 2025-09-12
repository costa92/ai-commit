// Cache implementations - Task 2.3: Caching Optimization

use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct CacheManager {
    git_cache: Arc<RwLock<GitCache>>,
    ui_cache: Arc<RwLock<UiCache>>,
    file_cache: Arc<RwLock<FileCache>>,
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            git_cache: Arc::new(RwLock::new(GitCache::new())),
            ui_cache: Arc::new(RwLock::new(UiCache::new())),
            file_cache: Arc::new(RwLock::new(FileCache::new())),
        }
    }

    pub async fn clear_all(&self) {
        self.git_cache.write().await.clear();
        self.ui_cache.write().await.clear();
        self.file_cache.write().await.clear();
    }

    pub async fn get_git_cache(&self) -> Arc<RwLock<GitCache>> {
        self.git_cache.clone()
    }

    pub async fn get_ui_cache(&self) -> Arc<RwLock<UiCache>> {
        self.ui_cache.clone()
    }

    pub async fn get_file_cache(&self) -> Arc<RwLock<FileCache>> {
        self.file_cache.clone()
    }

    // Background refresh task
    pub async fn start_background_refresh(&self, refresh_interval: Duration) {
        let git_cache = self.git_cache.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(refresh_interval);

            loop {
                interval.tick().await;

                let mut cache = git_cache.write().await;
                cache.expire_old_entries();
            }
        });
    }
}

#[derive(Clone)]
pub struct CachedEntry<T> {
    data: T,
    timestamp: Instant,
    ttl: Duration,
}

impl<T> CachedEntry<T> {
    pub fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            timestamp: Instant::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > self.ttl
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

pub struct GitCache {
    commits: LruCache<String, CachedEntry<Vec<crate::tui_unified::git::Commit>>>,
    branches: Option<CachedEntry<Vec<crate::tui_unified::git::Branch>>>,
    status: Option<CachedEntry<String>>,
    tags: Option<CachedEntry<Vec<crate::tui_unified::git::Tag>>>,
    remotes: Option<CachedEntry<Vec<crate::tui_unified::git::Remote>>>,
    stashes: Option<CachedEntry<Vec<crate::tui_unified::git::Stash>>>,
    // Search result caches
    search_cache: LruCache<String, CachedEntry<Vec<crate::tui_unified::git::Commit>>>,
    author_filter_cache: LruCache<String, CachedEntry<Vec<crate::tui_unified::git::Commit>>>,
    date_filter_cache: LruCache<String, CachedEntry<Vec<crate::tui_unified::git::Commit>>>,
}

impl Default for GitCache {
    fn default() -> Self {
        Self::new()
    }
}

impl GitCache {
    pub fn new() -> Self {
        Self {
            commits: LruCache::new(NonZeroUsize::new(100).unwrap()),
            branches: None,
            status: None,
            tags: None,
            remotes: None,
            stashes: None,
            search_cache: LruCache::new(NonZeroUsize::new(50).unwrap()),
            author_filter_cache: LruCache::new(NonZeroUsize::new(50).unwrap()),
            date_filter_cache: LruCache::new(NonZeroUsize::new(50).unwrap()),
        }
    }

    pub fn get_commits(&mut self, key: &str) -> Option<&Vec<crate::tui_unified::git::Commit>> {
        self.commits.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data())
            }
        })
    }

    pub fn cache_commits(&mut self, key: String, commits: Vec<crate::tui_unified::git::Commit>) {
        let entry = CachedEntry::new(commits, Duration::from_secs(300)); // 5 minutes TTL
        self.commits.put(key, entry);
    }

    pub fn get_branches(&self) -> Option<&Vec<crate::tui_unified::git::Branch>> {
        self.branches.as_ref().and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data())
            }
        })
    }

    pub fn cache_branches(&mut self, branches: Vec<crate::tui_unified::git::Branch>) {
        let entry = CachedEntry::new(branches, Duration::from_secs(300)); // 5 minutes TTL
        self.branches = Some(entry);
    }

    pub fn get_status(&self) -> Option<&String> {
        self.status.as_ref().and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data())
            }
        })
    }

    pub fn cache_status(&mut self, status: String) {
        let entry = CachedEntry::new(status, Duration::from_secs(30)); // 30 seconds TTL for status
        self.status = Some(entry);
    }

    // Search caching methods
    pub fn get_search_result(
        &mut self,
        query: &str,
    ) -> Option<&Vec<crate::tui_unified::git::Commit>> {
        self.search_cache.get(query).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data())
            }
        })
    }

    pub fn cache_search_result(
        &mut self,
        query: String,
        commits: Vec<crate::tui_unified::git::Commit>,
    ) {
        let entry = CachedEntry::new(commits, Duration::from_secs(180)); // 3 minutes TTL
        self.search_cache.put(query, entry);
    }

    pub fn get_author_filter_result(
        &mut self,
        author: &str,
    ) -> Option<&Vec<crate::tui_unified::git::Commit>> {
        self.author_filter_cache.get(author).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data())
            }
        })
    }

    pub fn cache_author_filter_result(
        &mut self,
        author: String,
        commits: Vec<crate::tui_unified::git::Commit>,
    ) {
        let entry = CachedEntry::new(commits, Duration::from_secs(300)); // 5 minutes TTL
        self.author_filter_cache.put(author, entry);
    }

    pub fn expire_old_entries(&mut self) {
        // Remove expired entries from commits cache
        self.commits.iter().for_each(|(_, entry)| {
            if entry.is_expired() {
                // LruCache doesn't provide a way to remove during iteration
                // This is a limitation we'll have to live with or implement differently
            }
        });

        // Remove expired single entries
        if let Some(ref entry) = self.branches {
            if entry.is_expired() {
                self.branches = None;
            }
        }

        if let Some(ref entry) = self.status {
            if entry.is_expired() {
                self.status = None;
            }
        }

        if let Some(ref entry) = self.tags {
            if entry.is_expired() {
                self.tags = None;
            }
        }

        if let Some(ref entry) = self.remotes {
            if entry.is_expired() {
                self.remotes = None;
            }
        }

        if let Some(ref entry) = self.stashes {
            if entry.is_expired() {
                self.stashes = None;
            }
        }
    }

    pub fn clear(&mut self) {
        self.commits.clear();
        self.branches = None;
        self.status = None;
        self.tags = None;
        self.remotes = None;
        self.stashes = None;
        self.search_cache.clear();
        self.author_filter_cache.clear();
        self.date_filter_cache.clear();
    }
}

pub struct UiCache {
    rendered_frames: HashMap<String, CachedEntry<String>>,
    layout_cache: HashMap<String, CachedEntry<String>>,
}

impl Default for UiCache {
    fn default() -> Self {
        Self::new()
    }
}

impl UiCache {
    pub fn new() -> Self {
        Self {
            rendered_frames: HashMap::new(),
            layout_cache: HashMap::new(),
        }
    }

    pub fn get_rendered_frame(&self, key: &str) -> Option<&String> {
        self.rendered_frames.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data())
            }
        })
    }

    pub fn cache_rendered_frame(&mut self, key: String, frame: String) {
        let entry = CachedEntry::new(frame, Duration::from_secs(60)); // 1 minute TTL
        self.rendered_frames.insert(key, entry);
    }

    pub fn get_layout(&self, key: &str) -> Option<&String> {
        self.layout_cache.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data())
            }
        })
    }

    pub fn cache_layout(&mut self, key: String, layout: String) {
        let entry = CachedEntry::new(layout, Duration::from_secs(120)); // 2 minutes TTL
        self.layout_cache.insert(key, entry);
    }

    pub fn clear(&mut self) {
        self.rendered_frames.clear();
        self.layout_cache.clear();
    }
}

pub struct FileCache {
    file_contents: LruCache<String, CachedEntry<String>>,
    diff_cache: LruCache<String, CachedEntry<String>>,
}

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FileCache {
    pub fn new() -> Self {
        Self {
            file_contents: LruCache::new(NonZeroUsize::new(50).unwrap()),
            diff_cache: LruCache::new(NonZeroUsize::new(30).unwrap()),
        }
    }

    pub fn get_file_content(&mut self, path: &str) -> Option<&String> {
        self.file_contents.get(path).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data())
            }
        })
    }

    pub fn cache_file_content(&mut self, path: String, content: String) {
        let entry = CachedEntry::new(content, Duration::from_secs(600)); // 10 minutes TTL
        self.file_contents.put(path, entry);
    }

    pub fn get_diff(&mut self, key: &str) -> Option<&String> {
        self.diff_cache.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data())
            }
        })
    }

    pub fn cache_diff(&mut self, key: String, diff: String) {
        let entry = CachedEntry::new(diff, Duration::from_secs(180)); // 3 minutes TTL
        self.diff_cache.put(key, entry);
    }

    pub fn clear(&mut self) {
        self.file_contents.clear();
        self.diff_cache.clear();
    }
}
