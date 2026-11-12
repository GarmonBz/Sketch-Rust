//! Ribbon (0,30 1400x99): the tab strip (28px, `--bg-ribbon`) and the active
//! tab's body (70px, `--bg-panel`, padding 6px 12px) over a 1px `--border`
//! line. Every button, icon and label is placed at its measured position.
//!
//! Tab behavior follows `refs/ui-manager.js` (the `.tab` click handlers):
//! clicking a tab marks it `.active` and shows its `.ribbon-content`; the
//! File tab instead toggles the backstage (`#file-menu`) and becomes the
//! `.active` tab itself while the previously shown body stays visible.

use iced::widget::text::Alignment as TextAlign;
use iced::widget::{button, column, container, row, slider as iced_slider, stack, Space};
use iced::{Background, Border, Color, Element, Font, Length, Padding, Pixels};

use crate::icons::icon;
use crate::options_dialog;
use crate::popup_menu;
use crate::ribbon_tabs;
use crate::theme::*;
use crate::Message;

/// A ribbon tab button (`.tab[data-target]`, the File tab included).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    File,
    Home,
    View,
    Tools,
    ReferenceImage,
    Help,
    /// The contextual tab that only exists while something is selected.
    EditTools,
}

/// A ribbon body (`.ribbon-content`). The contextual Edit Tools body only
/// appears with a selection (screen 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Body {
    Home,
    View,
    Tools,
    ReferenceImage,
    Help,
    /// `.contextual-ribbon#tab-edit-tools` (screen 6).
    EditTools,
}

impl TabId {
    /// The body a (non-File) tab shows.
    pub fn body(self) -> Option<Body> {
        match self {
            TabId::File => None,
            TabId::EditTools => Some(Body::EditTools),
            TabId::Home => Some(Body::Home),
            TabId::View => Some(Body::View),
            TabId::Tools => Some(Body::Tools),
            TabId::ReferenceImage => Some(Body::ReferenceImage),
            TabId::Help => Some(Body::Help),
        }
    }
}

/// What the Edit Tools panel shows for the current selection
/// (`updateAreaActionStates`: length 2dp, style, colour, and which handle the
/// length edit moves).
pub struct EditToolsData {
    pub length_text: String,
    pub endpoint_label: &'static str,
    pub style_text: &'static str,
    pub color: Color,
    /// Screen 8: a symbol is selected, so the flip/opening-width buttons apply.
    pub has_symbols: bool,
    /// Screen 8: at least one selected symbol is wall-hosted.
    pub hosted: bool,
    /// Screen 9: a text label is selected.
    pub has_text: bool,
    /// A wall is selected (the length / style / colour group shows).
    pub has_line: bool,
    /// The selected label's / area text's typography for the text group.
    pub text: Option<TextTools>,
    /// An area is selected: the fill / outline group.
    pub area: Option<AreaTools>,
}

/// The Edit Tools text group's two typed boxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextField {
    /// `#text-font-size` (a number, 6-72).
    Size,
    /// `#selected-text-value` (the label's wording).
    Value,
}

/// What the text group shows for the selected label or area text
/// (`syncContextualControls`): font family, size, colour and wording.
#[derive(Debug, Clone)]
pub struct TextTools {
    pub font: String,
    pub size: String,
    pub color: Color,
    pub value: String,
}

/// `#text-font-family`'s options.
pub const TEXT_FONTS: [&str; 6] = [
    "Arial",
    "Calibri",
    "Georgia",
    "Segoe UI",
    "Times New Roman",
    "Verdana",
];

/// The selected area's values for the Edit Tools area group
/// (`syncContextualControls`).
#[derive(Debug, Clone, Copy)]
pub struct AreaTools {
    pub pattern: &'static str,
    pub fill: Color,
    pub line_style: &'static str,
    pub thickness: &'static str,
    pub line_color: Color,
}

/// The contextual Edit Tools tab `(x, width)`. The original paints it
/// `#F8FAFC` with dark text whether or not it is the tab showing; here it
/// is styled like every other tab, so only the tab in view is highlighted.
pub const EDIT_TOOLS_TAB: (f32, f32) = (413.92, 83.81);
/// `#selected-line-length` — the length field's widget id.
const LENGTH_INPUT_ID: &str = "selected-line-length";

/// `(tab, label, width)` — laid out from x=0 at y=34, h=24 (measured).
pub const TABS: &[(TabId, &str, f32)] = &[
    (TabId::File, "File", 51.0),
    (TabId::Home, "Home", 64.0),
    (TabId::View, "View", 58.0),
    (TabId::Tools, "Tools", 60.0),
    (TabId::ReferenceImage, "Reference Image", 124.0),
    (TabId::Help, "Help", 57.0),
];

/// The File tab's box (page coordinates): x 0..51, y 34..58.
pub const FILE_TAB_W: f32 = 51.0;
pub const TAB_Y: f32 = 34.0;
pub const TAB_H: f32 = 24.0;

/// `.tab.file-tab:hover`
const FILE_HOVER: Color = hex(0x006bb3);

fn tab<'a>(id: TabId, name: &'a str, w: f32, active: bool) -> Element<'a, Message> {
    let file = id == TabId::File;
    let fg = if file { WHITE } else { text_main() };
    // `.tab.active` sets `border-top-color: var(--bg-active)` (2px).
    let stripe = if active { accent() } else { Color::TRANSPARENT };
    button(column![
        rect(w, 2.0, stripe),
        container(label(name, 12.0, ARIAL, fg, 14.0)).padding(Padding {
            top: 4.0,
            left: 16.0,
            ..Padding::ZERO
        }),
    ])
    .width(w)
    .height(TAB_H)
    .padding(0)
    .on_press(Message::TabClicked(id))
    .style(move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        // theme.css cascade: `.tab:hover` < `.tab.file-tab` < `.tab.active`
        // (equal specificity, later rule wins) < `.tab.file-tab:hover`.
        let bg = if file && hovered {
            Some(FILE_HOVER)
        } else if active {
            Some(bg_panel())
        } else if file {
            Some(accent())
        } else if hovered {
            Some(bg_hover())
        } else {
            None
        };
        button::Style {
            background: bg.map(Background::Color),
            ..button::Style::default()
        }
    })
    .into()
}

/// A Phosphor glyph at an absolute page position.
#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub name: &'static str,
    pub fill: bool,
    pub size: f32,
    pub color: Color,
    pub x: f32,
    pub y: f32,
}

