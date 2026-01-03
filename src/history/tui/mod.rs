//! TUI interfaces for history search

mod fzf;
mod full;

pub use fzf::run_fzf_picker;
pub use full::run_full_picker;

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
pub fn run_picker(entries: Vec<HistoryEntry>, style: TuiStyle) -> Result<Option<String>> {
    match style {
        TuiStyle::Fzf => run_fzf_picker(entries),
        TuiStyle::Full => run_full_picker(entries),
    }
}
