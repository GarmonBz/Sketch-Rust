//! The reference image ("photometrics") — `state.bgImage` (state-init
//! lines 14530-14545) and its handlers (sketch-app.js:4763-4943): load,
//! show/hide/remove, Set Scale (two clicks + a known distance), scale by
//! factor, rotate, flips, intensity, Image Edit Mode (drag + wheel),
//! Orient Guide (two clicks + the horizontal/vertical prompt), and the Geo
//! helpers (`parseGpsPair`, ui-manager.js).
//!
//! The image renders into the canvas pass (`drawBgImage`,
//! canvas-engine.js:3107) beneath the drawing; the scale/orient point
//! markers draw on top (`drawScalePoints`, canvas-engine.js:3138).
//!
//! The image is a SESSION-ONLY artifact in the captured original —
//! `buildSaveModel` never carries it (the `saveImageWithSketch` option
//! only widens the exported PNG's bounds) — so the port keeps it in
//! memory too, no save model changes.

use iced::widget::image::Handle;
use iced::{
    alignment,
    widget::{button, column, row, text},
    Background, Element, Length,
};

use crate::Message;
use sketch_core::geometry::Pt;
/// `state.bgImage`.
#[derive(Debug, Clone, Default)]
pub struct BgImage {
    /// The image's encoded bytes (PNG/JPEG — `Handle::from_bytes` decodes
    /// them; the original keeps an HTMLImageElement).
    pub bytes: Option<Handle>,
    /// The natural size, decoded from the file header.
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    /// The image's center, in world feet.
    pub x: f64,
    pub y: f64,
    /// Image pixels per world foot.
    pub scale: f64,
    /// Degrees.
    pub rotation: f64,
    /// 0-1 (the intensity slider's 50 default).
    pub opacity: f64,
    pub flip_h: bool,
    pub flip_v: bool,
    /// Set Scale mode (two clicks + a known distance).
    pub scale_mode: bool,
    pub scale_p1: Option<Pt>,
    pub scale_p2: Option<Pt>,
    /// Image Edit Mode (drag to move, wheel resizes).
    pub edit_mode: bool,
    /// Orient Guide mode (two clicks along a line, then the H/V prompt).
    pub orient_mode: bool,
    pub orient_p1: Option<Pt>,
    pub orient_p2: Option<Pt>,
}

impl BgImage {
    /// `loadBgImage`'s fresh-image state: visible, centered, auto-scaled to
    /// ~60 feet wide, reset flips, the 50 intensity.
    pub fn load(bytes: Handle, width: u32, height: u32) -> Self {
        const PPF: f64 = sketch_core::geometry::PIXELS_PER_FOOT;
        BgImage {
            bytes: Some(bytes),
            width,
            height,
            visible: true,
            x: 0.0,
            y: 0.0,
            scale: (60.0 * PPF) / width.max(1) as f64,
            rotation: 0.0,
            opacity: 0.5,
            flip_h: false,
            flip_v: false,
            ..BgImage::default()
        }
    }

    /// Whether an image is loaded (every handler's `if (!this.bgImage.img)`).
    pub fn loaded(&self) -> bool {
        self.bytes.is_some()
    }

    /// `scaleBgImageBy(factor)` — a no-op without an image.
    pub fn scale_by(&mut self, factor: f64) {
        if self.loaded() {
            self.scale *= factor;
        }
    }

    /// `rotateBgImage(degrees)` — a no-op without an image.
    pub fn rotate(&mut self, degrees: f64) {
        if self.loaded() {
            self.rotation = (self.rotation + degrees) % 360.0;
        }
    }

    /// `handleScaleClick`'s math: the two clicks measured `world_dist` feet
    /// apart on the canvas; the known distance rescales.
    pub fn apply_known_distance(&mut self, p1: Pt, p2: Pt, known_feet: f64) {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let world_dist = (dx * dx + dy * dy).sqrt();
        if world_dist <= 0.0 || known_feet <= 0.0 {
            return;
        }
        self.scale *= known_feet / world_dist;
    }

