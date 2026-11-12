//! `SketchApp.formatLength` / `formatFeetInches` / `formatLengthDual` /
//! `formatArea` (refs/sketch-app.js:541-576) — every length and area label
//! goes through here, driven by the Options (screen 12).
//!
//! Defaults are the original's (`state-init.js:89`): imperial, precision
//! 0.1, trailing zeroes kept, area suffix "sf". The expected strings in the
//! tests were read off the live original (`app.formatLength(..)` etc.), so
//! this file is a transcription of observed behaviour rather than of the
//! formulas alone — note for instance that precision 1 rounds 12.5 to "13"
//! (JS `toFixed`) and that `formatFeetInches(0.04)` is "0'" while 0.05 is
//! `0' 1"`.
//!
//! The HUD pill does NOT use these: `UIManager.updateHUD` takes a raw number
//! string and appends "'" itself (ui-manager.js:2866), which is why the live
//! readout ignores the unit system. That quirk is preserved in
//! `drawing::Hud`.

use crate::drawing::{js_round, js_to_fixed};

/// The formatter's slice of `state.options`.
/// Length/area formatting (metric, precision, stripping, area suffix).
///
/// Serde is manual and FLAT with the original's keys — `unitSystem` is a
/// `"imperial"|"metric"` string, so it reads/writes the save model's
/// `settings.options` verbatim (the `#[serde(flatten)]` on `Options.units`
/// lands these four keys inline).
#[derive(Debug, Clone, PartialEq)]
pub struct Units {
    pub metric: bool,
    pub decimal_precision: f64,
    pub strip_trailing_zeroes: bool,
    pub area_label_suffix: String,
}

fn one_tenth() -> f64 {
    0.1
}

impl Default for Units {
    fn default() -> Self {
        Self {
            metric: false,
            decimal_precision: 0.1,
            strip_trailing_zeroes: false,
            area_label_suffix: "sf".to_string(),
        }
    }
}

impl serde::Serialize for Units {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        // `applyOptions` guards precision by use; a non-finite/zero stored
        // value would break `decimals()`, so it never leaves undamped.
        let mut precision = self.decimal_precision;
        if !precision.is_finite() || precision <= 0.0 {
            precision = one_tenth();
        }
        let mut map = s.serialize_map(Some(4))?;
        map.serialize_entry(
            "unitSystem",
            if self.metric { "metric" } else { "imperial" },
        )?;
        map.serialize_entry("decimalPrecision", &precision)?;
        map.serialize_entry("stripTrailingZeroes", &self.strip_trailing_zeroes)?;
        map.serialize_entry("areaLabelSuffix", &self.area_label_suffix)?;
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for Units {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Raw {
            #[serde(default)]
            unit_system: String,
            #[serde(default = "one_tenth")]
            decimal_precision: f64,
            #[serde(default)]
            strip_trailing_zeroes: bool,
            #[serde(default = "suffix_sf")]
            area_label_suffix: String,
        }
        fn suffix_sf() -> String {
            "sf".to_string()
        }
        let raw = Raw::deserialize(d)?;
        Ok(Units {
            metric: raw.unit_system == "metric",
            decimal_precision: if raw.decimal_precision.is_finite() && raw.decimal_precision > 0.0 {
                raw.decimal_precision
            } else {
                one_tenth()
            },
            strip_trailing_zeroes: raw.strip_trailing_zeroes,
            area_label_suffix: raw.area_label_suffix,
        })
    }
}

impl Units {
    /// `Math.max(0, Math.ceil(-Math.log10(Math.max(0.0001, precision || 0.1))))`.
    pub fn decimals(&self) -> usize {
        let mut p = self.decimal_precision;
        if !p.is_finite() || p == 0.0 {
            p = 0.1;
        }
        let p = p.max(0.0001);
        let d = (-p.log10()).ceil().max(0.0);
        d as usize
    }

    /// `formatLength`: decimal length in the sketch's units.
    pub fn length(&self, value: f64) -> String {
        let converted = if self.metric { value * 0.3048 } else { value };
        let mut text = js_to_fixed(converted, self.decimals());
        if self.strip_trailing_zeroes {
            text = strip_zeroes(&text);
        }
        let suffix = if self.metric { " m" } else { "'" };
        format!("{text}{suffix}")
    }

