//! On-screen drawing keypad (`Pad`) for touch devices and stylus-free input
//! (`docs/ui-redesign/01-design-spec.md` §5.11, `07-touch-and-android.md` §3).

use iced::widget::{button, column, container, mouse_area, opaque, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Point};

use crate::disto::DistoMsg;
use crate::icons;
use crate::input::KeyCmd;
use crate::ui::fonts::{BARLOW_MEDIUM, BARLOW_SEMIBOLD, BSC_MEDIUM, BSC_SEMIBOLD};
use crate::ui::kit;
use crate::ui::layout::{COMPACT_SHEET_H, SizeClass};
use crate::ui::tokens::{
    self, RADIUS_CONTROL, RADIUS_DIALOG, RADIUS_TRAY, SPACE_1, SPACE_2, TEXT_11, TEXT_12, TEXT_13,
};
use crate::ui::{Msg as UiMsg, PadPopover};
use crate::view_state::Mode;
use crate::{App, Message};

const KEY_W: f32 = 54.0;
const KEY_H: f32 = 52.0;
const KEY_GAP: f32 = 6.0;
const CARD_PAD: f32 = 10.0;

pub const FLOAT_W: f32 = KEY_W * 4.0 + KEY_GAP * 3.0 + CARD_PAD * 2.0;
pub const FLOAT_H: f32 = 434.0;

/// True when the active tool benefits from automatic Pad visibility in Touch density.
pub fn is_drawing_tool(mode: Mode) -> bool {
    matches!(mode, Mode::DrawExterior | Mode::DrawInterior | Mode::Measure)
}

/// Top-left anchor for the floating Pad on Wide & Medium viewports.
pub fn floating_origin(app: &App) -> Point {
    let win = app.window_size;
    let h = FLOAT_H + if app.ui.pad_popover.is_some() { 48.0 } else { 0.0 };
    let min_x = 12.0;
    let min_y = app.ui.density.bar_h() + 12.0;
    let max_x = (win.width - FLOAT_W - 12.0).max(min_x);
    let max_y = (win.height - h - 12.0).max(min_y);
    // Keep clear of both the bottom tray and the Touch [Pad, Laser] pill row above it.
    let default = Point::new(
        (win.width - FLOAT_W - 16.0).clamp(min_x, max_x),
        (win.height - h - 132.0).clamp(min_y, max_y),
    );
    match app.ui.pad_pos {
        Some(p) => Point::new(p.x.clamp(min_x, max_x), p.y.clamp(min_y, max_y)),
        None => default,
    }
}

/// Floating Pad layer for Wide and Medium viewports (omitted on Compact).
pub fn layer<'a>(app: &'a App) -> Option<Element<'a, Message>> {
    if !app.touch_panel_open {
        return None;
    }
    if matches!(SizeClass::of(app.window_size.width), SizeClass::Compact) {
        return None;
    }

    let origin = floating_origin(app);
    let card = container(pad_body(app, false))
        .width(Length::Fixed(FLOAT_W))
        .padding(CARD_PAD)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::bar())),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: RADIUS_TRAY.into(),
            },
            shadow: tokens::floating_shadow(),
            ..container::Style::default()
        });

    let mut children: Vec<Element<'a, Message>> = Vec::new();
    if app.ui.pad_popover.is_some() {
        children.push(
            mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
                .on_press(Message::Ui(UiMsg::PadPopover(None)))
                .into(),
        );
    }

    let positioned = container(opaque(card))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: origin.y.max(0.0),
            left: origin.x.max(0.0),
            right: 0.0,
            bottom: 0.0,
        });
    children.push(positioned.into());

    Some(iced::widget::Stack::with_children(children).width(Length::Fill).height(Length::Fill).into())
}

/// Docked bottom Pad replacing the Compact bottom sheet above the Compact toolbar.
pub fn compact_sheet<'a>(app: &'a App) -> Element<'a, Message> {
    container(pad_body(app, true))
        .width(Length::Fill)
        .height(Length::Fixed(COMPACT_SHEET_H))
        .padding([6.0, 10.0])
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::bar())),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: iced::border::Radius {
                    top_left: 12.0,
                    top_right: 12.0,
                    bottom_left: 0.0,
                    bottom_right: 0.0,
                },
            },
            shadow: tokens::floating_shadow(),
            ..container::Style::default()
        })
        .into()
}

