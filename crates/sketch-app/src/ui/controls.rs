//! Floating drawing controls (zoom, snap/grid/pitch/angle/show/dimensions,
//! north, underlay, mini-map, and Touch Pad/Laser buttons) for `Shell::Next`
//! (`01-design-spec.md` §5.7, P9).
//!
//! In Wide + Comfortable, zoom sits bottom-left and snapping/display sits
//! bottom-right flanking the tray; North sits top-right (Underlay top-left).
//! In Medium, Compact, or Touch density, zoom moves to top-left and
//! snapping/display + North move to top-right.

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    button, checkbox, column, container, image, mouse_area, opaque, row, scrollable, slider,
    text, text_input, Column, Space,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding};

use crate::disto::DistoMsg;
use crate::icons;
use crate::statusbar::{PitchOption, StatusMsg};
use crate::ui::fonts::{BARLOW, BARLOW_MEDIUM, BARLOW_SEMIBOLD, BSC_SEMIBOLD};
use crate::ui::kit;
use crate::ui::layout::{self, SizeClass};
use crate::ui::tokens::{
    self, Density, FOCUS_RING_WIDTH, RADIUS_CONTROL, RADIUS_DIALOG, SPACE_1, SPACE_2, SPACE_3,
    TEXT_11, TEXT_12, TEXT_13,
};
use crate::ui::{ControlsPopover, Msg as UiMsg};
use crate::view_state::{DimensionPlacement, FILTERS, PITCHES};
use crate::{App, Message};

const ANGLE_STEPS: &[f64] = &[0.0, 5.0, 15.0, 45.0, 90.0];

fn corners_at_top(app: &App) -> bool {
    let sc = SizeClass::of(app.window_size.width);
    let d = app.ui.density;
    !matches!(sc, SizeClass::Wide) || matches!(d, Density::Touch) || layout::canvas_rect(app).width < 1240.0
}

