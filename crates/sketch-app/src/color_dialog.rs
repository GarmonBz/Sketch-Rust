//! The Color dialog state and colour-wheel helpers.

use std::sync::LazyLock;

use iced::widget::image;
use iced::Point;

/// The card height on the Standard / Custom tabs (kept for layout sizing).
#[cfg(test)]
pub const H_STANDARD: f32 = 493.0;
#[cfg(test)]
pub const H_CUSTOM: f32 = 441.0;

/// The wheel/SV canvases are 200x200 CSS pixels.
const CANVAS: u32 = 200;
/// The hue strip is 24x200.
const HUE_W: u32 = 24;

/// The 48 presets, in the original's order (6 rows of 8).
pub(crate) const PRESETS: [&str; 48] = [
    "#000000", "#7f0000", "#007f00", "#7f7f00", "#00007f", "#7f007f", "#007f7f", "#c0c0c0",
    "#ff0000", "#00ff00", "#ffff00", "#0000ff", "#ff00ff", "#00ffff", "#ffffff", "#7f7f7f",
    "#800000", "#008000", "#808000", "#000080", "#800080", "#008080", "#666666", "#999999",
    "#ff6a6a", "#6aff6a", "#ffff6a", "#6a6aff", "#ff6aff", "#6affff", "#cccccc", "#333333",
    "#a52a2a", "#d2691e", "#ff8c00", "#ffd700", "#8fbc8f", "#2e8b57", "#00ced1", "#4682b4",
    "#4169e1", "#6a5acd", "#9932cc", "#c71585", "#ff1493", "#ffa07a", "#f5f5dc", "#deb887",
];

/// The state's default `drawColor` (sketch-app.js:21).
pub const DEFAULT_COLOR: &str = "#3182CE";

/// What the dialog's OK applies the colour to: the drawing colour
/// (`toggleDrawColorPalette`) or the selected walls (Edit Tools › Line Color).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorTarget {
    #[default]
    DrawColor,
    SelectedLines,
    /// Edit Tools › Fill Color / Line Color of the selected areas.
    AreaFill,
    AreaLine,
    /// Edit Tools › Text Color of the selected label / area text.
    SelectedText,
    /// Area Type Editor › Line colour / Fill colour of the active type draft.
    TypeEditorLine,
    TypeEditorFill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Standard,
    Custom,
}

impl Tab {
    #[cfg(test)]
    fn height(self) -> f32 {
        match self {
            Tab::Standard => H_STANDARD,
            Tab::Custom => H_CUSTOM,
        }
    }
}

