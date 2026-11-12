//! InputController.handleKeyDown / handleKeyUp. Keys typed into a focused
//! text box never reach this (the root EventTap only offers keys no widget
//! captured — the original returns early for text-entry targets).

use iced::keyboard::{key::Named, Key, Modifiers};

/// A shortcut's effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyCmd {
    /// Pen up, else clear the selection, else `setMode('select')`.
    Escape,
    /// Ctrl+S / F2 — `saveToFile`.
    Save,
    /// Ctrl+O / F3 — the open picker.
    Open,
    /// Ctrl+N / File > New — the new-sketch confirm.
    NewSketch,
    /// F12 / Ctrl+F12 - `saveAs`.
    SaveAs,
    /// F1 - the Keyboard Shortcuts cheat sheet (S12-015).
    Shortcuts,
    /// Ctrl+F1 - the email support (S12-071).
    Email,
    /// Ctrl+A — select all (S6-043).
    SelectAll,
    /// Ctrl+Z — undo (S6-040).
    Undo,
    /// Ctrl+Shift+Z / Ctrl+Y — redo (S6-041, S6-042).
    Redo,
    /// Ctrl+C / Ctrl+X / Ctrl+V — clipboard (S6-030..032).
    Copy,
    Cut,
    Paste,
    /// Delete / Backspace — remove the selection (S6-034).
    Delete,
    DrawExterior,
    DrawInterior,
    ToggleGrid,
    /// `P` (not drawing): `setMode('select')`.
    Select,
    /// `S` / `T` (not drawing): the symbol / text library panel.
    ShowSymbols,
    ShowText,
    RoomShape,
    CommandPalette,
    /// Screen 8: H/V orient for selected symbols.
    OrientH,
    OrientV,
    /// `F4`: the Areas (define-area) panel.
    ShowAreas,
    /// `D` (empty buffer): toggle the curve mode.
    CurveMode,
    /// `C` / `Shift+C` (not drawing).
    CenterSketch,
    CenterFit,
    /// Space pressed: temporary hand tool.
    SpaceDown,
    /// Space released.
    SpaceUp,
    /// A drawing-buffer character: digits, '.', 'x', 'l', 'r', '<' or ','.
    Buffer(&'static str),
    /// `Tab` while drawing: a decimal point in the buffer.
    DecimalPoint,
    /// `Backspace` / `Delete` while drawing with a buffer: drop a character.
    BufferBackspace,
    /// `Enter` while drawing: pen down / commit / pen up.
    Enter,
    /// `Ctrl+F3` (and `A` while the pen is down): auto-close the chain.
    AutoClose,
    /// `B` while the pen is down: bay-window mirror.
    BayMirror,
    /// Insert: arm the Insert Vertex tool.
    InsertVertex,
    /// Shift+R (5) / Ctrl+Shift+R (1) with a selection: `rotateSelected`.
    RotateSelected(f64),
    /// `+` / `-` with a selection: `scaleSelected(1.1 / 0.9)`.
    ScaleSelected(f64),
    /// `F` (horizontal) / `M` (vertical) when not drawing: `flipSketch`.
    FlipSketch(bool),
    /// `L` / `R` when not drawing: `rotateSketch(-90 / 90)`.
    RotateSketch(f64),
    /// An arrow key with a selection, not drawing: `moveSelected(dx, dy)`
    /// (1 ft, Shift 0.1 ft; world y grows upward).
    Nudge(f64, f64),
    /// `J`: jump the cursor to the nearest endpoint.
    JumpEndpoint,
    /// An arrow / diagonal move: `dir` is 0..=7 (up, up-right, right, ...)
    /// and `ctrl` / `shift` select the alignment snaps.
    Arrow {
        dir: u8,
        ctrl: bool,
        shift: bool,
    },
}

/// The arrow / diagonal keys, as the original's key names.
pub const ARROW_KEYS: [(&str, u8); 8] = [
    ("ArrowUp", 0),
    ("PageUp", 1),
    ("ArrowRight", 2),
    ("PageDown", 3),
    ("ArrowDown", 4),
    ("End", 5),
    ("ArrowLeft", 6),
    ("Home", 7),
];

/// The original's key name for a direction (the code uses these strings).
pub fn arrow_key_name(dir: u8) -> &'static str {
    ARROW_KEYS
        .iter()
        .find(|(_, d)| *d == dir)
        .map(|(name, _)| *name)
        .unwrap_or("ArrowUp")
}

