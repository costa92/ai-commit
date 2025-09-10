// Git interface - real Git command implementations
use async_trait::async_trait;
use tokio::process::Command;
use super::models::*;

#[async_trait]
pub trait GitRepositoryAPI {
    async fn get_commits(&self, limit: Option<u32>) -> Result<Vec<Commit>, Box<dyn std::error::Error>>;
    async fn get_branches(&self) -> Result<Vec<Branch>, Box<dyn std::error::Error>>;
    async fn get_current_branch(&self) -> Result<String, Box<dyn std::error::Error>>;
    async fn switch_branch(&self, branch: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_status(&self) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_diff(&self, commit_hash: Option<&str>) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_commit_diff(&self, commit_hash: &str) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_file_diff(&self, file_path: &str, commit_hash: Option<&str>) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_tags(&self) -> Result<Vec<Tag>, Box<dyn std::error::Error>>;
    async fn get_remotes(&self) -> Result<Vec<Remote>, Box<dyn std::error::Error>>;
    async fn get_stashes(&self) -> Result<Vec<Stash>, Box<dyn std::error::Error>>;
}

pub struct AsyncGitImpl {
    pub repo_path: std::path::PathBuf,
}

impl AsyncGitImpl {
    pub fn new(repo_path: std::path::PathBuf) -> Self {
        Self { repo_path }
    }

    // Helper method to get file change count for a commit
    async fn get_commit_files_changed(&self, hash: &str) -> Result<u32, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["show", "--name-only", "--format=", hash])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(0);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let file_count = output_str.lines()
            .filter(|line| !line.trim().is_empty())
            .count();

        Ok(file_count as u32)
    }

    // Helper method to get detailed commit statistics (files changed, insertions, deletions)
    async fn get_commit_stats(&self, hash: &str) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["show", "--numstat", "--format=", hash])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            return Ok((0, 0, 0));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut files_changed = 0;
        let mut total_insertions = 0;
        let mut total_deletions = 0;

        for line in output_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                files_changed += 1;
                
                // Parse insertions and deletions
                if let Ok(insertions) = parts[0].parse::<usize>() {
                    total_insertions += insertions;
                }
                if let Ok(deletions) = parts[1].parse::<usize>() {
                    total_deletions += deletions;
                }
            }
        }

        Ok((files_changed, total_insertions, total_deletions))
    }
}

#[async_trait]
impl GitRepositoryAPI for AsyncGitImpl {
    async fn get_commits(&self, limit: Option<u32>) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
        let limit_arg = limit.unwrap_or(50).to_string();
        let output = Command::new("git")
            .args([
                "log",
                "--pretty=format:%H|%h|%an|%ae|%cn|%ce|%ad|%s|%b|%P|%D",
                "--date=iso-strict",
                "--stat=1,1", // Add minimal stat info for file counts
                "-n",
                &limit_arg
            ])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git log command failed: {}", stderr).into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for line in output_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 9 {
                let hash = parts[0].trim().to_string();
                let _short_hash = parts[1].trim().to_string(); // For future use
                let author = parts[2].trim().to_string();
                let _author_email = parts[3].trim().to_string(); // For future use
                let _committer = parts[4].trim().to_string(); // For future use
                let _committer_email = parts[5].trim().to_string(); // For future use
                let date = parts[6].trim().to_string();
                let subject = parts[7].trim().to_string();
                let _body = if parts.len() > 8 && !parts[8].trim().is_empty() {
                    Some(parts[8].trim().to_string())
                } else {
                    None
                }; // For future use
                let _parents: Vec<String> = if parts.len() > 9 && !parts[9].trim().is_empty() {
                    parts[9].split_whitespace().map(|s| s.to_string()).collect()
                } else {
                    Vec::new()
                }; // For future use
                let _refs: Vec<String> = if parts.len() > 10 && !parts[10].trim().is_empty() {
                    parts[10].split(", ").map(|s| s.trim().to_string()).collect()
                } else {
                    Vec::new()
                }; // For future use

