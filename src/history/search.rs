//! History search with fuzzy matching using nucleo

use super::entry::HistoryEntry;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Search result with score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: HistoryEntry,
    pub score: u32,
}

/// Filter options for history search
#[derive(Debug, Default, Clone)]
pub struct SearchFilter {
    /// Filter by directory (exact match or prefix)
    pub directory: Option<String>,
    /// Filter by session ID
    pub session: Option<String>,
    /// Filter by hostname
    pub hostname: Option<String>,
    /// Only show commands with exit code 0
    pub successful_only: bool,
    /// Minimum timestamp (unix seconds)
    pub after: Option<i64>,
    /// Maximum timestamp (unix seconds)
    pub before: Option<i64>,
}

impl SearchFilter {
    /// Check if an entry matches the filter
    pub fn matches(&self, entry: &HistoryEntry) -> bool {
        if let Some(ref dir) = self.directory {
            if !entry.dir.starts_with(dir) {
                return false;
            }
        }

        if let Some(ref sid) = self.session {
            if &entry.sid != sid {
                return false;
            }
        }

        if let Some(ref host) = self.hostname {
            if &entry.host != host {
                return false;
            }
        }

        if self.successful_only && entry.exit != 0 {
            return false;
        }

        if let Some(after) = self.after {
            if entry.ts < after {
                return false;
            }
        }

        if let Some(before) = self.before {
            if entry.ts > before {
                return false;
            }
        }

        true
    }
}

/// Search history entries with fuzzy matching
pub fn search(
    entries: &[HistoryEntry],
    query: &str,
    filter: &SearchFilter,
    max_results: usize,
) -> Vec<SearchResult> {
    if query.is_empty() {
        // No query - return filtered entries in reverse order (most recent first)
        return entries
            .iter()
            .rev()
            .filter(|e| filter.matches(e))
            .take(max_results)
            .map(|e| SearchResult {
                entry: e.clone(),
                score: 0,
            })
            .collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);

    let mut results: Vec<SearchResult> = entries
        .iter()
        .filter(|e| filter.matches(e))
        .filter_map(|entry| {
            let mut buf = Vec::new();
            let haystack = Utf32Str::new(&entry.cmd, &mut buf);
            pattern
                .score(haystack, &mut matcher)
                .map(|score| SearchResult {
                    entry: entry.clone(),
                    score,
                })
        })
        .collect();

    // Sort by score (highest first), then by timestamp (most recent first)
    results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.entry.ts.cmp(&a.entry.ts))
    });

    results.truncate(max_results);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(cmd: &str, ts: i64) -> HistoryEntry {
        HistoryEntry {
            ts,
            dir: "/home/user".to_string(),
            sid: "test".to_string(),
            cmd: cmd.to_string(),
            exit: 0,
            dur: 100,
            host: "localhost".to_string(),
        }
    }

    #[test]
    fn test_search_empty_query() {
        let entries = vec![
            make_entry("git status", 1000),
            make_entry("git commit", 2000),
            make_entry("git push", 3000),
        ];

        let results = search(&entries, "", &SearchFilter::default(), 10);
        assert_eq!(results.len(), 3);
        // Most recent first
        assert_eq!(results[0].entry.cmd, "git push");
    }

    #[test]
    fn test_search_fuzzy() {
        let entries = vec![
            make_entry("git status", 1000),
            make_entry("cargo test", 2000),
            make_entry("git stash", 3000),
        ];

        let results = search(&entries, "gst", &SearchFilter::default(), 10);
        // Should match "git status" and "git stash"
        assert!(results.iter().any(|r| r.entry.cmd == "git status"));
        assert!(results.iter().any(|r| r.entry.cmd == "git stash"));
    }

    #[test]
    fn test_filter_directory() {
        let mut entries = vec![make_entry("cmd1", 1000), make_entry("cmd2", 2000)];
        entries[0].dir = "/home/user/project".to_string();
        entries[1].dir = "/home/user/other".to_string();

        let filter = SearchFilter {
            directory: Some("/home/user/project".to_string()),
            ..Default::default()
        };

        let results = search(&entries, "", &filter, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.cmd, "cmd1");
    }
}
