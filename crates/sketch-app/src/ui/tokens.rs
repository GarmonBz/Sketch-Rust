//! Design tokens for the Next shell (`docs/ui-redesign/01-design-spec.md`).
//!
//! Colour tokens extend `crate::theme`'s Dark, Blue, and Light palettes.
//! Spacing, radii, type sizes, and `Density` metrics replace fixed pixel coordinates.

use iced::{Color, Shadow, Vector};

use crate::theme::{self, hex, InterfaceTheme};

// ---------------------------------------------------------------------------
// Colour tokens
// ---------------------------------------------------------------------------

/// Pop-ups, sheets, tray (`#181C23` / `#263F66` / `#DDE3EA`).
pub fn bar() -> Color {
    theme::bg_bar()
}

/// The top bar (`#0A0E15` / `#263F66` / `#DDE3EA`): the deepest surface in
/// Dark, the same as `bar` in the other themes.
pub fn chrome() -> Color {
    match theme::interface_theme() {
        InterfaceTheme::Dark => theme::DARK_CHROME,
        _ => theme::bg_bar(),
    }
}

/// Ledger, inspector, dialogs (`#181C23` / `#1E3150` / `#F7F8FA`).
pub fn panel() -> Color {
    theme::bg_panel()
}

/// Active tab, selected tile, non-input field (`#20252E` / `#17243A` / `#E9EDF2`).
pub fn field() -> Color {
    theme::bg_app()
}

/// Text inputs (`#20252E` / `#17243A` / `#FFFFFF` in Light per spec).
pub fn input_bg() -> Color {
    match theme::interface_theme() {
        InterfaceTheme::Light => Color::WHITE,
        _ => theme::bg_app(),
    }
}

/// Hovered and pressed rows and buttons (`#353941` / `#34547F` / `#CFD8E3`).
pub fn hover() -> Color {
    theme::bg_hover()
}

/// Pressed surface (blended slightly toward accent for tactile feedback).
pub fn pressed() -> Color {
    theme::over(accent(), 0.22, hover())
}

/// Active tool, selection, primary button (`#0091FF` / `#2B8AD6` / `#1677BD`).
pub fn accent() -> Color {
    theme::accent()
}

/// Subtle accent wash for selected list rows and chips.
pub fn accent_soft() -> Color {
    theme::over(accent(), 0.16, panel())
}

/// 1px separators and outlines (`#282F3B` / `#4A668C` / `#AAB4C0`).
pub fn rule() -> Color {
    theme::border()
}

/// Primary text (`#F0F4F8` / `#D4D4D4` / `#20252B`).
pub fn text() -> Color {
    theme::text_alt()
}

/// Secondary text (`#8B98A5` / `#9A9A9A` / `#586474`).
pub fn muted() -> Color {
    theme::muted_alt()
}

/// Disabled text and icons.
pub fn disabled() -> Color {
    theme::over(text(), 0.42, panel())
}

/// Drawing surface background (`#0F141A` / `#0B1728` / `#FFFFFF`).
pub fn canvas() -> Color {
    theme::bg_canvas()
}

/// Text buttons on panels (`#63B3ED` / `#63B3ED` / `#1677BD`).
pub fn link() -> Color {
    match theme::interface_theme() {
        InterfaceTheme::Light => hex(0x1677bd),
        InterfaceTheme::Dark | InterfaceTheme::Blue => hex(0x63b3ed),
    }
}

/// Length strip, Pad readout, and live placement previews only (`#F6AD55`).
pub fn tape() -> Color {
    hex(0xf6ad55)
}

/// Text on the tape colour (`#1E1E1E`).
pub fn tape_ink() -> Color {
    hex(0x1e1e1e)
}

/// "Needs a type" and destructive text (`#FC8181` / `#FC8181` / `#C53030`).
pub fn flag() -> Color {
    match theme::interface_theme() {
        InterfaceTheme::Light => hex(0xc53030),
        InterfaceTheme::Dark | InterfaceTheme::Blue => hex(0xfc8181),
    }
}

/// Icon tints, by meaning. These are the classic ribbon and tool-rail
/// colours (`ICON_BLUE`, `ICON_ORANGE`, `ICON_RED`, `ICON_SLATE` and the
/// green of the old View and Reference Image tabs), restored at the owner's
/// request on 2026-11-17. Light gets deeper shades of the same hues so the
/// icons keep their contrast on the pale panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconTint {
    /// Select, view, navigate, information.
    Blue,
    /// Draw, edit, measure, annotate.
    Orange,
    /// Remove, hide, and the Area tool.
    Red,
    /// Files, add, export, print.
    Green,
    /// Devices, locks and history.
    Slate,
}

pub fn icon_tint(tint: IconTint) -> Color {
    let light = matches!(theme::interface_theme(), InterfaceTheme::Light);
    match (tint, light) {
        (IconTint::Blue, false) => hex(0x63b3ed),
        (IconTint::Blue, true) => hex(0x1d6fb8),
        (IconTint::Orange, false) => hex(0xf6ad55),
        (IconTint::Orange, true) => hex(0xc05621),
        (IconTint::Red, false) => hex(0xfc8181),
        (IconTint::Red, true) => hex(0xc53030),
        (IconTint::Green, false) => hex(0x5dae84),
        (IconTint::Green, true) => hex(0x2f7d4f),
        (IconTint::Slate, false) => hex(0xa0aec0),
        (IconTint::Slate, true) => hex(0x4a5568),
    }
}

/// Text and icons on accent (`#FFFFFF`).
pub fn on_accent() -> Color {
    Color::WHITE
}

/// Soft shadow for floating surfaces (tray, pop-ups, Pad, slide-overs).
pub fn floating_shadow() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.28),
        offset: Vector::new(0.0, 4.0),
        blur_radius: 16.0,
    }
}

