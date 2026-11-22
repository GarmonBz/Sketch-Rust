//! The external symbol library as the Symbols dock sees it, plus the symbol
//! preview renderer.
//!
//! The library is NEVER embedded: it is read at runtime from `symbols.json`
//! (resolved by `sketch_core::symbols::resolve`: next to the exe, then
//! `assets/` beside it, then the working directory) — once per process.
//!
//! `SymIndex` mirrors ui-manager.js's constructor: `symbolIndex` (id → item
//! with the category icon), `symbolCategories` (library order, Favorites
//! excluded) and `symbolFavorites` (= `defaultFavorites` filtered to known
//! ids). `Preview` is CanvasEngine.renderSymbolPreview /
//! renderSymbolGeometry / traceSymbolPolyline, ported line for line.

use std::sync::{Arc, Mutex};

use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke, Text};
use iced::{alignment, mouse, Color, Point, Rectangle, Renderer, Theme, Vector};
use sketch_core::symbols::{Entity, EntityType, Symbol, SymbolLibrary};

use crate::theme::hex;

/// ui-manager.js `categoryIcons` (Phosphor names without the `ph-` prefix);
/// anything else falls back to `square`.
pub fn category_icon(category: &str) -> &'static str {
    match category.strip_prefix("caddillo-").unwrap_or(category) {
        "arrows" => "arrow-up-right",
        "bath" => "bathtub",
        "cameras" => "camera",
        "circles" => "circle",
        "doors" | "doors-windows" => "door-open",
        "fire" => "fire-extinguisher",
        "fireplaces" => "fire",
        "insurance" => "warning",
        "kitchen" => "oven",
        "miscellaneous" => "shapes",
        "pools" => "swimming-pool",
        "shapes" => "square",
        "stairs" => "stairs",
        "utility" | "laundry" | "utilities" => "washing-machine",
        "windows" => "browser",
        "favorites" => "star",
        _ => "square",
    }
}

#[derive(Debug, Clone)]
pub struct SymItem {
    pub id: String,
    pub label: String,
    pub icon: &'static str,
}

#[derive(Debug)]
pub struct SymCategory {
    pub key: String,
    pub name: String,
    pub items: Vec<SymItem>,
}

#[derive(Debug)]
pub struct SymIndex {
    pub hidden: std::collections::BTreeSet<String>,
    /// `SymbolLibrary::duplicate_groups`, worked out the first time the
    /// manager asks (it hashes every symbol's geometry).
    pub(crate) duplicates: std::sync::OnceLock<Vec<Vec<String>>>,
    pub library: SymbolLibrary,
    /// The library's categories in file order, `favorites` excluded.
    pub categories: Vec<SymCategory>,
    /// `defaultFavorites` that exist in the library.
    pub favorites: Vec<SymItem>,
}

impl SymIndex {
    /// One library item by type id (`symbolIndex.get`).
    pub fn by_id(&self, id: &str) -> Option<SymItem> {
        if self.hidden.contains(id) {
            return None;
        }
        self.library
            .symbols
            .get(id)
            .map(|d| SymItem {
                id: d.id.clone(),
                label: d.label.clone(),
                icon: category_icon(&d.category),
            })
            .or_else(|| self.favorites.iter().find(|i| i.id == id).cloned())
    }
}

impl SymIndex {
    pub fn build(library: SymbolLibrary) -> Self {
        let item = |id: &str| {
            library.symbols.get(id).map(|d| SymItem {
                id: d.id.clone(),
                label: d.label.clone(),
                icon: category_icon(&d.category),
            })
        };
        let categories = library
            .categories
            .iter()
            .filter(|c| c.key != "favorites")
            .map(|c| SymCategory {
                key: c.key.clone(),
                name: c.name.clone(),
                items: c.symbol_ids.iter().filter_map(|id| item(id)).collect(),
            })
            .collect();
        let favorites = library
            .default_favorites
            .iter()
            .filter_map(|id| item(id))
            .collect();
        SymIndex {
            hidden: Default::default(),
            duplicates: Default::default(),
            library,
            categories,
            favorites,
        }
    }

    /// `getSymbolItems(category)`.
    pub fn items(&self, category: &str) -> &[SymItem] {
        if category == "favorites" {
            return &self.favorites;
        }
        self.categories
            .iter()
            .find(|c| c.key == category)
            .map(|c| c.items.as_slice())
            .unwrap_or(&[])
    }

    /// `populateSymbolCategorySelect` options: Favorites, then each category.
    #[cfg(test)]
    pub fn options(&self) -> Vec<(String, String)> {
        std::iter::once(("favorites".to_string(), "Favorites".to_string()))
            .chain(
                self.categories
                    .iter()
                    .map(|c| (c.key.clone(), c.name.clone())),
            )
            .collect()
    }

