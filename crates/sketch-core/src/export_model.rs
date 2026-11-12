//! Neutral export model — the headless stand-in for the browser's
//! `buildExportModel()` page plus the shared helpers the exporters consume
//! (`refs/exporter.js`). Sketch-app glue maps its rich `model.rs` types into
//! these so every exporter (SVG/DXF/PNG/PDF) works without the GUI.

use crate::geometry::{AreaCode, Edge, Pt};
use std::collections::BTreeMap;

/// A wall prepared for export. `openings` are the visible runs of a straight
/// single-line wall once its hosted openings are cut, as fractions along the
/// wall — precomputed by the app glue from the wall-solids module so the
/// headless core stays out of it (`WallSolids.openingRuns`).
#[derive(Debug, Clone, PartialEq)]
pub struct ExportLine {
    pub id: i64,
    pub start: Pt,
    pub end: Pt,
    /// `'exterior'` | `'interior'`.
    pub wall_type: String,
    pub color: Option<String>,
    pub line_thickness: f64,
    /// `'dashed'` | `'dotted'` | plain.
    pub line_style: Option<String>,
    pub bow_angle: f64,
    pub openings: Vec<(f64, f64)>,
}

impl ExportLine {
    pub fn is_exterior(&self) -> bool {
        self.wall_type == "exterior"
    }
}

/// A closed/defined area prepared for export.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExportPolygon {
    pub points: Vec<Pt>,
    pub edges: Vec<Edge>,
    /// Area code, `'UND'` while undefined.
    pub area_type: String,
    pub name: String,
    pub net: Option<f64>,
    pub area: f64,
    pub gross: f64,
    /// Per-area Factor override (`effectiveMultiplier`); `None` inherits the
    /// code's multiplier.
    pub effective_multiplier: Option<f64>,
    pub perimeter: f64,
    pub line_color: Option<String>,
    pub fill_color: Option<String>,
    pub fill_opacity: Option<f64>,
    pub fill_pattern: Option<String>,
    pub line_thickness: Option<f64>,
    pub label_position: Option<Pt>,
    pub label_rotation: f64,
    pub label_font_size: Option<f64>,
    pub label_font_family: Option<String>,
    pub label_color: Option<String>,
    pub label_bold: Option<bool>,
    pub label_italic: Option<bool>,
}

/// A free text label prepared for export.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExportLabel {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub font_size: Option<f64>,
    pub rotation: f64,
    pub color: Option<String>,
    pub font_family: Option<String>,
    pub bold: bool,
    pub italic: bool,
}

/// A placed symbol prepared for export.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExportSymbol {
    pub x: f64,
    pub y: f64,
    pub symbol_type: String,
    pub rotation: f64,
    pub scale: f64,
    pub flip_h: bool,
    pub color: Option<String>,
}

/// Wall-solid pieces keyed by their host line, for the geometric-thickness
/// export path (`computeWallSolids` → pieces). Empty when solids are off.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportSolid {
    pub line_id: i64,
    pub pieces: Vec<Vec<Pt>>,
}

/// One export page — the app is single-page, so the exporters see one page.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExportPage {
    pub lines: Vec<ExportLine>,
    pub polygons: Vec<ExportPolygon>,
    pub labels: Vec<ExportLabel>,
    pub symbols: Vec<ExportSymbol>,
    pub wall_solids: Vec<ExportSolid>,
    /// Normalized north angle (degrees clockwise from up).
    pub north_angle: f64,
}