/// Modal backdrop scrim over inactive surfaces.
pub fn scrim() -> Color {
    Color::from_rgba(0.0, 0.0, 0.0, 0.36)
}

// ---------------------------------------------------------------------------
// Spacing & radii
// ---------------------------------------------------------------------------

pub const SPACE_1: f32 = 4.0;
pub const SPACE_2: f32 = 8.0;
pub const SPACE_3: f32 = 12.0;
pub const SPACE_4: f32 = 16.0;
#[allow(dead_code)]
pub const SPACE_6: f32 = 24.0;

pub const RADIUS_CONTROL: f32 = 6.0;
pub const RADIUS_TILE: f32 = 8.0;
pub const RADIUS_DIALOG: f32 = 10.0;
pub const RADIUS_TRAY: f32 = 14.0;
pub const RADIUS_SHEET: f32 = 16.0;

pub const FOCUS_RING_WIDTH: f32 = 2.0;

// ---------------------------------------------------------------------------
// Type sizes (logical px)
// ---------------------------------------------------------------------------

pub const TEXT_11: f32 = 11.0;
pub const TEXT_12: f32 = 12.0;
pub const TEXT_13: f32 = 13.0;
pub const TEXT_14: f32 = 14.0;
#[allow(dead_code)]
pub const TEXT_15: f32 = 15.0;
pub const TEXT_16: f32 = 16.0;

#[allow(dead_code)]
pub const NUM_12: f32 = 12.0;
#[allow(dead_code)]
pub const NUM_13: f32 = 13.0;
#[allow(dead_code)]
pub const NUM_14: f32 = 14.0;
#[allow(dead_code)]
pub const NUM_16: f32 = 16.0;
#[allow(dead_code)]
pub const NUM_18: f32 = 18.0;
#[allow(dead_code)]
pub const NUM_20: f32 = 20.0;
pub const NUM_24: f32 = 24.0;
#[allow(dead_code)]
pub const NUM_32: f32 = 32.0;

// ---------------------------------------------------------------------------
// Density
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    #[default]
    Comfortable,
    Touch,
}

impl Density {
    pub const fn bar_h(self) -> f32 {
        match self {
            Density::Comfortable => 44.0,
            Density::Touch => 56.0,
        }
    }

    pub const fn row_h(self) -> f32 {
        match self {
            Density::Comfortable => 36.0,
            Density::Touch => 48.0,
        }
    }

    pub const fn field_h(self) -> f32 {
        match self {
            Density::Comfortable => 32.0,
            Density::Touch => 44.0,
        }
    }

    pub const fn tray_button(self) -> (f32, f32) {
        match self {
            Density::Comfortable => (62.0, 58.0),
            Density::Touch => (68.0, 68.0),
        }
    }

    pub const fn min_target(self) -> f32 {
        match self {
            Density::Comfortable => 28.0,
            Density::Touch => 48.0,
        }
    }

    pub const fn ledger_w(self) -> f32 {
        match self {
            Density::Comfortable => 264.0,
            Density::Touch => 300.0,
        }
    }

    pub const fn inspector_w(self) -> f32 {
        match self {
            Density::Comfortable => 280.0,
            Density::Touch => 320.0,
        }
    }

    pub const fn icon_size(self) -> f32 {
        match self {
            Density::Comfortable => 20.0,
            Density::Touch => 22.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_and_density_metrics_match_design_spec() {
        assert_eq!(Density::Comfortable.bar_h(), 44.0);
        assert_eq!(Density::Touch.bar_h(), 56.0);
        assert_eq!(Density::Comfortable.row_h(), 36.0);
        assert_eq!(Density::Touch.row_h(), 48.0);
        assert_eq!(Density::Comfortable.field_h(), 32.0);
        assert_eq!(Density::Touch.field_h(), 44.0);
        assert_eq!(Density::Comfortable.tray_button(), (62.0, 58.0));
        assert_eq!(Density::Touch.tray_button(), (68.0, 68.0));
        assert_eq!(Density::Comfortable.min_target(), 28.0);
        assert_eq!(Density::Touch.min_target(), 48.0);

        for iface in [
            InterfaceTheme::Dark,
            InterfaceTheme::Blue,
            InterfaceTheme::Light,
        ] {
            theme::set_interface_theme(iface);
            assert_eq!(tape(), hex(0xf6ad55));
            assert_eq!(tape_ink(), hex(0x1e1e1e));
            assert_eq!(on_accent(), Color::WHITE);
        }
        theme::set_interface_theme(InterfaceTheme::Dark);
    }

    /// The Dark theme uses the field precision CAD palette (2026-11-17).
    #[test]
    fn dark_theme_uses_the_cad_palette() {
        theme::set_interface_theme(InterfaceTheme::Dark);
        assert_eq!(accent(), hex(0x0091ff));
        assert_eq!(canvas(), hex(0x0f141a));
        assert_eq!(chrome(), hex(0x0a0e15));
        assert_eq!(panel(), hex(0x181c23));
        assert_eq!(bar(), hex(0x181c23));
        assert_eq!(field(), hex(0x20252e));
        assert_eq!(hover(), hex(0x353941));
        assert_eq!(rule(), hex(0x282f3b));
        assert_eq!(text(), hex(0xf0f4f8));
        assert_eq!(muted(), hex(0x8b98a5));
        assert_eq!(flag(), hex(0xfc8181));
        assert_eq!(tape(), hex(0xf6ad55));
        // Blue keeps its own surfaces; only Dark changed.
        theme::set_interface_theme(InterfaceTheme::Blue);
        assert_eq!(chrome(), bar());
        assert_eq!(accent(), hex(0x2b8ad6));
        theme::set_interface_theme(InterfaceTheme::Dark);
    }
}
