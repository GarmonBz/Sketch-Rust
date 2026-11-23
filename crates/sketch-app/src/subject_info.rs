//! Subject Info (screen 12 - S12-001..011): `state.subjectInfo` and the
//! `editSubjectInfo` dialog (ui-manager.js:1722).
//!
//! The model is the original's object: the 13 parcel fields (`parcelNo`,
//! `fileNo`, `propertyAddress`, `city`, `state`, `zipCode`, `owner`, `county`,
//! `client`, `clientAddress`, `appraiserName`, `inspectionDate`,
//! `gpsCoordinates`) plus `gpsLatitude` / `gpsLongitude`, `generalComments`,
//! `commentTables` (three line lists) and `additionalFields` (`{name, value}`
//! pairs). Keys this port does not know (the suite's merge can add any) are
//! carried through untouched.
//!
//! The dialog edits a draft and commits on OK, like the original's form:
//! three tabs (Parcel Info / General Comments / Comment Tables), the
//! Auto-Fill row (suite-only, so it explains itself in the standalone), the
//! read-only metadata strip, the collapsible additional fields, and the
//! Save Default / Apply Default comments (`sketch.defaultSubjectComments`).
//! Measurements follow theme.css 1213-1248 / 1346-1376; they are transcribed
//! from the CSS, not pixel-compared against a capture.

use iced::widget::text_editor;
use serde::{Deserialize, Deserializer, Serialize};

/// A free-form `{name, value}` pair (`additionalFields`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExtraField {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
}

/// Older saves wrote plain strings here; keep them as a name with no value.
fn lenient_extras<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<ExtraField>, D::Error> {
    let raw = Vec::<serde_json::Value>::deserialize(d)?;
    Ok(raw
        .into_iter()
        .filter_map(|v| match v {
            serde_json::Value::String(name) => Some(ExtraField {
                name,
                value: String::new(),
            }),
            other => serde_json::from_value(other).ok(),
        })
        .collect())
}

/// `commentTables` should be arrays of lines; a table saved as one string is
/// split like the dialog does.
fn lenient_tables<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Vec<String>>, D::Error> {
    let raw = Vec::<serde_json::Value>::deserialize(d)?;
    Ok(raw
        .into_iter()
        .map(|v| match v {
            serde_json::Value::Array(rows) => rows
                .into_iter()
                .map(|r| r.as_str().unwrap_or_default().to_string())
                .collect(),
            serde_json::Value::String(s) => split_lines(&s),
            _ => Vec::new(),
        })
        .collect())
}

// serde's `skip_serializing_if` hands over `&String`.
#[allow(clippy::ptr_arg)]
fn is_empty(s: &String) -> bool {
    s.is_empty()
}

/// `state.subjectInfo`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SubjectInfo {
    #[serde(skip_serializing_if = "is_empty")]
    pub parcel_no: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub file_no: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub property_address: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub city: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub state: String,
    /// `zipCode` (saves from the first port wrote `zip`).
    #[serde(skip_serializing_if = "is_empty", alias = "zip")]
    pub zip_code: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub owner: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub county: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub client: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub client_address: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub appraiser_name: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub inspection_date: String,
    /// The Geo prompts' georeference bag (`gpsCoordinates` / `gpsLatitude` /
    /// `gpsLongitude`, ui-manager.js's showGeoReferenceDialog).
    #[serde(skip_serializing_if = "is_empty")]
    pub gps_coordinates: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub gps_latitude: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub gps_longitude: String,
    pub general_comments: String,
    /// Three comment tables, ten rows each in the report.
    #[serde(deserialize_with = "lenient_tables")]
    pub comment_tables: Vec<Vec<String>>,
    #[serde(deserialize_with = "lenient_extras")]
    pub additional_fields: Vec<ExtraField>,
    /// Keys this port does not model, carried through a save and reopen.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for SubjectInfo {
    fn default() -> Self {
        SubjectInfo {
            parcel_no: String::new(),
            file_no: String::new(),
            property_address: String::new(),
            city: String::new(),
            state: String::new(),
            zip_code: String::new(),
            owner: String::new(),
            county: String::new(),
            client: String::new(),
            client_address: String::new(),
            appraiser_name: String::new(),
            inspection_date: String::new(),
            gps_coordinates: String::new(),
            gps_latitude: String::new(),
            gps_longitude: String::new(),
            general_comments: String::new(),
            // state-init.js: `commentTables: [[], [], []]`.
            comment_tables: vec![Vec::new(), Vec::new(), Vec::new()],
            additional_fields: Vec::new(),
            extra: serde_json::Map::new(),
        }
    }
}