/// What the key handler needs to know about the app.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyContext {
    /// `hasSelection` — Delete/Backspace prefer it over the buffer.
    pub has_selection: bool,
    /// `draw-exterior` / `draw-interior`.
    pub drawing: bool,
    /// A modal dialog swallows every key (its capture listener).
    pub dialog_open: bool,
    /// `InputController.buffer` is not empty (Backspace / Delete only take
    /// the buffer when there is something to remove).
    pub buffer: bool,
}

/// The static strings for the digit buffer characters.
const DIGITS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

fn arrow(dir: u8, mods: Modifiers) -> KeyCmd {
    KeyCmd::Arrow {
        dir,
        ctrl: mods.control(),
        shift: mods.shift(),
    }
}

fn is_space(key: &Key) -> bool {
    matches!(key, Key::Named(Named::Space)) || matches!(key, Key::Character(c) if c.as_str() == " ")
}

/// handleKeyDown, in its order of precedence.
pub fn key_down(key: &Key, mods: Modifiers, ctx: KeyContext) -> Option<KeyCmd> {
    if ctx.dialog_open {
        return None;
    }
    let plain = !mods.control() && !mods.alt();
    match key {
        // Ctrl+F3: auto-close (checked before the bare F3 open).
        Key::Named(Named::F3) if mods.control() => return Some(KeyCmd::AutoClose),
        Key::Named(Named::F4) => return Some(KeyCmd::ShowAreas),
        Key::Named(Named::F5) => return Some(KeyCmd::DrawInterior),
        Key::Named(Named::Insert) => return Some(KeyCmd::InsertVertex),
        Key::Named(Named::Escape) => return Some(KeyCmd::Escape),
        // Ctrl+C / Ctrl+X / Ctrl+V (S6-030..032), before the drawing buffer so
        // they are never typed into it.
        Key::Character(c) if mods.control() && c.as_str().eq_ignore_ascii_case("c") => {
            return Some(KeyCmd::Copy)
        }
        Key::Character(c) if mods.control() && c.as_str().eq_ignore_ascii_case("x") => {
            return Some(KeyCmd::Cut)
        }
        Key::Character(c) if mods.control() && c.as_str().eq_ignore_ascii_case("v") => {
            return Some(KeyCmd::Paste)
        }
        // Delete / Backspace remove the selection (S6-034) — but only when
        // something is selected: with an empty selection they fall through to
        // the drawing buffer's own backspace, as the original's
        // `if (hasSelection)` guard does.
        Key::Named(Named::Delete) | Key::Named(Named::Backspace) if ctx.has_selection => {
            return Some(KeyCmd::Delete)
        }
        // Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y (S6-040..042), before the buffer.
        Key::Character(c) if mods.control() && c.as_str().eq_ignore_ascii_case("z") => {
            return Some(if mods.shift() {
                KeyCmd::Redo
            } else {
                KeyCmd::Undo
            })
        }
        Key::Character(c) if mods.control() && c.as_str().eq_ignore_ascii_case("y") => {
            return Some(KeyCmd::Redo)
        }
        // Ctrl+A: select all, before the buffer so it is not typed.
        Key::Character(c) if mods.control() && c.as_str().eq_ignore_ascii_case("a") => {
            return Some(KeyCmd::SelectAll)
        }
        // The file keyboard map: Ctrl+S / Ctrl+O / Ctrl+N and the F-key
        // mirrors (input-controller.js:112-150). F1/Ctrl+F1 (the shortcuts
        // dialog / email) stay unwired: the port has neither dialog yet.
        Key::Character(c) if mods.control() && c.as_str().eq_ignore_ascii_case("s") => {
            return Some(KeyCmd::Save)
        }
        Key::Character(c) if mods.control() && c.as_str().eq_ignore_ascii_case("o") => {
            return Some(KeyCmd::Open)
        }
        Key::Character(c) if mods.control() && c.as_str().eq_ignore_ascii_case("n") => {
            return Some(KeyCmd::NewSketch)
        }
        Key::Named(Named::F12) if mods.control() => return Some(KeyCmd::SaveAs),
        Key::Named(Named::F2) => return Some(KeyCmd::Save),
        Key::Named(Named::F3) => return Some(KeyCmd::Open),
        Key::Named(Named::F12) => return Some(KeyCmd::SaveAs),
        // The cheat sheet (S12-015) / the email support (S12-071). Ctrl+F1
        // checks before the bare F1 (the original's order).
        Key::Named(Named::F1) if mods.control() => return Some(KeyCmd::Email),
        Key::Named(Named::F1) => return Some(KeyCmd::Shortcuts),
        _ => {}
    }
    // The drawing-only keys, in the original's order: the buffer characters
    // and Enter come before the mode shortcuts, so typing "e" or "s" while
    // drawing goes into the buffer rather than switching tools.
    if ctx.drawing {
        match key {
            Key::Named(Named::Tab) => return Some(KeyCmd::DecimalPoint),
            Key::Named(Named::Backspace) | Key::Named(Named::Delete) if ctx.buffer => {
                return Some(KeyCmd::BufferBackspace)
            }
            Key::Named(Named::Enter) => return Some(KeyCmd::Enter),
            Key::Named(Named::ArrowUp) => return Some(arrow(0, mods)),
            Key::Named(Named::PageUp) => return Some(arrow(1, mods)),
            Key::Named(Named::ArrowRight) => return Some(arrow(2, mods)),
            Key::Named(Named::PageDown) => return Some(arrow(3, mods)),
            Key::Named(Named::ArrowDown) => return Some(arrow(4, mods)),
            Key::Named(Named::End) => return Some(arrow(5, mods)),
            Key::Named(Named::ArrowLeft) => return Some(arrow(6, mods)),
            Key::Named(Named::Home) => return Some(arrow(7, mods)),
            _ => {}
        }
        if let Key::Character(c) = key {
            let raw = c.as_str();
            let lower = raw.to_lowercase();
            match lower.as_str() {
                "a" if plain => return Some(KeyCmd::AutoClose),
                "b" if plain => return Some(KeyCmd::BayMirror),
                "j" if plain => return Some(KeyCmd::JumpEndpoint),
                "c" if !mods.control() => {
                    return Some(if mods.shift() {
                        KeyCmd::CenterFit
                    } else {
                        KeyCmd::CenterSketch
                    })
                }
                "x" | "l" | "r" if plain => {
                    return Some(KeyCmd::Buffer(if lower == "x" {
                        "x"
                    } else if lower == "l" {
                        "l"
                    } else {
                        "r"
                    }))
                }
                _ => {}
            }
            if raw.len() == 1 {
                let ch = raw.chars().next().unwrap_or(' ');
                if ch.is_ascii_digit() || ch == '.' {
                    return Some(KeyCmd::Buffer(if ch == '.' {
                        "."
                    } else {
                        DIGITS[ch as usize - 48]
                    }));
                }
                if ch == '<' || ch == ',' {
                    return Some(KeyCmd::Buffer("<"));
                }
            }
        }
    }
    if mods.control() {
        if let Key::Character(c) = key {
            if c.to_lowercase() == "k" {
                return Some(KeyCmd::CommandPalette);
            }
        }
    }
    // The arrows move the selection in any non-drawing mode, so a wall
    // picked with the pointer, fence or lasso can always be nudged.
    if !ctx.drawing && ctx.has_selection && !mods.control() && !mods.alt() {
        let step = if mods.shift() { 0.1 } else { 1.0 };
        match key {
            Key::Named(Named::ArrowUp) => return Some(KeyCmd::Nudge(0.0, step)),
            Key::Named(Named::ArrowDown) => return Some(KeyCmd::Nudge(0.0, -step)),
            Key::Named(Named::ArrowLeft) => return Some(KeyCmd::Nudge(-step, 0.0)),
            Key::Named(Named::ArrowRight) => return Some(KeyCmd::Nudge(step, 0.0)),
            _ => {}
        }
    }
    if let Key::Character(c) = key {
        let c = c.to_lowercase();
        match c.as_str() {
            "g" if !mods.control() => return Some(KeyCmd::ToggleGrid),
            "e" if plain => return Some(KeyCmd::DrawExterior),
            "i" if plain => return Some(KeyCmd::DrawInterior),
            "p" if plain && !ctx.drawing => return Some(KeyCmd::Select),
            "s" if plain && !ctx.drawing => return Some(KeyCmd::ShowSymbols),
            "t" if plain && !ctx.drawing => return Some(KeyCmd::ShowText),
            // The selection transforms, then the whole-sketch ones, in the
            // original's order (input-controller.js:250-280).
            "r" if ctx.has_selection && mods.shift() && mods.control() => {
                return Some(KeyCmd::RotateSelected(1.0))
            }
            "r" if ctx.has_selection && mods.shift() && !mods.control() => {
                return Some(KeyCmd::RotateSelected(5.0))
            }
            "h" if plain => return Some(KeyCmd::OrientH),
            "v" if plain => return Some(KeyCmd::OrientV),
            "+" if ctx.has_selection => return Some(KeyCmd::ScaleSelected(1.1)),
            "-" if ctx.has_selection => return Some(KeyCmd::ScaleSelected(0.9)),
            "f" if plain && !ctx.drawing => return Some(KeyCmd::FlipSketch(true)),
            "m" if plain && !ctx.drawing => return Some(KeyCmd::FlipSketch(false)),
            "l" if plain && !ctx.drawing => return Some(KeyCmd::RotateSketch(-90.0)),
            "r" if plain && !ctx.drawing => return Some(KeyCmd::RotateSketch(90.0)),
            // D only toggles the curve mode with an empty buffer.
            "d" if plain && !ctx.buffer => return Some(KeyCmd::CurveMode),
            "c" if !mods.control() && !ctx.drawing => {
                return Some(if mods.shift() {
                    KeyCmd::CenterFit
                } else {
                    KeyCmd::CenterSketch
                })
            }
            _ => {}
        }
    }
    if is_space(key) {
        return Some(KeyCmd::SpaceDown);
    }
    // Not drawing: Enter starts Draw Exterior.
    if !ctx.drawing && matches!(key, Key::Named(Named::Enter)) {
        return Some(KeyCmd::DrawExterior);
    }
    None
}