    /// `handleOrientClick`'s math: the guide's current angle; a vertical
    /// guide rotates the image to `90 - angle`, a horizontal one to
    /// `-angle` (degrees).
    pub fn apply_orient(&mut self, p1: Pt, p2: Pt, vertical: bool) {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let current = dy.atan2(dx) * 180.0 / std::f64::consts::PI;
        if vertical {
            self.rotation += 90.0 - current;
        } else {
            self.rotation -= current;
        }
    }

    /// `drawBgImage`'s placement: the on-screen rect for the image (the
    /// center's screen position + the scaled size), before rotation.
    pub fn screen_rect(&self, pan: (f64, f64), zoom: f64, ppf: f64) -> (f32, f32, f32, f32) {
        let k = ppf * zoom;
        let cx = (pan.0 + self.x * k) as f32;
        let cy = (pan.1 - self.y * k) as f32;
        let w = (self.width as f64 * self.scale * zoom) as f32;
        let h = (self.height as f64 * self.scale * zoom) as f32;
        (cx - w / 2.0, cy - h / 2.0, w, h)
    }
}

/// `showGeoMenu`'s command popup (ui-manager.js:10092): Load Map Image… /
/// Auto Locate by Coordinate… / Auto Locate by Address….
#[derive(Debug, Clone, Default)]
pub struct GeoMenu {
    pub open: bool,
}

impl GeoMenu {
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}

/// The natural size of a PNG or JPEG file's bytes (`new Image()`'s
/// `img.width/height` — enough formats for a reference screenshot).
pub fn image_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() >= 24 && bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        // The IHDR's width/height are big-endian at offsets 16/20.
        let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
        let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
        return Some((w, h));
    }
    if bytes.len() > 4 && bytes.starts_with(&[0xFF, 0xD8]) {
        // Walk the JPEG segments for the first SOF0/SOF1/SOF2.
        let mut i = 2;
        while i + 9 <= bytes.len() {
            if bytes[i] != 0xFF {
                i += 1;
                continue;
            }
            let marker = bytes[i + 1];
            // SOF0/SOF1/SOF2/SOF3/SOF5..SOF7/SOF9..SOF11/SOF13..SOF15 (the
            // DHT's 0xC4 and the 0xDx dummies excluded).
            let sof = matches!(
                marker,
                0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF
            );
            if sof {
                let h = u16::from_be_bytes(bytes[i + 5..i + 7].try_into().ok()?);
                let w = u16::from_be_bytes(bytes[i + 7..i + 9].try_into().ok()?);
                return Some((w as u32, h as u32));
            }
            let len = u16::from_be_bytes(bytes[i + 2..i + 4].try_into().ok()?) as usize;
            i += 2 + len.max(2);
            // The walk's bound: a SOF0 at the buffer's tail still reads
            // (i + 9 <= len).
            if i + 9 > bytes.len() {
                break;
            }
        }
        return None;
    }
    // PNG/JPEG only for now: other formats fall back to the auto-fit's
    // 1x1 assumption via None (documented gap).
    None
}

/// The Geo popup's dropdown layer (the `showCommandPopup` family): anchored
/// under the ribbon's Geo button at (901, 99), or None when closed.
pub fn geo_layer(_open: bool) -> Option<Element<'static, Message>> {
    use crate::theme::{at, bg_panel, border, boxed, hex, muted, text_main, SEGOE};
    let item = |icon_name: &'static str,
                label_s: &'static str,
                msg: Message|
     -> Element<'static, Message> {
        let row = row![
            crate::icons::ph(icon_name, 14.0, muted()),
            text(label_s).font(SEGOE).size(12.0).color(text_main()),
        ]
        .spacing(8)
        .padding(iced::Padding::new(4.0))
        .align_y(alignment::Vertical::Center);
        button(row)
            .width(Length::Fill)
            .on_press(msg)
            .style(move |_, status| button::Style {
                background: match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        Some(Background::Color(hex(0x3e3e42)))
                    }
                    _ => None,
                },
                ..button::Style::default()
            })
            .into()
    };
    let card = column![
        item("image", "Load Map Image…", Message::GeoLoadMap),
        item(
            "crosshair",
            "Auto Locate by Coordinate…",
            Message::GeoCoordinate
        ),
        item("map-pin", "Auto Locate by Address…", Message::GeoAddress),
    ]
    .padding(4)
    .width(230);
    let card = boxed(card, 230, 0.0, Some(bg_panel()), border(), 6.0);
    Some(at(901.0, 99.0, card))
}