/// Corner bars and mini-map rendered inside `layout::drawing_stack` (under the tray).
pub fn bars_layer<'a>(app: &'a App) -> Element<'a, Message> {
    let sc = SizeClass::of(app.window_size.width);
    let d = app.ui.density;
    let corners_at_top = corners_at_top(app);
    let show_underlay = app.pages.len() > 1 || app.pages.underlay_label() != "Underlay: None";

    let mk_zoom = || pill_surface(zoom_group(app, d, corners_at_top));
    let mk_snap = || pill_surface(snap_group(app, d));
    let north = pill_surface(north_chip(app, d));
    let underlay = show_underlay.then(|| pill_surface(underlay_chip(app, d)));

    let top_area: Element<'a, Message> = if matches!(sc, SizeClass::Compact) {
        let r1 = row![mk_zoom(), Space::new().width(Length::Fill), north].align_y(Alignment::Center);
        let r2 = row![Space::new().width(Length::Fill), mk_snap()].align_y(Alignment::Center);
        let mut col = column![r1, r2].spacing(SPACE_1).width(Length::Fill);
        if let Some(u) = underlay {
            col = col.push(row![u].align_y(Alignment::Center));
        }
        col.into()
    } else if matches!(sc, SizeClass::Medium) {
        let mut left = row![mk_zoom()].spacing(SPACE_1).align_y(Alignment::Center);
        if let Some(u) = underlay {
            left = left.push(u);
        }
        let r1 = row![left, Space::new().width(Length::Fill), north]
            .spacing(SPACE_2)
            .align_y(Alignment::Center)
            .width(Length::Fill);
        let r2 = row![Space::new().width(Length::Fill), mk_snap()]
            .align_y(Alignment::Center)
            .width(Length::Fill);
        column![r1, r2].spacing(SPACE_1).width(Length::Fill).into()
    } else if corners_at_top {
        let mut left = row![mk_zoom()].spacing(SPACE_1).align_y(Alignment::Center);
        if let Some(u) = underlay {
            left = left.push(u);
        }
        let right = row![mk_snap(), north]
            .spacing(SPACE_1)
            .align_y(Alignment::Center);
        row![left, Space::new().width(Length::Fill), right]
            .spacing(SPACE_2)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    } else {
        let mut r = row![].align_y(Alignment::Center).width(Length::Fill);
        if let Some(u) = underlay {
            r = r.push(u);
        }
        r = r.push(Space::new().width(Length::Fill));
        r = r.push(north);
        r.into()
    };

    let minimap_el: Option<Element<'a, Message>> = app.mini_map.as_ref().map(|handle| {
        row![Space::new().width(Length::Fill), minimap_card(handle, sc)]
            .width(Length::Fill)
            .into()
    });

    let touch_extras: Option<Element<'a, Message>> = matches!(d, Density::Touch).then(|| {
        let pad_btn = chip_toggle(
            "Pad",
            None,
            app.touch_panel_open,
            Message::TouchPanelToggle,
            d,
        );
        let laser_btn = chip_toggle(
            "Laser",
            Some("bluetooth"),
            app.disto.phase != crate::disto::Phase::Idle || app.disto.connected.is_some(),
            Message::Disto(DistoMsg::Connect),
            d,
        );
        pill_surface(row![pad_btn, laser_btn].spacing(SPACE_1).into())
    });

    let bottom_row: Element<'a, Message> = if !corners_at_top {
        let mut r = row![mk_zoom(), Space::new().width(Length::Fill), mk_snap()]
            .spacing(SPACE_2)
            .align_y(Alignment::Center)
            .width(Length::Fill);
        if let Some(extras) = touch_extras {
            r = r.push(extras);
        }
        r.into()
    } else if let Some(extras) = touch_extras {
        row![Space::new().width(Length::Fill), extras]
            .width(Length::Fill)
            .into()
    } else {
        Space::new().height(0).into()
    };

    let mut stack_col = column![top_area].spacing(SPACE_2).width(Length::Fill);
    let select_mode = matches!(
        app.view.mode,
        crate::view_state::Mode::Select
            | crate::view_state::Mode::Fence
            | crate::view_state::Mode::Lasso
    );
    if select_mode {
        let select_more_pill = pill_surface(chip_toggle(
            "Select more",
            Some("selection-all"),
            app.ui.select_more,
            Message::Ui(UiMsg::ToggleSelectMore),
            d,
        ));
        if let Some(mm) = minimap_el {
            stack_col = stack_col.push(
                row![
                    Space::new().width(Length::Fill),
                    select_more_pill,
                    Space::new().width(Length::Fill),
                    mm,
                ]
                .align_y(Alignment::Start)
                .width(Length::Fill),
            );
        } else {
            stack_col = stack_col.push(
                container(select_more_pill)
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            );
        }
    } else if let Some(mm) = minimap_el {
        stack_col = stack_col.push(mm);
    }
    stack_col = stack_col.push(Space::new().height(Length::Fill));
    stack_col = stack_col.push(bottom_row);

    let bottom_pad = if corners_at_top && matches!(d, Density::Touch) && !matches!(sc, SizeClass::Compact) {
        d.tray_button().1 + SPACE_3 * 2.0 + SPACE_2
    } else {
        SPACE_2
    };

    container(stack_col)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: SPACE_2,
            right: SPACE_2,
            bottom: bottom_pad,
            left: SPACE_2,
        })
        .into()
}

