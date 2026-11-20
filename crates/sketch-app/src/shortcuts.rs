//! The Keyboard Shortcuts cheat sheet (S12-015) — `showShortcuts`
//! (ui-manager.js): the 7 group cards in a `minmax(220px, 1fr)` grid on a
//! 760px dialog; Close / Escape / F1 / a backdrop press closes it.
//!
//! Measurements (theme.css 1302-1326): the dialog `min(760px, 100vw-32)`,
//! max-height `100vh-48`, gap 10; the grid `auto-fit minmax(220px, 1fr)`,
//! gap 12; the group cards on `--bg-app`, 1px `--border`, radius 3,
//! padding 8/10; the group title 11px/600 uppercase muted; the rows 11.5px
//! with a 96px-min mono key chip (1px border, 2px bottom border, radius 3).

/// The keyboard shortcuts cheat sheet groups, written in the current UI
/// vocabulary while preserving every keybinding.
pub const GROUPS: &[(&str, &[(&str, &str)])] = &[
    (
        "File",
        &[
            ("Ctrl + N", "New sketch"),
            ("Ctrl + O or F3", "Open file"),
            ("Ctrl + S or F2", "Save sketch"),
            ("Ctrl + F12 or F12", "Save a copy as"),
            ("Ctrl + F1", "Email support"),
        ],
    ),
    (
        "Edit",
        &[
            ("Ctrl + Z", "Undo"),
            ("Ctrl + Shift + Z or Ctrl + Y", "Redo"),
            ("Ctrl + X / C / V", "Cut / Copy / Paste"),
            ("Ctrl + A", "Select all"),
            ("Delete", "Remove selection"),
            ("Esc", "Stop drawing / close menu or sheet / clear selection"),
            ("Arrow keys", "Nudge selection by the snap pitch (Shift = 10x)"),
        ],
    ),
    (
        "Drawing",
        &[
            ("E", "Wall tool (exterior)"),
            ("I or F5", "Inner wall tool"),
            ("Enter", "Start exterior wall / commit typed length"),
            ("10 then arrow", "Draw 10 ft in arrow direction"),
            ("10x5", "Rise 10, run 5 diagonal"),
            ("10<45", "10 ft at 45 degrees"),
            ("10l / 10r", "Turn left / right (default 90 degrees)"),
            ("Tab", "Decimal point while typing"),
            ("A", "Close area back to start"),
            ("B", "Bay window mirror"),
            ("J", "Jump cursor to nearest corner"),
            ("D", "Toggle curve height / angle mode"),
            ("Insert", "Insert vertex on wall"),
            ("Backspace", "Erase last typed character"),
        ],
    ),
    (
        "Measure",
        &[
            ("Click", "Add point to measure run"),
            ("Shift + drag", "Lock segment to 45 degrees"),
            ("Enter / double-click", "Finish measure run"),
            ("Shift + Enter", "Close measure run and read area"),
            ("Backspace", "Undo last measure point"),
            ("Delete", "Clear measurements"),
            ("Ctrl + C", "Copy measurements as text"),
            ("Esc", "Cancel run, clear, then switch to Select"),
        ],
    ),
    (
        "Selection",
        &[
            ("P", "Select tool"),
            ("Shift + R", "Rotate selection 5 degrees"),
            ("Ctrl + Shift + R", "Rotate selection 1 degree"),
            ("H / V", "Orient selection horizontal / vertical"),
            ("+ / -", "Scale selection up / down"),
            ("Space (tap)", "Toggle wall dimension labels"),
        ],
    ),
    (
        "View",
        &[
            ("G", "Toggle grid"),
            ("C", "Center drawing"),
            ("Shift + C", "Center & fit all"),
            ("Space (hold)", "Temporary Pan tool"),
            ("F / M", "Flip sketch horizontally / vertically"),
            ("L / R", "Rotate sketch left / right 90 degrees"),
        ],
    ),
    (
        "Workspace",
        &[
            ("Ctrl + K", "Search commands"),
            ("F4", "Toggle Areas drawer"),
            ("F6", "Open Areas drawer totals"),
            ("S", "Open Insert drawer (Symbols)"),
            ("T", "Open Insert drawer (Room labels)"),
            ("F1", "Open Keyboard Shortcuts"),
        ],
    ),
];

/// The overlay's state (open or not — the sheet is static).
#[derive(Debug, Clone, Default)]
pub struct Shortcuts {
    pub open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutsMsg {
    Open,
    Close,
}
