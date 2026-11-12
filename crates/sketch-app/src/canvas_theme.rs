//! The drawing-background theme - `CanvasEngine.setBackgroundTheme`
//! (canvas-engine.js:68) and `state.canvasTheme`.
//!
//! Separate from the interface theme: it recolours only the canvas
//! background, the two grid weights, the dimension labels and the witness
//! lines. `light` (shown as "White") is the default.
//!
//! Held in a thread-local like `theme::interface_theme`: the view and the
//! canvas draw run on the thread that runs `update`, and the test harness
//! keeps each test's choice on its own thread.

use iced::Color;
use std::cell::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasTheme {
    #[default]
    Light,
    Dark,
    Blue,
}

impl CanvasTheme {
    /// `state.canvasTheme`'s value.
    pub fn key(self) -> &'static str {
        match self {
            CanvasTheme::Light => "light",
            CanvasTheme::Dark => "dark",
            CanvasTheme::Blue => "blue",
        }
    }

    /// `setBackgroundTheme`'s validation: only dark and blue are kept.
    pub fn from_key(key: &str) -> CanvasTheme {
        match key {
            "dark" => CanvasTheme::Dark,
            "blue" => CanvasTheme::Blue,
            _ => CanvasTheme::Light,
        }
    }

    /// The menu label (`showCanvasBackgroundMenu`).
    pub fn label(self) -> &'static str {
        match self {
            CanvasTheme::Light => "White",
            CanvasTheme::Dark => "Dark",
            CanvasTheme::Blue => "Blue",
        }
    }

    pub fn bg(self) -> Color {
        match self {
            CanvasTheme::Light => rgb(0xFFFFFF),
            CanvasTheme::Dark => rgb(0x111111),
            CanvasTheme::Blue => rgb(0x0b1728),
        }
    }

    pub fn grid10(self) -> Color {
        match self {
            CanvasTheme::Light => rgba(30, 41, 59, 0.09),
            CanvasTheme::Dark => rgba(255, 255, 255, 0.05),
            CanvasTheme::Blue => rgba(147, 197, 253, 0.08),
        }
    }

    pub fn grid50(self) -> Color {
        match self {
            CanvasTheme::Light => rgba(30, 41, 59, 0.20),
            CanvasTheme::Dark => rgba(255, 255, 255, 0.12),
            CanvasTheme::Blue => rgba(147, 197, 253, 0.18),
        }
    }

    pub fn dim_label(self) -> Color {
        match self {
            CanvasTheme::Light => rgb(0x475569),
            CanvasTheme::Dark => rgb(0xA0AEC0),
            CanvasTheme::Blue => rgb(0x93C5FD),
        }
    }

    pub fn witness(self) -> Color {
        match self {
            CanvasTheme::Light => rgba(37, 99, 235, 0.38),
            CanvasTheme::Dark => rgba(99, 179, 237, 0.3),
            CanvasTheme::Blue => rgba(99, 179, 237, 0.32),
        }
    }
}

const fn rgb(v: u32) -> Color {
    Color {
        r: ((v >> 16) & 0xff) as f32 / 255.0,
        g: ((v >> 8) & 0xff) as f32 / 255.0,
        b: (v & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a,
    }
}

thread_local! {
    static CURRENT: Cell<CanvasTheme> = const { Cell::new(CanvasTheme::Light) };
}

pub fn current() -> CanvasTheme {
    CURRENT.with(Cell::get)
}

pub fn set(theme: CanvasTheme) {
    CURRENT.with(|c| c.set(theme));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip_and_unknown_is_light() {
        for t in [CanvasTheme::Light, CanvasTheme::Dark, CanvasTheme::Blue] {
            assert_eq!(CanvasTheme::from_key(t.key()), t);
        }
        assert_eq!(CanvasTheme::from_key("sepia"), CanvasTheme::Light);
        assert_eq!(CanvasTheme::from_key(""), CanvasTheme::Light);
    }

    #[test]
    fn the_three_backgrounds_are_the_originals() {
        let hex = |c: Color| {
            format!(
                "{:02x}{:02x}{:02x}",
                (c.r * 255.0).round() as u8,
                (c.g * 255.0).round() as u8,
                (c.b * 255.0).round() as u8
            )
        };
        assert_eq!(hex(CanvasTheme::Light.bg()), "ffffff");
        assert_eq!(hex(CanvasTheme::Dark.bg()), "111111");
        assert_eq!(hex(CanvasTheme::Blue.bg()), "0b1728");
        assert_eq!(hex(CanvasTheme::Blue.dim_label()), "93c5fd");
    }

    #[test]
    fn the_choice_is_per_thread() {
        set(CanvasTheme::Blue);
        assert_eq!(current(), CanvasTheme::Blue);
        std::thread::spawn(|| assert_eq!(current(), CanvasTheme::Light))
            .join()
            .unwrap();
        set(CanvasTheme::Light);
    }
}