    pub fn symbol(&self, id: &str) -> Option<&Symbol> {
        self.library.symbols.get(id)
    }

    /// Groups of identical symbols in this library, keeper first.
    pub fn duplicates(&self) -> &[Vec<String>] {
        self.duplicates
            .get_or_init(|| self.library.duplicate_groups())
    }
}

#[cfg(test)]
thread_local! {
    /// A private copy of the library for this test thread, so deleting from
    /// the file never touches the repo's `assets/symbols.json`.
    static TEST_LIBRARY: std::cell::RefCell<Option<std::path::PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

/// Where the external library file is (or would be).
pub fn library_path() -> std::path::PathBuf {
    #[cfg(test)]
    if let Some(path) = TEST_LIBRARY.with(|p| p.borrow().clone()) {
        return path;
    }
    sketch_core::symbols::resolve(dev_hint().as_deref())
}

fn load_library(path: &std::path::Path) -> Arc<Result<SymIndex, String>> {
    let loaded = SymbolLibrary::load(path)
        .map(SymIndex::build)
        .map_err(|e| format!("Symbol library not found ({}): {e}", path.display()));
    if std::env::var_os("SKETCH_TRACE").is_some() {
        match &loaded {
            Ok(ix) => eprintln!(
                "TRACE symbols loaded from {}: {} symbols, {} categories, {} favorites",
                path.display(),
                ix.library.symbols.len(),
                ix.categories.len(),
                ix.favorites.len()
            ),
            Err(e) => eprintln!("TRACE {e}"),
        }
    }
    Arc::new(loaded)
}

static LIBRARY: Mutex<Option<Arc<Result<SymIndex, String>>>> = Mutex::new(None);

/// The process-wide library: loaded on first use from the external file and
/// again after [`delete_from_library`] rewrites it.
/// `Err` carries the message shown in place of the list.
pub fn library() -> Arc<Result<SymIndex, String>> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut cached = LIBRARY.lock().unwrap_or_else(|e| e.into_inner());
        return cached
            .get_or_insert_with(|| {
                let loaded = serde_json::from_str::<SymbolLibrary>(include_str!(
                    "../../../assets/symbols.json"
                ))
                .map(SymIndex::build)
                .map_err(|e| format!("Embedded symbol library invalid: {e}"));
                Arc::new(loaded)
            })
            .clone();
    }
    #[cfg(test)]
    if TEST_LIBRARY.with(|p| p.borrow().is_some()) {
        return load_library(&library_path());
    }
    let mut cached = LIBRARY.lock().unwrap_or_else(|e| e.into_inner());
    cached
        .get_or_insert_with(|| load_library(&library_path()))
        .clone()
}

