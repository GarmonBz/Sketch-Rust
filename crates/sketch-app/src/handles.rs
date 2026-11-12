//! Rotation handles for text labels, area labels / sizes and selected areas
//! (`getTextBoxTransform`, `getLabelTransform`, `getAreaTextTransform`,
//! `getSelectedAreaTransform`, `getRotationHandleHit` in
//! refs/canvas-engine.js).
//!
//! Everything here is in screen px (y down), so a positive rotation turns
//! clockwise on screen, as the original's does. The text metrics are the
//! canvas's own estimates (a canvas draw has no text measuring), shared with
//! the painter so the box wraps what is drawn.

use std::collections::BTreeMap;

use sketch_core::geometry::{Pt, PIXELS_PER_FOOT};
use sketch_core::model::Polygon;

use crate::calcs::Computed;
use crate::drawing::Drawing;
use crate::placed_text::PlacedText;
use crate::selection::Selection;

/// Which kind of text entity a box belongs to (`transform.type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKind {
    /// A free text label (`label`).
    Label,
    /// A defined area's name / code block (`areaLabel`).
    AreaLabel,
    /// A defined area's size block (`areaSize`).
    AreaSize,
}

/// `screen(p)` for a pan / zoom.
pub fn to_screen(p: Pt, pan: (f64, f64), zoom: f64) -> (f64, f64) {
    let k = PIXELS_PER_FOOT * zoom;
    (pan.0 + p.x * k, pan.1 - p.y * k)
}

/// `getTextZoom()`.
fn text_zoom(zoom: f64) -> f64 {
    if zoom == 0.0 {
        1.0
    } else {
        zoom
    }
}

/// The painted size and width of a free text label: the font size scaled by
/// the zoom (8..72 px) and ~0.52em a glyph (0.57 bold).
pub fn label_metrics(t: &PlacedText, zoom: f64) -> (f32, f32) {
    let size = ((t.font_size as f32) * (PIXELS_PER_FOOT as f32 * zoom as f32 / 12.0).max(1.0))
        .clamp(8.0, 72.0);
    let em = if t.bold { 0.57 } else { 0.52 };
    (size, t.text.chars().count() as f32 * size * em)
}

/// One of an area's two text blocks, laid out like `drawPolygons` paints it.
pub struct AreaBlock {
    pub kind: TextKind,
    pub lines: Vec<String>,
    /// World position of the block's centre.
    pub at: Pt,
    pub rotation: f64,
    /// `labelScale` / `areaSizeScale` (the size falls back to the label's).
    pub scale: f64,
    /// The area's own typography; `None` / 0 = the Options auto-post settings.
    pub font_size: f64,
    pub family: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: bool,
    pub color: Option<String>,
}

/// `Math.max(0.5, Math.min(4, Number(scale) || 1))`.
fn clamped_scale(scale: Option<f64>) -> f64 {
    scale.filter(|s| *s != 0.0).unwrap_or(1.0).clamp(0.5, 4.0)
}

/// `getAreaLabelPosition(poly)`: its own `labelPosition`, else the area's
/// label anchor.
pub fn label_position(d: &Drawing, poly: &Polygon) -> Pt {
    if let Some(p) = poly
        .label_position
        .filter(|p| p.x.is_finite() && p.y.is_finite())
    {
        return p;
    }
    if poly.points.is_empty() {
        return Pt::new(0.0, 0.0);
    }
    d.engine()
        .area_label_anchor(&d.engine().flatten_ring(&poly.points, &poly.edges, 0.05))
}

/// `getAreaSizePosition(poly)`: its own `areaSizePosition`, else below the
/// label by 7.5px a line plus a 6px gap (scaled), or on the label when neither code nor name
/// is posted.
pub fn size_position(d: &Drawing, poly: &Polygon) -> Pt {
    if let Some(p) = poly
        .area_size_position
        .filter(|p| p.x.is_finite() && p.y.is_finite())
    {
        return p;
    }
    let label = label_position(d, poly);
    let post = &poly.auto_post;
    if !post.code && !post.name {
        return label;
    }
    let label_lines = f64::from(u8::from(post.code)) + f64::from(u8::from(post.name));
    let size_lines = f64::from(u8::from(post.calcs))
        + f64::from(u8::from(post.base))
        + f64::from(u8::from(post.factor));
    let label_scale = clamped_scale(poly.label_scale);
    let size_scale = clamped_scale(poly.area_size_scale.or(poly.label_scale));
    let separation = (label_lines * label_scale + size_lines.max(1.0) * size_scale) * 7.5
        + 6.0 * label_scale.max(size_scale);
    // World y grows upward, so a smaller y puts the size below the label.
    Pt::new(label.x, label.y - separation / PIXELS_PER_FOOT)
}

