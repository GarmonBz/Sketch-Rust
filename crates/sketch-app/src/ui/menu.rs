//! Grouped application Menu (`docs/ui-redesign/01-design-spec.md` §5.9).
//!
//! Renders the 8 command groups (`File`, `Edit`, `View`, `Areas`, `Insert`,
//! `Image`, `Tools`, `Help`) from `ui::commands` along with nested panels for
//! Recent sketches, Select by type, Flip, Snap pitch, Show in sketch filters,
//! Drawing background, Interface theme, and Units.
//!
//! Layout adapts by size class:
//! - `Wide` / `Medium`: floating dropdown anchored below the top bar's Menu
//!   button with left group column + scrollable command list (+ optional
//!   submenu pane).
//! - `Compact`: full sheet with group switcher and clean back-navigable
//!   submenus.

use iced::widget::{
    button, column, container, mouse_area, opaque, row, scrollable, text, Space,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding};

use crate::canvas_theme::{self, CanvasTheme};
use crate::icons;
use crate::select_menu::SelectKind;
use crate::theme::{self, InterfaceTheme};
use crate::transform::Axis;
use crate::ui::commands::{self, Command, Group};
use crate::ui::fonts::{BARLOW, BARLOW_MEDIUM, BARLOW_SEMIBOLD, BSC_MEDIUM, BSC_SEMIBOLD};
use crate::ui::kit;
use crate::ui::layout::SizeClass;
use crate::ui::tokens::{
    self, Density, RADIUS_CONTROL, RADIUS_DIALOG, SPACE_1, SPACE_2, SPACE_3, TEXT_11, TEXT_12,
    TEXT_13, TEXT_14,
};
use crate::ui::Msg as UiMsg;
use crate::view_state::{FILTERS, PITCHES};
use crate::{App, Message};

/// Open submenu inside the Menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSub {
    Recent,
    SelectByType,
    Flip,
    Pitch,
    Filters,
    CanvasBg,
    Theme,
    Units,
}

impl MenuSub {
    pub fn title(self) -> &'static str {
        match self {
            MenuSub::Recent => "Recent sketches",
            MenuSub::SelectByType => "Select by type",
            MenuSub::Flip => "Flip sketch",
            MenuSub::Pitch => "Snap pitch",
            MenuSub::Filters => "Show in sketch",
            MenuSub::CanvasBg => "Drawing background",
            MenuSub::Theme => "Interface theme",
            MenuSub::Units => "Units",
        }
    }
}

