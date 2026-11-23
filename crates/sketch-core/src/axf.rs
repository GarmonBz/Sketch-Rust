//! AXF import — a read-only reader for legacy `.axf` sketch files written
//! by Apex-IV-era (`APX4WIN`) software, mapped into Sketch Rust's own model.
//!
//! ## What an AXF file is
//!
//! The whole file is one *raw deflate* stream (RFC 1951, no zlib/gzip header).
//! Inflated, it is a flat binary object stream that starts with the
//! length-prefixed signature `07 "APX4WIN"`. This reader never decrypts
//! anything: if the stream does not inflate, or does not carry the signature,
//! the file is rejected with a plain error.
//!
//! ## What this reader understands (layout derived from sample files)
//!
//! Little-endian throughout; `f64` are IEEE doubles in drawing units (feet).
//! `str06` is a fixed-width string field: tag `06`, a length byte, then that
//! many bytes (NUL padded, Windows-1252).
//!
//! * **Area table** — one entry per area:
//!   `str06 code, str06 description, 01, f64 factor, f64 area, f64 perimeter,
//!   …, ff ff 1d 02 79 00 01 01 00, str06 name`.
//!   `area`/`perimeter` are the values the writing program calculated; they
//!   include the circular-arc segments, so they double as a check on the
//!   boundary geometry.
//! * **Boundary vertices** — one record per vertex of each area boundary:
//!   `01 30 00, len, linetype name, f64 x, f64 y, f64 0.5, f64 w, f64 w,
//!   f64 bulge, …`. Consecutive vertex records form one closed boundary
//!   (clockwise in the samples). `bulge` is the DXF-style bulge of the edge
//!   *leaving* that vertex (tan of a quarter of the arc's included angle).
//! * **Text** — `kind, str06 text, 00 00 00 00, f64 x, f64 y, Win16 LOGFONT…`.
//!   Kind `01` is a free label (room names), kind `02` a dimension string
//!   (e.g. `11.0'`), which Sketch Rust regenerates from the walls.
//!
//! Block inserts (doors, fixtures), the property/subject header and anything
//! else are not mapped; they are counted so the app can say so.

use crate::geometry::{area_codes, AreaCode, Edge, GeometryEngine, Pt};
use crate::model::{Polygon, SketchLine, WallType};
use serde_json::{json, Value};
use std::io::Read;

/// Largest inflated size accepted (sketches are a few tens of kilobytes).
const MAX_INFLATED: u64 = 64 * 1024 * 1024;
/// Inflated streams start with this signature (length byte + name).
const SIGNATURE: &[u8] = b"\x07APX4WIN";

/// Why an AXF file could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AxfError {
    /// Not a raw deflate stream (or truncated / corrupt).
    NotCompressed,
    /// Inflated fine but is not an `APX4WIN` object stream.
    UnknownSignature,
    /// The stream is valid but is bigger than any sketch should be.
    TooLarge,
    /// A feature this reader cannot map honestly.
    Unsupported(String),
    /// No area boundaries were found.
    NoGeometry,
}

impl std::fmt::Display for AxfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AxfError::NotCompressed => write!(
                f,
                "This AXF file is not in a format Sketch Rust can read \
                 (it is encrypted, damaged or from an unsupported version)."
            ),
            AxfError::UnknownSignature => {
                write!(f, "This file does not look like an Apex sketch (AXF) file.")
            }
            AxfError::TooLarge => write!(f, "This AXF file is too large to open."),
            AxfError::Unsupported(why) => write!(f, "{why}"),
            AxfError::NoGeometry => write!(f, "No sketch geometry was found in this AXF file."),
        }
    }
}

impl std::error::Error for AxfError {}

/// One boundary vertex: its position and the bulge of the edge leaving it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxfVertex {
    pub x: f64,
    pub y: f64,
    pub bulge: f64,
}

/// An entry of the area table.
#[derive(Debug, Clone, PartialEq)]
pub struct AxfAreaEntry {
    pub code: String,
    pub description: String,
    pub name: String,
    pub factor: f64,
    /// As recorded by the writing program (after arcs).
    pub area: f64,
    pub perimeter: f64,
}

/// A closed boundary, tied to its area-table entry when one matches.
#[derive(Debug, Clone, PartialEq)]
pub struct AxfArea {
    pub entry: Option<AxfAreaEntry>,
    pub vertices: Vec<AxfVertex>,
}

impl AxfArea {
    /// Area and perimeter of the boundary, arcs included (always positive).
    pub fn measure(&self) -> (f64, f64) {
        let (points, edges) = self.points_and_edges();
        let engine = GeometryEngine::new();
        (
            engine.calculate_area(&points, &edges),
            engine.calculate_perimeter(&points, &edges),
        )
    }

    /// The boundary as Sketch Rust points plus one edge (bow angle) per
    /// point. Sketch Rust's positive bow bends an edge to the left of travel,
    /// the opposite of a DXF bulge, hence the sign.
    fn points_and_edges(&self) -> (Vec<Pt>, Vec<Edge>) {
        let points = self.vertices.iter().map(|v| Pt::new(v.x, v.y)).collect();
        let edges = self
            .vertices
            .iter()
            .map(|v| Edge {
                line_id: None,
                bow_angle: bulge_to_bow(v.bulge),
            })
            .collect();
        (points, edges)
    }
}

/// A free text label (room names and the like).
#[derive(Debug, Clone, PartialEq)]
pub struct AxfLabel {
    pub text: String,
    pub x: f64,
    pub y: f64,
    /// Degrees counter-clockwise.
    pub rotation: f64,
    pub italic: bool,
    pub underline: bool,
}

/// Everything read from one file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AxfDocument {
    pub areas: Vec<AxfArea>,
    pub labels: Vec<AxfLabel>,
    /// Dimension strings seen (Sketch Rust regenerates dimensions itself).
    pub dimension_texts: usize,
    /// Block inserts seen (doors, fixtures) that were not imported.
    pub blocks_skipped: usize,
    /// Notes the importer wants shown (mismatches between stored and
    /// recomputed areas, boundaries without an area-table entry, …).
    pub warnings: Vec<String>,
}