/// handleKeyUp: only Space matters.
pub fn key_up(key: &Key, ctx: KeyContext) -> Option<KeyCmd> {
    if ctx.dialog_open {
        return None;
    }
    is_space(key).then_some(KeyCmd::SpaceUp)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(c: &str) -> Key {
        Key::Character(c.into())
    }

    #[test]
    fn mode_and_panel_keys_follow_handle_key_down() {
        let idle = KeyContext::default();
        let drawing = KeyContext {
            drawing: true,
            ..KeyContext::default()
        };
        let none = Modifiers::empty();
        assert_eq!(key_down(&ch("e"), none, idle), Some(KeyCmd::DrawExterior));
        assert_eq!(
            key_down(&ch("E"), Modifiers::SHIFT, idle),
            Some(KeyCmd::DrawExterior)
        );
        assert_eq!(
            key_down(&ch("i"), none, drawing),
            Some(KeyCmd::DrawInterior)
        );
        assert_eq!(
            key_down(&Key::Named(Named::F5), none, idle),
            Some(KeyCmd::DrawInterior)
        );
        assert_eq!(key_down(&ch("g"), none, drawing), Some(KeyCmd::ToggleGrid));
        assert_eq!(key_down(&ch("g"), Modifiers::CTRL, idle), None);
        // Not while drawing: P, S, T, C, Enter.
        assert_eq!(key_down(&ch("p"), none, idle), Some(KeyCmd::Select));
        assert_eq!(key_down(&ch("p"), none, drawing), None);
        assert_eq!(key_down(&ch("s"), none, idle), Some(KeyCmd::ShowSymbols));
        assert_eq!(key_down(&ch("t"), none, idle), Some(KeyCmd::ShowText));
        assert_eq!(key_down(&ch("c"), none, idle), Some(KeyCmd::CenterSketch));
        assert_eq!(
            key_down(&ch("C"), Modifiers::SHIFT, idle),
            Some(KeyCmd::CenterFit)
        );
        // While drawing, C centres (the original's drawing block) and
        // Shift+C fits.
        assert_eq!(
            key_down(&ch("c"), none, drawing),
            Some(KeyCmd::CenterSketch)
        );
        assert_eq!(
            key_down(&ch("C"), Modifiers::SHIFT, drawing),
            Some(KeyCmd::CenterFit)
        );
        assert_eq!(
            key_down(&Key::Named(Named::Enter), none, idle),
            Some(KeyCmd::DrawExterior)
        );
        // While drawing, Enter commits (pen down / wall / pen up).
        assert_eq!(
            key_down(&Key::Named(Named::Enter), none, drawing),
            Some(KeyCmd::Enter)
        );
        assert_eq!(
            key_down(&Key::Named(Named::F4), none, idle),
            Some(KeyCmd::ShowAreas)
        );
        assert_eq!(key_down(&ch("d"), none, idle), Some(KeyCmd::CurveMode));
        assert_eq!(
            key_down(&Key::Named(Named::Escape), none, drawing),
            Some(KeyCmd::Escape)
        );
        assert_eq!(
            key_down(&Key::Named(Named::Space), none, idle),
            Some(KeyCmd::SpaceDown)
        );
        assert_eq!(key_down(&ch(" "), none, idle), Some(KeyCmd::SpaceDown));
        assert_eq!(
            key_up(&Key::Named(Named::Space), idle),
            Some(KeyCmd::SpaceUp)
        );
        assert_eq!(key_up(&ch("e"), idle), None);
        // Drawing keys, in the original's order.
        assert_eq!(
            key_down(&Key::Named(Named::Enter), none, drawing),
            Some(KeyCmd::Enter)
        );
        assert_eq!(key_down(&ch("a"), none, drawing), Some(KeyCmd::AutoClose));
        assert_eq!(key_down(&ch("b"), none, drawing), Some(KeyCmd::BayMirror));
        assert_eq!(
            key_down(&ch("j"), none, drawing),
            Some(KeyCmd::JumpEndpoint)
        );
        // The mode keys still win while drawing (the original's order).
        assert_eq!(
            key_down(&ch("e"), none, drawing),
            Some(KeyCmd::DrawExterior)
        );
        assert_eq!(
            key_down(&ch("i"), none, drawing),
            Some(KeyCmd::DrawInterior)
        );
        assert_eq!(key_down(&ch("s"), none, drawing), None);
        assert_eq!(key_down(&ch("5"), none, drawing), Some(KeyCmd::Buffer("5")));
        assert_eq!(key_down(&ch("."), none, drawing), Some(KeyCmd::Buffer(".")));
        assert_eq!(key_down(&ch("x"), none, drawing), Some(KeyCmd::Buffer("x")));
        assert_eq!(
            key_down(&ch("<"), Modifiers::SHIFT, drawing),
            Some(KeyCmd::Buffer("<"))
        );
        assert_eq!(key_down(&ch(","), none, drawing), Some(KeyCmd::Buffer("<")));
        assert_eq!(
            key_down(&Key::Named(Named::Tab), none, drawing),
            Some(KeyCmd::DecimalPoint)
        );
        assert_eq!(
            key_down(&Key::Named(Named::ArrowUp), none, drawing),
            Some(KeyCmd::Arrow {
                dir: 0,
                ctrl: false,
                shift: false
            })
        );
        assert_eq!(
            key_down(&Key::Named(Named::PageUp), Modifiers::CTRL, drawing),
            Some(KeyCmd::Arrow {
                dir: 1,
                ctrl: true,
                shift: false
            })
        );
        assert_eq!(
            key_down(
                &Key::Named(Named::End),
                Modifiers::CTRL | Modifiers::SHIFT,
                drawing
            ),
            Some(KeyCmd::Arrow {
                dir: 5,
                ctrl: true,
                shift: true
            })
        );
        assert_eq!(
            key_down(&Key::Named(Named::F3), Modifiers::CTRL, idle),
            Some(KeyCmd::AutoClose)
        );
        // Backspace only claims the buffer when there is one.
        let with_buffer = KeyContext {
            drawing: true,
            buffer: true,
            ..KeyContext::default()
        };
        assert_eq!(
            key_down(&Key::Named(Named::Backspace), none, with_buffer),
            Some(KeyCmd::BufferBackspace)
        );
        assert_eq!(key_down(&Key::Named(Named::Backspace), none, drawing), None);
        // A dialog swallows everything.
        let dialog = KeyContext {
            dialog_open: true,
            ..KeyContext::default()
        };
        assert_eq!(key_down(&ch("e"), none, dialog), None);
        assert_eq!(key_down(&Key::Named(Named::Escape), none, dialog), None);
    }
}

