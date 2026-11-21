//! Conversions between sketch geometry and [`sketch_core::symbols::Entity`]
//! definitions for "Make symbol from selection" (§6.1) and "Edit shape" (§6.2).
//!
//! Coordinate conventions (`render.rs:163` & `symlib.rs:334`):
//! - Sketch world coordinates are in **feet** (+X right, +Y up).
//! - Symbol coordinates are in **inches** (+X right, +Y up).
//! - Sketch `bow_angle` is in degrees and sweeps **clockwise** when positive
//!   (`render.rs:316`), whereas DXF / symbol `Arc` sweeps counter-clockwise and
//!   polyline `bulge` is counter-clockwise: `sweep_ccw = -bow_angle`.
//! - Sketch `PlacedText::rotation` is clockwise on screen; symbol `Entity::rotation`
//!   is counter-clockwise (`symlib.rs:435`): `entity.rotation = -placed.rotation`.

use std::collections::{BTreeMap, BTreeSet};

use sketch_core::geometry::{distance, Pt, MAX_BOW_ANGLE};
use sketch_core::model::{SketchLine, WallType};
use sketch_core::symbols::{Bounds, Entity, EntityType, Point, Symbol, Vertex};

use crate::drawing::{default_line_style, Drawing};
use crate::placed_text::PlacedText;
use crate::selection::Selection;

fn quant_pt(p: Pt) -> (i64, i64) {
    (
        (p.x * 10_000.0).round() as i64,
        (p.y * 10_000.0).round() as i64,
    )
}

fn to_inches(p: Pt, origin: Pt) -> Point {
    Point {
        x: (p.x - origin.x) * 12.0,
        y: (p.y - origin.y) * 12.0,
    }
}

fn to_feet(p: Point) -> Pt {
    Pt::new(p.x / 12.0, p.y / 12.0)
}

fn curve_circle_and_apex(start: Pt, end: Pt, bow_angle: f64) -> Option<(Pt, f64, Pt)> {
    let chord = distance(start, end);
    let bow = bow_angle.clamp(-MAX_BOW_ANGLE, MAX_BOW_ANGLE);
    let theta = bow.abs().to_radians();
    if chord < 1e-4 || theta < 1e-4 {
        return None;
    }
    let radius = (chord / 2.0) / (theta / 2.0).sin();
    let rise = chord / 2.0 * (theta / 4.0).tan();
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let nx = -dy / chord;
    let ny = dx / chord;
    let sign = if bow < 0.0 { -1.0 } else { 1.0 };
    let mx = (start.x + end.x) / 2.0;
    let my = (start.y + end.y) / 2.0;
    let center_dist = radius * (theta / 2.0).cos();
    Some((
        Pt::new(mx - nx * center_dist * sign, my - ny * center_dist * sign),
        radius,
        Pt::new(mx + nx * rise * sign, my + ny * rise * sign),
    ))
}

fn bow_to_bulge(bow_angle: f64) -> Option<f64> {
    if bow_angle.abs() <= 0.25 {
        return None;
    }
    let bow = bow_angle.clamp(-MAX_BOW_ANGLE, MAX_BOW_ANGLE);
    let bulge = -(bow.to_radians() / 4.0).tan();
    (bulge.is_finite() && bulge.abs() > 1e-6).then_some(bulge)
}

fn bulge_to_bow(bulge: f64) -> f64 {
    if !bulge.is_finite() || bulge.abs() < 1e-6 {
        return 0.0;
    }
    (-4.0 * bulge.atan())
        .to_degrees()
        .clamp(-MAX_BOW_ANGLE, MAX_BOW_ANGLE)
}