/// `split(/\r?\n/)`.
pub fn split_lines(text: &str) -> Vec<String> {
    text.split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l).to_string())
        .collect()
}

/// `formatGpsPair(lat, lon)`: `"lat, lon"` or empty when either is missing.
pub fn format_gps_pair(lat: &str, lon: &str) -> String {
    let (lat, lon) = (lat.trim(), lon.trim());
    if lat.is_empty() || lon.is_empty() {
        String::new()
    } else {
        format!("{lat}, {lon}")
    }
}

/// The 13 parcel fields in the dialog's order: (label, accessor index).
pub const FIELDS: [&str; 13] = [
    "Parcel No.",
    "File No.",
    "Property Address",
    "City",
    "State",
    "ZipCode",
    "Owner",
    "County",
    "Client",
    "Client Address",
    "Appraiser Name",
    "Inspection Date",
    "GPS Coordinates",
];
pub(crate) const IDX_INSPECTION: usize = 11;
pub(crate) const IDX_GPS: usize = 12;

/// The grid rows (`.subject-field-grid` auto-flow: the 3rd, 9th and 10th
/// labels span both columns, which strands the 8th and 13th on the left).
pub(crate) const GRID_ROWS: [&[usize]; 9] = [
    &[0, 1],
    &[2],
    &[3, 4],
    &[5, 6],
    &[7],
    &[8],
    &[9],
    &[10, 11],
    &[12],
];

pub(crate) const GPS_PLACEHOLDER: &str =
    "47.7737, -122.6539 or 47\u{b0}46'24.92\"N 122\u{b0}39'14.18\"W";
const GPS_ERROR: &str = "GPS Coordinates must be decimal degrees (47.7737, -122.6539) or DMS (47\u{b0}46'24.92\"N 122\u{b0}39'14.18\"W).";
const DATE_ERROR: &str = "Inspection Date must be a date like 2026-10-26.";

impl SubjectInfo {
    fn parcel(&self, i: usize) -> &str {
        match i {
            0 => &self.parcel_no,
            1 => &self.file_no,
            2 => &self.property_address,
            3 => &self.city,
            4 => &self.state,
            5 => &self.zip_code,
            6 => &self.owner,
            7 => &self.county,
            8 => &self.client,
            9 => &self.client_address,
            10 => &self.appraiser_name,
            11 => &self.inspection_date,
            _ => &self.gps_coordinates,
        }
    }

