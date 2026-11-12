//! Status bar (0,832 1400x28, `--bg-ribbon`, 1px `--border` top): view /
//! grid / snap buttons, snap-pitch select, filter, totals chip; zoom slider,
//! scale readout + 1:N input, defined-areas select, segment HUD; north
//! indicator, page navigator, underlay select, Bluetooth, Touch Panel.
//!
//! Live controls (screen 4a) follow refs/ui-manager.js bindEvents +
//! updateZoomDisplay; state lives in `view_state::View`. The centre block
//! (`.status-center`, flex, gap 12px) sits at its natural width between
//! equal flexible spacers, so a narrower readout ("+ 83%") re-centres it
//! exactly as the original does.

use iced::widget::text::LineHeight;
use iced::widget::{button, container, pick_list, row, slider, stack, svg, text_input, Space};
use iced::{alignment, Background, Border, Color, Element, Length, Padding, Pixels, Theme};

use crate::icons::{icon, ph, rotated};
use crate::pages::Pages;
use crate::snap::{snap, snap_floor};
use crate::theme::*;
use crate::view_state::{Mode, View, PITCHES};
use crate::Message;

/// Top of the status bar's content row on the page (below its border).
const TOP: f32 = 833.0;

/// `.status-btn#btn-zoom-fence.active`.
const FENCE_RED: Color = hex(0xff3b30);

#[derive(Debug, Clone)]
pub enum StatusMsg {
    CenterSketch,
    CenterFit,
    ZoomFence,
    ToggleGrid,
    /// View › Apply (Dims): outside -> inside -> hidden.
    CycleDimensions,
    ToggleSnap,
    ToggleAreaSnap,
    Pitch(PitchOption),
    ToggleFilter,
    /// Filter checkbox `index` (FILTERS order) toggled.
    Filter(usize),
    TotalsChip,
    ZoomOut,
    ZoomIn,
    ZoomSlider(f64),
    ScaleEdit(String),
    ScaleSubmit,
    North,
    PageDelete,
    PagePrev,
    PageNext,
    PageAdd,
    PageFirst,
    PageLast,
    Underlay(Option<usize>),
    /// The defined-areas select: that area becomes the selection.
    AreaPick(i64),
}

/// An `#area-snap-pitch` option.
#[derive(Debug, Clone, PartialEq)]
pub struct PitchOption {
    pub label: &'static str,
    pub value: f64,
}

impl std::fmt::Display for PitchOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label)
    }
}

fn msg(m: StatusMsg) -> Message {
    Message::Status(m)
}

/// Chrome's dark-scheme `<select>` chevron (9x6, ~1.8px stroke).
const CHEVRON: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 9 6" width="9" height="6"><path d="M0.6 0.6 L4.5 4.7 L8.4 0.6" fill="none" stroke="#ffffff" stroke-width="1.7"/></svg>"##;

fn chevron<'a>(color: Color) -> Element<'a, Message> {
    svg(svg::Handle::from_memory(CHEVRON.as_bytes()))
        .width(9)
        .height(6)
        .style(move |_, _| svg::Style { color: Some(color) })
        .into()
}

/// `.status-btn` — 22x22, 16px icon centered.
fn status_btn<'a>(
    name: &str,
    color: Color,
    on: Message,
    title: &'static str,
) -> Element<'a, Message> {
    let b: Element<'a, Message> = button(container(icon(name, false, 16.0, color)).padding(3))
        .width(22)
        .height(22)
        .padding(0)
        .on_press(on)
        .style(|_, status| button::Style {
            background: match status {
                button::Status::Hovered | button::Status::Pressed => Some(bg_hover().into()),
                _ => None,
            },
            ..button::Style::default()
        })
        .into();
    crate::tooltips::tip(b, title)
}

/// A plain icon button with no box (`.page-nav`, `.zoom-slider button`).
fn bare_btn<'a>(content: Element<'a, Message>, on: Message) -> Element<'a, Message> {
    button(content)
        .padding(0)
        .on_press(on)
        .style(|_, _| button::Style::default())
        .into()
}

/// Invisible input widgets: the visible control is drawn underneath to the
/// measured pixels; these only take the input.
fn clear_border() -> Border {
    Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: 0.0.into(),
    }
}

