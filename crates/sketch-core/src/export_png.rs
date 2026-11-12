//! PNG export — clean-room port of `SketchApp.exportPNG`
//! (`00) sketch standalone.html`, ~line 19874): an off-screen raster at the
//! chosen export resolution (default 2048x1536, `options.imageSize`), all
//! sketch content fit into it preserving aspect ratio with a 60-unit margin,
//! zoom clamped 0.1..10, drawn on a TRANSPARENT canvas (`renderTo(canvas,
//! { transparent: true, clean: true })`).
//!
//! Rasterized through tiny-skia with the same flattened geometry the canvas
//! paints: polygon fills/outlines, wall strokes (bowed walls flattened
//! through `GeometryEngine::flatten_ring`), and solid pieces when geometric
//! wall thickness is enabled. Text is not rastered here — the core has no
//! font dependency; `render_png` hands back the pixmap and the fit so the
//! app can lay its labels over it before `encode_png`.

use crate::export_model::{
    build_area_line_colors, curve_apex, flattened_points, num_or_zero, ExportLine, ExportOptions,
    ExportPage, ExportSolid,
};
use crate::geometry::{AreaCode, Pt, PIXELS_PER_FOOT};
use std::collections::BTreeMap;

/// The rasterizer the exports draw with, for callers that paint over the
/// pixmap `render_png` returns.
pub use tiny_skia;

/// Default export resolution (`options.imageSize` default '2048x1536').
pub const DEFAULT_IMAGE_SIZE: (u32, u32) = (2048, 1536);
/// Fit margin in pixels (`const margin = 60`).
pub const PNG_MARGIN: f64 = 60.0;
/// Zoom clamp (`Math.max(0.1, Math.min(10, ...))`).
const MIN_ZOOM: f64 = 0.1;
const MAX_ZOOM: f64 = 10.0;

/// Parse an `options.imageSize` 'WxH' string; unknown values fall back to
/// 2048x1536.
pub fn parse_image_size(size: Option<&str>) -> (u32, u32) {
    let Some(size) = size else {
        return DEFAULT_IMAGE_SIZE;
    };
    let mut parts = size.split('x');
    let (Some(w), Some(h)) = (parts.next(), parts.next()) else {
        return DEFAULT_IMAGE_SIZE;
    };
    match (w.trim().parse::<u32>(), h.trim().parse::<u32>()) {
        (Ok(w), Ok(h)) if w > 0 && h > 0 => (w, h),
        _ => DEFAULT_IMAGE_SIZE,
    }
}

/// The fit transform the PNG uses: content bounds (line endpoints, polygon
/// rings, labels, symbols — `getExportBounds`), margin 60, min-fit zoom
/// clamped 0.1..10, centered. y-down: y = H/2 + cy·PPF·z − y·PPF·z.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PngFit {
    pub target: (u32, u32),
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
}

impl PngFit {
    pub fn point(&self, world: Pt) -> (f64, f64) {
        (
            self.pan_x + world.x * PIXELS_PER_FOOT * self.zoom,
            self.pan_y - world.y * PIXELS_PER_FOOT * self.zoom,
        )
    }
}

/// Compute the PNG fit from the export bounds (`exportPNG`'s hasContent
/// branch). Empty bounds center the origin at zoom 1.
pub fn png_fit(target: (u32, u32), bounds_min: Pt, bounds_max: Pt, has_content: bool) -> PngFit {
    let (tw, th) = (f64::from(target.0), f64::from(target.1));
    if !has_content {
        return PngFit {
            target,
            zoom: 1.0,
            pan_x: tw / 2.0,
            pan_y: th / 2.0,
        };
    }
    let world_w = (bounds_max.x - bounds_min.x).max(1.0);
    let world_h = (bounds_max.y - bounds_min.y).max(1.0);
    let zoom_x = (tw - PNG_MARGIN * 2.0) / (world_w * PIXELS_PER_FOOT);
    let zoom_y = (th - PNG_MARGIN * 2.0) / (world_h * PIXELS_PER_FOOT);
    let zoom = zoom_x.min(zoom_y).clamp(MIN_ZOOM, MAX_ZOOM);
    let cx = (bounds_min.x + bounds_max.x) / 2.0;
    let cy = (bounds_min.y + bounds_max.y) / 2.0;
    PngFit {
        target,
        zoom,
        pan_x: tw / 2.0 - cx * PIXELS_PER_FOOT * zoom,
        pan_y: th / 2.0 + cy * PIXELS_PER_FOOT * zoom,
    }
}

