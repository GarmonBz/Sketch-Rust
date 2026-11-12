//! Pure Leica DISTO protocol, settings and distance helpers.
//!
//! Protocol source: the installed leica-disto-ble skill, measured on an
//! E7500i/D510. Distance indications are float32 LE metres. The GUI uses
//! Standard measurements as keyboard-buffer input and Continuous measurements
//! along the previous heading. Native Bluetooth remains hardware-unverified.

use crate::geometry::Pt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// E7500i/D510 GATT identifiers measured by the leica-disto-ble skill.
pub const LEICA_SERVICE: &str = "3ab10100-f831-4395-b29d-570977d5bf94";
pub const LEICA_DISTANCE: &str = "3ab10101-f831-4395-b29d-570977d5bf94";

/// Distance indications are float32 little-endian metres, regardless of
/// the instrument's displayed units. Other characteristics are not distances.
pub fn parse_meters(payload: &[u8]) -> Option<f64> {
    let meters = f32::from_le_bytes(payload.get(..4)?.try_into().ok()?) as f64;
    (meters.is_finite() && meters > 0.0).then_some(meters)
}

/// Numeric keyboard-buffer text, with the skill's four-decimal precision.
pub fn buffer_text(value: f64) -> Option<String> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let text = format!("{value:.4}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    (text != "0").then(|| text.to_owned())
}

/// `distoRounding` — the Options dialog's Rounding select
/// (`refs/ui-manager.js`: none / nearestHalf / nearestHundredth /
/// nearestTenth / nearestWhole; default `'none'`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DistoRounding {
    /// `none` (browser default): keep the measured distance as-is.
    #[default]
    None,
    /// `nearestHalf` — round to the nearest 0.5 ft.
    NearestHalf,
    /// `nearestHundredth` — round to the nearest 0.01 ft.
    NearestHundredth,
    /// `nearestTenth` — round to the nearest 0.1 ft.
    NearestTenth,
    /// `nearestWhole` — round to the nearest 1 ft.
    NearestWhole,
}

impl DistoRounding {
    /// The browser's option string.
    pub fn as_str(self) -> &'static str {
        match self {
            DistoRounding::None => "none",
            DistoRounding::NearestHalf => "nearestHalf",
            DistoRounding::NearestHundredth => "nearestHundredth",
            DistoRounding::NearestTenth => "nearestTenth",
            DistoRounding::NearestWhole => "nearestWhole",
        }
    }

    /// Parse an option string; unknown values fall back to the browser
    /// default (`none`) exactly like the original app's tolerant option
    /// handling.
    pub fn parse(value: &str) -> Self {
        match value {
            "nearestHalf" => DistoRounding::NearestHalf,
            "nearestHundredth" => DistoRounding::NearestHundredth,
            "nearestTenth" => DistoRounding::NearestTenth,
            "nearestWhole" => DistoRounding::NearestWhole,
            _ => DistoRounding::None,
        }
    }
}

// Persistence carries the browser's option strings verbatim, so the values
// round-trip through `.skx` byte-compatibly with the original app.
impl Serialize for DistoRounding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DistoRounding {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(DistoRounding::parse(&String::deserialize(deserializer)?))
    }
}

/// Round a measured distance (feet) per `distoRounding`. Nearest-multiple
/// rounding, half away from zero — the standard interpretation of the
/// browser's option labels. `None` returns the value unchanged.
pub fn round_distance(distance_ft: f64, rounding: DistoRounding) -> f64 {
    let step = match rounding {
        DistoRounding::None => 0.0,
        DistoRounding::NearestHalf => 0.5,
        DistoRounding::NearestHundredth => 0.01,
        DistoRounding::NearestTenth => 0.1,
        DistoRounding::NearestWhole => 1.0,
    };
    if step == 0.0 {
        return distance_ft;
    }
    (distance_ft / step).round() * step
}

/// `distoDrawingMode` — Standard fills the keyboard buffer and waits for a
/// direction key; Continuous commits along the previous drawing heading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DistoDrawingMode {
    /// `standard` (browser default).
    #[default]
    Standard,
    /// `continuous`.
    Continuous,
}

impl DistoDrawingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            DistoDrawingMode::Standard => "standard",
            DistoDrawingMode::Continuous => "continuous",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "continuous" => DistoDrawingMode::Continuous,
            _ => DistoDrawingMode::Standard,
        }
    }
}

impl Serialize for DistoDrawingMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DistoDrawingMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(DistoDrawingMode::parse(&String::deserialize(deserializer)?))
    }
}

/// `distoDiscovery` — Automatic (BLE scan, the port's device picker) or
/// Manual (a COM port string; the browser keeps the field but never reads
/// it — the port stores it verbatim so the option survives round-trips).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DistoDiscovery {
    /// `automatic` (browser default).
    #[default]
    Automatic,
    /// `manual`.
    Manual,
}

impl DistoDiscovery {
    pub fn as_str(self) -> &'static str {
        match self {
            DistoDiscovery::Automatic => "automatic",
            DistoDiscovery::Manual => "manual",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "manual" => DistoDiscovery::Manual,
            _ => DistoDiscovery::Automatic,
        }
    }
}