/// The three draggable colour surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    Wheel,
    Sv,
    Hue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgbField {
    R,
    G,
    B,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorMsg {
    /// `toggleDrawColorPalette`: open on `state.drawColor`.
    Open(String),
    Close,
    Tab(Tab),
    /// A press on one of the colour surfaces (the point is the last one the
    /// cursor reported inside it).
    Press(Area),
    /// A cursor move over a surface: applies only while dragging it.
    Hover(Area, Point),
    Release,
    Preset(&'static str),
    Rgb(RgbField, String),
    Ok,
}

/// hsvToRgb (ui-manager.js:1828) — `v=1` for the wheel.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

/// rgbToHsv (ui-manager.js:1842).
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> Hsv {
    let (rn, gn, bn) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let max = rn.max(gn).max(bn);
    let min = rn.min(gn).min(bn);
    let d = max - min;
    let mut h = 0.0;
    let s = if max == 0.0 { 0.0 } else { d / max };
    if d != 0.0 {
        if max == rn {
            h = ((gn - bn) / d + 6.0) % 6.0;
        } else if max == gn {
            h = (bn - rn) / d + 2.0;
        } else {
            h = (rn - gn) / d + 4.0;
        }
        h *= 60.0;
    }
    Hsv { h, s, v: max }
}

/// rgbToHex (ui-manager.js:1856) — lowercase, `padStart(2, '0')`.
pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// hexToRgb (ui-manager.js:1860) — anything unparsable is black.
pub fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    if h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit()) {
        (
            u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&h[4..6], 16).unwrap_or(0),
        )
    } else {
        (0, 0, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl Hsv {
    pub fn hex(self) -> String {
        let (r, g, b) = hsv_to_rgb(self.h, self.s, self.v);
        rgb_to_hex(r, g, b)
    }

    pub fn rgb(self) -> (u8, u8, u8) {
        hsv_to_rgb(self.h, self.s, self.v)
    }
}

/// The wheel's hexagonal radius (`Math.min(cx, cy) - 2`).
fn hex_radius(w: f32, h: f32) -> f32 {
    w.min(h) / 2.0 - 2.0
}

/// `radius * cos(pi/6) / cos(sector - pi/6)` for an angle's sector.
fn sector_radius(radius: f32, angle: f32) -> f32 {
    let sector = angle.abs() % (std::f32::consts::PI / 3.0);
    radius * (std::f32::consts::PI / 6.0).cos() / (sector - std::f32::consts::PI / 6.0).cos()
}

/// The Standard tab's wheel pixels (v = 1, transparent outside the hexagon).
fn wheel_pixels() -> Vec<u8> {
    let w = CANVAS;
    let (cx, cy) = (w as f32 / 2.0, w as f32 / 2.0);
    let radius = hex_radius(w as f32, w as f32);
    let mut buf = vec![0u8; (w * w * 4) as usize];
    for y in 0..w {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let angle = dy.atan2(dx);
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= sector_radius(radius, angle) {
                let hue = (angle * 180.0 / std::f32::consts::PI + 360.0) % 360.0;
                let sat = (dist / sector_radius(radius, angle)).min(1.0);
                let (r, g, b) = hsv_to_rgb(hue, sat, 1.0);
                let i = ((y * w + x) * 4) as usize;
                buf[i] = r;
                buf[i + 1] = g;
                buf[i + 2] = b;
                buf[i + 3] = 255;
            }
        }
    }
    buf
}

/// The hue strip: `h = (1 - y / (h-1)) * 360`.
fn hue_pixels() -> Vec<u8> {
    let (w, h) = (HUE_W, CANVAS);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        let hue = (1.0 - y as f32 / (h - 1) as f32) * 360.0;
        let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            buf[i] = r;
            buf[i + 1] = g;
            buf[i + 2] = b;
            buf[i + 3] = 255;
        }
    }
    buf
}

/// The SV square for a hue: `s = x/(w-1)`, `v = 1 - y/(h-1)`.
fn sv_pixels(hue: f32) -> Vec<u8> {
    let w = CANVAS;
    let mut buf = vec![0u8; (w * w * 4) as usize];
    for y in 0..w {
        for x in 0..w {
            let s = x as f32 / (w - 1) as f32;
            let v = 1.0 - y as f32 / (w - 1) as f32;
            let (r, g, b) = hsv_to_rgb(hue, s, v);
            let i = ((y * w + x) * 4) as usize;
            buf[i] = r;
            buf[i + 1] = g;
            buf[i + 2] = b;
            buf[i + 3] = 255;
        }
    }
    buf
}

pub(crate) static WHEEL: LazyLock<image::Handle> =
    LazyLock::new(|| image::Handle::from_rgba(CANVAS, CANVAS, wheel_pixels()));
pub(crate) static HUE: LazyLock<image::Handle> =
    LazyLock::new(|| image::Handle::from_rgba(HUE_W, CANVAS, hue_pixels()));

pub struct ColorDialog {
    pub open: bool,
    pub tab: Tab,
    pub hsv: Hsv,
    pub rgb: [String; 3],
    drag: Option<Area>,
    hover: Option<(Area, Point)>,
    /// (hue x1000, pixels) — rebuilt whenever the hue moves.
    pub(crate) sv: (u32, image::Handle),
}

impl Default for ColorDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorDialog {
    pub fn new() -> Self {
        let (r, g, b) = hex_to_rgb(DEFAULT_COLOR);
        let hsv = rgb_to_hsv(r, g, b);
        Self {
            open: false,
            tab: Tab::Standard,
            hsv,
            rgb: [r.to_string(), g.to_string(), b.to_string()],
            drag: None,
            hover: None,
            sv: (
                u32::MAX,
                image::Handle::from_rgba(CANVAS, CANVAS, sv_pixels(hsv.h)),
            ),
        }
    }