/// Bulge (`tan(θ/4)`, DXF sense) → Sketch Rust bow angle (degrees).
pub fn bulge_to_bow(bulge: f64) -> f64 {
    if bulge == 0.0 || !bulge.is_finite() {
        return 0.0;
    }
    -(4.0 * bulge.atan()).to_degrees()
}

/// True if `bytes` look like an AXF file (inflates to the `APX4WIN` stream).
pub fn recognizes(bytes: &[u8]) -> bool {
    inflate(bytes).is_ok()
}

fn inflate(bytes: &[u8]) -> Result<Vec<u8>, AxfError> {
    let mut out = Vec::new();
    let mut decoder = flate2::read::DeflateDecoder::new(bytes).take(MAX_INFLATED + 1);
    decoder
        .read_to_end(&mut out)
        .map_err(|_| AxfError::NotCompressed)?;
    if out.len() as u64 > MAX_INFLATED {
        return Err(AxfError::TooLarge);
    }
    if !out.starts_with(SIGNATURE) {
        return Err(if out.is_empty() {
            AxfError::NotCompressed
        } else {
            AxfError::UnknownSignature
        });
    }
    Ok(out)
}

/// Read an `.axf` file's bytes.
pub fn read(bytes: &[u8]) -> Result<AxfDocument, AxfError> {
    let data = inflate(bytes)?;
    decode(&data)
}

/// Decode an already inflated `APX4WIN` stream.
pub fn decode(data: &[u8]) -> Result<AxfDocument, AxfError> {
    let mut doc = AxfDocument::default();
    let entries = scan_area_table(data);
    let runs = scan_vertex_runs(data);
    let (labels, dimension_texts, metric) = scan_texts(data);
    if metric {
        return Err(AxfError::Unsupported(
            "This AXF file uses metric units, which are not supported yet.".into(),
        ));
    }
    doc.labels = labels;
    doc.dimension_texts = dimension_texts;
    doc.blocks_skipped = count_block_inserts(data);

    let closed: Vec<Vec<AxfVertex>> = runs.into_iter().filter(|r| r.len() >= 3).collect();
    if closed.is_empty() {
        return Err(AxfError::NoGeometry);
    }
    if entries.len() != closed.len() {
        doc.warnings.push(format!(
            "The file lists {} areas but {} boundaries; matched in order.",
            entries.len(),
            closed.len()
        ));
    }
    let mut entries = entries.into_iter();
    for vertices in closed {
        let entry = entries.next();
        let area = AxfArea { entry, vertices };
        if let Some(entry) = &area.entry {
            let (calculated, _) = area.measure();
            let tolerance = entry.area.abs().max(1.0) * 0.005;
            if (calculated - entry.area).abs() > tolerance {
                doc.warnings.push(format!(
                    "{}: the file records {:.2} sq ft but its boundary measures {:.2}.",
                    entry.name, entry.area, calculated
                ));
            }
        }
        doc.areas.push(area);
    }
    Ok(doc)
}

// ---------------------------------------------------------------------------
// Scanning
// ---------------------------------------------------------------------------

fn f64_at(data: &[u8], at: usize) -> Option<f64> {
    let bytes: [u8; 8] = data.get(at..at + 8)?.try_into().ok()?;
    Some(f64::from_le_bytes(bytes))
}

fn i16_at(data: &[u8], at: usize) -> Option<i16> {
    let bytes: [u8; 2] = data.get(at..at + 2)?.try_into().ok()?;
    Some(i16::from_le_bytes(bytes))
}

/// Windows-1252 for the bytes that differ from Latin-1.
fn cp1252(byte: u8) -> char {
    const HIGH: [char; 32] = [
        '€', '\u{fffd}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{fffd}', 'Ž',
        '\u{fffd}', '\u{fffd}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ',
        '\u{fffd}', 'ž', 'Ÿ',
    ];
    match byte {
        0x80..=0x9f => HIGH[(byte - 0x80) as usize],
        b => b as char,
    }
}

/// Text up to the first NUL, decoded; `None` if it holds control bytes.
fn decode_text(raw: &[u8]) -> Option<String> {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    let raw = &raw[..end];
    if raw.iter().any(|&b| b < 0x20 || b == 0x7f) {
        return None;
    }
    Some(raw.iter().map(|&b| cp1252(b)).collect())
}

/// A `str06` field at `at`: `(text, end offset)`. Empty strings are valid.
fn str06(data: &[u8], at: usize) -> Option<(String, usize)> {
    if *data.get(at)? != 0x06 {
        return None;
    }
    let len = *data.get(at + 1)? as usize;
    let raw = data.get(at + 2..at + 2 + len)?;
    Some((decode_text(raw)?, at + 2 + len))
}

const NAME_MARKER: &[u8] = &[0xff, 0xff, 0x1d, 0x02, 0x79, 0x00, 0x01, 0x01, 0x00, 0x06];

fn plausible_measure(v: f64, limit: f64) -> bool {
    v.is_finite() && v.abs() < limit
}

fn scan_area_table(data: &[u8]) -> Vec<AxfAreaEntry> {
    let mut entries = Vec::new();
    let mut at = 0;
    while at + 40 < data.len() {
        if let Some((entry, end)) = area_entry_at(data, at) {
            entries.push(entry);
            at = end;
        } else {
            at += 1;
        }
    }
    entries
}

fn area_entry_at(data: &[u8], at: usize) -> Option<(AxfAreaEntry, usize)> {
    let (code, next) = str06(data, at)?;
    if code.is_empty() || code.len() > 24 {
        return None;
    }
    let (description, next) = str06(data, next)?;
    if *data.get(next)? != 0x01 {
        return None;
    }
    let factor = f64_at(data, next + 1)?;
    let area = f64_at(data, next + 9)?;
    let perimeter = f64_at(data, next + 17)?;
    if !plausible_measure(factor, 1000.0)
        || !plausible_measure(area, 1e9)
        || !plausible_measure(perimeter, 1e8)
    {
        return None;
    }
    // The name sits a short, fixed-ish distance after the three numbers.
    let tail = next + 25;
    let window = data.get(tail..(tail + 80).min(data.len()))?;
    let marker = window
        .windows(NAME_MARKER.len())
        .position(|w| w == NAME_MARKER)?;
    let name_at = tail + marker + NAME_MARKER.len() - 1;
    let (name, end) = str06(data, name_at)?;
    Some((
        AxfAreaEntry {
            code,
            description,
            name,
            factor,
            area,
            perimeter,
        },
        end,
    ))
}

