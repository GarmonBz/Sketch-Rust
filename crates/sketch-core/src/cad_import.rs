//! CAD IMPORT — clean-room port of `refs/cad-import.js` (window.CadImport):
//! a minimal DXF reader (group-code/value pair stream), unit detection
//! ($INSUNITS → feet scale factors with size inference), and conversion of
//! 2D entities into the sketch app's feet-space lines/polygons/labels.
//!
//! UNIT HANDLING — read this before changing anything.
//! The sketch app is FOOT-based. CAD files are usually INCH-based (DXF
//! $INSUNITS 1) or metric. Insunits → feet scale factors:
//!   1 inches → 1/12      4 mm → 1/304.8
//!   2 feet   → 1         5 cm → 1/30.48
//!   3 miles  → 5280      6 m  → 1/0.3048
//!   0/other  → assume inches (1/12), the common case for US plan sets.
//! Metric drawings are auto-detected by extent when the file is unitless.

use crate::geometry::Pt;
use std::collections::BTreeMap;

const UNNAMED_LAYER: &str = "0";

/// One (group code, value) pair from the DXF text stream.
#[derive(Debug, Clone, PartialEq)]
pub struct DxfPair {
    pub code: i32,
    pub value: String,
}

/// `parseDxfPairs` — split into lines and read code/value pairs; a
/// non-numeric code line is skipped.
pub fn parse_dxf_pairs(text: &str) -> Vec<DxfPair> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalized.split('\n').collect();
    let mut pairs = Vec::new();
    let mut i = 0;
    while i + 1 < lines.len() {
        let code = lines[i].trim().parse::<i32>().ok();
        if let Some(code) = code {
            pairs.push(DxfPair {
                code,
                value: lines[i + 1].to_owned(),
            });
        }
        i += 2;
    }
    pairs
}

/// `sectionRanges` — map section name → pair-index range.
pub fn section_ranges(pairs: &[DxfPair]) -> BTreeMap<String, (usize, usize)> {
    let mut ranges = BTreeMap::new();
    let mut current: Option<String> = None;
    for (i, p) in pairs.iter().enumerate() {
        if p.code == 0 && p.value == "SECTION" {
            current = None;
            continue;
        }
        if p.code == 2 && current.is_none() {
            current = Some(p.value.clone());
            ranges.insert(current.clone().expect("set above"), (i, pairs.len()));
            continue;
        }
        if p.code == 0 && p.value == "ENDSEC" && current.is_some() {
            if let Some(name) = &current {
                if let Some(range) = ranges.get_mut(name) {
                    range.1 = i;
                }
            }
            current = None;
        }
    }
    ranges
}

/// `headerInsunits` — the integer $INSUNITS from the HEADER section.
pub fn header_insunits(
    pairs: &[DxfPair],
    ranges: &BTreeMap<String, (usize, usize)>,
) -> Option<i32> {
    let &(start, end) = ranges.get("HEADER")?;
    let mut i = start;
    while i + 1 < end {
        if pairs[i].code == 9 && pairs[i].value == "$INSUNITS" {
            return pairs[i + 1].value.trim().parse::<i32>().ok();
        }
        i += 1;
    }
    None
}

/// One 2D entity read from the ENTITIES section, in the file's own units.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DxfEntity {
    pub type_name: String,
    pub layer: String,
    pub flag: i32,
    pub x: f64,
    pub y: f64,
    pub x2: Option<f64>,
    pub y2: Option<f64>,
    pub radius: Option<f64>,
    pub start_angle: Option<f64>,
    pub end_angle: Option<f64>,
    pub text: String,
    pub name: Option<String>,
    pub scale_x: Option<f64>,
    pub scale_y: Option<f64>,
    pub extra_points: Vec<(f64, f64)>,
    pub filled: bool,
    pub vertices: Vec<DxfVertex>,
    /// Scratch pointer while accumulating legacy POLYLINE vertices.
    pub vtx_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DxfVertex {
    pub x: f64,
    pub y: f64,
    pub bulge: f64,
}

fn js_num(v: &str) -> f64 {
    v.trim().parse::<f64>().unwrap_or(0.0)
}

/// `readEntities` — the 2D entities we can represent. Vertex accumulation is
/// an explicit state machine: LWPOLYLINE and legacy POLYLINE both emit
/// repeated 10/20 pairs, and sharing per-entity fields collapses a
/// 4-vertex rectangle into duplicated points.
pub fn read_entities(
    pairs: &[DxfPair],
    ranges: &BTreeMap<String, (usize, usize)>,
) -> Vec<DxfEntity> {
    let Some(&(start, end)) = ranges.get("ENTITIES") else {
        return Vec::new();
    };
    read_entity_span(&pairs[start..end])
}