    /// `toggleDrawColorPalette`: open on the current drawing colour, always
    /// on the Standard tab.
    pub fn open_with(&mut self, hex: &str) {
        let (r, g, b) = hex_to_rgb(hex);
        self.hsv = rgb_to_hsv(r, g, b);
        self.rgb = [r.to_string(), g.to_string(), b.to_string()];
        self.tab = Tab::Standard;
        self.drag = None;
        self.hover = None;
        self.open = true;
        self.sync_sv();
    }

    pub fn close(&mut self) {
        self.open = false;
        self.drag = None;
        self.hover = None;
    }

    /// The card's current height (the two tabs differ).
    #[cfg(test)]
    pub fn height(&self) -> f32 {
        self.tab.height()
    }

    fn sync_sv(&mut self) {
        let key = (self.hsv.h * 1000.0) as u32;
        if self.sv.0 != key {
            self.sv = (
                key,
                image::Handle::from_rgba(CANVAS, CANVAS, sv_pixels(self.hsv.h)),
            );
        }
    }

    /// `updatePreview`: the preview box and the RGB fields follow the hsv.
    fn update_preview(&mut self, edited: Option<RgbField>) {
        let (r, g, b) = self.hsv.rgb();
        for (i, v) in [r, g, b].iter().enumerate() {
            let field = [RgbField::R, RgbField::G, RgbField::B][i];
            if Some(field) != edited {
                self.rgb[i] = v.to_string();
            }
        }
        self.sync_sv();
    }

    /// Applies a position inside a surface, exactly as the original's
    /// `updateFromStandard` / `updateFromSv` / `updateFromHue` do.
    fn apply(&mut self, area: Area, p: Point) {
        match area {
            Area::Wheel => {
                let (cx, cy) = (CANVAS as f32 / 2.0, CANVAS as f32 / 2.0);
                // `event.clientX - canvas.getBoundingClientRect().left`,
                // i.e. measured from the canvas's border box.
                let dx = p.x - cx;
                let dy = p.y - cy;
                let angle = dy.atan2(dx);
                let dist = (dx * dx + dy * dy).sqrt();
                let radius = hex_radius(CANVAS as f32, CANVAS as f32);
                if dist <= sector_radius(radius, angle) {
                    self.hsv.h = (angle * 180.0 / std::f32::consts::PI + 360.0) % 360.0;
                    self.hsv.s = (dist / sector_radius(radius, angle)).clamp(0.0, 1.0);
                    self.hsv.v = 1.0;
                    self.update_preview(None);
                }
            }
            Area::Sv => {
                let x = p.x.clamp(0.0, (CANVAS - 1) as f32);
                let y = p.y.clamp(0.0, (CANVAS - 1) as f32);
                self.hsv.s = x / (CANVAS - 1) as f32;
                self.hsv.v = 1.0 - y / (CANVAS - 1) as f32;
                self.update_preview(None);
            }
            Area::Hue => {
                let y = p.y.clamp(0.0, (CANVAS - 1) as f32);
                self.hsv.h = (1.0 - y / (CANVAS - 1) as f32) * 360.0;
                self.update_preview(None);
            }
        }
    }

    /// Returns the chosen hex when the dialog was confirmed with OK.
    pub fn update(&mut self, msg: ColorMsg) -> Option<String> {
        match msg {
            ColorMsg::Open(hex) => self.open_with(&hex),
            ColorMsg::Close => self.close(),
            ColorMsg::Tab(t) => self.tab = t,
            ColorMsg::Press(area) => {
                self.drag = Some(area);
                if let Some((a, p)) = self.hover {
                    if a == area {
                        self.apply(area, p);
                    }
                }
            }
            ColorMsg::Hover(area, p) => {
                self.hover = Some((area, p));
                if self.drag == Some(area) {
                    self.apply(area, p);
                }
            }
            ColorMsg::Release => self.drag = None,
            ColorMsg::Preset(hex) => {
                let (r, g, b) = hex_to_rgb(hex);
                self.hsv = rgb_to_hsv(r, g, b);
                self.update_preview(None);
            }
            ColorMsg::Rgb(field, raw) => {
                let i = match field {
                    RgbField::R => 0,
                    RgbField::G => 1,
                    RgbField::B => 2,
                };
                self.rgb[i] = raw.clone();
                // `parseInt(value, 10) || 0`, clamped to 0..255.
                let parsed = raw
                    .trim()
                    .parse::<i32>()
                    .ok()
                    .filter(|v| *v != 0)
                    .unwrap_or(0)
                    .clamp(0, 255) as u8;
                let mut rgb = self.hsv.rgb();
                match field {
                    RgbField::R => rgb.0 = parsed,
                    RgbField::G => rgb.1 = parsed,
                    RgbField::B => rgb.2 = parsed,
                }
                self.hsv = rgb_to_hsv(rgb.0, rgb.1, rgb.2);
                self.update_preview(Some(field));
            }
            ColorMsg::Ok => {
                let hex = self.hsv.hex();
                self.close();
                return Some(hex);
            }
        }
        None
    }

