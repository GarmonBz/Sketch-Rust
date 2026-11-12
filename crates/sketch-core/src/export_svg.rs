//! SVG export — clean-room port of `Exporter.exportSVG` (`refs/exporter.js`).
//! String-built with the browser's structure: 1 unit = 1 screen pixel
//! (20 units per foot), 40-unit margin, y-down mapping, mirrored canvas
//! appearance (wall colors/widths/dashes, fills, labels, symbols).

use crate::export_model::{
    auto_post_lines, build_area_line_colors, curve_apex, curve_circle, escape_xml,
    flattened_points, get_export_bounds_with_library, num_or_zero, solids_enabled, ExportLine,
    ExportOptions, ExportPage,
};
use crate::geometry::{AreaCode, Pt, PIXELS_PER_FOOT};
use crate::symbols::{Entity, EntityType, SymbolLibrary};

/// Stroke width of a wall (SVG px): per-line thickness, else the option
/// default, else the browser's 4/2 fallback (`strokeWallPath`).
fn wall_width(line: &ExportLine, options: &ExportOptions) -> f64 {
    if line.line_thickness != 0.0 {
        return line.line_thickness;
    }
    let option_width = if line.is_exterior() {
        options.exterior_width
    } else {
        options.interior_width
    };
    if option_width != 0.0 {
        return option_width;
    }
    if line.is_exterior() {
        4.0
    } else {
        2.0
    }
}

fn wall_color(
    line: &ExportLine,
    area_line_colors: &std::collections::BTreeMap<i64, String>,
    options: &ExportOptions,
) -> String {
    area_line_colors
        .get(&line.id)
        .cloned()
        .or_else(|| line.color.clone())
        .unwrap_or_else(|| {
            if line.is_exterior() {
                options.exterior_color.clone()
            } else {
                options.interior_color.clone()
            }
        })
}