/// `<i class="ph(-fill) ph-NAME">` at `(x, y)`, `size` px.
pub const fn g(name: &'static str, fill: bool, size: f32, color: Color, x: f32, y: f32) -> Glyph {
    Glyph {
        name,
        fill,
        size,
        color,
        x,
        y,
    }
}

/// A text span at an absolute page position. `\n` splits a two-line label,
/// whose lines are centered within `w` (the span width); `line` is the
/// line box height.
#[derive(Debug, Clone, Copy)]
pub struct Caption {
    pub text: &'static str,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub line: f32,
    pub size: f32,
    pub font: Font,
    pub color: Color,
}

/// [`Caption`] constructor in spec order (text, x, y, w, line, size, font, color).
#[allow(clippy::too_many_arguments)]
pub const fn cap(
    text: &'static str,
    x: f32,
    y: f32,
    w: f32,
    line: f32,
    size: f32,
    font: Font,
    color: Color,
) -> Caption {
    Caption {
        text,
        x,
        y,
        w,
        line,
        size,
        font,
        color,
    }
}

/// How a ribbon button paints its box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    /// `.ribbon-btn` — transparent; hover `--bg-hover` + `--border` border.
    Ribbon,
    /// Inline-styled `<button>` (transparent, no border, no hover).
    Plain,
    /// `.ribbon-btn` with the inline toggled style (Grid):
    /// `rgba(0,122,204,.2)` + 1px `--border`, which also beats `:hover`.
    Toggled,
}

/// One button, in absolute page coordinates.
#[derive(Debug, Clone)]
pub struct Btn {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub look: Look,
    pub enabled: bool,
    pub glyphs: Vec<Glyph>,
    pub captions: Vec<Caption>,
    /// An explicit click action; `None` falls back to `command(body, btn)`.
    pub action: Option<Message>,
}

/// One element of a ribbon body.
#[derive(Debug, Clone)]
pub enum Item {
    Btn(Btn),
    /// `.ribbon-divider` at x (1x40 at y=73).
    Divider(f32),
    /// A free-standing text span (not inside a button).
    Caption(Caption),
    /// A free-standing glyph.
    #[allow(dead_code)]
    Glyph(Glyph),
    /// `<input type="range">` (Chrome's default look, accent `--text-muted`,
    /// value 50 of 0..100); the invisible iced slider takes the input.
    Slider {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        /// The live value (0-100).
        value: f64,
    },
    /// `.ribbon-field` / `.ribbon-color`: a label plus a boxed input, select
    /// or colour swatch.
    Field {
        label: Caption,
        box_x: f32,
        box_y: f32,
        box_w: f32,
        box_h: f32,
        value: &'static str,
        /// `<input type="color">` shows a swatch instead of text.
        swatch: Option<Color>,
        /// A `<select>` draws its chevron.
        chevron: bool,
        /// What a click runs (`None` = inert).
        action: Option<Message>,
    },
    /// A `<select>` that lists `options` and sends the chosen one.
    Select {
        label: Caption,
        box_x: f32,
        box_y: f32,
        box_w: f32,
        value: String,
        options: Vec<String>,
        on_pick: fn(String) -> Message,
    },
    /// A typed box of the text group: Enter applies it.
    TextInput {
        label: Caption,
        box_x: f32,
        box_y: f32,
        box_w: f32,
        value: String,
        field: TextField,
    },
    /// `<input id="selected-line-length" type="number">`: an editable field
    /// whose Enter applies the length.
    LengthInput {
        label: Caption,
        box_x: f32,
        box_y: f32,
        box_w: f32,
        value: String,
    },
}

/// One caption relative to its owner's origin.
fn caption_el<'a>(c: &Caption, color: Color) -> Element<'a, Message> {
    if c.text.contains('\n') {
        let (t, w, size, font, line) = (c.text, c.w, c.size, c.font, c.line);
        ink(move || {
            text_line(t, size, font, color, line)
                .width(w)
                .align_x(TextAlign::Center)
        })
    } else {
        label(c.text, c.size, c.font, color, c.line)
    }
}

/// A device name leaked once per distinct value so a caption can keep its
/// `&'static str` text (a handful of names over a session).
fn intern(s: &str) -> &'static str {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static NAMES: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
    let mut names = NAMES.get_or_init(Default::default).lock().unwrap();
    if let Some(found) = names.get(s) {
        return found;
    }
    let leaked: &'static str = Box::leak(s.to_string().into_boxed_str());
    names.insert(s.to_string(), leaked);
    leaked
}

/// The chip's caption for a device name: centred in the 81px button (the
/// original's label is centred text), clipped with an ellipsis when it would
/// overflow.
fn chip_caption(base: &Caption, btn_x: f32, btn_w: f32, name: &str) -> Caption {
    const MAX_CHARS: usize = 13;
    let shown: String = if name.chars().count() > MAX_CHARS {
        let head: String = name.chars().take(MAX_CHARS - 1).collect();
        format!("{head}\u{2026}")
    } else {
        name.to_string()
    };
    // Arial 11px averages ~5.6px a glyph.
    let w = (shown.chars().count() as f32 * 5.6).min(btn_w - 2.0);
    Caption {
        text: intern(&shown),
        x: btn_x + (btn_w - w) / 2.0,
        w,
        ..*base
    }
}

/// `.ribbon-btn:disabled` — `opacity: .38`.
const DISABLED_ALPHA: f32 = 0.38;

