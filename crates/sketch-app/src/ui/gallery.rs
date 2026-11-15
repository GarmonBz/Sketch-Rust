//! Developer widget gallery for verifying P1 tokens, fonts, icons, densities,
//! and widget states (`SKETCH_UI_GALLERY=1`).

use iced::widget::{column, container, row, scrollable, text, tooltip::Position, Space};
use iced::{Alignment, Background, Border, Element, Length};

use crate::theme::hex;
use crate::ui::fonts::{BARLOW, BARLOW_SEMIBOLD, BSC_SEMIBOLD};
use crate::ui::kit::{self, FieldProps, ListRowProps, MenuEntry, Segment};
use crate::ui::tokens::{
    self, Density, NUM_24, RADIUS_CONTROL, RADIUS_DIALOG, SPACE_1, SPACE_2, SPACE_3, SPACE_4,
    TEXT_12, TEXT_14, TEXT_16,
};
use crate::ui::Msg;
use crate::{App, Message};

fn noop() -> Message {
    Message::Ui(Msg::Noop)
}

/// Root gallery view displaying every `kit.rs` widget in both `Comfortable`
/// and `Touch` densities across normal, active/selected, focused, and disabled states.
pub fn view(app: &App) -> Element<'_, Message> {
    let header = container(
        row![
            text("Sketch Rust UI Kit Gallery")
                .font(BARLOW_SEMIBOLD)
                .size(TEXT_16)
                .color(tokens::text()),
            Space::new().width(Length::Fill),
            kit::segmented(
                vec![
                    Segment {
                        label: "Comfortable",
                        selected: app.ui.density == Density::Comfortable,
                        focused: false,
                        on_press: Some(Message::Ui(Msg::SetDensity(Density::Comfortable))),
                    },
                    Segment {
                        label: "Touch",
                        selected: app.ui.density == Density::Touch,
                        focused: false,
                        on_press: Some(Message::Ui(Msg::SetDensity(Density::Touch))),
                    },
                ],
                app.ui.density,
            ),
            kit::text_button("Close Gallery", Some(Message::Ui(Msg::SetGallery(false))), app.ui.density),
        ]
        .spacing(SPACE_3)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .height(Length::Fixed(app.ui.density.bar_h()))
    .padding([0, 16])
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(tokens::bar())),
        ..container::Style::default()
    });

    let body = row![
        density_column(app, Density::Comfortable, "COMFORTABLE (44 / 36 / 32 / 62×58)"),
        density_column(app, Density::Touch, "TOUCH (56 / 48 / 44 / 68×68)"),
    ]
    .spacing(SPACE_4)
    .width(Length::Fill);

    let scroller = scrollable(
        container(body)
            .padding(SPACE_4)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    container(column![header, kit::divider(), scroller].width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::canvas())),
            ..container::Style::default()
        })
        .into()
}

