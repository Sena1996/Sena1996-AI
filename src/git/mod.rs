use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git command failed: {0}")]
    CommandFailed(String),
    #[error("Not a git repository")]
    NotGitRepo,
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type GitResult<T> = Result<T, GitError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<String>,
    pub is_clean: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub status: ChangeStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Unmerged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiff {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub file: String,
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub content: String,
}

pub struct GitRepo {
    working_dir: std::path::PathBuf,
}

impl GitRepo {
    pub fn open(path: &Path) -> GitResult<Self> {
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Err(GitError::NotGitRepo);
        }

        Ok(Self {
            working_dir: path.to_path_buf(),
        })
    }

    pub fn current_dir() -> GitResult<Self> {
        let cwd = std::env::current_dir()?;
        Self::open(&cwd)
    }

    fn run_git(&self, args: &[&str]) -> GitResult<String> {
        let output = Command::new("git")
            .current_dir(&self.working_dir)
            .args(args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitError::CommandFailed(stderr.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn status(&self) -> GitResult<GitStatus> {
        let output = self.run_git(&["status", "--porcelain=v2", "-b"])?;

        let mut branch = String::from("HEAD");
        let mut ahead = 0;
        let mut behind = 0;
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();

        for line in output.lines() {
            if line.starts_with("# branch.head") {
                branch = line.split_whitespace().last().unwrap_or("HEAD").to_string();
            } else if line.starts_with("# branch.ab") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    ahead = parts[2].trim_start_matches('+').parse().unwrap_or(0);
                    behind = parts[3].trim_start_matches('-').parse().unwrap_or(0);
                }
            } else if line.starts_with('1') || line.starts_with('2') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 {
                    let xy = parts[1];
                    let path = parts.last().unwrap_or(&"").to_string();

                    let x_status = parse_status_char(xy.chars().next().unwrap_or('.'));
                    let y_status = parse_status_char(xy.chars().nth(1).unwrap_or('.'));

                    if let Some(status) = x_status {
                        staged.push(FileChange {
                            path: path.clone(),
                            status,
                        });
                    }

                    if let Some(status) = y_status {
                        unstaged.push(FileChange { path, status });
                    }
                }
            } else if line.starts_with('?') {
                let path = line.split_whitespace().last().unwrap_or("").to_string();
                untracked.push(path);
            }
        }

        let is_clean = staged.is_empty() && unstaged.is_empty() && untracked.is_empty();

        Ok(GitStatus {
            branch,
            ahead,
            behind,
            staged,
            unstaged,
            untracked,
            is_clean,
        })
    }

    pub fn log(&self, count: usize) -> GitResult<Vec<GitCommit>> {
        let output = self.run_git(&[
            "log",
            &format!("-{}", count),
            "--pretty=format:%H|%h|%an|%ad|%s",
            "--date=short",
        ])?;

        let commits = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(5, '|').collect();
                if parts.len() >= 5 {
                    Some(GitCommit {
                        hash: parts[0].to_string(),
                        short_hash: parts[1].to_string(),
                        author: parts[2].to_string(),
                        date: parts[3].to_string(),
                        message: parts[4].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(commits)
    }

    pub fn diff(&self, staged: bool) -> GitResult<GitDiff> {
        let args = if staged {
            vec!["diff", "--cached", "--stat"]
        } else {
            vec!["diff", "--stat"]
        };

        let stat_output = self.run_git(&args)?;

        let mut files_changed = 0;
        let mut insertions = 0;
        let mut deletions = 0;

        if let Some(summary_line) = stat_output.lines().last() {
            let parts: Vec<&str> = summary_line.split(',').collect();
            for part in parts {
                let trimmed = part.trim();
                if trimmed.contains("file") {
                    files_changed = trimmed
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                } else if trimmed.contains("insertion") {
                    insertions = trimmed
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                } else if trimmed.contains("deletion") {
                    deletions = trimmed
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                }
            }
        }

        Ok(GitDiff {
            files_changed,
            insertions,
            deletions,
            hunks: Vec::new(),
        })
    }

    pub fn add(&self, paths: &[&str]) -> GitResult<()> {
        let mut args = vec!["add"];
        args.extend(paths);
        self.run_git(&args)?;
        Ok(())
    }

    pub fn add_all(&self) -> GitResult<()> {
        self.run_git(&["add", "-A"])?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> GitResult<String> {
        self.run_git(&["commit", "-m", message])
    }

    pub fn current_branch(&self) -> GitResult<String> {
        let output = self.run_git(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        Ok(output.trim().to_string())
    }

    pub fn remote_url(&self, remote: &str) -> GitResult<String> {
        let output = self.run_git(&["remote", "get-url", remote])?;
        Ok(output.trim().to_string())
    }

    pub fn branches(&self) -> GitResult<Vec<String>> {
        let output = self.run_git(&["branch", "--list", "--format=%(refname:short)"])?;
        Ok(output.lines().map(String::from).collect())
    }

    pub fn stash(&self) -> GitResult<()> {
        self.run_git(&["stash"])?;
        Ok(())
    }

    pub fn stash_pop(&self) -> GitResult<()> {
        self.run_git(&["stash", "pop"])?;
        Ok(())
    }

    pub fn checkout(&self, branch: &str) -> GitResult<()> {
        self.run_git(&["checkout", branch])?;
        Ok(())
    }

    pub fn create_branch(&self, branch: &str) -> GitResult<()> {
        self.run_git(&["checkout", "-b", branch])?;
        Ok(())
    }

    pub fn pull(&self) -> GitResult<String> {
        self.run_git(&["pull"])
    }

    pub fn push(&self) -> GitResult<String> {
        self.run_git(&["push"])
    }

    pub fn push_set_upstream(&self, remote: &str, branch: &str) -> GitResult<String> {
        self.run_git(&["push", "-u", remote, branch])
    }
}

fn parse_status_char(c: char) -> Option<ChangeStatus> {
    match c {
        'A' => Some(ChangeStatus::Added),
        'M' => Some(ChangeStatus::Modified),
        'D' => Some(ChangeStatus::Deleted),
        'R' => Some(ChangeStatus::Renamed),
        'C' => Some(ChangeStatus::Copied),
        'U' => Some(ChangeStatus::Unmerged),
        '.' | ' ' => None,
        _ => None,
    }
}

pub fn generate_commit_message(diff_summary: &str, file_changes: &[FileChange]) -> String {
    let change_count = file_changes.len();

    if change_count == 0 {
        return "Update files".to_string();
    }

    if change_count == 1 {
        let change = &file_changes[0];
        let action = match change.status {
            ChangeStatus::Added => "Add",
            ChangeStatus::Modified => "Update",
            ChangeStatus::Deleted => "Remove",
            ChangeStatus::Renamed => "Rename",
            _ => "Update",
        };
        return format!("{} {}", action, change.path);
    }

    let has_adds = file_changes.iter().any(|c| c.status == ChangeStatus::Added);
    let has_mods = file_changes
        .iter()
        .any(|c| c.status == ChangeStatus::Modified);
    let has_dels = file_changes
        .iter()
        .any(|c| c.status == ChangeStatus::Deleted);

    let actions = vec![
        if has_adds { Some("Add") } else { None },
        if has_mods { Some("Update") } else { None },
        if has_dels { Some("Remove") } else { None },
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("/");

    let prefix = if diff_summary.contains("src/") {
        "code"
    } else if diff_summary.contains(".md") || diff_summary.contains("README") {
        "docs"
    } else if diff_summary.contains("test") {
        "tests"
    } else if diff_summary.contains("Cargo") || diff_summary.contains("package") {
        "deps"
    } else {
        "files"
    };

    format!("{} {} ({} files)", actions, prefix, change_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_char() {
        assert_eq!(parse_status_char('A'), Some(ChangeStatus::Added));
        assert_eq!(parse_status_char('M'), Some(ChangeStatus::Modified));
        assert_eq!(parse_status_char('D'), Some(ChangeStatus::Deleted));
        assert_eq!(parse_status_char('.'), None);
    }

    #[test]
    fn test_generate_commit_message_single() {
        let changes = vec![FileChange {
            path: "src/main.rs".to_string(),
            status: ChangeStatus::Modified,
        }];

        let msg = generate_commit_message("", &changes);
        assert!(msg.contains("Update src/main.rs"));
    }

    #[test]
    fn test_generate_commit_message_multiple() {
        let changes = vec![
            FileChange {
                path: "src/main.rs".to_string(),
                status: ChangeStatus::Modified,
            },
            FileChange {
                path: "src/lib.rs".to_string(),
                status: ChangeStatus::Added,
            },
        ];

        let msg = generate_commit_message("src/", &changes);
        assert!(msg.contains("code"));
        assert!(msg.contains("2 files"));
    }

    #[test]
    fn test_git_repo_not_found() {
        let result = GitRepo::open(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