fn parse_hex_color(color: &str) -> Option<[u8; 3]> {
    let hex = color.trim().trim_start_matches('#');
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}

fn wall_width(line: &ExportLine, options: &ExportOptions) -> f32 {
    let base = if line.line_thickness != 0.0 {
        line.line_thickness
    } else if line.is_exterior() {
        options.exterior_width
    } else {
        options.interior_width
    };
    (if base != 0.0 {
        base
    } else if line.is_exterior() {
        4.0
    } else {
        2.0
    }) as f32
}

fn wall_color(
    line: &ExportLine,
    area_line_colors: &BTreeMap<i64, String>,
    options: &ExportOptions,
) -> tiny_skia::Color {
    let color = area_line_colors
        .get(&line.id)
        .cloned()
        .or_else(|| line.color.clone())
        .unwrap_or_else(|| {
            if line.is_exterior() {
                options.exterior_color.clone()
            } else {
                options.interior_color.clone()
            }
        });
    parse_hex_color(&color)
        .map(|[r, g, b]| tiny_skia::Color::from_rgba8(r, g, b, 255))
        .unwrap_or(tiny_skia::Color::from_rgba8(0x11, 0x11, 0x11, 255))
}

/// Split a polyline into dash runs, `on`/`off` alternating in px.
fn dash_runs(points: &[(f64, f64)], on: f64, off: f64) -> Vec<Vec<(f64, f64)>> {
    let mut runs = Vec::new();
    let mut current: Vec<(f64, f64)> = Vec::new();
    let mut remaining = on;
    let mut on_dash = true;
    for pair in points.windows(2) {
        let (ax, ay) = pair[0];
        let (bx, by) = pair[1];
        let mut seg = (bx - ax).hypot(by - ay);
        let mut x = ax;
        let mut y = ay;
        let dx = (bx - ax) / seg;
        let dy = (by - ay) / seg;
        while seg > 1e-9 {
            let take = remaining.min(seg);
            let nx = x + dx * take;
            let ny = y + dy * take;
            if on_dash {
                if current.is_empty() {
                    current.push((x, y));
                }
                current.push((nx, ny));
            } else if !current.is_empty() {
                runs.push(std::mem::take(&mut current));
            }
            x = nx;
            y = ny;
            seg -= take;
            remaining -= take;
            if remaining <= 1e-9 {
                if on_dash && current.len() > 1 {
                    runs.push(std::mem::take(&mut current));
                }
                current = Vec::new();
                on_dash = !on_dash;
                remaining = if on_dash { on } else { off };
            }
        }
    }
    if current.len() > 1 {
        runs.push(current);
    }
    runs
}

fn stroke_path(
    pixmap: &mut tiny_skia::Pixmap,
    points: &[(f64, f64)],
    width: f32,
    color: tiny_skia::Color,
    dash: Option<(f64, f64)>,
) {
    if points.len() < 2 {
        return;
    }
    let segments: Vec<Vec<(f64, f64)>> = match dash {
        Some((on, off)) => dash_runs(points, on, off),
        None => vec![points.to_vec()],
    };
    for segment in &segments {
        if segment.len() < 2 {
            continue;
        }
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(segment[0].0 as f32, segment[0].1 as f32);
        for p in &segment[1..] {
            pb.line_to(p.0 as f32, p.1 as f32);
        }
        if let Some(path) = pb.finish() {
            let mut stroke = tiny_skia::Stroke {
                width: width.max(0.5),
                ..Default::default()
            };
            stroke.line_cap = tiny_skia::LineCap::Round;
            stroke.line_join = tiny_skia::LineJoin::Round;
            let paint = tiny_skia::Paint {
                shader: tiny_skia::Shader::SolidColor(color),
                ..Default::default()
            };
            pixmap.stroke_path(
                &path,
                &paint,
                &stroke,
                tiny_skia::Transform::identity(),
                None,
            );
        }
    }
}

/// Dash lengths in px per the canvas style (`dashed 8 5, dotted 1.5 3.5`).
fn dash_of(line: &ExportLine) -> Option<(f64, f64)> {
    match line.line_style.as_deref() {
        Some("dashed") => Some((8.0, 5.0)),
        Some("dotted") => Some((1.5, 3.5)),
        _ => None,
    }
}