fn left<'a>(v: &View, totals: crate::calcs::AreaTotals, area_count: usize) -> Element<'a, Message> {
    let y = |abs: f32| abs - TOP;
    let on = |b: bool| if b { accent() } else { text_main() };
    let fence = if v.mode == Mode::ZoomFence {
        FENCE_RED
    } else {
        text_main()
    };
    // status-left ends at 8 + 364.656 (live rect); the centre block's
    // space-between position depends on it.
    let mut s = stack![].width(372.656).height(27);
    let btns: [(f32, &str, Color, StatusMsg, &'static str); 7] = [
        (
            8.0,
            "corners-out",
            text_main(),
            StatusMsg::CenterSketch,
            "Center",
        ),
        (
            34.0,
            "arrows-out",
            text_main(),
            StatusMsg::CenterFit,
            "Center & Fit",
        ),
        (
            60.0,
            "magnifying-glass",
            fence,
            StatusMsg::ZoomFence,
            "Zoom Fence",
        ),
        (86.0, "grid-four", on(v.grid), StatusMsg::ToggleGrid, "Grid"),
        (112.0, "magnet", on(v.snap), StatusMsg::ToggleSnap, "Snap"),
        (
            138.0,
            "bounding-box",
            on(v.area_snap),
            StatusMsg::ToggleAreaSnap,
            "Area Grid Snap",
        ),
        (
            214.0,
            "funnel",
            text_main(),
            StatusMsg::ToggleFilter,
            "Filter",
        ),
    ];
    for (x, name, c, m, title) in btns {
        s = s.push(at(x, y(836.0), status_btn(name, c, msg(m), title)));
    }
    // `.snap-pitch-select` 46x20: drawn, with an invisible pick_list on top.
    let options: Vec<PitchOption> = PITCHES
        .iter()
        .map(|&(label, value)| PitchOption { label, value })
        .collect();
    let selected = options.iter().find(|o| o.label == v.pitch_label()).cloned();
    let pick = pick_list(options, selected, |o| msg(StatusMsg::Pitch(o)))
        .width(46)
        .padding(0)
        .text_size(10)
        .handle(pick_list::Handle::None)
        .menu_style(menu_style)
        .style(invisible_pick);
    s = s.push(at(
        164.0,
        y(837.0),
        crate::tooltips::tip(
            stack![
                boxed(
                    stack![
                        at(
                            8.0,
                            4.0,
                            label(v.pitch_label(), 10.0, ARIAL, text_main(), 12.0)
                        ),
                        at(32.0, 7.0, chevron(text_main())),
                    ],
                    46,
                    20,
                    Some(bg_app()),
                    border(),
                    0.0,
                ),
                container(pick).width(46).height(20),
            ],
            "Snap grid pitch",
        ),
    ));
    // `.totals-chip` 133x18 — opens the Calcs panel.
    let sep = over(muted(), 0.45, bg_panel());
    // Content-sized like the original's inline-flex chip: 10px padding + the
    // 1px border, 6px between the three runs, 18px tall.
    let (gla, count) = chip_texts(totals, area_count);
    let chip = container(
        row![
            label(&gla, 11.5, ARIAL_BOLD, accent(), 13.0),
            label("·", 11.5, ARIAL, sep, 13.0),
            label(&count, 11.5, ARIAL, muted(), 13.0),
        ]
        .spacing(6),
    )
    .height(18)
    .padding(Padding {
        top: 2.0,
        right: 10.0,
        bottom: 0.0,
        left: 10.0,
    })
    .style(move |_| container::Style {
        background: Some(Background::Color(bg_panel())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: 3.0.into(),
        },
        ..container::Style::default()
    });
    s = s.push(at(
        240.0,
        y(838.0),
        crate::tooltips::tip(
            button(chip)
                .padding(0)
                .on_press(msg(StatusMsg::TotalsChip))
                .style(|_, _| button::Style::default()),
            "Calculation summary - click to open the Calcs panel",
        ),
    ));
    s.into()
}

/// `.zoom-slider` (115x15): minus, the 80px Chrome range, plus. The range is
/// drawn from the value (thumb travels 65px over 10..500); an invisible
/// iced slider on top takes the input.
fn zoom_slider<'a>(v: &View) -> Element<'a, Message> {
    let value = (v.zoom_percent() as f64).clamp(10.0, 500.0);
    // Thumb box left edge in group px: the range starts at 17 (page 401);
    // its 15px thumb travels 65px (Screen 1 measured 413 at 100%).
    let thumb_left = 17.0 + 65.0 * ((value - 10.0) / 490.0) as f32;
    let track_light = hex(0xefefef);
    let fill = (thumb_left + 15.0 - 17.5).max(0.0);
    let slider_el: Element<'a, Message> =
        slider::<f64, Message, Theme>(10.0..=500.0, value, |n| msg(StatusMsg::ZoomSlider(n)))
            .step(1.0)
            .width(80)
            .height(15)
            .style(|_, _| slider::Style {
                rail: slider::Rail {
                    backgrounds: (
                        Background::Color(Color::TRANSPARENT),
                        Background::Color(Color::TRANSPARENT),
                    ),
                    width: 4.0,
                    border: clear_border(),
                },
                handle: slider::Handle {
                    shape: slider::HandleShape::Circle { radius: 7.5 },
                    background: Background::Color(Color::TRANSPARENT),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            })
            .into();
    let input: Element<'a, Message> = slider_el;
    stack![
        at(
            0.0,
            0.0,
            bare_btn(ph("minus", 13.0, text_main()), msg(StatusMsg::ZoomOut))
        ),
        // Chrome dark-scheme range: ~78x3.4 rounded bar (partial top/bottom
        // rows), accent fill to the thumb's right edge, 15px accent thumb.
        at(
            17.5,
            6.0,
            boxed(Space::new(), 78, 4, Some(hex(0xb2b2b2)), hex(0xb2b2b2), 2.0)
        ),
        at(
            17.5,
            7.0,
            boxed(Space::new(), 78, 2, Some(track_light), track_light, 1.0)
        ),
        at(
            17.5,
            6.0,
            boxed(
                Space::new(),
                fill,
                4,
                Some(hex(0x3b78a1)),
                hex(0x3b78a1),
                2.0
            )
        ),
        at(
            17.5,
            7.0,
            boxed(Space::new(), fill, 2, Some(accent()), accent(), 1.0)
        ),
        at(
            thumb_left,
            0.5,
            boxed(Space::new(), 15, 15, Some(accent()), accent(), 7.5)
        ),
        at(17.0, 0.0, input),
        at(
            101.0,
            0.0,
            bare_btn(ph("plus", 13.0, text_main()), msg(StatusMsg::ZoomIn))
        ),
    ]
    .width(115)
    .height(15)
    .into()
}

/// `.scale-input` 40x21 — Chrome lays the value at the content-box start.
fn scale_input<'a>(value: String) -> Element<'a, Message> {
    text_input("", &value)
        .on_input(|s| msg(StatusMsg::ScaleEdit(s)))
        .on_submit(msg(StatusMsg::ScaleSubmit))
        .font(ARIAL)
        .size(13.333)
        .line_height(LineHeight::Absolute(Pixels(15.0)))
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 5.0,
            right: 5.0,
        })
        .width(40)
        .style(|_, _| text_input::Style {
            background: Background::Color(bg_app()),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 0.0.into(),
            },
            icon: muted(),
            placeholder: muted(),
            value: text_main(),
            selection: Color { a: 0.4, ..accent() },
        })
        .into()
}