fn read_entity_span(pairs: &[DxfPair]) -> Vec<DxfEntity> {
    let mut out: Vec<DxfEntity> = Vec::new();
    let mut cur: Option<usize> = None;

    for p in pairs {
        let code = p.code;
        let v = &p.value;

        if code == 0 {
            let t = v.trim().to_uppercase();
            match t.as_str() {
                "LINE" | "LWPOLYLINE" | "POLYLINE" | "CIRCLE" | "ARC" | "TEXT" | "MTEXT"
                | "INSERT" | "POINT" | "ELLIPSE" | "SOLID" => {
                    out.push(DxfEntity {
                        type_name: t.to_owned(),
                        layer: UNNAMED_LAYER.to_owned(),
                        ..Default::default()
                    });
                    cur = Some(out.len() - 1);
                }
                "VERTEX" => match cur {
                    Some(index) if out[index].type_name == "POLYLINE" => {
                        out[index].vertices.push(DxfVertex::default());
                        out[index].vtx_open = true;
                    }
                    _ => cur = None,
                },
                "SEQEND" => {
                    if let Some(index) = cur {
                        if out[index].type_name == "POLYLINE" {
                            out[index].vtx_open = false;
                        }
                    }
                    cur = None;
                }
                _ => cur = None,
            }
            continue;
        }
        let Some(index) = cur else { continue };
        let is_polyline =
            out[index].type_name == "LWPOLYLINE" || out[index].type_name == "POLYLINE";
        match code {
            8 => {
                out[index].layer = if v.is_empty() {
                    UNNAMED_LAYER.to_owned()
                } else {
                    v.clone()
                }
            }
            10 => {
                if is_polyline {
                    if out[index].type_name == "LWPOLYLINE" || !out[index].vtx_open {
                        out[index].vertices.push(DxfVertex::default());
                        out[index].vtx_open = true;
                    }
                    let last = out[index].vertices.len() - 1;
                    out[index].vertices[last].x = js_num(v);
                    out[index].x = js_num(v);
                } else {
                    out[index].x = js_num(v);
                }
            }
            20 => {
                if is_polyline {
                    if !out[index].vtx_open {
                        out[index].vertices.push(DxfVertex::default());
                        out[index].vtx_open = true;
                    }
                    let last = out[index].vertices.len() - 1;
                    out[index].vertices[last].y = js_num(v);
                    out[index].y = js_num(v);
                } else {
                    out[index].y = js_num(v);
                }
            }
            11 => out[index].x2 = Some(js_num(v)),
            21 => out[index].y2 = Some(js_num(v)),
            40 => out[index].radius = Some(js_num(v)),
            41 => {
                out[index].scale_x = Some(js_num(v));
                if out[index].type_name == "ELLIPSE" {
                    out[index].start_angle = Some(js_num(v));
                }
            }
            42 => {
                if out[index].type_name == "INSERT" {
                    out[index].scale_y = Some(js_num(v));
                    continue;
                }
                if out[index].type_name == "ELLIPSE" {
                    out[index].end_angle = Some(js_num(v));
                    continue;
                }
                let bulge = js_num(v);
                if out[index].vtx_open {
                    let last = out[index].vertices.len() - 1;
                    out[index].vertices[last].bulge = bulge;
                } else {
                    out[index].vertices.push(DxfVertex {
                        bulge,
                        ..Default::default()
                    });
                    out[index].vtx_open = true;
                }
            }
            50 => out[index].start_angle = Some(js_num(v)),
            51 => out[index].end_angle = Some(js_num(v)),
            70 => out[index].flag = v.trim().parse::<i32>().unwrap_or(0),
            1 => out[index].text += v,
            3 => out[index].text += v,
            2 => out[index].name = Some(v.clone()),
            12 | 13 if out[index].type_name == "SOLID" => {
                out[index].extra_points.push((js_num(v), 0.0));
            }
            22 | 23 if out[index].type_name == "SOLID" => {
                if let Some(p) = out[index].extra_points.last_mut() {
                    p.1 = js_num(v);
                }
            }
            _ => {}
        }
    }
    out
}