/// The `data-command` a ribbon button runs (refs/ui-manager.js
/// executeCommand), keyed by its body and normalized caption. Visual line
/// wrapping must not change which command a button invokes.
fn command(body: Body, b: &Btn) -> Message {
    use crate::popup_menu::PopupMsg;
    use crate::statusbar::StatusMsg;
    let caption = b.captions.first().map(|c| c.text.replace('\n', " "));
    match (body, caption.as_deref()) {
        // Home > Draw Color = 'draw-color' (toggleDrawColorPalette). The
        // dialog opens on the current colour, which the app owns.
        (Body::Home, Some("Draw Color")) => Message::OpenColorDialog,
        // View › Grid = 'toggle-grid'; View › Filter = 'filter'.
        (Body::View, Some("Grid")) => Message::Status(StatusMsg::ToggleGrid),
        (Body::View, Some("Filter")) => Message::Status(StatusMsg::ToggleFilter),
        // View › Apply (Dims) = 'cycle-dimensions'.
        (Body::View, Some("Apply")) => Message::Status(StatusMsg::CycleDimensions),
        // View › Pages: page-prev / page-next / page-first / page-last /
        // page-add / page-delete.
        (Body::View, Some("Previous")) => Message::Status(StatusMsg::PagePrev),
        (Body::View, Some("Next")) => Message::Status(StatusMsg::PageNext),
        (Body::View, Some("First Page")) => Message::Status(StatusMsg::PageFirst),
        (Body::View, Some("Last Page")) => Message::Status(StatusMsg::PageLast),
        (Body::View, Some("Add")) => Message::Status(StatusMsg::PageAdd),
        (Body::View, Some("Delete")) => Message::Status(StatusMsg::PageDelete),
        (Body::View, Some("View Calcs")) => Message::ToggleCalcs,
        // Tools > Connect = 'disto-connect' (`connectDisto`).
        (Body::Tools, Some("Connect")) => Message::Disto(crate::disto::DistoMsg::Connect),
        // Home > Flip Sketch / View > Background / Interface open their
        // popups under the button (left, bottom).
        (Body::Home, Some("Flip Sketch")) => {
            Message::PopupMenu(PopupMsg::Open(popup_menu::Kind::Flip, 570.0, 124.0))
        }
        (Body::View, Some("Background")) => Message::PopupMenu(PopupMsg::Open(
            popup_menu::Kind::CanvasBackground,
            510.0,
            117.0,
        )),
        (Body::View, Some("Interface")) => Message::PopupMenu(PopupMsg::Open(
            popup_menu::Kind::InterfaceTheme,
            596.0,
            117.0,
        )),
        // View › Mini-Map = 'mini-map' (toggleMiniMap).
        (Body::View, Some("Mini-Map")) => Message::ToggleMiniMap,
        (Body::Tools, Some("Area Codes")) => Message::OpenCodesEditor,
        // Tools › Area Breakdown = 'area-breakdown'.
        (Body::Tools, Some("Area Breakdown")) => Message::OpenBreakdown,
        // Home and Edit Tools invoke the same clipboard and selection actions.
        (Body::Home | Body::EditTools, Some("Cut")) => Message::Cut,
        (Body::Home | Body::EditTools, Some("Copy")) => Message::Copy,
        (Body::Home | Body::EditTools, Some("Paste")) => Message::Paste,
        (Body::Home | Body::EditTools, Some("Select All")) => Message::SelectAll,
        (Body::Home | Body::EditTools, Some("Select")) => Message::OpenSelectMenu,
        (Body::Home, Some("Rotate Sketch")) => Message::RotateSketch,
        (Body::EditTools, Some("Area Props")) => Message::AreaProps,
        // The Home ribbon views the attributes read-only; Edit Tools edits them.
        (Body::Home, Some("Area Attributes")) => Message::OpenAreaAttributes(false),
        (Body::EditTools, Some("Area Attributes")) => Message::OpenAreaAttributes(true),
        (Body::Home | Body::EditTools, Some("Delete")) => Message::Delete,
        // Home / Edit Tools › Undo and Redo (S6-014, S6-015, S6-022).
        (Body::Home | Body::EditTools, Some("Undo")) => Message::Undo,
        (Body::Home | Body::EditTools, Some("Redo")) => Message::Redo,
        // Tools › Options (S11-001): showOptions — the flat options model +
        // dialog gate the small behaviors the port still hardcodes.
        (Body::Tools, Some("Options")) => Message::Options(options_dialog::OptMsg::Open),
        _ => Message::Noop,
    }
}

fn ribbon_button<'a>(b: &Btn, on: Message) -> Element<'a, Message> {
    let alpha = if b.enabled { 1.0 } else { DISABLED_ALPHA };
    let mut body = stack![].width(b.w).height(b.h);
    for gl in &b.glyphs {
        let c = over(gl.color, alpha, bg_panel());
        body = body.push(at(
            gl.x - b.x,
            gl.y - b.y,
            icon(gl.name, gl.fill, gl.size, c),
        ));
    }
    for c in &b.captions {
        let color = over(c.color, alpha, bg_panel());
        body = body.push(at(c.x - b.x, c.y - b.y, caption_el(c, color)));
    }
    let look = b.look;
    let btn = button(body)
        .width(b.w)
        .height(b.h)
        .padding(0)
        .style(move |_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let boxed = |bg: Color| button::Style {
                background: Some(bg.into()),
                border: Border {
                    color: border(),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..button::Style::default()
            };
            match look {
                Look::Toggled => boxed(over(accent(), 0.2, bg_panel())),
                Look::Ribbon if hovered => boxed(bg_hover()),
                _ => button::Style::default(),
            }
        });
    if b.enabled {
        btn.on_press(on).into()
    } else {
        btn.into()
    }
}

/// Chrome's default `<input type="range">` with `accent-color: #858585` at
/// value 50: an 8px rounded track (filled left of the thumb; `#3b3b3b` with
/// a 1px accent rim right of it) and a ~15px round thumb. Geometry sampled
/// from `ribbon-Reference_Image.png`, relative to the input's box.
fn slider<'a>(w: f32, h: f32) -> Element<'a, Message> {
    let rim = muted();
    const EMPTY: Color = hex(0x3b3b3b);
    const TRACK_X: f32 = 1.0;
    const TRACK_Y: f32 = 4.0;
    const THUMB_CX: f32 = 40.0;
    const THUMB_D: f32 = 15.3;
    let pill = |w: f32, fill: Color| -> Element<'a, Message> {
        container(Space::new().width(w).height(8))
            .style(move |_| container::Style {
                background: Some(fill.into()),
                border: Border {
                    color: rim,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..container::Style::default()
            })
            .into()
    };
    // Drawn as an SVG circle: quads snap to whole pixels, but Chrome's
    // thumb is ~15.3px across, centred on the 16px box (sampled).
    let thumb: Element<'a, Message> = iced::widget::svg(iced::widget::svg::Handle::from_memory(
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><circle cx="8" cy="8" r="{}" fill="#858585"/></svg>"##,
            THUMB_D / 2.0
        )
        .into_bytes(),
    ))
    .width(16)
    .height(16)
    .into();
    stack![
        at(TRACK_X, TRACK_Y, pill(w - 2.0 * TRACK_X, EMPTY)),
        at(TRACK_X, TRACK_Y, pill(THUMB_CX + 7.0 - TRACK_X, rim)),
        at(THUMB_CX - 8.0, h / 2.0 - 8.0, thumb),
    ]
    .width(w)
    .height(h)
    .into()
}