fn pad_body<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    let key_h = if compact { 42.0 } else { KEY_H };
    let gap = if compact { 4.0 } else { KEY_GAP };
    let col_gap = if compact { 4.0 } else { 8.0 };
    let pen_down = app.drawing.pen_down;

    let mut col = column![header_row(app, compact), readout_bar(app, compact)].spacing(col_gap);

    if let Some(pop) = app.ui.pad_popover {
        col = col.push(popover_strip(app, pop));
    }

    // Row 1: Turn L | Up | Turn R | Backspace
    let r1 = row![
        text_icon_key(
            "arrow-u-up-left",
            "Turn L",
            Message::Ui(UiMsg::PadTurn(true)),
            None,
            compact,
            key_h,
        ),
        arrow_key(0, "arrow-up", "Up", compact, key_h),
        text_icon_key(
            "arrow-u-up-right",
            "Turn R",
            Message::Ui(UiMsg::PadTurn(false)),
            None,
            compact,
            key_h,
        ),
        icon_only_key(
            "backspace",
            Message::Key(KeyCmd::BufferBackspace),
            compact,
            key_h,
            false,
        ),
    ]
    .spacing(gap);

    // Row 2: Left | Down | Right | Laser
    let laser_connected = app.disto.connected.is_some();
    let laser_msg = if laser_connected {
        Message::Ui(UiMsg::PadLaser)
    } else {
        Message::Disto(DistoMsg::Connect)
    };
    let r2 = row![
        arrow_key(6, "arrow-left", "Left", compact, key_h),
        arrow_key(4, "arrow-down", "Down", compact, key_h),
        arrow_key(2, "arrow-right", "Right", compact, key_h),
        text_icon_key(
            "bluetooth",
            "Laser",
            laser_msg,
            laser_connected.then_some(tokens::accent()),
            compact,
            key_h,
        ),
    ]
    .spacing(gap);

    // Row 3: 7 | 8 | 9 | '
    let r3 = row![
        digit_key("7", Message::Key(KeyCmd::Buffer("7")), compact, key_h),
        digit_key("8", Message::Key(KeyCmd::Buffer("8")), compact, key_h),
        digit_key("9", Message::Key(KeyCmd::Buffer("9")), compact, key_h),
        digit_key("'", Message::Key(KeyCmd::Buffer("'")), compact, key_h),
    ]
    .spacing(gap);

    // Row 4: 4 | 5 | 6 | .
    let r4 = row![
        digit_key("4", Message::Key(KeyCmd::Buffer("4")), compact, key_h),
        digit_key("5", Message::Key(KeyCmd::Buffer("5")), compact, key_h),
        digit_key("6", Message::Key(KeyCmd::Buffer("6")), compact, key_h),
        digit_key(".", Message::Key(KeyCmd::DecimalPoint), compact, key_h),
    ]
    .spacing(gap);

    // Row 5: 1 | 2 | 3 | 0
    let r5 = row![
        digit_key("1", Message::Key(KeyCmd::Buffer("1")), compact, key_h),
        digit_key("2", Message::Key(KeyCmd::Buffer("2")), compact, key_h),
        digit_key("3", Message::Key(KeyCmd::Buffer("3")), compact, key_h),
        digit_key("0", Message::Key(KeyCmd::Buffer("0")), compact, key_h),
    ]
    .spacing(gap);

    // Bottom full-width Close area button (long-press / right-click opens More).
    let close_msg = if app.view.mode == Mode::Measure {
        Message::Ui(UiMsg::MeasureClose)
    } else {
        Message::Key(KeyCmd::AutoClose)
    };
    let close_inner = row![
        icons::mono("check", 15.0, tokens::on_accent()),
        text("Close area")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_13)
            .color(tokens::on_accent()),
        Space::new().width(Length::Fill),
        icons::mono("dots-three", 14.0, tokens::on_accent()),
    ]
    .spacing(SPACE_1)
    .padding([0, 10])
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let close_btn = button(
        container(close_inner)
            .width(Length::Fill)
            .height(Length::Fixed(if compact { 40.0 } else { 44.0 }))
            .align_y(Alignment::Center),
    )
    .padding(0)
    .width(Length::Fill)
    .style(|_, status| {
        let bg = match status {
            button::Status::Pressed => tokens::pressed(),
            button::Status::Hovered => crate::theme::over(Color::WHITE, 0.10, tokens::accent()),
            _ => tokens::accent(),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: tokens::on_accent(),
            border: Border {
                radius: RADIUS_CONTROL.into(),
                ..Border::default()
            },
            ..button::Style::default()
        }
    })
    .on_press(close_msg);

    let more_msg = Message::Ui(UiMsg::PadPopover(Some(if app.ui.pad_popover == Some(PadPopover::More) {
        // Re-holding toggles cleanly
        PadPopover::More
    } else {
        PadPopover::More
    })));
    let close_area_key = kit::context_press(close_btn, more_msg);

    let grid = column![r1, r2, r3, r4, r5].spacing(gap);
    col = col.push(grid).push(close_area_key);

    if !pen_down && app.view.mode == Mode::Measure && compact {
        // Keep vertical budget exact in compact; no extra rows needed.
    }

    col.width(Length::Fill).into()
}