/// Optional open popover layer rendered above the controls and tray.
pub fn popover_layer<'a>(app: &'a App) -> Option<Element<'a, Message>> {
    let sc = SizeClass::of(app.window_size.width);
    let d = app.ui.density;
    let corners_at_top = corners_at_top(app);

    let (align_x, align_y, card): (Alignment, Alignment, Element<'a, Message>) =
        if app.view.filter_open {
            let va = if corners_at_top {
                Alignment::Start
            } else {
                Alignment::End
            };
            (Alignment::End, va, show_popover(app, d))
        } else {
            let kind = app.ui.controls_popover?;
            match kind {
                ControlsPopover::Zoom => {
                    let va = if corners_at_top {
                        Alignment::Start
                    } else {
                        Alignment::End
                    };
                    (Alignment::Start, va, zoom_popover(app, d))
                }
                ControlsPopover::Fit => {
                    let va = if corners_at_top {
                        Alignment::Start
                    } else {
                        Alignment::End
                    };
                    (Alignment::Start, va, fit_popover(d))
                }
                ControlsPopover::Snap => {
                    let va = if corners_at_top {
                        Alignment::Start
                    } else {
                        Alignment::End
                    };
                    (Alignment::End, va, snap_popover(app, d))
                }
                ControlsPopover::North => (Alignment::End, Alignment::Start, north_popover(app, d)),
                ControlsPopover::Underlay => {
                    (Alignment::Start, Alignment::Start, underlay_popover(app, d))
                }
            }
        };

    let show_underlay = app.pages.len() > 1 || app.pages.underlay_label() != "Underlay: None";
    let bar_offset = if matches!(sc, SizeClass::Compact) {
        if show_underlay {
            120.0
        } else {
            86.0
        }
    } else {
        56.0
    };
    let pad = Padding {
        top: if align_y == Alignment::Start {
            bar_offset
        } else {
            SPACE_2
        },
        bottom: if align_y == Alignment::End {
            bar_offset
        } else {
            SPACE_2
        },
        left: SPACE_2,
        right: SPACE_2,
    };

    let positioned = container(opaque(card))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(pad)
        .align_x(align_x)
        .align_y(align_y);

    Some(
        iced::widget::stack![
            mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
                .on_press(Message::Ui(UiMsg::CloseOverlays)),
            positioned,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    )
}

// ---------------------------------------------------------------------------
// Trigger bars
// ---------------------------------------------------------------------------

fn zoom_group<'a>(app: &'a App, d: Density, at_top: bool) -> Element<'a, Message> {
    let pct = format!("{}%", app.view.zoom_percent());
    let zoom_open = app.ui.controls_popover == Some(ControlsPopover::Zoom);
    let fit_open = app.ui.controls_popover == Some(ControlsPopover::Fit);

    let zoom_fence = kit::tooltip(
        compact_icon_btn(
            "magnifying-glass",
            app.view.mode == crate::view_state::Mode::ZoomFence,
            Message::Status(StatusMsg::ZoomFence),
            d,
        ),
        "Zoom to rectangle. Drag on the canvas.",
        if at_top {
            iced::widget::tooltip::Position::Bottom
        } else {
            iced::widget::tooltip::Position::Top
        },
    );
    let fit_btn = kit::long_press(
        mouse_area(chip_toggle(
            "Fit",
            Some("arrows-out"),
            false,
            Message::Status(StatusMsg::CenterFit),
            d,
        ))
        .on_right_press(Message::Ui(UiMsg::ToggleControlsPopover(ControlsPopover::Fit))),
        Message::Ui(UiMsg::ToggleControlsPopover(ControlsPopover::Fit)),
    );

    let fit_menu = compact_icon_btn(
        "caret-down",
        fit_open,
        Message::Ui(UiMsg::ToggleControlsPopover(ControlsPopover::Fit)),
        d,
    );

    row![
        compact_icon_btn("minus", false, Message::Status(StatusMsg::ZoomOut), d),
        chip_toggle(
            pct,
            None,
            zoom_open,
            Message::Ui(UiMsg::ToggleControlsPopover(ControlsPopover::Zoom)),
            d,
        ),
        compact_icon_btn("plus", false, Message::Status(StatusMsg::ZoomIn), d),
        zoom_fence,
        fit_btn,
        fit_menu,
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .into()
}

fn snap_group<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let pitch_open = app.ui.controls_popover == Some(ControlsPopover::Snap);
    let show_open = app.view.filter_open;
    let dim_label = if !app.view.dimensions {
        "Dims: Off"
    } else if app.view.dimension_placement == DimensionPlacement::Out {
        "Dims: Out"
    } else {
        "Dims: In"
    };

    row![
        chip_toggle(
            "Grid",
            Some("grid-four"),
            app.view.grid,
            Message::Status(StatusMsg::ToggleGrid),
            d,
        ),
        chip_toggle(
            "Snap",
            Some("magnet"),
            app.view.snap,
            Message::Status(StatusMsg::ToggleSnap),
            d,
        ),
        chip_toggle(
            app.view.pitch_label(),
            None,
            pitch_open || app.view.area_snap,
            Message::Ui(UiMsg::ToggleControlsPopover(ControlsPopover::Snap)),
            d,
        ),
        chip_toggle(
            "Show",
            Some("funnel"),
            show_open,
            Message::Status(StatusMsg::ToggleFilter),
            d,
        ),
        chip_toggle(
            dim_label,
            None,
            app.view.dimensions,
            Message::Status(StatusMsg::CycleDimensions),
            d,
        ),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .into()
}