#[cfg(test)]
mod clipboard_key_tests {
    use super::*;
    use iced::keyboard::key::Named;

    fn ctx(has_selection: bool, buffer: bool) -> KeyContext {
        KeyContext {
            drawing: true,
            dialog_open: false,
            buffer,
            has_selection,
        }
    }

    #[test]
    fn delete_prefers_the_selection_over_the_buffer() {
        // The original: `if (hasSelection) { deleteSelected(); return; }`
        // before the buffer's own backspace.
        assert_eq!(
            key_down(
                &Key::Named(Named::Delete),
                Modifiers::default(),
                ctx(true, true)
            ),
            Some(KeyCmd::Delete)
        );
        assert_eq!(
            key_down(
                &Key::Named(Named::Backspace),
                Modifiers::default(),
                ctx(true, true)
            ),
            Some(KeyCmd::Delete)
        );
        // Nothing selected: the buffer keeps Backspace, as before.
        assert_eq!(
            key_down(
                &Key::Named(Named::Backspace),
                Modifiers::default(),
                ctx(false, true)
            ),
            Some(KeyCmd::BufferBackspace)
        );
    }

    #[test]
    fn the_clipboard_shortcuts_are_bound() {
        for (ch, want) in [
            ("c", KeyCmd::Copy),
            ("x", KeyCmd::Cut),
            ("v", KeyCmd::Paste),
        ] {
            let key = Key::Character(ch.into());
            assert_eq!(
                key_down(&key, Modifiers::CTRL, ctx(false, false)),
                Some(want)
            );
        }
    }
}
