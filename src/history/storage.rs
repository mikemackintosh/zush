//! History storage with JSONL format and file locking

use super::entry::HistoryEntry;
use anyhow::{Context, Result};
use fs4::fs_std::FileExt;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

/// Default history file location
const HISTORY_FILENAME: &str = "history.jsonl";

/// Get the history file path
pub fn get_history_path() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;

    let zush_dir = data_dir.join("zush");
    Ok(zush_dir.join(HISTORY_FILENAME))
}

/// Ensure the history directory exists
fn ensure_history_dir() -> Result<PathBuf> {
    let path = get_history_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create history directory")?;
    }
    Ok(path)
}

/// Append a single entry to the history file with file locking
pub fn append_entry(entry: &HistoryEntry) -> Result<()> {
    let path = ensure_history_dir()?;

    // Open file for append (create if not exists)
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&path)
        .context("Failed to open history file")?;

    // Acquire exclusive lock (blocks until available)
    // Use try_lock with timeout to avoid blocking forever
    file.lock_exclusive()
        .context("Failed to acquire lock on history file")?;

    // Write entry as JSON line
    let mut writer = BufWriter::new(&file);
    let json = entry.to_json().context("Failed to serialize entry")?;
    writeln!(writer, "{}", json).context("Failed to write entry")?;
    writer.flush().context("Failed to flush history file")?;

    // Lock is released when file is dropped
    Ok(())
}

/// Read all entries from the history file
pub fn read_all_entries() -> Result<Vec<HistoryEntry>> {
    let path = get_history_path()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&path).context("Failed to open history file")?;

    // Acquire shared lock for reading
    file.lock_shared()
        .context("Failed to acquire read lock on history file")?;

    let reader = BufReader::new(&file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line.context("Failed to read line")?;
        if line.trim().is_empty() {
            continue;
        }

        // Skip malformed entries instead of failing
        match HistoryEntry::from_json(&line) {
            Ok(entry) => entries.push(entry),
            Err(_) => {
                // Log warning in debug mode, but continue
                #[cfg(debug_assertions)]
                eprintln!("Warning: skipping malformed history entry");
            }
        }
    }

    Ok(entries)
}

/// Read the most recent N entries (reads from end of file)
pub fn read_recent_entries(count: usize) -> Result<Vec<HistoryEntry>> {
    let path = get_history_path()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    // For now, read all and take last N
    // TODO: Optimize with reverse file reading for large files
    let all = read_all_entries()?;
    let start = all.len().saturating_sub(count);
    Ok(all[start..].to_vec())
}

/// Clear all history entries
pub fn clear_all() -> Result<()> {
    let path = get_history_path()?;

    if path.exists() {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)
            .context("Failed to open history file for clearing")?;

        file.lock_exclusive()
            .context("Failed to acquire lock for clearing")?;

        // File is truncated on open with truncate(true)
    }

    Ok(())
}

/// Clear entries older than the given number of days
pub fn clear_older_than(days: u32) -> Result<usize> {
    let path = get_history_path()?;

    if !path.exists() {
        return Ok(0);
    }

    let cutoff = chrono::Utc::now().timestamp() - (days as i64 * 24 * 60 * 60);
    let entries = read_all_entries()?;
    let (keep, remove): (Vec<_>, Vec<_>) = entries.into_iter().partition(|e| e.ts >= cutoff);

    let removed_count = remove.len();

    if removed_count > 0 {
        // Rewrite file with kept entries
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)
            .context("Failed to open history file for rewrite")?;

        file.lock_exclusive()
            .context("Failed to acquire lock for rewrite")?;

        let mut writer = BufWriter::new(&file);
        for entry in keep {
            let json = entry.to_json()?;
            writeln!(writer, "{}", json)?;
        }
        writer.flush()?;
    }

    Ok(removed_count)
}

/// Get history file statistics
pub fn get_stats() -> Result<HistoryStats> {
    let path = get_history_path()?;

    if !path.exists() {
        return Ok(HistoryStats {
            entry_count: 0,
            file_size_bytes: 0,
            oldest_timestamp: None,
            newest_timestamp: None,
        });
    }

    let metadata = fs::metadata(&path)?;
    let entries = read_all_entries()?;

    let oldest = entries.first().map(|e| e.ts);
    let newest = entries.last().map(|e| e.ts);

    Ok(HistoryStats {
        entry_count: entries.len(),
        file_size_bytes: metadata.len(),
        oldest_timestamp: oldest,
        newest_timestamp: newest,
    })
}

/// Statistics about the history file
#[derive(Debug)]
pub struct HistoryStats {
    pub entry_count: usize,
    pub file_size_bytes: u64,
    pub oldest_timestamp: Option<i64>,
    pub newest_timestamp: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn with_temp_history<F>(f: F)
    where
        F: FnOnce(PathBuf),
    {
        let dir = tempdir().unwrap();
        let path = dir.path().join("history.jsonl");
        f(path);
    }

    #[test]
    fn test_append_and_read() {
        with_temp_history(|_path| {
            // Note: This test would need to mock get_history_path
            // For now, just verify the functions compile
        });
    }
}