/// An area's label and size blocks (`getAreaLabelGroups` + the block
/// positions); empty for an undefined area.
pub fn area_blocks(
    d: &Drawing,
    computed: &BTreeMap<i64, Computed>,
    poly: &Polygon,
) -> Vec<AreaBlock> {
    if poly.kind == "UND" || poly.kind.is_empty() {
        return Vec::new();
    }
    let post = &poly.auto_post;
    let mut label: Vec<String> = Vec::new();
    let mut size: Vec<String> = Vec::new();
    let (net, factor) = computed
        .get(&poly.id)
        .map_or((poly.area, poly.factor), |c| (c.net, c.factor));
    if post.code {
        label.push(poly.kind.clone());
    }
    if post.name {
        label.push(poly.name.clone());
    }
    if post.calcs {
        size.push(format!("{net:.1} {}", post.suffix));
    }
    if post.base {
        size.push(format!("Base: {:.1}", poly.area));
    }
    if post.factor {
        size.push(format!("Factor: {factor:.2}"));
    }
    let label_rotation = poly.label_rotation.unwrap_or(0.0);
    // Rendering uses the raw scale (`Number(scale) || 1`), unclamped.
    let raw = |s: Option<f64>| s.filter(|s| *s != 0.0).unwrap_or(1.0);
    let label_scale = raw(poly.label_scale);
    let block = |kind, lines: Vec<String>, at, rotation, scale| AreaBlock {
        kind,
        lines,
        at,
        rotation,
        scale,
        font_size: poly.label_font_size.unwrap_or(0.0),
        family: poly.label_font_family.clone().filter(|f| !f.is_empty()),
        bold: poly.label_bold,
        italic: poly.label_italic,
        underline: poly.label_underline.unwrap_or(false),
        color: poly.label_color.clone().filter(|c| !c.is_empty()),
    };
    let mut out = Vec::new();
    if !label.is_empty() {
        out.push(block(
            TextKind::AreaLabel,
            label,
            label_position(d, poly),
            label_rotation,
            label_scale,
        ));
    }
    if !size.is_empty() {
        out.push(block(
            TextKind::AreaSize,
            size,
            size_position(d, poly),
            poly.area_size_rotation.unwrap_or(label_rotation),
            raw(poly.area_size_scale.or(poly.label_scale)),
        ));
    }
    out
}

/// A block's per-line painted font size and weight: `(size px, bold)`.
pub fn block_line_font(
    d: &Drawing,
    block: &AreaBlock,
    label_lines: usize,
    index: usize,
    zoom: f64,
) -> (f32, bool) {
    let opts = &d.options;
    let first = if block.kind == TextKind::AreaSize {
        label_lines == 0 && index == 0
    } else {
        index == 0
    };
    let base = if block.font_size > 0.0 {
        block.font_size as f32
    } else if opts.auto_post_font_size > 0.0 {
        opts.auto_post_font_size as f32
    } else if index == 0 {
        12.0
    } else {
        11.0
    };
    let size = (base * block.scale as f32 * text_zoom(zoom) as f32).max(9.0);
    (size, block.bold.unwrap_or(opts.auto_post_bold) || first)
}

/// The block's italic flag (`poly.labelItalic ?? options.autoPostItalic`).
pub fn block_italic(d: &Drawing, block: &AreaBlock) -> bool {
    block.italic.unwrap_or(d.options.auto_post_italic)
}