/// `#area-status-select` with no areas: "No defined areas 0.0". Once
/// updateCalcsPanel has run it is `disabled` — measured as muted text with
/// the whole control at 0.7 opacity over the status bar.
fn areas_select<'a>(
    disabled: bool,
    width: f32,
    options: &[AreaOption],
    selected: Option<i64>,
) -> Element<'a, Message> {
    if !options.is_empty() {
        // Populated: enabled, showing the chosen area (the selected one, else
        // the last choice, else the last area); the real select on top.
        let shown = options
            .iter()
            .find(|o| Some(o.id) == selected)
            .unwrap_or(&options[options.len() - 1])
            .clone();
        let pick = pick_list(options.to_vec(), Some(shown.clone()), |o| {
            msg(StatusMsg::AreaPick(o.id))
        })
        .width(width)
        .padding(0)
        .text_size(11)
        .font(CONSOLAS)
        .handle(pick_list::Handle::None)
        .menu_style(menu_style)
        .style(invisible_pick);
        return stack![
            boxed(
                stack![
                    at(
                        12.0,
                        6.0,
                        label(&shown.label, 11.0, CONSOLAS, text_main(), 13.0)
                    ),
                    at(width - 13.0, 8.0, chevron(text_main())),
                ],
                width,
                23,
                Some(bg_app()),
                border(),
                0.0,
            ),
            container(pick).width(width).height(23),
        ]
        .into();
    }
    let (fg, bg, border) = if disabled {
        (
            over(muted(), 0.7, bg_ribbon()),
            over(bg_app(), 0.7, bg_ribbon()),
            over(border(), 0.7, bg_ribbon()),
        )
    } else {
        (text_main(), bg_app(), border())
    };
    boxed(
        stack![
            at(
                12.0,
                6.0,
                label("No defined areas 0.0", 11.0, CONSOLAS, fg, 13.0)
            ),
            at(width - 13.0, 8.0, chevron(fg)),
        ],
        width,
        23,
        Some(bg),
        border,
        0.0,
    )
    .into()
}

