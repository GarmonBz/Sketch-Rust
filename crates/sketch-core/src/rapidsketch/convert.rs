//! Mapping between a RapidSketch [`Document`] and Sketch Rust's save model.
//!
//! Sketch Rust draws in feet with y up; the file uses [`UNITS_PER_FOOT`] with
//! y down. An arc is always carried across as its three points (start, apex,
//! end), so neither side's sign convention can leak into the other.

use super::*;
use crate::geometry::{area_codes, AreaCode, Edge, GeometryEngine};
use crate::model::{Polygon, SketchLine, WallType};
use crate::symbols::{
    world_corners, Bounds as SymbolBounds, Entity, EntityType, Point as SymbolPoint, Symbol as Def,
    Vertex, WallSnap,
};
use serde_json::{json, Map, Value};

/// A label's size in Sketch Rust units per typographic point: points to
/// 96 dpi pixels, to feet at the file's scale, to twelfths of a foot.
const TEXT_UNITS_PER_POINT: f64 = 96.0 / 72.0 / UNITS_PER_FOOT * 12.0;
/// The category imported symbols are filed under.
const SYMBOL_CATEGORY: &str = "rapidsketch";

fn to_world(p: Pt) -> Pt {
    Pt::new(p.x / UNITS_PER_FOOT, -p.y / UNITS_PER_FOOT)
}

fn to_file(p: Pt) -> Pt {
    Pt::new(p.x * UNITS_PER_FOOT, -p.y * UNITS_PER_FOOT)
}

/// How far `apex` stands to the left of travel from `start` to `end`, with y
/// up, and the chord's length.
fn rise(start: Pt, apex: Pt, end: Pt) -> (f64, f64) {
    let (dx, dy) = (end.x - start.x, end.y - start.y);
    let chord = dx.hypot(dy);
    if chord < 1e-9 {
        return (0.0, chord);
    }
    let ox = apex.x - (start.x + end.x) / 2.0;
    let oy = apex.y - (start.y + end.y) / 2.0;
    ((oy * dx - ox * dy) / chord, chord)
}

/// The apex standing `left` of the middle of a chord, with y up.
fn apex_left(start: Pt, end: Pt, left: f64) -> Pt {
    let (dx, dy) = (end.x - start.x, end.y - start.y);
    let chord = dx.hypot(dy);
    let mid = Pt::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
    if chord < 1e-9 {
        return mid;
    }
    Pt::new(mid.x - left * dy / chord, mid.y + left * dx / chord)
}

/// Sketch Rust's bow angle (degrees, positive to the left) for a segment.
fn bow_of(s: &Segment) -> f64 {
    if s.arc == 0.0 {
        return 0.0;
    }
    let (left, chord) = rise(to_world(s.start), to_world(s.apex()), to_world(s.end));
    if chord < 1e-9 {
        return 0.0;
    }
    GeometryEngine::new().clamp_bow_angle((4.0 * (2.0 * left / chord).atan()).to_degrees())
}

/// A wall of the model as a segment of the file.
fn file_segment(start: Pt, end: Pt, bow: f64) -> Segment {
    let chord = (end.x - start.x).hypot(end.y - start.y);
    let left = chord / 2.0 * (bow.to_radians() / 4.0).tan();
    Segment::through(
        to_file(start),
        to_file(apex_left(start, end, left)),
        to_file(end),
    )
}

/// What the app needs after an import.
#[derive(Debug, Clone, PartialEq)]
pub struct RapidImport {
    /// A save-model JSON value (same shape as a `.skx` model).
    pub model: Value,
    /// Notes to show the user.
    pub notes: Vec<String>,
    pub area_count: usize,
}

/// The area code for a RapidSketch area. The file has only a name and a
/// "living area" switch, so the switch decides what counts as living area and
/// the name picks the code within that.
fn code_for(name: &str, living: bool) -> &'static str {
    let name = name.to_ascii_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| name.contains(w));
    if living {
        return if has(&["second", "2nd", "upper"]) {
            "GLA2"
        } else if has(&["third", "3rd"]) {
            "GLA3"
        } else if has(&["fourth", "4th"]) {
            "GLA4"
        } else {
            "GLA1"
        };
    }
    if has(&["basement", "bsmt"]) {
        if has(&["unfinished", "unfin"]) {
            "BSMT2"
        } else {
            "BSMT"
        }
    } else if has(&["carport", "car port"]) {
        "CPT"
    } else if has(&["garage"]) || counts_cars(&name) {
        "GAR"
    } else if has(&["patio"]) {
        "PAT"
    } else if has(&["deck"]) {
        "DK"
    } else if has(&["screen"]) {
        "SFP"
    } else if has(&["enclosed"]) {
        "EFP"
    } else if has(&["porch"]) {
        "PORCH"
    } else if has(&["balcony"]) {
        "BAL"
    } else if has(&["pool"]) {
        "POOL"
    } else if has(&["shed", "storage"]) {
        "SHED"
    } else {
        "OTH"
    }
}