/// Interactive row entry inside the active group pane.
pub enum GroupItem {
    Cmd(&'static Command),
    Sub(MenuSub),
}

/// Actionable submenu entry used for both rendering and Enter-key execution.
pub struct SubAction {
    pub label: String,
    pub checked: bool,
    pub enabled: bool,
    pub on_press: Option<Message>,
}

/// Ordered items displayed in the middle pane for `group`.
pub fn group_items(group: Group) -> Vec<GroupItem> {
    let mut items = Vec::new();
    match group {
        Group::File => {
            items.push(GroupItem::Sub(MenuSub::Recent));
        }
        Group::Edit => {
            items.push(GroupItem::Sub(MenuSub::SelectByType));
            items.push(GroupItem::Sub(MenuSub::Flip));
        }
        Group::View => {
            items.push(GroupItem::Sub(MenuSub::Pitch));
            items.push(GroupItem::Sub(MenuSub::Filters));
            items.push(GroupItem::Sub(MenuSub::CanvasBg));
            items.push(GroupItem::Sub(MenuSub::Theme));
            items.push(GroupItem::Sub(MenuSub::Units));
        }
        Group::Areas | Group::Insert | Group::Image | Group::Tools | Group::Help => {}
    }
    for cmd in commands::in_group(group) {
        items.push(GroupItem::Cmd(cmd));
    }
    items
}

/// Submenu entries for `sub` against current `app` state.
pub fn sub_actions(app: &App, sub: MenuSub) -> Vec<SubAction> {
    match sub {
        MenuSub::Recent => {
            if app.recent.is_empty() {
                vec![SubAction {
                    label: "No recent sketches".to_owned(),
                    checked: false,
                    enabled: false,
                    on_press: None,
                }]
            } else {
                let mut out: Vec<SubAction> = app
                    .recent
                    .iter()
                    .enumerate()
                    .map(|(i, rec)| SubAction {
                        label: rec.name.clone(),
                        checked: false,
                        enabled: true,
                        on_press: Some(Message::Ui(UiMsg::MenuRunRecent(i))),
                    })
                    .collect();
                out.push(SubAction {
                    label: "Clear recent sketches".to_owned(),
                    checked: false,
                    enabled: true,
                    on_press: Some(Message::Ui(UiMsg::MenuClearRecent)),
                });
                out
            }
        }
        MenuSub::SelectByType => SelectKind::ALL
            .iter()
            .copied()
            .map(|kind| SubAction {
                label: kind.label().to_owned(),
                checked: false,
                enabled: true,
                on_press: Some(Message::Ui(UiMsg::MenuSelectKind(kind))),
            })
            .collect(),
        MenuSub::Flip => vec![
            SubAction {
                label: "Flip horizontally".to_owned(),
                checked: false,
                enabled: !app.drawing.lines.is_empty() || !app.drawing.polygons.is_empty(),
                on_press: Some(Message::Ui(UiMsg::MenuFlip(Axis::Horizontal))),
            },
            SubAction {
                label: "Flip vertically".to_owned(),
                checked: false,
                enabled: !app.drawing.lines.is_empty() || !app.drawing.polygons.is_empty(),
                on_press: Some(Message::Ui(UiMsg::MenuFlip(Axis::Vertical))),
            },
        ],
        MenuSub::Pitch => PITCHES
            .iter()
            .enumerate()
            .map(|(idx, (label, val))| SubAction {
                label: (*label).to_owned(),
                checked: (app.view.area_snap_size - val).abs() < 1e-4,
                enabled: true,
                on_press: Some(Message::Ui(UiMsg::MenuPitch(idx))),
            })
            .collect(),
        MenuSub::Filters => FILTERS
            .iter()
            .enumerate()
            .map(|(idx, (_key, label))| SubAction {
                label: (*label).to_owned(),
                checked: app.view.filters.get(idx).copied().unwrap_or(true),
                enabled: true,
                on_press: Some(Message::Ui(UiMsg::MenuFilter(idx))),
            })
            .collect(),
        MenuSub::CanvasBg => [CanvasTheme::Light, CanvasTheme::Dark, CanvasTheme::Blue]
            .into_iter()
            .map(|t| SubAction {
                label: t.label().to_owned(),
                checked: canvas_theme::current() == t,
                enabled: true,
                on_press: Some(Message::Ui(UiMsg::MenuCanvasTheme(t))),
            })
            .collect(),
        MenuSub::Theme => [
            (InterfaceTheme::Dark, "Dark"),
            (InterfaceTheme::Blue, "Blue"),
            (InterfaceTheme::Light, "Light"),
        ]
        .into_iter()
        .map(|(t, label)| SubAction {
            label: label.to_owned(),
            checked: theme::interface_theme() == t,
            enabled: true,
            on_press: Some(Message::Ui(UiMsg::MenuInterfaceTheme(t))),
        })
        .collect(),
        MenuSub::Units => vec![
            SubAction {
                label: "Imperial (ft / sf)".to_owned(),
                checked: !app.drawing.options.units.metric,
                enabled: true,
                on_press: Some(Message::Ui(UiMsg::MenuUnitsMetric(false))),
            },
            SubAction {
                label: "Metric (m / m²)".to_owned(),
                checked: app.drawing.options.units.metric,
                enabled: true,
                on_press: Some(Message::Ui(UiMsg::MenuUnitsMetric(true))),
            },
        ],
    }
}

/// Number of rows currently navigable with Up/Down arrows.
pub fn active_row_count(app: &App) -> usize {
    if let Some(sub) = app.ui.menu_sub {
        sub_actions(app, sub).len()
    } else {
        group_items(app.ui.menu_group).len()
    }
}

/// Renders the Next Menu overlay when `app.ui.menu_open` is true.
pub fn layer<'a>(app: &'a App) -> Option<Element<'a, Message>> {
    if !app.ui.menu_open {
        return None;
    }

    let d = app.ui.density;
    let size_class = SizeClass::of(app.window_size.width);

    let body: Element<'a, Message> = match size_class {
        SizeClass::Wide | SizeClass::Medium => desktop_dropdown(app, d),
        SizeClass::Compact => compact_sheet(app, d),
    };

    let backdrop = mouse_area(
        container(opaque(body))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(match size_class {
                SizeClass::Wide | SizeClass::Medium => Padding {
                    top: d.bar_h() + 4.0,
                    left: SPACE_3,
                    right: SPACE_3,
                    bottom: SPACE_3,
                },
                SizeClass::Compact => Padding {
                    top: d.bar_h(),
                    left: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                },
            })
            .style(|_| container::Style {
                background: Some(Background::Color(tokens::scrim())),
                ..container::Style::default()
            }),
    )
    .on_press(Message::Ui(UiMsg::CloseMenu));

    Some(opaque(backdrop))
}