impl Serialize for DistoDiscovery {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DistoDiscovery {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(DistoDiscovery::parse(&String::deserialize(deserializer)?))
    }
}

/// The Disto settings block — camelCase field names so the block round-trips
/// through `.skx` byte-compatibly with the browser's option keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistoSettings {
    /// `distoDiscovery` — `automatic` | `manual`.
    pub discovery: DistoDiscovery,
    /// `distoComPort` — free text, stored verbatim (never read by the
    /// browser; the manual path is unverified by necessity).
    pub com_port: String,
    /// `distoRounding`.
    pub rounding: DistoRounding,
    /// `distoDrawingMode`.
    pub drawing_mode: DistoDrawingMode,
}

impl Default for DistoSettings {
    fn default() -> Self {
        DistoSettings {
            discovery: DistoDiscovery::Automatic,
            com_port: String::new(),
            rounding: DistoRounding::None,
            drawing_mode: DistoDrawingMode::Standard,
        }
    }
}

/// The bearing (degrees, world space y-down, normalized [0, 360)) from the
/// run anchor toward the cursor — the direction a measured distance is laid
/// out along. With the anchor exactly on the cursor the bearing is 0 (east).
pub fn bearing_from_cursor(anchor: Pt, cursor: Pt) -> f64 {
    let degrees = (cursor.y - anchor.y)
        .atan2(cursor.x - anchor.x)
        .to_degrees();
    let degrees = degrees % 360.0;
    if degrees < 0.0 {
        degrees + 360.0
    } else {
        degrees
    }
}

/// The endpoint `distance_ft` along `bearing_deg` from `anchor` — the exact
/// inverse of [`bearing_from_cursor`] (world units are feet).
pub fn endpoint_from_bearing_distance(anchor: Pt, bearing_deg: f64, distance_ft: f64) -> Pt {
    let radians = bearing_deg.to_radians();
    Pt::new(
        anchor.x + distance_ft * radians.cos(),
        anchor.y + distance_ft * radians.sin(),
    )
}

