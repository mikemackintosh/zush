//! TUI interfaces for history search

#[cfg(unix)]
mod full;
#[cfg(unix)]
mod fzf;

#[cfg(unix)]
pub use full::run_full_picker;
#[cfg(unix)]
pub use fzf::run_fzf_picker;

use crate::history::HistoryEntry;
use anyhow::Result;

/// TUI style selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TuiStyle {
    #[default]
    Fzf,
    Full,
}

impl TuiStyle {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "full" | "table" | "pane" => TuiStyle::Full,
            _ => TuiStyle::Fzf,
        }
    }
}

/// Run the TUI picker with the specified style
#[cfg(unix)]
pub fn run_picker(entries: Vec<HistoryEntry>, style: TuiStyle) -> Result<Option<String>> {
    match style {
        TuiStyle::Fzf => run_fzf_picker(entries),
        TuiStyle::Full => run_full_picker(entries),
    }
}

/// Windows fallback - TUI picker not supported
#[cfg(not(unix))]
pub fn run_picker(_entries: Vec<HistoryEntry>, _style: TuiStyle) -> Result<Option<String>> {
    anyhow::bail!("Interactive history picker is not supported on Windows. Use 'zush-prompt history list' instead.")
}

#[cfg(not(unix))]
pub fn run_fzf_picker(_entries: Vec<HistoryEntry>) -> Result<Option<String>> {
    anyhow::bail!("Interactive history picker is not supported on Windows.")
}

#[cfg(not(unix))]
pub fn run_full_picker(_entries: Vec<HistoryEntry>) -> Result<Option<String>> {
    anyhow::bail!("Interactive history picker is not supported on Windows.")
}
