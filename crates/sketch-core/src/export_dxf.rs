//! DXF (R12 ASCII) export — clean-room port of `Exporter.exportDXF`
//! (`refs/exporter.js`). World coordinates are FEET (y-up, used as-is);
//! straight walls as LINE (or closed solid polylines when geometric wall
//! solids are enabled), bowed walls as ARC (counter-clockwise 50→51 passing
//! through the apex), area polygons as closed POLYLINE, labels as TEXT.
//! Layer names: WALLS-EXTERIOR / WALLS-INTERIOR / AREA-<CODE> /
//! AREA-UNDEFINED / AREA-TEXT / TEXT. Header pins $ACADVER AC1009 and
//! $EXTMIN/$EXTMAX to the export bounds.

use crate::export_model::{
    auto_post_lines, build_area_line_colors, curve_apex, curve_circle, flattened_points,
    get_export_bounds, solids_enabled, ExportOptions, ExportPage,
};
use crate::geometry::{AreaCode, Pt, PIXELS_PER_FOOT};

/// One DXF group-code/value pair line (`dxfPair`): two leading spaces.
fn dxf_pair(code: u16, value: &str) -> String {
    format!("  {code}\n{value}\n")
}

/// DXF text: strip control characters (`dxfText`).
fn dxf_text_escape(value: &str) -> String {
    value
        .chars()
        .map(|c| if (c as u32) < 0x20 { ' ' } else { c })
        .collect()
}

fn dxf_line(layer: &str, a: Pt, b: Pt) -> String {
    dxf_pair(0, "LINE")
        + &dxf_pair(8, layer)
        + &dxf_pair(10, &fmt4(a.x))
        + &dxf_pair(20, &fmt4(a.y))
        + &dxf_pair(11, &fmt4(b.x))
        + &dxf_pair(21, &fmt4(b.y))
}

fn dxf_arc(layer: &str, center: Pt, radius: f64, start_angle: f64, end_angle: f64) -> String {
    dxf_pair(0, "ARC")
        + &dxf_pair(8, layer)
        + &dxf_pair(10, &fmt4(center.x))
        + &dxf_pair(20, &fmt4(center.y))
        + &dxf_pair(40, &fmt4(radius))
        + &dxf_pair(50, &fmt4(start_angle))
        + &dxf_pair(51, &fmt4(end_angle))
}

fn dxf_polyline(layer: &str, points: &[Pt]) -> String {
    let mut out =
        dxf_pair(0, "POLYLINE") + &dxf_pair(8, layer) + &dxf_pair(66, "1") + &dxf_pair(70, "1");
    for p in points {
        out += &dxf_pair(0, "VERTEX");
        out += &dxf_pair(8, layer);
        out += &dxf_pair(10, &fmt4(p.x));
        out += &dxf_pair(20, &fmt4(p.y));
    }
    out + &dxf_pair(0, "SEQEND")
}

fn dxf_text(layer: &str, position: Pt, height: f64, text: &str, rotation: f64) -> String {
    dxf_pair(0, "TEXT")
        + &dxf_pair(8, layer)
        + &dxf_pair(10, &fmt4(position.x))
        + &dxf_pair(20, &fmt4(position.y))
        + &dxf_pair(40, &fmt2(height))
        + &dxf_pair(1, &dxf_text_escape(text))
        + &(if rotation != 0.0 {
            dxf_pair(50, &fmt2(rotation))
        } else {
            String::new()
        })
}

fn fmt4(v: f64) -> String {
    format!("{v:.4}")
}

fn fmt2(v: f64) -> String {
    format!("{v:.2}")
}

