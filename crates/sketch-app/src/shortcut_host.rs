//! Host adaptation for shortcut **labels**, with a thin chord helper that
//! follows iced’s platform primary modifier.
//!
//! ## Chords (idiomatic iced)
//!
//! iced documents [`Modifiers::command`] / [`Modifiers::COMMAND`] as the main
//! hotkey modifier: ⌘ on macOS, Ctrl elsewhere. Widgets (`text_input`,
//! `text_editor`, …) use `modifiers.command()` the same way. Native builds
//! should too — see [`primary`].
//!
//! Mac-only text motions use iced’s [`Modifiers::macos_command`]; word-jump
//! uses [`Modifiers::jump`]. Those stay on the iced APIs directly.
//!
//! ## Labels
//!
//! iced has no built-in shortcut string localizer. We store Windows-style
//! `Ctrl+…` copy and rewrite it for Apple hosts via [`localize_shortcut`].
//!
//! ## Future web (`wasm32`)
//!
//! iced’s `command()` is **compile-time** (`cfg(target_os = "macos")`). A single
//! wasm binary served to Mac and Windows browsers cannot rely on that. When a
//! web target exists, fill in [`detect_web`] and keep using [`primary`] so
//! Mac browsers get Meta and Windows browsers get Ctrl.

use std::borrow::Cow;

use iced::keyboard::Modifiers;

/// Host conventions for shortcut **labels** (and wasm chord overrides).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // both variants are real; native builds only construct one
pub enum ShortcutHost {
    /// ⌘ labels (and Meta chords on wasm).
    Apple,
    /// Ctrl labels (and Ctrl chords on wasm).
    WindowsLike,
}

/// Active host for label localization (and wasm chord overrides).
pub fn current() -> ShortcutHost {
    detect()
}

fn detect() -> ShortcutHost {
    #[cfg(target_os = "macos")]
    {
        ShortcutHost::Apple
    }
    #[cfg(all(not(target_os = "macos"), not(target_arch = "wasm32")))]
    {
        ShortcutHost::WindowsLike
    }
    #[cfg(target_arch = "wasm32")]
    {
        detect_web()
    }
}

/// Future web entry: return [`ShortcutHost::Apple`] for Mac browsers.
#[cfg(target_arch = "wasm32")]
fn detect_web() -> ShortcutHost {
    // Placeholder for JS interop (navigator.userAgentData / platform).
    ShortcutHost::WindowsLike
}

impl ShortcutHost {
    /// iced [`Modifiers::COMMAND`] bit for synthetic input / tests.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn primary_modifiers(self) -> Modifiers {
        match self {
            // Match iced: COMMAND is LOGO on Apple, CTRL elsewhere — but when
            // testing a host explicitly, use the physical bit that host means.
            ShortcutHost::Apple => Modifiers::LOGO,
            ShortcutHost::WindowsLike => Modifiers::CTRL,
        }
    }

    /// Rewrite stored Windows-style shortcut labels for this host.
    ///
    /// On Apple: `Ctrl` → `⌘`, `Shift` → `⇧`, `Alt` → `⌥` so menus/tooltips
    /// match typical Mac desktop apps.
    pub fn localize(self, label: &str) -> Cow<'_, str> {
        match self {
            ShortcutHost::WindowsLike => Cow::Borrowed(label),
            ShortcutHost::Apple => {
                if !(label.contains("Ctrl") || label.contains("Shift") || label.contains("Alt")) {
                    return Cow::Borrowed(label);
                }
                // Order matters: longer tokens first. "Ctrl X" (space form) and
                // "Ctrl+X" both become "⌘X".
                Cow::Owned(
                    label
                        .replace("Ctrl + ", "⌘")
                        .replace("Ctrl+", "⌘")
                        .replace("Ctrl ", "⌘")
                        .replace("Ctrl", "⌘")
                        .replace("Shift + ", "⇧")
                        .replace("Shift+", "⇧")
                        .replace("Shift ", "⇧")
                        .replace("Shift", "⇧")
                        .replace("Alt + ", "⌥")
                        .replace("Alt+", "⌥")
                        .replace("Alt ", "⌥")
                        .replace("Alt", "⌥"),
                )
            }
        }
    }
}

/// Whether the platform primary hotkey modifier is held.
///
/// Native: iced’s [`Modifiers::command`] (⌘ on macOS, Ctrl elsewhere).
/// Wasm: host from [`detect_web`] so Mac browsers can use Meta.
#[inline]
pub fn primary(mods: Modifiers) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        match current() {
            ShortcutHost::Apple => mods.logo(),
            ShortcutHost::WindowsLike => mods.control(),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        mods.command()
    }
}

/// Localize a `Ctrl+…` label for the active host.
#[inline]
pub fn localize_shortcut(label: &str) -> Cow<'_, str> {
    current().localize(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apple_localizes_ctrl_labels() {
        assert_eq!(ShortcutHost::Apple.localize("Ctrl + K").as_ref(), "⌘K");
        assert_eq!(
            ShortcutHost::Apple.localize("Open (Ctrl+O)").as_ref(),
            "Open (⌘O)"
        );
        assert_eq!(ShortcutHost::Apple.localize("Ctrl+Shift+S").as_ref(), "⌘⇧S");
        assert_eq!(
            ShortcutHost::Apple.localize("Shift+Enter").as_ref(),
            "⇧Enter"
        );
        assert_eq!(ShortcutHost::Apple.localize("Ctrl X").as_ref(), "⌘X");
        assert_eq!(ShortcutHost::Apple.localize("F1").as_ref(), "F1");
    }

    #[test]
    fn windows_like_keeps_ctrl_labels() {
        assert_eq!(
            ShortcutHost::WindowsLike.localize("Ctrl + K").as_ref(),
            "Ctrl + K"
        );
    }

    #[test]
    fn primary_matches_iced_command_on_native() {
        // Same contract as iced widgets: COMMAND bit is the hotkey modifier.
        assert_eq!(primary(Modifiers::COMMAND), Modifiers::COMMAND.command());
        assert!(primary(Modifiers::COMMAND));
        assert!(!primary(Modifiers::empty()));
    }

    #[test]
    fn native_detect_matches_compile_target() {
        #[cfg(target_os = "macos")]
        assert_eq!(detect(), ShortcutHost::Apple);
        #[cfg(all(not(target_os = "macos"), not(target_arch = "wasm32")))]
        assert_eq!(detect(), ShortcutHost::WindowsLike);
    }

    #[test]
    fn primary_modifiers_align_with_iced_command_on_native_host() {
        assert_eq!(current().primary_modifiers(), Modifiers::COMMAND);
    }
}
