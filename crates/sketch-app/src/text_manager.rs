//! The Text Library manager — `showTextLibraryManager` (ui-manager.js:359):
//! a `.sidebar-manager-overlay` dialog with the label list (select size=14),
//! the label input, the alpha range rail and Add / Remove / Replace.
//!
//! Measurements (theme.css 1213-1248): the overlay dims `rgba(0,0,0,.46)`;
//! the dialog is `min(430px, 100vw-32)` wide, gap 10, padding 12, radius 5,
//! `--bg-panel`; the grid splits `minmax(0,1fr) | 52px` with a 300px min
//! height; the list and input sit on `--bg-app` with 1px `--border`; the
//! alpha rail and the action buttons are `--bg-hover` chips, the active
//! range `#D69E2E` on `#171717` with a `#F6AD55` border. A press on the
//! backdrop closes (the original's `event.target === overlay`).

/// The alpha rail, in the original's order (`manager-alpha`).
pub const RANGES: &[(&str, &str)] = &[
    ("All", "all"),
    ("ABC", "abc"),
    ("DEF", "def"),
    ("GHI", "ghi"),
    ("JKL", "jkl"),
    ("MNO", "mno"),
    ("PQR", "pqr"),
    ("STU", "stu"),
    ("VWX", "vwx"),
    ("YZ", "yz"),
];

/// The label input's widget id (focused on open, like the original's
/// `input.focus()`).
pub const INPUT_ID: &str = "text-manager-input";

/// The dialog's local state (`#text-library-manager`).
#[derive(Debug, Clone, Default)]
pub struct TextManager {
    pub open: bool,
    /// `range` — `'all'` or a lowercase letter run (`'abc'`).
    pub range: &'static str,
    /// The label input's live buffer.
    pub buffer: String,
    /// The list's selected label.
    pub selected: Option<String>,
}

/// `refresh()`'s list: the labels in the range, `localeCompare`-sorted.
pub fn visible_labels(labels: &[String], range: &str) -> Vec<String> {
    let mut items: Vec<String> = labels
        .iter()
        .filter(|label| {
            range == "all"
                || range.contains(
                    &label
                        .trim()
                        .chars()
                        .next()
                        .unwrap_or_default()
                        .to_lowercase()
                        .to_string(),
                )
        })
        .cloned()
        .collect();
    items.sort_by_key(|a| a.to_lowercase());
    items
}

impl TextManager {
    /// Open over the live library (`this.textLibraryLabels` stays the dock's
    /// `text_labels` — the dialog edits it in place).
    pub fn open(&mut self) {
        self.open = true;
        self.range = "all";
        self.buffer = String::new();
        self.selected = None;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.buffer = String::new();
        self.selected = None;
    }

    /// The dialog's own controls; the App arm returns the toasts the
    /// original's `notify()` shows.
    pub fn update(&mut self, msg: TextManagerMsg, labels: &mut Vec<String>) -> Option<String> {
        match msg {
            TextManagerMsg::Range(range) => self.range = range,
            TextManagerMsg::Input(value) => self.buffer = value,
            TextManagerMsg::Pick(label) => {
                self.selected = Some(label.clone());
                // `list.addEventListener('change', () => input.value = list.value)`.
                self.buffer = label;
            }
            TextManagerMsg::Add => {
                let value = self.buffer.trim().to_string();
                if value.is_empty() {
                    return Some("Enter a text label first.".to_string());
                }
                if labels
                    .iter()
                    .any(|l| l.to_lowercase() == value.to_lowercase())
                {
                    return Some("That label already exists.".to_string());
                }
                labels.push(value.clone());
                self.save(labels, Some(value));
            }
            TextManagerMsg::Remove => {
                let Some(value) = self.selected.clone() else {
                    return Some("Select a label to remove.".to_string());
                };
                labels.retain(|l| *l != value);
                self.buffer = String::new();
                self.save(labels, None);
            }
            TextManagerMsg::Replace => {
                let old = self.selected.clone().unwrap_or_default();
                let value = self.buffer.trim().to_string();
                if old.is_empty() || value.is_empty() {
                    return Some("Select a label and enter its replacement.".to_string());
                }
                if labels
                    .iter()
                    .any(|l| *l != old && l.to_lowercase() == value.to_lowercase())
                {
                    return Some("That label already exists.".to_string());
                }
                for l in labels.iter_mut() {
                    if *l == old {
                        *l = value.clone();
                    }
                }
                self.save(labels, Some(value));
            }
            TextManagerMsg::Open | TextManagerMsg::Close => {}
        }
        None
    }

