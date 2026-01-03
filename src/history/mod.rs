//! History tracking and search module
//!
//! This module provides:
//! - JSONL-based history storage with file locking for concurrent access
//! - Fuzzy search using nucleo
//! - TUI interfaces (fzf-style and full table view)

pub mod entry;
pub mod search;
pub mod storage;
pub mod tui;

pub use entry::HistoryEntry;
pub use search::SearchFilter;
pub use storage::{
    append_entry, clear_all, clear_older_than, get_history_path, get_stats, read_all_entries,
    read_recent_entries,
};