fn header_row<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    let pen_down = app.drawing.pen_down;
    let pen_label = if pen_down { "Pen: Down" } else { "Pen: Up" };
    let pen_fg = if pen_down {
        tokens::on_accent()
    } else {
        tokens::text()
    };
    let pen_bg = if pen_down {
        tokens::accent()
    } else {
        tokens::hover()
    };
    let pen_pill = button(
        row![
            icons::mono("pencil-simple", 12.0, pen_fg),
            text(pen_label)
                .font(BARLOW_SEMIBOLD)
                .size(TEXT_11)
                .color(pen_fg),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .padding([4, 8])
    .style(move |_, status| button::Style {
        background: Some(Background::Color(match status {
            button::Status::Pressed => tokens::pressed(),
            _ => pen_bg,
        })),
        text_color: pen_fg,
        border: Border {
            color: tokens::rule(),
            width: if pen_down { 0.0 } else { 1.0 },
            radius: 999.0.into(),
        },
        ..button::Style::default()
    })
    .on_press(Message::Ui(UiMsg::PadPen));

    let title_inner = row![
        icons::mono("dots-six-vertical", 14.0, tokens::muted()),
        text("Pad")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_12)
            .color(tokens::text()),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let title_drag: Element<'a, Message> = if compact {
        title_inner.into()
    } else {
        mouse_area(container(title_inner).padding([2, 4]))
            .on_press(Message::Ui(UiMsg::PadDragStart))
            .interaction(iced::mouse::Interaction::Grab)
            .into()
    };

    let close_btn = button(icons::mono("x", 13.0, tokens::muted()))
        .padding([4, 6])
        .style(|_, status| button::Style {
            background: Some(Background::Color(match status {
                button::Status::Hovered | button::Status::Pressed => tokens::hover(),
                _ => Color::TRANSPARENT,
            })),
            text_color: tokens::muted(),
            border: Border {
                radius: RADIUS_CONTROL.into(),
                ..Border::default()
            },
            ..button::Style::default()
        })
        .on_press(Message::TouchPanelToggle);

    row![
        title_drag,
        Space::new().width(Length::Fill),
        pen_pill,
        close_btn,
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn readout_bar<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    let tape_bg = tokens::tape();
    let ink = tokens::tape_ink();
    let hud = &app.drawing.hud;
    let state_txt = if hud.down { "DOWN" } else { "UP" };
    let h = if compact { 30.0 } else { 34.0 };

    let mut r = row![
        text(app.drawing.length_readout())
            .font(BSC_SEMIBOLD)
            .size(if compact { 14.0 } else { 15.0 })
            .color(ink),
        text(format!("@ {}", hud.ang))
            .font(BSC_MEDIUM)
            .size(TEXT_12)
            .color(ink),
        Space::new().width(Length::Fill),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center);

    // When the buffer carries an `<` angle entry, offer a direct commit tap.
    if app.drawing.buffer.contains('<') {
        let ok = button(
            text("OK")
                .font(BARLOW_SEMIBOLD)
                .size(TEXT_11)
                .color(tokens::on_accent()),
        )
        .padding([2, 8])
        .style(|_, _| button::Style {
            background: Some(Background::Color(tokens::accent())),
            border: Border {
                radius: RADIUS_CONTROL.into(),
                ..Border::default()
            },
            ..button::Style::default()
        })
        .on_press(Message::Key(KeyCmd::Enter));
        r = r.push(ok);
    }

    r = r.push(
        text(state_txt)
            .font(BSC_SEMIBOLD)
            .size(TEXT_11)
            .color(ink),
    );

    container(r)
        .width(Length::Fill)
        .height(Length::Fixed(h))
        .padding([0, 10])
        .align_y(Alignment::Center)
        .style(move |_| container::Style {
            background: Some(Background::Color(tape_bg)),
            border: Border {
                radius: RADIUS_CONTROL.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

fn popover_strip<'a>(_app: &'a App, pop: PadPopover) -> Element<'a, Message> {
    let content: Element<'a, Message> = match pop {
        PadPopover::Diagonals => row![
            diag_btn("NW", 7),
            diag_btn("NE", 1),
            diag_btn("SW", 5),
            diag_btn("SE", 3),
        ]
        .spacing(SPACE_1)
        .width(Length::Fill)
        .into(),
        PadPopover::More => column![
            row![
                more_btn("Bay mirror", Message::Key(KeyCmd::BayMirror)),
                more_btn("Jump endpoint", Message::Key(KeyCmd::JumpEndpoint)),
            ]
            .spacing(SPACE_1),
            row![
                more_btn("Rise × run (x)", Message::Key(KeyCmd::Buffer("x"))),
                more_btn("Angle (<)", Message::Key(KeyCmd::Buffer("<"))),
                more_btn("Commit", Message::Key(KeyCmd::Enter)),
            ]
            .spacing(SPACE_1),
        ]
        .spacing(SPACE_1)
        .width(Length::Fill)
        .into(),
    };

    container(content)
        .padding(SPACE_2)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::panel())),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: RADIUS_DIALOG.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn diag_btn<'a>(label: &'static str, dir: u8) -> Element<'a, Message> {
    button(
        container(
            text(label)
                .font(BSC_SEMIBOLD)
                .size(TEXT_12)
                .color(tokens::text()),
        )
        .width(Length::Fill)
        .height(Length::Fixed(36.0))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center),
    )
    .padding(0)
    .width(Length::FillPortion(1))
    .style(pad_key_style(false))
    .on_press(Message::Ui(UiMsg::PadArrow(dir)))
    .into()
}

