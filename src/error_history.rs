use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorEntry {
    fingerprint: String,
    command: String,
    error_type: String,
    error_message: String,
    timestamp: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ErrorHistoryData {
    entries: Vec<ErrorEntry>,
}

pub struct ErrorHistory {
    path: PathBuf,
    data: ErrorHistoryData,
}

impl ErrorHistory {
    pub fn load() -> Self {
        let path = history_path();
        let data = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            ErrorHistoryData::default()
        };
        Self { path, data }
    }

    pub fn record(
        &mut self,
        fingerprint: &str,
        command: &str,
        error_type: &str,
        error_message: &str,
    ) {
        let now = chrono_now();
        self.data.entries.push(ErrorEntry {
            fingerprint: fingerprint.to_string(),
            command: command.to_string(),
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            timestamp: now,
        });
        // FIFO: keep at most 50 entries
        if self.data.entries.len() > 50 {
            let excess = self.data.entries.len() - 50;
            self.data.entries.drain(..excess);
        }
        self.save();
    }

    pub fn check_threshold(
        &self,
        fingerprint: &str,
        window_minutes: i64,
        threshold: usize,
    ) -> bool {
        let cutoff = chrono_cutoff(window_minutes);
        self.data
            .entries
            .iter()
            .filter(|e| e.fingerprint == fingerprint && e.timestamp >= cutoff)
            .count()
            >= threshold
    }

    fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string(&self.data) {
            let _ = std::fs::write(&self.path, json);
        }
    }
}

fn history_path() -> PathBuf {
    dirs_next().unwrap_or_else(|| PathBuf::from(".")).join(".biolab_error_history.json")
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .ok()
            .map(PathBuf::from)
            .or_else(|| dirs_next_home())
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs_next_home()
    }
}

fn dirs_next_home() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".local/share"))
            })
    }
    #[cfg(not(target_os = "linux"))]
    {
        std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("USERPROFILE")
                    .ok()
                    .map(PathBuf::from)
            })
    }
}

fn chrono_now() -> String {
    // ISO 8601 UTC timestamp without pulling in the chrono crate
    #[cfg(not(test))]
    {
        use std::time::SystemTime;
        let dur = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = dur.as_secs();
        // Convert unix timestamp to a basic ISO 8601 string
        unix_to_iso8601(secs)
    }
    #[cfg(test)]
    {
        "2026-06-17T10:30:00Z".to_string()
    }
}

#[cfg(not(test))]
fn unix_to_iso8601(secs: u64) -> String {
    // Days since 1970-01-01
    let total_days = (secs / 86400) as i64;
    let remaining_secs = (secs % 86400) as u32;
    let hours = remaining_secs / 3600;
    let minutes = (remaining_secs % 3600) / 60;
    let seconds = remaining_secs % 60;

    // Convert days to year-month-day
    let (year, month, day) = days_to_ymd(total_days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

#[cfg(not(test))]
fn days_to_ymd(mut days: i64) -> (i64, u32, u32) {
    // Algorithm: convert days since 1970-01-01 to civil date
    days += 719468; // shift epoch to year 0-03-01
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[allow(unused_variables)]
fn chrono_cutoff(minutes_ago: i64) -> String {
    #[cfg(not(test))]
    {
        use std::time::SystemTime;
        let dur = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = dur.as_secs().saturating_sub((minutes_ago * 60) as u64);
        unix_to_iso8601(secs)
    }
    #[cfg(test)]
    {
        "2026-06-17T10:20:00Z".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_detects_threshold() {
        let mut history = ErrorHistory::load();
        let fp = "orders::create::HttpError(422)";
        // Record 3 entries with the same fingerprint
        history.record(fp, "orders create", "HttpError", "validation failed");
        history.record(fp, "orders create", "HttpError", "validation failed");
        history.record(fp, "orders create", "HttpError", "validation failed");
        // Within 10-minute window, threshold 3 should trigger
        assert!(history.check_threshold(fp, 10, 3));
    }

    #[test]
    fn threshold_not_reached_with_insufficient_entries() {
        let mut history = ErrorHistory::load();
        let fp = "inventory::get::ParseError";
        history.record(fp, "inventory get", "ParseError", "bad json");
        history.record(fp, "inventory get", "ParseError", "bad json");
        assert!(!history.check_threshold(fp, 10, 3));
    }

    #[test]
    fn different_fingerprints_dont_combine() {
        let mut history = ErrorHistory::load();
        history.record("a::b", "cmd1", "E1", "msg");
        history.record("a::b", "cmd1", "E1", "msg");
        history.record("x::y", "cmd2", "E2", "msg");
        assert!(!history.check_threshold("a::b", 10, 3));
    }

    #[test]
    fn fifo_truncates_at_50() {
        let mut history = ErrorHistory::load();
        for i in 0..60 {
            history.record(
                &format!("fp_{i}"),
                "cmd",
                "E",
                "msg",
            );
        }
        // Should have only 50 entries, dropping the first 10
        // The first entry should now be fp_10
        let first = &history.data.entries[0];
        assert_eq!(first.fingerprint, "fp_10");
        assert_eq!(history.data.entries.len(), 50);
    }
}