    fn set_parcel(&mut self, i: usize, v: String) {
        match i {
            0 => self.parcel_no = v,
            1 => self.file_no = v,
            2 => self.property_address = v,
            3 => self.city = v,
            4 => self.state = v,
            5 => self.zip_code = v,
            6 => self.owner = v,
            7 => self.county = v,
            8 => self.client = v,
            9 => self.client_address = v,
            10 => self.appraiser_name = v,
            11 => self.inspection_date = v,
            _ => self.gps_coordinates = v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Parcel,
    Comments,
    Tables,
}

/// The dialog's draft (`editSubjectInfo`'s form).
#[derive(Debug, Default)]
pub struct SubjectDialog {
    pub open: bool,
    pub tab: Tab,
    /// The 13 inputs, in `FIELDS` order.
    pub fields: [String; 13],
    pub extras_open: bool,
    pub extra: [(String, String); 3],
    pub comments: text_editor::Content,
    pub tables: [text_editor::Content; 3],
}

#[derive(Debug, Clone)]
pub enum SubjectMsg {
    Open,
    Close,
    Tab(Tab),
    Field(usize, String),
    ExtraName(usize, String),
    ExtraValue(usize, String),
    ToggleExtras,
    Comments(text_editor::Action),
    Table(usize, text_editor::Action),
    AutoFill,
    #[cfg(windows)]
    AutoFillResult(Result<Option<crate::appraise_it::ReportDetails>, String>),
    SaveDefault,
    ApplyDefault,
    Ok,
}

impl SubjectDialog {
    /// Open over the current model (the form is rebuilt each time).
    pub fn open_from(&mut self, info: &SubjectInfo) {
        *self = SubjectDialog::default();
        self.open = true;
        for (i, field) in self.fields.iter_mut().enumerate().take(FIELDS.len()) {
            *field = info.parcel(i).to_string();
        }
        // `gpsValue`: the typed text, else the lat/lon pair.
        let gps = info.gps_coordinates.trim();
        self.fields[IDX_GPS] = if gps.is_empty() {
            format_gps_pair(&info.gps_latitude, &info.gps_longitude)
        } else {
            gps.to_string()
        };
        for (i, f) in info.additional_fields.iter().take(3).enumerate() {
            self.extra[i] = (f.name.clone(), f.value.clone());
        }
        self.comments = text_editor::Content::with_text(&info.general_comments);
        for (i, table) in self.tables.iter_mut().enumerate() {
            let rows = info.comment_tables.get(i).cloned().unwrap_or_default();
            *table = text_editor::Content::with_text(&rows.join("\n"));
        }
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// The comments textarea's value (`Content::text` appends a trailing
    /// newline the textarea does not have).
    fn trimmed(content: &text_editor::Content) -> String {
        let t = content.text();
        t.strip_suffix('\n').unwrap_or(&t).to_string()
    }

    pub fn comments_text(&self) -> String {
        Self::trimmed(&self.comments)
    }

    pub fn set_comments(&mut self, text: &str) {
        self.comments = text_editor::Content::with_text(text);
    }

    /// The form's submit handler: validate the GPS pair, then build the new
    /// model from the draft over `base` (unmodelled keys survive).
    pub fn commit(&self, base: &SubjectInfo) -> Result<SubjectInfo, &'static str> {
        let mut next = base.clone();
        for (i, field) in self.fields.iter().enumerate().take(FIELDS.len()) {
            next.set_parcel(i, field.clone());
        }
        let date = next.inspection_date.trim();
        if !date.is_empty() && !valid_iso_date(date) {
            return Err(DATE_ERROR);
        }
        let gps = next.gps_coordinates.clone();
        let pair = crate::reference_image::parse_gps_pair(&gps);
        if !gps.is_empty() && pair.is_none() {
            return Err(GPS_ERROR);
        }
        (next.gps_latitude, next.gps_longitude) = match pair {
            Some((lat, lon)) => (format!("{lat}"), format!("{lon}")),
            None => (String::new(), String::new()),
        };
        next.general_comments = self.comments_text();
        next.comment_tables = self
            .tables
            .iter()
            .map(|t| split_lines(&Self::trimmed(t)))
            .collect();
        next.additional_fields = self
            .extra
            .iter()
            .map(|(name, value)| ExtraField {
                name: name.clone(),
                value: value.clone(),
            })
            .filter(|f| !f.name.is_empty() || !f.value.is_empty())
            .collect();
        Ok(next)
    }
}

/// An `<input type="date">` value: `YYYY-MM-DD` with a real calendar day.
fn valid_iso_date(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    let (Ok(y), Ok(m), Ok(d)) = (
        parts[0].parse::<u32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_model_matches_state_init() {
        let v = serde_json::to_value(SubjectInfo::default()).unwrap();
        assert_eq!(
            v,
            serde_json::json!({
                "generalComments": "",
                "commentTables": [[], [], []],
                "additionalFields": []
            })
        );
    }

    #[test]
    fn the_original_shape_round_trips_with_unknown_keys() {
        let src = serde_json::json!({
            "parcelNo": "P-1", "zipCode": "98370", "inspectionDate": "2026-10-26",
            "generalComments": "hi",
            "commentTables": [["a", "b"], [], ["c"]],
            "additionalFields": [{"name": "Lot", "value": "7"}],
            "suiteOnly": "keep me"
        });
        let info: SubjectInfo = serde_json::from_value(src.clone()).unwrap();
        assert_eq!(info.parcel_no, "P-1");
        assert_eq!(info.zip_code, "98370");
        assert_eq!(info.additional_fields[0].name, "Lot");
        assert_eq!(serde_json::to_value(&info).unwrap(), src);
    }

    #[test]
    fn first_port_saves_still_load() {
        let src = serde_json::json!({
            "propertyAddress": "123 Main St", "zip": "98370",
            "additionalFields": ["legacy"],
            "commentTables": ["x\ny", ["z"]]
        });
        let info: SubjectInfo = serde_json::from_value(src).unwrap();
        assert_eq!(info.zip_code, "98370");
        assert_eq!(info.additional_fields[0].name, "legacy");
        assert_eq!(info.comment_tables[0], vec!["x", "y"]);
        assert_eq!(info.comment_tables[1], vec!["z"]);
    }

    #[test]
    fn gps_prefill_falls_back_to_the_lat_lon_pair() {
        let info = SubjectInfo {
            gps_latitude: "47.7737".into(),
            gps_longitude: "-122.6539".into(),
            ..SubjectInfo::default()
        };
        let mut d = SubjectDialog::default();
        d.open_from(&info);
        assert_eq!(d.fields[IDX_GPS], "47.7737, -122.6539");
        assert_eq!(format_gps_pair("1", ""), "");
    }

    #[test]
    fn ok_commits_the_draft() {
        let base = SubjectInfo::default();
        let mut d = SubjectDialog::default();
        d.open_from(&base);
        d.fields[0] = "P-9".into();
        d.fields[IDX_GPS] = "47.7737, -122.6539".into();
        d.fields[IDX_INSPECTION] = "2026-10-26".into();
        d.extra[1] = ("Lot".into(), "7".into());
        d.set_comments("first\nsecond");
        d.tables[0] = text_editor::Content::with_text("a\r\nb");
        let next = d.commit(&base).unwrap();
        assert_eq!(next.parcel_no, "P-9");
        assert_eq!(next.gps_latitude, "47.7737");
        assert_eq!(next.gps_longitude, "-122.6539");
        assert_eq!(next.general_comments, "first\nsecond");
        assert_eq!(next.comment_tables[0], vec!["a", "b"]);
        // An empty table splits to one empty line, like `''.split(/\r?\n/)`.
        assert_eq!(next.comment_tables[1], vec![""]);
        assert_eq!(
            next.additional_fields,
            vec![ExtraField {
                name: "Lot".into(),
                value: "7".into()
            }]
        );
    }

    #[test]
    fn bad_gps_or_date_refuses() {
        let base = SubjectInfo::default();
        let mut d = SubjectDialog::default();
        d.open_from(&base);
        d.fields[IDX_GPS] = "nowhere".into();
        assert_eq!(d.commit(&base), Err(GPS_ERROR));
        d.fields[IDX_GPS] = String::new();
        d.fields[IDX_INSPECTION] = "2026-02-30".into();
        assert_eq!(d.commit(&base), Err(DATE_ERROR));
        d.fields[IDX_INSPECTION] = "2028-02-29".into();
        assert!(d.commit(&base).is_ok());
    }
}