fn line_to_entity(line: &SketchLine, origin: Pt) -> Entity {
    if let Some((center, radius, apex)) =
        curve_circle_and_apex(line.start, line.end, line.bow_angle)
    {
        let c_in = to_inches(center, origin);
        let r_in = radius * 12.0;
        let a_s = (line.start.y - center.y).atan2(line.start.x - center.x);
        let a_e = (line.end.y - center.y).atan2(line.end.x - center.x);
        let a_m = (apex.y - center.y).atan2(apex.x - center.x);
        let tau = std::f64::consts::TAU;
        let ccw = (a_m - a_s).rem_euclid(tau) <= (a_e - a_s).rem_euclid(tau);
        let (s_rad, e_rad) = if ccw { (a_s, a_e) } else { (a_e, a_s) };
        let to_deg = |r: f64| r.to_degrees().rem_euclid(360.0);
        return Entity {
            entity_type: EntityType::Arc,
            center: Some(c_in),
            axis_point: Some(Point {
                x: c_in.x + r_in,
                y: c_in.y,
            }),
            ratio: Some(1.0),
            start_angle: Some(to_deg(s_rad)),
            end_angle: Some(to_deg(e_rad)),
            ..Default::default()
        };
    }
    Entity {
        entity_type: EntityType::Line,
        start_point: Some(to_inches(line.start, origin)),
        end_point: Some(to_inches(line.end, origin)),
        ..Default::default()
    }
}

fn lines_and_texts_to_entities(
    lines: &[&SketchLine],
    texts: &[&PlacedText],
    origin: Pt,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    // Detect closed degree-2 chains so drawn closed shapes (rectangles, etc.)
    // become closed POLYLINE entities per spec §6.1.
    let mut adj: BTreeMap<(i64, i64), Vec<usize>> = BTreeMap::new();
    for (idx, l) in lines.iter().enumerate() {
        let a = quant_pt(l.start);
        let b = quant_pt(l.end);
        if a == b {
            continue;
        }
        adj.entry(a).or_default().push(idx);
        adj.entry(b).or_default().push(idx);
    }

    let mut used = vec![false; lines.len()];
    for start_idx in 0..lines.len() {
        if used[start_idx] {
            continue;
        }
        let l0 = lines[start_idx];
        let k_start = quant_pt(l0.start);
        let k_next = quant_pt(l0.end);
        if k_start == k_next {
            continue;
        }
        if adj.get(&k_start).is_none_or(|v| v.len() != 2)
            || adj.get(&k_next).is_none_or(|v| v.len() != 2)
        {
            continue;
        }

        let mut chain_indices = vec![start_idx];
        let mut chain_pts: Vec<Pt> = vec![l0.start];
        let mut chain_bows: Vec<f64> = vec![l0.bow_angle];
        let mut cur_pt = l0.end;
        let mut cur_key = k_next;
        let mut prev_edge = start_idx;
        let mut closed = false;

        loop {
            if cur_key == k_start {
                closed = chain_indices.len() >= 3;
                break;
            }
            let Some(neighbors) = adj.get(&cur_key) else {
                break;
            };
            if neighbors.len() != 2 {
                break;
            }
            let next_edge = if neighbors[0] == prev_edge {
                neighbors[1]
            } else {
                neighbors[0]
            };
            if next_edge == start_idx || chain_indices.contains(&next_edge) || used[next_edge] {
                break;
            }
            let nl = lines[next_edge];
            let (next_pt, orient_bow) = if quant_pt(nl.start) == cur_key {
                (nl.end, nl.bow_angle)
            } else {
                (nl.start, -nl.bow_angle)
            };
            chain_indices.push(next_edge);
            chain_pts.push(cur_pt);
            chain_bows.push(orient_bow);
            cur_pt = next_pt;
            cur_key = quant_pt(next_pt);
            prev_edge = next_edge;
        }

        if closed {
            for idx in chain_indices {
                used[idx] = true;
            }
            let vertices: Vec<Vertex> = chain_pts
                .into_iter()
                .zip(chain_bows)
                .map(|(pt, bow)| {
                    let p = to_inches(pt, origin);
                    Vertex {
                        x: p.x,
                        y: p.y,
                        b: bow_to_bulge(bow),
                    }
                })
                .collect();
            entities.push(Entity {
                entity_type: EntityType::Polyline,
                closed: Some(true),
                vertices: Some(vertices),
                ..Default::default()
            });
        }
    }

    for (idx, line) in lines.iter().enumerate() {
        if !used[idx] && distance(line.start, line.end) > 1e-4 {
            entities.push(line_to_entity(line, origin));
        }
    }

    for t in texts {
        let rot = (-t.rotation).rem_euclid(360.0);
        entities.push(Entity {
            entity_type: EntityType::Text,
            insert_point: Some(to_inches(Pt::new(t.x, t.y), origin)),
            font_size: Some(t.font_size.max(1.0)),
            rotation: if rot.abs() > 1e-6 && (rot - 360.0).abs() > 1e-6 {
                Some(rot)
            } else {
                None
            },
            alignment: Some("MIDDLE_CENTER".into()),
            text_string: Some(t.text.clone()),
            ..Default::default()
        });
    }

    entities
}