fn more_btn<'a>(label: &'static str, msg: Message) -> Element<'a, Message> {
    button(
        container(
            text(label)
                .font(BARLOW_MEDIUM)
                .size(TEXT_11)
                .color(tokens::text()),
        )
        .width(Length::Fill)
        .height(Length::Fixed(32.0))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center),
    )
    .padding([0, 4])
    .width(Length::FillPortion(1))
    .style(pad_key_style(false))
    .on_press(msg)
    .into()
}

fn key_width(compact: bool) -> Length {
    if compact {
        Length::FillPortion(1)
    } else {
        Length::Fixed(KEY_W)
    }
}

fn digit_key<'a>(
    label: &'static str,
    msg: Message,
    compact: bool,
    height: f32,
) -> Element<'a, Message> {
    button(
        container(
            text(label)
                .font(BARLOW_SEMIBOLD)
                .size(18.0)
                .color(tokens::text()),
        )
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center),
    )
    .padding(0)
    .width(key_width(compact))
    .style(pad_key_style(false))
    .on_press(msg)
    .into()
}

fn text_icon_key<'a>(
    icon: &'static str,
    label: &'static str,
    msg: Message,
    highlight: Option<Color>,
    compact: bool,
    height: f32,
) -> Element<'a, Message> {
    let fg = highlight.unwrap_or_else(tokens::text);
    let inner = column![
        icons::mono(icon, 15.0, fg),
        text(label).font(BSC_MEDIUM).size(TEXT_11).color(fg),
    ]
    .spacing(2)
    .align_x(Alignment::Center);

    button(
        container(inner)
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .width(key_width(compact))
    .style(pad_key_style(highlight.is_some()))
    .on_press(msg)
    .into()
}

fn icon_only_key<'a>(
    icon: &'static str,
    msg: Message,
    compact: bool,
    height: f32,
    active: bool,
) -> Element<'a, Message> {
    let fg = if active {
        tokens::on_accent()
    } else {
        tokens::text()
    };
    button(
        container(icons::mono(icon, 18.0, fg))
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .width(key_width(compact))
    .style(pad_key_style(active))
    .on_press(msg)
    .into()
}

fn arrow_key<'a>(
    dir: u8,
    icon: &'static str,
    _label: &'static str,
    compact: bool,
    height: f32,
) -> Element<'a, Message> {
    let btn = button(
        container(icons::mono(icon, 19.0, tokens::text()))
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .width(key_width(compact))
    .style(pad_key_style(false))
    .on_press(Message::Ui(UiMsg::PadArrow(dir)));

    container(kit::context_press(
        btn,
        Message::Ui(UiMsg::PadPopover(Some(PadPopover::Diagonals))),
    ))
    .width(key_width(compact))
    .into()
}

fn pad_key_style(highlighted: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let base = if highlighted {
            tokens::accent_soft()
        } else {
            tokens::panel()
        };
        let bg = match status {
            button::Status::Pressed => tokens::pressed(),
            button::Status::Hovered => tokens::hover(),
            _ => base,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: tokens::text(),
            border: Border {
                color: if highlighted {
                    tokens::accent()
                } else {
                    tokens::rule()
                },
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..button::Style::default()
        }
    }
}