/// `exportPNG` — raster the page's flattened geometry at the target size on
/// a transparent background. Returns the encoded PNG bytes.
pub fn export_png(
    page: &ExportPage,
    options: &ExportOptions,
    area_codes: &[AreaCode],
    target: (u32, u32),
) -> Option<Vec<u8>> {
    export_png_with_backdrop(page, options, area_codes, target, None)
}

/// The reference image `exportPNG` includes when `options.saveImageWithSketch`
/// is on and the image is visible (`drawBgImage` runs even in a clean render):
/// straight-alpha RGBA pixels plus the image's placement (`bgImage`).
#[derive(Debug, Clone)]
pub struct ExportBackdrop {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// World centre, feet.
    pub x: f64,
    pub y: f64,
    /// `bgImage.scale`: canvas pixels per image pixel at zoom 1.
    pub scale: f64,
    /// Degrees.
    pub rotation: f64,
    pub opacity: f64,
    pub flip_h: bool,
    pub flip_v: bool,
}

/// `getExportBounds`' reference-image term: the image's centre grown by its
/// half-diagonal, with the original's `img.width / scale` terms.
fn backdrop_bounds(b: &ExportBackdrop) -> (Pt, Pt) {
    let half_w = (f64::from(b.width) / b.scale) / 2.0;
    let half_h = (f64::from(b.height) / b.scale) / 2.0;
    let half_diag = half_w.hypot(half_h);
    (
        Pt::new(b.x - half_diag, b.y - half_diag),
        Pt::new(b.x + half_diag, b.y + half_diag),
    )
}

/// `exportPNG` with the optional reference image drawn beneath the sketch.
pub fn export_png_with_backdrop(
    page: &ExportPage,
    options: &ExportOptions,
    area_codes: &[AreaCode],
    target: (u32, u32),
    backdrop: Option<&ExportBackdrop>,
) -> Option<Vec<u8>> {
    let (pixmap, _) = render_png(page, options, area_codes, target, backdrop)?;
    encode_png(&pixmap)
}

/// `exportPNG`'s raster before encoding, with the fit it was drawn through
/// (so text can be placed in the same pixels).
pub fn fit_page(
    page: &ExportPage,
    target: (u32, u32),
    backdrop: Option<&ExportBackdrop>,
) -> PngFit {
    fit_page_with_library(page, target, backdrop, None)
}

pub fn fit_page_with_library(
    page: &ExportPage,
    target: (u32, u32),
    backdrop: Option<&ExportBackdrop>,
    library: Option<&crate::symbols::SymbolLibrary>,
) -> PngFit {
    let (mut bounds_min, mut bounds_max) =
        crate::export_model::get_export_bounds_with_library(page, library);
    let mut has_content = !page.lines.is_empty()
        || !page.polygons.is_empty()
        || !page.labels.is_empty()
        || !page.symbols.is_empty();
    if let Some(b) = backdrop {
        // The empty-bounds fallback is (0,0)-(0,0); the image then defines it.
        let (lo, hi) = backdrop_bounds(b);
        if has_content {
            bounds_min = Pt::new(bounds_min.x.min(lo.x), bounds_min.y.min(lo.y));
            bounds_max = Pt::new(bounds_max.x.max(hi.x), bounds_max.y.max(hi.y));
        } else {
            bounds_min = lo;
            bounds_max = hi;
        }
        has_content = true;
    }
    png_fit(target, bounds_min, bounds_max, has_content)
}