/// The block's family (`labelFontFamily || options.autoPostFont || 'Inter'`).
pub fn block_family<'a>(d: &'a Drawing, block: &'a AreaBlock) -> &'a str {
    block
        .family
        .as_deref()
        .or_else(|| Some(d.options.auto_post_font.as_str()).filter(|f| !f.is_empty()))
        .unwrap_or("Inter")
}

/// The pitch between a block's lines in px.
pub fn block_line_step(block: &AreaBlock, zoom: f64) -> f32 {
    (15.0 * block.scale as f32 * text_zoom(zoom) as f32).max(9.0)
}

/// A block's box size in px: the widest line + 10 by the stacked lines + 8.
fn block_size(d: &Drawing, block: &AreaBlock, label_lines: usize, zoom: f64) -> (f64, f64) {
    let mut width: f32 = 0.0;
    for (index, text) in block.lines.iter().enumerate() {
        let (size, bold) = block_line_font(d, block, label_lines, index, zoom);
        let em = if bold { 0.57 } else { 0.52 };
        width = width.max(text.chars().count() as f32 * size * em);
    }
    let step = block_line_step(block, zoom);
    (
        f64::from(width) + 10.0,
        f64::from(step) * block.lines.len() as f64 + 8.0,
    )
}

/// `getTextBoxTransform`, in screen px.
#[derive(Debug, Clone, Copy)]
pub struct TextBox {
    pub kind: TextKind,
    pub id: i64,
    pub center_world: Pt,
    pub center: (f64, f64),
    /// Degrees, clockwise on screen.
    pub rotation: f64,
    pub corners: [(f64, f64); 4],
    pub top_anchor: (f64, f64),
    pub handle: (f64, f64),
}

impl TextBox {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: TextKind,
        id: i64,
        center_world: Pt,
        pan: (f64, f64),
        zoom: f64,
        width: f64,
        height: f64,
        rotation: f64,
    ) -> TextBox {
        let center = to_screen(center_world, pan, zoom);
        let radians = rotation.to_radians();
        let (cos, sin) = (radians.cos(), radians.sin());
        let rotate = |x: f64, y: f64| (center.0 + x * cos - y * sin, center.1 + x * sin + y * cos);
        let half_w = (width / 2.0).max(8.0);
        let half_h = (height / 2.0).max(7.0);
        TextBox {
            kind,
            id,
            center_world,
            center,
            rotation,
            corners: [
                rotate(-half_w, -half_h),
                rotate(half_w, -half_h),
                rotate(half_w, half_h),
                rotate(-half_w, half_h),
            ],
            top_anchor: rotate(0.0, -half_h),
            handle: rotate(0.0, -half_h - 30.0),
        }
    }
}

/// The boxes of every selected text entity, in `getRotationHandleHit`'s
/// order: area sizes, area labels, free labels.
pub fn selected_text_boxes(
    d: &Drawing,
    sel: &Selection,
    computed: &BTreeMap<i64, Computed>,
    pan: (f64, f64),
    zoom: f64,
) -> Vec<TextBox> {
    let mut out = Vec::new();
    for kind in [TextKind::AreaSize, TextKind::AreaLabel] {
        let set = if kind == TextKind::AreaSize {
            &sel.area_sizes
        } else {
            &sel.area_labels
        };
        for poly in d.polygons.iter().filter(|p| set.contains(&p.id)) {
            let blocks = area_blocks(d, computed, poly);
            let label_lines = blocks
                .iter()
                .find(|b| b.kind == TextKind::AreaLabel)
                .map_or(0, |b| b.lines.len());
            if let Some(block) = blocks.iter().find(|b| b.kind == kind) {
                let (w, h) = block_size(d, block, label_lines, zoom);
                out.push(TextBox::new(
                    kind,
                    poly.id,
                    block.at,
                    pan,
                    zoom,
                    w,
                    h,
                    block.rotation,
                ));
            }
        }
    }
    for t in d.texts.iter().filter(|t| sel.texts.contains(&t.id)) {
        let (size, width) = label_metrics(t, zoom);
        out.push(TextBox::new(
            TextKind::Label,
            t.id,
            Pt::new(t.x, t.y),
            pan,
            zoom,
            f64::from(width) + 8.0,
            f64::from(size) + 8.0,
            t.rotation,
        ));
    }
    out
}

