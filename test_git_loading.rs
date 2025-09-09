use anyhow::Result;
use chrono::{DateTime, Local};
use tokio::process::Command;

#[derive(Debug)]
struct GitCommit {
    hash: String,
    message: String,
    author: String,
    timestamp: DateTime<Local>,
    refs: String,
}

async fn load_git_commits() -> Result<Vec<GitCommit>> {
    println!("Starting to load git commits...");
    
    let output = Command::new("git")
        .args([
            "log",
            "--pretty=format:%H|%s|%an|%ai|%D",
            "-n", "10"
        ])
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Git log failed: {:?}", output.status.code());
    }

    let log_output = String::from_utf8_lossy(&output.stdout);
    println!("Git log output ({} bytes):", log_output.len());
    
    let mut commits = Vec::new();
    
    for (i, line) in log_output.lines().enumerate() {
        println!("Line {}: {}", i + 1, line);
        
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(5, '|').collect();
        println!("  Parts count: {}", parts.len());
        
        if parts.len() >= 4 {
            let hash = parts[0].to_string();
            let message = parts[1].to_string();
            let author = parts[2].to_string();
            let timestamp_str = parts[3];
            let refs = parts.get(4).unwrap_or(&"").to_string();
            
            println!("  Parsing timestamp: '{}'", timestamp_str);
            
            match DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S %z") {
                Ok(dt) => {
                    let timestamp = dt.with_timezone(&Local);
                    println!("  ✓ Parsed successfully: {}", timestamp);
                    commits.push(GitCommit {
                        hash,
                        message,
                        author,
                        timestamp,
                        refs,
                    });
                }
                Err(e) => {
                    println!("  ✗ Failed to parse: {}", e);
                }
            }
        }
    }
    
    println!("\nLoaded {} commits successfully", commits.len());
    Ok(commits)
}

#[tokio::main]
async fn main() -> Result<()> {
    let commits = load_git_commits().await?;
    
    println!("\nCommits summary:");
    for (i, commit) in commits.iter().enumerate() {
        println!("{}: {} - {} by {}", 
            i + 1,
            &commit.hash[..8],
            commit.message,
            commit.author
        );
    }
    
    Ok(())
}