/// A garage named by its size alone, such as "1 Car Attached".
fn counts_cars(name: &str) -> bool {
    let words: Vec<&str> = name.split(|c: char| !c.is_ascii_alphanumeric()).collect();
    let has = |any: &[&str]| words.iter().any(|word| any.contains(word));
    has(&["car", "cars"]) && has(&["attached", "detached", "built"])
}

fn hex_color(argb: u32) -> String {
    format!("#{:06X}", argb & 0x00FF_FFFF)
}

fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

/// A symbol definition, in inches with y up, for one placed symbol's drawing.
fn definition(symbol: &Symbol) -> Def {
    let inches = |p: Pt| {
        let w = to_world(p);
        Pt::new(
            (w.x * 12.0 * 1000.0).round() / 1000.0,
            (w.y * 12.0 * 1000.0).round() / 1000.0,
        )
    };
    let mut bounds = Bounds::EMPTY;
    let mut geometry = Vec::new();
    for shape in &symbol.shapes {
        let mut vertices = Vec::new();
        for (i, s) in shape.segments.iter().enumerate() {
            let (start, apex, end) = (inches(s.start), inches(s.apex()), inches(s.end));
            for p in [start, apex, end] {
                bounds.add(p, 0.0);
            }
            let (left, chord) = rise(start, apex, end);
            // A bulge is positive when the arc stands to the right.
            let bulge = if s.arc == 0.0 || chord < 1e-9 {
                None
            } else {
                Some((-2.0 * left / chord * 1.0e6).round() / 1.0e6)
            };
            vertices.push(Vertex {
                x: start.x,
                y: start.y,
                b: bulge,
            });
            if i + 1 == shape.segments.len() && !shape.closed {
                vertices.push(Vertex {
                    x: end.x,
                    y: end.y,
                    b: None,
                });
            }
        }
        geometry.push(Entity {
            entity_type: EntityType::Polyline,
            vertices: Some(vertices),
            closed: Some(shape.closed),
            ..Entity::default()
        });
    }
    let name = if symbol.name.trim().is_empty() {
        "Symbol".to_string()
    } else {
        symbol.name.trim().to_string()
    };
    // The same drawing at the same size is the same definition.
    let digest = crc32fast::hash(&serde_json::to_vec(&geometry).unwrap_or_default());
    Def {
        id: format!(
            "{SYMBOL_CATEGORY}/{}-{}-{digest:08x}",
            slug(&symbol.category),
            slug(&name)
        ),
        category: SYMBOL_CATEGORY.into(),
        category_name: "RapidSketch imports".into(),
        name: name.clone(),
        label: name.clone(),
        description: format!("{} {name}", symbol.category).trim().to_string(),
        units: "INCHES".into(),
        wall_snap: WallSnap::None,
        bounds: SymbolBounds {
            min_x: bounds.min.x,
            min_y: bounds.min.y,
            max_x: bounds.max.x,
            max_y: bounds.max.y,
        },
        geometry,
        ..Def::default()
    }
}

fn subject_json(info: &Information) -> Value {
    let mut out = Map::new();
    for (key, value) in [
        ("fileNo", &info.file_number),
        ("propertyAddress", &info.address),
        ("city", &info.city),
        ("state", &info.state),
        ("zipCode", &info.zip),
        ("owner", &info.borrower),
        ("county", &info.county),
        ("client", &info.client),
        ("clientAddress", &info.client_address),
        ("appraiserName", &info.appraiser_name),
        ("rapidSketchCaseNumber", &info.case_number),
        ("rapidSketchAppraiserAddress", &info.appraiser_address),
    ] {
        if !value.is_empty() {
            out.insert(key.into(), Value::String(value.clone()));
        }
    }
    Value::Object(out)
}