/// `.status-hud`: a flex pill (gap 8, padding 2px 8px, 1px border) of
/// `#hud-len` @ `#hud-ang` `#hud-state`. Its width follows the text
/// (measured rects: Consolas 12px advances 6.5977 px/char, "@" 10.52, "UP"
/// 14.17, "DOWN" 35.28 — 119.47 x 21 at "0.0' @ 0.0° UP"), so the status
/// centre re-lays out as the readout changes.
fn hud_pill<'a>(hud: &crate::drawing::Hud) -> Element<'a, Message> {
    let (w_len, w_ang, w_state) = hud_widths(hud);
    // Chrome layout x of each child inside the pill.
    let x_len = 9.0;
    let x_at = x_len + w_len + 8.0;
    let x_ang = x_at + HUD_AT_W + 8.0;
    let x_state = x_ang + w_ang + 8.0;
    let _ = w_state;
    let (state, color) = if hud.down {
        ("DOWN", HUD_DOWN)
    } else {
        ("UP", muted())
    };
    // Ink lands one px left of the layout x for the Consolas / semibold
    // runs and at the rounded x for "@" (measured at the default pill).
    boxed(
        stack![
            at(
                x_len.round() - 1.0,
                5.0,
                label(&hud.len, 12.0, CONSOLAS, text_main(), 14.0)
            ),
            at(
                x_at.round(),
                3.0,
                label("@", 11.0, SEGOE, text_main(), 15.0)
            ),
            at(
                x_ang.round() - 1.0,
                5.0,
                label(&hud.ang, 12.0, CONSOLAS, text_main(), 14.0)
            ),
            at(
                x_state.round() - 1.0,
                3.0,
                label(state, 11.0, SEGOE_SEMIBOLD, color, 15.0)
            ),
        ],
        hud_width(hud),
        21,
        Some(bg_app()),
        border(),
        10.0,
    )
    .into()
}

const CONSOLAS12_ADVANCE: f32 = 6.597_656;
const HUD_AT_W: f32 = 10.52;

fn hud_widths(hud: &crate::drawing::Hud) -> (f32, f32, f32) {
    let mono = |s: &str| s.chars().count() as f32 * CONSOLAS12_ADVANCE;
    let state = if hud.down { 35.28 } else { 14.17 };
    (mono(&hud.len), mono(&hud.ang), state)
}

/// The pill's border-box width.
pub fn hud_width(hud: &crate::drawing::Hud) -> f32 {
    let (l, a, st) = hud_widths(hud);
    9.0 + l + 8.0 + HUD_AT_W + 8.0 + a + 8.0 + st + 9.0
}

/// `.hud-state.down` colour.
const HUD_DOWN: Color = Color::from_rgb(
    0x5D as f32 / 255.0,
    0xAE as f32 / 255.0,
    0x84 as f32 / 255.0,
);

