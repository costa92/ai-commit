// Git models - placeholder implementations

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub files_changed: u32,
}

impl Commit {
    pub fn new(hash: String, message: String, author: String, date: String) -> Self {
        Self {
            hash,
            message,
            author,
            date,
            files_changed: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub upstream: Option<String>,
}

impl Branch {
    pub fn new(name: String, is_current: bool) -> Self {
        Self {
            name,
            is_current,
            upstream: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub commit_hash: String,
    pub message: Option<String>,
}

impl Tag {
    pub fn new(name: String, commit_hash: String) -> Self {
        Self {
            name,
            commit_hash,
            message: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

impl Remote {
    pub fn new(name: String, url: String) -> Self {
        Self { name, url }
    }
}

#[derive(Debug, Clone)]
pub struct Stash {
    pub index: u32,
    pub message: String,
    pub branch: String,
}

impl Stash {
    pub fn new(index: u32, message: String, branch: String) -> Self {
        Self {
            index,
            message,
            branch,
        }
    }
}