/// Where the shipped read-only `symbols.original.json` copy lives, if present.
pub fn original_library_path() -> Option<std::path::PathBuf> {
    let lib_dir_candidate = library_path().with_file_name("symbols.original.json");
    #[cfg(test)]
    if TEST_LIBRARY.with(|p| p.borrow().is_some()) {
        return lib_dir_candidate.is_file().then_some(lib_dir_candidate);
    }
    let mut candidates = vec![lib_dir_candidate];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("symbols.original.json"));
            candidates.push(dir.join("assets").join("symbols.original.json"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("symbols.original.json"));
        candidates.push(cwd.join("assets").join("symbols.original.json"));
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// Clears the cached shared library so the next `library()` re-reads disk.
pub fn invalidate_cache() {
    *LIBRARY.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

/// Runs one `sketch_core::symbol_edit` operation on the library file and returns
/// `(changed)` on success or `(message, is_io_error)` on failure so callers can
/// distinguish read-only/permission errors from malformed payloads.
pub fn try_edit_library(
    edit: impl FnOnce(&std::path::Path) -> Result<usize, sketch_core::symbols::SymbolLibraryError>,
) -> Result<usize, (String, bool)> {
    let path = library_path();
    let changed = edit(&path).map_err(|e| {
        let is_io = matches!(e, sketch_core::symbols::SymbolLibraryError::Io(_));
        (format!("{}: {e}", path.display()), is_io)
    })?;
    invalidate_cache();
    Ok(changed)
}

/// Runs one `sketch_core::symbol_edit` operation on the library file, then
/// drops the cached library so the next `library()` reads the edited file.
/// Returns how many symbols (or categories) the edit changed.
pub fn edit_library(
    edit: impl FnOnce(&std::path::Path) -> Result<usize, sketch_core::symbols::SymbolLibraryError>,
) -> Result<usize, String> {
    try_edit_library(edit).map_err(|(msg, _)| msg)
}

/// Permanently removes `ids` from the library file and reloads it; returns how
/// many symbols the file held.
pub fn delete_from_library(ids: &std::collections::BTreeSet<String>) -> Result<usize, String> {
    edit_library(|path| sketch_core::symbol_edit::delete(path, ids))
}

/// Runs `test` against a private copy of the library file for this thread
/// (also seeding a read-only `symbols.original.json` sibling).
#[cfg(test)]
pub fn with_temp_library(test: impl FnOnce()) {
    with_temp_library_opts(true, test);
}

/// Same as [`with_temp_library`], optionally omitting `symbols.original.json`.
#[cfg(test)]
pub fn with_temp_library_opts(with_original: bool, test: impl FnOnce()) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/test-profiles/library-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let copy = dir.join("symbols.json");
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/symbols.json");
    std::fs::copy(&src, &copy).unwrap();
    if with_original {
        std::fs::copy(&src, dir.join("symbols.original.json")).unwrap();
    }
    TEST_LIBRARY.with(|p| *p.borrow_mut() = Some(copy));
    test();
    TEST_LIBRARY.with(|p| *p.borrow_mut() = None);
    let _ = std::fs::remove_dir_all(dir);
}

/// Tests run from the crate dir, not beside an exe with assets: point them at
/// the repo's `assets/symbols.json` (still read from disk at runtime).
fn dev_hint() -> Option<std::path::PathBuf> {
    if cfg!(test) {
        Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/symbols.json"))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Preview (CanvasEngine.renderSymbolPreview on a 92x68 canvas)
// ---------------------------------------------------------------------------

/// The canvas theme is `light` by default (white drawing surface):
/// stroke `#334155`, closed-shape background `colors.bg` = `#FFFFFF`.
const PREVIEW_STROKE: Color = hex(0x334155);
const PREVIEW_BG: Color = hex(0xffffff);

pub struct Preview<'a> {
    pub symbol: &'a Symbol,
}

impl<Message> canvas::Program<Message> for Preview<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        draw_preview(&mut frame, self.symbol, bounds.width, bounds.height);
        vec![frame.into_geometry()]
    }
}

fn draw_preview(frame: &mut Frame, def: &Symbol, cw: f32, ch: f32) {
    render_fitted(frame, def, cw, ch, 14.0, PREVIEW_STROKE, PREVIEW_BG);
}

/// Fit a symbol's geometry inside `(cw, ch)` minus `pad` and stroke it with `stroke`.
pub fn render_fitted(
    frame: &mut Frame,
    def: &Symbol,
    cw: f32,
    ch: f32,
    pad: f64,
    stroke: Color,
    background: Color,
) {
    let (scale, ox, oy) = fit_transform(def, cw, ch, pad);
    render_geometry(frame, def, scale, ox, oy, stroke, background);
}

pub(crate) fn fit_transform(def: &Symbol, cw: f32, ch: f32, pad: f64) -> (f64, f64, f64) {
    let b = &def.bounds;
    let (min_x, min_y, max_x, max_y) = if b.max_x > b.min_x || b.max_y > b.min_y {
        (b.min_x, b.min_y, b.max_x, b.max_y)
    } else {
        (-6.0, -6.0, 6.0, 6.0)
    };
    let width = (max_x - min_x).max(1.0);
    let height = (max_y - min_y).max(1.0);
    let scale = ((cw as f64 - pad).max(4.0) / width).min((ch as f64 - pad).max(4.0) / height);
    let cx = (min_x + max_x) / 2.0;
    let cy = (min_y + max_y) / 2.0;
    // translate(w/2 - cx*s, h/2 + cy*s) then scale(s, -s).
    let ox = cw as f64 / 2.0 - cx * scale;
    let oy = ch as f64 / 2.0 + cy * scale;
    (scale, ox, oy)
}

/// renderSymbolGeometry in screen space: symbol point (x, y) maps to
/// `(ox + x*s, oy - y*s)`; line width 1.25px, round caps and joins.
pub fn render_geometry(
    frame: &mut Frame,
    def: &Symbol,
    s: f64,
    ox: f64,
    oy: f64,
    stroke: Color,
    background: Color,
) {
    render_entities(frame, &def.geometry, s, ox, oy, stroke, background);
}