/// The Edit Tools body (screen 6) — `.contextual-ribbon#tab-edit-tools`,
/// measured on the live original with a wall selected: 8 base buttons at
/// y 68.41 (h 49.19, 22px icons at y 73.41, 11px captions at y 99.41), the
/// divider at x 410.25, then the line group at x 418.25 — the Length field
/// (label "Length · Start handle" 10px at y 72.5, input 54x23 at y 90.5),
/// Set Length, the Line Style select (92x25 at y 89.5), the Line Color
/// swatch (32x24 at y 72) and Curve.
/// An Edit Tools base button: 46x49.19 at y 68.41, 22px icon, 11px caption.
fn b(x: f32, w: f32, name: &'static str, text: &'static str, tx: f32) -> Btn {
    Btn {
        x,
        y: 68.41,
        w,
        h: 49.19,
        look: Look::Ribbon,
        enabled: true,
        glyphs: vec![g(name, false, 22.0, text_main(), x + w / 2.0 - 11.0, 73.41)],
        captions: vec![Caption {
            text,
            x: tx,
            y: 99.41,
            w,
            line: 13.2,
            size: 11.0,
            font: ARIAL,
            color: text_main(),
        }],
        action: None,
    }
}

/// A small field label above a control.
fn field_label(text: &'static str, x: f32, y: f32, w: f32) -> Caption {
    Caption {
        text,
        x,
        y,
        w,
        line: 14.0,
        size: 10.0,
        font: ARIAL,
        color: text_main(),
    }
}

fn edit_tools(edit: Option<&EditToolsData>) -> Vec<Item> {
    let mut items = vec![
        Item::Btn(b(12.0, 46.0, "scissors", "Cut", 26.44)),
        Item::Btn(b(60.0, 46.0, "copy", "Copy", 70.16)),
        Item::Btn(b(108.0, 46.0, "clipboard-text", "Paste", 116.92)),
        Item::Btn(b(156.0, 55.25, "selection-all", "Select All", 161.0)),
        Item::Btn(b(213.25, 46.0, "intersect", "Select", 220.95)),
        Item::Btn(b(261.25, 46.0, "x", "Delete", 268.34)),
        Item::Btn(b(309.25, 46.0, "arrow-u-up-left", "Undo", 319.09)),
        Item::Btn(b(357.25, 46.0, "arrow-u-up-right", "Redo", 367.09)),
    ];
    // The area group (`contextual-area-tools`) comes first while an area is
    // selected; the groups after it move right by its width.
    if let Some(edit) = edit {
        let mut shift = 0.0;
        if let Some(area) = &edit.area {
            let (group, width) = area_group(area);
            items.extend(group);
            shift = width + 15.0;
        }
        // The line group (`contextual-line-tools`) exists only while a wall
        // is selected — the original hides it for an area-only selection.
        let rest = if edit.has_text {
            text_group(edit.text.as_ref())
        } else if !edit.has_symbols {
            if edit.has_line {
                line_group(edit)
            } else {
                Vec::new()
            }
        } else if edit.hosted {
            symbol_group()
        } else {
            Vec::new()
        };
        items.extend(shifted(rest, shift));
    }
    items
}

/// A group moved `dx` px right (it follows an earlier group in the flow).
fn shifted(items: Vec<Item>, dx: f32) -> Vec<Item> {
    if dx == 0.0 {
        return items;
    }
    items
        .into_iter()
        .map(|item| match item {
            Item::Btn(mut btn) => {
                btn.x += dx;
                btn.glyphs.iter_mut().for_each(|g| g.x += dx);
                btn.captions.iter_mut().for_each(|c| c.x += dx);
                Item::Btn(btn)
            }
            Item::Divider(x) => Item::Divider(x + dx),
            Item::Field {
                mut label,
                box_x,
                box_y,
                box_w,
                box_h,
                value,
                swatch,
                chevron,
                action,
            } => {
                label.x += dx;
                Item::Field {
                    label,
                    box_x: box_x + dx,
                    box_y,
                    box_w,
                    box_h,
                    value,
                    swatch,
                    chevron,
                    action,
                }
            }
            Item::LengthInput {
                mut label,
                box_x,
                box_y,
                box_w,
                value,
            } => {
                label.x += dx;
                Item::LengthInput {
                    label,
                    box_x: box_x + dx,
                    box_y,
                    box_w,
                    value,
                }
            }
            Item::Select {
                mut label,
                box_x,
                box_y,
                box_w,
                value,
                options,
                on_pick,
            } => {
                label.x += dx;
                Item::Select {
                    label,
                    box_x: box_x + dx,
                    box_y,
                    box_w,
                    value,
                    options,
                    on_pick,
                }
            }
            Item::TextInput {
                mut label,
                box_x,
                box_y,
                box_w,
                value,
                field,
            } => {
                label.x += dx;
                Item::TextInput {
                    label,
                    box_x: box_x + dx,
                    box_y,
                    box_w,
                    value,
                    field,
                }
            }
            other => other,
        })
        .collect()
}