fn information(subject: &Value) -> Information {
    let text = |key: &str| {
        subject
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    Information {
        case_number: text("rapidSketchCaseNumber"),
        file_number: text("fileNo"),
        address: text("propertyAddress"),
        city: text("city"),
        state: text("state"),
        zip: text("zipCode"),
        borrower: text("owner"),
        county: text("county"),
        client: text("client"),
        client_address: text("clientAddress"),
        appraiser_name: text("appraiserName"),
        appraiser_address: text("rapidSketchAppraiserAddress"),
    }
}

impl Document {
    /// Build the save model: one page, a wall per drawn line, a closed area per
    /// closed outline, the notes, and the symbols with their own drawings.
    pub fn to_model(&self, file_name: &str) -> RapidImport {
        let engine = GeometryEngine::new();
        let codes: Vec<AreaCode> = area_codes::default_area_codes();
        let mut next_id: i64 = 1;
        let mut take_id = || {
            next_id += 1;
            next_id - 1
        };
        let mut lines: Vec<SketchLine> = Vec::new();
        let mut polygons: Vec<Polygon> = Vec::new();
        let mut notes = self.warnings.clone();
        let mut unclosed = Vec::new();

        for area in &self.areas {
            let segments: Vec<Segment> = area.walls.iter().map(|w| w.segment).collect();
            let mut outlines = 0;
            for chain in chains(&segments) {
                let run: Vec<(Segment, &Wall)> = chain
                    .links
                    .iter()
                    .map(|&(i, reversed)| {
                        let s = if reversed {
                            segments[i].reversed()
                        } else {
                            segments[i]
                        };
                        (s, &area.walls[i])
                    })
                    .collect();
                // The outline's corners; each wall ends where the next starts.
                let mut points: Vec<Pt> = run.iter().map(|(s, _)| to_world(s.start)).collect();
                if !chain.closed {
                    if let Some((last, _)) = run.last() {
                        points.push(to_world(last.end));
                    }
                }
                let mut edges = Vec::with_capacity(run.len());
                for (i, (s, wall)) in run.iter().enumerate() {
                    let bow = bow_of(s);
                    let id = take_id();
                    lines.push(SketchLine {
                        id,
                        start: points[i],
                        end: points[(i + 1) % points.len()],
                        kind: WallType::Exterior,
                        color: None,
                        line_thickness: None,
                        thickness: None,
                        dimension_hidden: !wall.dimension.visible,
                        layer_id: None,
                        bow_angle: bow,
                        arc_height: 0.0,
                        line_style: match wall.dash {
                            0 => None,
                            1 => Some("dashed".into()),
                            _ => Some("dotted".into()),
                        },
                        dimension_position: None,
                        dimension_anchor: None,
                        extra: Default::default(),
                    });
                    edges.push(Edge {
                        line_id: Some(id),
                        bow_angle: bow,
                    });
                }
                if !chain.closed {
                    continue;
                }
                outlines += 1;
                let code = code_for(&area.name, area.living);
                let factor = if area.multiplier != 0.0 {
                    area.multiplier
                } else {
                    1.0
                };
                let gross = engine.calculate_area(&points, &edges);
                let name = if area.name.trim().is_empty() {
                    "Area"
                } else {
                    area.name.trim()
                };
                let mut polygon: Polygon = serde_json::from_value(json!({
                    "id": take_id(),
                    "points": points,
                    "edges": edges,
                    "type": code,
                    "name": name,
                    "gross": gross,
                    "area": gross * factor,
                    "factor": factor,
                    // The file's areas are drawn side by side, never netted.
                    "subtractMode": "never",
                }))
                .expect("polygon defaults deserialize");
                if factor != 1.0 {
                    polygon.factor_override = Some(factor);
                }
                polygon.auto_post.name = area.label.visible;
                polygon.auto_post.calcs = area.size_label.visible;
                if area.label.visible {
                    polygon.label_position = Some(to_world(area.label.at));
                    polygon.label_rotation =
                        (area.label.rotation != 0.0).then_some(area.label.rotation);
                }
                if area.size_label.visible {
                    polygon.area_size_position = Some(to_world(area.size_label.at));
                    polygon.area_size_rotation = Some(area.size_label.rotation);
                }
                let alpha = area.fill >> 24;
                // A white fill is the program's "no colour".
                if alpha > 0 && area.fill & 0x00FF_FFFF != 0x00FF_FFFF {
                    polygon.fill_color = Some(hex_color(area.fill));
                    polygon.fill_opacity = Some((f64::from(alpha) / 255.0 * 100.0).round());
                }
                polygons.push(polygon);
            }
            if outlines == 0 && !area.walls.is_empty() && area.label.visible {
                unclosed.push(area.name.clone());
            }
        }
        for wall in &self.interior_walls {
            lines.push(SketchLine {
                id: take_id(),
                start: to_world(wall.start),
                end: to_world(wall.end),
                kind: WallType::Interior,
                color: None,
                line_thickness: None,
                thickness: None,
                // The program draws no dimension on these.
                dimension_hidden: true,
                layer_id: None,
                bow_angle: bow_of(wall),
                arc_height: 0.0,
                line_style: None,
                dimension_position: None,
                dimension_anchor: None,
                extra: Default::default(),
            });
        }
        if !unclosed.is_empty() {
            notes.push(format!(
                "Not closed, so imported as walls only: {}.",
                unclosed.join(", ")
            ));
        }

        let labels: Vec<Value> = self
            .notes
            .iter()
            .map(|note| {
                let at = to_world(note.at);
                json!({
                    "id": take_id(),
                    "text": note.text,
                    "x": at.x,
                    "y": at.y,
                    "rotation": note.rotation,
                    "fontFamily": note.font.name,
                    "fontSize": (note.font.size * TEXT_UNITS_PER_POINT * 10.0).round() / 10.0,
                    "bold": note.font.style & 1 != 0,
                    "italic": note.font.style & 2 != 0,
                    "underline": note.font.style & 4 != 0,
                    "color": hex_color(note.color),
                })
            })
            .collect();

        let mut definitions = std::collections::BTreeMap::new();
        let symbols: Vec<Value> = self
            .symbols
            .iter()
            .map(|symbol| {
                let def = definition(symbol);
                let at = to_world(symbol.center);
                // Sketch Rust mirrors across one axis only: a vertical flip is
                // a horizontal one turned half way round.
                let rotation = if symbol.flip_v {
                    (symbol.rotation + 180.0) % 360.0
                } else {
                    symbol.rotation
                };
                let placed = json!({
                    "id": take_id(),
                    "symbolType": def.id,
                    "x": at.x,
                    "y": at.y,
                    "rotation": rotation,
                    "scale": 1.0,
                    "flipH": symbol.flip_h != symbol.flip_v,
                    "wallSnap": "NONE",
                });
                definitions.insert(def.id.clone(), def);
                placed
            })
            .collect();

        let area_count = polygons.len();
        let mut model = json!({
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
                "symbols": symbols,
                "dimensionChains": [],
                "northAngle": 0.0,
            }],
            "subjectInfo": subject_json(&self.information),
        });
        if !definitions.is_empty() {
            model["settings"] = json!({ "customSymbols": definitions });
        }
        RapidImport {
            model,
            notes,
            area_count,
        }
    }
}