/// The options the exporters read (`state.options` + `state.autoPost`).
#[derive(Debug, Clone, PartialEq)]
pub struct ExportOptions {
    pub exterior_color: String,
    pub interior_color: String,
    pub exterior_width: f64,
    pub interior_width: f64,
    /// Geometric wall-thickness master switch (`options.wallSolids`).
    pub wall_solids: bool,
    /// `state.autoPost.name`.
    pub auto_post_name: bool,
    /// `state.autoPost.calcs`.
    pub auto_post_calcs: bool,
    pub auto_post_suffix: String,
    /// `options.autoPostFont` for the SVG text family fallback.
    pub auto_post_font: String,
    /// `options.autoPostFontSize`.
    pub auto_post_font_size: f64,
    pub auto_post_bold: bool,
    pub auto_post_italic: bool,
    /// `options.textColor`.
    pub text_color: String,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            exterior_color: "#2B6CB0".to_owned(),
            interior_color: "#63B3ED".to_owned(),
            exterior_width: 4.0,
            interior_width: 2.0,
            wall_solids: false,
            auto_post_name: true,
            auto_post_calcs: true,
            auto_post_suffix: "sf".to_owned(),
            auto_post_font: "Arial".to_owned(),
            auto_post_font_size: 16.0,
            auto_post_bold: false,
            auto_post_italic: false,
            text_color: "#111111".to_owned(),
        }
    }
}