/// Edit Tools › the selected area's group: Fill Pattern, Fill Color, Area
/// Props, Area Attributes, Auto Subtract, Line Style, Line Thickness, Line
/// Color and Flip Area, laid out by the flex flow (a field is its 92 px
/// control plus 10 px padding, a colour field 62, a button its text plus 10,
/// 2 px between). Returns the items and the group's width.
fn area_group(area: &AreaTools) -> (Vec<Item>, f32) {
    let start = 418.25;
    let mut x = start;
    let mut items = Vec::new();
    let select = |items: &mut Vec<Item>,
                  x: &mut f32,
                  label: &'static str,
                  value: &'static str,
                  action: Message| {
        items.push(Item::Field {
            label: field_label(label, *x + 5.0, 74.5, 92.0),
            box_x: *x + 5.0,
            box_y: 89.5,
            box_w: 92.0,
            box_h: 25.0,
            value,
            swatch: None,
            chevron: true,
            action: Some(action),
        });
        *x += 102.0 + 2.0;
    };
    let color = |items: &mut Vec<Item>,
                 x: &mut f32,
                 label: &'static str,
                 swatch: Color,
                 action: Message| {
        items.push(Item::Field {
            label: field_label(label, *x, 99.41, 62.0),
            box_x: *x + 15.0,
            box_y: 72.0,
            box_w: 32.0,
            box_h: 24.0,
            value: "",
            swatch: Some(swatch),
            chevron: false,
            action: Some(action),
        });
        *x += 62.0 + 2.0;
    };
    let button = |items: &mut Vec<Item>,
                  x: &mut f32,
                  w: f32,
                  icon: &'static str,
                  text: &'static str,
                  action: Message| {
        let mut btn = if text.contains('\n') {
            // Two lines: the button is a line taller, centred on the row.
            let mut btn = b(*x, w, icon, text, *x);
            btn.y = 61.8;
            btn.h = 62.4;
            btn.glyphs[0].y = 66.8;
            btn.captions[0].y = 92.8;
            btn
        } else {
            b(*x, w, icon, text, *x + 5.0)
        };
        btn.action = Some(action);
        items.push(Item::Btn(btn));
        *x += w + 2.0;
    };
    use crate::area_props::AreaCycle;
    select(
        &mut items,
        &mut x,
        "Fill Pattern",
        area.pattern,
        Message::AreaCycle(AreaCycle::FillPattern),
    );
    color(
        &mut items,
        &mut x,
        "Fill Color",
        area.fill,
        Message::AreaColorClicked(true),
    );
    button(
        &mut items,
        &mut x,
        64.3,
        "house-line",
        "Area Props",
        Message::AreaProps,
    );
    button(
        &mut items,
        &mut x,
        55.9,
        "list-dashes",
        "Area\nAttributes",
        Message::OpenAreaAttributes(true),
    );
    button(
        &mut items,
        &mut x,
        50.5,
        "selection-background",
        "Auto\nSubtract",
        Message::ContextAction(crate::context_menu::Action::AutoSubtract),
    );
    select(
        &mut items,
        &mut x,
        "Line Style",
        area.line_style,
        Message::AreaCycle(AreaCycle::LineStyle),
    );
    select(
        &mut items,
        &mut x,
        "Line Thickness",
        area.thickness,
        Message::AreaCycle(AreaCycle::LineThickness),
    );
    color(
        &mut items,
        &mut x,
        "Line Color",
        area.line_color,
        Message::AreaColorClicked(false),
    );
    button(
        &mut items,
        &mut x,
        53.4,
        "arrows-left-right",
        "Flip Area",
        Message::ContextAction(crate::context_menu::Action::FlipHorz),
    );
    (items, x - 2.0 - start)
}

/// The selected wall's length/style/colour group, after the divider.
fn line_group(edit: &EditToolsData) -> Vec<Item> {
    vec![
        Item::Divider(410.25),
        // The selected wall's length; the endpoint the edit moves is named in
        // the label.
        Item::LengthInput {
            label: field_label(edit.endpoint_label, 423.25, 74.5, 91.2),
            box_x: 423.25,
            box_y: 90.5,
            box_w: 54.0,
            value: edit.length_text.clone(),
        },
        Item::Btn({
            let mut set = b(521.45, 63.22, "ruler", "Set Length", 526.45);
            set.action = Some(Message::ApplyLineLength);
            set
        }),
        Item::Field {
            label: field_label("Line Style", 591.67, 74.5, 92.0),
            box_x: 591.67,
            box_y: 89.5,
            box_w: 92.0,
            box_h: 25.0,
            value: edit.style_text,
            swatch: None,
            chevron: true,
            action: Some(Message::CycleLineStyle),
        },
        Item::Field {
            label: field_label("Line Color", 690.67, 99.41, 62.0),
            box_x: 705.67,
            box_y: 72.0,
            box_w: 32.0,
            box_h: 24.0,
            value: "",
            swatch: Some(edit.color),
            chevron: false,
            action: Some(Message::LineColorClicked),
        },
        Item::Btn({
            let mut curve = b(754.67, 46.0, "bezier-curve", "Curve", 763.0);
            curve.action = Some(Message::CurveSelected);
            curve
        }),
    ]
}

/// The Home body (Screen 1, accepted) — positions from
/// `refs/ui-spec/chrome-spec-home.json`.
/// Screen 8: flip/opening-width buttons for a wall-hosted symbol selection.
fn symbol_group() -> Vec<Item> {
    vec![
        Item::Divider(410.25),
        Item::Btn({
            let mut btn = b(418.25, 46.0, "arrows-left-right", "Flip", 426.5);
            btn.action = Some(Message::FlipOpening);
            btn
        }),
        Item::Btn({
            let mut btn = b(464.25, 63.22, "plus", "Wider", 472.0);
            btn.action = Some(Message::OpeningWider);
            btn
        }),
        Item::Btn({
            let mut btn = b(527.47, 63.22, "minus", "Narrower", 535.0);
            btn.action = Some(Message::OpeningNarrower);
            btn
        }),
    ]
}

/// Screen 9: text styling buttons.
/// Edit Tools › the text group, in the original's order: Font, Size, Bold,
/// Italic, Underline, Text Color, Grow, Shrink, Text (laid out by the flex
/// flow: a select / colour field is its control plus 10 px padding, 2 px
/// between). Widths: Font 102, Size 64, Text Color 62, the Text field 160.
fn text_group(tools: Option<&TextTools>) -> Vec<Item> {
    let fallback = TextTools {
        font: "Arial".into(),
        size: "12".into(),
        color: hex(0xE2E8F0),
        value: String::new(),
    };
    let tools = tools.unwrap_or(&fallback);
    let mut x = 418.25;
    let mut items = vec![Item::Divider(410.25)];
    // Font.
    items.push(Item::Select {
        label: field_label("Font", x + 5.0, 74.5, 92.0),
        box_x: x + 5.0,
        box_y: 89.5,
        box_w: 92.0,
        value: tools.font.clone(),
        options: TEXT_FONTS.iter().map(|f| f.to_string()).collect(),
        on_pick: Message::TextFontPicked,
    });
    x += 102.0 + 2.0;
    // Size.
    items.push(Item::TextInput {
        label: field_label("Size", x + 5.0, 74.5, 54.0),
        box_x: x + 5.0,
        box_y: 89.5,
        box_w: 54.0,
        value: tools.size.clone(),
        field: TextField::Size,
    });
    x += 64.0 + 2.0;
    let button = |items: &mut Vec<Item>,
                  x: &mut f32,
                  w: f32,
                  icon: &'static str,
                  text: &'static str,
                  tx: f32,
                  action: Message| {
        let mut btn = b(*x, w, icon, text, *x + tx);
        btn.action = Some(action);
        items.push(Item::Btn(btn));
        *x += w + 2.0;
    };
    button(
        &mut items,
        &mut x,
        46.0,
        "text-b",
        "Bold",
        8.25,
        Message::ToggleSelectedTextStyle("bold".into()),
    );
    button(
        &mut items,
        &mut x,
        46.0,
        "text-italic",
        "Italic",
        7.75,
        Message::ToggleSelectedTextStyle("italic".into()),
    );
    button(
        &mut items,
        &mut x,
        55.0,
        "text-underline",
        "Underline",
        5.75,
        Message::ToggleSelectedTextStyle("underline".into()),
    );
    // Text Color.
    items.push(Item::Field {
        label: field_label("Text Color", x, 99.41, 62.0),
        box_x: x + 15.0,
        box_y: 72.0,
        box_w: 32.0,
        box_h: 24.0,
        value: "",
        swatch: Some(tools.color),
        chevron: false,
        action: Some(Message::TextColorClicked),
    });
    x += 62.0 + 2.0;
    button(
        &mut items,
        &mut x,
        46.0,
        "text-aa",
        "Grow",
        7.75,
        Message::AdjustSelectedTextSize(1),
    );
    button(
        &mut items,
        &mut x,
        55.0,
        "text-a-underline",
        "Shrink",
        6.75,
        Message::AdjustSelectedTextSize(-1),
    );
    // Text.
    items.push(Item::TextInput {
        label: field_label("Text", x + 5.0, 74.5, 150.0),
        box_x: x + 5.0,
        box_y: 89.5,
        box_w: 150.0,
        value: tools.value.clone(),
        field: TextField::Value,
    });
    items
}