fn north_chip<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let deg = format!("{}°", app.pages.page().north_angle.round() as i64);
    let open = app.ui.controls_popover == Some(ControlsPopover::North);
    chip_toggle(
        deg,
        Some("compass"),
        open,
        Message::Ui(UiMsg::ToggleControlsPopover(ControlsPopover::North)),
        d,
    )
}

fn underlay_chip<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let label = app.pages.underlay_label();
    let active = label != "Underlay: None";
    let open = app.ui.controls_popover == Some(ControlsPopover::Underlay);
    chip_toggle(
        label,
        Some("files"),
        active || open,
        Message::Ui(UiMsg::ToggleControlsPopover(ControlsPopover::Underlay)),
        d,
    )
}

// ---------------------------------------------------------------------------
// Popovers
// ---------------------------------------------------------------------------

fn zoom_popover<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let pct = (app.view.zoom_percent() as f64).clamp(10.0, 500.0);
    let scale_val = app
        .scale_edit
        .clone()
        .unwrap_or_else(|| crate::view_state::js_number(app.view.drawing_scale));

    let slider_row = row![
        text("10%")
            .font(BARLOW)
            .size(TEXT_11)
            .color(tokens::muted()),
        slider(10.0..=500.0, pct, |v| {
            Message::Status(StatusMsg::ZoomSlider(v))
        })
        .style(kit::slider_style)
        .width(Length::Fill),
        text("500%")
            .font(BARLOW)
            .size(TEXT_11)
            .color(tokens::muted()),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center);

    let scale_input = text_input("10", &scale_val)
        .on_input(|s| Message::Status(StatusMsg::ScaleEdit(s)))
        .on_submit(Message::Status(StatusMsg::ScaleSubmit))
        .size(TEXT_12)
        .font(BARLOW_MEDIUM)
        .padding([4, 8])
        .width(Length::Fixed(64.0))
        .style(input_style);

    let scale_row = row![
        text("Scale  1 :")
            .font(BARLOW_MEDIUM)
            .size(TEXT_12)
            .color(tokens::text()),
        scale_input,
        kit::text_button("Apply", Some(Message::Status(StatusMsg::ScaleSubmit)), d),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center);

    let body = column![
        row![
            text("Zoom & Scale")
                .font(BARLOW_SEMIBOLD)
                .size(TEXT_13)
                .color(tokens::text()),
            Space::new().width(Length::Fill),
            text(format!("{}%", app.view.zoom_percent()))
                .font(BSC_SEMIBOLD)
                .size(TEXT_13)
                .color(tokens::accent()),
        ]
        .align_y(Alignment::Center),
        slider_row,
        kit::divider(),
        scale_row,
    ]
    .spacing(SPACE_2)
    .width(Length::Fixed(240.0));

    kit::popover(body, RADIUS_DIALOG)
}

fn fit_popover<'a>(d: Density) -> Element<'a, Message> {
    let body = column![
        kit::text_button("Center", Some(Message::Status(StatusMsg::CenterSketch)), d),
        kit::text_button("Center & Fit", Some(Message::Status(StatusMsg::CenterFit)), d),
        kit::text_button("Zoom Fence", Some(Message::Status(StatusMsg::ZoomFence)), d),
    ]
    .spacing(SPACE_1)
    .width(Length::Fixed(164.0));

    kit::popover(body, RADIUS_DIALOG)
}