    /// `save(selected)`: dedupe + sort, persist, and keep the picked label.
    fn save(&mut self, labels: &mut Vec<String>, selected: Option<String>) {
        let mut seen = std::collections::HashSet::new();
        labels.retain(|l| seen.insert(l.to_lowercase()));
        labels.sort_by_key(|a| a.to_lowercase());
        self.selected = selected;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextManagerMsg {
    Open,
    Close,
    Range(&'static str),
    Input(String),
    Pick(String),
    Add,
    Remove,
    Replace,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> Vec<String> {
        ["Living Room", "kitchen", "Half Bath", "Basement"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn the_range_filters_by_the_first_letter_and_sorts() {
        let all = visible_labels(&labels(), "all");
        // `localeCompare` is case-insensitive at these boundaries.
        assert_eq!(all, vec!["Basement", "Half Bath", "kitchen", "Living Room"]);
        let k = visible_labels(&labels(), "k");
        assert_eq!(k, vec!["kitchen"]);
        // The trim precedes the letter test (the original trims then reads
        // charAt(0)).
        let h = visible_labels(&labels(), "h");
        assert_eq!(h, vec!["Half Bath"]);
    }

    #[test]
    fn add_trims_refuses_blank_and_duplicates() {
        let mut m = TextManager {
            open: true,
            ..TextManager::default()
        };
        let mut lib = labels();
        // Blank → the toast, nothing added.
        m.buffer = "   ".to_string();
        assert_eq!(
            m.update(TextManagerMsg::Add, &mut lib),
            Some("Enter a text label first.".to_string())
        );
        assert_eq!(lib.len(), 4);
        // A case-insensitive duplicate is refused...
        m.buffer = "KITCHEN".to_string();
        assert_eq!(
            m.update(TextManagerMsg::Add, &mut lib),
            Some("That label already exists.".to_string())
        );
        // ...a fresh label lands trimmed, sorted, selected.
        m.buffer = "  Pantry  ".to_string();
        assert_eq!(m.update(TextManagerMsg::Add, &mut lib), None);
        assert_eq!(m.selected.as_deref(), Some("Pantry"));
        assert_eq!(
            visible_labels(&lib, "all"),
            vec!["Basement", "Half Bath", "kitchen", "Living Room", "Pantry"]
        );
    }

    #[test]
    fn replace_maps_and_remove_deletes() {
        let mut m = TextManager {
            open: true,
            ..TextManager::default()
        };
        let mut lib = labels();
        // Remove without a selection toasts.
        assert_eq!(
            m.update(TextManagerMsg::Remove, &mut lib),
            Some("Select a label to remove.".to_string())
        );
        // Pick (the change handler syncs the input) then remove.
        m.update(TextManagerMsg::Pick("Basement".to_string()), &mut lib);
        assert_eq!(m.buffer, "Basement");
        assert_eq!(m.update(TextManagerMsg::Remove, &mut lib), None);
        assert!(!lib.contains(&"Basement".to_string()));
        assert!(m.buffer.is_empty());
        // Replace needs a selection AND a value.
        assert_eq!(
            m.update(TextManagerMsg::Replace, &mut lib),
            Some("Select a label and enter its replacement.".to_string())
        );
        // Pick again; the replacement maps the old label (trimmed, deduped,
        // sorted) and keeps it selected.
        m.update(TextManagerMsg::Pick("kitchen".to_string()), &mut lib);
        m.buffer = "  Pantry  ".to_string();
        assert_eq!(m.update(TextManagerMsg::Replace, &mut lib), None);
        assert_eq!(m.selected.as_deref(), Some("Pantry"));
        assert!(lib.contains(&"Pantry".to_string()) && !lib.contains(&"kitchen".to_string()));
    }
}
