use git2::{Repository, StatusOptions, StatusShow};
use serde_json::{json, Value};
use std::fs;
use std::io::Read as IoRead;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Default, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub staged: usize,
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
    pub renamed: usize,
    pub untracked: usize,
    pub conflicted: usize,
    /// Whether status counts are from a background cache (may be stale)
    pub from_cache: bool,
}

/// Cache file location for async git status results
fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("zush")
}

fn status_cache_path(repo_path: &Path) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    repo_path.hash(&mut hasher);
    let hash = hasher.finish();

    cache_dir().join(format!("git-status-{:x}.json", hash))
}

/// Get git status for the current directory.
///
/// Strategy for large repos:
/// 1. Branch name is always read instantly from .git/HEAD (no libgit2)
/// 2. If a background status cache exists and is fresh, use it
/// 3. For small repos (< threshold), compute status synchronously
/// 4. For large repos, return branch-only and kick off a background status worker
pub fn get_git_status(path: &Path) -> Option<GitStatus> {
    let git_dir = find_git_dir(path)?;
    let branch = read_branch_fast(&git_dir)?;

    let mut status = GitStatus {
        branch,
        ..Default::default()
    };

    // Check if minimal mode is enabled (skip all status checks)
    if is_env_truthy("ZUSH_GIT_MINIMAL") {
        return Some(status);
    }

    // Try to read cached background status first
    let repo_root = git_dir.parent().unwrap_or(path);
    let cache_path = status_cache_path(repo_root);

    if let Some(cached) = read_status_cache(&cache_path) {
        // Use cached counts
        status.staged = cached.staged;
        status.modified = cached.modified;
        status.added = cached.added;
        status.deleted = cached.deleted;
        status.renamed = cached.renamed;
        status.untracked = cached.untracked;
        status.conflicted = cached.conflicted;
        status.from_cache = true;

        // If the cache is stale (older than 10s), kick off a background refresh
        if is_cache_stale(&cache_path, 10) {
            spawn_background_status(repo_root, &cache_path);
        }

        return Some(status);
    }

    // No cache — check repo size to decide sync vs async
    // Use the git index size as a heuristic: large index = large repo
    let index_path = git_dir.join("index");
    let index_size = fs::metadata(&index_path).map(|m| m.len()).unwrap_or(0);

    // Repos with index > 512KB get async treatment (roughly 10k+ files)
    let large_repo_threshold = std::env::var("ZUSH_GIT_LARGE_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(512 * 1024);

    if index_size > large_repo_threshold {
        // Large repo: return branch-only now, compute status in background
        spawn_background_status(repo_root, &cache_path);
        return Some(status);
    }

    // Small repo: compute status synchronously (fast enough)
    if let Some(sync_status) = compute_status_counts(path) {
        status.staged = sync_status.staged;
        status.modified = sync_status.modified;
        status.added = sync_status.added;
        status.deleted = sync_status.deleted;
        status.renamed = sync_status.renamed;
        status.untracked = sync_status.untracked;
        status.conflicted = sync_status.conflicted;
    }

    Some(status)
}

fn is_env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Fast path: read branch name directly from .git/HEAD (no libgit2)
fn read_branch_fast(git_dir: &Path) -> Option<String> {
    let head_path = git_dir.join("HEAD");
    let contents = fs::read_to_string(head_path).ok()?;
    let trimmed = contents.trim();

    if let Some(ref_path) = trimmed.strip_prefix("ref: ") {
        ref_path
            .strip_prefix("refs/heads/")
            .map(|s| s.to_string())
            .or_else(|| Some(ref_path.to_string()))
    } else {
        // Detached HEAD — short hash
        Some(trimmed.chars().take(7).collect())
    }
}

/// Walk up from path to find .git directory (supports worktrees)
fn find_git_dir(mut path: &Path) -> Option<PathBuf> {
    loop {
        let git_dir = path.join(".git");

        if git_dir.is_dir() {
            return Some(git_dir);
        }

        // Git worktree: .git is a file containing "gitdir: <path>"
        if git_dir.is_file() {
            if let Ok(contents) = fs::read_to_string(&git_dir) {
                if let Some(gitdir) = contents.trim().strip_prefix("gitdir: ") {
                    return Some(PathBuf::from(gitdir));
                }
            }
        }

        path = path.parent()?;
    }
}

/// Compute status counts using libgit2 (synchronous)
fn compute_status_counts(path: &Path) -> Option<GitStatus> {
    let repo = Repository::discover(path).ok()?;
    let mut status = GitStatus::default();

    let mut opts = StatusOptions::new();
    opts.show(StatusShow::IndexAndWorkdir);

    let include_untracked = !is_env_truthy("ZUSH_GIT_DISABLE_UNTRACKED");
    opts.include_untracked(include_untracked);
    opts.recurse_untracked_dirs(false);

    // Exclude submodules — they're a major source of slowness
    opts.exclude_submodules(true);

    if let Ok(statuses) = repo.statuses(Some(&mut opts)) {
        for entry in statuses.iter() {
            let flags = entry.status();

            if flags.is_index_new() {
                status.added += 1;
                status.staged += 1;
            }
            if flags.is_index_modified() {
                status.staged += 1;
            }
            if flags.is_index_deleted() {
                status.deleted += 1;
                status.staged += 1;
            }
            if flags.is_index_renamed() {
                status.renamed += 1;
                status.staged += 1;
            }
            if flags.is_wt_modified() {
                status.modified += 1;
            }
            if flags.is_wt_deleted() {
                status.deleted += 1;
            }
            if flags.is_wt_new() {
                status.untracked += 1;
            }
            if flags.is_conflicted() {
                status.conflicted += 1;
            }
        }
    }

    Some(status)
}

/// Spawn a background process to compute git status and write it to a cache file.
/// The next prompt render will pick up the cached result.
fn spawn_background_status(repo_path: &Path, cache_path: &Path) {
    let repo_str = repo_path.to_string_lossy().to_string();
    let cache_str = cache_path.to_string_lossy().to_string();

    // Fork a background process that computes status and writes JSON to cache
    // We use the zush-prompt binary itself with a hidden subcommand,
    // but for simplicity, use a simple shell + git approach which is more robust
    // and avoids re-linking libgit2 in a child.
    let disable_untracked = if is_env_truthy("ZUSH_GIT_DISABLE_UNTRACKED") {
        "-uno"
    } else {
        "-unormal"
    };

    // Use git's porcelain output — stable, parseable, and respects .gitignore
    let _ = std::process::Command::new("git")
        .args([
            "-C",
            &repo_str,
            "status",
            "--porcelain=v1",
            disable_untracked,
            "--ignore-submodules=all",
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .and_then(|child| {
            // Do the parsing in a detached thread so we don't block
            let cache_str_owned = cache_str.clone();
            std::thread::spawn(move || {
                if let Ok(output) = child.wait_with_output() {
                    if output.status.success() {
                        let counts = parse_porcelain_status(&output.stdout);
                        let _ = write_status_cache(&PathBuf::from(cache_str_owned), &counts);
                    }
                }
            });
            Ok(())
        });
}

/// Parse git status --porcelain=v1 output into counts
fn parse_porcelain_status(output: &[u8]) -> GitStatus {
    let mut status = GitStatus::default();

    for line in output.split(|&b| b == b'\n') {
        if line.len() < 2 {
            continue;
        }
        let index = line[0];
        let worktree = line[1];

        // Index (staged) changes
        match index {
            b'A' => {
                status.added += 1;
                status.staged += 1;
            }
            b'M' => status.staged += 1,
            b'D' => {
                status.deleted += 1;
                status.staged += 1;
            }
            b'R' => {
                status.renamed += 1;
                status.staged += 1;
            }
            _ => {}
        }

        // Worktree changes
        match worktree {
            b'M' => status.modified += 1,
            b'D' => status.deleted += 1,
            _ => {}
        }

        // Untracked
        if index == b'?' && worktree == b'?' {
            status.untracked += 1;
        }

        // Conflicted
        if index == b'U' || worktree == b'U' {
            status.conflicted += 1;
        }
    }

    status
}

/// Write status counts to a JSON cache file
fn write_status_cache(cache_path: &Path, status: &GitStatus) -> std::io::Result<()> {
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = json!({
        "staged": status.staged,
        "modified": status.modified,
        "added": status.added,
        "deleted": status.deleted,
        "renamed": status.renamed,
        "untracked": status.untracked,
        "conflicted": status.conflicted,
    });

    fs::write(cache_path, json.to_string())
}

/// Read status counts from cache file
fn read_status_cache(cache_path: &Path) -> Option<GitStatus> {
    // Cache must exist and be less than 30s old
    if is_cache_stale(cache_path, 30) && !cache_path.exists() {
        return None;
    }

    let mut contents = String::new();
    fs::File::open(cache_path)
        .ok()?
        .read_to_string(&mut contents)
        .ok()?;

    let v: Value = serde_json::from_str(&contents).ok()?;
    Some(GitStatus {
        staged: v["staged"].as_u64().unwrap_or(0) as usize,
        modified: v["modified"].as_u64().unwrap_or(0) as usize,
        added: v["added"].as_u64().unwrap_or(0) as usize,
        deleted: v["deleted"].as_u64().unwrap_or(0) as usize,
        renamed: v["renamed"].as_u64().unwrap_or(0) as usize,
        untracked: v["untracked"].as_u64().unwrap_or(0) as usize,
        conflicted: v["conflicted"].as_u64().unwrap_or(0) as usize,
        from_cache: true,
        ..Default::default()
    })
}

/// Check if a cache file is older than `max_age_secs`
fn is_cache_stale(path: &Path, max_age_secs: u64) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return true;
    };
    let Ok(modified) = meta.modified() else {
        return true;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return true;
    };
    age.as_secs() > max_age_secs
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
        let cwd = env::current_dir().unwrap();
        if let Some(status) = get_git_status(&cwd) {
            println!("Branch: {}", status.branch);
            println!("Staged: {}", status.staged);
            println!("Modified: {}", status.modified);
            assert!(!status.branch.is_empty());
        }
    }

    #[test]
    fn test_parse_porcelain_status() {
        let output = b"M  src/main.rs\n?? new_file.txt\nA  added.rs\nD  deleted.rs\n";
        let status = parse_porcelain_status(output);
        assert_eq!(status.staged, 1); // M in index
        assert_eq!(status.untracked, 1); // ??
        assert_eq!(status.added, 1); // A
        assert_eq!(status.deleted, 1); // D in index
    }

    #[test]
    fn test_parse_porcelain_empty() {
        let status = parse_porcelain_status(b"");
        assert_eq!(status.modified, 0);
        assert_eq!(status.staged, 0);
        assert_eq!(status.untracked, 0);
    }
}
