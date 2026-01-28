//! History entry type and serialization

use serde::{Deserialize, Serialize};

/// A single history entry with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unix timestamp (seconds)
    pub ts: i64,
    /// Working directory
    pub dir: String,
    /// Session ID (unique per shell instance)
    pub sid: String,
    /// Command text
    pub cmd: String,
    /// Exit code
    pub exit: i32,
    /// Duration in milliseconds
    pub dur: u64,
    /// Hostname
    pub host: String,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(
        command: String,
        directory: String,
        session_id: String,
        exit_code: i32,
        duration_ms: u64,
        hostname: String,
    ) -> Self {
        Self {
            ts: chrono::Utc::now().timestamp(),
            dir: directory,
            sid: session_id,
            cmd: command,
            exit: exit_code,
            dur: duration_ms,
            host: hostname,
        }
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON string (compact, no trailing newline)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Get formatted timestamp for display
    pub fn formatted_time(&self) -> String {
        use chrono::{Local, TimeZone};
        Local
            .timestamp_opt(self.ts, 0)
            .single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Get formatted duration for display
    pub fn formatted_duration(&self) -> String {
        if self.dur < 1000 {
            format!("{}ms", self.dur)
        } else if self.dur < 60_000 {
            format!("{:.1}s", self.dur as f64 / 1000.0)
        } else {
            let mins = self.dur / 60_000;
            let secs = (self.dur % 60_000) / 1000;
            format!("{}m{}s", mins, secs)
        }
    }

    /// Get shortened directory for display
    pub fn short_dir(&self) -> String {
        if let Ok(home) = std::env::var("HOME") {
            self.dir.replace(&home, "~")
        } else {
            self.dir.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_roundtrip() {
        let entry = HistoryEntry::new(
            "git status".to_string(),
            "/home/user/project".to_string(),
            "abc123".to_string(),
            0,
            150,
            "localhost".to_string(),
        );

        let json = entry.to_json().unwrap();
        let parsed = HistoryEntry::from_json(&json).unwrap();

        assert_eq!(parsed.cmd, "git status");
        assert_eq!(parsed.exit, 0);
        assert_eq!(parsed.dur, 150);
    }

    #[test]
    fn test_formatted_duration() {
        let mut entry = HistoryEntry::new(
            "test".to_string(),
            "/".to_string(),
            "x".to_string(),
            0,
            500,
            "h".to_string(),
        );
        assert_eq!(entry.formatted_duration(), "500ms");

        entry.dur = 2500;
        assert_eq!(entry.formatted_duration(), "2.5s");

        entry.dur = 125_000;
        assert_eq!(entry.formatted_duration(), "2m5s");
    }
}
