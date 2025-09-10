use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct GitRepoState {
    pub current_branch: String,
    pub repo_path: PathBuf,
    pub repo_name: String,
    pub status: RepoStatus,
    pub remotes: Vec<Remote>,
    pub branches: Vec<Branch>,
    pub tags: Vec<Tag>,
    pub commits: Vec<Commit>,
    pub stashes: Vec<Stash>,
    pub last_refresh: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct RepoStatus {
    pub staged_files: Vec<FileStatus>,
    pub unstaged_files: Vec<FileStatus>,
    pub untracked_files: Vec<PathBuf>,
    pub conflicts: Vec<PathBuf>,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub is_clean: bool,
    pub is_detached: bool,
}

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: PathBuf,
    pub status: ChangeType,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
    Copied { from: PathBuf },
    Unmerged,
    TypeChange,
}

#[derive(Debug, Clone)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub fetch_url: String,
    pub push_url: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub full_name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub last_commit: Option<String>,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub commit_hash: String,
    pub message: Option<String>,
    pub tagger: Option<String>,
    pub date: DateTime<Utc>,
    pub is_annotated: bool,
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub author_email: String,
    pub committer: String,
    pub committer_email: String,
    pub date: DateTime<Utc>,
    pub message: String,
    pub subject: String,
    pub body: Option<String>,
    pub parents: Vec<String>,
    pub refs: Vec<String>,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone)]
pub struct Stash {
    pub index: usize,
    pub hash: String,
    pub branch: String,
    pub message: String,
    pub date: DateTime<Utc>,
    pub files_changed: usize,
}

impl GitRepoState {
    pub fn new(repo_path: PathBuf) -> Self {
        let repo_name = repo_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Self {
            current_branch: String::new(),
            repo_path,
            repo_name,
            status: RepoStatus::default(),
            remotes: Vec::new(),
            branches: Vec::new(),
            tags: Vec::new(),
            commits: Vec::new(),
            stashes: Vec::new(),
            last_refresh: Utc::now(),
        }
    }

    pub fn update_current_branch(&mut self, branch: String) {
        self.current_branch = branch;
        self.last_refresh = Utc::now();
    }

    pub fn update_status(&mut self, status: RepoStatus) {
        self.status = status;
        self.last_refresh = Utc::now();
    }

    pub fn update_commits(&mut self, commits: Vec<Commit>) {
        self.commits = commits;
        self.last_refresh = Utc::now();
    }

    pub fn update_branches(&mut self, branches: Vec<Branch>) {
        self.branches = branches;
        self.last_refresh = Utc::now();
    }

    pub fn update_tags(&mut self, tags: Vec<Tag>) {
        self.tags = tags;
        self.last_refresh = Utc::now();
    }

    pub fn update_remotes(&mut self, remotes: Vec<Remote>) {
        self.remotes = remotes;
        self.last_refresh = Utc::now();
    }

    pub fn update_stashes(&mut self, stashes: Vec<Stash>) {
        self.stashes = stashes;
        self.last_refresh = Utc::now();
    }

    pub fn get_current_branch(&self) -> &str {
        &self.current_branch
    }

    pub fn is_dirty(&self) -> bool {
        !self.status.is_clean
    }

    pub fn has_conflicts(&self) -> bool {
        !self.status.conflicts.is_empty()
    }

    pub fn get_branch_by_name(&self, name: &str) -> Option<&Branch> {
        self.branches.iter().find(|b| b.name == name)
    }

    pub fn get_commit_by_hash(&self, hash: &str) -> Option<&Commit> {
        self.commits.iter().find(|c| c.hash.starts_with(hash) || c.short_hash == hash)
    }

    pub fn get_tag_by_name(&self, name: &str) -> Option<&Tag> {
        self.tags.iter().find(|t| t.name == name)
    }

    pub fn get_remote_by_name(&self, name: &str) -> Option<&Remote> {
        self.remotes.iter().find(|r| r.name == name)
    }

    pub fn get_stash_by_index(&self, index: usize) -> Option<&Stash> {
        self.stashes.iter().find(|s| s.index == index)
    }

    pub fn needs_refresh(&self, max_age_seconds: i64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.last_refresh);
        age.num_seconds() > max_age_seconds
    }

    pub fn get_total_file_changes(&self) -> usize {
        self.status.staged_files.len() + self.status.unstaged_files.len() + self.status.untracked_files.len()
    }

    pub fn get_repo_summary(&self) -> RepoSummary {
        RepoSummary {
            name: self.repo_name.clone(),
            current_branch: self.current_branch.clone(),
            total_commits: self.commits.len(),
            total_branches: self.branches.len(),
            total_tags: self.tags.len(),
            total_remotes: self.remotes.len(),
            total_stashes: self.stashes.len(),
            pending_changes: self.get_total_file_changes(),
            is_dirty: self.is_dirty(),
            has_conflicts: self.has_conflicts(),
            ahead_count: self.status.ahead_count,
            behind_count: self.status.behind_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepoSummary {
    pub name: String,
    pub current_branch: String,
    pub total_commits: usize,
    pub total_branches: usize,
    pub total_tags: usize,
    pub total_remotes: usize,
    pub total_stashes: usize,
    pub pending_changes: usize,
    pub is_dirty: bool,
    pub has_conflicts: bool,
    pub ahead_count: usize,
    pub behind_count: usize,
}

impl Default for GitRepoState {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}