/// `.status-center` children's flex-basis widths (measured).
const SLIDER_W: f32 = 114.69;
const INFO_W_100: f32 = 77.1875;
/// Segoe UI 11px digit advance (the readout's percentage; live: "+ 91%"
/// is 71.25 wide against 77.1875 for "+ 100%").
const SEGOE11_DIGIT: f32 = 5.9375;
const INPUT_W: f32 = 40.0;
const SELECT_W: f32 = 300.0;
/// The select's inner (content-box) width: 300 - padding 7 + 26 - border 2.
/// CSS flex-shrink weighs items by INNER base size.
const SELECT_INNER: f32 = 265.0;
const CENTER_GAP: f32 = 12.0;
/// The widest the centre block can be at 1400px with one page and no
/// scrollbar: the bar's 1384px content box minus status-left (364.66) and
/// status-right (297.78), live rects.
const CENTER_MAX_1400: f32 = 721.56;
/// The bar's vertical scrollbar width (`::-webkit-scrollbar`).
pub const SCROLLBAR_W: f32 = 10.0;
/// Underlay select width with one page.
const UNDERLAY_W_1: f32 = 107.0;

/// Widths of the readout and the areas select after CSS flex-shrink: when
/// the centre's content overflows its maximum, the slider, scale input and
/// HUD hold their min-content widths and the overflow is shared between
/// the readout (which then wraps) and the select, weighted by inner size.
pub fn center_shrink(info_natural: f32, hud_w: f32, max: f32) -> (f32, f32) {
    let natural = SLIDER_W + info_natural + INPUT_W + SELECT_W + hud_w + 4.0 * CENTER_GAP;
    let overflow = natural - max;
    if overflow <= 0.0 {
        return (info_natural, SELECT_W);
    }
    let total = info_natural + SELECT_INNER;
    (
        info_natural - overflow * info_natural / total,
        SELECT_W - overflow * SELECT_INNER / total,
    )
}

fn center<'a>(
    v: &View,
    scale_text: String,
    areas_disabled: bool,
    areas: (&[AreaOption], Option<i64>),
    hud: &crate::drawing::Hud,
    max: f32,
) -> Element<'a, Message> {
    let pct = v.zoom_percent();
    let readout = format!("+ {pct}% (Imp) 1:");
    let info_natural = info_natural(v);
    let (info_w, select_w) = center_shrink(info_natural, hud_width(hud), max);
    let info: Element<'a, Message> = if info_w < info_natural - 0.01 {
        // Shrunk below its one-line width: the text wraps (30px tall,
        // overflowing the bar like the original).
        // Ink measured 1px low against the original: nudge up, draw-only.
        crate::snap::nudge_y(
            -1.0,
            container(
                text_line(readout.clone(), 11.0, SEGOE, muted(), 15.0)
                    .wrapping(iced::widget::text::Wrapping::Word)
                    .width(info_w),
            )
            .width(info_w)
            .height(30),
        )
        .into()
    } else {
        snap_floor(label(&readout, 11.0, SEGOE, muted(), 15.0)).into()
    };
    row![
        snap(zoom_slider(v)),
        info,
        snap(scale_input(scale_text)),
        // `.area-status-select` 300x23 (Consolas 11px); options with areas
        // arrive with screen 7.
        snap(crate::tooltips::tip(
            areas_select(areas_disabled, select_w, areas.0, areas.1),
            "Defined areas",
        )),
        snap(hud_pill(hud)),
    ]
    .spacing(12)
    .height(27)
    .align_y(alignment::Vertical::Center)
    .into()
}

/// Arial advance widths (units of 2048) for the underlay select's text.
fn arial_advance(c: char) -> f32 {
    match c {
        'U' | 'N' => 1479.0,
        'P' => 1366.0,
        'r' => 682.0,
        'l' => 455.0,
        'y' => 1024.0,
        ':' | ' ' => 569.0,
        _ => 1139.0, // n d e a o g and the digits
    }
}

/// Width of `s` in Arial 11px.
pub fn arial11_width(s: &str) -> f32 {
    s.chars().map(arial_advance).sum::<f32>() * 11.0 / 2048.0
}