fn home() -> Vec<Item> {
    let b = |x: f32, name: &'static str, color: Color, icon_x: f32, text: &'static str, tx: f32| {
        let (iy, ty) = if text.contains('\n') {
            (67.0, 93.0)
        } else {
            (73.0, 99.0)
        };
        Btn {
            x,
            y: 62.0,
            w: 60.0,
            h: 62.0,
            look: Look::Ribbon,
            enabled: true,
            glyphs: vec![g(name, false, 22.0, color, icon_x, iy)],
            captions: vec![cap(text, tx, ty, 0.0, 13.0, 11.0, ARIAL, text_main())],
            action: None,
        }
    };
    let caret = |x: f32| g("caret-down", true, 8.0, text_main(), x, 101.0);
    let disabled = |mut s: Btn| {
        s.enabled = false;
        s
    };
    let two_line = |mut s: Btn, w: f32| {
        s.captions[0].w = w;
        s
    };
    let mut select = b(260.0, "intersect", ICON_BLUE, 279.0, "Select", 269.0);
    select.glyphs[0].fill = true;
    select.glyphs.push(caret(303.0));
    let mut flip = b(
        570.0,
        "arrows-left-right",
        text_main(),
        599.0,
        "Flip Sketch",
        577.0,
    );
    flip.w = 79.0;
    flip.glyphs.push(caret(634.0));
    let mut draw_color = b(
        674.0,
        "pencil-simple",
        ICON_ORANGE,
        704.0,
        "Draw Color",
        681.0,
    );
    draw_color.y = 68.0;
    draw_color.w = 80.0;
    draw_color.h = 49.0;
    draw_color.glyphs.push(caret(740.0));
    let mut subject = b(780.0, "house-line", ICON_BLUE, 805.0, "Subject Info", 787.0);
    subject.w = 72.0;
    subject.action = Some(Message::OpenSubjectInfo);
    vec![
        Item::Btn(b(12.0, "scissors", ICON_BLUE, 31.0, "Cut", 33.0)),
        Item::Btn(b(74.0, "copy", ICON_ORANGE, 93.0, "Copy", 91.0)),
        Item::Btn(b(
            136.0,
            "clipboard-text",
            ICON_ORANGE,
            155.0,
            "Paste",
            152.0,
        )),
        Item::Btn(b(
            198.0,
            "selection-all",
            ICON_BLUE,
            217.0,
            "Select All",
            205.0,
        )),
        Item::Btn(select),
        // Delete's glyph is font-weight 700 (synthetic bold) — the bold cut.
        Item::Btn(b(322.0, "x-bold", ICON_RED, 341.0, "Delete", 336.0)),
        Item::Btn(disabled(b(
            384.0,
            "arrow-u-up-left",
            text_main(),
            403.0,
            "Undo",
            401.0,
        ))),
        Item::Btn(disabled(b(
            446.0,
            "arrow-u-up-right",
            text_main(),
            465.0,
            "Redo",
            463.0,
        ))),
        Item::Btn(two_line(
            b(
                508.0,
                "arrow-clockwise",
                ICON_BLUE,
                527.0,
                "Rotate\nSketch",
                521.0,
            ),
            34.0,
        )),
        Item::Btn(flip),
        Item::Divider(661.0),
        Item::Btn(draw_color),
        Item::Divider(767.0),
        Item::Btn(subject),
        Item::Btn(two_line(
            b(
                854.0,
                "list-dashes",
                ICON_SLATE,
                873.0,
                "Area\nAttributes",
                861.0,
            ),
            46.0,
        )),
    ]
}

/// The reference image's ribbon state (the Edit Mode toggle + the
/// intensity value the slider shows).
#[derive(Debug, Clone, Copy, Default)]
pub struct PhotoRibbon {
    pub edit_active: bool,
    pub intensity: f64,
}

/// The items of `body`.
pub fn items(body: Body, edit: Option<&EditToolsData>, photo: PhotoRibbon) -> Vec<Item> {
    match body {
        Body::EditTools => edit_tools(edit),
        Body::Home => home(),
        Body::View => ribbon_tabs::view(),
        Body::Tools => ribbon_tabs::tools(),
        Body::ReferenceImage => ribbon_tabs::reference_image(photo.edit_active, photo.intensity),
        Body::Help => ribbon_tabs::help(),
    }
}

/// The ribbon body's top edge in page coordinates.
const BODY_Y: f32 = 58.0;