/// Resolve BLOCK/INSERT records into model-space geometry. Curves are sampled
/// before affine transforms so rotated, mirrored and nonuniform inserts retain
/// their shape. Missing references remain visible in the normal skipped count.
pub fn expanded_entities(
    pairs: &[DxfPair],
    ranges: &BTreeMap<String, (usize, usize)>,
) -> Result<Vec<DxfEntity>, String> {
    let mut blocks = BTreeMap::new();
    if let Some(&(start, end)) = ranges.get("BLOCKS") {
        let mut i = start;
        while i < end {
            if pairs[i].code != 0 || pairs[i].value.trim() != "BLOCK" {
                i += 1;
                continue;
            }
            i += 1;
            let mut name = String::new();
            let mut base = (0.0, 0.0);
            while i < end && pairs[i].code != 0 {
                match pairs[i].code {
                    2 => name = pairs[i].value.clone(),
                    10 => base.0 = js_num(&pairs[i].value),
                    20 => base.1 = js_num(&pairs[i].value),
                    _ => {}
                }
                i += 1;
            }
            let first = i;
            while i < end && !(pairs[i].code == 0 && pairs[i].value.trim() == "ENDBLK") {
                i += 1;
            }
            blocks.insert(name, (base, read_entity_span(&pairs[first..i])));
        }
    }
    // x'=a*x+c*y+tx; y'=b*x+d*y+ty.
    type Matrix = [f64; 6];
    fn point(m: Matrix, x: f64, y: f64) -> (f64, f64) {
        (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
    }
    fn compose(p: Matrix, m: Matrix) -> Matrix {
        let (tx, ty) = point(p, m[4], m[5]);
        [
            p[0] * m[0] + p[2] * m[1],
            p[1] * m[0] + p[3] * m[1],
            p[0] * m[2] + p[2] * m[3],
            p[1] * m[2] + p[3] * m[3],
            tx,
            ty,
        ]
    }
    type Blocks = BTreeMap<String, ((f64, f64), Vec<DxfEntity>)>;
    fn visit(
        entities: &[DxfEntity],
        blocks: &Blocks,
        m: Matrix,
        depth: usize,
        inherited_layer: &str,
        out: &mut Vec<DxfEntity>,
        budget: &mut usize,
    ) -> Result<(), String> {
        if depth > 24 {
            return Err("DXF block nesting is cyclic or too deep.".into());
        }
        for source in entities {
            if *budget == 0 {
                return Err("DXF expands to too many entities.".into());
            }
            *budget -= 1;
            if ![
                Some(source.x),
                Some(source.y),
                source.x2,
                source.y2,
                source.radius,
                source.start_angle,
                source.end_angle,
                source.scale_x,
                source.scale_y,
            ]
            .into_iter()
            .flatten()
            .all(f64::is_finite)
                || source
                    .vertices
                    .iter()
                    .any(|v| ![v.x, v.y, v.bulge].into_iter().all(f64::is_finite))
            {
                return Err("DXF contains invalid coordinates.".into());
            }
            let layer = if source.layer == "0" {
                inherited_layer
            } else {
                &source.layer
            };
            if source.type_name == "INSERT" {
                if let Some((base, children)) = source.name.as_ref().and_then(|n| blocks.get(n)) {
                    let angle = source.start_angle.unwrap_or(0.0).to_radians();
                    let (sin, cos) = angle.sin_cos();
                    let sx = source.scale_x.unwrap_or(1.0);
                    let sy = source.scale_y.unwrap_or(1.0);
                    let local = [
                        cos * sx,
                        sin * sx,
                        -sin * sy,
                        cos * sy,
                        source.x - cos * sx * base.0 + sin * sy * base.1,
                        source.y - sin * sx * base.0 - cos * sy * base.1,
                    ];
                    visit(
                        children,
                        blocks,
                        compose(m, local),
                        depth + 1,
                        layer,
                        out,
                        budget,
                    )?;
                    continue;
                }
            }
            let mut e = source.clone();
            e.layer = layer.to_owned();
            let mut samples = Vec::new();
            match e.type_name.as_str() {
                "CIRCLE" | "ARC" => {
                    let start = if e.type_name == "CIRCLE" {
                        0.0
                    } else {
                        e.start_angle.unwrap_or(0.0).to_radians()
                    };
                    let mut end = if e.type_name == "CIRCLE" {
                        std::f64::consts::TAU
                    } else {
                        e.end_angle.unwrap_or(0.0).to_radians()
                    };
                    if end < start {
                        end = start + (end - start).rem_euclid(std::f64::consts::TAU);
                    }
                    let count = ((end - start).abs() / (std::f64::consts::PI / 48.0))
                        .ceil()
                        .clamp(2.0, 2048.0) as usize;
                    for i in 0..=count {
                        let a = start + (end - start) * i as f64 / count as f64;
                        let r = e.radius.unwrap_or(0.0);
                        samples.push((e.x + r * a.cos(), e.y + r * a.sin()));
                    }
                    e.flag = i32::from(e.type_name == "CIRCLE");
                }
                "ELLIPSE" => {
                    let start = e.start_angle.unwrap_or(0.0);
                    let mut end = e.end_angle.unwrap_or(std::f64::consts::TAU);
                    if end < start {
                        end = start + (end - start).rem_euclid(std::f64::consts::TAU);
                    }
                    let ax = e.x2.unwrap_or(0.0);
                    let ay = e.y2.unwrap_or(0.0);
                    let ratio = e.radius.unwrap_or(1.0);
                    for i in 0..=96 {
                        let a = start + (end - start) * i as f64 / 96.0;
                        samples.push((
                            e.x + ax * a.cos() - ay * ratio * a.sin(),
                            e.y + ay * a.cos() + ax * ratio * a.sin(),
                        ));
                    }
                    e.flag = i32::from((end - start - std::f64::consts::TAU).abs() < 1e-6);
                }
                "SOLID" => {
                    e.filled = true;
                    samples.push((e.x, e.y));
                    samples.push((e.x2.unwrap_or(e.x), e.y2.unwrap_or(e.y)));
                    if e.extra_points.len() == 2 {
                        samples.push(e.extra_points[1]);
                    }
                    if let Some(p) = e.extra_points.first() {
                        samples.push(*p);
                    }
                    e.flag = 1;
                }
                "LWPOLYLINE" | "POLYLINE" => {
                    for (i, v) in e.vertices.iter().enumerate() {
                        samples.push((v.x, v.y));
                        let next = e.vertices.get(i + 1).or_else(|| {
                            if e.flag & 1 != 0 {
                                e.vertices.first()
                            } else {
                                None
                            }
                        });
                        if let Some(next) = next {
                            samples.extend(
                                bulge_to_arc_points(
                                    Pt::new(v.x, v.y),
                                    Pt::new(next.x, next.y),
                                    v.bulge,
                                )
                                .iter()
                                .map(|p| (p.x, p.y)),
                            );
                        }
                    }
                }
                _ => {}
            }
            if !samples.is_empty() {
                e.type_name = "LWPOLYLINE".into();
                e.vertices = samples
                    .into_iter()
                    .map(|(x, y)| {
                        let (x, y) = point(m, x, y);
                        DxfVertex { x, y, bulge: 0.0 }
                    })
                    .collect();
            } else {
                (e.x, e.y) = point(m, e.x, e.y);
                if let (Some(x), Some(y)) = (e.x2, e.y2) {
                    let (x, y) = point(m, x, y);
                    e.x2 = Some(x);
                    e.y2 = Some(y);
                }
                if e.type_name == "TEXT" || e.type_name == "MTEXT" {
                    let a = source.start_angle.unwrap_or(0.0).to_radians();
                    e.start_angle = Some(
                        (m[1] * a.cos() + m[3] * a.sin())
                            .atan2(m[0] * a.cos() + m[2] * a.sin())
                            .to_degrees(),
                    );
                    e.radius = Some(source.radius.unwrap_or(6.0) * m[2].hypot(m[3]));
                }
            }
            out.push(e);
        }
        Ok(())
    }
    let mut out = Vec::new();
    visit(
        &read_entities(pairs, ranges),
        &blocks,
        [1., 0., 0., 1., 0., 0.],
        0,
        "0",
        &mut out,
        &mut 100_000,
    )?;
    Ok(out)
}

/// `extentOf` — geometry extent in the file's own units.
pub fn extent_of(entities: &[DxfEntity]) -> Option<(f64, f64, f64, f64)> {
    let mut min = (f64::INFINITY, f64::INFINITY);
    let mut max = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut n = 0;
    let mut add = |x: f64, y: f64| {
        if x.is_finite() && y.is_finite() {
            min.0 = min.0.min(x);
            min.1 = min.1.min(y);
            max.0 = max.0.max(x);
            max.1 = max.1.max(y);
            n += 1;
        }
    };
    for e in entities {
        if !e.vertices.is_empty() {
            for v in &e.vertices {
                add(v.x, v.y);
            }
        } else if let (Some(radius), true) = (e.radius, e.radius.is_some()) {
            add(e.x - radius, e.y - radius);
            add(e.x + radius, e.y + radius);
        } else {
            add(e.x, e.y);
            if let (Some(x2), Some(y2)) = (e.x2, e.y2) {
                add(x2, y2);
            }
        }
    }
    if n == 0 {
        return None;
    }
    Some((min.0, min.1, max.0, max.1))
}

/// The chosen file-units → feet conversion (`pickScale`).
#[derive(Debug, Clone, PartialEq)]
pub struct UnitChoice {
    pub scale: f64,
    pub units: &'static str,
    pub source: &'static str,
}

/// `pickScale` — insunits table with size inference for unitless files.
pub fn pick_scale(insunits: Option<i32>, extent: Option<(f64, f64, f64, f64)>) -> UnitChoice {
    const INCH: f64 = 1.0 / 12.0;
    const FEET: f64 = 1.0;
    const MILE: f64 = 5280.0;
    const MM: f64 = 1.0 / 304.8;
    const CM: f64 = 1.0 / 30.48;
    const M: f64 = 1.0 / 0.3048;

    match insunits {
        Some(2) => {
            return UnitChoice {
                scale: FEET,
                units: "feet",
                source: "insunits",
            }
        }
        Some(3) => {
            return UnitChoice {
                scale: MILE,
                units: "miles",
                source: "insunits",
            }
        }
        Some(4) => {
            return UnitChoice {
                scale: MM,
                units: "mm",
                source: "insunits",
            }
        }
        Some(5) => {
            return UnitChoice {
                scale: CM,
                units: "cm",
                source: "insunits",
            }
        }
        Some(6) => {
            return UnitChoice {
                scale: M,
                units: "m",
                source: "insunits",
            }
        }
        Some(1) => {
            return UnitChoice {
                scale: INCH,
                units: "inches",
                source: "insunits",
            }
        }
        _ => {}
    }

    // Unitless or unknown: infer from size. A house is tens of metres or
    // hundreds of inches; both are four-digit numbers.
    if let Some((min_x, min_y, max_x, max_y)) = extent {
        let span = (max_x - min_x).max(max_y - min_y);
        if span > 0.0 {
            let as_inches = span * INCH;
            let as_feet = span * FEET;
            let as_mm = span * MM;
            let as_m = span * M;
            let plausible = |ft: f64| ft > 8.0 && ft < 2000.0;
            if plausible(as_mm) && span > 1000.0 {
                return UnitChoice {
                    scale: MM,
                    units: "mm",
                    source: "inferred",
                };
            }
            if plausible(as_inches) {
                return UnitChoice {
                    scale: INCH,
                    units: "inches",
                    source: "inferred",
                };
            }
            if plausible(as_feet) {
                return UnitChoice {
                    scale: FEET,
                    units: "feet",
                    source: "inferred",
                };
            }
            if plausible(as_m) {
                return UnitChoice {
                    scale: M,
                    units: "m",
                    source: "inferred",
                };
            }
        }
    }
    UnitChoice {
        scale: INCH,
        units: "inches",
        source: "default",
    }
}

/// `bulgeToArcPoints` — flatten a DXF bulge (bulge = tan(sweep/4)) into the
/// intermediate arc points between v0 and v1.
pub fn bulge_to_arc_points(v0: Pt, v1: Pt, bulge: f64) -> Vec<Pt> {
    let b = bulge;
    if b.abs() < 1e-9 {
        return Vec::new();
    }
    let chord = (v1.x - v0.x).hypot(v1.y - v0.y);
    if chord < 1e-12 {
        return Vec::new();
    }
    let sweep = 4.0 * b.atan();
    let radius = chord / (2.0 * (sweep.abs() / 2.0).sin().max(1e-12));
    // Centre lies on the perpendicular bisector; sign of bulge picks the side.
    let mx = (v0.x + v1.x) / 2.0;
    let my = (v0.y + v1.y) / 2.0;
    let nx = -(v1.y - v0.y) / chord;
    let ny = (v1.x - v0.x) / chord;
    let off = chord * (1.0 - b * b) / (4.0 * b);
    let cx = mx + nx * off;
    let cy = my + ny * off;
    let a0 = (v0.y - cy).atan2(v0.x - cx);
    let steps =
        2.0f64.max(48f64.min(6f64.max((sweep.abs() / (std::f64::consts::PI / 18.0)).round())));
    let steps = steps as usize;
    let mut pts = Vec::new();
    for s in 1..steps {
        let a = a0 + sweep * (s as f64 / steps as f64);
        pts.push(Pt::new(cx + radius * a.cos(), cy + radius * a.sin()));
    }
    pts
}

fn circle_points(cx: f64, cy: f64, r: f64) -> Vec<Pt> {
    let steps = 32;
    (0..steps)
        .map(|i| {
            let a = (i as f64 / steps as f64) * std::f64::consts::TAU;
            Pt::new(cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

fn arc_points(cx: f64, cy: f64, r: f64, start_deg: f64, end_deg: f64) -> Vec<Pt> {
    let a0 = start_deg * std::f64::consts::PI / 180.0;
    let mut a1 = end_deg * std::f64::consts::PI / 180.0;
    // DXF arcs run counter-clockwise; unwrap so a 350→10 arc sweeps 20 deg.
    while a1 < a0 {
        a1 += std::f64::consts::TAU;
    }
    let sweep = a1 - a0;
    let steps =
        6usize.max(64usize.min((sweep.abs() / (std::f64::consts::PI / 36.0)).round() as usize + 1));
    (0..=steps)
        .map(|i| {
            let a = a0 + sweep * (i as f64 / steps as f64);
            Pt::new(cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// Converted sketch geometry in feet (`entitiesToSketch`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConvertedSketch {
    pub lines: Vec<(Pt, Pt)>,
    pub polygons: Vec<Vec<Pt>>,
    pub labels: Vec<(Pt, String)>,
    pub skipped: BTreeMap<String, usize>,
}

pub fn entities_to_sketch(
    entities: &[DxfEntity],
    scale: f64,
    offset: (f64, f64),
) -> ConvertedSketch {
    let mut out = ConvertedSketch::default();
    let (dx, dy) = offset;
    let t = |x: f64, y: f64| Pt::new(x * scale + dx, y * scale + dy);

    for e in entities {
        let type_name = e.type_name.as_str();
        match type_name {
            "LINE" => {
                let a = t(e.x, e.y);
                let bx = e.x2.unwrap_or(0.0);
                let by = e.y2.unwrap_or(0.0);
                let b = t(bx, by);
                if (b.x - a.x).hypot(b.y - a.y) > 1e-9 {
                    out.lines.push((a, b));
                } else {
                    *out.skipped.entry("degenerate".to_owned()).or_insert(0) += 1;
                }
            }
            "LWPOLYLINE" | "POLYLINE" => {
                let verts = &e.vertices;
                if verts.len() < 2 {
                    *out.skipped.entry("short".to_owned()).or_insert(0) += 1;
                    continue;
                }
                let mut pts: Vec<Pt> = Vec::new();
                for (i, v0) in verts.iter().enumerate() {
                    pts.push(t(v0.x, v0.y));
                    let next = verts.get(i + 1);
                    // Bulge on vertex i describes the arc to vertex i+1.
                    if let Some(v1) = next {
                        if v0.bulge != 0.0 {
                            for p in bulge_to_arc_points(
                                Pt::new(v0.x, v0.y),
                                Pt::new(v1.x, v1.y),
                                v0.bulge,
                            ) {
                                pts.push(t(p.x, p.y));
                            }
                        }
                    } else if v0.bulge != 0.0 && (e.flag & 1) != 0 {
                        // Closed polyline: last vertex bulges back to the first.
                        let first = &verts[0];
                        for p in bulge_to_arc_points(
                            Pt::new(v0.x, v0.y),
                            Pt::new(first.x, first.y),
                            v0.bulge,
                        ) {
                            pts.push(t(p.x, p.y));
                        }
                    }
                }
                if (e.flag & 1) != 0 && pts.len() > 2 {
                    out.polygons.push(pts);
                } else {
                    for k in 0..pts.len().saturating_sub(1) {
                        out.lines.push((pts[k], pts[k + 1]));
                    }
                }
            }
            "CIRCLE" => {
                let radius = e.radius.unwrap_or(0.0);
                let cp = circle_points(e.x, e.y, radius);
                if !cp.is_empty() {
                    out.polygons.push(cp.iter().map(|p| t(p.x, p.y)).collect());
                }
            }
            "ARC" => {
                let radius = e.radius.unwrap_or(0.0);
                let ap = arc_points(
                    e.x,
                    e.y,
                    radius,
                    e.start_angle.unwrap_or(0.0),
                    e.end_angle.unwrap_or(0.0),
                );
                for j in 0..ap.len().saturating_sub(1) {
                    out.lines
                        .push((t(ap[j].x, ap[j].y), t(ap[j + 1].x, ap[j + 1].y)));
                }
            }
            "TEXT" | "MTEXT" => {
                let trimmed = strip_mtext_codes(&e.text);
                if !trimmed.trim().is_empty() {
                    out.labels.push((t(e.x, e.y), trimmed));
                }
            }
            "POINT" => {
                out.labels.push((t(e.x, e.y), String::new()));
            }
            other => {
                *out.skipped
                    .entry(if other.is_empty() {
                        "unknown".to_owned()
                    } else {
                        other.to_lowercase()
                    })
                    .or_insert(0) += 1;
            }
        }
    }
    out
}

/// `String(e.text).replace(/\\[A-Za-z0-9.| ;]+;?/g, '')` — strip MTEXT
/// inline codes.
fn strip_mtext_codes(text: &str) -> String {
    let mut out = String::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == '\\' {
            // Try to match the code pattern.
            let mut j = i + 1;
            let matched = |from: usize, to: usize| -> Option<String> {
                if from >= to {
                    return None;
                }
                Some(bytes[from..to].iter().collect())
            };
            let mut k = j;
            while k < bytes.len()
                && (bytes[k].is_ascii_alphanumeric() || matches!(bytes[k], '.' | '|' | ' ' | ';'))
            {
                k += 1;
            }
            if k > j {
                if let Some(_code) = matched(j, k) {
                    // Optional trailing ';'
                    if k < bytes.len() && bytes[k] == ';' {
                        k += 1;
                    }
                    i = k;
                    continue;
                }
            }
            j = k;
            i = j;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    out.trim().to_owned()
}

/// The normalized, scaled parse result (`CadImport.parseDxf`).
#[derive(Debug, Clone, PartialEq)]
pub struct DxfParseResult {
    pub extent: Option<(f64, f64, f64, f64)>,
    pub units: &'static str,
    pub unit_source: &'static str,
    pub scale: f64,
    pub entity_count: usize,
    pub layers: Vec<String>,
    pub lines: Vec<(Pt, Pt)>,
    pub polygons: Vec<Vec<Pt>>,
    pub labels: Vec<(Pt, String)>,
    pub skipped: BTreeMap<String, usize>,
}

/// `CadImport.parseDxf` — parse DXF text into a normalized, scaled sketch
/// payload. `scale_override` (feet per file unit) and `offset` mirror the
/// JS options; `None`/origin keep the detected unit.
pub fn parse_dxf(
    text: &str,
    scale_override: Option<f64>,
    offset: (f64, f64),
) -> Result<DxfParseResult, String> {
    let pairs = parse_dxf_pairs(text);
    let ranges = section_ranges(&pairs);
    let insunits = header_insunits(&pairs, &ranges);
    let entities = expanded_entities(&pairs, &ranges)?;
    if entities.is_empty() {
        return Err("No supported entities found in DXF.".to_owned());
    }
    let extent = extent_of(&entities);
    let unit = match scale_override {
        Some(scale) => UnitChoice {
            scale,
            units: "custom",
            source: "override",
        },
        None => pick_scale(insunits, extent),
    };
    let converted = entities_to_sketch(&entities, unit.scale, offset);
    // Insertion-ordered unique layer names (the JS `new Set` semantics).
    let mut layers: Vec<String> = Vec::new();
    for e in &entities {
        let layer = if e.layer.is_empty() {
            UNNAMED_LAYER.to_owned()
        } else {
            e.layer.clone()
        };
        if !layers.contains(&layer) {
            layers.push(layer);
        }
    }
    Ok(DxfParseResult {
        extent,
        units: unit.units,
        unit_source: unit.source,
        scale: unit.scale,
        entity_count: entities.len(),
        layers,
        lines: converted.lines,
        polygons: converted.polygons,
        labels: converted.labels,
        skipped: converted.skipped,
    })
}

/// The CAD IMPORT UI layer's `describe(result)`: the alert text after an
/// import - what went in, from how many entities, the units and their source,
/// and what was skipped.
pub fn describe_import(r: &DxfParseResult) -> String {
    let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
    let mut parts: Vec<String> = Vec::new();
    if !r.lines.is_empty() {
        parts.push(plural(r.lines.len(), "line"));
    }
    if !r.polygons.is_empty() {
        parts.push(plural(r.polygons.len(), "shape"));
    }
    // `insertIntoApp` drops labels with no text.
    let labels = r.labels.iter().filter(|(_, t)| !t.is_empty()).count();
    if labels > 0 {
        parts.push(plural(labels, "label"));
    }
    let summary = if parts.is_empty() {
        "no supported geometry".to_owned()
    } else {
        parts.join(", ")
    };
    let skipped = if r.skipped.is_empty() {
        String::new()
    } else {
        let list: Vec<String> = r.skipped.iter().map(|(k, n)| format!("{k} x{n}")).collect();
        format!(" Skipped: {}.", list.join(", "))
    };
    format!(
        "Imported {summary} from {} CAD entities.\nUnits detected: {} ({}).{skipped}",
        r.entity_count, r.units, r.unit_source
    )
}

/// `CadImport.cadImportDwgHint`.
pub fn cad_import_dwg_hint() -> &'static str {
    "DWG is a binary format this app cannot read directly. Convert it to DXF first (free options: the ODA File Converter on Windows, or LibreCAD / QCAD on any platform), then use Import CAD on the .dxf."
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECT_DXF: &str = "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n1\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n8\nWALLS\n10\n0.0\n20\n0.0\n11\n240.0\n21\n0.0\n0\nLWPOLYLINE\n8\nAREAS\n70\n1\n10\n0.0\n20\n0.0\n10\n240.0\n20\n0.0\n10\n240.0\n20\n192.0\n10\n0.0\n20\n192.0\n0\nENDSEC\n0\nEOF\n";

    #[test]
    fn the_import_alert_matches_the_originals_describe() {
        let r = parse_dxf(RECT_DXF, None, (0.0, 0.0)).unwrap();
        let text = describe_import(&r);
        assert!(text.starts_with("Imported 1 line, 1 shape from "), "{text}");
        assert!(text.contains(" CAD entities.\nUnits detected: "), "{text}");
        assert!(text.contains("(insunits)."), "{text}");
        let mut empty = r.clone();
        empty.lines.clear();
        empty.polygons.clear();
        empty.skipped.insert("SPLINE".into(), 3);
        let t = describe_import(&empty);
        assert!(t.starts_with("Imported no supported geometry from "), "{t}");
        assert!(t.ends_with(" Skipped: SPLINE x3."), "{t}");
    }

    #[test]
    fn pairs_reader_skips_bad_code_lines() {
        // The JS `i += 2` pairing advances past a skipped code line, so a
        // bad code line re-aligns the stream (transcribed behavior).
        let pairs = parse_dxf_pairs("junk\n0\n10\n1.5\n");
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].code, 10);
        assert_eq!(pairs[0].value, "1.5");
    }

    #[test]
    fn header_insunits_read() {
        let pairs = parse_dxf_pairs(RECT_DXF);
        let ranges = section_ranges(&pairs);
        assert_eq!(header_insunits(&pairs, &ranges), Some(1));
    }

    #[test]
    fn minimal_dxf_fixture_round_trip_through_export() {
        // Export a simple sketch to DXF, then parse it back through the real
        // parser and check the geometry round-trips in feet.
        use crate::export_dxf::export_dxf;
        use crate::export_model::{ExportLine, ExportOptions, ExportPage, ExportPolygon};
        use crate::geometry::Edge;

        let page = ExportPage {
            lines: vec![ExportLine {
                id: 1,
                start: Pt::new(0.0, 0.0),
                end: Pt::new(20.0, 0.0),
                wall_type: "exterior".to_owned(),
                color: None,
                line_thickness: 4.0,
                line_style: None,
                bow_angle: 0.0,
                openings: vec![],
            }],
            polygons: vec![ExportPolygon {
                points: vec![
                    Pt::new(0.0, 0.0),
                    Pt::new(20.0, 0.0),
                    Pt::new(20.0, 10.0),
                    Pt::new(0.0, 10.0),
                ],
                edges: vec![Edge::default(); 4],
                area_type: "LIV".to_owned(),
                name: "Living Area".to_owned(),
                net: Some(200.0),
                area: 200.0,
                gross: 200.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        let dxf = export_dxf(&page, &ExportOptions::default(), &[]);
        // Our export has no $INSUNITS, so the parser infers the unit: a
        // 20-unit wall is feet (20 ft is a plausible building dimension).
        let parsed = parse_dxf(&dxf, None, (0.0, 0.0)).expect("parses");
        assert_eq!(parsed.entity_count, 4); // line + polygon + 2 area labels
        assert_eq!(parsed.lines.len(), 1);
        let (_a, b) = parsed.lines[0];
        assert!((b.x - 20.0).abs() < 1e-3 && (b.y - 0.0).abs() < 1e-3);
        assert_eq!(parsed.polygons.len(), 1);
        let ring = &parsed.polygons[0];
        assert_eq!(ring.len(), 4);
        assert!((ring[2].x - 20.0).abs() < 1e-3 && (ring[2].y - 10.0).abs() < 1e-3);
        assert!(parsed.layers.contains(&"WALLS-EXTERIOR".to_owned()));
        assert!(parsed.layers.contains(&"AREA-LIV".to_owned()));
        assert!(parsed.layers.contains(&"AREA-TEXT".to_owned()));
    }

    #[test]
    fn insunits_scale_factors() {
        let extent = Some((0.0, 0.0, 240.0, 192.0));
        assert_eq!(pick_scale(Some(1), extent).scale, 1.0 / 12.0);
        assert_eq!(pick_scale(Some(2), extent).scale, 1.0);
        assert_eq!(pick_scale(Some(3), extent).scale, 5280.0);
        assert_eq!(pick_scale(Some(4), extent).scale, 1.0 / 304.8);
        assert_eq!(pick_scale(Some(5), extent).scale, 1.0 / 30.48);
        assert_eq!(pick_scale(Some(6), extent).scale, 1.0 / 0.3048);
        assert_eq!(
            pick_scale(Some(0), extent),
            UnitChoice {
                scale: 1.0 / 12.0,
                units: "inches",
                source: "inferred"
            }
        );
    }

    #[test]
    fn unitless_inference_by_extent() {
        // 240 units: inches (240 in = 20 ft, plausible).
        let extent = Some((0.0, 0.0, 240.0, 192.0));
        let choice = pick_scale(None, extent);
        assert_eq!(choice.units, "inches");
        assert_eq!(choice.source, "inferred");
        // 12000 units with mm plausibility: mm (a 40 ft building).
        let extent = Some((0.0, 0.0, 12000.0, 9000.0));
        let choice = pick_scale(None, extent);
        assert_eq!(choice.units, "mm");
    }

    #[test]
    fn inch_dxf_converts_to_feet() {
        let parsed = parse_dxf(RECT_DXF, None, (0.0, 0.0)).expect("parses");
        // $INSUNITS 1 = inches -> 240 in = 20 ft.
        assert_eq!(parsed.units, "inches");
        assert_eq!(parsed.lines.len(), 1);
        let (_a, b) = parsed.lines[0];
        assert!((b.x - 20.0).abs() < 1e-6);
        let ring = &parsed.polygons[0];
        assert!((ring[2].y - 16.0).abs() < 1e-6);
    }

    #[test]
    fn closed_polyline_flag_makes_polygon() {
        let parsed = parse_dxf(RECT_DXF, None, (0.0, 0.0)).expect("parses");
        assert_eq!(parsed.polygons.len(), 1);
        assert_eq!(
            parsed.polygons[0].len(),
            4,
            "rectangle keeps its 4 vertices"
        );
    }

    #[test]
    fn legacy_polyline_vertex_state_machine() {
        // Legacy POLYLINE with separate VERTEX entities + SEQEND.
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n8\nW\n70\n0\n0\nVERTEX\n8\nW\n10\n0.0\n20\n0.0\n0\nVERTEX\n8\nW\n10\n12.0\n20\n0.0\n0\nVERTEX\n8\nW\n10\n12.0\n20\n9.0\n0\nVERTEX\n8\nW\n10\n0.0\n20\n9.0\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
        let pairs = parse_dxf_pairs(dxf);
        let ranges = section_ranges(&pairs);
        let entities = read_entities(&pairs, &ranges);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].vertices.len(), 4);
        assert!((entities[0].vertices[2].x - 12.0).abs() < 1e-9);
    }

    #[test]
    fn bulge_flattening_arc_points() {
        // bulge = tan(sweep/4): bulge 1.0 = 90° sweep.
        let pts = bulge_to_arc_points(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), 1.0);
        assert!(!pts.is_empty());
        // Positive bulge sweeps CCW from v0 to v1, bulging below the chord.
        assert!(pts.iter().all(|p| p.y < 0.0));
        // Zero bulge: no intermediate points.
        assert!(bulge_to_arc_points(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), 0.0).is_empty());
    }

    #[test]
    fn arc_unwraps_counter_clockwise() {
        // 350° -> 10° sweeps 20°, not -340°.
        let pts = arc_points(0.0, 0.0, 5.0, 350.0, 10.0);
        assert_eq!(
            pts.len(),
            7,
            "20° sweep at 5°/step: round(4)+1 -> max(6,5)=6 steps +1"
        );
        // CCW unwrap: the arc runs from 350° (below the axis) to 10° (above).
        assert!(pts[0].y < 0.0);
        assert!(pts.last().expect("points").y > 0.0);
    }

    #[test]
    fn circle_becomes_closed_polygon() {
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nCIRCLE\n8\nP\n10\n0.0\n20\n0.0\n40\n5.0\n0\nENDSEC\n0\nEOF\n";
        let parsed = parse_dxf(dxf, None, (0.0, 0.0)).expect("parses");
        assert_eq!(parsed.polygons.len(), 1);
        assert_eq!(parsed.polygons[0].len(), 97);
        let points = &parsed.polygons[0];
        assert!((points[0].x - points[96].x).hypot(points[0].y - points[96].y) < 1e-9);
    }

    #[test]
    fn text_and_mtext_labels_stripped() {
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nTEXT\n8\nT\n10\n1.0\n20\n2.0\n1\nFront Door\n0\nMTEXT\n8\nT\n10\n3.0\n20\n4.0\n1\nArea: \\P100 sf;\n0\nENDSEC\n0\nEOF\n";
        let parsed = parse_dxf(dxf, None, (0.0, 0.0)).expect("parses");
        assert_eq!(parsed.labels.len(), 2);
        assert_eq!(parsed.labels[0].1, "Front Door");
        // MTEXT inline codes (including the code's argument text) stripped
        // by the JS regex `\\[A-Za-z0-9.| ;]+;?`.
        assert_eq!(parsed.labels[1].1, "Area:");
    }

    #[test]
    fn empty_entities_rejected() {
        assert!(parse_dxf(
            "0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n",
            None,
            (0.0, 0.0)
        )
        .is_err());
    }

    #[test]
    fn unsupported_entities_counted_in_skipped() {
        // INSERT is read but has no sketch conversion, so it lands in the
        // skipped counts (like the JS app).
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nINSERT\n8\nD\n2\nBLK\n0\nENDSEC\n0\nEOF\n";
        let parsed = parse_dxf(dxf, None, (0.0, 0.0)).expect("parses");
        assert_eq!(parsed.skipped.get("insert"), Some(&1));
    }

    #[test]
    fn scale_override_beats_detection() {
        let parsed = parse_dxf(RECT_DXF, Some(1.0), (0.0, 0.0)).expect("parses");
        assert_eq!(parsed.unit_source, "override");
        assert_eq!(parsed.units, "custom");
        assert!((parsed.lines[0].1.x - 240.0).abs() < 1e-6);
    }
}