/// `getSelectedAreaTransform`: the selected geometry's centre and the handle
/// 34px above its top edge.
#[derive(Debug, Clone, Copy)]
pub struct AreaBox {
    pub center_world: Pt,
    pub max_y: f64,
    pub center: (f64, f64),
    pub top: (f64, f64),
    pub handle: (f64, f64),
}

pub fn selected_area_box(
    d: &Drawing,
    sel: &Selection,
    pan: (f64, f64),
    zoom: f64,
) -> Option<AreaBox> {
    if sel.polys.is_empty() {
        return None;
    }
    let model = crate::transform::selected_model(d, sel)?;
    let center = to_screen(model.center, pan, zoom);
    let top = to_screen(Pt::new(model.center.x, model.max_y), pan, zoom);
    Some(AreaBox {
        center_world: model.center,
        max_y: model.max_y,
        center,
        top,
        handle: (center.0, top.1 - 34.0),
    })
}

/// What a press on a rotation handle grabbed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandleHit {
    Text {
        kind: TextKind,
        id: i64,
        center_world: Pt,
        start_rotation: f64,
    },
    Area {
        center_world: Pt,
    },
}

/// `getRotationHandleHit` for the text and area handles (the symbol handle
/// has its own hit test): within 11px of a handle's centre.
pub fn hit_handle(
    d: &Drawing,
    sel: &Selection,
    computed: &BTreeMap<i64, Computed>,
    pan: (f64, f64),
    zoom: f64,
    screen: (f64, f64),
) -> Option<HandleHit> {
    let near = |h: (f64, f64)| (screen.0 - h.0).hypot(screen.1 - h.1) <= 11.0;
    // Later entries win, as the original's reverse scan does.
    for b in selected_text_boxes(d, sel, computed, pan, zoom)
        .iter()
        .rev()
    {
        if near(b.handle) {
            return Some(HandleHit::Text {
                kind: b.kind,
                id: b.id,
                center_world: b.center_world,
                start_rotation: b.rotation,
            });
        }
    }
    let area = selected_area_box(d, sel, pan, zoom)?;
    near(area.handle).then_some(HandleHit::Area {
        center_world: area.center_world,
    })
}

/// `pointInTextTransform` for the area text under the pointer: the
/// precise box (+5px) first, else within 22px of a block's position
/// (`hitTestAt`'s area-label pass). Returns the nearest block's kind.
pub fn area_text_hit(
    d: &Drawing,
    computed: &BTreeMap<i64, Computed>,
    selectable: &dyn Fn(i64) -> bool,
    world: Pt,
    pan: (f64, f64),
    zoom: f64,
) -> Option<(TextKind, i64)> {
    let screen = to_screen(world, pan, zoom);
    for poly in d.polygons.iter().rev() {
        if !selectable(poly.id) {
            continue;
        }
        let blocks = area_blocks(d, computed, poly);
        if blocks.is_empty() {
            continue;
        }
        let label_lines = blocks
            .iter()
            .find(|b| b.kind == TextKind::AreaLabel)
            .map_or(0, |b| b.lines.len());
        // Candidates (kind, distance to the block's position): the nearer
        // block wins when both match.
        let mut found: Vec<(TextKind, f64)> = Vec::new();
        for block in &blocks {
            let (w, h) = block_size(d, block, label_lines, zoom);
            let b = TextBox::new(
                block.kind,
                poly.id,
                block.at,
                pan,
                zoom,
                w,
                h,
                block.rotation,
            );
            let (dx, dy) = (screen.0 - b.center.0, screen.1 - b.center.1);
            let radians = (-block.rotation).to_radians();
            let local = (
                dx * radians.cos() - dy * radians.sin(),
                dx * radians.sin() + dy * radians.cos(),
            );
            let half_w = (w / 2.0).max(8.0);
            let half_h = (h / 2.0).max(7.0);
            if local.0.abs() <= half_w + 5.0 && local.1.abs() <= half_h + 5.0 {
                found.push((
                    block.kind,
                    (world.x - block.at.x).hypot(world.y - block.at.y),
                ));
            }
        }
        if let Some(kind) = nearest(&found) {
            return Some((kind, poly.id));
        }
        // The 22px fallback around each block's position.
        let tolerance = 22.0 / (PIXELS_PER_FOOT * zoom.max(1e-9));
        for block in &blocks {
            let distance = (world.x - block.at.x).hypot(world.y - block.at.y);
            if distance <= tolerance {
                found.push((block.kind, distance));
            }
        }
        if let Some(kind) = nearest(&found) {
            return Some((kind, poly.id));
        }
    }
    None
}