/// Computes the world bounding box in inches for `geometry`.
pub fn compute_bounds(geometry: &[Entity]) -> Bounds {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    let mut include = |x: f64, y: f64| {
        if x.is_finite() && y.is_finite() {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    };

    for e in geometry {
        match e.entity_type {
            EntityType::Line => {
                if let Some(p) = e.start_point {
                    include(p.x, p.y);
                }
                if let Some(p) = e.end_point {
                    include(p.x, p.y);
                }
            }
            EntityType::Polyline => {
                if let Some(vs) = &e.vertices {
                    let closed = e.closed.unwrap_or(false);
                    for i in 0..vs.len() {
                        include(vs[i].x, vs[i].y);
                        let bulge = vs[i].b.unwrap_or(0.0);
                        if bulge.abs() > 1e-5 && (closed || i + 1 < vs.len()) {
                            let next = &vs[(i + 1) % vs.len()];
                            let chord = (next.x - vs[i].x).hypot(next.y - vs[i].y);
                            if chord > 1e-5 {
                                let nx = -(next.y - vs[i].y) / chord;
                                let ny = (next.x - vs[i].x) / chord;
                                let off = chord * (1.0 - bulge * bulge) / (4.0 * bulge);
                                let cx = (vs[i].x + next.x) / 2.0 + nx * off;
                                let cy = (vs[i].y + next.y) / 2.0 + ny * off;
                                let r = chord * (1.0 + bulge * bulge) / (4.0 * bulge.abs());
                                let a0 = (vs[i].y - cy).atan2(vs[i].x - cx);
                                let sweep = 4.0 * bulge.atan();
                                for s in 1..16 {
                                    let a = a0 + sweep * (s as f64) / 16.0;
                                    include(cx + r * a.cos(), cy + r * a.sin());
                                }
                            }
                        }
                    }
                }
            }
            EntityType::Ellipse | EntityType::Arc => {
                if let (Some(c), Some(ax)) = (e.center, e.axis_point) {
                    let dx = ax.x - c.x;
                    let dy = ax.y - c.y;
                    let rx = dx.hypot(dy);
                    let ry = rx * e.ratio.filter(|r| *r != 0.0).unwrap_or(1.0).abs();
                    let rot = dy.atan2(dx);
                    let (sr, cr) = rot.sin_cos();
                    let (a0, a1) = if matches!(e.entity_type, EntityType::Arc) {
                        let start = e.start_angle.unwrap_or(0.0).to_radians();
                        let end = e.end_angle.unwrap_or(0.0).to_radians();
                        let mut sweep = (end - start).rem_euclid(std::f64::consts::TAU);
                        if sweep < 1e-6 {
                            sweep = std::f64::consts::TAU;
                        }
                        (start, start + sweep)
                    } else {
                        (0.0, std::f64::consts::TAU)
                    };
                    for s in 0..=24 {
                        let t = a0 + (a1 - a0) * (s as f64) / 24.0;
                        let x = rx * t.cos();
                        let y = ry * t.sin();
                        include(c.x + x * cr - y * sr, c.y + x * sr + y * cr);
                    }
                }
            }
            EntityType::Text => {
                if let Some(p) = e.insertion_point.or(e.insert_point) {
                    let h = e.font_size.unwrap_or(10.0).max(1.0);
                    let len = e.text_string.as_deref().map_or(1, str::len).max(1) as f64;
                    let hw = len * h * 0.3;
                    let hh = h * 0.5;
                    include(p.x - hw, p.y - hh);
                    include(p.x + hw, p.y + hh);
                }
            }
        }
    }

    if !min_x.is_finite() {
        Bounds {
            min_x: -6.0,
            min_y: -6.0,
            max_x: 6.0,
            max_y: 6.0,
        }
    } else {
        Bounds {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }
}

fn collect_selected_lines_and_texts<'a>(
    drawing: &'a Drawing,
    selection: &Selection,
) -> (Vec<&'a SketchLine>, Vec<&'a PlacedText>) {
    let mut line_ids: BTreeSet<i64> = selection.lines.iter().copied().collect();
    for poly_id in &selection.polys {
        if let Some(poly) = drawing.polygons.iter().find(|p| p.id == *poly_id) {
            for edge in &poly.edges {
                if let Some(lid) = edge.line_id {
                    line_ids.insert(lid);
                }
            }
        }
    }
    let lines: Vec<&SketchLine> = drawing
        .lines
        .iter()
        .filter(|l| line_ids.contains(&l.id))
        .collect();
    let texts: Vec<&PlacedText> = drawing
        .texts
        .iter()
        .filter(|t| selection.texts.contains(&t.id))
        .collect();
    (lines, texts)
}

/// True when the active selection contains at least one wall/line, area, or text label
/// convertible into a symbol (§6.1).
pub fn selection_convertible(drawing: &Drawing, selection: &Selection) -> bool {
    let (lines, texts) = collect_selected_lines_and_texts(drawing, selection);
    !lines.is_empty() || !texts.is_empty()
}

/// Converts selected walls, curved walls, polygons, and text labels into symbol
/// entities in inches centred on the selection's bounding box (§6.1).
pub fn from_selection(
    drawing: &Drawing,
    selection: &Selection,
) -> Result<(Vec<Entity>, Bounds), String> {
    let (lines, texts) = collect_selected_lines_and_texts(drawing, selection);
    if lines.is_empty() && texts.is_empty() {
        return Err("Select one or more walls, areas, or text labels first.".into());
    }
    let temp = lines_and_texts_to_entities(&lines, &texts, Pt::new(0.0, 0.0));
    let b0 = compute_bounds(&temp);
    let origin = Pt::new((b0.min_x + b0.max_x) / 24.0, (b0.min_y + b0.max_y) / 24.0);
    let entities = lines_and_texts_to_entities(&lines, &texts, origin);
    let bounds = compute_bounds(&entities);
    Ok((entities, bounds))
}

fn alignment_is_editable(align: Option<&str>) -> bool {
    let raw = align.unwrap_or("").trim().to_uppercase();
    raw.is_empty()
        || raw == "CENTER"
        || raw == "MIDDLE"
        || raw == "MIDDLE_CENTER"
        || raw == "MIDDLECENTER"
}

/// Opens a symbol's geometry inside a blank [`Drawing`] (1 inch = 1/12 ft around
/// origin `(0, 0)`), returning `(drawing, locked_entities)` for entities the
/// sketch tools cannot represent directly (§6.2).
pub fn symbol_to_drawing(sym: &Symbol) -> (Drawing, Vec<Entity>) {
    let mut drawing = Drawing::default();
    let mut locked = Vec::new();
    let mut next_id = 1i64;
    let (ext_col, ext_w) = default_line_style(WallType::Exterior);

    let mut push_line = |start: Pt, end: Pt, bow_angle: f64| {
        if distance(start, end) < 1e-4 {
            return;
        }
        let id = next_id;
        next_id += 1;
        drawing.lines.push(SketchLine {
            id,
            start,
            end,
            kind: WallType::Exterior,
            color: Some(ext_col.to_owned()),
            line_thickness: Some(ext_w),
            thickness: None,
            dimension_hidden: true,
            layer_id: None,
            bow_angle,
            arc_height: 0.0,
            line_style: None,
            dimension_position: None,
            dimension_anchor: None,
        });
    };

    for e in &sym.geometry {
        match e.entity_type {
            EntityType::Line => {
                if let (Some(a), Some(b)) = (e.start_point, e.end_point) {
                    push_line(to_feet(a), to_feet(b), 0.0);
                }
            }
            EntityType::Polyline => {
                let Some(vs) = e.vertices.as_ref().filter(|v| v.len() >= 2) else {
                    continue;
                };
                let closed = e.closed.unwrap_or(false);
                let count = if closed { vs.len() } else { vs.len() - 1 };
                for i in 0..count {
                    let a = &vs[i];
                    let b = &vs[(i + 1) % vs.len()];
                    push_line(
                        to_feet(Point { x: a.x, y: a.y }),
                        to_feet(Point { x: b.x, y: b.y }),
                        bulge_to_bow(a.b.unwrap_or(0.0)),
                    );
                }
            }
            EntityType::Arc => {
                let ratio = e.ratio.filter(|r| *r != 0.0).unwrap_or(1.0).abs();
                let (Some(c), Some(ax)) = (e.center, e.axis_point) else {
                    continue;
                };
                if (ratio - 1.0).abs() > 1e-2 {
                    locked.push(e.clone());
                    continue;
                }
                let dx = ax.x - c.x;
                let dy = ax.y - c.y;
                let radius = dx.hypot(dy);
                if radius < 1e-4 {
                    continue;
                }
                let base_ang = dy.atan2(dx);
                let a0 = base_ang + e.start_angle.unwrap_or(0.0).to_radians();
                let a1 = base_ang + e.end_angle.unwrap_or(0.0).to_radians();
                let mut sweep = (a1 - a0).rem_euclid(std::f64::consts::TAU);
                if sweep < 1e-5 {
                    sweep = std::f64::consts::TAU;
                }
                let sweep_deg = sweep.to_degrees();
                if sweep_deg >= MAX_BOW_ANGLE {
                    let mid = a0 + sweep / 2.0;
                    let p0 = to_feet(Point {
                        x: c.x + radius * a0.cos(),
                        y: c.y + radius * a0.sin(),
                    });
                    let pm = to_feet(Point {
                        x: c.x + radius * mid.cos(),
                        y: c.y + radius * mid.sin(),
                    });
                    let p1 = to_feet(Point {
                        x: c.x + radius * a1.cos(),
                        y: c.y + radius * a1.sin(),
                    });
                    let half = -(sweep_deg / 2.0);
                    push_line(p0, pm, half);
                    push_line(pm, p1, half);
                } else {
                    let p0 = to_feet(Point {
                        x: c.x + radius * a0.cos(),
                        y: c.y + radius * a0.sin(),
                    });
                    let p1 = to_feet(Point {
                        x: c.x + radius * a1.cos(),
                        y: c.y + radius * a1.sin(),
                    });
                    push_line(p0, p1, -sweep_deg);
                }
            }
            EntityType::Ellipse => {
                locked.push(e.clone());
            }
            EntityType::Text => {
                let text = e.text_string.clone().unwrap_or_default();
                if text.trim().is_empty() || !alignment_is_editable(e.alignment.as_deref()) {
                    locked.push(e.clone());
                    continue;
                }
                let pt = to_feet(
                    e.insertion_point
                        .or(e.insert_point)
                        .unwrap_or(Point { x: 0.0, y: 0.0 }),
                );
                drawing.texts.push(PlacedText {
                    id: 0,
                    text,
                    x: pt.x,
                    y: pt.y,
                    rotation: -e.rotation.unwrap_or(0.0),
                    font_family: "Arial".into(),
                    font_size: e.font_size.unwrap_or(10.0).max(1.0),
                    bold: false,
                    italic: false,
                    underline: false,
                    color: String::new(),
                    layer_id: 0,
                });
            }
        }
    }

    for t in &mut drawing.texts {
        t.id = next_id;
        next_id += 1;
    }
    drawing.set_next_id(next_id);
    (drawing, locked)
}

/// Converts a temporary shape-editing [`Drawing`] back to symbol entities + bounds
/// relative to `(0, 0)`, appending any `locked` entities unchanged (§6.2).
pub fn drawing_to_geometry(
    lines: &[SketchLine],
    texts: &[PlacedText],
    locked: &[Entity],
) -> (Vec<Entity>, Bounds) {
    let line_refs: Vec<&SketchLine> = lines.iter().collect();
    let text_refs: Vec<&PlacedText> = texts.iter().collect();
    let mut entities = lines_and_texts_to_entities(&line_refs, &text_refs, Pt::new(0.0, 0.0));
    entities.extend(locked.iter().cloned());
    let bounds = compute_bounds(&entities);
    (entities, bounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_line(id: i64, x1: f64, y1: f64, x2: f64, y2: f64, bow: f64) -> SketchLine {
        SketchLine {
            id,
            start: Pt::new(x1, y1),
            end: Pt::new(x2, y2),
            kind: WallType::Exterior,
            color: None,
            line_thickness: None,
            thickness: None,
            dimension_hidden: false,
            layer_id: None,
            bow_angle: bow,
            arc_height: 0.0,
            line_style: None,
            dimension_position: None,
            dimension_anchor: None,
        }
    }

    #[test]
    fn selection_rectangle_and_label_round_trip_in_inches() {
        let mut drawing = Drawing::default();
        drawing.lines = vec![
            mk_line(1, 10.0, 20.0, 14.0, 20.0, 0.0),
            mk_line(2, 14.0, 20.0, 14.0, 22.0, 0.0),
            mk_line(3, 14.0, 22.0, 10.0, 22.0, 0.0),
            mk_line(4, 10.0, 22.0, 10.0, 20.0, 0.0),
        ];
        drawing.texts = vec![PlacedText {
            id: 5,
            text: "WH".into(),
            x: 12.0,
            y: 21.0,
            rotation: 15.0,
            font_family: "Arial".into(),
            font_size: 8.0,
            bold: false,
            italic: false,
            underline: false,
            color: String::new(),
            layer_id: 0,
        }];
        let mut selection = Selection::default();
        selection.lines.extend([1, 2, 3, 4]);
        selection.texts.insert(5);

        let (entities, bounds) = from_selection(&drawing, &selection).expect("convertible");
        assert_eq!(entities.len(), 2, "closed loop polyline + 1 text label");
        assert_eq!(entities[0].entity_type, EntityType::Polyline);
        assert_eq!(entities[0].closed, Some(true));
        // 4 ft wide -> 48 in (-24..+24); 2 ft tall -> 24 in (-12..+12)
        assert!((bounds.max_x - bounds.min_x - 48.0).abs() < 1e-6);
        assert!((bounds.max_y - bounds.min_y - 24.0).abs() < 1e-6);
        assert_eq!(entities[1].entity_type, EntityType::Text);
        assert_eq!(entities[1].rotation, Some(345.0));

        let sym = Symbol {
            id: "custom/wh".into(),
            geometry: entities,
            bounds,
            ..Default::default()
        };
        let (edited, locked) = symbol_to_drawing(&sym);
        assert!(locked.is_empty());
        assert_eq!(edited.lines.len(), 4);
        assert_eq!(edited.texts.len(), 1);
        assert!((edited.texts[0].rotation + 345.0).abs() < 1e-6);
    }

    #[test]
    fn bowed_wall_and_locked_ellipse_round_trip() {
        let line = mk_line(1, -1.0, 0.0, 1.0, 0.0, 60.0);
        let ellipse = Entity {
            entity_type: EntityType::Ellipse,
            center: Some(Point { x: 0.0, y: 0.0 }),
            axis_point: Some(Point { x: 6.0, y: 0.0 }),
            ratio: Some(0.5),
            ..Default::default()
        };
        let right_text = Entity {
            entity_type: EntityType::Text,
            insert_point: Some(Point { x: 4.0, y: 4.0 }),
            alignment: Some("RIGHT".into()),
            text_string: Some("Note".into()),
            ..Default::default()
        };
        let arc_entity = line_to_entity(&line, Pt::new(0.0, 0.0));
        assert_eq!(arc_entity.entity_type, EntityType::Arc);

        let sym = Symbol {
            id: "custom/arc".into(),
            geometry: vec![arc_entity.clone(), ellipse.clone(), right_text.clone()],
            bounds: Bounds {
                min_x: -12.0,
                min_y: -6.0,
                max_x: 12.0,
                max_y: 6.0,
            },
            ..Default::default()
        };
        let (drawing, locked) = symbol_to_drawing(&sym);
        assert_eq!(drawing.lines.len(), 1);
        assert!((drawing.lines[0].bow_angle.abs() - 60.0).abs() < 1e-4);
        let (_, _, orig_apex) =
            curve_circle_and_apex(line.start, line.end, line.bow_angle).expect("orig apex");
        let (_, _, edit_apex) = curve_circle_and_apex(
            drawing.lines[0].start,
            drawing.lines[0].end,
            drawing.lines[0].bow_angle,
        )
        .expect("edit apex");
        assert!((orig_apex.x - edit_apex.x).abs() < 1e-4);
        assert!((orig_apex.y - edit_apex.y).abs() < 1e-4);
        assert_eq!(locked.len(), 2);

        let (out, _) = drawing_to_geometry(&drawing.lines, &drawing.texts, &locked);
        assert_eq!(out.len(), 3);
        assert_eq!(out[1], ellipse);
        assert_eq!(out[2], right_text);
    }
}