/// The library symbol that stands in for the programs' corner fireplace.
pub const CORNER_FIREPLACE_ID: &str = "custom/fireplace-corner-36";
/// Where that drawing's wall corner is, in its own inches. Its walls run from
/// there toward +x and -y.
pub const CORNER_FIREPLACE_WALL_CORNER: (f64, f64) = (-28.2144, 28.2144);

/// Where an imported corner fireplace's wall corner belongs, in the drawing's
/// own inches, and how far clockwise to turn the library one to face the same
/// way. The file's size is far out of square, and its longer side would reach
/// through a wall, so the corner is that of a square as wide as the shorter
/// side, about the same centre.
fn fireplace_corner(def: &Def) -> Option<(SymbolPoint, f64)> {
    let imported = def.category == SYMBOL_CATEGORY
        && def.name.eq_ignore_ascii_case("corner")
        && def.description.eq_ignore_ascii_case("fireplaces corner");
    if !imported {
        return None;
    }
    // The first closed three-point outline is the unit itself.
    let outline = def.geometry.iter().find_map(|e| {
        let v = e.vertices.as_deref()?;
        (e.closed == Some(true) && v.len() == 3).then_some(v)
    })?;
    (0..3).find_map(|i| {
        let (at, a, b) = (outline[i], outline[(i + 1) % 3], outline[(i + 2) % 3]);
        let level = |p: Vertex| (p.y - at.y).abs() < 0.01;
        let upright = |p: Vertex| (p.x - at.x).abs() < 0.01;
        let (across, along) = if level(a) && upright(b) {
            (a.x - at.x, b.y - at.y)
        } else if level(b) && upright(a) {
            (b.x - at.x, a.y - at.y)
        } else {
            return None;
        };
        // The library drawing's walls run right and down from its corner.
        let turn = match (across > 0.0, along > 0.0) {
            (true, false) => 0.0,
            (false, false) => 90.0,
            (false, true) => 180.0,
            (true, true) => 270.0,
        };
        let half = across.abs().min(along.abs()) / 2.0;
        let corner = SymbolPoint {
            x: at.x + across / 2.0 - half.copysign(across),
            y: at.y + along / 2.0 - half.copysign(along),
        };
        Some((corner, turn))
    })
}