/// The kind with the smallest distance.
fn nearest(found: &[(TextKind, f64)]) -> Option<TextKind> {
    found
        .iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(kind, _)| *kind)
}

/// A selected symbol's transform box (`getSymbolTransform`), in screen px: the
/// four corner (resize) handles and the rotation handle 30px above the top
/// edge, all turned with the symbol. `rotation` is the symbol's own
/// `rotation` - degrees clockwise on screen, as `ctx.rotate` takes it.
#[derive(Debug, Clone, Copy)]
pub struct SymbolBox {
    pub center: (f64, f64),
    pub corners: [(f64, f64); 4],
    pub handle: (f64, f64),
}

/// `bounds` is the definition's `(min_x, min_y, max_x, max_y)`.
pub fn symbol_box(
    at: Pt,
    rotation: f64,
    scale: f64,
    bounds: (f64, f64, f64, f64),
    pan: (f64, f64),
    zoom: f64,
) -> SymbolBox {
    let center = to_screen(at, pan, zoom);
    let pixel_scale = PIXELS_PER_FOOT * zoom * scale / 12.0;
    let bw = ((bounds.2 - bounds.0) * pixel_scale).max(12.0);
    let bh = ((bounds.3 - bounds.1) * pixel_scale).max(12.0);
    let radians = rotation.to_radians();
    let (cos, sin) = (radians.cos(), radians.sin());
    let turn = |x: f64, y: f64| (center.0 + x * cos - y * sin, center.1 + x * sin + y * cos);
    SymbolBox {
        center,
        corners: [
            turn(-bw / 2.0, -bh / 2.0),
            turn(bw / 2.0, -bh / 2.0),
            turn(bw / 2.0, bh / 2.0),
            turn(-bw / 2.0, bh / 2.0),
        ],
        handle: turn(0.0, -bh / 2.0 - 30.0),
    }
}

/// What a press on a selected symbol's handles grabbed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolHandle {
    Rotate,
    Resize,
}

/// `getRotationHandleHit` (symbol) / `getSymbolResizeHandleHit`: the rotation
/// handle within 11px, a corner within 7px on each axis.
pub fn hit_symbol_handle(b: &SymbolBox, screen: (f64, f64)) -> Option<SymbolHandle> {
    if (screen.0 - b.handle.0).hypot(screen.1 - b.handle.1) <= 11.0 {
        return Some(SymbolHandle::Rotate);
    }
    b.corners
        .iter()
        .any(|c| (screen.0 - c.0).abs() <= 7.0 && (screen.1 - c.1).abs() <= 7.0)
        .then_some(SymbolHandle::Resize)
}

/// The rotation a symbol's handle drag gives: the start rotation minus the
/// pointer's world-angle change in steps (15 degrees, 1 with Ctrl) - the same
/// rule as the text handles.
pub fn symbol_drag_rotation(
    start_rotation: f64,
    start_angle: f64,
    current_angle: f64,
    ctrl: bool,
) -> f64 {
    text_drag_rotation(start_rotation, start_angle, current_angle, ctrl)
}

/// The scale a symbol's corner drag gives: the start scale times the pointer's
/// distance ratio, in 5% steps (1% with Ctrl), 0.25..8.
pub fn symbol_drag_scale(start_scale: f64, start_dist: f64, dist: f64, ctrl: bool) -> f64 {
    let step = if ctrl { 0.01 } else { 0.05 };
    let raw = start_scale * dist / start_dist.max(1e-9);
    ((raw / step).round() * step).clamp(0.25, 8.0)
}