fn snap_popover<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let mut pitches = row![].spacing(SPACE_1);
    for (label, value) in PITCHES {
        let active = (app.view.area_snap_size - value).abs() < 1e-6;
        pitches = pitches.push(kit::chip(
            label,
            active,
            Some(Message::Status(StatusMsg::Pitch(PitchOption { label, value }))),
            d,
        ));
    }

    let area_snap = checkbox(app.view.area_snap)
        .label("Area snap")
        .on_toggle(|_| Message::Status(StatusMsg::ToggleAreaSnap))
        .size(16)
        .text_size(TEXT_12);

    let current_angle = app.drawing.options.ortho_angle;
    let mut angles = row![].spacing(SPACE_1);
    for &deg in ANGLE_STEPS {
        let label = if deg == 0.0 {
            "Off".to_string()
        } else {
            format!("{}°", deg as i64)
        };
        let active = (current_angle - deg).abs() < 0.01;
        angles = angles.push(kit::chip(
            label,
            active,
            Some(Message::Ui(UiMsg::SetAngleStep(deg))),
            d,
        ));
    }

    let body = column![
        text("Snap Pitch")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_12)
            .color(tokens::muted()),
        pitches,
        area_snap,
        kit::divider(),
        text("Angle Step")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_12)
            .color(tokens::muted()),
        angles,
    ]
    .spacing(SPACE_2)
    .width(Length::Fixed(250.0));

    kit::popover(body, RADIUS_DIALOG)
}

fn show_popover<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let mut list = Column::new().spacing(SPACE_1);
    for (i, &(_, label)) in FILTERS.iter().enumerate() {
        list = list.push(
            checkbox(app.view.filters[i])
                .label(label)
                .on_toggle(move |_| Message::Status(StatusMsg::Filter(i)))
                .size(16)
                .text_size(TEXT_12),
        );
    }

    let units = &app.drawing.options.units;
    let formats = kit::segmented(
        [("Decimal feet", false), ("Feet and inches", true)]
            .into_iter()
            .map(|(label, feet_and_inches)| kit::Segment {
                label,
                selected: !units.metric && units.feet_and_inches == feet_and_inches,
                focused: false,
                on_press: Some(Message::Status(StatusMsg::LengthFormat { feet_and_inches })),
            })
            .collect(),
        d,
    );
    let body = column![
        text("Show in sketch")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_13)
            .color(tokens::text()),
        kit::divider(),
        list,
        kit::divider(),
        text("Length format")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_13)
            .color(tokens::text()),
        formats,
    ]
    .spacing(SPACE_2)
    .width(Length::Fixed(204.0));

    kit::popover(body, RADIUS_DIALOG)
}

fn north_popover<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let input = text_input("0", &app.ui.north_draft)
        .id("north-angle-input")
        .on_input(|s| Message::Ui(UiMsg::NorthEdit(s)))
        .on_submit(Message::Ui(UiMsg::NorthSubmit))
        .size(TEXT_12)
        .font(BARLOW_MEDIUM)
        .padding([4, 8])
        .width(Length::Fixed(72.0))
        .style(input_style);

    let body = column![
        text("North angle (°)")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_12)
            .color(tokens::text()),
        row![
            input,
            kit::primary_button("Set", Some(Message::Ui(UiMsg::NorthSubmit)), d),
        ]
        .spacing(SPACE_1)
        .align_y(Alignment::Center),
    ]
    .spacing(SPACE_2)
    .width(Length::Fixed(164.0));

    kit::popover(body, RADIUS_DIALOG)
}

fn underlay_popover<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let current = app.pages.underlay_label();
    let options = app.pages.underlay_options();
    let mut col = Column::new().spacing(SPACE_1).width(Length::Fill);
    for (idx, label) in options {
        let selected = current == label;
        col = col.push(kit::list_row_owned(
            kit::OwnedListRowProps {
                swatch: None,
                title: label,
                value: None,
                selected,
                flagged: false,
                focused: false,
                on_press: Some(Message::Status(StatusMsg::Underlay(idx))),
            },
            d,
        ));
    }

    let scroll = scrollable(
        container(col).padding(Padding {
            right: 10.0,
            ..Padding::ZERO
        }),
    )
    .direction(Direction::Vertical(
        Scrollbar::new().width(5).scroller_width(5),
    ))
    .height(Length::Shrink);

    let body = column![
        text("Underlay page")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_12)
            .color(tokens::text()),
        scroll,
    ]
    .spacing(SPACE_2)
    .width(Length::Fixed(210.0));

    kit::popover(body, RADIUS_DIALOG)
}