/// `.underlay-select` width: Chrome sizes a select to its widest option
/// (measured: 107 for "Underlay: None", 115 once "Underlay: Page N" exists).
pub fn underlay_width(options: &[(Option<usize>, String)]) -> f32 {
    let widest = options
        .iter()
        .map(|(_, l)| arial11_width(l))
        .fold(0.0, f32::max);
    (30.6 + widest).floor()
}

/// A `#area-status-select` option: `code  name  area`, padded into columns.
#[derive(Debug, Clone, PartialEq)]
pub struct AreaOption {
    pub id: i64,
    pub label: String,
}

impl std::fmt::Display for AreaOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

/// `updateCalcsPanel`'s options for the defined areas: the code and the name
/// padded to the widest (min 3 and 4), then the gross area to one decimal.
/// `rows` is `(id, code, name, area)`.
pub fn area_options(rows: &[(i64, String, String, f64)]) -> Vec<AreaOption> {
    let code_w = rows
        .iter()
        .map(|r| r.1.chars().count())
        .chain([3])
        .max()
        .unwrap_or(3);
    let name_w = rows
        .iter()
        .map(|r| r.2.chars().count())
        .chain([4])
        .max()
        .unwrap_or(4);
    rows.iter()
        .map(|(id, code, name, area)| AreaOption {
            id: *id,
            label: format!("{code:<code_w$}  {name:<name_w$}  {area:.1}"),
        })
        .collect()
}

/// The chip's text: `GLA 123.4 sf` and `3 areas` (`1 area`).
pub fn chip_texts(totals: crate::calcs::AreaTotals, count: usize) -> (String, String) {
    (
        format!("GLA {:.1} sf", totals.total_gla),
        format!("{count} area{}", if count == 1 { "" } else { "s" }),
    )
}

/// An `#underlay-select` option.
#[derive(Debug, Clone, PartialEq)]
pub struct UnderlayOption {
    pub index: Option<usize>,
    pub label: String,
}

impl std::fmt::Display for UnderlayOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

/// Status bar inputs from the page model.
pub struct PageView<'p> {
    pub pages: &'p Pages,
    /// `#area-status-select.disabled` (set by updateCalcsPanel once a page
    /// has been (re)loaded with no areas).
    pub areas_disabled: bool,
    /// The totals chip: GLA and the defined-area count (`updateCalcsPanel`).
    pub totals: crate::calcs::AreaTotals,
    pub area_count: usize,
    /// The defined-areas select's options and the one it shows.
    pub areas: &'p [AreaOption],
    pub area_selected: Option<i64>,
    /// The drawing HUD (`#hud-len` / `#hud-ang` / `#hud-state`).
    pub hud: &'p crate::drawing::Hud,
    /// The window's logical width (the bar's width).
    pub width: f32,
    /// The bar's vertical scrollbar is showing.
    pub scroll: bool,
}

/// A child placed `top` px below the content row's top (the original's
/// measured y), its draw x snapped to the pixel grid.
fn cell<'a>(top: f32, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    snap(iced::widget::column![
        Space::new().height(top),
        content.into()
    ])
    .into()
}