/// What the measurement-label tests need of the view: the drawing, the pan /
/// zoom, the label placement, and the two switches `isDimensionVisible`
/// reads (the Dimensions toggle and the Dimensions filter).
pub struct DimView<'a> {
    pub drawing: &'a Drawing,
    pub pan: (f64, f64),
    pub zoom: f64,
    pub placement: crate::view_state::DimensionPlacement,
    pub enabled: bool,
    pub filter_on: bool,
}

/// `getDimensionAnchor(line)`: the label's centre in screen px and its angle
/// (the wall's, flipped to read upright). A pinned label sits on its pin;
/// otherwise 16 px (min 9) off the wall's middle on the label side - the
/// hit test's own offset, a little wider than the painter's 10 px.
pub fn dimension_anchor(
    v: &DimView,
    line: &sketch_core::model::SketchLine,
) -> Option<(f64, f64, f64)> {
    let a = to_screen(line.start, v.pan, v.zoom);
    let b = to_screen(line.end, v.pan, v.zoom);
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = dx.hypot(dy);
    if len < 1.0 {
        return None;
    }
    let mut angle = dy.atan2(dx);
    let half_pi = std::f64::consts::FRAC_PI_2;
    if angle > half_pi {
        angle -= std::f64::consts::PI;
    }
    if angle < -half_pi {
        angle += std::f64::consts::PI;
    }
    if let Some(pin) = crate::drawing::pinned_dimension_point(line) {
        let s = to_screen(pin, v.pan, v.zoom);
        return Some((s.0, s.1, angle));
    }
    let (nx, ny) = (-dy / len, dx / len);
    let offset = (16.0 * text_zoom(v.zoom)).max(9.0);
    let side = f64::from(crate::render::line_dimension_side(
        v.drawing,
        line,
        v.placement,
    ));
    Some((
        (a.0 + b.0) / 2.0 + nx * offset * side,
        (a.1 + b.1) / 2.0 + ny * offset * side,
        angle,
    ))
}

/// `getDimensionLabelWorld(line)`.
pub fn dimension_label_world(v: &DimView, line: &sketch_core::model::SketchLine) -> Option<Pt> {
    let (x, y, _) = dimension_anchor(v, line)?;
    let k = PIXELS_PER_FOOT * v.zoom;
    Some(Pt::new((x - v.pan.0) / k, (v.pan.1 - y) / k))
}

/// The walls whose measurement label is visible and pickable
/// (`isDimensionVisible` + the layer gate).
pub fn visible_dimension_ids(v: &DimView, selectable: &dyn Fn(i64) -> bool) -> Vec<i64> {
    v.drawing
        .lines
        .iter()
        .filter(|l| selectable(l.id) && v.drawing.dimension_visible(l, v.enabled, v.filter_on))
        .map(|l| l.id)
        .collect()
}

/// `hitDimensionAt(worldPt)`: the nearest label whose box (the text + 6 px
/// each side, turned with the wall) holds the point.
pub fn hit_dimension(v: &DimView, world: Pt, selectable: &dyn Fn(i64) -> bool) -> Option<i64> {
    let screen = to_screen(world, v.pan, v.zoom);
    let opts = &v.drawing.options;
    let size = f64::from((opts.dimension_font_size as f32 * text_zoom(v.zoom) as f32).max(9.0));
    let mut best: Option<(i64, f64)> = None;
    for line in &v.drawing.lines {
        if !selectable(line.id) || !v.drawing.dimension_visible(line, v.enabled, v.filter_on) {
            continue;
        }
        let Some((ax, ay, angle)) = dimension_anchor(v, line) else {
            continue;
        };
        let text = opts
            .units
            .length(sketch_core::geometry::distance(line.start, line.end));
        // No text metrics inside a hit test: ~0.52em a glyph.
        let half_w = text.chars().count() as f64 * size * 0.52 / 2.0 + 6.0;
        let half_h = size / 2.0 + 6.0;
        let (dx, dy) = (screen.0 - ax, screen.1 - ay);
        let (cos, sin) = ((-angle).cos(), (-angle).sin());
        let (lx, ly) = (cos * dx - sin * dy, sin * dx + cos * dy);
        if lx.abs() <= half_w && ly.abs() <= half_h {
            let d = lx.hypot(ly);
            if best.is_none_or(|(_, b)| d < b) {
                best = Some((line.id, d));
            }
        }
    }
    best.map(|(id, _)| id)
}