fn desktop_dropdown<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let has_sub = app.ui.menu_sub.is_some();
    let max_h = (app.window_size.height - d.bar_h() - SPACE_3 * 2.0).clamp(260.0, 560.0);
    let target_w: f32 = if has_sub { 760.0 } else { 520.0 };
    let w = target_w.min((app.window_size.width - SPACE_3 * 2.0).max(320.0));

    let group_col = group_list_column(app, d);
    let item_pane = scrollable(items_column(app, d, app.ui.menu_sub.is_none()))
        .height(Length::Fill)
        .width(Length::Fill);

    let mut columns = row![
        container(group_col).width(Length::Fixed(156.0)).height(Length::Fill),
        container(Space::new())
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(tokens::rule())),
                ..container::Style::default()
            }),
        container(item_pane)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([SPACE_2, SPACE_2]),
    ]
    .height(Length::Fill);

    if let Some(sub) = app.ui.menu_sub {
        columns = columns
            .push(
                container(Space::new())
                    .width(Length::Fixed(1.0))
                    .height(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(Background::Color(tokens::rule())),
                        ..container::Style::default()
                    }),
            )
            .push(
                container(sub_pane(app, sub, d, false))
                    .width(Length::Fixed(236.0))
                    .height(Length::Fill)
                    .padding([SPACE_2, SPACE_2]),
            );
    }

    let header = row![
        text("MENU")
            .font(BSC_SEMIBOLD)
            .size(TEXT_12)
            .color(tokens::muted()),
        Space::new().width(Length::Fill),
        kit::text_button_with(
            "Search commands (Ctrl+K)",
            Some(Message::Ui(UiMsg::MenuOpenSearch)),
            d,
            false,
            false,
        ),
    ]
    .align_y(Alignment::Center)
    .padding([SPACE_2, SPACE_3]);

    let panel = container(column![header, kit::divider(), columns])
        .width(Length::Fixed(w))
        .height(Length::Fixed(max_h))
        .clip(true)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::bar())),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: RADIUS_DIALOG.into(),
            },
            shadow: tokens::floating_shadow(),
            ..container::Style::default()
        });

    mouse_area(panel).on_press(Message::Ui(UiMsg::Noop)).into()
}