    /// The standard wheel's cursor, in the 202 box's coordinates.
    pub(crate) fn wheel_cursor(&self) -> Point {
        let (cx, cy) = (CANVAS as f32 / 2.0, CANVAS as f32 / 2.0);
        let radius = hex_radius(CANVAS as f32, CANVAS as f32);
        let angle = self.hsv.h * std::f32::consts::PI / 180.0;
        let dist = self.hsv.s * sector_radius(radius, angle);
        Point::new(cx + angle.cos() * dist, cy + angle.sin() * dist)
    }

    pub(crate) fn sv_cursor(&self) -> Point {
        Point::new(
            self.hsv.s * (CANVAS - 1) as f32,
            (1.0 - self.hsv.v) * (CANVAS - 1) as f32,
        )
    }

    pub(crate) fn hue_cursor(&self) -> Point {
        Point::new(
            HUE_W as f32 / 2.0,
            (1.0 - self.hsv.h / 360.0) * (CANVAS - 1) as f32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_colour_round_trips_through_hsv() {
        let (r, g, b) = hex_to_rgb(DEFAULT_COLOR);
        assert_eq!((r, g, b), (49, 130, 206));
        let hsv = rgb_to_hsv(r, g, b);
        assert_eq!(hsv.rgb(), (49, 130, 206));
        assert_eq!(hsv.hex(), "#3182ce");
    }

    #[test]
    fn primaries_convert_like_the_original() {
        assert_eq!(rgb_to_hex(255, 0, 0), "#ff0000");
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), (255, 0, 0));
        assert_eq!(hsv_to_rgb(120.0, 1.0, 1.0), (0, 255, 0));
        assert_eq!(hsv_to_rgb(240.0, 1.0, 1.0), (0, 0, 255));
        assert_eq!(rgb_to_hsv(255, 0, 0).h, 0.0);
        assert_eq!(rgb_to_hsv(0, 255, 0).h, 120.0);
        assert_eq!(rgb_to_hsv(0, 0, 255).h, 240.0);
    }

    #[test]
    fn presets_match_the_originals_table() {
        assert_eq!(PRESETS.len(), 48);
        assert_eq!(PRESETS[0], "#000000");
        assert_eq!(PRESETS[8], "#ff0000");
        assert_eq!(PRESETS[47], "#deb887");
    }

    #[test]
    fn opening_starts_on_the_draw_colour() {
        let mut d = ColorDialog::new();
        d.open_with("#ff0000");
        assert!(d.open);
        assert_eq!(d.tab, Tab::Standard);
        assert_eq!(d.rgb, ["255", "0", "0"]);
        assert_eq!(d.height(), H_STANDARD);
    }

    #[test]
    fn ok_returns_the_hex_and_closes_but_cancel_does_not() {
        let mut d = ColorDialog::new();
        d.open_with(DEFAULT_COLOR);
        d.update(ColorMsg::Preset("#00ff00"));
        assert_eq!(d.update(ColorMsg::Close), None);
        assert!(!d.open);
        d.open_with(DEFAULT_COLOR);
        d.update(ColorMsg::Preset("#00ff00"));
        assert_eq!(d.update(ColorMsg::Ok).as_deref(), Some("#00ff00"));
        assert!(!d.open);
    }

    #[test]
    fn the_tabs_change_the_card_height() {
        let mut d = ColorDialog::new();
        d.open_with(DEFAULT_COLOR);
        d.update(ColorMsg::Tab(Tab::Custom));
        assert_eq!(d.height(), H_CUSTOM);
        d.update(ColorMsg::Tab(Tab::Standard));
        assert_eq!(d.height(), H_STANDARD);
    }

