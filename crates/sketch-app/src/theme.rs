//! Palette, fonts and small layout helpers shared across the application.

use iced::font::{Family, Style, Weight};
use std::cell::Cell;

use iced::{Color, Font};

/// `#rrggbb` → [`Color`], usable in consts.
pub const fn hex(v: u32) -> Color {
    Color {
        r: ((v >> 16) & 0xff) as f32 / 255.0,
        g: ((v >> 8) & 0xff) as f32 / 255.0,
        b: (v & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

/// The interface theme (`dark` / `blue` / `light`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InterfaceTheme {
    #[default]
    Dark,
    Blue,
    Light,
}

impl InterfaceTheme {
    /// The persisted / menu key.
    pub fn key(self) -> &'static str {
        match self {
            InterfaceTheme::Dark => "dark",
            InterfaceTheme::Blue => "blue",
            InterfaceTheme::Light => "light",
        }
    }

    /// Theme key validation: anything other than `light` or `blue` defaults to `dark`.
    pub fn from_key(key: &str) -> InterfaceTheme {
        match key.trim() {
            "light" => InterfaceTheme::Light,
            "blue" => InterfaceTheme::Blue,
            _ => InterfaceTheme::Dark,
        }
    }
}

/// One theme's custom properties.
struct Palette {
    bg_app: Color,
    bg_panel: Color,
    bg_bar: Color,
    bg_hover: Color,
    accent: Color,
    border: Color,
    text: Color,
    muted: Color,
}

/// Dark theme palette.
const DARK: Palette = Palette {
    bg_app: hex(0x1e1e1e),
    bg_panel: hex(0x252526),
    bg_bar: hex(0x2d2d30),
    bg_hover: hex(0x3e3e42),
    accent: hex(0x007acc),
    border: hex(0x3f3f46),
    text: hex(0xcccccc),
    muted: hex(0x858585),
};

/// Blue theme palette (text colours stay the dark theme's).
const BLUE: Palette = Palette {
    bg_app: hex(0x17243a),
    bg_panel: hex(0x1e3150),
    bg_bar: hex(0x263f66),
    bg_hover: hex(0x34547f),
    accent: hex(0x2b8ad6),
    border: hex(0x4a668c),
    text: hex(0xcccccc),
    muted: hex(0x858585),
};

/// Light theme palette.
const LIGHT: Palette = Palette {
    bg_app: hex(0xe9edf2),
    bg_panel: hex(0xf7f8fa),
    bg_bar: hex(0xdde3ea),
    bg_hover: hex(0xcfd8e3),
    accent: hex(0x1677bd),
    border: hex(0xaab4c0),
    text: hex(0x20252b),
    muted: hex(0x586474),
};

thread_local! {
    /// The live interface theme. Thread-local: iced builds the view on the
    /// thread that runs `update`, and the test harness keeps each test's
    /// theme to its own thread.
    static INTERFACE: Cell<InterfaceTheme> = const { Cell::new(InterfaceTheme::Dark) };
}

pub fn interface_theme() -> InterfaceTheme {
    INTERFACE.with(Cell::get)
}

pub fn set_interface_theme(theme: InterfaceTheme) {
    INTERFACE.with(|c| c.set(theme));
}

fn palette() -> &'static Palette {
    match interface_theme() {
        InterfaceTheme::Dark => &DARK,
        InterfaceTheme::Blue => &BLUE,
        InterfaceTheme::Light => &LIGHT,
    }
}

/// Application background surface color.
pub fn bg_app() -> Color {
    palette().bg_app
}
/// Panel background surface color.
pub fn bg_panel() -> Color {
    palette().bg_panel
}
/// Chrome bar background surface color.
pub fn bg_bar() -> Color {
    palette().bg_bar
}
/// `--bg-hover`
pub fn bg_hover() -> Color {
    palette().bg_hover
}
/// `--bg-active` (the accent)
pub fn accent() -> Color {
    palette().accent
}
/// `--border`
pub fn border() -> Color {
    palette().border
}
/// `--bg-canvas`
pub fn bg_canvas() -> Color {
    match interface_theme() {
        InterfaceTheme::Dark => hex(0x111111),
        InterfaceTheme::Blue => hex(0x0b1728),
        InterfaceTheme::Light => hex(0xffffff),
    }
}
/// `--text-main`
pub fn text_main() -> Color {
    palette().text
}
/// `--text-muted`
pub fn muted() -> Color {
    palette().muted
}
/// The measured native-dialog greys that differ slightly from the palette
/// in the dark theme (`#9A9A9A` muted, `#3C3C3C` border, `#D4D4D4` text);
/// the other themes use the palette's own.
pub fn muted_alt() -> Color {
    match interface_theme() {
        InterfaceTheme::Light => LIGHT.muted,
        _ => hex(0x9a9a9a),
    }
}
pub fn text_alt() -> Color {
    match interface_theme() {
        InterfaceTheme::Light => LIGHT.text,
        _ => hex(0xd4d4d4),
    }
}
/// Icon accents used inline in the original markup.
pub const ICON_BLUE: Color = hex(0x63b3ed);
pub const WHITE: Color = Color::WHITE;

/// `ctx.font = "<italic> <bold> <size>px "<family>""`: a font from an
/// option's family name, weight and style. The family name is interned (a
/// handful of names over a session) because `Family::Name` wants `'static`.
pub fn font_named(family: &str, bold: bool, italic: bool) -> Font {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static NAMES: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
    let name = if family.trim().is_empty() {
        "Arial"
    } else {
        family.trim()
    };
    let interned: &'static str = {
        let mut names = NAMES.get_or_init(Default::default).lock().unwrap();
        match names.get(name) {
            Some(found) => found,
            None => {
                let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
                names.insert(name.to_string(), leaked);
                leaked
            }
        }
    };
    Font {
        family: Family::Name(interned),
        weight: if bold { Weight::Bold } else { Weight::Normal },
        style: if italic { Style::Italic } else { Style::Normal },
        ..Font::DEFAULT
    }
}

/// The browser renders the UI in Segoe UI; form controls (buttons, selects,
/// inputs) fall back to Chrome's control font, Arial.
pub const SEGOE: Font = Font {
    family: Family::Name("Segoe UI"),
    ..Font::DEFAULT
};
pub const SEGOE_SEMIBOLD: Font = Font {
    family: Family::Name("Segoe UI"),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};
pub const ARIAL: Font = Font {
    family: Family::Name("Arial"),
    ..Font::DEFAULT
};
pub const CONSOLAS: Font = Font {
    family: Family::Name("Consolas"),
    ..Font::DEFAULT
};
/// CSS `font-weight: 600` on `monospace` (Consolas has no semibold face).
pub const CONSOLAS_BOLD: Font = Font {
    family: Family::Name("Consolas"),
    weight: Weight::Bold,
    ..Font::DEFAULT
};
pub const SEGOE_BOLD: Font = Font {
    family: Family::Name("Segoe UI"),
    weight: Weight::Bold,
    ..Font::DEFAULT
};

/// Composite `fg` at `alpha` over the opaque `bg` (CSS `opacity`).
pub fn over(fg: Color, alpha: f32, bg: Color) -> Color {
    Color {
        r: fg.r * alpha + bg.r * (1.0 - alpha),
        g: fg.g * alpha + bg.g * (1.0 - alpha),
        b: fg.b * alpha + bg.b * (1.0 - alpha),
        a: 1.0,
    }
}