/// Swap the corner fireplace that comes in with a RapidSketch or ACI Sketch
/// file for the library's own, in a save model. The files store it stretched
/// out of square. Each one goes in its wall corner, facing the same way.
/// Returns how many placed symbols changed.
pub fn replace_imported_fireplaces(model: &mut Value) -> usize {
    let Some(definitions) = model
        .pointer("/settings/customSymbols")
        .and_then(Value::as_object)
    else {
        return 0;
    };
    let old: Vec<(String, SymbolPoint, f64)> = definitions
        .iter()
        .filter_map(|(id, value)| {
            let def: Def = serde_json::from_value(value.clone()).ok()?;
            fireplace_corner(&def).map(|(corner, turn)| (id.clone(), corner, turn))
        })
        .collect();
    if old.is_empty() {
        return 0;
    }
    // Where a point of a placed drawing lands on the page.
    let on_page = |p: SymbolPoint, x: f64, y: f64, rotation: f64, scale: f64, flip: bool| {
        let at = SymbolBounds {
            min_x: p.x,
            min_y: p.y,
            max_x: p.x,
            max_y: p.y,
        };
        world_corners(at, x, y, rotation, scale, flip)[0]
    };
    let mut changed = 0;
    let pages = model.get_mut("pages").and_then(Value::as_array_mut);
    for page in pages.into_iter().flatten() {
        let symbols = page.get_mut("symbols").and_then(Value::as_array_mut);
        for symbol in symbols.into_iter().flatten() {
            let Ok(placed) = serde_json::from_value::<Placed>(symbol.clone()) else {
                continue;
            };
            let Some(&(_, corner, turn)) = old.iter().find(|(id, ..)| *id == placed.symbol_type)
            else {
                continue;
            };
            let scale = placed.scale.filter(|s| *s > 0.0).unwrap_or(1.0);
            let (rotation, flip) = (placed.rotation, placed.flip_h);
            let wall_corner = on_page(corner, placed.x, placed.y, rotation, scale, flip);
            // A mirrored symbol turns the other way.
            let rotation = if flip {
                rotation - turn
            } else {
                rotation + turn
            }
            .rem_euclid(360.0);
            let (x, y) = CORNER_FIREPLACE_WALL_CORNER;
            let offset = on_page(SymbolPoint { x, y }, 0.0, 0.0, rotation, scale, flip);
            symbol["symbolType"] = json!(CORNER_FIREPLACE_ID);
            symbol["x"] = json!(wall_corner.x - offset.x);
            symbol["y"] = json!(wall_corner.y - offset.y);
            symbol["rotation"] = json!(rotation);
            changed += 1;
        }
    }
    if let Some(definitions) = model
        .pointer_mut("/settings/customSymbols")
        .and_then(Value::as_object_mut)
    {
        for (id, ..) in &old {
            definitions.remove(id);
        }
    }
    changed
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

/// `#RRGGBB` or `rgba(r, g, b, a)` as (rgb, alpha 0..1).
fn parse_css(text: &str) -> Option<(u32, f64)> {
    let text = text.trim();
    if let Some(hex) = text.strip_prefix('#') {
        return (hex.len() == 6)
            .then(|| u32::from_str_radix(hex, 16).ok())
            .flatten()
            .map(|rgb| (rgb, 1.0));
    }
    let inner = text.strip_prefix("rgba(")?.strip_suffix(')')?;
    let parts: Vec<f64> = inner
        .split(',')
        .filter_map(|p| p.trim().parse().ok())
        .collect();
    let [r, g, b, a] = parts[..] else { return None };
    let byte = |v: f64| v.clamp(0.0, 255.0) as u32;
    Some((byte(r) << 16 | byte(g) << 8 | byte(b), a))
}

fn fill_of(polygon: &Polygon, code: Option<&AreaCode>) -> u32 {
    let own = polygon.fill_color.as_deref().and_then(parse_css);
    let parsed =
        own.or_else(|| code.and_then(|c| parse_css(&c.fill_color).or_else(|| parse_css(&c.fill))));
    let Some((rgb, alpha)) = parsed else {
        return 0x7DFF_FFFF;
    };
    let alpha = polygon
        .fill_opacity
        .or_else(|| code.map(|c| c.fill_opacity))
        .map_or(alpha, |percent| percent / 100.0);
    ((alpha.clamp(0.0, 1.0) * 255.0).round() as u32) << 24 | rgb
}

/// Feet as the program writes a dimension: `10' 7⅝"`.
fn feet_and_inches(feet: f64) -> String {
    const EIGHTHS: [&str; 8] = ["", "⅛", "¼", "⅜", "½", "⅝", "¾", "⅞"];
    let total = (feet.abs() * 96.0).round() as u64;
    let (whole, rest) = (total / 96, total % 96);
    let (inches, eighths) = (rest / 8, EIGHTHS[(rest % 8) as usize]);
    if rest == 0 {
        return format!("{whole}'");
    }
    let inches = match (inches, eighths) {
        (0, fraction) => format!("{fraction}\""),
        (n, fraction) => format!("{n}{fraction}\""),
    };
    if whole == 0 {
        inches
    } else {
        format!("{whole}' {inches}")
    }
}

fn plain_text(
    text: String,
    at: Pt,
    rotation: f64,
    visible: bool,
    size: f64,
    bold: bool,
) -> TextBlock {
    TextBlock {
        text,
        at,
        rotation: rotation.rem_euclid(360.0),
        visible,
        font: Font {
            name: "Microsoft Sans Serif".into(),
            size,
            style: i64::from(bold),
        },
        color: 0xFF00_0000,
    }
}

/// Which side of its walls a run's dimensions go: `1.0` for the side
/// `(dy, -dx)` points to (outward for an outline whose shoelace sum is
/// positive in file coordinates), `-1.0` for the other.
fn outward_side(segments: &[Segment]) -> f64 {
    let twice: f64 = segments
        .iter()
        .flat_map(|s| [(s.start, s.apex()), (s.apex(), s.end)])
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum();
    if twice < 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// A wall with its dimension placed beside it, as the program places them.
fn dimensioned(segment: Segment, side: f64, line: Option<&SketchLine>, feet: f64) -> Wall {
    let (dx, dy) = (
        segment.end.x - segment.start.x,
        segment.end.y - segment.start.y,
    );
    let chord = dx.hypot(dy).max(1e-9);
    let (nx, ny) = (side * dy / chord, -side * dx / chord);
    // Short walls get smaller figures, closer in.
    let (size, gap) = if feet < 4.0 {
        (4.0, 6.0)
    } else if feet < 13.0 {
        (6.0, 8.0)
    } else {
        (8.0, 10.0)
    };
    let apex = segment.apex();
    let at = Pt::new(apex.x + nx * gap, apex.y + ny * gap);
    let rotation = nx.atan2(-ny).to_degrees();
    let visible = line.is_none_or(|l| !l.dimension_hidden);
    Wall {
        segment,
        dimension: plain_text(feet_and_inches(feet), at, rotation, visible, size, false),
        dash: match line.and_then(|l| l.line_style.as_deref()) {
            Some("dashed") => 1,
            Some("dotted") => 2,
            _ => 0,
        },
    }
}

/// The strokes of a symbol definition as (start, apex, end) triples in inches,
/// grouped into shapes, and how many text entities were left out.
type Stroke3 = [Pt; 3];

fn definition_strokes(def: &Def) -> (Vec<(Vec<Stroke3>, bool)>, usize) {
    let straight = |a: Pt, b: Pt| [a, apex_left(a, b, 0.0), b];
    let pt = |p: SymbolPoint| Pt::new(p.x, p.y);
    let mut shapes = Vec::new();
    let mut texts = 0;
    for e in &def.geometry {
        if e.stroke_alpha == Some(0) {
            continue;
        }
        match e.entity_type {
            EntityType::Line => {
                if let (Some(a), Some(b)) = (e.start_point, e.end_point) {
                    shapes.push((vec![straight(pt(a), pt(b))], false));
                }
            }
            EntityType::Polyline => {
                let Some(vs) = e.vertices.as_ref().filter(|v| v.len() >= 2) else {
                    continue;
                };
                let closed = e.closed.unwrap_or(false);
                let count = if closed { vs.len() } else { vs.len() - 1 };
                let strokes = (0..count)
                    .map(|i| {
                        let (a, b) = (vs[i], vs[(i + 1) % vs.len()]);
                        let (start, end) = (Pt::new(a.x, a.y), Pt::new(b.x, b.y));
                        let chord = (end.x - start.x).hypot(end.y - start.y);
                        let left = -a.b.unwrap_or(0.0) * chord / 2.0;
                        [start, apex_left(start, end, left), end]
                    })
                    .collect();
                shapes.push((strokes, closed));
            }
            EntityType::Arc | EntityType::Ellipse => {
                let (Some(c), Some(axis)) = (e.center, e.axis_point) else {
                    continue;
                };
                let (ax, ay) = (axis.x - c.x, axis.y - c.y);
                let rx = ax.hypot(ay);
                let ry = rx * e.ratio.filter(|r| *r != 0.0).unwrap_or(1.0).abs();
                let (sin, cos) = ay.atan2(ax).sin_cos();
                let at = |t: f64| {
                    let (x, y) = (rx * t.cos(), ry * t.sin());
                    Pt::new(c.x + x * cos - y * sin, c.y + x * sin + y * cos)
                };
                let is_arc = e.entity_type == EntityType::Arc;
                let (from, mut sweep) = if is_arc {
                    let from = e.start_angle.unwrap_or(0.0).to_radians();
                    let to = e.end_angle.unwrap_or(0.0).to_radians();
                    (from, to - from)
                } else {
                    (0.0, std::f64::consts::TAU)
                };
                if !sweep.is_finite() || !rx.is_finite() {
                    continue;
                }
                if sweep <= 0.0 {
                    sweep = sweep.rem_euclid(std::f64::consts::TAU);
                }
                let pieces = ((sweep / (std::f64::consts::PI / 8.0)).ceil() as usize).clamp(1, 64);
                let step = sweep / pieces as f64;
                let strokes = (0..pieces)
                    .map(|i| {
                        let t = from + step * i as f64;
                        [at(t), at(t + step / 2.0), at(t + step)]
                    })
                    .collect();
                shapes.push((strokes, !is_arc));
            }
            EntityType::Text => texts += 1,
        }
    }
    (shapes, texts)
}

/// One placed symbol of a page.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Placed {
    symbol_type: String,
    x: f64,
    y: f64,
    #[serde(default)]
    rotation: f64,
    #[serde(default)]
    scale: Option<f64>,
    #[serde(default)]
    flip_h: bool,
}

fn file_symbol(placed: &Placed, def: &Def) -> Option<(Symbol, usize)> {
    let (shapes, texts) = definition_strokes(def);
    let mut bounds = Bounds::EMPTY;
    for p in shapes.iter().flat_map(|(s, _)| s.iter().flatten()) {
        bounds.add(*p, 0.0);
    }
    if shapes.is_empty() {
        return None;
    }
    let middle = Pt::new(
        (bounds.min.x + bounds.max.x) / 2.0,
        (bounds.min.y + bounds.max.y) / 2.0,
    );
    let scale = placed.scale.filter(|s| *s > 0.0).unwrap_or(1.0);
    // Inches to file units.
    let units = scale * UNITS_PER_FOOT / 12.0;
    let map = |p: Pt| Pt::new((p.x - middle.x) * units, -(p.y - middle.y) * units);
    let [center, ..] = world_corners(
        SymbolBounds {
            min_x: middle.x,
            min_y: middle.y,
            max_x: middle.x,
            max_y: middle.y,
        },
        placed.x,
        placed.y,
        placed.rotation,
        scale,
        placed.flip_h,
    );
    let (category, name) = (def.category_name.clone(), def.name.clone());
    // An imported symbol goes back under the category it came from.
    let category = if def.category == SYMBOL_CATEGORY {
        def.description
            .strip_suffix(&name)
            .map_or(category, |c| c.trim().to_string())
    } else {
        category
    };
    Some((
        Symbol {
            category,
            name,
            center: to_file(center),
            // The size covers the pen as well as the strokes.
            width: (bounds.max.x - bounds.min.x) * units + SYMBOL_PEN,
            height: (bounds.max.y - bounds.min.y) * units + SYMBOL_PEN,
            rotation: placed.rotation.rem_euclid(360.0),
            flip_h: placed.flip_h,
            flip_v: false,
            shapes: shapes
                .into_iter()
                .map(|(strokes, closed)| Shape {
                    segments: strokes
                        .into_iter()
                        .map(|[a, apex, b]| Segment::through(map(a), map(apex), map(b)))
                        .collect(),
                    closed,
                })
                .collect(),
        },
        texts,
    ))
}

fn list<T: serde::de::DeserializeOwned>(page: &Value, key: &str) -> Result<Vec<T>, String> {
    match page.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(v) => serde_json::from_value(v.clone())
            .map_err(|e| format!("the page's {key} did not parse: {e}")),
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedLabel {
    text: String,
    x: f64,
    y: f64,
    #[serde(default)]
    rotation: f64,
    #[serde(default)]
    font_family: Option<String>,
    #[serde(default)]
    font_size: Option<f64>,
    #[serde(default)]
    bold: bool,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    underline: bool,
    #[serde(default)]
    color: Option<String>,
}

/// Build a file document from one saved page. `subject` is the model's
/// `subjectInfo`; `symbol` looks up a placed symbol's definition.
pub fn from_page(
    page: &Value,
    codes: &[AreaCode],
    subject: &Value,
    symbol: &dyn Fn(&str) -> Option<Def>,
) -> Result<Document, String> {
    let polygons: Vec<Polygon> = list(page, "polygons")?;
    let lines: Vec<SketchLine> = list(page, "lines")?;
    let engine = GeometryEngine::new();
    let mut doc = Document {
        information: information(subject),
        ..Document::default()
    };
    let mut used = std::collections::HashSet::new();

    for polygon in &polygons {
        let count = polygon.points.len();
        if count < 2 {
            continue;
        }
        let edge = |i: usize| polygon.edges.get(i).copied().unwrap_or_default();
        let edges: Vec<Edge> = (0..count).map(edge).collect();
        let segments: Vec<Segment> = (0..count)
            .map(|i| {
                file_segment(
                    polygon.points[i],
                    polygon.points[(i + 1) % count],
                    edges[i].bow_angle,
                )
            })
            .collect();
        let side = outward_side(&segments);
        let walls = segments
            .iter()
            .enumerate()
            .map(|(i, &segment)| {
                let line = edges[i]
                    .line_id
                    .and_then(|id| lines.iter().find(|l| l.id == id));
                used.extend(edges[i].line_id);
                let feet = engine.arc_length(
                    polygon.points[i],
                    polygon.points[(i + 1) % count],
                    engine.clamp_bow_angle(edges[i].bow_angle),
                );
                dimensioned(segment, side, line, feet)
            })
            .collect();

        let code = codes.iter().find(|c| c.code == polygon.kind);
        let name = [Some(polygon.name.as_str()), code.map(|c| c.name.as_str())]
            .into_iter()
            .flatten()
            .map(str::trim)
            .find(|n| !n.is_empty())
            .unwrap_or("Area")
            .to_string();
        let typed = !polygon.kind.is_empty() && polygon.kind != "UND";
        let anchor = polygon.label_position.unwrap_or_else(|| {
            engine.area_label_anchor(&engine.flatten_ring(&polygon.points, &edges, 0.05))
        });
        let label_at = to_file(anchor);
        let size_at = polygon
            .area_size_position
            .map_or(Pt::new(label_at.x, label_at.y + 15.0), to_file);
        let rotation = polygon.label_rotation.unwrap_or(0.0);
        let measured = engine.calculate_area(&polygon.points, &edges);
        let multiplier = if polygon.factor.is_finite() && polygon.factor != 0.0 {
            polygon.factor
        } else {
            1.0
        };
        doc.areas.push(Area {
            living: code.is_some_and(|c| c.include_in_gla),
            multiplier,
            fill: fill_of(polygon, code),
            walls,
            label: plain_text(
                name.clone(),
                label_at,
                rotation,
                typed && polygon.auto_post.name,
                10.0,
                true,
            ),
            size_label: plain_text(
                format!("[Area: {:.0} ft²]", measured * multiplier),
                size_at,
                polygon.area_size_rotation.unwrap_or(rotation),
                typed && polygon.auto_post.calcs,
                8.25,
                false,
            ),
            name,
        });
    }

    // Walls that bound no area: interior walls and unfinished outlines.
    let same_span = |l: &SketchLine| {
        polygons.iter().any(|p| {
            (0..p.points.len()).any(|i| {
                let (a, b) = (p.points[i], p.points[(i + 1) % p.points.len()]);
                (crate::model::same(a, l.start) && crate::model::same(b, l.end))
                    || (crate::model::same(a, l.end) && crate::model::same(b, l.start))
            })
        })
    };
    // A straight, solid interior wall is the file's own `FreeformLine`. That
    // can hold nothing more, so a curved or dashed one goes with the others.
    let freeform = |l: &SketchLine| {
        l.kind == WallType::Interior && l.bow_angle.abs() <= 0.5 && l.line_style.is_none()
    };
    doc.interior_walls = lines
        .iter()
        .filter(|l| !used.contains(&l.id) && freeform(l))
        .map(|l| file_segment(l.start, l.end, 0.0))
        .collect();
    let loose: Vec<(&SketchLine, Segment)> = lines
        .iter()
        .filter(|l| !used.contains(&l.id) && !freeform(l) && !same_span(l))
        .map(|l| (l, file_segment(l.start, l.end, l.bow_angle)))
        .collect();
    let segments: Vec<Segment> = loose.iter().map(|(_, s)| *s).collect();
    for chain in chains(&segments) {
        let walls = chain
            .links
            .iter()
            .map(|&(i, reversed)| {
                let (line, s) = loose[i];
                let segment = if reversed { s.reversed() } else { s };
                let feet =
                    engine.arc_length(line.start, line.end, engine.clamp_bow_angle(line.bow_angle));
                dimensioned(segment, 1.0, Some(line), feet)
            })
            .collect();
        doc.open_walls.push(walls);
    }

    for label in list::<SavedLabel>(page, "labels")? {
        if label.text.trim().is_empty() {
            continue;
        }
        let rgb = label
            .color
            .as_deref()
            .and_then(parse_css)
            // The dark canvas's automatic ink would vanish on white paper.
            .filter(|(rgb, _)| *rgb != 0x00E2_E8F0)
            .map_or(0, |(rgb, _)| rgb);
        let points = label.font_size.unwrap_or(12.0) / TEXT_UNITS_PER_POINT;
        doc.notes.push(TextBlock {
            text: label.text,
            at: to_file(Pt::new(label.x, label.y)),
            rotation: label.rotation.rem_euclid(360.0),
            visible: true,
            font: Font {
                name: label
                    .font_family
                    .filter(|f| !f.is_empty())
                    .unwrap_or_else(|| "Arial".into()),
                size: ((points * 4.0).round() / 4.0).clamp(1.0, 200.0),
                style: i64::from(label.bold)
                    | i64::from(label.italic) << 1
                    | i64::from(label.underline) << 2,
            },
            color: 0xFF00_0000 | rgb,
        });
    }

    let (mut missing, mut texts) = (0, 0);
    for placed in list::<Placed>(page, "symbols")? {
        match symbol(&placed.symbol_type).and_then(|def| file_symbol(&placed, &def)) {
            Some((s, skipped)) => {
                doc.symbols.push(s);
                texts += skipped;
            }
            None => missing += 1,
        }
    }
    if missing > 0 {
        doc.warnings.push(format!(
            "{missing} symbol{} had no drawing to export.",
            if missing == 1 { "" } else { "s" }
        ));
    }
    if texts > 0 {
        doc.warnings
            .push("Text inside symbols was not exported.".into());
    }
    if doc.areas.is_empty() && doc.open_walls.is_empty() && doc.interior_walls.is_empty() {
        return Err("The page has no walls or areas to export.".into());
    }
    Ok(doc)
}