    #[test]
    fn pressing_the_hue_strip_sets_the_hue() {
        let mut d = ColorDialog::new();
        d.open_with("#ff0000");
        // The strip is 200 tall: the top is 360, the bottom 0. Positions are
        // measured from the canvas's border box, as the original does.
        d.update(ColorMsg::Hover(Area::Hue, Point::new(13.0, 0.0)));
        d.update(ColorMsg::Press(Area::Hue));
        assert!((d.hsv.h - 360.0).abs() < 0.01, "hue {}", d.hsv.h);
        d.update(ColorMsg::Hover(Area::Hue, Point::new(13.0, 199.0)));
        assert!(d.hsv.h.abs() < 0.01, "hue {}", d.hsv.h);
    }

    #[test]
    fn dragging_updates_only_while_pressed() {
        let mut d = ColorDialog::new();
        d.open_with("#ff0000");
        // A hover without a press must not move the colour.
        d.update(ColorMsg::Hover(Area::Sv, Point::new(100.0, 100.0)));
        assert_eq!(d.hsv.s, 1.0);
        d.update(ColorMsg::Press(Area::Sv));
        assert!(d.hsv.s < 0.6 && d.hsv.s > 0.4, "s {}", d.hsv.s);
        d.update(ColorMsg::Hover(Area::Sv, Point::new(200.0, 1.0)));
        assert!(d.hsv.s > 0.99 && d.hsv.v > 0.99);
        d.update(ColorMsg::Release);
        d.update(ColorMsg::Hover(Area::Sv, Point::new(1.0, 200.0)));
        assert!(d.hsv.s > 0.99, "release stopped the drag");
    }

    #[test]
    fn the_wheel_sets_saturation_and_keeps_value_at_one() {
        let mut d = ColorDialog::new();
        d.open_with("#3182ce");
        // The centre is white (s=0), the hexagon's right edge is full sat.
        d.update(ColorMsg::Hover(Area::Wheel, Point::new(100.0, 100.0)));
        d.update(ColorMsg::Press(Area::Wheel));
        assert!(d.hsv.s < 0.01, "s {}", d.hsv.s);
        assert_eq!(d.hsv.v, 1.0);
        d.update(ColorMsg::Hover(Area::Wheel, Point::new(197.0, 100.0)));
        assert!(d.hsv.s > 0.9, "s {}", d.hsv.s);
    }

    #[test]
    fn rgb_fields_apply_like_parse_int() {
        let mut d = ColorDialog::new();
        d.open_with("#000000");
        d.update(ColorMsg::Rgb(RgbField::R, "255".into()));
        assert_eq!(d.hsv.rgb(), (255, 0, 0));
        // Out of range clamps; junk reads as 0.
        d.update(ColorMsg::Rgb(RgbField::G, "999".into()));
        assert_eq!(d.hsv.rgb(), (255, 255, 0));
        d.update(ColorMsg::Rgb(RgbField::G, "abc".into()));
        assert_eq!(d.hsv.rgb(), (255, 0, 0));
        // The edited field keeps what was typed; the others are rewritten.
        assert_eq!(d.rgb[0], "255");
        assert_eq!(d.rgb[2], "0");
    }

    #[test]
    fn the_wheel_pixels_are_a_hexagon_of_colour() {
        let px = wheel_pixels();
        let at = |x: u32, y: u32| {
            let i = ((y * CANVAS + x) * 4) as usize;
            (px[i], px[i + 1], px[i + 2], px[i + 3])
        };
        // Centre: white (s=0), opaque.
        assert_eq!(at(100, 100), (255, 255, 255, 255));
        // The corners are outside the hexagon: transparent.
        assert_eq!(at(0, 0).3, 0);
        // The right edge is full saturation red-ish at hue 0.
        let (r, _, _, a) = at(196, 100);
        assert_eq!(a, 255);
        assert!(r > 200, "r {r}");
    }

    #[test]
    fn the_hue_strip_runs_from_360_to_0() {
        let px = hue_pixels();
        let row = |y: u32| {
            let i = ((y * HUE_W) * 4) as usize;
            (px[i], px[i + 1], px[i + 2])
        };
        assert_eq!(row(0), (255, 0, 0));
        assert_eq!(row(CANVAS - 1), (255, 0, 0));
        let mid = row(CANVAS / 2);
        assert!(mid.1 > 200 || mid.2 > 200, "mid {mid:?}");
    }
}