/// `exportDXF` — the R12 ASCII document. The caller writes it to the chosen
/// path.
pub fn export_dxf(page: &ExportPage, options: &ExportOptions, area_codes: &[AreaCode]) -> String {
    let (bounds_min, bounds_max) = get_export_bounds(page);
    let mut entities: Vec<String> = Vec::new();

    let wall_layer = |exterior: bool| -> &'static str {
        if exterior {
            "WALLS-EXTERIOR"
        } else {
            "WALLS-INTERIOR"
        }
    };

    // Walls. With geometric wall thickness enabled the straight walls export
    // as closed solid polygons instead; bowed walls always export as ARC.
    let solids = solids_enabled(options);
    if solids {
        for solid in &page.wall_solids {
            let Some(line) = page.lines.iter().find(|l| l.id == solid.line_id) else {
                continue;
            };
            let layer = wall_layer(line.is_exterior());
            for piece in &solid.pieces {
                entities.push(dxf_polyline(layer, piece));
            }
        }
    }
    for line in &page.lines {
        let layer = wall_layer(line.is_exterior());
        let Some((circle_center, circle_radius)) = curve_circle(line) else {
            // Straight wall: break through its opening runs unless the solid
            // path already covered it.
            if !solids {
                let dx = line.end.x - line.start.x;
                let dy = line.end.y - line.start.y;
                let runs: Vec<(f64, f64)> = if line.openings.is_empty() {
                    vec![(0.0, 1.0)]
                } else {
                    line.openings.clone()
                };
                for (start, end) in &runs {
                    if end <= start {
                        continue;
                    }
                    entities.push(dxf_line(
                        layer,
                        Pt::new(line.start.x + dx * start, line.start.y + dy * start),
                        Pt::new(line.start.x + dx * end, line.start.y + dy * end),
                    ));
                }
            }
            continue;
        };
        let apex = curve_apex(line).expect("apex present when circle is");
        let to_deg = |v: f64| (v * 180.0 / std::f64::consts::PI).rem_euclid(360.0);
        let a_s = (line.start.y - circle_center.y).atan2(line.start.x - circle_center.x);
        let a_e = (line.end.y - circle_center.y).atan2(line.end.x - circle_center.x);
        let a_m = (apex.y - circle_center.y).atan2(apex.x - circle_center.x);
        let two_pi = std::f64::consts::TAU;
        let mod2pi = |v: f64| v.rem_euclid(two_pi);
        // DXF arcs run counter-clockwise from 50 to 51; pick the orientation
        // whose CCW travel passes through the apex.
        let ccw = mod2pi(a_m - a_s) <= mod2pi(a_e - a_s);
        entities.push(dxf_arc(
            layer,
            circle_center,
            circle_radius,
            to_deg(if ccw { a_s } else { a_e }),
            to_deg(if ccw { a_e } else { a_s }),
        ));
    }

    // Area polygons as closed polylines (flattened so bowed edges export).
    for poly in &page.polygons {
        if poly.points.len() < 3 {
            continue;
        }
        let layer = if !poly.area_type.is_empty() && poly.area_type != "UND" {
            format!("AREA-{}", poly.area_type)
        } else {
            "AREA-UNDEFINED".to_owned()
        };
        entities.push(dxf_polyline(&layer, &flattened_points(poly)));
    }

    // Auto-posted area labels and free text labels.
    for poly in &page.polygons {
        let lines = auto_post_lines(poly, options, area_codes);
        if lines.is_empty() || poly.points.len() < 3 {
            continue;
        }
        let engine = crate::geometry::GeometryEngine::new();
        let centroid = poly
            .label_position
            .unwrap_or_else(|| engine.area_label_anchor(&flattened_points(poly)));
        let font = poly
            .label_font_size
            .map(crate::export_model::num_or_zero)
            .filter(|f| *f != 0.0)
            .unwrap_or(options.auto_post_font_size)
            .max(16.0);
        let height = (font / PIXELS_PER_FOOT).max(0.5);
        for (index, text) in lines.iter().enumerate() {
            let y =
                centroid.y + (((lines.len() as f64 - 1.0) / 2.0 - index as f64) * height * 1.25);
            entities.push(dxf_text(
                "AREA-TEXT",
                Pt::new(centroid.x, y),
                height,
                text,
                crate::export_model::num_or_zero(poly.label_rotation),
            ));
        }
    }
    for label in &page.labels {
        let raw_size = label
            .font_size
            .map(crate::export_model::num_or_zero)
            .filter(|f| *f != 0.0)
            .unwrap_or(12.0);
        let height = (raw_size / PIXELS_PER_FOOT).max(0.5);
        entities.push(dxf_text(
            "TEXT",
            Pt::new(label.x, label.y),
            height,
            &label.text,
            crate::export_model::num_or_zero(label.rotation),
        ));
    }

    let _ = build_area_line_colors(page, area_codes);
    let mut dxf = String::new();
    dxf += &dxf_pair(0, "SECTION");
    dxf += &dxf_pair(2, "HEADER");
    dxf += &dxf_pair(9, "$ACADVER");
    dxf += &dxf_pair(1, "AC1009");
    dxf += &dxf_pair(9, "$EXTMIN");
    dxf += &dxf_pair(10, &fmt4(bounds_min.x));
    dxf += &dxf_pair(20, &fmt4(bounds_min.y));
    dxf += &dxf_pair(9, "$EXTMAX");
    dxf += &dxf_pair(10, &fmt4(bounds_max.x));
    dxf += &dxf_pair(20, &fmt4(bounds_max.y));
    dxf += &dxf_pair(0, "ENDSEC");
    dxf += &dxf_pair(0, "SECTION");
    dxf += &dxf_pair(2, "ENTITIES");
    for entity in &entities {
        dxf += entity;
    }
    dxf += &dxf_pair(0, "ENDSEC");
    dxf += &dxf_pair(0, "EOF");
    dxf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_model::{ExportLine, ExportPage, ExportPolygon};
    use crate::geometry::Edge;

    fn simple_page() -> ExportPage {
        ExportPage {
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
        }
    }

    #[test]
    fn dxf_header_and_eof() {
        let dxf = export_dxf(&simple_page(), &ExportOptions::default(), &[]);
        assert!(dxf.contains("  0\nSECTION\n"));
        assert!(dxf.contains("  2\nHEADER\n"));
        assert!(dxf.contains("  9\n$ACADVER\n"));
        assert!(dxf.contains("  1\nAC1009\n"));
        assert!(dxf.contains("  0\nENDSEC\n"));
        assert!(dxf.trim_end().ends_with("EOF"));
    }

    #[test]
    fn dxf_extents_and_layers() {
        let dxf = export_dxf(&simple_page(), &ExportOptions::default(), &[]);
        assert!(dxf.contains("  9\n$EXTMIN\n"));
        // Wall on WALLS-EXTERIOR, polygon on AREA-LIV, label on AREA-TEXT.
        assert!(dxf.contains("  8\nWALLS-EXTERIOR\n"));
        assert!(dxf.contains("  8\nAREA-LIV\n"));
        assert!(dxf.contains("  8\nAREA-TEXT\n"));
        assert!(dxf.contains("  1\nLiving Area\n"));
        assert!(dxf.contains("  1\n200 sf\n"));
    }

    #[test]
    fn dxf_straight_wall_line_and_opening_runs() {
        let mut page = simple_page();
        page.lines[0].openings = vec![(0.0, 0.5)];
        let dxf = export_dxf(&page, &ExportOptions::default(), &[]);
        assert!(dxf.contains("  0\nLINE\n"));
        // One opening run -> one LINE entity on the walls layer.
        let wall_section = dxf.split("ENTITIES").nth(1).expect("entities");
        assert_eq!(wall_section.matches("  0\nLINE\n").count(), 1);
    }

    #[test]
    fn dxf_bowed_wall_arc_counter_clockwise_through_apex() {
        let mut page = simple_page();
        page.lines[0].bow_angle = 30.0;
        let dxf = export_dxf(&page, &ExportOptions::default(), &[]);
        assert!(dxf.contains("  0\nARC\n"));
        assert!(!dxf.contains("  0\nLINE\n"));
    }

    #[test]
    fn dxf_solids_export_closed_polylines() {
        let mut page = simple_page();
        page.wall_solids = vec![crate::export_model::ExportSolid {
            line_id: 1,
            pieces: vec![vec![
                Pt::new(0.0, 0.25),
                Pt::new(20.0, 0.25),
                Pt::new(20.0, -0.25),
                Pt::new(0.0, -0.25),
            ]],
        }];
        let options = ExportOptions {
            wall_solids: true,
            ..Default::default()
        };
        let dxf = export_dxf(&page, &options, &[]);
        assert!(dxf.contains("  0\nPOLYLINE\n"));
        assert!(
            !dxf.contains("  0\nLINE\n"),
            "straight walls are solid polylines when enabled"
        );
    }

    #[test]
    fn dxf_text_strips_control_chars() {
        assert_eq!(dxf_text_escape("a\u{1}b"), "a b");
    }
}