/// A boundary-vertex record starting at `at` (the `01 30 00` tag), if it is
/// one: `(vertex, record end)`.
fn vertex_at(data: &[u8], at: usize) -> Option<(AxfVertex, usize)> {
    if data.get(at..at + 3)? != [0x01, 0x30, 0x00] {
        return None;
    }
    let len = *data.get(at + 3)? as usize;
    if !(1..=31).contains(&len) {
        return None;
    }
    let name = data.get(at + 4..at + 4 + len)?;
    if !name
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b' ' | b'-'))
    {
        return None;
    }
    let p = at + 4 + len;
    let (x, y) = (f64_at(data, p)?, f64_at(data, p + 8)?);
    let (side, w1, w2, bulge) = (
        f64_at(data, p + 16)?,
        f64_at(data, p + 24)?,
        f64_at(data, p + 32)?,
        f64_at(data, p + 40)?,
    );
    // Header records of the same shape hold denormal junk where x/y would be.
    let sane = |v: f64| v == 0.0 || (v.abs() > 1e-6 && v.abs() < 1e7);
    if !(sane(x) && sane(y)) || !(0.0..=1.0).contains(&side) {
        return None;
    }
    if !(0.0..100.0).contains(&w1) || !(0.0..100.0).contains(&w2) {
        return None;
    }
    if !bulge.is_finite() || bulge.abs() > 1e3 {
        return None;
    }
    Some((AxfVertex { x, y, bulge }, p + 48))
}

fn scan_vertex_runs(data: &[u8]) -> Vec<Vec<AxfVertex>> {
    let mut runs: Vec<Vec<AxfVertex>> = Vec::new();
    let mut current: Vec<AxfVertex> = Vec::new();
    let mut last_end = 0usize;
    let mut at = 0;
    while at + 4 < data.len() {
        if data[at] == 0x01 && data[at + 1] == 0x30 && data[at + 2] == 0x00 {
            if let Some((vertex, end)) = vertex_at(data, at) {
                // A long gap means something else sits between two runs.
                if !current.is_empty() && at - last_end > 200 {
                    runs.push(std::mem::take(&mut current));
                }
                current.push(vertex);
                last_end = end;
                at = end;
                continue;
            }
            // A record of the same tag that is not a vertex ends the run.
            if !current.is_empty() {
                runs.push(std::mem::take(&mut current));
            }
        }
        at += 1;
    }
    if !current.is_empty() {
        runs.push(current);
    }
    runs
}

/// `(labels, dimension strings seen, metric?)`.
fn scan_texts(data: &[u8]) -> (Vec<AxfLabel>, usize, bool) {
    let mut labels = Vec::new();
    let mut dimensions = 0;
    let mut feet_marks = 0;
    let mut metre_marks = 0;
    let mut at = 1;
    while at + 40 < data.len() {
        let kind = data[at - 1];
        if (kind == 1 || kind == 2) && data[at] == 0x06 {
            if let Some((label, end, text)) = text_at(data, at) {
                if kind == 1 {
                    if !label.text.trim().is_empty() {
                        labels.push(label);
                    }
                } else {
                    dimensions += 1;
                    if text.ends_with('\'') || text.ends_with('"') {
                        feet_marks += 1;
                    } else if text.ends_with('m') {
                        metre_marks += 1;
                    }
                }
                at = end;
                continue;
            }
        }
        at += 1;
    }
    (labels, dimensions, metre_marks > feet_marks)
}

/// A text record whose `str06` starts at `at`: `(label, end, raw text)`.
fn text_at(data: &[u8], at: usize) -> Option<(AxfLabel, usize, String)> {
    let (text, end) = str06(data, at)?;
    if text.is_empty() || data.get(end..end + 4)? != [0, 0, 0, 0] {
        return None;
    }
    let (x, y) = (f64_at(data, end + 4)?, f64_at(data, end + 12)?);
    if !(x.is_finite() && y.is_finite() && x.abs() < 1e7 && y.abs() < 1e7) {
        return None;
    }
    // Win16 LOGFONT: height, width, escapement, orientation, weight (i16),
    // italic, underline, strike-out, charset…, then the face name.
    let font = end + 20;
    let height = i16_at(data, font)?;
    let escapement = i16_at(data, font + 4)?;
    let weight = i16_at(data, font + 8)?;
    let face = data.get(font + 18)?;
    if height == 0 || !(0..=1000).contains(&weight) || !face.is_ascii_alphabetic() {
        return None;
    }
    let label = AxfLabel {
        text: text.clone(),
        x,
        y,
        rotation: f64::from(escapement) / 10.0,
        italic: *data.get(font + 10)? != 0,
        underline: *data.get(font + 11)? != 0,
    };
    Some((label, font + 18, text))
}