pub fn render_png(
    page: &ExportPage,
    options: &ExportOptions,
    area_codes: &[AreaCode],
    target: (u32, u32),
    backdrop: Option<&ExportBackdrop>,
) -> Option<(tiny_skia::Pixmap, PngFit)> {
    let fit = fit_page(page, target, backdrop);
    let mut pixmap = tiny_skia::Pixmap::new(target.0, target.1)?;
    if let Some(b) = backdrop {
        draw_backdrop(&mut pixmap, &fit, b);
    }
    let area_line_colors = build_area_line_colors(page, area_codes);

    // Polygon fills then outlines.
    for poly in &page.polygons {
        if poly.points.len() < 3 {
            continue;
        }
        let screen: Vec<(f64, f64)> = flattened_points(poly)
            .iter()
            .map(|p| fit.point(*p))
            .collect();
        let outline = poly
            .line_color
            .clone()
            .or_else(|| {
                area_codes
                    .iter()
                    .find(|c| c.code == poly.area_type)
                    .map(|c| c.color.clone())
            })
            .unwrap_or_else(|| "#111111".to_owned());
        let fill = poly
            .fill_color
            .clone()
            .or_else(|| {
                area_codes
                    .iter()
                    .find(|c| c.code == poly.area_type)
                    .map(|c| c.fill_color.clone())
                    .filter(|s| !s.is_empty())
            })
            .or_else(|| {
                area_codes
                    .iter()
                    .find(|c| c.code == poly.area_type)
                    .map(|c| c.color.clone())
            })
            .unwrap_or_else(|| "#ffffff".to_owned());
        let style = poly.fill_pattern.as_deref().unwrap_or("solid");
        let fill_alpha = if style == "none" || poly.area_type == "UND" {
            0.0
        } else {
            num_or_zero(poly.fill_opacity.unwrap_or(10.0)).clamp(0.0, 100.0) / 100.0
        };
        if fill_alpha > 0.0 {
            let [r, g, b] = parse_hex_color(&fill).unwrap_or([255, 255, 255]);
            let Some(fill_color) = tiny_skia::Color::from_rgba(
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
                fill_alpha as f32,
            ) else {
                continue;
            };
            let mut pb = tiny_skia::PathBuilder::new();
            pb.move_to(screen[0].0 as f32, screen[0].1 as f32);
            for p in &screen[1..] {
                pb.line_to(p.0 as f32, p.1 as f32);
            }
            pb.close();
            if let Some(path) = pb.finish() {
                let paint = tiny_skia::Paint {
                    shader: tiny_skia::Shader::SolidColor(fill_color),
                    ..Default::default()
                };
                pixmap.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    tiny_skia::Transform::identity(),
                    None,
                );
            }
        }
        let thickness = num_or_zero(poly.line_thickness.unwrap_or(0.0));
        let stroke_width = (if thickness != 0.0 { thickness } else { 1.5 }).max(1.0) as f32;
        let [r, g, b] = parse_hex_color(&outline).unwrap_or([0x11, 0x11, 0x11]);
        stroke_path(
            &mut pixmap,
            &screen,
            stroke_width,
            tiny_skia::Color::from_rgba8(r, g, b, 255),
            None,
        );
    }

    // Wall solids when geometric thickness is enabled.
    if options.wall_solids {
        let by_id = |line_id: i64| page.lines.iter().find(|l| l.id == line_id);
        for solid in &page.wall_solids {
            let Some(line) = by_id(solid.line_id) else {
                continue;
            };
            let color = wall_color(line, &area_line_colors, options);
            for piece in &solid.pieces {
                let screen: Vec<(f64, f64)> = piece.iter().map(|p| fit.point(*p)).collect();
                let mut pb = tiny_skia::PathBuilder::new();
                pb.move_to(screen[0].0 as f32, screen[0].1 as f32);
                for p in &screen[1..] {
                    pb.line_to(p.0 as f32, p.1 as f32);
                }
                pb.close();
                if let Some(path) = pb.finish() {
                    let paint = tiny_skia::Paint {
                        shader: tiny_skia::Shader::SolidColor(color),
                        ..Default::default()
                    };
                    pixmap.fill_path(
                        &path,
                        &paint,
                        tiny_skia::FillRule::Winding,
                        tiny_skia::Transform::identity(),
                        None,
                    );
                }
            }
        }
    }

    // Wall strokes: bowed walls flatten through the ring flattener, straight
    // walls break through their opening runs.
    for line in &page.lines {
        let color = wall_color(line, &area_line_colors, options);
        let width = wall_width(line, options);
        let dash = dash_of(line);
        let apex = curve_apex(line);
        let screen: Vec<(f64, f64)> = if apex.is_some() {
            let faux_poly = crate::export_model::ExportPolygon {
                points: vec![line.start, line.end],
                edges: vec![
                    crate::geometry::Edge {
                        line_id: Some(line.id),
                        bow_angle: line.bow_angle,
                    },
                    crate::geometry::Edge::default(),
                ],
                ..Default::default()
            };
            flattened_points(&faux_poly)
                .iter()
                .map(|p| fit.point(*p))
                .collect()
        } else {
            let runs: Vec<(f64, f64)> = if line.openings.is_empty() {
                vec![(0.0, 1.0)]
            } else {
                line.openings.clone()
            };
            let mut pts = Vec::new();
            for (start, end) in runs {
                let a = Pt::new(
                    line.start.x + (line.end.x - line.start.x) * start,
                    line.start.y + (line.end.y - line.start.y) * start,
                );
                let b = Pt::new(
                    line.start.x + (line.end.x - line.start.x) * end,
                    line.start.y + (line.end.y - line.start.y) * end,
                );
                pts.push(fit.point(a));
                pts.push(fit.point(b));
            }
            pts
        };
        stroke_path(&mut pixmap, &screen, width, color, dash);
    }

    Some((pixmap, fit))
}

