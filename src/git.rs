use git2::{Repository, StatusOptions, StatusShow};
use serde_json::{json, Value};
use std::path::Path;

#[derive(Debug, Default)]
pub struct GitStatus {
    pub branch: String,
    pub staged: usize,
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
    pub renamed: usize,
    pub untracked: usize,
    pub conflicted: usize,
}

/// Get git status for the current directory
/// This is much faster than calling `git` commands because:
/// 1. No process spawning
/// 2. Direct file reading from .git directory
/// 3. No shell overhead
pub fn get_git_status(path: &Path) -> Option<GitStatus> {
    // Try to open repository - this is fast, just checks for .git
    let repo = Repository::discover(path).ok()?;
    
    let mut status = GitStatus::default();
    
    // Get current branch name
    if let Ok(head) = repo.head() {
        if let Some(branch_name) = head.shorthand() {
            status.branch = branch_name.to_string();
        }
    }
    
    // Get status - this reads .git/index directly
    let mut opts = StatusOptions::new();
    opts.show(StatusShow::IndexAndWorkdir);
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(false);
    
    if let Ok(statuses) = repo.statuses(Some(&mut opts)) {
        for entry in statuses.iter() {
            let status_flags = entry.status();
            
            // Index (staged) changes
            if status_flags.is_index_new() {
                status.added += 1;
                status.staged += 1;
            }
            if status_flags.is_index_modified() {
                status.staged += 1;
            }
            if status_flags.is_index_deleted() {
                status.deleted += 1;
                status.staged += 1;
            }
            if status_flags.is_index_renamed() {
                status.renamed += 1;
                status.staged += 1;
            }
            
            // Working tree changes
            if status_flags.is_wt_modified() {
                status.modified += 1;
            }
            if status_flags.is_wt_deleted() {
                status.deleted += 1;
            }
            
            // Untracked
            if status_flags.is_wt_new() {
                status.untracked += 1;
            }
            
            // Conflicted
            if status_flags.is_conflicted() {
                status.conflicted += 1;
            }
        }
    }
    
    Some(status)
}

/// Convert GitStatus to JSON for template context
pub fn git_status_to_json(status: &GitStatus) -> Value {
    json!({
        "git_branch": status.branch,
        "git_staged": status.staged,
        "git_modified": status.modified,
        "git_added": status.added,
        "git_deleted": status.deleted,
        "git_renamed": status.renamed,
        "git_untracked": status.untracked,
        "git_conflicted": status.conflicted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_git_status() {
        // Test in current directory (should be a git repo during development)
        let cwd = env::current_dir().unwrap();
        if let Some(status) = get_git_status(&cwd) {
            println!("Branch: {}", status.branch);
            println!("Staged: {}", status.staged);
            println!("Modified: {}", status.modified);
            assert!(!status.branch.is_empty());
        }
    }
}