/// `parseGpsPair` (ui-manager.js): decimal degrees ("47.7737, -122.6539")
/// or DMS ("47°46'24.92\"N 122°39'14.18\"W" — the mojibake'd degree signs
/// the original's regex strips around, too). Decimal wins when both parts
/// are plain floats; DMS runs combine deg + min/60 + sec/3600 and take the
/// sign from the hemisphere letter (S/W negative).
pub fn parse_gps_pair(text: &str) -> Option<(f64, f64)> {
    // Normalize the degree-sign family (the capture's mojibake bytes) to °.
    let normalized: String = text
        .chars()
        .map(|c| match c {
            '\u{221E}' => '\u{00B0}',
            other => other,
        })
        .collect();
    // Decimal: two plain floats separated by a comma.
    let parts: Vec<&str> = normalized.split(',').map(str::trim).collect();
    if parts.len() == 2 && !parts[0].contains('\u{00B0}') && !parts[1].contains('\u{00B0}') {
        if let (Ok(lat), Ok(lon)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
            if lat.is_finite() && lon.is_finite() && lat.abs() <= 90.0 && lon.abs() <= 180.0 {
                return Some((lat, lon));
            }
        }
        return None;
    }
    // DMS: two runs, each `[deg]°[min]'[sec]"[H]` — the markers decide each
    // number's unit; the hemisphere letter closes the run.
    let chars: Vec<char> = normalized.chars().collect();
    let mut runs: Vec<f64> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let read_number = |i: &mut usize| -> Option<f64> {
            let start = *i;
            while *i < chars.len() && (chars[*i].is_ascii_digit() || chars[*i] == '.') {
                *i += 1;
            }
            chars[start..*i].iter().collect::<String>().parse().ok()
        };
        let skip_markers = |i: &mut usize| {
            while *i < chars.len()
                && matches!(
                    chars[*i],
                    '\u{00B0}' | '\'' | '\u{2019}' | '"' | '\u{201D}' | ' '
                )
            {
                *i += 1;
            }
        };
        let at_letter = |i: &mut usize| -> Option<char> {
            let letter = chars.get(*i).copied()?;
            if letter.is_ascii_alphabetic() {
                *i += 1;
                Some(letter.to_ascii_uppercase())
            } else {
                None
            }
        };
        let deg = read_number(&mut i)?;
        skip_markers(&mut i);
        if let Some(hemisphere) = at_letter(&mut i) {
            // A bare degree run.
            runs.push(match hemisphere {
                'S' | 'W' => -deg,
                _ => deg,
            });
        } else {
            let min = read_number(&mut i)?;
            skip_markers(&mut i);
            if let Some(hemisphere) = at_letter(&mut i) {
                let value = deg + min / 60.0;
                runs.push(match hemisphere {
                    'S' | 'W' => -value,
                    _ => value,
                });
            } else {
                let sec = read_number(&mut i)?;
                skip_markers(&mut i);
                let hemisphere = at_letter(&mut i).unwrap_or('N');
                let value = deg + min / 60.0 + sec / 3600.0;
                runs.push(match hemisphere {
                    'S' | 'W' => -value,
                    _ => value,
                });
            }
        }
        if runs.len() == 2 {
            break;
        }
    }
    if runs.len() == 2 {
        Some((runs[0], runs[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PPF: f64 = sketch_core::geometry::PIXELS_PER_FOOT;

    #[test]
    fn load_fits_sixty_feet_wide_and_resets_the_transform() {
        let handle = iced::widget::image::Handle::from_bytes(vec![0u8; 4]);
        let img = BgImage::load(handle, 1200, 800);
        assert!(img.visible && img.loaded());
        assert_eq!((img.x, img.y), (0.0, 0.0));
        assert!((img.scale - (60.0 * PPF) / 1200.0).abs() < 1e-9);
        assert_eq!(img.rotation, 0.0);
        assert!(!img.flip_h && !img.flip_v);
        assert!((img.opacity - 0.5).abs() < 1e-9);
    }

    #[test]
    fn known_distance_rescales_by_the_ratio() {
        let mut img = BgImage {
            scale: 2.0,
            ..BgImage::default()
        };
        // Ten world feet apart measured; the known distance is 20 feet:
        // the scale doubles.
        img.apply_known_distance(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), 20.0);
        assert!((img.scale - 4.0).abs() < 1e-9);
        // A zero-length guide changes nothing.
        img.apply_known_distance(Pt::new(1.0, 1.0), Pt::new(1.0, 1.0), 20.0);
        assert!((img.scale - 4.0).abs() < 1e-9);
    }

    #[test]
    fn orient_rolls_the_image_square_to_the_guide() {
        let mut img = BgImage::default();
        // The guide points 30° below horizontal: a horizontal guide rotates
        // the image by -30.
        img.apply_orient(
            Pt::new(0.0, 0.0),
            Pt::new(10.0, (-30f64).to_radians().tan() * 10.0),
            false,
        );
        // The image rotates BY the guide's angle (the original's
        // `rotation -= currentAngle`): -(-30) = +30 squares the guide.
        assert!((img.rotation - 30.0).abs() < 1e-6, "{}", img.rotation);
        // A vertical guide lands on 90 - angle instead.
        let mut img = BgImage::default();
        img.apply_orient(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), true);
        assert!((img.rotation - 90.0).abs() < 1e-6);
    }

    #[test]
    fn png_and_jpeg_headers_decode() {
        // A PNG's IHDR: the 8-byte signature, a 4-byte length, "IHDR", then
        // w/h big-endian.
        let mut png = vec![
            0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, b'I', b'H', b'D', b'R',
        ];
        png.extend_from_slice(&1200u32.to_be_bytes());
        png.extend_from_slice(&800u32.to_be_bytes());
        assert_eq!(
            image_size(&png),
            Some((1200, 800)),
            "png: len {}",
            png.len()
        );
        // A JPEG's SOF0: marker scan to the frame header (the APP0's payload
        // is its full 16 bytes: JFIF\0 + version + units + density + thumb).
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0];
        jpeg.extend_from_slice(&16u16.to_be_bytes());
        jpeg.extend_from_slice(&[0x4A, 0x46, 0x49, 0x46, 0x00]); // JFIF\0
        jpeg.extend_from_slice(&[1, 1]); // version
        jpeg.extend_from_slice(&[0]); // units
        jpeg.extend_from_slice(&1u16.to_be_bytes()); // x density
        jpeg.extend_from_slice(&1u16.to_be_bytes()); // y density
        jpeg.extend_from_slice(&[0, 0]); // thumbnail
        jpeg.extend_from_slice(&[0xFF, 0xDB]); // DQT, len 67
        jpeg.extend_from_slice(&67u16.to_be_bytes());
        jpeg.extend_from_slice(&[0u8; 65]);
        jpeg.extend_from_slice(&[0xFF, 0xC0]); // SOF0
        jpeg.extend_from_slice(&17u16.to_be_bytes());
        jpeg.extend_from_slice(&[8]); // precision
        jpeg.extend_from_slice(&640u16.to_be_bytes()); // height first
        jpeg.extend_from_slice(&480u16.to_be_bytes()); // then width
        assert_eq!(image_size(&jpeg), Some((480, 640)));
        assert_eq!(image_size(b"nope"), None);
    }

    #[test]
    fn gps_pairs_parse_decimal_and_dms() {
        let (lat, lon) = parse_gps_pair("47.7737, -122.6539").expect("decimal");
        assert!((lat - 47.7737).abs() < 1e-9 && (lon + 122.6539).abs() < 1e-9);
        // DMS with the capture's mojibake degree sign.
        let (lat, lon) =
            parse_gps_pair("47\u{00B0}46'24.92\"N 122\u{00B0}39'14.18\"W").expect("dms");
        assert!((lat - 47.7736).abs() < 0.001, "{lat}");
        assert!((lon + 122.6539).abs() < 0.001, "{lon}");
        assert!(parse_gps_pair("not coordinates").is_none());
        assert!(parse_gps_pair("95.0, 999.0").is_none(), "the range check");
    }
}
