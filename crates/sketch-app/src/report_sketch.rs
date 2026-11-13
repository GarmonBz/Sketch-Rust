//! The HTML report's SVG drawing rules, rendered at 300 DPI for PDF printing.
//! Unlike the interactive export, report strokes and text sizes are in points.
use crate::drawing::Drawing;
use sketch_core::{
    export_model::{escape_xml as esc, ExportPage},
    geometry::{Edge, Pt},
};

pub fn svg(
    d: &Drawing,
    page: &ExportPage,
    codes: &[sketch_core::geometry::AreaCode],
    library: &sketch_core::symbols::SymbolLibrary,
    height: f64,
    comment_inset: f64,
) -> String {
    // Keep chains, moved text and symbols inside the printable sketch box.
    let mut bounds = page.clone();
    let mut include = |p: Pt| {
        bounds.lines.push(sketch_core::export_model::ExportLine {
            id: 0,
            start: p,
            end: p,
            wall_type: String::new(),
            color: None,
            line_thickness: 0.0,
            line_style: None,
            bow_angle: 0.0,
            openings: Vec::new(),
        })
    };
    for chain in &d.dimension_chains {
        if let Some(computed) = d.compute_chain(chain) {
            let tiers = crate::chains::ChainTiers::solve(computed, chain);
            for at in [0.0, tiers.computed.length] {
                include(tiers.point_at(at, 0.0));
                if chain.show_overall {
                    include(tiers.point_at(at, tiers.overall_extra()));
                }
            }
        }
    }
    for label in &d.texts {
        include(Pt::new(label.x, label.y));
    }
    for symbol in &d.symbols {
        include(Pt::new(symbol.x, symbol.y));
        if let Some(def) = library.symbol(&symbol.symbol_type) {
            for p in sketch_core::symbols::world_corners(
                def.bounds,
                symbol.x,
                symbol.y,
                symbol.rotation,
                symbol.scale,
                symbol.flip_h,
            ) {
                include(p);
            }
        }
    }
    for poly in &d.polygons {
        if let Some(p) = poly.label_position {
            include(p);
        }
        if let Some(p) = poly.area_size_position {
            include(p);
        }
    }
    let fit = sketch_core::print_layout::make_transform(&bounds, height - comment_inset);
    let point = |p: Pt| fit.point(p);
    let path = |points: &[Pt], closed: bool| {
        let mut out = String::new();
        for (i, p) in points.iter().enumerate() {
            let (x, y) = point(*p);
            out += &format!("{} {x:.4} {y:.4} ", if i == 0 { "M" } else { "L" });
        }
        if closed {
            out += "Z";
        }
        out
    };
    let text = |at: (f64, f64),
                value: &str,
                size: f64,
                rotation: f64,
                family: &str,
                color: &str,
                bold: bool,
                italic: bool| {
        format!("<text x=\"{:.4}\" y=\"{:.4}\" transform=\"rotate({rotation:.4} {:.4} {:.4})\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"{}\" font-size=\"{size:.4}\" font-weight=\"{}\" font-style=\"{}\" fill=\"{}\">{}</text>",at.0,at.1,at.0,at.1,esc(family),if bold{"bold"}else{"normal"},if italic{"italic"}else{"normal"},esc(color),esc(value))
    };
    let colors = sketch_core::export_model::build_area_line_colors(page, codes);
    let mut body = String::new();
    for p in &page.polygons {
        if p.points.len() < 3 {
            continue;
        }
        let def = codes.iter().find(|c| c.code == p.area_type);
        let ink = p
            .line_color
            .as_deref()
            .or(def.map(|c| c.color.as_str()))
            .unwrap_or("#111111");
        let fill = p
            .fill_color
            .as_deref()
            .or(def.map(|c| c.fill_color.as_str()))
            .unwrap_or("#ffffff");
        let opacity = p
            .fill_opacity
            .or(def.map(|c| c.fill_opacity))
            .unwrap_or(10.0)
            .clamp(0.0, 100.0)
            / 100.0;
        let style = p
            .fill_pattern
            .as_deref()
            .or(def.map(|c| c.fill_style.as_str()))
            .unwrap_or("solid");
        let fill = match style {
            "none" => "none".to_string(),
            _ if p.area_type == "UND" => "none".into(),
            "diagonal" => "url(#diag)".into(),
            "crosshatch" => "url(#cross)".into(),
            "dots" => "url(#dots)".into(),
            _ => esc(fill),
        };
        body+=&format!("<path d=\"{}\" fill=\"{fill}\" fill-opacity=\"{}\" color=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>",path(&sketch_core::export_model::flattened_points(p),true),if style=="solid"{opacity}else{1.0},esc(ink),esc(ink),p.line_thickness.unwrap_or(0.8).max(0.65));
    }
    let openings = crate::placed_symbol::openings(&d.symbols);
    let solids = if d.options.wall_solids_enabled() {
        Some(sketch_core::solids::compute_wall_solids(
            &d.lines,
            &|l| d.wall_thickness_of(l),
            &openings,
        ))
    } else {
        None
    };
    if let Some(solids) = &solids {
        for solid in &solids.solids {
            let Some(line) = d.lines.iter().find(|l| l.id == solid.line_id) else {
                continue;
            };
            let color = colors
                .get(&line.id)
                .map(String::as_str)
                .or(line.color.as_deref())
                .unwrap_or("#111111");
            for piece in &solid.pieces {
                body += &format!(
                    "<path d=\"{}\" fill=\"{}\"/>",
                    path(piece, true),
                    esc(color)
                );
            }
        }
    }
    let covered = d.chain_coverage();
    for line in &d.lines {
        let color = colors
            .get(&line.id)
            .map(String::as_str)
            .or(line.color.as_deref())
            .unwrap_or("#111111");
        if !solids
            .as_ref()
            .is_some_and(|s| s.solids.iter().any(|s| s.line_id == line.id))
        {
            let points = d.engine().flatten_ring(
                &[line.start, line.end],
                &[
                    Edge {
                        line_id: Some(line.id),
                        bow_angle: line.bow_angle,
                    },
                    Edge::default(),
                ],
                0.005,
            );
            let width = line
                .line_thickness
                .unwrap_or(if line.kind == sketch_core::model::WallType::Exterior {
                    2.0
                } else {
                    1.0
                })
                .max(if line.kind == sketch_core::model::WallType::Exterior {
                    1.3
                } else {
                    0.65
                });
            let dash = match line.line_style.as_deref() {
                Some("dashed") => "8 5",
                Some("dotted") => "1.5 3.5",
                _ => "",
            };
            body+=&format!("<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{width}\" stroke-dasharray=\"{dash}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",{
                let runs=sketch_core::solids::visible_runs(line,&openings);
                if runs.is_empty(){path(&points,false)}else{runs.iter().filter(|(a,b)|b>a).map(|(a,b)|path(&[Pt::new(line.start.x+(line.end.x-line.start.x)*a,line.start.y+(line.end.y-line.start.y)*a),Pt::new(line.start.x+(line.end.x-line.start.x)*b,line.start.y+(line.end.y-line.start.y)*b)],false)).collect::<String>()}
            },esc(color));
        }
        let length = sketch_core::geometry::distance(line.start, line.end);
        if !line.dimension_hidden && !covered.contains(&line.id) && length >= 0.1 {
            let (a, b) = (point(line.start), point(line.end));
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let len = dx.hypot(dy).max(1.0);
            let side = f64::from(crate::render::line_dimension_side(
                d,
                line,
                crate::view_state::DimensionPlacement::Out,
            ));
            let at = crate::drawing::pinned_dimension_point(line)
                .map(point)
                .unwrap_or((
                    (a.0 + b.0) / 2.0 - dy / len * 10.0 * side,
                    (a.1 + b.1) / 2.0 + dx / len * 10.0 * side,
                ));
            let mut angle = dy.atan2(dx).to_degrees();
            if angle > 90.0 {
                angle -= 180.0;
            }
            if angle < -90.0 {
                angle += 180.0;
            }
            body += &text(
                at,
                &format!("{length:.1}'"),
                7.98,
                angle,
                "Arial",
                color,
                false,
                false,
            );
        }
    }
    // Chain witness/tick geometry and label sizes match renderChains in points.
    for chain in &d.dimension_chains {
        let Some(computed) = d.compute_chain(chain) else {
            continue;
        };
        let tiers = crate::chains::ChainTiers::solve(computed, chain);
        let refs = &tiers.computed.refs;
        if refs.len() < 2 {
            continue;
        }
        let (a, b) = (
            point(tiers.point_at(0.0, 0.0)),
            point(tiers.point_at(tiers.computed.length, 0.0)),
        );
        let len = (b.0 - a.0).hypot(b.1 - a.1).max(1.0);
        let (mut nx, mut ny) = (-(b.1 - a.1) / len, (b.0 - a.0) / len);
        let wall = point(tiers.wall_at(0.0));
        if nx * (wall.0 - a.0) + ny * (wall.1 - a.1) >= 0.0 {
            nx = -nx;
            ny = -ny;
        }
        let segment = |a: (f64, f64), b: (f64, f64)| {
            format!("<path d=\"M {:.3} {:.3} L {:.3} {:.3}\" stroke=\"#111\" stroke-width=\".6\" fill=\"none\"/>",a.0,a.1,b.0,b.1)
        };
        let tick = |p: (f64, f64)| segment((p.0 - 3.0, p.1 + 3.0), (p.0 + 3.0, p.1 - 3.0));
        body += &segment(a, b);
        for r in refs {
            let wall = point(tiers.wall_at(*r));
            let at = point(tiers.point_at(*r, 0.0));
            body += &segment(
                (wall.0 + nx * 2.0, wall.1 + ny * 2.0),
                (at.0 + nx * 3.0, at.1 + ny * 3.0),
            );
            body += &tick(at);
        }
        let mut angle = (b.1 - a.1).atan2(b.0 - a.0).to_degrees();
        if angle > 90.0 {
            angle -= 180.0;
        }
        if angle < -90.0 {
            angle += 180.0;
        }
        for (i, span) in refs.windows(2).enumerate() {
            let length = span[1] - span[0];
            if chain.hidden_spans.contains(&(i as i64))
                || length < 0.05
                || (d.options.minimum_dimension_enabled && length < d.options.minimum_dimension)
            {
                continue;
            }
            let at = point(tiers.point_at((span[0] + span[1]) / 2.0, 0.0));
            body += &text(
                (at.0 + nx * 7.0, at.1 + ny * 7.0),
                &format!("{length:.1}'"),
                7.98,
                angle,
                "Arial",
                "#111111",
                false,
                false,
            );
        }
        if chain.show_overall {
            let extra = tiers.overall_extra();
            let first = point(tiers.point_at(0.0, extra));
            let last = point(tiers.point_at(tiers.computed.length, extra));
            for r in [0.0, tiers.computed.length] {
                let inner = point(tiers.point_at(r, 0.0));
                let outer = point(tiers.point_at(r, extra));
                body += &segment(
                    (inner.0 + nx * 1.5, inner.1 + ny * 1.5),
                    (outer.0 + nx * 3.0, outer.1 + ny * 3.0),
                );
            }
            body += &segment(first, last);
            body += &tick(first);
            body += &tick(last);
            body += &text(
                (
                    (first.0 + last.0) / 2.0 + nx * 7.0,
                    (first.1 + last.1) / 2.0 + ny * 7.0,
                ),
                &format!("{:.1}'", tiers.computed.length),
                7.98,
                angle,
                "Arial",
                "#111111",
                true,
                false,
            );
        }
    }
    for poly in &d.polygons {
        if poly.kind == "UND" || poly.points.len() < 3 {
            continue;
        }
        let label = crate::handles::label_position(d, poly);
        let scale = poly.label_scale.unwrap_or(1.0).clamp(0.5, 4.0);
        let size_scale = poly.area_size_scale.unwrap_or(scale).clamp(0.5, 4.0);
        let size_at = if poly.area_size_position.is_some() {
            point(crate::handles::size_position(d, poly))
        } else {
            // Report fonts use physical points; the interactive label offset
            // uses world units and can collapse when a large sketch is fitted.
            let at = point(label);
            let angle = -poly.label_rotation.unwrap_or(0.).to_radians();
            let gap = 16. * scale.max(size_scale);
            (at.0 - angle.sin() * gap, at.1 + angle.cos() * gap)
        };
        let family = poly.label_font_family.as_deref().unwrap_or("Arial");
        let color = poly.label_color.as_deref().unwrap_or("#111111");
        let name = if poly.name.is_empty() {
            codes
                .iter()
                .find(|c| c.code == poly.kind)
                .map(|c| c.name.as_str())
                .unwrap_or(&poly.kind)
        } else {
            &poly.name
        };
        let net = page
            .polygons
            .iter()
            .find(|p| p.name == poly.name && p.points == poly.points)
            .and_then(|p| p.net)
            .unwrap_or(poly.area);
        body += &text(
            point(label),
            name,
            9.72 * scale,
            -poly.label_rotation.unwrap_or(0.0),
            family,
            color,
            poly.label_bold != Some(false),
            poly.label_italic.unwrap_or(false),
        );
        body += &text(
            size_at,
            &format!("{net:.1} sf"),
            9.72 * size_scale,
            -poly
                .area_size_rotation
                .or(poly.label_rotation)
                .unwrap_or(0.0),
            family,
            color,
            poly.label_bold != Some(false),
            poly.label_italic.unwrap_or(false),
        );
    }
    for label in &d.texts {
        body += &text(
            point(Pt::new(label.x, label.y)),
            &label.text,
            (label.font_size * 0.78).clamp(7.0, 16.0),
            -label.rotation,
            &label.font_family,
            if label.color.is_empty() || label.color.eq_ignore_ascii_case("#e2e8f0") {
                "#111111"
            } else {
                &label.color
            },
            label.bold,
            label.italic,
        );
    }
    for symbol in &d.symbols {
        let export = sketch_core::export_model::ExportSymbol {
            x: symbol.x,
            y: symbol.y,
            symbol_type: symbol.symbol_type.clone(),
            rotation: symbol.rotation,
            scale: symbol.scale,
            flip_h: symbol.flip_h,
            color: Some("#111111".into()),
        };
        if let Some((entities, color)) =
            sketch_core::export_svg::symbol_entities(&export, Some(library))
        {
            let (x, y) = point(Pt::new(symbol.x, symbol.y));
            let scale = fit.scale * symbol.scale.max(0.001) / 12.0;
            body+=&format!("<g transform=\"translate({x:.4} {y:.4}) rotate({:.4}) scale({:.6} {scale:.6})\" color=\"{}\" stroke=\"currentColor\" stroke-width=\"{:.4}\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{entities}</g>",-symbol.rotation,if symbol.flip_h{-scale}else{scale},esc(&color),(0.55/scale).max(0.15));
        }
    }
    let north = page.north_angle.rem_euclid(360.0);
    body+=&format!("<g transform=\"translate(516 24) rotate({north:.2})\"><path d=\"M0 -12 L4.2 6 L0 2.4 L-4.2 6 Z\" fill=\"#111\"/><circle r=\"14\" fill=\"none\" stroke=\"#111\" stroke-width=\".6\"/></g>");
    body += &text(
        (516.0, 46.0),
        &if north == 0.0 {
            "N".into()
        } else {
            format!("N ({north:.0}°)")
        },
        6.5,
        0.0,
        "Arial",
        "#111111",
        false,
        false,
    );
    format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"540\" height=\"{height}\" viewBox=\"0 0 540 {height}\"><rect width=\"540\" height=\"{height}\" fill=\"white\"/><defs><pattern id=\"diag\" width=\"7\" height=\"7\" patternUnits=\"userSpaceOnUse\" patternTransform=\"rotate(45)\"><path d=\"M0 0V7\" stroke=\"#111\" stroke-width=\".45\"/></pattern><pattern id=\"cross\" width=\"7\" height=\"7\" patternUnits=\"userSpaceOnUse\"><path d=\"M0 0L7 7M7 0L0 7\" stroke=\"#111\" stroke-width=\".4\"/></pattern><pattern id=\"dots\" width=\"7\" height=\"7\" patternUnits=\"userSpaceOnUse\"><circle cx=\"2\" cy=\"2\" r=\".6\" fill=\"#111\"/></pattern></defs>{body}</svg>")
}

pub fn raster(svg: &str) -> Option<sketch_core::export_pdf::ReportSketch> {
    use std::sync::{Arc, OnceLock};
    static FONTS: OnceLock<Arc<resvg::usvg::fontdb::Database>> = OnceLock::new();
    let fonts = FONTS.get_or_init(|| {
        let mut db = resvg::usvg::fontdb::Database::new();
        db.load_system_fonts();
        Arc::new(db)
    });
    let options = resvg::usvg::Options {
        fontdb: fonts.clone(),
        font_family: "Arial".into(),
        ..Default::default()
    };
    let tree = resvg::usvg::Tree::from_str(svg, &options).ok()?;
    let scale = 300.0 / 72.0;
    let width = (tree.size().width() * scale).ceil() as u32;
    let height = (tree.size().height() * scale).ceil() as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    let rgb = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let a = p.alpha();
            [
                p.red().saturating_add(255 - a),
                p.green().saturating_add(255 - a),
                p.blue().saturating_add(255 - a),
            ]
        })
        .collect();
    Some(sketch_core::export_pdf::ReportSketch { width, height, rgb })
}
