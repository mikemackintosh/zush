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
    pub stash_count: usize,
    pub ahead: usize,
    pub behind: usize,
    /// Whether status counts are from a background cache (may be stale)
    pub from_cache: bool,
    /// Whether a background worker was spawned (prompt should async-refresh)
    pub async_pending: bool,
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

/// Path for the signal file that tells Zsh the background worker is done
fn signal_file_path() -> Option<PathBuf> {
    let zsh_pid = std::env::var("ZUSH_ZSH_PID").ok()?;
    Some(cache_dir().join(format!("prompt-signal-{}", zsh_pid)))
}

/// Path for the lock file that prevents concurrent workers for the same repo
fn lock_file_path(cache_path: &Path) -> PathBuf {
    cache_path.with_extension("lock")
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

    // Read stash count (fast — just counts lines in reflog file)
    status.stash_count = read_stash_count_fast(&git_dir);

    // Read ahead/behind counts (fast — reads packed-refs and loose refs)
    let (ahead, behind) = read_ahead_behind(&git_dir, &status.branch);
    status.ahead = ahead;
    status.behind = behind;

    // Check if minimal mode is enabled (skip all status checks)
    if is_env_truthy("ZUSH_GIT_MINIMAL") {
        return Some(status);
    }

    // Try to read cached background status first
    let repo_root = git_dir.parent().unwrap_or(path);
    let cache_path = status_cache_path(repo_root);

    if let Some(cached) = read_status_cache(&cache_path) {
        // Use cached counts (stash/ahead/behind already set from fast path above)
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
            if spawn_background_status(repo_root, &cache_path) {
                status.async_pending = true;
            }
        }

        return Some(status);
    }

    // No cache — check repo size to decide sync vs async
    let index_path = git_dir.join("index");
    let index_size = fs::metadata(&index_path).map(|m| m.len()).unwrap_or(0);

    // Repos with index > 512KB get async treatment (roughly 10k+ files)
    let large_repo_threshold = std::env::var("ZUSH_GIT_LARGE_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(512 * 1024);

    if index_size > large_repo_threshold {
        // Large repo: return branch-only now, compute status in background
        if spawn_background_status(repo_root, &cache_path) {
            status.async_pending = true;
        }
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

pub fn is_env_truthy(key: &str) -> bool {
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

/// Fast stash count: count lines in .git/logs/refs/stash (no libgit2)
fn read_stash_count_fast(git_dir: &Path) -> usize {
    let stash_log = git_dir.join("logs").join("refs").join("stash");
    match fs::read_to_string(stash_log) {
        Ok(contents) => contents.lines().count(),
        Err(_) => 0,
    }
}

/// Fast ahead/behind: use libgit2's graph_ahead_behind (requires repo open
/// but avoids spawning git subprocess). Falls back to 0/0 on any error.
fn read_ahead_behind(git_dir: &Path, branch: &str) -> (usize, usize) {
    if branch.is_empty() {
        return (0, 0);
    }

    // Resolve the repo root from git_dir
    let repo_root = match git_dir.parent() {
        Some(p) => p,
        None => return (0, 0),
    };

    let repo = match Repository::discover(repo_root) {
        Ok(r) => r,
        Err(_) => return (0, 0),
    };

    // Get the local branch reference
    let local_ref = match repo.find_branch(branch, git2::BranchType::Local) {
        Ok(b) => b,
        Err(_) => return (0, 0),
    };

    // Get the upstream tracking branch
    let upstream = match local_ref.upstream() {
        Ok(u) => u,
        Err(_) => return (0, 0), // No upstream configured
    };

    let local_oid = match local_ref.get().target() {
        Some(oid) => oid,
        None => return (0, 0),
    };

    let upstream_oid = match upstream.get().target() {
        Some(oid) => oid,
        None => return (0, 0),
    };

    match repo.graph_ahead_behind(local_oid, upstream_oid) {
        Ok((ahead, behind)) => (ahead, behind),
        Err(_) => (0, 0),
    }
}

/// Compute full git status including stash and ahead/behind (for background worker)
pub fn compute_full_status(path: &Path) -> Option<GitStatus> {
    let git_dir = find_git_dir(path)?;
    let branch = read_branch_fast(&git_dir).unwrap_or_default();
    let mut status = compute_status_counts(path)?;
    status.branch = branch.clone();
    status.stash_count = read_stash_count_fast(&git_dir);
    let (ahead, behind) = read_ahead_behind(&git_dir, &branch);
    status.ahead = ahead;
    status.behind = behind;
    Some(status)
}

/// Compute status counts using libgit2 (synchronous)
pub fn compute_status_counts(path: &Path) -> Option<GitStatus> {
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

/// Spawn a daemonized background process to compute git status.
///
/// Uses self-re-exec with a hidden subcommand + setsid() so the worker
/// survives the parent prompt process exiting. Returns true if a worker
/// was actually spawned (false if one is already running).
fn spawn_background_status(repo_path: &Path, cache_path: &Path) -> bool {
    let lock_path = lock_file_path(cache_path);

    // Check if a worker is already running for this repo
    if is_worker_running(&lock_path) {
        return false;
    }

    let repo_str = repo_path.to_string_lossy().to_string();
    let cache_str = cache_path.to_string_lossy().to_string();

    // Resolve our own binary path for self-re-exec
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let mut cmd = std::process::Command::new(exe);
    cmd.args([
        "_internal-git-status",
        "--repo-path",
        &repo_str,
        "--cache-path",
        &cache_str,
    ]);

    // Pass signal file path if Zsh PID is available
    if let Some(signal_path) = signal_file_path() {
        cmd.args(["--signal-file", &signal_path.to_string_lossy()]);
    }

    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    // On Unix: create new session so the worker isn't killed when parent exits
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
    }

    match cmd.spawn() {
        Ok(_child) => {
            // Don't wait on the child — it's daemonized
            true
        }
        Err(_) => false,
    }
}

/// Check if a background worker is already running for this cache path
fn is_worker_running(lock_path: &Path) -> bool {
    let contents = match fs::read_to_string(lock_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let pid: u32 = match contents.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            // Invalid lock file, clean up
            let _ = fs::remove_file(lock_path);
            return false;
        }
    };

    // Check if the process is still alive
    #[cfg(unix)]
    {
        let alive = unsafe { libc::kill(pid as libc::pid_t, 0) } == 0;
        if !alive {
            // Stale lock file, clean up
            let _ = fs::remove_file(lock_path);
        }
        alive
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        // On non-Unix, just check if lock is recent (< 30s)
        !is_cache_stale(lock_path, 30)
    }
}

/// Write a lock file with current PID
pub fn write_lock_file(cache_path: &Path) -> std::io::Result<()> {
    let lock_path = lock_file_path(cache_path);
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&lock_path, format!("{}", std::process::id()))
}

/// Remove the lock file
pub fn remove_lock_file(cache_path: &Path) {
    let lock_path = lock_file_path(cache_path);
    let _ = fs::remove_file(lock_path);
}

/// Parse git status --porcelain=v1 output into counts
#[cfg(test)]
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

/// Write status counts to a JSON cache file (atomic via temp + rename)
pub fn write_status_cache(cache_path: &Path, status: &GitStatus) -> std::io::Result<()> {
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
        "stash_count": status.stash_count,
        "ahead": status.ahead,
        "behind": status.behind,
    });

    // Atomic write: write to temp file, then rename into place
    let tmp_path = cache_path.with_extension("tmp");
    fs::write(&tmp_path, json.to_string())?;
    fs::rename(&tmp_path, cache_path)?;

    Ok(())
}