/// Renders a slice of symbol `Entity` primitives in screen space.
pub fn render_entities(
    frame: &mut Frame,
    geometry: &[Entity],
    s: f64,
    ox: f64,
    oy: f64,
    stroke: Color,
    background: Color,
) {
    if geometry.is_empty() || !s.is_finite() || s <= 0.0 {
        return;
    }
    let to = |x: f64, y: f64| Point::new((ox + x * s) as f32, (oy - y * s) as f32);
    let pen = Stroke::default()
        .with_color(stroke)
        .with_width(1.25)
        .with_line_cap(canvas::LineCap::Round)
        .with_line_join(canvas::LineJoin::Round);

    for e in geometry {
        let should_stroke = e.stroke_alpha.is_none_or(|a| a > 0);
        let fill_of = |c: i64| if c == 0 { stroke } else { background };
        match e.entity_type {
            EntityType::Line => {
                let (Some(a), Some(b)) = (&e.start_point, &e.end_point) else {
                    continue;
                };
                let path = Path::line(to(a.x, a.y), to(b.x, b.y));
                if should_stroke {
                    frame.stroke(&path, pen);
                }
            }
            EntityType::Polyline => {
                let Some(vs) = e.vertices.as_ref().filter(|v| !v.is_empty()) else {
                    continue;
                };
                let closed = e.closed.unwrap_or(false);
                let path = Path::new(|p| {
                    p.move_to(to(vs[0].x, vs[0].y));
                    let count = if closed { vs.len() } else { vs.len() - 1 };
                    for i in 0..count {
                        let a = &vs[i];
                        let b = &vs[(i + 1) % vs.len()];
                        let bulge = a.b.unwrap_or(0.0);
                        let chord = (b.x - a.x).hypot(b.y - a.y);
                        if bulge.abs() < 1e-7 || chord < 1e-7 {
                            p.line_to(to(b.x, b.y));
                            continue;
                        }
                        let nx = -(b.y - a.y) / chord;
                        let ny = (b.x - a.x) / chord;
                        let off = chord * (1.0 - bulge * bulge) / (4.0 * bulge);
                        let cx = (a.x + b.x) / 2.0 + nx * off;
                        let cy = (a.y + b.y) / 2.0 + ny * off;
                        let r = chord * (1.0 + bulge * bulge) / (4.0 * bulge.abs());
                        let start = (a.y - cy).atan2(a.x - cx);
                        let sweep = 4.0 * bulge.atan();
                        arc(p, &to, cx, cy, r, start, start + sweep);
                    }
                    if closed {
                        p.close();
                    }
                });
                if closed {
                    frame.fill(&path, fill_of(e.fill_color.unwrap_or(-1)));
                }
                if should_stroke {
                    frame.stroke(&path, pen);
                }
            }
            EntityType::Ellipse | EntityType::Arc => {
                let (Some(c), Some(ax)) = (&e.center, &e.axis_point) else {
                    continue;
                };
                let dx = ax.x - c.x;
                let dy = ax.y - c.y;
                let rx = dx.hypot(dy);
                let ratio = e.ratio.filter(|r| *r != 0.0).unwrap_or(1.0).abs();
                let ry = rx * ratio;
                let rot = dy.atan2(dx);
                let is_arc = matches!(e.entity_type, EntityType::Arc);
                let (a0, a1) = if is_arc {
                    (
                        e.start_angle.unwrap_or(0.0).to_radians(),
                        e.end_angle.unwrap_or(0.0).to_radians(),
                    )
                } else {
                    (0.0, std::f64::consts::TAU)
                };
                let path = Path::new(|p| ellipse(p, &to, c.x, c.y, rx, ry, rot, a0, a1));
                if !is_arc {
                    if let Some(fc) = e.fill_color {
                        frame.fill(&path, fill_of(fc));
                    }
                }
                if should_stroke {
                    frame.stroke(&path, pen);
                }
            }
            EntityType::Text => {
                let pt = e
                    .insertion_point
                    .as_ref()
                    .or(e.insert_point.as_ref())
                    .map(|p| (p.x, p.y))
                    .unwrap_or((0.0, 0.0));
                let align = e.alignment.clone().unwrap_or_default().to_uppercase();
                let size = (e.font_size.unwrap_or(10.0).max(1.0) * s) as f32;
                let rot = e.rotation.unwrap_or(0.0).to_radians() as f32;
                let at = to(pt.0, pt.1);
                frame.with_save(|f| {
                    f.translate(Vector::new(at.x, at.y));
                    // Symbol space is y-up: a CCW rotation there is CW on screen.
                    f.rotate(-rot);
                    f.fill_text(Text {
                        content: e.text_string.clone().unwrap_or_default(),
                        position: Point::ORIGIN,
                        color: stroke,
                        size: size.into(),
                        font: crate::theme::ARIAL,
                        align_x: if align.contains("RIGHT") {
                            iced::widget::text::Alignment::Right
                        } else if align.contains("CENTER") {
                            iced::widget::text::Alignment::Center
                        } else {
                            iced::widget::text::Alignment::Left
                        },
                        align_y: if align.contains("TOP") {
                            alignment::Vertical::Top
                        } else if align.contains("BOTTOM") {
                            alignment::Vertical::Bottom
                        } else {
                            alignment::Vertical::Center
                        },
                        ..Text::default()
                    });
                });
            }
        }
    }
}