fn right<'a>(p: &PageView<'_>) -> Element<'a, Message> {
    let pages = p.pages;
    // North: arrow (13) + gap 4 + "N°" (Consolas 10px, 5.498px/char),
    // centred as a group on the 22px button (overflowing it).
    let angle = pages.page().north_angle;
    let degrees = format!("{}°", angle.round() as i64);
    let label_w = degrees.chars().count() as f32 * 5.498;
    let ax = 11.0 - (17.0 + label_w) / 2.0;
    let north = crate::tooltips::tip(
        button(
            stack![
                at(
                    ax,
                    4.0,
                    rotated("arrow-up", true, 13.0, ICON_BLUE, angle as f32)
                ),
                at(
                    ax + 17.0,
                    6.0,
                    label(&degrees, 10.0, CONSOLAS, muted(), 12.0)
                ),
            ]
            .width(22)
            .height(22),
        )
        .padding(0)
        .width(22)
        .height(22)
        .on_press(msg(StatusMsg::North))
        .style(|_, status| button::Style {
            background: match status {
                button::Status::Hovered | button::Status::Pressed => Some(bg_hover().into()),
                _ => None,
            },
            ..button::Style::default()
        }),
        "North direction (click to set)",
    );
    // `.page-indicator`: gap 4, padding-right 8, 1px border-right. Previous
    // is `disabled` on page 1 but keeps its color (`.page-nav` sets it).
    let prev: Element<'a, Message> = if pages.can_go_prev() {
        bare_btn(
            ph("caret-left", 14.0, text_main()),
            msg(StatusMsg::PagePrev),
        )
    } else {
        button(ph("caret-left", 14.0, text_main()))
            .padding(0)
            .style(|_, _| button::Style::default())
            .into()
    };
    let prev = crate::tooltips::tip(prev, "Previous Page");
    let items = row![
        cell(
            0.0,
            crate::tooltips::tip(
                bare_btn(
                    ph("file-minus", 14.0, text_main()),
                    msg(StatusMsg::PageDelete)
                ),
                "Delete Page"
            )
        ),
        cell(0.0, prev),
        cell(0.0, label(&pages.display(), 11.0, SEGOE, text_main(), 15.0)),
        cell(
            0.0,
            crate::tooltips::tip(
                bare_btn(
                    ph("caret-right", 14.0, text_main()),
                    msg(StatusMsg::PageNext)
                ),
                "Next Page"
            )
        ),
        cell(
            0.0,
            crate::tooltips::tip(
                bare_btn(ph("file-plus", 14.0, text_main()), msg(StatusMsg::PageAdd)),
                "Add Page"
            )
        ),
    ]
    .spacing(4);
    // padding-right 8, then the 1px border (iced gives a zero-width child
    // no spacing, so the padding is real padding).
    let indicator = row![
        container(items).padding(Padding {
            right: 8.0,
            ..Padding::ZERO
        }),
        cell(0.0, rect(1.0, 16.0, border())),
    ];
    // `.underlay-select`: transparent, muted, radius 4; drawn, with an
    // invisible pick_list on top.
    let options = pages.underlay_options();
    let width = underlay_width(&options);
    let choices: Vec<UnderlayOption> = options
        .into_iter()
        .map(|(index, label)| UnderlayOption { index, label })
        .collect();
    let shown = pages.underlay_label();
    let selected = choices.iter().find(|o| o.label == shown).cloned();
    let pick = pick_list(choices, selected, |o| msg(StatusMsg::Underlay(o.index)))
        .width(width)
        .padding(0)
        .text_size(11)
        .handle(pick_list::Handle::None)
        .menu_style(menu_style)
        .style(invisible_pick);
    let underlay = stack![
        boxed(
            stack![
                at(8.0, 4.0, label(&shown, 11.0, ARIAL, muted(), 13.0)),
                at(width - 13.0, 7.0, chevron(muted())),
            ],
            width,
            20,
            None,
            border(),
            4.0,
        ),
        container(pick).width(width).height(20),
    ];
    row![
        cell(3.0, north),
        cell(6.0, indicator),
        cell(
            4.0,
            crate::tooltips::tip(underlay, "Ghost another page beneath this one"),
        ),
        cell(
            3.0,
            status_btn(
                "bluetooth",
                text_main(),
                Message::Disto(crate::disto::DistoMsg::Connect),
                "Bluetooth",
            ),
        ),
        cell(
            3.0,
            status_btn(
                "hand-pointing",
                text_main(),
                Message::TouchPanelToggle,
                "Touch Panel"
            ),
        ),
        // The bar's 8px right padding = the row gap + 4.
        Space::new().width(4),
    ]
    .spacing(4)
    .height(27)
    .into()
}

fn menu_style(_: &iced::Theme) -> iced::overlay::menu::Style {
    iced::overlay::menu::Style {
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
    }
}

fn invisible_pick(_: &iced::Theme, _: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: Color::TRANSPARENT,
        placeholder_color: Color::TRANSPARENT,
        handle_color: Color::TRANSPARENT,
        background: Background::Color(Color::TRANSPARENT),
        border: clear_border(),
    }
}