                // Get detailed file stats for this commit
                let (files_changed, _insertions, _deletions) = self.get_commit_stats(&hash).await.unwrap_or((0, 0, 0));

                commits.push(Commit {
                    hash,
                    message: subject.clone(),
                    author,
                    date,
                    files_changed: files_changed as u32,
                });
            }
        }

        Ok(commits)
    }
    
    async fn get_branches(&self) -> Result<Vec<Branch>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["branch", "-vv", "--color=never"])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git branch command failed: {}", stderr).into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut branches = Vec::new();

        for line in output_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let is_current = line.starts_with('*');
            let line = line.trim_start_matches('*').trim();
            
            // Parse format: "branch_name commit_hash [upstream: ahead X, behind Y] commit_message"
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let name = parts[0].trim().to_string();
                let _commit_hash = parts[1].trim().to_string(); // For future use
                
                let mut upstream = None;
                let mut _ahead_count = 0; // For future use  
                let mut _behind_count = 0; // For future use

                // Extract upstream info if present
                if let Some(bracket_start) = line.find('[') {
                    if let Some(bracket_end) = line.find(']') {
                        let upstream_info = &line[bracket_start+1..bracket_end];
                        
                        // Parse upstream format: "origin/main: ahead 2, behind 1" or "origin/main"
                        if let Some(colon_pos) = upstream_info.find(':') {
                            upstream = Some(upstream_info[..colon_pos].trim().to_string());
                            
                            let status_info = &upstream_info[colon_pos+1..];
                            
                            // Parse ahead/behind counts
                            if let Some(ahead_start) = status_info.find("ahead ") {
                                let ahead_str = &status_info[ahead_start + 6..];
                                if let Some(next_comma_or_end) = ahead_str.find(&[',', ']'][..]).or(Some(ahead_str.len())) {
                                    if let Ok(count) = ahead_str[..next_comma_or_end].trim().parse::<usize>() {
                                        _ahead_count = count;
                                    }
                                }
                            }
                            
                            if let Some(behind_start) = status_info.find("behind ") {
                                let behind_str = &status_info[behind_start + 7..];
                                if let Some(next_comma_or_end) = behind_str.find(&[',', ']'][..]).or(Some(behind_str.len())) {
                                    if let Ok(count) = behind_str[..next_comma_or_end].trim().parse::<usize>() {
                                        _behind_count = count;
                                    }
                                }
                            }
                        } else {
                            upstream = Some(upstream_info.trim().to_string());
                        }
                    }
                }

                branches.push(Branch {
                    name,
                    is_current,
                    upstream,
                });
            }
        }

        Ok(branches)
    }
    
    async fn get_current_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git rev-parse command failed: {}", stderr).into());
        }

        let branch_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(branch_name)
    }
    
    async fn switch_branch(&self, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["checkout", branch])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git checkout command failed: {}", stderr).into());
        }

        Ok(())
    }
    
    async fn get_status(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git status command failed: {}", stderr).into());
        }

        let status_output = String::from_utf8_lossy(&output.stdout);
        if status_output.trim().is_empty() {
            Ok("Working tree clean".to_string())
        } else {
            // Format the porcelain output into a more readable format
            let mut formatted_status = String::new();
            for line in status_output.lines() {
                if line.len() >= 3 {
                    let status_code = &line[..2];
                    let file_path = &line[3..];
                    
                    let status_text = match status_code {
                        " M" => "Modified",
                        " A" => "Added", 
                        " D" => "Deleted",
                        "M " => "Modified (staged)",
                        "A " => "Added (staged)",
                        "D " => "Deleted (staged)",
                        "??" => "Untracked",
                        "MM" => "Modified (staged and unstaged)",
                        _ => "Unknown status",
                    };
                    
                    formatted_status.push_str(&format!("{}: {}\n", status_text, file_path));
                }
            }
            Ok(formatted_status)
        }
    }
    
    async fn get_diff(&self, commit_hash: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let mut args = vec!["diff"];
        let parent_hash;
        
        if let Some(hash) = commit_hash {
            // 显示特定提交相对于其父提交的差异
            parent_hash = format!("{}^", hash);
            args.push(&parent_hash);
            args.push(hash);
        }
        // 如果没有commit_hash，显示工作区相对于HEAD的差异（默认行为）

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .await?;

        // Git diff 在有差异时返回状态码1，这是正常的
        if !output.status.success() && output.status.code() != Some(1) {
            return Err(format!("Git diff failed with status: {:?}", output.status).into());
        }

        let diff_content = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(diff_content)
    }

    async fn get_commit_diff(&self, commit_hash: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 获取特定提交的完整差异
        let output = Command::new("git")
            .args(["show", "--format=", commit_hash])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(format!("Git show failed for commit {}: {:?}", commit_hash, output.status).into());
        }

        let diff_content = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(diff_content)
    }

    async fn get_file_diff(&self, file_path: &str, commit_hash: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let mut args = vec!["diff"];
        let parent_hash;
        
        if let Some(hash) = commit_hash {
            parent_hash = format!("{}^", hash);
            args.push(&parent_hash);
            args.push(hash);
        }
        
        args.push("--");
        args.push(file_path);

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .await?;

        // Git diff 在有差异时返回状态码1，这是正常的
        if !output.status.success() && output.status.code() != Some(1) {
            return Err(format!("Git file diff failed for {}: {:?}", file_path, output.status).into());
        }

        let diff_content = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(diff_content)
    }

    async fn get_tags(&self) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["tag", "-l", "--format=%(refname:short)|%(objectname:short)|%(creatordate:iso-strict)|%(subject)|%(taggername)", "--sort=-creatordate"])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git tag command failed: {}", stderr).into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut tags = Vec::new();

        for line in output_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                let name = parts[0].trim().to_string();
                let commit_hash = parts[1].trim().to_string();
                let message = if parts.len() > 3 && !parts[3].trim().is_empty() {
                    Some(parts[3].trim().to_string())
                } else {
                    None
                };

                tags.push(Tag {
                    name,
                    commit_hash,
                    message,
                });
            }
        }

        Ok(tags)
    }

    async fn get_remotes(&self) -> Result<Vec<Remote>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["remote", "-v"])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git remote command failed: {}", stderr).into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut remotes = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        for line in output_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[0].trim().to_string();
                let url = parts[1].trim().to_string();
                let direction = parts[2].trim();

                // Only add each remote name once (prefer fetch URL)
                if direction == "(fetch)" && !seen_names.contains(&name) {
                    seen_names.insert(name.clone());
                    remotes.push(Remote {
                        name,
                        url,
                    });
                }
            }
        }

        Ok(remotes)
    }

    async fn get_stashes(&self) -> Result<Vec<Stash>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["stash", "list", "--format=%gd|%H|%gs|%gD"])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git stash command failed: {}", stderr).into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut stashes = Vec::new();

        for line in output_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                // Extract index from stash@{N} format
                let stash_ref = parts[0].trim();
                let index = if let Some(start) = stash_ref.find('{') {
                    if let Some(end) = stash_ref.find('}') {
                        stash_ref[start+1..end].parse::<usize>().unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };

                let _hash = parts[1].trim().to_string(); // Keep for future use
                let message = parts[2].trim().to_string();
                
                // Extract branch from message if available
                let branch = if message.starts_with("WIP on ") {
                    message.split_whitespace().nth(2).unwrap_or("unknown").to_string()
                } else {
                    "unknown".to_string()
                };

                stashes.push(Stash {
                    index: index as u32,
                    message,
                    branch,
                });
            }
        }

        Ok(stashes)
    }
}