fn compact_sheet<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let header = row![
        text("MENU")
            .font(BSC_SEMIBOLD)
            .size(TEXT_12)
            .color(tokens::muted()),
        Space::new().width(Length::Fill),
        kit::text_button("Search", Some(Message::Ui(UiMsg::MenuOpenSearch)), d),
        kit::text_button("Close", Some(Message::Ui(UiMsg::CloseMenu)), d),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .padding([SPACE_2, SPACE_3]);

    let mut row1 = row![].spacing(SPACE_1);
    let mut row2 = row![].spacing(SPACE_1);
    for (idx, &group) in Group::ALL.iter().enumerate() {
        let pill = kit::chip(
            group.label(),
            app.ui.menu_group == group,
            Some(Message::Ui(UiMsg::MenuGroup(group))),
            d,
        );
        if idx < 4 {
            row1 = row1.push(pill);
        } else {
            row2 = row2.push(pill);
        }
    }
    let group_strip = container(column![row1, row2].spacing(SPACE_1))
        .padding([SPACE_1, SPACE_3])
        .width(Length::Fill);

    let body: Element<'a, Message> = if let Some(sub) = app.ui.menu_sub {
        sub_pane(app, sub, d, true)
    } else {
        scrollable(items_column(app, d, true))
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    };

    let sheet = container(column![
        header,
        kit::divider(),
        group_strip,
        kit::divider(),
        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([SPACE_2, SPACE_3]),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(tokens::bar())),
        border: Border {
            color: tokens::rule(),
            width: 1.0,
            ..Border::default()
        },
        ..container::Style::default()
    });

    mouse_area(sheet).on_press(Message::Ui(UiMsg::Noop)).into()
}

fn group_list_column<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let mut col = column![].spacing(SPACE_1).padding(SPACE_2);
    for &group in &Group::ALL {
        let selected = app.ui.menu_group == group;
        let fg = if selected {
            tokens::accent()
        } else {
            tokens::text()
        };
        let bg = if selected {
            tokens::accent_soft()
        } else {
            Color::TRANSPARENT
        };
        let border_c = if selected {
            Some(tokens::accent())
        } else {
            None
        };
        let btn = button(
            container(
                row![
                    icons::mono(group.icon(), 15.0, fg),
                    text(group.label())
                        .font(if selected { BARLOW_SEMIBOLD } else { BARLOW_MEDIUM })
                        .size(TEXT_13)
                        .color(fg)
                        .width(Length::Fill),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            )
            .height(Length::Fixed(d.row_h().min(36.0)))
            .padding([0, 8])
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .padding(0)
        .width(Length::Fill)
        .style(move |_, status| {
            let hover_bg = match status {
                button::Status::Hovered => tokens::hover(),
                button::Status::Pressed => tokens::pressed(),
                _ => bg,
            };
            button::Style {
                background: Some(Background::Color(hover_bg)),
                text_color: fg,
                border: Border {
                    color: border_c.unwrap_or(Color::TRANSPARENT),
                    width: if border_c.is_some() { 1.0 } else { 0.0 },
                    radius: RADIUS_CONTROL.into(),
                },
                ..button::Style::default()
            }
        })
        .on_press(Message::Ui(UiMsg::MenuGroup(group)));
        col = col.push(btn);
    }
    col.into()
}

fn items_column<'a>(app: &'a App, d: Density, keyboard_active: bool) -> Element<'a, Message> {
    let items = group_items(app.ui.menu_group);
    let mut col = column![].spacing(2).width(Length::Fill);

    for (idx, item) in items.into_iter().enumerate() {
        let focused = keyboard_active && app.ui.menu_row == idx;
        match item {
            GroupItem::Sub(sub) => {
                let open = app.ui.menu_sub == Some(sub);
                let fg = if open { tokens::accent() } else { tokens::text() };
                let r = row![
                    Space::new().width(Length::Fixed(14.0)),
                    text(sub.title())
                        .font(if open { BARLOW_SEMIBOLD } else { BARLOW_MEDIUM })
                        .size(TEXT_13)
                        .color(fg)
                        .width(Length::Fill),
                    icons::mono("caret-right", 12.0, tokens::muted()),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center);
                col = col.push(menu_row_button(
                    r,
                    d,
                    open,
                    focused,
                    Some(Message::Ui(UiMsg::MenuSub(Some(sub)))),
                ));
            }
            GroupItem::Cmd(cmd) => {
                let enabled = (cmd.enabled)(app);
                let checked = cmd.checked.is_some_and(|f| f(app));
                let fg = if enabled {
                    tokens::text()
                } else {
                    tokens::disabled()
                };
                let mut r = row![].spacing(SPACE_2).align_y(Alignment::Center);
                if checked {
                    r = r.push(icons::mono("check", 14.0, tokens::accent()));
                } else {
                    r = r.push(Space::new().width(Length::Fixed(14.0)));
                }
                r = r.push(
                    text(cmd.label)
                        .font(BARLOW_MEDIUM)
                        .size(TEXT_13)
                        .color(fg)
                        .width(Length::Fill),
                );
                if !cmd.shortcut.is_empty() {
                    r = r.push(
                        text(cmd.shortcut)
                            .font(BSC_MEDIUM)
                            .size(TEXT_12)
                            .color(if enabled {
                                tokens::muted()
                            } else {
                                tokens::disabled()
                            }),
                    );
                }
                let on_press = enabled.then(|| Message::Ui(UiMsg::MenuRunId(cmd.id)));
                col = col.push(menu_row_button(r, d, false, focused, on_press));
            }
        }
    }

    container(col)
        .padding(Padding {
            top: 0.0,
            right: 14.0,
            bottom: 0.0,
            left: 0.0,
        })
        .width(Length::Fill)
        .into()
}

