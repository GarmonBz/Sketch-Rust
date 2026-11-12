//! Phosphor icons (MIT, `@phosphor-icons/core@2.1.1`) — the glyphs the
//! original renders through its `ph` / `ph-fill` icon fonts. The SVGs in
//! `assets/icons/` are embedded at compile time. A font glyph of size N
//! fills an NxN em box, which is exactly a 256-unit viewBox drawn at NxN.

use iced::widget::svg;
use iced::{Color, Element};

use crate::theme::ink;
use crate::Message;

macro_rules! icon_table {
    (regular: [$($r:literal),* $(,)?], fill: [$($f:literal),* $(,)?]) => {
        fn bytes(name: &str, fill: bool) -> &'static [u8] {
            match (fill, name) {
                $((false, $r) => include_bytes!(concat!("../../../assets/icons/regular/", $r, ".svg")),)*
                $((true, $f) => include_bytes!(concat!("../../../assets/icons/fill/", $f, "-fill.svg")),)*
                // A missing icon is a bug, caught loudly by debug builds and
                // the tests; a release build draws nothing there instead of
                // taking the whole app (and the open sketch) down.
                _ if cfg!(debug_assertions) => panic!("icon not embedded: {name} (fill={fill})"),
                _ => br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"/>"#,
            }
        }
    };
}

icon_table! {
    regular: [
        "arrow-clockwise", "arrow-square-in", "drop", "moon", "sun", "arrow-u-up-left", "arrow-u-up-right", "arrows-down-up",
        "arrows-horizontal", "arrows-in", "arrows-left-right", "arrows-out",
        "arrows-out-cardinal", "bezier-curve", "bluetooth", "bounding-box", "caret-down",
        "caret-left", "caret-right", "check-circle", "circle-half-tilt", "clipboard-text",
        "copy", "corners-out", "couch", "crop", "crosshair", "cursor", "file", "file-image",
        "file-minus", "image", "map-pin",
        "file-plus", "file-x", "funnel", "globe-hemisphere-west", "grid-four", "hand-pointing",
        "house-line", "info", "intersect", "keyboard", "lasso", "layout", "line-segments", "list-bullets",
        "list-dashes", "list-numbers", "magnet", "magnifying-glass", "map-trifold", "minus",
        "monitor", "pencil-simple", "plus", "polygon", "printer", "ruler", "scissors",
        "clock-counter-clockwise",
        "selection-all", "selection-background", "square", "square-split-vertical", "text-t", "text-b", "text-italic", "text-underline", "text-aa",
        "text-a-underline",
        "x", "x-bold",
        // Canvas context menu (screen 6f).
        "minus-circle", "trash", "arrow-line-down", "arrow-line-up", "copy-simple", "sliders",
        // Symbols dock category icons (ui-manager.js categoryIcons).
        "arrow-up-right", "bathtub", "browser", "camera", "door-open", "fire",
        "fire-extinguisher", "oven", "shapes", "stairs", "star", "swimming-pool", "warning",
        "washing-machine", "circle",
        // Layer rows + their context menu (screen 10's `.layer-row`). The
        // locked state renders `ph-lock-simple-fill` (updateLayersPanel).
        "folder-simple", "lock-simple", "lock-simple-open", "export", "eye", "eye-slash",
        // Wall-shape menu (UIManager.showWallShapeMenu).
        "rectangle", "line-segment",
    ],
    fill: [
        "arrow-clockwise", "arrow-counter-clockwise", "arrow-square-in", "arrow-square-out",
        "arrow-up", "blueprint", "bluetooth", "bluetooth-connected", "calculator", "caret-down",
        "caret-left", "caret-line-left", "caret-line-right", "caret-right",
        "clock-counter-clockwise", "envelope-simple", "eye", "eye-slash", "file", "file-pdf",
        "floppy-disk", "folder-open", "gear", "intersect", "lock-simple", "map-pin",
        "pencil-simple", "power", "printer", "push-pin", "vector-two",
    ]
}

/// The QAT logo, verbatim from the original's `<svg class="qat-logo">`.
const QAT_LOGO: &str = r##"<svg width="20" height="20" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128" color="#FFFFFF"><rect width="128" height="128" rx="24" fill="#174396"/><path d="M102 22H26V72H78V86H26V106H102V54H50V42H102Z" fill="currentColor" fill-opacity="0.12"/><path d="M102 22H26V72H78V86H26V106H102V54H50V42H102Z" fill="none" stroke="currentColor" stroke-width="4" stroke-linejoin="miter"/><g fill="none" stroke="currentColor"><path d="M64 22V27M64 39V42M64 86V89M64 101V106" stroke-width="3"/><path d="M64 39H76M76 39A12 12 0 0 0 64 27M64 89H52M52 89A12 12 0 0 0 64 101" stroke-width="1.2"/><path d="M26 34V48M102 69V83" stroke="#174396" stroke-width="5"/><path d="M24.5 34V48M27.5 34V48" stroke-width="0.9"/><path d="M76 22H92M36 106H52" stroke="#174396" stroke-width="5"/><path d="M76 20.5H92M76 23.5H92M36 104.5H52M36 107.5H52" stroke-width="0.9"/><path d="M102 83H88" stroke-width="1.6"/><path d="M88 83A14 14 0 0 1 102 69" stroke-width="1.2"/></g></svg>"##;

fn handle(name: &str, fill: bool) -> svg::Handle {
    svg::Handle::from_memory(bytes(name, fill))
}

fn glyph<'a>(h: svg::Handle, size: f32, color: Color) -> Element<'a, Message> {
    ink(move || {
        svg(h.clone())
            .width(size)
            .height(size)
            .style(move |_, _| svg::Style { color: Some(color) })
    })
}

/// `<i class="ph ph-NAME">` (or `ph-fill`) at `size` px in `color`.
pub fn icon<'a>(name: &str, fill: bool, size: f32, color: Color) -> Element<'a, Message> {
    glyph(handle(name, fill), size, color)
}

/// Regular-weight icon element.
pub fn ph<'a>(name: &str, size: f32, color: Color) -> Element<'a, Message> {
    icon(name, false, size, color)
}

/// Fill-weight icon element.
pub fn ph_fill<'a>(name: &str, size: f32, color: Color) -> Element<'a, Message> {
    icon(name, true, size, color)
}

/// Icon rotated in place (CSS `transform: rotate(deg)`). The rotation is
/// baked into the SVG (about the 256-unit viewBox center) because the
/// tiny-skia backend mis-sizes rotated vector images.
pub fn rotated<'a>(
    name: &str,
    fill: bool,
    size: f32,
    color: Color,
    degrees: f32,
) -> Element<'a, Message> {
    let src = std::str::from_utf8(bytes(name, fill)).unwrap_or_default();
    let open_end = src.find('>').map_or(0, |i| i + 1);
    let close = src.rfind("</svg>").unwrap_or(src.len());
    let rotated = format!(
        "{}<g transform=\"rotate({degrees} 128 128)\">{}</g></svg>",
        &src[..open_end],
        &src[open_end..close]
    );
    glyph(svg::Handle::from_memory(rotated.into_bytes()), size, color)
}

/// The 20x20 application logo at the left of the title bar.
pub fn qat_logo<'a>() -> Element<'a, Message> {
    svg(svg::Handle::from_memory(QAT_LOGO.as_bytes()))
        .width(20)
        .height(20)
        .into()
}