/// Block inserts (doors, fixtures): a `01 30` record whose name follows
/// either directly (`ff len name`) or after a line-style name
/// (`00 len style, len name`).
fn count_block_inserts(data: &[u8]) -> usize {
    let name_ok = |at: usize| -> Option<usize> {
        let len = *data.get(at)? as usize;
        let raw = data.get(at + 1..at + 1 + len)?;
        let printable = (3..=60).contains(&len)
            && raw.iter().all(|&b| (0x20..0x7f).contains(&b) || b >= 0xa0)
            && raw.iter().any(|b| b.is_ascii_alphabetic());
        printable.then_some(at + 1 + len)
    };
    let mut count = 0;
    let mut at = 0;
    while at + 6 < data.len() {
        if data[at] == 0x01 && data[at + 1] == 0x30 {
            let end = if data[at + 2] == 0xff {
                name_ok(at + 3)
            } else if data[at + 2] == 0x00 && vertex_at(data, at).is_none() {
                // Line-style name, then the block name.
                let style_len = *data.get(at + 3).unwrap_or(&0) as usize;
                if (1..=31).contains(&style_len) {
                    name_ok(at + 4 + style_len)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(end) = end {
                count += 1;
                at = end;
                continue;
            }
        }
        at += 1;
    }
    count
}

// ---------------------------------------------------------------------------
// Mapping into Sketch Rust's model
// ---------------------------------------------------------------------------

/// Default size (px at zoom 1) of imported labels.
const LABEL_FONT_SIZE: f64 = 14.0;

/// What the app needs after conversion.
#[derive(Debug, Clone, PartialEq)]
pub struct AxfImport {
    /// A save-model JSON value (same shape as a `.skx` model).
    pub model: Value,
    /// Notes to show the user.
    pub notes: Vec<String>,
    /// Areas imported and the sum of their areas, for a one-line summary.
    pub area_count: usize,
}

impl AxfDocument {
    /// Build the save model: one page, a wall per boundary edge, one closed
    /// area per boundary (area code, name and factor from the area table) and
    /// the free labels.
    pub fn to_model(&self, file_name: &str) -> AxfImport {
        let mut codes = area_codes::default_area_codes();
        let mut next_id: i64 = 1;
        let mut lines: Vec<SketchLine> = Vec::new();
        let mut polygons: Vec<Polygon> = Vec::new();
        let mut notes = self.warnings.clone();

        for area in &self.areas {
            let (points, mut edges) = area.points_and_edges();
            let polygon_id = next_id;
            next_id += 1;
            for (i, edge) in edges.iter_mut().enumerate() {
                let line_id = next_id;
                next_id += 1;
                lines.push(SketchLine {
                    id: line_id,
                    start: points[i],
                    end: points[(i + 1) % points.len()],
                    kind: WallType::Exterior,
                    color: None,
                    line_thickness: None,
                    thickness: None,
                    dimension_hidden: false,
                    layer_id: None,
                    bow_angle: edge.bow_angle,
                    arc_height: 0.0,
                    line_style: None,
                    dimension_position: None,
                    dimension_anchor: None,
                    extra: Default::default(),
                });
                edge.line_id = Some(line_id);
            }
            let engine = GeometryEngine::new();
            let gross = engine.calculate_area(&points, &edges);
            let (code, name, factor) = match &area.entry {
                Some(entry) => {
                    let code = entry.code.trim().to_ascii_uppercase();
                    if !codes.iter().any(|c| c.code == code) {
                        codes.push(AreaCode {
                            code: code.clone(),
                            name: entry.name.clone(),
                            description: if entry.description.is_empty() {
                                entry.name.clone()
                            } else {
                                entry.description.clone()
                            },
                            ..AreaCode::default()
                        });
                    }
                    let factor = if entry.factor.is_finite() && entry.factor != 0.0 {
                        entry.factor
                    } else {
                        1.0
                    };
                    (code, entry.name.clone(), factor)
                }
                None => {
                    notes.push("A boundary has no area-table entry; imported as undefined.".into());
                    ("UND".to_string(), "Undefined Area".to_string(), 1.0)
                }
            };
            let mut polygon: Polygon = serde_json::from_value(json!({
                "id": polygon_id,
                "points": points,
                "edges": edges,
                "type": code,
                "name": name,
                "gross": gross,
                "area": gross * factor,
                "factor": factor,
                "subtractMode": "never",
            }))
            .expect("polygon defaults deserialize");
            if factor != 1.0 {
                polygon.factor_override = Some(factor);
            }
            polygons.push(polygon);
        }

        let mut labels = Vec::new();
        for label in &self.labels {
            labels.push(json!({
                "id": next_id,
                "text": label.text,
                "x": label.x,
                "y": label.y,
                "rotation": label.rotation,
                "fontFamily": "Arial",
                "fontSize": LABEL_FONT_SIZE,
                "bold": false,
                "italic": label.italic,
                "underline": label.underline,
                "color": "#000000",
            }));
            next_id += 1;
        }

        if self.blocks_skipped > 0 {
            notes.push(format!(
                "{} door/fixture symbol{} not imported.",
                self.blocks_skipped,
                if self.blocks_skipped == 1 {
                    " was"
                } else {
                    "s were"
                }
            ));
        }

        let model = json!({
            "fileName": file_name,
            "nextId": next_id,
            "areaCodes": codes,
            "layers": [{ "id": "layer-1", "name": "Layer 1", "visible": true, "locked": false,
                         "isGroup": false, "collapsed": false }],
            "activeLayerId": "layer-1",
            "pages": [{
                "name": "Page 1",
                "lines": lines,
                "polygons": polygons,
                "labels": labels,
                "symbols": [],
                "dimensionChains": [],
                "northAngle": 0.0,
            }],
            "subjectInfo": {},
        });
        AxfImport {
            model,
            notes,
            area_count: self.areas.len(),
        }
    }
}

// ---------------------------------------------------------------------------
// Encoding — the same layouts written back out. Clean-room: derived from the
// importer's own documented layout only; no vendor code is used.
// ---------------------------------------------------------------------------

/// Sketch Rust bow angle (degrees) → DXF-style bulge. Inverse of
/// `bulge_to_bow`.
pub fn bow_to_bulge(bow: f64) -> f64 {
    if !bow.is_finite() || bow == 0.0 {
        return 0.0;
    }
    let bulge = (-(bow).to_radians() / 4.0).tan();
    if bulge.is_finite() {
        bulge
    } else {
        0.0
    }
}

/// A `str06` field of exactly `bytes.len()` bytes (NUL-padded to `width`).
fn write_str06_bytes(out: &mut Vec<u8>, bytes: &[u8], width: usize) {
    out.push(0x06);
    out.push(width.min(255) as u8);
    let mut field = bytes.to_vec();
    field.resize(width.min(255), 0);
    out.extend(field);
}

/// Text as Windows-1252 (the reverse of `cp1252`); unknown code points
/// become `?`.
fn encode_cp1252(text: &str) -> Vec<u8> {
    const HIGH: [char; 32] = [
        '€', '\u{fffd}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{fffd}', 'Ž',
        '\u{fffd}', '\u{fffd}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ',
        '\u{fffd}', 'ž', 'Ÿ',
    ];
    text.chars()
        .map(|c| {
            let v = c as u32;
            match v {
                0x00..=0x7f => v as u8,
                0x80..=0x9f => HIGH
                    .iter()
                    .position(|&h| h == c)
                    .map(|i| (0x80 + i) as u8)
                    .unwrap_or(b'?'),
                0xa0..=0xff => v as u8,
                _ => b'?',
            }
        })
        .collect()
}

/// One area-table entry record.
fn write_area_entry(out: &mut Vec<u8>, entry: &AxfAreaEntry) {
    // The reader requires a non-empty code of at most 24 bytes.
    let mut code = encode_cp1252(entry.code.trim());
    if code.is_empty() || code.len() > 24 {
        code = b"UND".to_vec();
    }
    write_str06_bytes(out, &code, code.len());
    let description = encode_cp1252(&entry.description);
    write_str06_bytes(out, &description, description.len());
    out.push(0x01);
    for v in [entry.factor, entry.area, entry.perimeter] {
        out.extend(v.to_le_bytes());
    }
    out.extend([0u8; 15]);
    out.extend(NAME_MARKER[..NAME_MARKER.len() - 1].iter());
    let name = encode_cp1252(entry.name.trim());
    let name = if name.is_empty() {
        b"Area".to_vec()
    } else {
        name
    };
    write_str06_bytes(out, &name, name.len());
    out.extend([0u8; 11]);
}

/// A record with the boundary tag that is NOT a vertex: it ends the current
/// boundary run, so areas stay separate.
fn write_boundary_marker(out: &mut Vec<u8>) {
    out.extend([0x01, 0x30, 0x00, 0x0a]);
    out.extend(b"CONTINUOUS");
    out.extend([0x01, 0x00]);
    out.extend(0.3325f64.to_le_bytes());
    out.extend(0.3325f64.to_le_bytes());
    out.extend([0u8; 40]);
}

/// One boundary-vertex record (the bulge of the edge leaving the vertex).
fn write_vertex(out: &mut Vec<u8>, vertex: &AxfVertex) {
    out.extend([0x01, 0x30, 0x00, 0x0a]);
    out.extend(b"CONTINUOUS");
    for v in [vertex.x, vertex.y, 0.5, 0.3325, 0.3325, vertex.bulge] {
        out.extend(v.to_le_bytes());
    }
    out.extend([0u8; 61]);
}

/// One free-text label record with its Win16 LOGFONT.
fn write_label(out: &mut Vec<u8>, label: &AxfLabel) {
    out.push(0x01);
    let text = encode_cp1252(&label.text);
    write_str06_bytes(out, &text, text.len());
    out.extend([0u8; 4]);
    out.extend(label.x.to_le_bytes());
    out.extend(label.y.to_le_bytes());
    let escapement = (label.rotation.rem_euclid(360.0) * 10.0).round();
    let escapement = escapement.clamp(-32767.0, 32767.0) as i16;
    out.extend((-5i16).to_le_bytes());
    out.extend(0i16.to_le_bytes());
    out.extend(escapement.to_le_bytes());
    out.extend(0i16.to_le_bytes());
    out.extend(400i16.to_le_bytes());
    out.extend([label.italic as u8, label.underline as u8, 0, 1, 0, 0, 0, 0]);
    out.extend(b"Arial\0");
}

/// Encode a document as an inflated `APX4WIN` object stream.
pub fn encode(doc: &AxfDocument) -> Vec<u8> {
    let mut out = SIGNATURE.to_vec();
    out.extend(b"\0\0\0@\x01\0\0\0");
    for _ in 0..6 {
        write_str06_bytes(&mut out, &[], 0);
    }
    for area in &doc.areas {
        if let Some(entry) = &area.entry {
            write_area_entry(&mut out, entry);
        }
    }
    for area in &doc.areas {
        write_boundary_marker(&mut out);
        for vertex in &area.vertices {
            write_vertex(&mut out, vertex);
        }
    }
    for label in &doc.labels {
        write_label(&mut out, label);
    }
    out.extend([0u8; 30]);
    out
}

/// Encode and raw-deflate: the bytes of an `.axf` file.
pub fn write(doc: &AxfDocument) -> Vec<u8> {
    use std::io::Write as _;
    let mut encoder =
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(&encode(doc))
        .expect("writing to a Vec never fails");
    encoder.finish().expect("finishing a Vec never fails")
}

impl AxfDocument {
    /// Rebuild a document from a save model (`pages[0]`), so a saved page can
    /// be exported. See `from_page` for the page-level variant the app uses.
    pub fn from_model(model: &Value) -> Result<AxfDocument, String> {
        let page = model
            .get("pages")
            .and_then(|pages| pages.get(0))
            .ok_or("the model has no pages")?;
        Self::from_page(page)
    }

    /// Rebuild a document from one saved page: every polygon becomes an area
    /// (code, name, factor; area and perimeter re-measured, arcs included)
    /// and every text label round-trips with position, rotation, italic and
    /// underline.
    pub fn from_page(page: &Value) -> Result<AxfDocument, String> {
        let polygons: Vec<Polygon> =
            serde_json::from_value(page.get("polygons").cloned().unwrap_or(Value::Null))
                .map_err(|e| format!("the page's areas did not parse: {e}"))?;
        if polygons.is_empty() {
            return Err("The page has no closed areas to export.".into());
        }
        let mut areas = Vec::new();
        for poly in &polygons {
            let mut vertices = Vec::with_capacity(poly.points.len());
            for (point, edge) in poly
                .points
                .iter()
                .zip(poly.edges.iter().chain(std::iter::repeat(&DEFAULT_EDGE)))
            {
                vertices.push(AxfVertex {
                    x: point.x,
                    y: point.y,
                    bulge: bow_to_bulge(edge.bow_angle),
                });
            }
            let mut area = AxfArea {
                entry: None,
                vertices,
            };
            let (measured, perimeter) = area.measure();
            let code = poly.kind.trim().to_ascii_uppercase();
            let code = if code.is_empty() {
                "UND".to_string()
            } else {
                code
            };
            let name = poly.name.trim().to_string();
            let name = if name.is_empty() {
                "Undefined Area".to_string()
            } else {
                name
            };
            area.entry = Some(AxfAreaEntry {
                code,
                description: name.clone(),
                name,
                factor: if poly.factor.is_finite() && poly.factor > 0.0 {
                    poly.factor
                } else {
                    1.0
                },
                area: measured,
                perimeter,
            });
            areas.push(area);
        }
        #[derive(serde::Deserialize)]
        struct SavedLabel {
            text: String,
            x: f64,
            y: f64,
            #[serde(default)]
            rotation: f64,
            #[serde(default)]
            italic: bool,
            #[serde(default)]
            underline: bool,
        }
        let labels: Vec<SavedLabel> = serde_json::from_value(
            page.get("labels")
                .cloned()
                .unwrap_or_else(|| Value::Array(vec![])),
        )
        .unwrap_or_default();
        let labels = labels
            .into_iter()
            .filter(|l| !l.text.trim().is_empty())
            .map(|l| AxfLabel {
                text: l.text,
                x: l.x,
                y: l.y,
                rotation: l.rotation,
                italic: l.italic,
                underline: l.underline,
            })
            .collect();
        Ok(AxfDocument {
            areas,
            labels,
            dimension_texts: 0,
            blocks_skipped: 0,
            warnings: vec![],
        })
    }
}

/// The edge a point without one would carry (straight).
const DEFAULT_EDGE: Edge = Edge {
    line_id: None,
    bow_angle: 0.0,
};

// ---------------------------------------------------------------------------
// Tests — a synthetic stream in the same layout (no private data).
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use flate2::{write::DeflateEncoder, Compression};
    use std::io::Write;

    fn str06(out: &mut Vec<u8>, text: &str, width: usize) {
        out.push(0x06);
        out.push(width as u8);
        let mut field = text.as_bytes().to_vec();
        field.resize(width, 0);
        out.extend(field);
    }

    fn area_entry(out: &mut Vec<u8>, code: &str, name: &str, area: f64, perimeter: f64) {
        out.extend([0x01, 0x00]);
        str06(out, code, code.len());
        str06(out, code, code.len());
        out.push(0x01);
        for v in [1.0, area, perimeter] {
            out.extend(v.to_le_bytes());
        }
        out.extend([0u8; 15]);
        out.extend(NAME_MARKER[..NAME_MARKER.len() - 1].iter());
        str06(out, name, name.len());
        out.extend([0u8; 11]);
    }

    fn vertex(out: &mut Vec<u8>, x: f64, y: f64, bulge: f64) {
        out.extend([0x01, 0x30, 0x00, 0x0a]);
        out.extend(b"CONTINUOUS");
        for v in [x, y, 0.5, 0.3325, 0.3325, bulge] {
            out.extend(v.to_le_bytes());
        }
        out.extend([0u8; 61]);
    }

    fn header_record(out: &mut Vec<u8>) {
        out.extend([0x01, 0x30, 0x00, 0x0a]);
        out.extend(b"CONTINUOUS");
        out.extend([0x01, 0x00]);
        out.extend(0.3325f64.to_le_bytes());
        out.extend(0.3325f64.to_le_bytes());
        out.extend([0u8; 40]);
    }

    fn text(out: &mut Vec<u8>, kind: u8, text: &str, x: f64, y: f64, escapement: i16) {
        out.push(kind);
        str06(out, text, text.len() + 2);
        out.extend([0u8; 4]);
        out.extend(x.to_le_bytes());
        out.extend(y.to_le_bytes());
        out.extend((-5i16).to_le_bytes());
        out.extend(0i16.to_le_bytes());
        out.extend(escapement.to_le_bytes());
        out.extend(0i16.to_le_bytes());
        out.extend(400i16.to_le_bytes());
        out.extend([0, 0, 0, 1, 0, 0, 0, 0]);
        out.extend(b"Arial\0New Roman\0");
    }

    /// Two areas: a 20 x 10 rectangle (clockwise) whose top edge bows out by a
    /// 90-degree arc, and a plain 10 x 10 square; two labels and a dimension.
    pub(crate) fn synthetic_stream() -> Vec<u8> {
        let mut out = SIGNATURE.to_vec();
        out.extend(b"\0\0\0@\x01\0\0\0");
        for _ in 0..6 {
            str06(&mut out, "", 0);
        }
        // Rectangle (0,0) → (0,10) → (20,10) → (20,0): clockwise; the edge
        // leaving (0,10) bows left of travel (outward) with bulge -tan(22.5°).
        let bulge = -(22.5f64).to_radians().tan();
        let chord_20 = 20.0;
        let theta = 90f64.to_radians();
        let radius = chord_20 / (2.0 * (theta / 2.0).sin());
        let segment = radius * radius / 2.0 * (theta - theta.sin());
        let area_one = 200.0 + segment;
        let perimeter_one = 10.0 + radius * theta + 10.0 + 20.0;
        area_entry(&mut out, "GLA1", "First Floor", area_one, perimeter_one);
        area_entry(&mut out, "ZZZ", "Shed", 100.0, 40.0);
        text(&mut out, 2, "20.0'", 10.0, 12.0, 0);
        text(&mut out, 2, "10.0'", -1.0, 5.0, 900);
        text(&mut out, 1, "Living\u{0}", 10.0, 5.0, 0);
        let mut half = b"\xbd Bath".to_vec();
        half.push(0);
        // Raw bytes, not UTF-8: build the record by hand.
        let mut record = vec![1u8, 0x06, (half.len() + 1) as u8];
        record.extend(&half);
        record.push(0);
        record.extend([0u8; 4]);
        record.extend(4.0f64.to_le_bytes());
        record.extend(2.0f64.to_le_bytes());
        record.extend((-5i16).to_le_bytes());
        record.extend(0i16.to_le_bytes());
        record.extend(450i16.to_le_bytes());
        record.extend(0i16.to_le_bytes());
        record.extend(400i16.to_le_bytes());
        record.extend([0, 0, 0, 1, 0, 0, 0, 0]);
        record.extend(b"Arial\0");
        out.extend(record);
        out.extend([0u8; 12]);

        header_record(&mut out);
        vertex(&mut out, 0.0, 0.0, 0.0);
        vertex(&mut out, 0.0, 10.0, bulge);
        vertex(&mut out, 20.0, 10.0, 0.0);
        vertex(&mut out, 20.0, 0.0, 0.0);
        header_record(&mut out);
        vertex(&mut out, 30.0, 0.0, 0.0);
        vertex(&mut out, 30.0, 10.0, 0.0);
        vertex(&mut out, 40.0, 10.0, 0.0);
        vertex(&mut out, 40.0, 0.0, 0.0);
        out.extend([0u8; 30]);
        // A block insert record (door).
        out.extend([0x01, 0x30, 0xff, 0x0c]);
        out.extend(b"DOOR- Left 90");
        out.extend([0u8; 40]);
        out
    }

    pub(crate) fn synthetic_axf() -> Vec<u8> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&synthetic_stream()).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn decodes_areas_labels_and_counts_the_rest() {
        let doc = read(&synthetic_axf()).expect("reads");
        assert_eq!(doc.areas.len(), 2);
        assert_eq!(doc.areas[0].vertices.len(), 4);
        let entry = doc.areas[0].entry.as_ref().expect("matched entry");
        assert_eq!(
            (entry.code.as_str(), entry.name.as_str()),
            ("GLA1", "First Floor")
        );
        assert_eq!(doc.areas[1].entry.as_ref().unwrap().name, "Shed");
        assert_eq!(doc.dimension_texts, 2);
        assert_eq!(doc.blocks_skipped, 1);
        assert_eq!(doc.labels.len(), 2);
        assert_eq!(doc.labels[0].text, "Living");
        assert_eq!(doc.labels[1].text, "½ Bath");
        assert_eq!(
            (doc.labels[1].x, doc.labels[1].y, doc.labels[1].rotation),
            (4.0, 2.0, 45.0)
        );
        assert!(doc.warnings.is_empty(), "{:?}", doc.warnings);
    }

    #[test]
    fn recorded_area_matches_the_arc_aware_boundary() {
        let doc = read(&synthetic_axf()).unwrap();
        let (area, perimeter) = doc.areas[0].measure();
        let entry = doc.areas[0].entry.as_ref().unwrap();
        assert!((area - entry.area).abs() < 1e-6, "{area} vs {}", entry.area);
        assert!((perimeter - entry.perimeter).abs() < 1e-6);
        // Plain square.
        assert!((doc.areas[1].measure().0 - 100.0).abs() < 1e-9);
    }

    #[test]
    fn model_has_walls_polygons_codes_and_labels() {
        let doc = read(&synthetic_axf()).unwrap();
        let import = doc.to_model("Example");
        assert_eq!(import.area_count, 2);
        let model = &import.model;
        assert_eq!(model["fileName"], "Example");
        let page = &model["pages"][0];
        assert_eq!(page["lines"].as_array().unwrap().len(), 8);
        let polygons = page["polygons"].as_array().unwrap();
        assert_eq!(polygons[0]["type"], "GLA1");
        assert_eq!(polygons[0]["name"], "First Floor");
        assert_eq!(polygons[1]["type"], "ZZZ");
        assert!(
            (polygons[0]["area"].as_f64().unwrap() - doc.areas[0].entry.as_ref().unwrap().area)
                .abs()
                < 1e-6
        );
        // The bowed wall carries a 90-degree bow, outward (area grows).
        let bowed: Vec<_> = page["lines"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|l| l.get("bowAngle").is_some())
            .collect();
        assert_eq!(bowed.len(), 1);
        assert!((bowed[0]["bowAngle"].as_f64().unwrap() - 90.0).abs() < 1e-9);
        assert_eq!(page["labels"].as_array().unwrap().len(), 2);
        // The unknown code is added to the table, existing ones are not.
        let codes = model["areaCodes"].as_array().unwrap();
        assert_eq!(codes.iter().filter(|c| c["code"] == "ZZZ").count(), 1);
        assert_eq!(codes.iter().filter(|c| c["code"] == "GLA1").count(), 1);
        assert!(import.notes.iter().any(|n| n.contains("door/fixture")));
    }

    #[test]
    fn rejects_files_that_are_not_axf() {
        assert_eq!(
            read(b"{\"format\":\"sketchy\"}").unwrap_err(),
            AxfError::NotCompressed
        );
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"just some other deflated data").unwrap();
        let other = encoder.finish().unwrap();
        assert_eq!(read(&other).unwrap_err(), AxfError::UnknownSignature);
        assert!(!recognizes(&[]));
        // A valid signature with no boundaries.
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(SIGNATURE).unwrap();
        encoder.write_all(&[0u8; 64]).unwrap();
        assert_eq!(
            read(&encoder.finish().unwrap()).unwrap_err(),
            AxfError::NoGeometry
        );
    }

    #[test]
    fn metric_files_are_refused_rather_than_misscaled() {
        let mut stream = synthetic_stream();
        let mut extra = Vec::new();
        for i in 0..6 {
            text(&mut extra, 2, "3.5m", f64::from(i), 1.0, 0);
        }
        stream.extend(extra);
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&stream).unwrap();
        let err = read(&encoder.finish().unwrap()).unwrap_err();
        assert!(matches!(err, AxfError::Unsupported(_)), "{err:?}");
    }

    #[test]
    fn bulge_signs_follow_the_bow_convention() {
        assert_eq!(bulge_to_bow(0.0), 0.0);
        // A DXF bulge of -tan(22.5°) bows 90° to the left of travel.
        assert!((bulge_to_bow(-(22.5f64).to_radians().tan()) - 90.0).abs() < 1e-9);
        assert!((bulge_to_bow((22.5f64).to_radians().tan()) + 90.0).abs() < 1e-9);
    }

    /// Optional check against a real file: set `SKETCH_AXF_SAMPLE` to its
    /// path. Skipped when unset, so no private data lives in the repository.
    #[test]
    fn real_file_matches_its_own_recorded_areas() {
        let Some(path) = std::env::var_os("SKETCH_AXF_SAMPLE") else {
            return;
        };
        let doc = read(&std::fs::read(path).unwrap()).expect("reads");
        eprintln!(
            "areas {} labels {} dims {} blocks {} warnings {:?}",
            doc.areas.len(),
            doc.labels.len(),
            doc.dimension_texts,
            doc.blocks_skipped,
            doc.warnings
        );
        for area in &doc.areas {
            let (a, p) = area.measure();
            let e = area.entry.as_ref().unwrap();
            eprintln!(
                "{} [{}] n={} calc {:.4}/{:.4} stored {:.4}/{:.4}",
                e.name,
                e.code,
                area.vertices.len(),
                a,
                p,
                e.area,
                e.perimeter
            );
        }
        for l in &doc.labels {
            eprintln!(
                "label {:?} {:.1},{:.1} rot {}",
                l.text, l.x, l.y, l.rotation
            );
        }
        assert!(doc.warnings.is_empty(), "{:?}", doc.warnings);
    }

    // -----------------------------------------------------------------
    // Export round trips (clean-room encoder against the reader above).
    // -----------------------------------------------------------------

    fn export_document() -> AxfDocument {
        let bulge = -(22.5f64).to_radians().tan();
        let mut areas = vec![
            AxfArea {
                entry: None,
                vertices: vec![
                    AxfVertex {
                        x: 0.0,
                        y: 0.0,
                        bulge: 0.0,
                    },
                    AxfVertex {
                        x: 0.0,
                        y: 10.0,
                        bulge,
                    },
                    AxfVertex {
                        x: 20.0,
                        y: 10.0,
                        bulge: 0.0,
                    },
                    AxfVertex {
                        x: 20.0,
                        y: 0.0,
                        bulge: 0.0,
                    },
                ],
            },
            AxfArea {
                entry: None,
                vertices: vec![
                    AxfVertex {
                        x: 30.0,
                        y: 0.0,
                        bulge: 0.0,
                    },
                    AxfVertex {
                        x: 30.0,
                        y: 10.0,
                        bulge: 0.0,
                    },
                    AxfVertex {
                        x: 40.0,
                        y: 10.0,
                        bulge: 0.0,
                    },
                    AxfVertex {
                        x: 40.0,
                        y: 0.0,
                        bulge: 0.0,
                    },
                ],
            },
        ];
        for (area, (code, description, name)) in areas
            .iter_mut()
            .zip([("GLA1", "Main", "First Floor"), ("UND", "Main", "Shed")])
        {
            let (measured, perimeter) = area.measure();
            area.entry = Some(AxfAreaEntry {
                code: code.to_string(),
                description: description.to_string(),
                name: name.to_string(),
                factor: 1.0,
                area: measured,
                perimeter,
            });
        }
        AxfDocument {
            areas,
            labels: vec![
                AxfLabel {
                    text: "Living".into(),
                    x: 10.0,
                    y: 5.0,
                    rotation: 0.0,
                    italic: true,
                    underline: false,
                },
                AxfLabel {
                    text: "½ Bath".into(),
                    x: 4.0,
                    y: 2.0,
                    rotation: 45.0,
                    italic: false,
                    underline: true,
                },
            ],
            dimension_texts: 0,
            blocks_skipped: 0,
            warnings: vec![],
        }
    }

    #[test]
    fn export_round_trips_through_the_importer() {
        let doc = export_document();
        let bytes = write(&doc);
        assert!(recognizes(&bytes), "the export is a valid AXF stream");
        let back = read(&bytes).expect("the importer accepts its own export");
        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
        assert_eq!(back.areas.len(), 2, "both areas");
        for (got, want) in back.areas.iter().zip(doc.areas.iter()) {
            assert_eq!(got.vertices.len(), want.vertices.len(), "vertex count");
            for (v, w) in got.vertices.iter().zip(want.vertices.iter()) {
                assert!((v.x - w.x).abs() < 1e-9, "x {} vs {}", v.x, w.x);
                assert!((v.y - w.y).abs() < 1e-9, "y {} vs {}", v.y, w.y);
                assert!(
                    (v.bulge - w.bulge).abs() < 1e-9,
                    "bulge {} vs {}",
                    v.bulge,
                    w.bulge
                );
            }
            let (g, w) = (got.entry.as_ref(), want.entry.as_ref());
            assert_eq!(g.is_some(), w.is_some(), "entry presence");
            if let (Some(g), Some(w)) = (g, w) {
                assert_eq!(g.code, w.code);
                assert_eq!(g.description, w.description);
                assert_eq!(g.name, w.name);
                assert!((g.factor - w.factor).abs() < 1e-9);
                assert!(
                    (g.area - w.area).abs() < 1e-6,
                    "area {} vs {}",
                    g.area,
                    w.area
                );
                assert!(
                    (g.perimeter - w.perimeter).abs() < 1e-6,
                    "perimeter {} vs {}",
                    g.perimeter,
                    w.perimeter
                );
            }
        }
        assert_eq!(back.labels.len(), 2, "labels");
        for (got, want) in back.labels.iter().zip(doc.labels.iter()) {
            assert_eq!(got.text, want.text);
            assert!((got.x - want.x).abs() < 1e-9);
            assert!((got.y - want.y).abs() < 1e-9);
            assert!((got.rotation - want.rotation).abs() < 1e-6);
            assert_eq!(got.italic, want.italic, "italic");
            assert_eq!(got.underline, want.underline, "underline");
        }
    }

    #[test]
    fn model_to_axf_to_model_preserves_the_page() {
        let doc = read(&synthetic_axf()).unwrap();
        let import = doc.to_model("t");
        let back = AxfDocument::from_model(&import.model).expect("page has areas");
        assert_eq!(back.areas.len(), 2);
        let first = back.areas[0].entry.as_ref().unwrap();
        assert_eq!(
            (first.code.as_str(), first.name.as_str()),
            ("GLA1", "First Floor")
        );
        assert_eq!(back.areas[1].entry.as_ref().unwrap().name, "Shed");
        // Arc geometry survives both conversions.
        let want_bulge = doc.areas[0].vertices[1].bulge;
        let got_bulge = back.areas[0].vertices[1].bulge;
        assert!(
            (got_bulge - want_bulge).abs() < 1e-9,
            "bulge {} vs {}",
            got_bulge,
            want_bulge
        );
        let (got_area, _) = back.areas[0].measure();
        let (want_area, _) = doc.areas[0].measure();
        assert!((got_area - want_area).abs() < 1e-6);
        // Labels survive with their styling.
        assert_eq!(back.labels.len(), 2);
        assert_eq!(back.labels[0].text, "Living");
        assert_eq!(back.labels[1].text, "½ Bath");
        assert_eq!(back.labels[1].rotation, 45.0);
        // And the rebuilt document re-encodes into a file the reader accepts.
        let bytes = write(&back);
        let reread = read(&bytes).expect("re-imports");
        assert_eq!(reread.areas.len(), 2);
        assert!(reread.warnings.is_empty(), "{:?}", reread.warnings);
    }
}