/// `ctx.arc(cx, cy, r, a0, a1, a1 < a0)` in symbol space (y-up), traced as a
/// polyline continuing the current path (canvas arcs connect with a line).
fn arc(
    p: &mut canvas::path::Builder,
    to: &impl Fn(f64, f64) -> Point,
    cx: f64,
    cy: f64,
    r: f64,
    a0: f64,
    a1: f64,
) {
    let steps = (((a1 - a0).abs() * r.max(1.0)).ceil() as usize).clamp(8, 256);
    for k in 0..=steps {
        let t = a0 + (a1 - a0) * k as f64 / steps as f64;
        p.line_to(to(cx + r * t.cos(), cy + r * t.sin()));
    }
}

/// `ctx.ellipse(cx, cy, rx, ry, rot, a0, a1, false)` in symbol space: the
/// sweep runs counter-clockwise (increasing angle) from a0 to a1.
#[allow(clippy::too_many_arguments)]
fn ellipse(
    p: &mut canvas::path::Builder,
    to: &impl Fn(f64, f64) -> Point,
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    rot: f64,
    a0: f64,
    a1: f64,
) {
    let tau = std::f64::consts::TAU;
    let mut sweep = a1 - a0;
    if sweep >= tau {
        sweep = tau;
    } else {
        sweep = sweep.rem_euclid(tau);
        if sweep == 0.0 && (a1 - a0).abs() > 0.0 {
            sweep = tau;
        }
    }
    let steps = ((sweep * rx.max(ry).max(1.0)).ceil() as usize).clamp(16, 360);
    let (sr, cr) = rot.sin_cos();
    let point = |t: f64| {
        let (x, y) = (rx * t.cos(), ry * t.sin());
        to(cx + x * cr - y * sr, cy + x * sr + y * cr)
    };
    p.move_to(point(a0));
    for k in 1..=steps {
        p.line_to(point(a0 + sweep * k as f64 / steps as f64));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_loads_from_the_external_file_with_the_originals_index() {
        let lib = library();
        let ix = lib.as_ref().as_ref().expect("assets/symbols.json loads");
        assert_eq!(ix.library.symbols.len(), 499);
        assert_eq!(ix.categories.len(), 18);
        assert_eq!(ix.categories[0].key, "caddillo-doors-windows");
        assert_eq!(ix.categories[0].items.len(), 21);
        assert!(ix
            .categories
            .iter()
            .any(|c| c.key == "fire" && c.items.len() == 119));
        assert_eq!(ix.favorites.len(), 18);
        assert_eq!(ix.favorites[0].label, "Door Swing 36");
        // Plain category names (no brand), merged CADdillo categories, and
        // retired duplicates replaced through aliases
        // (tools/symbol-curation.json, 2026-11-17).
        assert!(ix.categories.iter().all(|c| !c.name.contains("CADdillo")));
        assert_eq!(ix.categories[0].name, "Doors & windows");
        let kitchen = ix.categories.iter().find(|c| c.key == "kitchen").unwrap();
        assert_eq!(kitchen.name, "Kitchen");
        assert!(kitchen.items.iter().any(|i| i.id == "caddillo/fridge-36"));
        assert!(kitchen
            .items
            .iter()
            .all(|i| i.id != "kitchen/refrigerator-36in"));
        // Retired duplicates are gone; their ids alias the replacements.
        assert!(ix.library.symbol("kitchen/refrigerator-36in").is_none());
        assert!(ix.by_id("kitchen/refrigerator-36in").is_none());
        assert_eq!(
            ix.library.aliases.len(),
            68,
            "retired duplicates, plus the old double garage door"
        );
        assert_eq!(
            ix.library.aliases["kitchen/refrigerator-36in"],
            "caddillo/fridge-36"
        );
        assert!(ix
            .library
            .aliases
            .values()
            .all(|to| ix.library.symbols.contains_key(to)));
        assert_eq!(ix.favorites[0].icon, "door-open");
        assert_eq!(ix.options()[0], ("favorites".into(), "Favorites".into()));
    }
}