fn minimap_card<'a>(handle: &image::Handle, sc: SizeClass) -> Element<'a, Message> {
    let w = if matches!(sc, SizeClass::Compact) {
        156.0
    } else {
        220.0
    };
    let close = button(
        container(icons::mono("x", 12.0, tokens::on_accent()))
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .on_press(Message::MiniMapClose)
    .style(|_, _| button::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.65))),
        border: Border {
            radius: RADIUS_CONTROL.into(),
            ..Border::default()
        },
        ..button::Style::default()
    });

    let card = container(iced::widget::stack![
        image(handle.clone())
            .width(Length::Fill)
            .content_fit(iced::ContentFit::Contain),
        container(close)
            .width(Length::Fill)
            .align_x(Alignment::End)
            .padding(2),
    ])
    .width(Length::Fixed(w))
    .padding(SPACE_1)
    .style(|_| container::Style {
        background: Some(Background::Color(tokens::panel())),
        border: Border {
            color: tokens::rule(),
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        shadow: tokens::floating_shadow(),
        ..container::Style::default()
    });

    opaque(card)
}

// ---------------------------------------------------------------------------
// Shared chrome widgets
// ---------------------------------------------------------------------------

fn pill_surface<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    opaque(
        container(content)
            .padding(Padding {
                top: 3.0,
                right: 4.0,
                bottom: 3.0,
                left: 4.0,
            })
            .style(|_| container::Style {
                background: Some(Background::Color(tokens::bar())),
                border: Border {
                    color: tokens::rule(),
                    width: 1.0,
                    radius: RADIUS_DIALOG.into(),
                },
                shadow: tokens::floating_shadow(),
                ..container::Style::default()
            }),
    )
}

fn chip_toggle<'a>(
    label: impl Into<String>,
    icon_name: Option<&str>,
    active: bool,
    on_press: Message,
    d: Density,
) -> Element<'a, Message> {
    let fg = if active {
        tokens::accent()
    } else {
        tokens::text()
    };
    let h = if matches!(d, Density::Touch) {
        32.0
    } else {
        26.0
    };
    let mut r = row![].spacing(SPACE_1).align_y(Alignment::Center);
    if let Some(icon) = icon_name {
        r = r.push(if active {
            icons::mono(icon, 13.0, fg)
        } else {
            icons::tinted(icon, false, 13.0, fg)
        });
    }
    r = r.push(
        text(label.into())
            .font(if active { BARLOW_SEMIBOLD } else { BARLOW_MEDIUM })
            .size(TEXT_12)
            .color(fg),
    );
    button(
        container(r)
            .height(Length::Fixed(h))
            .padding([0, 7])
            .align_y(Alignment::Center)
            .align_x(Alignment::Center),
    )
    .padding(0)
    .on_press(on_press)
    .style(move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if active {
                tokens::accent_soft()
            } else if hovered {
                tokens::hover()
            } else {
                Color::TRANSPARENT
            })),
            text_color: fg,
            border: Border {
                color: if active {
                    tokens::accent()
                } else {
                    Color::TRANSPARENT
                },
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..button::Style::default()
        }
    })
    .into()
}

fn compact_icon_btn<'a>(
    icon_name: &str,
    active: bool,
    on_press: Message,
    d: Density,
) -> Element<'a, Message> {
    let fg = if active {
        tokens::accent()
    } else {
        tokens::text()
    };
    let size = if matches!(d, Density::Touch) {
        32.0
    } else {
        26.0
    };
    button(
        container(if active {
            icons::mono(icon_name, 16.0, fg)
        } else {
            icons::tinted(icon_name, false, 16.0, fg)
        })
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .on_press(on_press)
    .style(move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if active {
                tokens::accent_soft()
            } else if hovered {
                tokens::hover()
            } else {
                Color::TRANSPARENT
            })),
            border: Border {
                radius: RADIUS_CONTROL.into(),
                ..Border::default()
            },
            ..button::Style::default()
        }
    })
    .into()
}

fn input_style(_: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let focused = matches!(status, text_input::Status::Focused { .. });
    text_input::Style {
        background: Background::Color(tokens::input_bg()),
        border: Border {
            color: if focused {
                tokens::accent()
            } else {
                tokens::rule()
            },
            width: if focused { FOCUS_RING_WIDTH } else { 1.0 },
            radius: RADIUS_CONTROL.into(),
        },
        icon: tokens::muted(),
        placeholder: tokens::muted(),
        value: tokens::text(),
        selection: tokens::accent_soft(),
    }
}