fn density_column<'a>(app: &'a App, d: Density, heading: &'a str) -> Element<'a, Message> {
    let buttons_card = kit::panel_card(
        column![
            kit::group_header("BUTTONS & ICON BUTTONS", None),
            row![
                kit::tooltip(kit::text_button("Normal", Some(noop()), d), "Secondary button", Position::Bottom),
                kit::text_button_with("Active", Some(noop()), d, true, false),
                kit::text_button_with("Focused", Some(noop()), d, false, true),
                kit::text_button("Disabled", None, d),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
            row![
                kit::primary_button("Primary", Some(noop()), d),
                kit::primary_button_with("Focused", Some(noop()), d, true),
                kit::primary_button("Disabled", None, d),
                kit::icon_button("list", false, Some(noop()), d),
                kit::icon_button("push-pin", true, Some(noop()), d),
                kit::icon_button_with("compass", false, true, Some(noop()), d),
                kit::icon_button("dots-three", false, None, d),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        ]
        .spacing(SPACE_2),
    );

    let inputs_card = kit::panel_card(
        column![
            kit::group_header("FIELDS, STEPPER, SEGMENTED, CHIPS", None),
            row![
                container(kit::field(
                    FieldProps {
                        label: "Length / Label",
                        value: &app.ui.gallery_input,
                        placeholder: "Type here...",
                        on_input: Some(Box::new(|s| Message::Ui(Msg::GalleryInput(s)))),
                        on_submit: None,
                        focused: false,
                    },
                    d,
                ))
                .width(Length::FillPortion(1)),
                container(kit::field(
                    FieldProps {
                        label: "Focused",
                        value: "24'-6\"",
                        placeholder: "",
                        on_input: Some(Box::new(|_| noop())),
                        on_submit: None,
                        focused: true,
                    },
                    d,
                ))
                .width(Length::FillPortion(1)),
                container(kit::field(
                    FieldProps {
                        label: "Disabled",
                        value: "Read only",
                        placeholder: "",
                        on_input: None,
                        on_submit: None,
                        focused: false,
                    },
                    d,
                ))
                .width(Length::FillPortion(1)),
            ]
            .spacing(SPACE_2),
            row![
                kit::stepper("Scale", "1/4\"", Some(noop()), Some(noop()), d),
                kit::segmented(
                    vec![
                        Segment { label: "Feet", selected: true, focused: false, on_press: Some(noop()) },
                        Segment { label: "Inches", selected: false, focused: true, on_press: Some(noop()) },
                        Segment { label: "Metric", selected: false, focused: false, on_press: None },
                    ],
                    d,
                ),
            ]
            .spacing(SPACE_3)
            .align_y(Alignment::Center),
            row![
                kit::chip("Floor 1", false, Some(noop()), d),
                kit::chip("Selected", true, Some(noop()), d),
                kit::chip_with("Focused", false, true, Some(noop()), d),
                kit::chip("Disabled", false, None, d),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        ]
        .spacing(SPACE_2),
    );

    let ledger_card = kit::panel_card(
        column![
            kit::group_header(
                "LEDGER ROWS & SWATCHES",
                Some(
                    text("1,482 sq ft")
                        .font(BSC_SEMIBOLD)
                        .size(TEXT_12)
                        .color(tokens::muted())
                        .into(),
                ),
            ),
            kit::list_row(
                ListRowProps {
                    swatch: Some(hex(0x4a9eff)),
                    title: "Living Area 1 (Normal)",
                    value: Some("1,240 sq ft"),
                    selected: false,
                    flagged: false,
                    focused: false,
                    on_press: Some(noop()),
                },
                d,
            ),
            kit::list_row(
                ListRowProps {
                    swatch: Some(hex(0x52c482)),
                    title: "Garage (Selected)",
                    value: Some("420 sq ft"),
                    selected: true,
                    flagged: false,
                    focused: false,
                    on_press: Some(noop()),
                },
                d,
            ),
            kit::list_row(
                ListRowProps {
                    swatch: None,
                    title: "Closed Loop 2 (Focused / Needs a type)",
                    value: Some("185 sq ft"),
                    selected: false,
                    flagged: true,
                    focused: true,
                    on_press: Some(noop()),
                },
                d,
            ),
            kit::list_row(
                ListRowProps {
                    swatch: Some(hex(0xf6ad55)),
                    title: "Locked Area (Disabled)",
                    value: Some("96 sq ft"),
                    selected: false,
                    flagged: false,
                    focused: false,
                    on_press: None,
                },
                d,
            ),
            row![
                kit::swatch(hex(0x4a9eff), false, false, Some(noop()), d),
                kit::swatch(hex(0x52c482), true, false, Some(noop()), d),
                kit::swatch(hex(0x9b7ee6), false, true, Some(noop()), d),
                kit::swatch(hex(0xf6ad55), false, false, None, d),
                container(
                    text("12'-6\"")
                        .font(BSC_SEMIBOLD)
                        .size(TEXT_12)
                        .color(tokens::tape_ink()),
                )
                .padding([3, 8])
                .style(|_| container::Style {
                    background: Some(Background::Color(tokens::tape())),
                    border: Border {
                        radius: RADIUS_CONTROL.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }),
                Space::new().width(Length::Fill),
                text("1,941")
                    .font(BSC_SEMIBOLD)
                    .size(NUM_24)
                    .color(tokens::text()),
                text("sq ft").font(BARLOW).size(TEXT_12).color(tokens::muted()),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        ]
        .spacing(SPACE_1),
    );

    let floating_card = kit::panel_card(
        column![
            kit::group_header("TRAY TILES, MENU, POPOVER, SHEET HANDLE", None),
            kit::tray_surface(
                row![
                    kit::tile("pencil-simple", "Line", false, false, Some(noop()), d),
                    kit::tile("polygon", "Area", true, false, Some(noop()), d),
                    kit::tile("house-line", "Room", false, true, Some(noop()), d),
                    kit::tile("bezier-curve", "Arc", false, false, None, d),
                ]
                .spacing(SPACE_1),
            ),
            row![
                kit::menu(
                    vec![
                        MenuEntry::Item {
                            icon: Some("files"),
                            label: "Save Drawing",
                            shortcut: Some("Ctrl+S"),
                            checked: false,
                            focused: false,
                            on_press: Some(noop()),
                        },
                        MenuEntry::Item {
                            icon: None,
                            label: "Snap to Grid",
                            shortcut: Some("G"),
                            checked: true,
                            focused: true,
                            on_press: Some(noop()),
                        },
                        MenuEntry::Separator,
                        MenuEntry::Item {
                            icon: Some("arrow-counter-clockwise"),
                            label: "Undo Last",
                            shortcut: Some("Ctrl+Z"),
                            checked: false,
                            focused: false,
                            on_press: None,
                        },
                    ],
                    240.0,
                    d,
                ),
                column![
                    kit::popover(
                        column![
                            text("Popover Card")
                                .font(BARLOW_SEMIBOLD)
                                .size(TEXT_14)
                                .color(tokens::text()),
                            text("Floating shadow & 1px rule border")
                                .font(BARLOW)
                                .size(TEXT_12)
                                .color(tokens::muted()),
                        ]
                        .spacing(SPACE_1),
                        RADIUS_DIALOG,
                    ),
                    kit::sheet_handle("Inspector Sheet", Some(noop())),
                ]
                .spacing(SPACE_2)
                .width(Length::Fill),
            ]
            .spacing(SPACE_3)
            .align_y(Alignment::Start),
        ]
        .spacing(SPACE_2),
    );

    column![
        text(heading)
            .font(BSC_SEMIBOLD)
            .size(TEXT_12)
            .color(tokens::muted()),
        buttons_card,
        inputs_card,
        ledger_card,
        floating_card,
    ]
    .spacing(SPACE_3)
    .width(Length::FillPortion(1))
    .into()
}