/// `normalizeDegrees`: into -180..180.
pub fn normalize_degrees(value: f64) -> f64 {
    let mut result = value % 360.0;
    if result > 180.0 {
        result -= 360.0;
    }
    if result < -180.0 {
        result += 360.0;
    }
    result
}

/// The rotation a drag gives a text entity: the start rotation minus the
/// pointer's angle change, in 15 degree steps (1 with Ctrl).
pub fn text_drag_rotation(
    start_rotation: f64,
    start_angle: f64,
    current_angle: f64,
    ctrl: bool,
) -> f64 {
    let step = if ctrl { 1.0 } else { 15.0 };
    let delta = normalize_degrees((current_angle - start_angle).to_degrees());
    start_rotation - (delta / step).round() * step
}

/// The preview a drag gives the selected area: the pointer's angle change in
/// steps.
pub fn area_drag_preview(start_angle: f64, current_angle: f64, ctrl: bool) -> f64 {
    let step = if ctrl { 1.0 } else { 15.0 };
    let raw = normalize_degrees((current_angle - start_angle).to_degrees());
    (raw / step).round() * step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_box_sits_on_its_centre_with_the_handle_30px_above_the_top() {
        let b = TextBox::new(
            TextKind::Label,
            1,
            Pt::new(0.0, 0.0),
            (100.0, 100.0),
            1.0,
            60.0,
            20.0,
            0.0,
        );
        assert_eq!(b.center, (100.0, 100.0));
        assert_eq!(b.corners[0], (70.0, 90.0));
        assert_eq!(b.corners[2], (130.0, 110.0));
        assert_eq!(b.top_anchor, (100.0, 90.0));
        assert_eq!(b.handle, (100.0, 60.0));
        // Quarter turn clockwise on screen: the handle swings to the right.
        let b = TextBox::new(
            TextKind::Label,
            1,
            Pt::new(0.0, 0.0),
            (100.0, 100.0),
            1.0,
            60.0,
            20.0,
            90.0,
        );
        assert!((b.handle.0 - 140.0).abs() < 1e-9 && (b.handle.1 - 100.0).abs() < 1e-9);
        // The box never shrinks below 16 x 14.
        let tiny = TextBox::new(
            TextKind::Label,
            1,
            Pt::new(0.0, 0.0),
            (0.0, 0.0),
            1.0,
            2.0,
            2.0,
            0.0,
        );
        assert_eq!(tiny.corners[2], (8.0, 7.0));
    }

    #[test]
    fn drags_snap_to_15_degrees_or_1_with_ctrl() {
        let quarter = std::f64::consts::FRAC_PI_2;
        // A text label turns opposite the pointer's world-angle change.
        assert_eq!(text_drag_rotation(10.0, 0.0, quarter, false), 10.0 - 90.0);
        assert_eq!(
            text_drag_rotation(0.0, 0.0, 0.1, false),
            0.0,
            "under half a step"
        );
        assert_eq!(text_drag_rotation(0.0, 0.0, 0.2, true), -11.0);
        // The area preview follows the pointer.
        assert_eq!(area_drag_preview(0.0, quarter, false), 90.0);
        assert_eq!(area_drag_preview(0.0, 0.27, false), 15.0);
        // The wrap at 180.
        assert_eq!(normalize_degrees(270.0), -90.0);
        assert_eq!(normalize_degrees(-270.0), 90.0);
    }

    #[test]
    fn a_symbols_handles_turn_with_it_and_snap_in_steps() {
        // 36 x 12 in definition, 100% zoom (20 px/ft): 60 x 20 px box.
        let bounds = (-18.0, -6.0, 18.0, 6.0);
        let b = symbol_box(Pt::new(0.0, 0.0), 0.0, 1.0, bounds, (100.0, 100.0), 1.0);
        assert_eq!(b.center, (100.0, 100.0));
        // The handle is above the top edge (smaller screen y), 30px clear.
        assert!((b.handle.0 - 100.0).abs() < 1e-9 && (b.handle.1 - 60.0).abs() < 1e-9);
        assert_eq!(
            hit_symbol_handle(&b, (100.0, 66.0)),
            Some(SymbolHandle::Rotate)
        );
        assert_eq!(
            hit_symbol_handle(&b, (130.0, 110.0)),
            Some(SymbolHandle::Resize)
        );
        assert_eq!(hit_symbol_handle(&b, (100.0, 100.0)), None);
        // A quarter turn clockwise swings the handle to the right.
        let b = symbol_box(Pt::new(0.0, 0.0), 90.0, 1.0, bounds, (100.0, 100.0), 1.0);
        assert!((b.handle.0 - 140.0).abs() < 1e-9 && (b.handle.1 - 100.0).abs() < 1e-9);
        // Steps.
        let quarter = std::f64::consts::FRAC_PI_2;
        assert_eq!(symbol_drag_rotation(0.0, quarter, 0.0, false), 90.0);
        assert_eq!(
            symbol_drag_rotation(0.0, quarter, quarter - 0.1, false),
            0.0
        );
        assert_eq!(symbol_drag_scale(1.0, 10.0, 12.6, false), 1.25);
        assert_eq!(symbol_drag_scale(1.0, 10.0, 12.6, true), 1.26);
        assert_eq!(symbol_drag_scale(1.0, 10.0, 1.0, false), 0.25, "clamped");
    }

    fn sample_area() -> (Drawing, Polygon) {
        let mut d = Drawing::default();
        let poly: Polygon = serde_json::from_value(serde_json::json!({
            "id": 1,
            "type": "GLA1",
            "name": "Main",
            "points": [{"x":0.0,"y":0.0},{"x":10.0,"y":0.0},{"x":10.0,"y":10.0},{"x":0.0,"y":10.0}],
        }))
        .unwrap();
        d.polygons.push(poly.clone());
        (d, poly)
    }

    #[test]
    fn area_blocks_sit_at_their_own_position_scale_and_typography() {
        let (d, mut poly) = sample_area();
        let anchor = label_position(&d, &poly);
        assert!(
            (anchor.x - 5.0).abs() < 1e-6 && (anchor.y - 5.0).abs() < 1e-6,
            "{anchor:?}"
        );
        // The size sits below the label: name + one size line, plus breathing room.
        let below = size_position(&d, &poly);
        assert!((below.y - (anchor.y - ((1.0 + 1.0) * 7.5 + 6.0) / PIXELS_PER_FOOT)).abs() < 1e-9);
        // A label scale pushes the figure further down; the size scale is its
        // own (falling back to the label's).
        poly.label_scale = Some(2.0);
        let scaled = size_position(&d, &poly);
        assert!(
            (scaled.y - (anchor.y - ((2.0 + 2.0) * 7.5 + 12.0) / PIXELS_PER_FOOT)).abs() < 1e-9
        );
        poly.area_size_scale = Some(1.0);
        let split = size_position(&d, &poly);
        assert!((split.y - (anchor.y - ((2.0 + 1.0) * 7.5 + 12.0) / PIXELS_PER_FOOT)).abs() < 1e-9);
        // Positions of their own win.
        poly.label_position = Some(Pt::new(1.0, 2.0));
        poly.area_size_position = Some(Pt::new(3.0, 4.0));
        assert_eq!(label_position(&d, &poly), Pt::new(1.0, 2.0));
        assert_eq!(size_position(&d, &poly), Pt::new(3.0, 4.0));
        // Typography: the area's own size / weight / family beat the options.
        poly.label_font_size = Some(20.0);
        poly.label_bold = Some(true);
        let blocks = area_blocks(&d, &BTreeMap::new(), &poly);
        let label = &blocks[0];
        let (size, bold) = block_line_font(&d, label, 1, 1, 1.0);
        assert_eq!(size, 20.0 * 2.0, "20 x the label scale x zoom 1");
        assert!(bold);
        assert_eq!(block_family(&d, label), d.options.auto_post_font.as_str());
        poly.label_font_family = Some("Georgia".into());
        let blocks = area_blocks(&d, &BTreeMap::new(), &poly);
        assert_eq!(block_family(&d, &blocks[0]), "Georgia");
    }
}