/// Stroke path for a single wall (`strokeWallPath`): bowed walls arc,
/// straight walls break through their opening runs.
fn wall_path(
    line: &ExportLine,
    transform: &dyn Fn(Pt) -> (f64, f64),
    area_line_colors: &std::collections::BTreeMap<i64, String>,
    options: &ExportOptions,
) -> String {
    let color = escape_xml(&wall_color(line, area_line_colors, options));
    let width = wall_width(line, options);
    let dash = match line.line_style.as_deref() {
        Some("dashed") => " stroke-dasharray=\"10 6\"",
        Some("dotted") => " stroke-dasharray=\"2 5\"",
        _ => "",
    };
    let apex = curve_apex(line);
    let at = |f: f64| {
        Pt::new(
            line.start.x + (line.end.x - line.start.x) * f,
            line.start.y + (line.end.y - line.start.y) * f,
        )
    };
    let d = match apex {
        Some(_) => curved_wall_path(line, transform),
        None => {
            let runs: Vec<(f64, f64)> = if line.openings.is_empty() {
                vec![(0.0, 1.0)]
            } else {
                line.openings.clone()
            };
            runs.iter()
                .filter(|(start, end)| end > start)
                .map(|(start, end)| {
                    let a = at(*start);
                    let b = at(*end);
                    format!(
                        "M{},{} L{},{}",
                        fmt2(transform(a).0),
                        fmt2(transform(a).1),
                        fmt2(transform(b).0),
                        fmt2(transform(b).1)
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    };
    if d.is_empty() {
        return String::new();
    }
    format!(
        "<path d=\"{d}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{width}\"{dash} stroke-linecap=\"round\" stroke-linejoin=\"round\"/>"
    )
}

fn fmt2(v: f64) -> String {
    format!("{v:.2}")
}

/// SVG arc path segment for a curved wall (`curvedWallPath`), choosing the
/// sweep flag so the arc passes through the apex.
fn curved_wall_path(line: &ExportLine, transform: &dyn Fn(Pt) -> (f64, f64)) -> String {
    let Some((circle_center, circle_radius)) = curve_circle(line) else {
        let (sx, sy) = transform(line.start);
        let (ex, ey) = transform(line.end);
        return format!("M{},{} L{},{}", fmt2(sx), fmt2(sy), fmt2(ex), fmt2(ey));
    };
    let _ = circle_center;
    let (sx, sy) = transform(line.start);
    let (ex, ey) = transform(line.end);
    let apex = curve_apex(line).expect("apex present when circle is");
    let (mx, my) = transform(apex);
    let r = circle_radius * PIXELS_PER_FOOT;
    // Candidate center: offset from the chord midpoint along the chord normal.
    let chord = (ex - sx).hypot(ey - sy);
    let half_chord = chord / 2.0;
    let rise = (mx - (sx + ex) / 2.0).hypot(my - (sy + ey) / 2.0);
    let center_dist = (r * r - half_chord * half_chord).max(0.0).sqrt();
    let n = (
        (mx - (sx + ex) / 2.0) / rise.max(1.0),
        (my - (sy + ey) / 2.0) / rise.max(1.0),
    );
    let cx = (sx + ex) / 2.0 - n.0 * center_dist;
    let cy = (sy + ey) / 2.0 - n.1 * center_dist;
    let a1 = (sy - cy).atan2(sx - cx);
    let a_m = (my - cy).atan2(mx - cx);
    let a2 = (ey - cy).atan2(ex - cx);
    let two_pi = std::f64::consts::TAU;
    let mod2pi = |v: f64| v.rem_euclid(two_pi);
    let d_m = mod2pi(a_m - a1);
    let d2 = mod2pi(a2 - a1);
    // Positive-angle direction passes through the apex when dM <= d2.
    let positive = d_m <= d2;
    let sweep_angle = if positive { d2 } else { two_pi - d2 };
    let large_arc = if sweep_angle > std::f64::consts::PI {
        1
    } else {
        0
    };
    let sweep = if positive { 1 } else { 0 };
    format!(
        "M{},{} A{},{} 0 {},{} {},{}",
        fmt2(sx),
        fmt2(sy),
        fmt2(r),
        fmt2(r),
        large_arc,
        sweep,
        fmt2(ex),
        fmt2(ey)
    )
}

/// Symbol geometry as inline SVG entities (`symbolEntities`), drawn in the
/// symbol's local inches with y negated. Returns the inner markup plus the
/// stroke color.
pub fn symbol_entities(
    symbol: &crate::export_model::ExportSymbol,
    library: Option<&SymbolLibrary>,
) -> Option<(String, String)> {
    let definition = library.and_then(|lib| lib.symbol(&symbol.symbol_type))?;
    if definition.geometry.is_empty() {
        return None;
    }
    let stroke = escape_xml(symbol.color.as_deref().unwrap_or("#111111"));
    let polyline_path = |vertices: &[crate::symbols::Vertex], closed: bool| -> String {
        if vertices.is_empty() {
            return String::new();
        }
        let mut d = format!(
            "M{},{}",
            fmt2(num_or_zero(vertices[0].x)),
            fmt2(-num_or_zero(vertices[0].y))
        );
        let count = if closed {
            vertices.len()
        } else {
            vertices.len() - 1
        };
        for i in 0..count {
            let start = &vertices[i];
            let end = &vertices[(i + 1) % vertices.len()];
            let bulge = start.b.map(num_or_zero).unwrap_or(0.0);
            let dx = num_or_zero(end.x) - num_or_zero(start.x);
            let dy = num_or_zero(end.y) - num_or_zero(start.y);
            let chord = dx.hypot(dy);
            if bulge.abs() < 1e-7 || chord < 1e-7 {
                d += &format!(
                    " L{},{}",
                    fmt2(num_or_zero(end.x)),
                    fmt2(-num_or_zero(end.y))
                );
            } else {
                let radius = chord * (1.0 + bulge * bulge) / (4.0 * bulge.abs());
                let sweep = (4.0 * bulge.atan()).abs();
                d += &format!(
                    " A{},{} 0 {},{} {},{}",
                    fmt2(radius),
                    fmt2(radius),
                    if sweep > std::f64::consts::PI { 1 } else { 0 },
                    if bulge > 0.0 { 0 } else { 1 },
                    fmt2(num_or_zero(end.x)),
                    fmt2(-num_or_zero(end.y))
                );
            }
        }
        if closed {
            d += " Z";
        }
        d
    };
    let mut entities = String::new();
    for entity in &definition.geometry {
        match entity.entity_type {
            EntityType::Line => {
                let s = entity.start_point.unwrap_or_default();
                let e = entity.end_point.unwrap_or_default();
                entities += &format!(
                    "<path d=\"M{},{} L{},{}\"/>",
                    num_or_zero(s.x),
                    -num_or_zero(s.y),
                    num_or_zero(e.x),
                    -num_or_zero(e.y)
                );
            }
            EntityType::Polyline => {
                let vertices = entity.vertices.as_deref().unwrap_or(&[]);
                entities += &format!(
                    "<path d=\"{}\" fill=\"none\"/>",
                    polyline_path(vertices, entity.closed.unwrap_or(false))
                );
            }
            EntityType::Ellipse | EntityType::Arc => {
                let center = entity.center.unwrap_or_default();
                let axis = entity.axis_point.unwrap_or(center);
                let rx = (num_or_zero(axis.x) - num_or_zero(center.x))
                    .hypot(num_or_zero(axis.y) - num_or_zero(center.y));
                let ry = rx * (entity.ratio.map(num_or_zero).unwrap_or(1.0)).abs();
                if entity.entity_type == EntityType::Ellipse {
                    entities += &format!(
                        "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"none\"/>",
                        num_or_zero(center.x),
                        -num_or_zero(center.y),
                        rx,
                        ry
                    );
                } else {
                    let start_angle = entity.start_angle.map(num_or_zero).unwrap_or(0.0)
                        * std::f64::consts::PI
                        / 180.0;
                    let end_angle = entity.end_angle.map(num_or_zero).unwrap_or(0.0)
                        * std::f64::consts::PI
                        / 180.0;
                    let sx = num_or_zero(center.x) + rx * start_angle.cos();
                    let sy = -num_or_zero(center.y) - ry * start_angle.sin();
                    let ex = num_or_zero(center.x) + rx * end_angle.cos();
                    let ey = -num_or_zero(center.y) - ry * end_angle.sin();
                    let start_deg = entity.start_angle.map(num_or_zero).unwrap_or(0.0);
                    let end_deg = entity.end_angle.map(num_or_zero).unwrap_or(0.0);
                    let sweep = (end_deg - start_deg).rem_euclid(360.0);
                    entities += &format!(
                        "<path d=\"M{},{} A{},{} 0 {},0 {},{}\" fill=\"none\"/>",
                        sx,
                        sy,
                        rx,
                        ry,
                        if sweep > 180.0 { 1 } else { 0 },
                        ex,
                        ey
                    );
                }
            }
            EntityType::Text => {
                let insert = entity
                    .insertion_point
                    .or(entity.insert_point)
                    .unwrap_or_default();
                let raw_font = entity.font_size.map(num_or_zero).unwrap_or(0.0);
                let font_size = (if raw_font != 0.0 { raw_font } else { 10.0 }).max(1.0);
                entities += &format!(
                    "<text x=\"{}\" y=\"{}\" font-size=\"{}\" stroke=\"none\" fill=\"currentColor\">{}</text>",
                    num_or_zero(insert.x),
                    -num_or_zero(insert.y),
                    font_size,
                    escape_xml(entity.text_string.as_deref().unwrap_or(""))
                );
            }
        }
    }
    Some((entities, stroke))
}

/// `exportSVG` — the full SVG document string. The caller writes it to the
/// chosen path.
pub fn export_svg(
    page: &ExportPage,
    options: &ExportOptions,
    area_codes: &[AreaCode],
    library: Option<&SymbolLibrary>,
) -> String {
    let (bounds_min, bounds_max) = get_export_bounds_with_library(page, library);
    let margin = 40.0; // export units
    let width = (bounds_max.x - bounds_min.x) * PIXELS_PER_FOOT + margin * 2.0;
    let height = (bounds_max.y - bounds_min.y) * PIXELS_PER_FOOT + margin * 2.0;
    // Padded origin so the margin appears on every side.
    let pad = margin / PIXELS_PER_FOOT;
    let origin_min = Pt::new(bounds_min.x - pad, bounds_min.y - pad);
    let origin_max_y = bounds_max.y + pad;
    let transform = |world: Pt| {
        (
            (num_or_zero(world.x) - origin_min.x) * PIXELS_PER_FOOT,
            (origin_max_y - num_or_zero(world.y)) * PIXELS_PER_FOOT,
        )
    };
    let area_line_colors = build_area_line_colors(page, area_codes);

    // Filled area polygons.
    let mut polygons = String::new();
    for poly in &page.polygons {
        if poly.points.len() < 3 {
            continue;
        }
        let definition = crate::export_model::area_code_lookup(area_codes, &poly.area_type);
        let outline = escape_xml(
            poly.line_color
                .as_deref()
                .or(definition.map(|c| c.color.as_str()))
                .unwrap_or("#111111"),
        );
        let fill_color = poly
            .fill_color
            .as_deref()
            .or(definition
                .map(|c| c.fill_color.as_str())
                .filter(|s| !s.is_empty()))
            .or(definition.map(|c| c.color.as_str()))
            .unwrap_or("#ffffff");
        let opacity = poly
            .fill_opacity
            .or(definition.map(|c| c.fill_opacity))
            .unwrap_or(10.0);
        let style = poly
            .fill_pattern
            .as_deref()
            .or(definition.map(|c| c.fill_style.as_str()))
            .unwrap_or("solid");
        let (fill, fill_opacity) = if style == "none" || poly.area_type == "UND" {
            ("none", 0.0)
        } else {
            (fill_color, num_or_zero(opacity).clamp(0.0, 100.0) / 100.0)
        };
        let d = flattened_points(poly)
            .iter()
            .map(|p| {
                let (x, y) = transform(*p);
                format!("{},{}", fmt2(x), fmt2(y))
            })
            .collect::<Vec<_>>()
            .join(" L");
        let thickness = num_or_zero(poly.line_thickness.unwrap_or(0.0));
        let stroke_width = (if thickness != 0.0 { thickness } else { 1.5 }).max(1.0);
        polygons += &format!(
            "<path d=\"M{d} Z\" fill=\"{fill}\" fill-opacity=\"{fill_opacity:.2}\" stroke=\"{outline}\" stroke-width=\"{stroke_width}\" stroke-linejoin=\"round\"/>"
        );
    }

    // Walls: stroked paths (straight walls break through their openings,
    // bowed walls stay stroked arcs); solid pieces fill first when enabled.
    let mut walls = String::new();
    if solids_enabled(options) {
        for solid in &page.wall_solids {
            let Some(line) = page.lines.iter().find(|l| l.id == solid.line_id) else {
                continue;
            };
            let color = escape_xml(&wall_color(line, &area_line_colors, options));
            for piece in &solid.pieces {
                let d = piece
                    .iter()
                    .map(|p| {
                        let (x, y) = transform(*p);
                        format!("{},{}", fmt2(x), fmt2(y))
                    })
                    .collect::<Vec<_>>()
                    .join(" L");
                walls += &format!("<path d=\"M{d} Z\" fill=\"{color}\"/>");
            }
        }
        for line in &page.lines {
            if curve_apex(line).is_some() {
                walls += &wall_path(line, &transform, &area_line_colors, options);
            }
        }
    } else {
        for line in &page.lines {
            walls += &wall_path(line, &transform, &area_line_colors, options);
        }
    }

    // Auto-posted area labels.
    let mut area_labels = String::new();
    for poly in &page.polygons {
        let lines = auto_post_lines(poly, options, area_codes);
        if lines.is_empty() || poly.points.len() < 3 {
            continue;
        }
        let anchor_world = poly.label_position.unwrap_or_else(|| {
            let engine = crate::geometry::GeometryEngine::new();
            engine.area_label_anchor(&flattened_points(poly))
        });
        let (ax, ay) = transform(anchor_world);
        let rotation = -num_or_zero(poly.label_rotation);
        let poly_font = num_or_zero(poly.label_font_size.unwrap_or(0.0));
        let option_font = num_or_zero(options.auto_post_font_size);
        let font_size = if poly_font != 0.0 {
            poly_font
        } else if option_font != 0.0 {
            option_font
        } else {
            16.0
        }
        .max(9.0);
        let family = escape_xml(
            poly.label_font_family
                .as_deref()
                .or(Some(options.auto_post_font.as_str()))
                .unwrap_or("Arial"),
        );
        let color = escape_xml(
            poly.label_color
                .as_deref()
                .or(Some(options.text_color.as_str()))
                .unwrap_or("#111111"),
        );
        let line_spacing = font_size * 1.25;
        let mut texts = String::new();
        for (index, text) in lines.iter().enumerate() {
            // Keep extra space between the area name and its measurement.
            let gap = if options.auto_post_name && options.auto_post_calcs && index > 0 {
                6.0
            } else {
                0.0
            };
            let y = (index as f64 - (lines.len() as f64 - 1.0) / 2.0) * line_spacing + gap;
            let bold = (poly.label_bold.unwrap_or(options.auto_post_bold)) || index == 0;
            let italic = if poly.label_italic.unwrap_or(options.auto_post_italic) {
                " font-style=\"italic\""
            } else {
                ""
            };
            texts += &format!(
                "<text x=\"0\" y=\"{:.2}\" font-size=\"{}\"{italic} font-weight=\"{}\">{}</text>",
                y,
                font_size,
                if bold { "bold" } else { "normal" },
                escape_xml(text)
            );
        }
        area_labels += &format!(
            "<g transform=\"translate({:.2} {:.2}) rotate({:.2})\" fill=\"{color}\" font-family=\"{family}\" text-anchor=\"middle\" dominant-baseline=\"middle\">{texts}</g>",
            fmt2(ax),
            fmt2(ay),
            fmt2(rotation)
        );
    }

    // Free text labels.
    let mut labels = String::new();
    for label in &page.labels {
        let (x, y) = transform(Pt::new(label.x, label.y));
        let raw_size = num_or_zero(label.font_size.unwrap_or(0.0));
        let size = (if raw_size != 0.0 { raw_size } else { 12.0 }).max(7.0);
        let color = escape_xml(match &label.color {
            Some(c) if !c.eq_ignore_ascii_case("#e2e8f0") => c.as_str(),
            _ => "#111111",
        });
        let style = format!(
            "{}{}",
            if label.bold { "font-weight:bold;" } else { "" },
            if label.italic {
                "font-style:italic;"
            } else {
                ""
            }
        );
        labels += &format!(
            "<text x=\"{x:.2}\" y=\"{y:.2}\" transform=\"rotate({:.2} {x:.2} {y:.2})\" text-anchor=\"middle\" dominant-baseline=\"middle\" fill=\"{color}\" font-family=\"{}\" font-size=\"{size}\" style=\"{style}\">{}</text>",
            fmt2(-num_or_zero(label.rotation)),
            escape_xml(label.font_family.as_deref().unwrap_or("Arial")),
            escape_xml(&label.text)
        );
    }

    // Symbols drawn through the external library geometry.
    let mut symbols = String::new();
    for symbol in &page.symbols {
        let scale = PIXELS_PER_FOOT
            * (if symbol.scale != 0.0 {
                symbol.scale
            } else {
                1.0
            })
            / 12.0;
        let Some((entities, stroke)) = symbol_entities(symbol, library) else {
            continue;
        };
        let (x, y) = transform(Pt::new(symbol.x, symbol.y));
        let stroke_width = (1.5 / scale).max(0.2);
        let flip = if symbol.flip_h { " scale(-1 1)" } else { "" };
        symbols += &format!(
            "<g transform=\"translate({x:.2} {y:.2}) rotate({:.2}) scale({scale:.4}){flip}\" color=\"{stroke}\" stroke=\"currentColor\" stroke-width=\"{stroke_width:.3}\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{entities}</g>",
            fmt2(-num_or_zero(symbol.rotation))
        );
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n<!-- Generated by Appraisal Sketch 1.1. 1 unit = 1 screen pixel ({} units per foot). North: {:.1} degrees clockwise from up. -->\n{}{}{}{}{}\n</svg>",
        fmt0(width),
        fmt0(height),
        fmt0(width),
        fmt0(height),
        PIXELS_PER_FOOT as i64,
        page.north_angle,
        polygons,
        walls,
        area_labels,
        labels,
        symbols
    )
}

fn fmt0(v: f64) -> String {
    format!("{v:.0}")
}

/// Unused helper kept for symmetry with the browser's `Entity` import.
#[allow(dead_code)]
fn entity_is_line(entity: &Entity) -> bool {
    entity.entity_type == EntityType::Line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_model::{ExportLabel, ExportLine, ExportPage, ExportPolygon};
    use crate::geometry::Edge;

    fn rect_poly(x0: f64, y0: f64, x1: f64, y1: f64) -> ExportPolygon {
        ExportPolygon {
            points: vec![
                Pt::new(x0, y0),
                Pt::new(x1, y0),
                Pt::new(x1, y1),
                Pt::new(x0, y1),
            ],
            edges: vec![Edge::default(); 4],
            area_type: "LIV".to_owned(),
            name: "Living Area".to_owned(),
            net: Some(100.0),
            area: 100.0,
            gross: 100.0,
            ..Default::default()
        }
    }

    #[test]
    fn svg_well_formed_with_viewbox_and_pixel_comment() {
        let page = ExportPage {
            polygons: vec![rect_poly(0.0, 0.0, 10.0, 8.0)],
            ..Default::default()
        };
        let options = ExportOptions::default();
        let svg = export_svg(&page, &options, &[], None);
        assert!(svg.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(svg.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        // 10x8 ft * 20 px/ft + 40*2 margin = 280 x 240.
        assert!(svg.contains("width=\"280\" height=\"240\""));
        assert!(svg.contains("viewBox=\"0 0 280 240\""));
        assert!(svg.contains("20 units per foot"));
        assert_eq!(svg.matches("<svg").count(), 1);
        assert!(svg.trim_end().ends_with("</svg>"));
    }

    #[test]
    fn svg_polygon_fill_and_label_present() {
        let page = ExportPage {
            polygons: vec![rect_poly(0.0, 0.0, 10.0, 8.0)],
            ..Default::default()
        };
        let options = ExportOptions::default();
        let svg = export_svg(&page, &options, &[], None);
        assert!(svg.contains("fill-opacity=\"0.10\""));
        assert!(svg.contains("Living Area"));
        assert!(svg.contains("100 sf"));
        // Default opacity 10 -> 0.10; polygon path is closed.
        assert!(svg.contains(" Z\""));
    }

    #[test]
    fn svg_wall_dash_and_opening_runs() {
        let line = ExportLine {
            id: 1,
            start: Pt::new(0.0, 0.0),
            end: Pt::new(20.0, 0.0),
            wall_type: "exterior".to_owned(),
            color: Some("#FF0000".to_owned()),
            line_thickness: 4.0,
            line_style: Some("dashed".to_owned()),
            bow_angle: 0.0,
            openings: vec![(0.0, 0.25), (0.75, 1.0)],
        };
        let page = ExportPage {
            lines: vec![line],
            ..Default::default()
        };
        let options = ExportOptions::default();
        let svg = export_svg(&page, &options, &[], None);
        assert!(svg.contains("stroke=\"#FF0000\""));
        assert!(svg.contains("stroke-width=\"4\""));
        assert!(svg.contains("stroke-dasharray=\"10 6\""));
        // Two opening runs -> two M/L segments.
        assert_eq!(svg.matches("stroke-width=\"4\"").count(), 1);
        let path = svg
            .split("<path d=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .expect("first path is the wall");
        assert_eq!(path.matches("M").count(), 2);
    }

    #[test]
    fn svg_bowed_wall_becomes_arc() {
        let line = ExportLine {
            id: 1,
            start: Pt::new(0.0, 0.0),
            end: Pt::new(20.0, 0.0),
            wall_type: "exterior".to_owned(),
            color: None,
            line_thickness: 4.0,
            line_style: None,
            bow_angle: 30.0,
            openings: vec![],
        };
        let page = ExportPage {
            lines: vec![line],
            ..Default::default()
        };
        let options = ExportOptions::default();
        let svg = export_svg(&page, &options, &[], None);
        let path = svg
            .split("<path d=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .expect("wall path");
        assert!(path.starts_with("M"));
        assert!(
            path.contains('A'),
            "bowed wall exports an A command: {path}"
        );
    }

    #[test]
    fn svg_labels_escaped() {
        let page = ExportPage {
            labels: vec![ExportLabel {
                x: 5.0,
                y: 5.0,
                text: "a<b & \"c\"".to_owned(),
                font_size: None,
                rotation: 0.0,
                color: None,
                font_family: None,
                bold: true,
                italic: false,
            }],
            ..Default::default()
        };
        let options = ExportOptions::default();
        let svg = export_svg(&page, &options, &[], None);
        assert!(svg.contains("a&lt;b &amp; &quot;c&quot;"));
        assert!(svg.contains("font-weight:bold;"));
        assert!(!svg.contains("<b"));
    }
}