/// Encode with `u16`-safe dimensions checked before encoding.
/// `drawBgImage`: the image centred on its world point, rotated, flipped, at
/// its opacity, `width * scale * zoom` pixels wide.
fn draw_backdrop(pixmap: &mut tiny_skia::Pixmap, fit: &PngFit, b: &ExportBackdrop) {
    let expected = (b.width as usize) * (b.height as usize) * 4;
    if b.width == 0 || b.height == 0 || b.rgba.len() < expected {
        return;
    }
    let Some(mut image) = tiny_skia::Pixmap::new(b.width, b.height) else {
        return;
    };
    // tiny-skia stores premultiplied alpha.
    let (dst, _) = image.data_mut().as_chunks_mut::<4>();
    let (src, _) = b.rgba.as_chunks::<4>();
    for (dst, src) in dst.iter_mut().zip(src) {
        let a = u32::from(src[3]);
        let mul = |c: u8| ((u32::from(c) * a + 127) / 255) as u8;
        *dst = [mul(src[0]), mul(src[1]), mul(src[2]), src[3]];
    }
    let (cx, cy) = fit.point(Pt::new(b.x, b.y));
    let k = (b.scale * fit.zoom) as f32;
    let sx = if b.flip_h { -k } else { k };
    let sy = if b.flip_v { -k } else { k };
    let transform = tiny_skia::Transform::from_translate(cx as f32, cy as f32)
        .pre_rotate(b.rotation as f32)
        .pre_scale(sx, sy)
        .pre_translate(-(b.width as f32) / 2.0, -(b.height as f32) / 2.0);
    let paint = tiny_skia::PixmapPaint {
        opacity: b.opacity.clamp(0.0, 1.0) as f32,
        quality: tiny_skia::FilterQuality::Bilinear,
        ..tiny_skia::PixmapPaint::default()
    };
    pixmap.draw_pixmap(0, 0, image.as_ref(), &paint, transform, None);
}

pub fn encode_png(pixmap: &tiny_skia::Pixmap) -> Option<Vec<u8>> {
    let _ = u16::try_from(pixmap.width());
    let _ = u16::try_from(pixmap.height());
    pixmap.encode_png().ok()
}