/// `prev_enabled`: View › Previous follows updatePageDisplay (disabled on
/// the first page).
#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    active: TabId,
    body: Body,
    prev_enabled: bool,
    edit: Option<&EditToolsData>,
    undo_enabled: bool,
    redo_enabled: bool,
    photo: PhotoRibbon,
    disto_chip: Option<&str>,
) -> Element<'a, Message> {
    let edit_tools = edit.is_some();
    // Tab strip: 28px tall, tabs start 4px down (padding-top). The contextual
    // Edit Tools tab joins at the end while a selection exists.
    let mut tabs = row![];
    for &(id, name, w) in TABS {
        tabs = tabs.push(tab(id, name, w, id == active));
    }
    if edit_tools {
        tabs = tabs.push(tab(
            TabId::EditTools,
            "Edit Tools",
            EDIT_TOOLS_TAB.1,
            active == TabId::EditTools,
        ));
    }
    let strip = container(tabs)
        .width(Length::Fill)
        .height(28)
        .padding(Padding {
            top: 4.0,
            ..Padding::ZERO
        });

    // Body: positions are absolute page coords; the body starts at y=58.
    let mut content = stack![].width(Length::Fill).height(70);
    for item in items(body, edit, photo) {
        content = content.push(match item {
            Item::Btn(mut b) => {
                if body == Body::View && b.captions.first().map(|c| c.text) == Some("Previous") {
                    b.enabled = prev_enabled;
                }
                // Undo/Redo follow the stacks (`updateUndoRedoControls`).
                match b.captions.first().map(|c| c.text) {
                    Some("Undo") if matches!(body, Body::Home | Body::EditTools) => {
                        b.enabled = undo_enabled
                    }
                    Some("Redo") if matches!(body, Body::Home | Body::EditTools) => {
                        b.enabled = redo_enabled
                    }
                    // `#disto-status` takes the chosen device's name.
                    Some("Disconnected") if body == Body::Tools => {
                        if let Some(name) = disto_chip {
                            b.captions[0] = chip_caption(&b.captions[0], b.x, b.w, name);
                        }
                    }
                    _ => {}
                }
                {
                    // An explicit action (set where the group is built) wins
                    // over the caption-keyed command table.
                    let on = b.action.clone().unwrap_or_else(|| command(body, &b));
                    let title = crate::tooltips::ribbon_title(
                        body,
                        b.captions.first().map(|c| c.text).unwrap_or(""),
                    );
                    at(
                        b.x,
                        b.y - BODY_Y,
                        crate::tooltips::tip_below(ribbon_button(&b, on), title),
                    )
                }
            }
            Item::Divider(x) => at(x, 73.0 - BODY_Y, rect(1.0, 40.0, border())),
            Item::Caption(c) => at(c.x, c.y - BODY_Y, caption_el(&c, c.color)),
            Item::Glyph(gl) => at(
                gl.x,
                gl.y - BODY_Y,
                icon(gl.name, gl.fill, gl.size, gl.color),
            ),
            Item::Slider { x, y, w, h, value } => stack![
                at(x, y - BODY_Y, slider(w, h)),
                // An invisible iced slider on top takes the input (the
                // statusbar's zoom slider pattern).
                iced::widget::slider::<f64, Message, iced::Theme>(0.0..=100.0, value, |n| {
                    Message::PhotoIntensity(n as f32)
                },)
                .step(1.0)
                .width(w)
                .height(h)
                .style(|_, _| iced_slider::Style {
                    rail: iced_slider::Rail {
                        backgrounds: (
                            Background::Color(Color::TRANSPARENT),
                            Background::Color(Color::TRANSPARENT),
                        ),
                        width: 4.0,
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 0.0.into(),
                        },
                    },
                    handle: iced_slider::Handle {
                        shape: iced_slider::HandleShape::Circle { radius: 7.5 },
                        background: Background::Color(Color::TRANSPARENT),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                }),
            ]
            .into(),
            Item::Select {
                label,
                box_x,
                box_y,
                box_w,
                value,
                options,
                on_pick,
            } => {
                // The drawn box (text + chevron) with an invisible pick_list on
                // top taking the choice, as the status bar's selects do.
                let mut cell = stack![].width(box_w).height(25);
                cell = cell.push(at(
                    5.0,
                    4.0,
                    text_line(value.clone(), 13.33, ARIAL, text_main(), 17.0),
                ));
                cell = cell.push(at(
                    box_w - 20.0,
                    25.0 / 2.0 - 6.0,
                    icon("caret-down", true, 12.0, text_main()),
                ));
                let shown = options.iter().find(|o| **o == value).cloned();
                let pick = iced::widget::pick_list(options, shown, on_pick)
                    .width(box_w)
                    .padding(0)
                    .text_size(13)
                    .handle(iced::widget::pick_list::Handle::None)
                    .menu_style(|_| iced::overlay::menu::Style {
                        background: Background::Color(bg_panel()),
                        border: Border {
                            color: border(),
                            width: 1.0,
                            radius: 0.0.into(),
                        },
                        text_color: text_main(),
                        selected_text_color: WHITE,
                        selected_background: Background::Color(accent()),
                        shadow: iced::Shadow::default(),
                    })
                    .style(|_, _| iced::widget::pick_list::Style {
                        text_color: Color::TRANSPARENT,
                        placeholder_color: Color::TRANSPARENT,
                        handle_color: Color::TRANSPARENT,
                        background: Background::Color(Color::TRANSPARENT),
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 0.0.into(),
                        },
                    });
                let boxed_cell: Element<'a, Message> = stack![
                    boxed(cell, box_w, 25, Some(bg_app()), border(), 0.0),
                    container(pick).width(box_w).height(25),
                ]
                .into();
                let mut group = stack![].width(Length::Fill).height(70);
                group = group.push(at(box_x, box_y - BODY_Y, boxed_cell));
                group = group.push(at(
                    label.x,
                    label.y - BODY_Y,
                    caption_el(&label, label.color),
                ));
                group.into()
            }
            Item::TextInput {
                label,
                box_x,
                box_y,
                box_w,
                value,
                field,
            } => {
                let input = iced::widget::text_input("", &value)
                    .on_input(move |s| Message::TextFieldInput(field, s))
                    .on_submit(Message::TextFieldApply(field))
                    .font(ARIAL)
                    .size(13.33)
                    .line_height(iced::widget::text::LineHeight::Absolute(Pixels(17.0)))
                    .padding(Padding {
                        top: 2.0,
                        bottom: 2.0,
                        left: 5.0,
                        right: 3.0,
                    })
                    .width(box_w)
                    .style(|_, _| iced::widget::text_input::Style {
                        background: Background::Color(bg_app()),
                        border: Border {
                            color: border(),
                            width: 1.0,
                            radius: 0.0.into(),
                        },
                        icon: muted(),
                        placeholder: muted(),
                        value: text_main(),
                        selection: crate::dialog::SELECTION,
                    });
                let mut group = stack![].width(Length::Fill).height(70);
                group = group.push(at(box_x, box_y - BODY_Y, input));
                group = group.push(at(
                    label.x,
                    label.y - BODY_Y,
                    caption_el(&label, label.color),
                ));
                group.into()
            }
            Item::LengthInput {
                label,
                box_x,
                box_y,
                box_w,
                value,
            } => {
                let input = iced::widget::text_input("", &value)
                    .id(LENGTH_INPUT_ID)
                    .on_input(Message::LineLengthInput)
                    .on_submit(Message::ApplyLineLength)
                    .font(ARIAL)
                    .size(13.33)
                    .line_height(iced::widget::text::LineHeight::Absolute(Pixels(17.0)))
                    .padding(Padding {
                        top: 2.0,
                        bottom: 2.0,
                        left: 5.0,
                        right: 3.0,
                    })
                    .width(box_w)
                    .style(|_, _| iced::widget::text_input::Style {
                        background: Background::Color(bg_app()),
                        border: Border {
                            color: border(),
                            width: 1.0,
                            radius: 0.0.into(),
                        },
                        icon: muted(),
                        placeholder: muted(),
                        value: text_main(),
                        selection: crate::dialog::SELECTION,
                    });
                let mut group = stack![].width(Length::Fill).height(70);
                group = group.push(at(box_x, box_y - BODY_Y, input));
                group = group.push(at(
                    label.x,
                    label.y - BODY_Y,
                    caption_el(&label, label.color),
                ));
                group.into()
            }
            Item::Field {
                label,
                box_x,
                box_y,
                box_w,
                box_h,
                value,
                swatch,
                chevron,
                action,
            } => {
                // The box: `--bg-app` with a 1px `--border`; a colour input
                // shows its swatch instead of text, a select its chevron.
                let mut cell = stack![].width(box_w).height(box_h);
                if let Some(color) = swatch {
                    cell = cell.push(rect(box_w, box_h, color));
                } else {
                    cell = cell.push(at(
                        5.0,
                        4.0,
                        text_line(value, 13.33, ARIAL, text_main(), 17.0),
                    ));
                }
                if chevron {
                    cell = cell.push(at(
                        box_w - 20.0,
                        box_h / 2.0 - 6.0,
                        icon("caret-down", true, 12.0, text_main()),
                    ));
                }
                let boxed_cell: Element<'a, Message> =
                    boxed(cell, box_w, box_h, Some(bg_app()), border(), 0.0).into();
                let boxed_cell = match action {
                    Some(msg) => iced::widget::mouse_area(boxed_cell).on_press(msg).into(),
                    None => boxed_cell,
                };
                // The children sit at absolute page coords, so the wrapper must
                // span the whole body — a box-sized stack would clip them.
                let mut group = stack![].width(Length::Fill).height(70);
                group = group.push(at(box_x, box_y - BODY_Y, boxed_cell));
                group = group.push(at(
                    label.x,
                    label.y - BODY_Y,
                    caption_el(&label, label.color),
                ));
                group.into()
            }
        });
    }

    let content = fill_bg(content, bg_panel()).width(Length::Fill).height(70);

    fill_bg(
        column![
            strip,
            content,
            fill_bg(Space::new().width(Length::Fill).height(1), border()).width(Length::Fill),
        ],
        bg_ribbon(),
    )
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every enabled button/field must have an action. Behavior is exercised
    /// separately by the real-widget tests in e2e.rs.
    #[test]
    fn control_wiring_audit_ribbon() {
        let mut rows = Vec::new();
        let mut scan = |case: &str, body, edit: Option<&EditToolsData>| {
            for item in items(body, edit, PhotoRibbon::default()) {
                match item {
                    Item::Btn(b) => {
                        let action = b.action.clone().unwrap_or_else(|| command(body, &b));
                        rows.push(serde_json::json!({
                            "case": case, "label": b.captions.first().map(|c| c.text),
                            "enabled": b.enabled, "action": format!("{action:?}"),
                            "unwired": b.enabled && matches!(action, Message::Noop),
                            "point": [b.x + b.w / 2., b.y + b.h / 2.]
                        }));
                    }
                    Item::Field { label, action, .. } => rows.push(serde_json::json!({
                        "case": case, "label": label.text, "enabled": true,
                        "unwired": action.is_none() || matches!(action, Some(Message::Noop)),
                        "action": format!("{action:?}")
                    })),
                    _ => {}
                }
            }
        };
        for body in [
            Body::Home,
            Body::View,
            Body::Tools,
            Body::ReferenceImage,
            Body::Help,
        ] {
            scan(&format!("{body:?}"), body, None);
        }
        let mut edit = EditToolsData {
            length_text: "10".into(),
            endpoint_label: "End",
            style_text: "Solid",
            color: text_main(),
            has_symbols: false,
            hosted: false,
            has_text: false,
            has_line: true,
            text: None,
            area: None,
        };
        scan("EditTools/wall", Body::EditTools, Some(&edit));
        edit.area = Some(AreaTools {
            pattern: "Solid",
            fill: text_main(),
            line_style: "Solid",
            thickness: "4",
            line_color: text_main(),
        });
        scan("EditTools/area+wall", Body::EditTools, Some(&edit));
        edit.has_line = false;
        scan("EditTools/area", Body::EditTools, Some(&edit));
        edit.area = None;
        edit.has_text = true;
        edit.text = Some(TextTools {
            font: "Arial".into(),
            size: "12".into(),
            color: text_main(),
            value: "Test".into(),
        });
        scan("EditTools/text", Body::EditTools, Some(&edit));
        edit.has_text = false;
        edit.has_symbols = true;
        scan("EditTools/free symbol", Body::EditTools, Some(&edit));
        edit.hosted = true;
        scan("EditTools/hosted symbol", Body::EditTools, Some(&edit));
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/control-audit");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("ribbon.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
        for row in rows.iter().filter(|r| r["unwired"] == true) {
            println!("UNWIRED {row}");
        }
        println!("Inventoried {} ribbon button/field instances", rows.len());
        assert!(
            rows.iter().all(|r| r["unwired"] != true),
            "enabled ribbon control has no action"
        );
    }

    fn disabled(b: Body) -> Vec<&'static str> {
        items(b, None, PhotoRibbon::default())
            .into_iter()
            .filter_map(|i| match i {
                Item::Btn(b) if !b.enabled => Some(b.captions[0].text),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn disabled_buttons_match_the_original_per_tab() {
        assert_eq!(disabled(Body::Home), ["Undo", "Redo"]);
        assert_eq!(disabled(Body::View), ["Previous"]);
        assert_eq!(disabled(Body::Tools), ["Disconnected"]);
        assert!(disabled(Body::ReferenceImage).is_empty());
        assert!(disabled(Body::Help).is_empty());
    }

    #[test]
    fn tab_strip_matches_the_spec() {
        let total: f32 = TABS.iter().map(|t| t.2).sum();
        // File 0..51 … Help 357..414.
        assert_eq!(total, 414.0);
        assert_eq!(TABS[0], (TabId::File, "File", FILE_TAB_W));
        for (id, _, _) in TABS {
            assert_eq!(id.body().is_none(), *id == TabId::File);
        }
    }
}
