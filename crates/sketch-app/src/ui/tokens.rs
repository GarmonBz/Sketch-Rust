//! Design tokens for the Next shell (`docs/ui-redesign/01-design-spec.md`).
//!
//! Colour tokens extend `crate::theme`'s Dark, Blue, and Light palettes.
//! Spacing, radii, type sizes, and `Density` metrics replace fixed pixel coordinates.

use iced::{Color, Shadow, Vector};

use crate::theme::{self, hex, InterfaceTheme};

// ---------------------------------------------------------------------------
// Colour tokens
// ---------------------------------------------------------------------------

/// Top bar, pop-ups, sheets, tray (`#2D2D30` / `#263F66` / `#DDE3EA`).
pub fn bar() -> Color {
    theme::bg_bar()
}

/// Ledger, inspector, dialogs (`#252526` / `#1E3150` / `#F7F8FA`).
pub fn panel() -> Color {
    theme::bg_panel()
}

/// Active tab, selected tile, non-input field (`#1E1E1E` / `#17243A` / `#E9EDF2`).
pub fn field() -> Color {
    theme::bg_app()
}

/// Text inputs (`#1E1E1E` / `#17243A` / `#FFFFFF` in Light per spec).
pub fn input_bg() -> Color {
    match theme::interface_theme() {
        InterfaceTheme::Light => Color::WHITE,
        _ => theme::bg_app(),
    }
}

/// Hovered and pressed rows and buttons (`#3E3E42` / `#34547F` / `#CFD8E3`).
pub fn hover() -> Color {
    theme::bg_hover()
}

/// Pressed surface (blended slightly toward accent for tactile feedback).
pub fn pressed() -> Color {
    theme::over(accent(), 0.22, hover())
}

/// Active tool, selection, primary button (`#007ACC` / `#2B8AD6` / `#1677BD`).
pub fn accent() -> Color {
    theme::accent()
}

/// Subtle accent wash for selected list rows and chips.
pub fn accent_soft() -> Color {
    theme::over(accent(), 0.16, panel())
}

/// 1px separators and outlines (`#3F3F46` / `#4A668C` / `#AAB4C0`).
pub fn rule() -> Color {
    theme::border()
}

/// Primary text (`#D4D4D4` / `#D4D4D4` / `#20252B`).
pub fn text() -> Color {
    theme::text_alt()
}

/// Secondary text (`#9A9A9A` / `#9A9A9A` / `#586474`).
pub fn muted() -> Color {
    theme::muted_alt()
}

/// Disabled text and icons.
pub fn disabled() -> Color {
    theme::over(text(), 0.42, panel())
}

/// Drawing surface background (`#111111` / `#0B1728` / `#FFFFFF`).
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
}