/// JS `Number(value)` with the NaN→0 guard (`Exporter.num`).
pub fn num_or_zero(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

pub fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Highest bow point of a curved wall in world coordinates (`curveApex`).
pub fn curve_apex(line: &ExportLine) -> Option<Pt> {
    let dx = line.end.x - line.start.x;
    let dy = line.end.y - line.start.y;
    let chord = dx.hypot(dy);
    let bow = num_or_zero(line.bow_angle);
    if chord < 1e-6 || bow.abs() < 0.5 {
        return None;
    }
    let rise = chord / 2.0 * (bow * std::f64::consts::PI / 720.0).tan();
    Some(Pt::new(
        (line.start.x + line.end.x) / 2.0 + (-dy / chord) * rise,
        (line.start.y + line.end.y) / 2.0 + (dx / chord) * rise,
    ))
}

/// Circle through a curved wall's endpoints and apex (`curveCircle`).
pub fn curve_circle(line: &ExportLine) -> Option<(Pt, f64)> {
    let apex = curve_apex(line)?;
    let s = line.start;
    let e = line.end;
    let d1 = 2.0 * (s.x * (e.y - apex.y) + e.x * (apex.y - s.y) + apex.x * (s.y - e.y));
    if d1.abs() < 1e-9 {
        return None;
    }
    let sq = |p: &Pt| p.x * p.x + p.y * p.y;
    let cx = (sq(&s) * (e.y - apex.y) + sq(&e) * (apex.y - s.y) + sq(&apex) * (s.y - e.y)) / d1;
    let cy = (sq(&s) * (apex.x - e.x) + sq(&e) * (s.x - apex.x) + sq(&apex) * (e.x - s.x)) / d1;
    Some((Pt::new(cx, cy), (s.x - cx).hypot(s.y - cy)))
}

/// Flattened ring points for export (`flattenRing(poly, undefined, 0.02)`).
pub fn flattened_points(poly: &ExportPolygon) -> Vec<Pt> {
    let engine = crate::geometry::GeometryEngine::new();
    engine.flatten_ring(&poly.points, &poly.edges, 0.02)
}

/// Bounds (world feet) covering everything the file export draws
/// (`getExportBounds`): line endpoints + curved apexes, flattened polygon
/// rings, labels, symbols and label positions. Empty default {0,10,0,10}.
pub fn get_export_bounds(page: &ExportPage) -> (Pt, Pt) {
    let mut min = Pt::new(f64::INFINITY, f64::INFINITY);
    let mut max = Pt::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut expand = |p: Pt| {
        if p.x.is_finite() && p.y.is_finite() {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
    };
    for line in &page.lines {
        expand(line.start);
        expand(line.end);
        if let Some(apex) = curve_apex(line) {
            expand(apex);
        }
    }
    for poly in &page.polygons {
        for p in flattened_points(poly) {
            expand(p);
        }
    }
    for label in &page.labels {
        expand(Pt::new(num_or_zero(label.x), num_or_zero(label.y)));
    }
    for symbol in &page.symbols {
        expand(Pt::new(num_or_zero(symbol.x), num_or_zero(symbol.y)));
    }
    for poly in &page.polygons {
        if let Some(position) = poly.label_position {
            expand(position);
        }
    }
    if min.x.is_infinite() || min.y.is_infinite() {
        return (Pt::new(0.0, 0.0), Pt::new(10.0, 10.0));
    }
    (min, max)
}

/// Include the complete rotated/scaled symbol footprint, not just its anchor.
pub fn get_export_bounds_with_library(
    page: &ExportPage,
    library: Option<&crate::symbols::SymbolLibrary>,
) -> (Pt, Pt) {
    let (mut min, mut max) = get_export_bounds(page);
    if let Some(library) = library {
        for s in &page.symbols {
            if let Some(def) = library.symbol(&s.symbol_type) {
                let scale = if s.scale == 0.0 { 1.0 } else { s.scale };
                for p in
                    crate::symbols::world_corners(def.bounds, s.x, s.y, s.rotation, scale, s.flip_h)
                {
                    min.x = min.x.min(p.x);
                    min.y = min.y.min(p.y);
                    max.x = max.x.max(p.x);
                    max.y = max.y.max(p.y);
                }
            }
        }
    }
    (min, max)
}

/// Area Line-color overrides keyed by line id, mirroring
/// `canvas.buildAreaLineColors` / `Exporter.buildAreaLineColors`.
pub fn build_area_line_colors(page: &ExportPage, area_codes: &[AreaCode]) -> BTreeMap<i64, String> {
    let code_map = area_codes::as_map(area_codes);
    let mut colors = BTreeMap::new();
    for poly in &page.polygons {
        if poly.area_type.is_empty() || poly.area_type == "UND" {
            continue;
        }
        let color = poly
            .line_color
            .clone()
            .or_else(|| code_map.get(&poly.area_type).map(|c| c.color.clone()));
        let Some(color) = color else { continue };
        for edge in &poly.edges {
            if let Some(line_id) = edge.line_id {
                colors.entry(line_id).or_insert_with(|| color.clone());
            }
        }
    }
    colors
}

/// Auto-posted label lines of an area (`autoPostLines`): name and net calcs.
pub fn auto_post_lines(
    poly: &ExportPolygon,
    options: &ExportOptions,
    area_codes: &[AreaCode],
) -> Vec<String> {
    if poly.area_type.is_empty() || poly.area_type == "UND" {
        return Vec::new();
    }
    let mut lines = Vec::new();
    if options.auto_post_name {
        lines.push(if !poly.name.is_empty() {
            poly.name.clone()
        } else {
            area_codes::as_map(area_codes)
                .get(&poly.area_type)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| poly.area_type.clone())
        });
    }
    if options.auto_post_calcs {
        let value = num_or_zero(poly.net.unwrap_or(poly.area)).round();
        lines.push(format!("{} {}", value, options.auto_post_suffix));
    }
    lines
}

/// Wall-solid pieces for the geometric-thickness export path: the exporter
/// consults precomputed pieces rather than the wall-solids module so the
/// core stays headless.
pub fn solids_enabled(options: &ExportOptions) -> bool {
    options.wall_solids
}

mod area_codes {
    use super::AreaCode;
    use std::collections::BTreeMap;

    pub fn as_map(codes: &[AreaCode]) -> BTreeMap<String, AreaCode> {
        crate::geometry::area_codes::as_map(codes)
    }

    /// Look a code definition up by code string.
    pub fn map_lookup<'a>(codes: &'a [AreaCode], code: &str) -> Option<&'a AreaCode> {
        codes.iter().find(|c| c.code == code)
    }
}

/// Lookup helper re-exported for the exporters.
pub fn area_code_lookup<'a>(codes: &'a [AreaCode], code: &str) -> Option<&'a AreaCode> {
    area_codes::map_lookup(codes, code)
}