/// The readout's one-line width ("+ 100% (Imp) 1:" = 77.19).
fn info_natural(v: &View) -> f32 {
    let digits = v.zoom_percent().to_string().len() as f32;
    INFO_W_100 + (digits - 3.0) * SEGOE11_DIGIT
}

/// The centre block's content width before any shrinking.
pub fn center_natural(v: &View, hud: &crate::drawing::Hud) -> f32 {
    SLIDER_W + info_natural(v) + INPUT_W + SELECT_W + hud_width(hud) + 4.0 * CENTER_GAP
}

/// The centre block's maximum width without the scrollbar: the bar minus
/// the left and right blocks (the right block grows with the underlay
/// select).
pub fn center_max(pages: &Pages, width: f32) -> f32 {
    let underlay = underlay_width(&pages.underlay_options());
    CENTER_MAX_1400 + (width - 1400.0) - (underlay - UNDERLAY_W_1)
}

/// The native vertical scrollbar as Chrome paints it for this bar (content
/// 30px in a 27px viewport): arrows, track and thumb, from the reference.
static VSCROLL: std::sync::LazyLock<iced::widget::image::Handle> = std::sync::LazyLock::new(|| {
    iced::widget::image::Handle::from_bytes(
        &include_bytes!("../../../assets/ui/statusbar-vscroll.png")[..],
    )
});

/// `scale_text`: the 1:N input's text (the edit buffer while typing).
pub fn view<'a>(v: &View, scale_text: String, p: &PageView<'_>) -> Element<'a, Message> {
    // `justify-content: space-between`: the centre sits one free-space gap
    // right of status-left. Pinned from the measured block widths (status-
    // right is 297.78 + the bar's 8px padding with one page) so iced's
    // rounding of the right block cannot move it.
    let max = center_max(p.pages, p.width) - if p.scroll { SCROLLBAR_W } else { 0.0 };
    let width = center_natural(v, p.hud).min(max);
    let gap = ((max - width) / 2.0).max(0.0);
    let bar = row![
        left(v, p.totals, p.area_count),
        Space::new().width(gap),
        center(
            v,
            scale_text,
            p.areas_disabled,
            (p.areas, p.area_selected),
            p.hud,
            max
        ),
        Space::new().width(Length::Fill),
        // Chrome lays status-right out at the content box's right edge; iced
        // rounds it a pixel early, which shows in the drawing states.
        if p.scroll {
            right(p)
        } else {
            snap(right(p)).into()
        },
    ]
    .height(27);
    let bar = if p.scroll {
        bar.push(
            iced::widget::image(VSCROLL.clone())
                .width(SCROLLBAR_W)
                .height(27)
                .filter_method(iced::widget::image::FilterMethod::Nearest),
        )
    } else {
        bar
    };
    fill_bg(
        iced::widget::column![
            fill_bg(Space::new().width(Length::Fill).height(1), border()).width(Length::Fill),
            bar,
        ],
        bg_ribbon(),
    )
    .width(Length::Fill)
    .height(28)
    .into()
}

#[cfg(test)]
mod center_tests {
    use super::*;
    use crate::drawing::Hud;

    #[test]
    fn hud_and_center_widths_match_the_live_rects() {
        // Live original: default pill 119.47; drawing "8.0' @ 285° DOWN"
        // pill 140.58, the readout shrinks 77.19 -> 75.19 and the select
        // 300 -> 293.11 (the centre capped at 711.56).
        let up = Hud::default();
        assert!((hud_width(&up) - 119.47).abs() < 0.02);
        let down = Hud {
            len: "8.0'".into(),
            ang: "285°".into(),
            down: true,
        };
        let w = hud_width(&down);
        assert!((w - 140.58).abs() < 0.02, "{w}");
        assert_eq!(
            center_shrink(INFO_W_100, hud_width(&up), CENTER_MAX_1400),
            (INFO_W_100, 300.0)
        );
        let with_scrollbar = CENTER_MAX_1400 - SCROLLBAR_W;
        let (info, select) = center_shrink(INFO_W_100, w, with_scrollbar);
        assert!((info - 75.19).abs() < 0.02, "{info}");
        assert!((select - 293.11).abs() < 0.02, "{select}");
    }
}