/// Touch the signal file to notify Zsh that background data is ready
pub fn touch_signal_file(signal_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = signal_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(signal_path, "")
}

/// Read status counts from cache file
fn read_status_cache(cache_path: &Path) -> Option<GitStatus> {
    // Cache must exist and be less than 30s old
    if !cache_path.exists() || is_cache_stale(cache_path, 30) {
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
        stash_count: v["stash_count"].as_u64().unwrap_or(0) as usize,
        ahead: v["ahead"].as_u64().unwrap_or(0) as usize,
        behind: v["behind"].as_u64().unwrap_or(0) as usize,
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
        "git_stash": status.stash_count,
        "git_ahead": status.ahead,
        "git_behind": status.behind,
        "git_from_cache": status.from_cache,
        "git_async_pending": status.async_pending,
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
        assert_eq!(status.staged, 3); // M + A + D in index
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

    #[test]
    fn test_read_status_cache_missing_file() {
        let path = PathBuf::from("/tmp/zush-test-nonexistent-cache.json");
        assert!(read_status_cache(&path).is_none());
    }

    #[test]
    fn test_write_and_read_cache() {
        let tmp = std::env::temp_dir().join("zush-test-cache.json");
        let status = GitStatus {
            staged: 3,
            modified: 2,
            added: 1,
            ..Default::default()
        };
        write_status_cache(&tmp, &status).unwrap();
        let cached = read_status_cache(&tmp).unwrap();
        assert_eq!(cached.staged, 3);
        assert_eq!(cached.modified, 2);
        assert_eq!(cached.added, 1);
        assert!(cached.from_cache);
        let _ = fs::remove_file(&tmp);
    }
}