    /// `formatFeetInches`: feet and whole inches, the way a tape reads.
    pub fn feet_inches(&self, value: f64) -> String {
        let number = if value.is_finite() { value } else { 0.0 };
        let absolute = number.abs();
        let mut feet = (absolute + 1e-9).floor();
        let mut inches = js_round((absolute - feet) * 12.0);
        if inches == 12.0 {
            feet += 1.0;
            inches = 0.0;
        }
        let sign = if number < 0.0 { "-" } else { "" };
        let feet = feet as i64;
        if inches == 0.0 {
            format!("{sign}{feet}'")
        } else {
            format!("{sign}{feet}' {}\"", inches as i64)
        }
    }

    /// `formatLengthDual`: the decimal length plus its feet/inches form.
    pub fn length_dual(&self, value: f64) -> String {
        let text = self.length(value);
        if self.metric {
            return text;
        }
        format!("{text} ({})", self.feet_inches(value))
    }

    /// `formatArea`: grouped, one decimal in feet (two in metres).
    pub fn area(&self, square_feet: f64) -> String {
        let value = if self.metric {
            square_feet * 0.09290304
        } else {
            square_feet
        };
        let value = if value.is_finite() { value } else { 0.0 };
        let text = js_to_fixed(value, if self.metric { 2 } else { 1 });
        let (whole, fraction) = match text.split_once('.') {
            Some((w, f)) => (w.to_string(), f.to_string()),
            None => (text, String::new()),
        };
        let grouped = group_thousands(&whole);
        let suffix = if self.metric {
            "m\u{b2}".to_string()
        } else {
            self.area_label_suffix.clone()
        };
        format!("{grouped}.{fraction} {suffix}")
    }
}

/// `text.replace(/\.0+$|(?<=\.[0-9]*?)0+$/g, '').replace(/\.$/, '')` — drop
/// the zeroes after a decimal point, then a trailing point.
fn strip_zeroes(text: &str) -> String {
    if !text.contains('.') {
        return text.to_string();
    }
    let trimmed = text.trim_end_matches('0');
    trimmed.trim_end_matches('.').to_string()
}