/// Convenience: build the export solids list for a page (app-side glue
/// mirrors `computeWallSolids`); the core exposes the type only.
#[allow(dead_code)]
pub fn solids_of(page: &ExportPage) -> &[ExportSolid] {
    &page.wall_solids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_model::{ExportLine, ExportPage};

    #[test]
    fn a_backdrop_is_drawn_beneath_and_grows_the_bounds() {
        let backdrop = ExportBackdrop {
            rgba: [200u8, 40, 40, 255].repeat(16),
            width: 4,
            height: 4,
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            rotation: 0.0,
            opacity: 1.0,
            flip_h: false,
            flip_v: false,
        };
        // An empty sketch with an image still renders (the image is content).
        let bytes = export_png_with_backdrop(
            &ExportPage::default(),
            &ExportOptions::default(),
            &[],
            (200, 200),
            Some(&backdrop),
        )
        .expect("renders");
        let decoded = tiny_skia::Pixmap::decode_png(&bytes).expect("decodes");
        // The fit centres the image; its middle pixel carries its colour.
        let mid = decoded.pixel(100, 100).expect("pixel");
        assert!(mid.alpha() > 200 && mid.red() > mid.blue(), "{mid:?}");
        // Without the image the same call stays transparent there.
        let bare = export_png(
            &ExportPage::default(),
            &ExportOptions::default(),
            &[],
            (200, 200),
        )
        .expect("renders");
        let bare = tiny_skia::Pixmap::decode_png(&bare).expect("decodes");
        assert_eq!(bare.pixel(100, 100).expect("pixel").alpha(), 0);
    }

    #[test]
    fn image_size_option_strings_parse() {
        assert_eq!(parse_image_size(Some("1280x720")), (1280, 720));
        assert_eq!(parse_image_size(Some("4800x4800")), (4800, 4800));
        assert_eq!(parse_image_size(Some("garbage")), DEFAULT_IMAGE_SIZE);
        assert_eq!(parse_image_size(None), DEFAULT_IMAGE_SIZE);
    }

    #[test]
    fn image_size_parsing() {
        assert_eq!(parse_image_size(None), (2048, 1536));
        assert_eq!(parse_image_size(Some("2048x1536")), (2048, 1536));
        assert_eq!(parse_image_size(Some("1920x1080")), (1920, 1080));
        assert_eq!(parse_image_size(Some("junk")), (2048, 1536));
        assert_eq!(parse_image_size(Some("0x100")), (2048, 1536));
    }

    #[test]
    fn png_dimensions_match_target() {
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
            ..Default::default()
        };
        let bytes = export_png(&page, &ExportOptions::default(), &[], (320, 240)).expect("encodes");
        // Decode the IHDR: width/height big-endian at bytes 16..24.
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        assert_eq!((width, height), (320, 240));
    }

    #[test]
    fn png_fit_math_and_clamps() {
        // 20 ft wide content in 320x240 with margin 60:
        // zoom = min(200/(20*20), 120/20) = 0.5
        let fit = png_fit((320, 240), Pt::new(0.0, 0.0), Pt::new(20.0, 5.0), true);
        assert!((fit.zoom - 0.5).abs() < 1e-9);
        // pan centers the content: pan_x = 160 - 10*20*0.5 = 60.
        assert!((fit.pan_x - 60.0).abs() < 1e-9);
        // y-down: pan_y = 120 + 2.5*20*0.5 = 145.
        assert!((fit.pan_y - 145.0).abs() < 1e-9);
        // Clamped 0.1..10.
        let tiny = png_fit((320, 240), Pt::new(0.0, 0.0), Pt::new(50000.0, 1.0), true);
        assert!((tiny.zoom - 0.1).abs() < 1e-9);
        // Empty content: zoom 1 centered origin.
        let empty = png_fit((320, 240), Pt::new(0.0, 0.0), Pt::new(10.0, 10.0), false);
        assert_eq!(empty.zoom, 1.0);
        assert_eq!((empty.pan_x, empty.pan_y), (160.0, 120.0));
    }

    #[test]
    fn png_is_transparent_outside_geometry() {
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
            ..Default::default()
        };
        let pixmap =
            render_page_pixmap(&page, &ExportOptions::default(), &[], (320, 240)).expect("renders");
        // Corner pixel is fully transparent (browser exports transparent).
        let corner = pixmap.pixel(2, 2).expect("pixel");
        assert_eq!(corner.alpha(), 0);
    }

    fn render_page_pixmap(
        page: &ExportPage,
        options: &ExportOptions,
        area_codes: &[AreaCode],
        target: (u32, u32),
    ) -> Option<tiny_skia::Pixmap> {
        // Mirror of export_png's render for tests that inspect pixels.
        let (bounds_min, bounds_max) = crate::export_model::get_export_bounds(page);
        let has_content = !page.lines.is_empty() || !page.polygons.is_empty();
        let fit = png_fit(target, bounds_min, bounds_max, has_content);
        let mut pixmap = tiny_skia::Pixmap::new(target.0, target.1)?;
        let colors = build_area_line_colors(page, area_codes);
        for line in &page.lines {
            let color = wall_color(line, &colors, options);
            let a = fit.point(line.start);
            let b = fit.point(line.end);
            stroke_path(&mut pixmap, &[a, b], wall_width(line, options), color, None);
        }
        Some(pixmap)
    }

    #[test]
    fn dash_runs_alternate() {
        let runs = dash_runs(&[(0.0, 0.0), (100.0, 0.0)], 8.0, 5.0);
        // 100px with 8 on / 5 off: 8,5,8,5,8,5,8,5,8,5,8,5,8,5,8,5,8 = 17 runs
        // (last on-run may be partial).
        assert!(runs.len() >= 8);
        assert_eq!(runs[0][0], (0.0, 0.0));
        assert_eq!(runs[0][1], (8.0, 0.0));
    }
}
