//! Recent Files (S11-049..053) — `getRecentFiles` / `recordRecentFile` /
//! `renderRecentFiles` / `openRecentFile` / `clearRecentFiles`
//! (ui-manager.js:1245-1320).
//!
//! The browser keeps the list in `localStorage['sketch.recentFiles']`; the
//! port keeps it in memory for the session and mirrors it to a sidecar JSON
//! under `%APPDATA%\SketchRust` (main() opts in; the e2e never does, so its
//! behavior is deterministic). Entries embed the model JSON ("cache the
//! model, not the raw bytes"), are deduped by name, newest first, capped at
//! 10, and anything over ~1.5MB is skipped (the original's `1536000` cap).

use serde::{Deserialize, Serialize};

/// One recent entry (`{name, lastOpened, data}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentFile {
    pub name: String,
    #[serde(rename = "lastOpened")]
    pub last_opened: String,
    /// The sketch model's JSON text.
    pub data: String,
    /// Real source for new native entries. Older cache-only entries remain valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<std::path::PathBuf>,
}

/// The localStorage cap: `data.length > 1536000` skips the record.
const DATA_CAP: usize = 1_536_000;
/// `recent.length > 10` truncates.
pub const MAX_FILES: usize = 10;

impl RecentFile {
    pub fn new(name: &str, data: &str) -> Option<Self> {
        if name.is_empty() || data.len() > DATA_CAP {
            return None;
        }
        Some(RecentFile {
            name: name.to_string(),
            last_opened: iso_now(),
            data: data.to_string(),
            path: None,
        })
    }
}

/// `getRecentFiles`' parse: per-entry tolerance (the original filters each
/// entry and requires the `name`/`data` strings; one bad entry does not
/// sink the list).
pub fn parse(raw: &str) -> Vec<RecentFile> {
    serde_json::from_str::<Vec<serde_json::Value>>(raw)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| serde_json::from_value::<RecentFile>(v).ok())
        .filter(|e| !e.name.is_empty() && !e.data.is_empty())
        .collect()
}

/// `recordRecentFile`: dedupe by name, unshift, cap at 10.
pub fn record(recent: &mut Vec<RecentFile>, entry: RecentFile) {
    recent.retain(|e| e.name != entry.name);
    recent.insert(0, entry);
    recent.truncate(MAX_FILES);
}

/// `new Date().toISOString()` — `YYYY-MM-DDTHH:MM:SS.mmmZ` (UTC).
pub fn iso_now() -> String {
    iso_from_unix(unix_millis())
}

fn unix_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Civil-date math (Howard Hinnant's `civil_from_days`) + the clock.
fn iso_from_unix(millis: i64) -> String {
    let secs = millis.div_euclid(1000);
    let ms = millis.rem_euclid(1000);
    let days = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400);
    let (h, m, s) = (
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60,
    );
    // March-based era shift (Hinnant 2013).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mth = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mth <= 2 { y + 1 } else { y };
    format!("{y:04}-{mth:02}-{d:02}T{h:02}:{m:02}:{s:02}.{ms:03}Z")
}

/// `new Date(entry.lastOpened).toLocaleString()` for the entry rows — the
/// en-US-ish `M/D/YYYY, h:mm:ss AM` the capture shows.
pub fn display_time(iso: &str) -> String {
    // Parse back the fields iso_from_unix wrote (a foreign ISO also parses:
    // YYYY-MM-DD + an optional T-time).
    let (date, time) = match iso.split_once('T') {
        Some(pair) => pair,
        None => (iso, ""),
    };
    let mut date_parts = date.split('-');
    let (y, m, d) = match (date_parts.next(), date_parts.next(), date_parts.next()) {
        (Some(y), Some(m), Some(d)) => (y, m, d.trim_end_matches('Z')),
        _ => return iso.to_string(),
    };
    let m = m.parse::<i64>().unwrap_or(1);
    let d = d.parse::<i64>().unwrap_or(1);
    let y = y.parse::<i64>().unwrap_or(0);
    let hms = time.trim_end_matches('Z');
    let hms = hms.split('.').next().unwrap_or("");
    let clock: Vec<&str> = hms.split(':').collect();
    if clock.len() < 3 {
        return format!("{m}/{d}/{y}");
    }
    let h: i64 = clock[0].parse().unwrap_or(0);
    let hour12 = if h % 12 == 0 { 12 } else { h % 12 };
    let am_pm = if h < 12 { "AM" } else { "PM" };
    format!("{m}/{d}/{y}, {hour12}:{}:{} {am_pm}", clock[1], clock[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_dedupes_unshifts_and_caps_at_ten() {
        let mut recent: Vec<RecentFile> = Vec::new();
        for i in 0..12 {
            record(
                &mut recent,
                RecentFile::new(&format!("f{i}.skx"), "{}").unwrap(),
            );
        }
        assert_eq!(recent.len(), 10);
        assert_eq!(recent[0].name, "f11.skx");
        // A re-save of an older name moves it to the front.
        record(&mut recent, RecentFile::new("f5.skx", "{}").unwrap());
        assert_eq!(recent[0].name, "f5.skx");
        assert_eq!(recent.len(), 10);
        // The ~1.5MB cap skips.
        let big = RecentFile::new("big.skx", &"x".repeat(1_536_001));
        assert!(big.is_none());
    }

    #[test]
    fn parse_tolerates_garbage() {
        assert!(parse("null").is_empty());
        assert!(parse("not json").is_empty());
        let ok = parse(r#"[{"name":"a.skx","lastOpened":"","data":"{}"},{"name":"","data":"{}"}]"#);
        assert_eq!(ok.len(), 1, "a nameless entry filters out");
    }

    #[test]
    fn iso_and_display_round_trip() {
        // A known instant: 2026-10-26T00:00:00Z (1767225600 for 2026-01-01
        // + 298 days).
        let iso = iso_from_unix(1_792_972_800_000);
        assert!(iso.starts_with("2026-10-26T"), "{iso}");
        let shown = display_time(&iso);
        assert_eq!(shown, "10/26/2026, 12:00:00 AM", "{shown}");
        // PM + a mid-minute time.
        let shown = display_time("2026-01-05T15:04:09.123Z");
        assert_eq!(shown, "1/5/2026, 3:04:09 PM", "{shown}");
    }
}