fn sub_pane<'a>(app: &'a App, sub: MenuSub, d: Density, compact: bool) -> Element<'a, Message> {
    let mut header = row![].spacing(SPACE_2).align_y(Alignment::Center);
    if compact {
        header = header.push(kit::text_button(
            "Back",
            Some(Message::Ui(UiMsg::MenuSub(None))),
            d,
        ));
    }
    header = header.push(
        text(sub.title())
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_14)
            .color(tokens::text()),
    );

    let actions = sub_actions(app, sub);
    let mut list = column![].spacing(2).width(Length::Fill);
    for (idx, act) in actions.into_iter().enumerate() {
        let focused = app.ui.menu_row == idx;
        let fg = if act.enabled {
            tokens::text()
        } else {
            tokens::disabled()
        };
        let mut r = row![].spacing(SPACE_2).align_y(Alignment::Center);
        if act.checked {
            r = r.push(icons::mono("check", 14.0, tokens::accent()));
        } else {
            r = r.push(Space::new().width(Length::Fixed(14.0)));
        }
        r = r.push(
            text(act.label)
                .font(BARLOW)
                .size(TEXT_13)
                .color(fg)
                .width(Length::Fill),
        );
        if act.checked {
            r = r.push(
                text("Active")
                    .font(BSC_MEDIUM)
                    .size(TEXT_11)
                    .color(tokens::accent()),
            );
        }
        list = list.push(menu_row_button(r, d, act.checked, focused, act.on_press));
    }

    column![
        container(header).padding([SPACE_1, SPACE_1]),
        kit::divider(),
        scrollable(
            container(list)
                .padding(Padding {
                    top: 0.0,
                    right: 14.0,
                    bottom: 0.0,
                    left: 0.0,
                })
                .width(Length::Fill),
        )
        .height(Length::Fill)
        .width(Length::Fill),
    ]
    .spacing(SPACE_2)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn menu_row_button<'a>(
    content: row::Row<'a, Message>,
    density: Density,
    active: bool,
    focused: bool,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let disabled = on_press.is_none();
    let base_bg = if (active || focused) && !disabled {
        tokens::accent_soft()
    } else {
        Color::TRANSPARENT
    };
    let mut btn = button(
        container(content)
            .height(Length::Fixed(density.row_h().min(36.0)))
            .padding([0, 8])
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .padding(0)
    .width(Length::Fill)
    .style(move |_, status| {
        let bg = match status {
            button::Status::Disabled => Color::TRANSPARENT,
            button::Status::Hovered => tokens::hover(),
            button::Status::Pressed => tokens::pressed(),
            button::Status::Active => base_bg,
        };
        let border = if focused && !disabled {
            Border {
                color: tokens::accent(),
                width: 1.5,
                radius: RADIUS_CONTROL.into(),
            }
        } else {
            Border {
                radius: RADIUS_CONTROL.into(),
                ..Border::default()
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            border,
            ..button::Style::default()
        }
    });
    if let Some(msg) = on_press {
        btn = btn.on_press(msg);
    }
    btn.into()
}
