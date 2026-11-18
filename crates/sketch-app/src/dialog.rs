//! The app's modal dialog — UIManager.showDialog (alertDialog /
//! confirmDialog / promptDialog), measured from the live original:
//! backdrop `rgba(0,0,0,.46)`; dialog 380 wide, centred (510,348 for a
//! prompt), 1px border, radius 5, `--bg-panel`, padding 14/16, gap 10,
//! shadow `0 16px 46px rgba(0,0,0,.5)`; title 13px/600 (h17), message
//! 12px line 1.45 (h17); input 16px, padding 6/8, h35, radius 2,
//! `--bg-app`; actions right-aligned, gap 8, margin-top 2; buttons 16px,
//! padding 5/12, radius 2, min-width 72 (Cancel 73 wide, OK 72),
//! Cancel `--bg-hover`, OK `--bg-active` + white.
//!
//! Keys (document capture listener): Escape = Cancel (OK when there is no
//! Cancel), Enter = OK; a press on the backdrop = the same as Escape.

/// `.app-dialog-input`'s widget id (focused + selected on open).
pub const INPUT_ID: &str = "app-dialog-input";

/// What OK does.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// An alert: nothing.
    None,
    DeleteAreaType(String),
    /// `newSketch()` after "Start a new sketch? Unsaved changes will be
    /// lost." (Ctrl+N / File > New).
    New,
    /// `setNorthAngle(value)`.
    SetNorth,
    /// `addPage()` (Next on the last page).
    AddPage,
    /// `deletePage()` after "Delete page N?".
    DeletePage,
    /// A text label's "Text:" prompt (double-click): the label's id.
    EditText(i64),
    /// `deleteSelected()` after "Delete the selected sketch items?"
    /// (`options.promptOnDeletion`).
    DeleteSelected,
    /// A wall-shape prompt's value (showWallShapeMenu's chain).
    ShapeValue,
    /// The reference image's Set Scale prompt (the known feet value).
    PhotoScale,
    /// The Orient Guide's Horizontal/Vertical prompt (the input decides).
    PhotoOrient,
    /// Remove the background image's confirm.
    PhotoRemove,
    /// The Geo prompts (`showGeoReferenceDialog`).
    GeoCoordinate,
    GeoAddress,
    /// Save the linked sketch to the report and close after acknowledgment.
    /// Closing with unsaved changes: Save (then close), Don't save, Cancel.
    SaveAndExit,
    /// Open the sketch file dropped on the window, discarding the changes.
    OpenDropped(std::path::PathBuf),
    /// Rename the document title displayed in the top bar.
    RenameDocument,
    /// Rename page at index `usize`.
    RenamePage(usize),
}

#[derive(Debug, Clone)]
pub struct Dialog {
    pub title: String,
    pub message: String,
    /// `Some(value)` for a prompt.
    pub input: Option<String>,
    pub has_cancel: bool,
    pub on_ok: Action,
}

impl Dialog {
    pub fn alert(message: &str) -> Self {
        Dialog {
            title: "SketchQuoder".into(),
            message: message.into(),
            input: None,
            has_cancel: false,
            on_ok: Action::None,
        }
    }

    pub fn confirm(message: &str, on_ok: Action) -> Self {
        Dialog {
            has_cancel: true,
            on_ok,
            ..Dialog::alert(message)
        }
    }

    pub fn save_before_close(file_name: &str) -> Self {
        Self {
            title: "Close sketch".into(),
            ..Self::confirm(
                &format!("Save changes to \"{file_name}\" before closing?"),
                Action::SaveAndExit,
            )
        }
    }

    pub fn prompt(message: &str, value: &str, on_ok: Action) -> Self {
        Dialog {
            input: Some(value.into()),
            ..Dialog::confirm(message, on_ok)
        }
    }
}

#[derive(Debug, Clone)]
pub enum DialogMsg {
    Input(String),
    Ok,
    Cancel,
    Discard,
    /// Escape or a backdrop press: Cancel, or OK when there is no Cancel.
    Dismiss,
}
