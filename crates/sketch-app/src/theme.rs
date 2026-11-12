//! Palette, fonts and small layout helpers shared by every chrome region.
//!
//! Colors are the original's dark-theme CSS variables (`refs/theme.css`
//! `:root`) plus the per-icon accent colors measured in
//! `refs/ui-spec/chrome-spec-home.json`. Every region is laid out in CSS
//! pixels measured from the original at 1400x860; `at()` pins an element at
//! its measured offset inside the region that owns it.

use iced::font::{Family, Style, Weight};
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::widget::{container, pin, row, text, Space};
use std::cell::Cell;

use iced::{Background, Border, Color, Element, Font, Length, Padding, Pixels};

use crate::Message;

/// `#rrggbb` → [`Color`], usable in consts.
pub const fn hex(v: u32) -> Color {
    Color {
        r: ((v >> 16) & 0xff) as f32 / 255.0,
        g: ((v >> 8) & 0xff) as f32 / 255.0,
        b: (v & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

/// The interface theme (`body.dark-theme` / `blue-theme` / `light-theme`,
/// `UIManager.applyInterfaceTheme`): which set of `refs/theme.css` custom
/// properties the chrome reads.
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

    /// `applyInterfaceTheme`'s validation: anything but light/blue is dark.
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
    bg_ribbon: Color,
    bg_hover: Color,
    accent: Color,
    border: Color,
    text: Color,
    muted: Color,
}

/// `:root` (the dark theme).
const DARK: Palette = Palette {
    bg_app: hex(0x1e1e1e),
    bg_panel: hex(0x252526),
    bg_ribbon: hex(0x2d2d30),
    bg_hover: hex(0x3e3e42),
    accent: hex(0x007acc),
    border: hex(0x3f3f46),
    text: hex(0xcccccc),
    muted: hex(0x858585),
};

/// `body.blue-theme` (text colours stay the dark theme's).
const BLUE: Palette = Palette {
    bg_app: hex(0x17243a),
    bg_panel: hex(0x1e3150),
    bg_ribbon: hex(0x263f66),
    bg_hover: hex(0x34547f),
    accent: hex(0x2b8ad6),
    border: hex(0x4a668c),
    text: hex(0xcccccc),
    muted: hex(0x858585),
};

/// `body.light-theme`.
const LIGHT: Palette = Palette {
    bg_app: hex(0xe9edf2),
    bg_panel: hex(0xf7f8fa),
    bg_ribbon: hex(0xdde3ea),
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

/// `--bg-app`
pub fn bg_app() -> Color {
    palette().bg_app
}
/// `--bg-panel`
pub fn bg_panel() -> Color {
    palette().bg_panel
}
/// `--bg-ribbon`
pub fn bg_ribbon() -> Color {
    palette().bg_ribbon
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
pub fn border_alt() -> Color {
    match interface_theme() {
        InterfaceTheme::Dark => hex(0x3c3c3c),
        _ => palette().border,
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
pub const ICON_ORANGE: Color = hex(0xf6ad55);
pub const ICON_RED: Color = hex(0xfc8181);
pub const ICON_SLATE: Color = hex(0xa0aec0);
pub const WHITE: Color = Color::WHITE;
/// `.window-controls .close-btn:hover`
pub const CLOSE_HOVER: Color = hex(0xe81123);

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
pub const ARIAL_BOLD: Font = Font {
    family: Family::Name("Arial"),
    weight: Weight::Bold,
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
pub const SYMBOL: Font = Font {
    family: Family::Name("Segoe UI Symbol"),
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

/// How many times glyphs (text and icons) are composited. The original is
/// rendered by Chrome/DirectWrite, whose light-on-dark contrast boost makes
/// glyph edges ~20% heavier than a plain linear rasterizer; compositing the
/// same coverage twice (`a' = 2a - a^2`) reproduces that edge weight.
pub const INK_PASSES: usize = 2;

/// Composite `make()` [`INK_PASSES`] times at the same spot.
pub fn ink<'a, W: Into<Element<'a, Message>>>(make: impl Fn() -> W) -> Element<'a, Message> {
    // Chrome's light-on-dark contrast boost does not apply to dark-on-light.
    let passes = if interface_theme() == InterfaceTheme::Light {
        1
    } else {
        INK_PASSES
    };
    if passes <= 1 {
        return make().into();
    }
    iced::widget::Stack::with_children((0..passes).map(|_| make().into())).into()
}

/// One line of text with an explicit line box height (the measured span
/// height), so the glyph baseline lands where the browser put it.
pub fn label(s: &str, size: f32, font: Font, color: Color, line: f32) -> Element<'static, Message> {
    let s = s.to_string();
    let el = ink(move || text_line(s.clone(), size, font, color, line));
    let dx = TEXT_NUDGE.with(Cell::get);
    if dx == 0.0 {
        el
    } else {
        container(el)
            .padding(Padding {
                left: dx,
                ..Padding::ZERO
            })
            .into()
    }
}

thread_local! {
    static TEXT_NUDGE: Cell<f32> = const { Cell::new(0.0) };
}

/// Build `f` with every [`label`] shifted right by `dx` px. Chrome lays text
/// out at fractional x positions (the spec rounds them); each region's
/// sub-pixel nudge was measured against the gallery with tools/compare.py.
pub fn nudged<T>(dx: f32, f: impl FnOnce() -> T) -> T {
    let prev = TEXT_NUDGE.with(|n| n.replace(dx));
    let out = f();
    TEXT_NUDGE.with(|n| n.set(prev));
    out
}

/// The raw text widget behind [`label`] (single pass).
pub fn text_line<'a>(
    s: impl text::IntoFragment<'a>,
    size: f32,
    font: Font,
    color: Color,
    line: f32,
) -> iced::widget::Text<'a> {
    text(s)
        .size(size)
        .font(font)
        .color(color)
        .line_height(LineHeight::Absolute(Pixels(line)))
        .wrapping(Wrapping::None)
        .shaping(Shaping::Advanced)
}

/// Text with CSS `letter-spacing` (iced has none): one glyph per widget.
pub fn spaced<'a>(
    s: &str,
    size: f32,
    font: Font,
    color: Color,
    line: f32,
    spacing: f32,
) -> Element<'a, Message> {
    row(s
        .chars()
        .map(|c| label(&c.to_string(), size, font, color, line)))
    .spacing(spacing)
    .into()
}

/// Pin `content` at `(x, y)` inside the enclosing stack.
pub fn at<'a>(x: f32, y: f32, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    pin(content).x(x).y(y).into()
}

/// A solid rectangle `w`x`h`.
pub fn rect<'a>(w: f32, h: f32, color: Color) -> Element<'a, Message> {
    container(Space::new().width(w).height(h))
        .style(move |_| container::Style {
            background: Some(Background::Color(color)),
            ..container::Style::default()
        })
        .into()
}

/// A rectangle with background, 1px border and radius.
pub fn boxed<'a>(
    content: impl Into<Element<'a, Message>>,
    w: impl Into<Length>,
    h: impl Into<Length>,
    bg: Option<Color>,
    border: Color,
    radius: f32,
) -> container::Container<'a, Message> {
    container(content)
        .width(w)
        .height(h)
        .style(move |_| container::Style {
            background: bg.map(Background::Color),
            border: Border {
                color: border,
                width: 1.0,
                radius: radius.into(),
            },
            ..container::Style::default()
        })
}

/// A full-size solid background container.
pub fn fill_bg<'a>(
    content: impl Into<Element<'a, Message>>,
    color: Color,
) -> container::Container<'a, Message> {
    container(content).style(move |_| container::Style {
        background: Some(Background::Color(color)),
        ..container::Style::default()
    })
}