/// `whole.replace(/\B(?=(\d{3})+$)/g, ',')` — thousands separators.
fn group_thousands(whole: &str) -> String {
    let (sign, digits) = match whole.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", whole),
    };
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{sign}{out}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn imperial(precision: f64, strip: bool) -> Units {
        Units {
            metric: false,
            decimal_precision: precision,
            strip_trailing_zeroes: strip,
            area_label_suffix: "sf".into(),
        }
    }

    fn metric(precision: f64, strip: bool) -> Units {
        Units {
            metric: true,
            ..imperial(precision, strip)
        }
    }

    const CASES: [f64; 11] = [
        0.0, 0.04, 0.05, 9.95, 10.25, 12.0, 12.5, 99.96, 1234.56, -12.5, 0.0001,
    ];

    #[test]
    fn length_matches_the_live_original_imperial() {
        // app.formatLength under {unitSystem:'imperial', decimalPrecision:0.1}
        let u = imperial(0.1, false);
        let want = [
            "0.0'", "0.0'", "0.1'", "9.9'", "10.3'", "12.0'", "12.5'", "100.0'", "1234.6'",
            "-12.5'", "0.0'",
        ];
        let got: Vec<String> = CASES.iter().map(|v| u.length(*v)).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn length_matches_the_live_original_stripped() {
        let u = imperial(0.1, true);
        let want = [
            "0'", "0'", "0.1'", "9.9'", "10.3'", "12'", "12.5'", "100'", "1234.6'", "-12.5'", "0'",
        ];
        let got: Vec<String> = CASES.iter().map(|v| u.length(*v)).collect();
        assert_eq!(got, want);

        // precision 0.01 keeps two decimals, then strips.
        let u = imperial(0.01, true);
        let want = [
            "0'", "0.04'", "0.05'", "9.95'", "10.25'", "12'", "12.5'", "99.96'", "1234.56'",
            "-12.5'", "0'",
        ];
        let got: Vec<String> = CASES.iter().map(|v| u.length(*v)).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn length_matches_the_live_original_whole_feet() {
        // precision 1 -> no decimals, JS toFixed rounding (12.5 -> 13).
        let u = imperial(1.0, false);
        let want = [
            "0'", "0'", "0'", "10'", "10'", "12'", "13'", "100'", "1235'", "-13'", "0'",
        ];
        let got: Vec<String> = CASES.iter().map(|v| u.length(*v)).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn length_matches_the_live_original_metric() {
        let u = metric(0.1, false);
        let want = [
            "0.0 m", "0.0 m", "0.0 m", "3.0 m", "3.1 m", "3.7 m", "3.8 m", "30.5 m", "376.3 m",
            "-3.8 m", "0.0 m",
        ];
        let got: Vec<String> = CASES.iter().map(|v| u.length(*v)).collect();
        assert_eq!(got, want);

        let u = metric(0.001, true);
        let want = [
            "0 m",
            "0.012 m",
            "0.015 m",
            "3.033 m",
            "3.124 m",
            "3.658 m",
            "3.81 m",
            "30.468 m",
            "376.294 m",
            "-3.81 m",
            "0 m",
        ];
        let got: Vec<String> = CASES.iter().map(|v| u.length(*v)).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn feet_and_inches_match_the_live_original() {
        let u = Units::default();
        let want = [
            "0'",
            "0'",
            "0' 1\"",
            "9' 11\"",
            "10' 3\"",
            "12'",
            "12' 6\"",
            "100'",
            "1234' 7\"",
            "-12' 6\"",
            "0'",
        ];
        let got: Vec<String> = CASES.iter().map(|v| u.feet_inches(*v)).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn dual_matches_the_live_original() {
        let u = Units::default();
        assert_eq!(u.length_dual(9.95), "9.9' (9' 11\")");
        assert_eq!(u.length_dual(12.0), "12.0' (12')");
        assert_eq!(u.length_dual(-12.5), "-12.5' (-12' 6\")");
        // Metric has no dual form.
        assert_eq!(metric(0.1, false).length_dual(9.95), "3.0 m");
    }

    #[test]
    fn area_matches_the_live_original() {
        let u = Units::default();
        assert_eq!(u.area(0.0), "0.0 sf");
        assert_eq!(u.area(12.34), "12.3 sf");
        assert_eq!(u.area(1234.56), "1,234.6 sf");
        assert_eq!(u.area(1234567.8), "1,234,567.8 sf");
        assert_eq!(u.area(0.04), "0.0 sf");
        assert_eq!(u.area(100.0), "100.0 sf");
        // The suffix is an option (areaLabelSuffix).
        let custom = Units {
            area_label_suffix: "sq ft".into(),
            ..Units::default()
        };
        assert_eq!(custom.area(100.0), "100.0 sq ft");
    }

    #[test]
    fn area_matches_the_live_original_metric() {
        let u = metric(0.1, false);
        assert_eq!(u.area(0.0), "0.00 m\u{b2}");
        assert_eq!(u.area(12.34), "1.15 m\u{b2}");
        assert_eq!(u.area(1234.56), "114.69 m\u{b2}");
        assert_eq!(u.area(1234567.8), "114,695.10 m\u{b2}");
    }

    #[test]
    fn decimals_follow_the_precision() {
        assert_eq!(Units::default().decimals(), 1);
        assert_eq!(imperial(0.01, false).decimals(), 2);
        assert_eq!(imperial(1.0, false).decimals(), 0);
        assert_eq!(imperial(0.001, false).decimals(), 3);
        assert_eq!(imperial(0.0001, false).decimals(), 4);
        // Junk falls back to the 0.1 default, as `|| 0.1` does.
        assert_eq!(imperial(0.0, false).decimals(), 1);
        assert_eq!(imperial(f64::NAN, false).decimals(), 1);
        // Below the floor the floor wins.
        assert_eq!(imperial(1e-9, false).decimals(), 4);
    }

    #[test]
    fn trailing_zeroes_are_stripped_like_the_original_regex() {
        assert_eq!(strip_zeroes("12.30"), "12.3");
        assert_eq!(strip_zeroes("12.00"), "12");
        assert_eq!(strip_zeroes("12.05"), "12.05");
        assert_eq!(strip_zeroes("12"), "12");
        assert_eq!(strip_zeroes("0.0"), "0");
        assert_eq!(strip_zeroes("-3.810"), "-3.81");
    }

    #[test]
    fn thousands_group_from_the_right() {
        assert_eq!(group_thousands("1"), "1");
        assert_eq!(group_thousands("123"), "123");
        assert_eq!(group_thousands("1234"), "1,234");
        assert_eq!(group_thousands("1234567"), "1,234,567");
        assert_eq!(group_thousands("-1234"), "-1,234");
    }
}