/// One measured leg: round the raw distance per `distoRounding`, take the
/// bearing from the anchor toward the cursor, and land the endpoint. The
/// caller decides when to apply the resulting endpoint.
pub fn apply_measurement(
    anchor: Pt,
    cursor: Pt,
    raw_distance_ft: f64,
    settings: &DistoSettings,
) -> (f64, f64, Pt) {
    let rounded = round_distance(raw_distance_ft, settings.rounding);
    let bearing = bearing_from_cursor(anchor, cursor);
    let endpoint = endpoint_from_bearing_distance(anchor, bearing, rounded);
    (rounded, bearing, endpoint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_e7500i_indication_decodes_and_rounds_in_feet() {
        let meters = parse_meters(&[0xcf, 0xf7, 0xdb, 0x3f]).unwrap();
        assert!((meters - 1.7185).abs() < 1e-6);
        let feet = meters / 0.3048;
        for (rounding, expected) in [
            (DistoRounding::NearestWhole, "6"),
            (DistoRounding::NearestHalf, "5.5"),
            (DistoRounding::NearestTenth, "5.6"),
            (DistoRounding::NearestHundredth, "5.64"),
        ] {
            assert_eq!(
                buffer_text(round_distance(feet, rounding)).as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn invalid_indications_and_zero_length_buffers_are_rejected() {
        assert_eq!(parse_meters(&[1, 2, 3]), None);
        for invalid in [0.0_f32, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(parse_meters(&invalid.to_le_bytes()), None);
        }
        assert_eq!(buffer_text(0.00001), None);
        assert_eq!(buffer_text(f64::NAN), None);
        assert_eq!(buffer_text(20.0).as_deref(), Some("20"));
        assert_eq!(buffer_text(5.638123).as_deref(), Some("5.6381"));
        assert_eq!(buffer_text(0.0001).as_deref(), Some("0.0001"));
    }

    fn settings(rounding: DistoRounding, mode: DistoDrawingMode) -> DistoSettings {
        DistoSettings {
            rounding,
            drawing_mode: mode,
            ..DistoSettings::default()
        }
    }

    #[test]
    fn rounding_parse_and_str_round_trip() {
        for value in [
            "none",
            "nearestHalf",
            "nearestHundredth",
            "nearestTenth",
            "nearestWhole",
        ] {
            assert_eq!(DistoRounding::parse(value).as_str(), value);
        }
        // Unknown → browser default.
        assert_eq!(DistoRounding::parse(""), DistoRounding::None);
        assert_eq!(DistoRounding::parse("bogus"), DistoRounding::None);
        assert_eq!(
            DistoDrawingMode::parse("continuous"),
            DistoDrawingMode::Continuous
        );
        assert_eq!(
            DistoDrawingMode::parse("anything"),
            DistoDrawingMode::Standard
        );
        assert_eq!(DistoDiscovery::parse("manual"), DistoDiscovery::Manual);
        assert_eq!(
            DistoDiscovery::parse("automatic"),
            DistoDiscovery::Automatic
        );
    }

    #[test]
    fn browser_defaults_match_state_init() {
        let defaults = DistoSettings::default();
        // `distoDiscovery: 'automatic', distoRounding: 'none',
        // distoDrawingMode: 'standard'`, empty COM port.
        assert_eq!(defaults.discovery, DistoDiscovery::Automatic);
        assert_eq!(defaults.com_port, "");
        assert_eq!(defaults.rounding, DistoRounding::None);
        assert_eq!(defaults.drawing_mode, DistoDrawingMode::Standard);
    }

    #[test]
    fn rounding_rules() {
        // none keeps the value.
        assert_eq!(round_distance(12.3456, DistoRounding::None), 12.3456);
        // nearestHalf → 0.5 ft multiples (half away from zero).
        assert!((round_distance(12.3, DistoRounding::NearestHalf) - 12.5).abs() < 1e-9);
        assert!((round_distance(12.2, DistoRounding::NearestHalf) - 12.0).abs() < 1e-9);
        assert!((round_distance(-12.3, DistoRounding::NearestHalf) + 12.5).abs() < 1e-9);
        // nearestHundredth → 0.01 ft.
        assert!((round_distance(12.3456, DistoRounding::NearestHundredth) - 12.35).abs() < 1e-9);
        // nearestTenth → 0.1 ft.
        assert!((round_distance(12.344, DistoRounding::NearestTenth) - 12.3).abs() < 1e-9);
        assert!((round_distance(12.37, DistoRounding::NearestTenth) - 12.4).abs() < 1e-9);
        // nearestWhole → 1 ft.
        assert_eq!(round_distance(12.49, DistoRounding::NearestWhole), 12.0);
        assert_eq!(round_distance(12.5, DistoRounding::NearestWhole), 13.0);
    }

    #[test]
    fn bearing_from_cursor_world_space() {
        // World space is y-down: a cursor straight below the anchor bears 90.
        let a = Pt::new(0.0, 0.0);
        assert!((bearing_from_cursor(a, Pt::new(10.0, 0.0)) - 0.0).abs() < 1e-9);
        assert!((bearing_from_cursor(a, Pt::new(0.0, 10.0)) - 90.0).abs() < 1e-9);
        assert!((bearing_from_cursor(a, Pt::new(-10.0, 0.0)) - 180.0).abs() < 1e-9);
        assert!((bearing_from_cursor(a, Pt::new(0.0, -10.0)) - 270.0).abs() < 1e-9);
        // Degenerate: cursor on the anchor → east.
        assert!((bearing_from_cursor(a, a) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn endpoint_round_trips_bearing_and_distance() {
        let a = Pt::new(3.0, 4.0);
        for bearing in [0.0, 45.0, 90.0, 137.5, 270.0, 359.0] {
            let e = endpoint_from_bearing_distance(a, bearing, 12.0);
            assert!((bearing_from_cursor(a, e) - bearing.rem_euclid(360.0)).abs() < 1e-9);
            assert!((crate::geometry::distance(a, e) - 12.0).abs() < 1e-9);
        }
    }

    #[test]
    fn apply_measurement_rounds_then_lands() {
        // Anchor (0,0), cursor due south (y-down) → bearing 90; raw 20.26 ft
        // with nearestTenth rounds to 20.3 along the cursor bearing.
        let (dist, bearing, endpoint) = apply_measurement(
            Pt::new(0.0, 0.0),
            Pt::new(0.0, 5.0),
            20.26,
            &settings(DistoRounding::NearestTenth, DistoDrawingMode::Standard),
        );
        assert!((dist - 20.3).abs() < 1e-9);
        assert!((bearing - 90.0).abs() < 1e-9);
        assert!((endpoint.x - 0.0).abs() < 1e-9);
        assert!((endpoint.y - 20.3).abs() < 1e-9);

        // Continuous mode does not change the math — the mode only decides
        // commit-vs-stage at the call site.
        let (_, _, endpoint2) = apply_measurement(
            Pt::new(0.0, 0.0),
            Pt::new(0.0, 5.0),
            20.26,
            &settings(DistoRounding::NearestTenth, DistoDrawingMode::Continuous),
        );
        assert_eq!(endpoint, endpoint2);
    }

    #[test]
    fn settings_camel_case_round_trip() {
        let value = DistoSettings {
            com_port: "COM7".to_owned(),
            ..DistoSettings::default()
        };
        let json = serde_json::to_value(&value).unwrap();
        // The browser's exact option keys + string values.
        assert_eq!(json["discovery"], "automatic");
        assert_eq!(json["comPort"], "COM7");
        assert_eq!(json["rounding"], "none");
        assert_eq!(json["drawingMode"], "standard");
        let text = serde_json::to_string(&value).unwrap();
        assert!(text.contains("\"rounding\":\"none\""));
        let back: DistoSettings = serde_json::from_str(&text).unwrap();
        assert_eq!(back, value);
        // Unknown persisted strings fall back to the browser defaults.
        let legacy: DistoSettings = serde_json::from_str(
            "{\"discovery\":\"x\",\"comPort\":\"\",\"rounding\":\"x\",\"drawingMode\":\"x\"}",
        )
        .unwrap();
        assert_eq!(legacy, DistoSettings::default());
    }
}
