//! Left-docked / sheet Ledger (`ui/ledger.rs`):
//! Drawn areas, Area types palette, Layers tree, Totals foot, and `"..."` menu
//! for the Next shell (`docs/ui-redesign/01-design-spec.md` §5.3 & `03-phases.md` P6).

use std::collections::HashMap;

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    button, column, container, mouse_area, opaque, row, scrollable as make_scrollable, stack,
    text, text_input, Column, Space,
};
use iced::{alignment, mouse, Alignment, Background, Border, Color, Element, Length, Padding};
use sketch_core::geometry::{distance, AreaCode};

use crate::area_workflow::{self, Msg as WorkflowMsg};
use crate::dock::{self, palette_group_label, DragItem};
use crate::icons;
use crate::layers::{DragPos, Row as LayerRow};
use crate::ui::fonts::{BARLOW, BARLOW_MEDIUM, BARLOW_SEMIBOLD, BSC_MEDIUM, BSC_SEMIBOLD};
use crate::ui::kit::{self, MenuEntry};
use crate::ui::layout::SizeClass;
use crate::ui::tokens::{
    self, Density, NUM_24, NUM_32, RADIUS_CONTROL, RADIUS_DIALOG, SPACE_1, SPACE_2, SPACE_3,
    TEXT_11, TEXT_12, TEXT_13, TEXT_14,
};
use crate::ui::Msg as UiMsg;
use crate::view_state::DimensionPlacement;
use crate::{calcs, App, Message};

pub const MIN_W: f32 = 220.0;
pub const MAX_W: f32 = 400.0;
pub const CANVAS_MIN_W: f32 = 220.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LedgerView {
    #[default]
    Drawn,
    Types,
    Layers,
}

/// Clamps dragged ledger width to `[MIN_W, MAX_W]` while preserving `CANVAS_MIN_W`.
pub fn clamp_width(w: f32, viewport_w: f32) -> f32 {
    let avail = MIN_W.max(viewport_w - CANVAS_MIN_W);
    MIN_W.max(MAX_W.min(w.round()).min(avail))
}

/// Horizontal resize strip straddling the right edge of the Wide docked Ledger.
pub fn wide_resize_handle<'a>(app: &'a App) -> Element<'a, Message> {
    let lit = app.ui.ledger_handle_hover || app.ui.ledger_resizing;
    let strip = container(Space::new().width(7).height(Length::Fill)).style(move |_| {
        container::Style {
            background: lit.then_some(Background::Color(Color {
                a: 0.35,
                ..tokens::accent()
            })),
            ..container::Style::default()
        }
    });
    let handle = mouse_area(strip)
        .on_press(Message::Ui(UiMsg::LedgerResizeStart))
        .on_double_click(Message::Ui(UiMsg::LedgerResizeReset))
        .on_enter(Message::Ui(UiMsg::LedgerHandleHover(true)))
        .on_exit(Message::Ui(UiMsg::LedgerHandleHover(false)))
        .interaction(mouse::Interaction::ResizingHorizontally);
    let left_pad = (app.ui.effective_ledger_width() - 3.0).max(0.0);
    row![
        Space::new().width(Length::Fixed(left_pad)),
        handle,
        Space::new().width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Main Ledger panel for Wide dock, Medium slide-over, or Compact bottom sheet.
pub fn panel<'a>(
    app: &'a App,
    class: SizeClass,
    d: Density,
    overlay: bool,
) -> Element<'a, Message> {
    if app.workflow.advanced {
        return framed_panel(app, d, overlay, advanced_subview(app, d, overlay));
    }

    let calcs_model = calcs::build(&app.drawing.polygons, &app.computed);
    let view = match class {
        SizeClass::Wide if app.ui.ledger_view == LedgerView::Layers => LedgerView::Drawn,
        _ => app.ui.ledger_view,
    };

    let header = panel_header(app, class, view, d, overlay);
    let main_list: Element<'a, Message> = match view {
        LedgerView::Drawn => drawn_view(app, &calcs_model, d),
        LedgerView::Types => types_view(app, d),
        LedgerView::Layers => layers_view(app, d, false),
    };

    let mut content = column![header, container(main_list).width(Length::Fill).height(Length::Fill)]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .height(Length::Fill);

    if matches!(class, SizeClass::Wide) {
        content = content.push(wide_layers_section(app, d));
    }

    content = content.push(totals_foot(app, &calcs_model, class, d));

    framed_panel(app, d, overlay, content.into())
}

fn framed_panel<'a>(
    app: &'a App,
    _d: Density,
    overlay: bool,
    inner: Element<'a, Message>,
) -> Element<'a, Message> {
    let mut box_el = container(inner)
        .padding(SPACE_3)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(tokens::panel())),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: if overlay { RADIUS_DIALOG } else { 0.0 }.into(),
            },
            shadow: if overlay {
                tokens::floating_shadow()
            } else {
                iced::Shadow::default()
            },
            ..container::Style::default()
        });

    if !overlay {
        box_el = box_el.width(Length::Fixed(app.ui.effective_ledger_width()));
    } else {
        box_el = box_el.width(Length::Fill);
    }

    box_el.into()
}

fn panel_header<'a>(
    app: &'a App,
    class: SizeClass,
    active_view: LedgerView,
    d: Density,
    overlay: bool,
) -> Element<'a, Message> {
    let mut title_row = row![
        text("Areas")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_14)
            .color(tokens::text()),
        Space::new().width(Length::Fill),
        kit::icon_button(
            "dots-three",
            app.ui.ledger_menu,
            Some(Message::Ui(UiMsg::ToggleLedgerMenu)),
            d,
        ),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    if overlay {
        title_row = title_row.push(kit::text_button(
            "Close",
            Some(Message::Ui(UiMsg::CloseOverlays)),
            d,
        ));
    }

    let mut segments = vec![
        kit::Segment {
            label: "Drawn",
            selected: active_view == LedgerView::Drawn,
            focused: false,
            on_press: Some(Message::Ui(UiMsg::SetLedgerView(LedgerView::Drawn))),
        },
        kit::Segment {
            label: "Types",
            selected: active_view == LedgerView::Types,
            focused: false,
            on_press: Some(Message::Ui(UiMsg::SetLedgerView(LedgerView::Types))),
        },
    ];
    if !matches!(class, SizeClass::Wide) {
        segments.push(kit::Segment {
            label: "Layers",
            selected: active_view == LedgerView::Layers,
            focused: false,
            on_press: Some(Message::Ui(UiMsg::SetLedgerView(LedgerView::Layers))),
        });
    }

    column![title_row, kit::segmented(segments, d)]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .into()
}

/// Floating `"..."` overflow menu for the Ledger.
pub fn layer<'a>(app: &'a App) -> Option<Element<'a, Message>> {
    if !app.ui.ledger_menu {
        return None;
    }
    let d = app.ui.density;
    let class = SizeClass::of(app.window_size.width);
    let mut entries = vec![
        MenuEntry::Item {
            icon: Some("arrows-out"),
            label: "Expand all",
            shortcut: None,
            checked: false,
            focused: false,
            on_press: Some(Message::CalcsExpandAll),
        },
        MenuEntry::Item {
            icon: Some("arrows-in"),
            label: "Collapse all",
            shortcut: None,
            checked: false,
            focused: false,
            on_press: Some(Message::CalcsCollapseAll),
        },
        MenuEntry::Separator,
        MenuEntry::Item {
            icon: Some("warning"),
            label: "Find areas that need a type",
            shortcut: None,
            checked: app.workflow.find_undefined,
            focused: false,
            on_press: Some(Message::AreaWorkflow(WorkflowMsg::FindUndefined)),
        },
        MenuEntry::Item {
            icon: Some("bounding-box"),
            label: "Detect regions",
            shortcut: None,
            checked: false,
            focused: false,
            on_press: Some(Message::AreaWorkflow(WorkflowMsg::Detect)),
        },
        MenuEntry::Item {
            icon: Some("sliders"),
            label: "Label defaults & area tools…",
            shortcut: None,
            checked: app.workflow.advanced,
            focused: false,
            on_press: Some(Message::AreaWorkflow(WorkflowMsg::Advanced(true))),
        },
    ];
    if matches!(class, SizeClass::Wide) {
        entries.push(MenuEntry::Separator);
        entries.push(MenuEntry::Item {
            icon: Some("square-split-vertical"),
            label: "Show layers section",
            shortcut: None,
            checked: app.ui.layers_expanded,
            focused: false,
            on_press: Some(Message::Ui(UiMsg::ToggleLayersSection)),
        });
    }

    let card = kit::popover(kit::menu(entries, 240.0, d), RADIUS_DIALOG);
    let top_pad = d.bar_h()
        + if matches!(class, SizeClass::Wide) {
            SPACE_3 * 2.0 + 24.0
        } else {
            crate::ui::layout::PAGE_BAR_H + SPACE_3 * 2.0 + 24.0
        };
    let left_pad = match class {
        SizeClass::Wide => (app.ui.effective_ledger_width() - 252.0).max(SPACE_3),
        SizeClass::Medium | SizeClass::Compact => SPACE_3,
    };

    Some(
        opaque(
            mouse_area(
                container(opaque(card))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding {
                        top: top_pad,
                        left: left_pad,
                        right: SPACE_3,
                        bottom: SPACE_3,
                    })
                    .align_x(alignment::Horizontal::Left)
                    .align_y(alignment::Vertical::Top),
            )
            .on_press(Message::Ui(UiMsg::ToggleLedgerMenu)),
        ),
    )
}

// ---------------------------------------------------------------------------
// Drawn view
// ---------------------------------------------------------------------------

struct DrawnGroup {
    key: String,
    label: String,
    total: f64,
    codes: Vec<String>,
    items: Vec<(calcs::AreaRow, Color)>,
}

fn drawn_view<'a>(app: &'a App, calcs_model: &calcs::Calcs, d: Density) -> Element<'a, Message> {
    let mut rows: Vec<Element<'a, Message>> = Vec::new();

    if app.drawing.pen_down && !app.drawing.points.is_empty() {
        rows.push(drawing_in_progress_row(app));
    }

    let und_polys: Vec<_> = app
        .drawing
        .polygons
        .iter()
        .filter(|p| p.kind == "UND" || p.kind.is_empty())
        .collect();
    if !und_polys.is_empty() {
        let und_color = area_color(&app.area_codes, "UND");
        let und_total: f64 = und_polys.iter().map(|p| p.gross).sum();
        let und_collapsed = app.dock.calcs_collapsed.contains("UND");
        rows.push(group_toggle_row(
            "Needs a type",
            &format!("{} · {:.1} sf", und_polys.len(), und_total),
            und_collapsed,
            true,
            Message::CalcsGroup("UND".to_owned()),
            d,
        ));
        if !und_collapsed {
            for poly in und_polys {
                let title = if poly.name.is_empty() {
                    "Undefined Area"
                } else {
                    poly.name.as_str()
                };
                let area_row = calcs::AreaRow {
                    id: poly.id,
                    name: title.to_owned(),
                    base_area: poly.gross,
                    factor: poly.factor,
                    total: poly.gross * poly.factor,
                    subtracted: app.computed.get(&poly.id).map(|c| c.subtracted),
                    net_area: poly.area,
                    perimeter: app.computed.get(&poly.id).map_or(0.0, |c| c.perimeter),
                };
                rows.push(drawn_area_block(
                    app,
                    &area_row,
                    und_color,
                    true,
                    d,
                ));
            }
        }
    }

    let mut groups: Vec<DrawnGroup> = Vec::new();
    let mut group_index: HashMap<String, usize> = HashMap::new();
    for cg in &calcs_model.groups {
        let meta = app.area_codes.iter().find(|c| c.code == cg.code);
        let group_key = meta
            .map(|c| c.group.clone())
            .filter(|g| !g.is_empty())
            .unwrap_or_else(|| cg.code.clone());
        let label = match palette_group_label(&group_key) {
            "" => group_key.clone(),
            known => known.to_owned(),
        };
        let swatch = meta.map(code_color_rgb).unwrap_or_else(tokens::accent);
        let idx = *group_index.entry(group_key.clone()).or_insert_with(|| {
            groups.push(DrawnGroup {
                key: group_key,
                label,
                total: 0.0,
                codes: Vec::new(),
                items: Vec::new(),
            });
            groups.len() - 1
        });
        groups[idx].total += cg.total;
        groups[idx].codes.push(cg.code.clone());
        for area in &cg.areas {
            groups[idx].items.push((area.clone(), swatch));
        }
    }

    for grp in groups {
        let collapsed = app.dock.calcs_collapsed.contains(&grp.key)
            || (!grp.codes.is_empty()
                && grp
                    .codes
                    .iter()
                    .all(|c| app.dock.calcs_collapsed.contains(c)));
        rows.push(group_toggle_row(
            &grp.label,
            &format!("{:.1} sf", grp.total),
            collapsed,
            false,
            Message::CalcsGroup(grp.key),
            d,
        ));
        if collapsed {
            continue;
        }
        for (area, swatch) in &grp.items {
            rows.push(drawn_area_block(app, area, *swatch, false, d));
        }
    }

    if rows.is_empty() {
        return container(
            column![
                text(calcs_model.empty_note())
                    .font(BARLOW_MEDIUM)
                    .size(TEXT_13)
                    .color(tokens::muted()),
                text("Draw closed walls or drag a type from Types onto an outline.")
                    .font(BARLOW)
                    .size(TEXT_11)
                    .color(tokens::disabled()),
            ]
            .spacing(SPACE_1),
        )
        .padding([SPACE_2, 0.0])
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    make_scrollable(Column::with_children(rows).spacing(4).width(Length::Fill))
        .direction(Direction::Vertical(Scrollbar::new().width(5).scroller_width(5)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn drawing_in_progress_row<'a>(app: &'a App) -> Element<'a, Message> {
    let wall_count = app.drawing.line_ids.len();
    let total_len: f64 = app
        .drawing
        .points
        .windows(2)
        .map(|w| distance(w[0], w[1]))
        .sum();
    let detail = format!(
        "{wall_count} wall{} · {}",
        if wall_count == 1 { "" } else { "s" },
        app.drawing.options.units.length(total_len)
    );
    container(
        row![
            icons::mono("pencil-simple", 14.0, tokens::accent()),
            text("Drawing an area")
                .font(BARLOW_MEDIUM)
                .size(TEXT_12)
                .color(tokens::text())
                .width(Length::Fill),
            text(detail)
                .font(BSC_MEDIUM)
                .size(TEXT_12)
                .color(tokens::accent()),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([6, 8])
    .style(|_| container::Style {
        background: Some(Background::Color(tokens::accent_soft())),
        border: Border {
            color: tokens::accent(),
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn group_toggle_row<'a>(
    title: &str,
    total_text: &str,
    collapsed: bool,
    flagged: bool,
    on_press: Message,
    d: Density,
) -> Element<'a, Message> {
    let fg = if flagged {
        tokens::flag()
    } else {
        tokens::muted()
    };
    let caret = if collapsed {
        "caret-right"
    } else {
        "caret-down"
    };
    button(
        row![
            icons::mono(caret, 12.0, fg),
            text(title.to_owned())
                .font(BSC_SEMIBOLD)
                .size(TEXT_12)
                .color(fg)
                .width(Length::Fill),
            text(total_text.to_owned())
                .font(BSC_SEMIBOLD)
                .size(TEXT_12)
                .color(if flagged { tokens::flag() } else { tokens::text() }),
        ]
        .spacing(SPACE_1)
        .align_y(Alignment::Center)
        .height(Length::Fixed(if matches!(d, Density::Touch) { 32.0 } else { 26.0 }))
        .padding([0, 4]),
    )
    .padding(0)
    .width(Length::Fill)
    .on_press(on_press)
    .style(|_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: hovered.then_some(Background::Color(tokens::hover())),
            border: Border {
                radius: RADIUS_CONTROL.into(),
                ..Border::default()
            },
            ..button::Style::default()
        }
    })
    .into()
}

fn drawn_area_block<'a>(
    app: &'a App,
    area: &calcs::AreaRow,
    swatch: Color,
    flagged: bool,
    d: Density,
) -> Element<'a, Message> {
    let selected = app.selection.polys.contains(&area.id);
    let expanded = app.dock.calcs_expanded.contains(&area.id);
    let value = format!("{:.1} sf", area.net_area);
    let title_color = if flagged { tokens::flag() } else { tokens::text() };
    let value_color = if flagged { tokens::flag() } else { tokens::muted() };
    let caret = if expanded { "caret-down" } else { "caret-right" };

    let row_content = row![
        container(Space::new())
            .width(Length::Fixed(10.0))
            .height(Length::Fixed(10.0))
            .style(move |_| container::Style {
                background: Some(Background::Color(swatch)),
                border: Border {
                    color: tokens::rule(),
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..container::Style::default()
            }),
        text(area.name.clone())
            .font(if selected { BARLOW_SEMIBOLD } else { BARLOW_MEDIUM })
            .size(TEXT_13)
            .color(title_color)
            .width(Length::Fill),
        text(value)
            .font(BSC_MEDIUM)
            .size(TEXT_13)
            .color(value_color),
        icons::mono(caret, 12.0, tokens::muted()),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    let bg = if selected {
        tokens::accent_soft()
    } else {
        Color::TRANSPARENT
    };
    let row_btn = button(
        container(row_content)
            .height(Length::Fixed(d.row_h()))
            .padding([0, 8])
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .padding(0)
    .width(Length::Fill)
    .on_press(Message::CalcsArea(area.id))
    .style(move |_, status| button::Style {
        background: Some(Background::Color(match status {
            button::Status::Hovered => tokens::hover(),
            button::Status::Pressed => tokens::pressed(),
            _ => bg,
        })),
        border: Border {
            color: if selected {
                tokens::accent()
            } else {
                Color::TRANSPARENT
            },
            width: if selected { 1.0 } else { 0.0 },
            radius: RADIUS_CONTROL.into(),
        },
        ..button::Style::default()
    });
    let open_area_msg = Message::AreaWorkflow(WorkflowMsg::Open(area.id));
    let header = kit::long_press(
        mouse_area(row_btn).on_right_press(open_area_msg.clone()),
        open_area_msg,
    );

    if !expanded {
        return header;
    }

    let sub = area.subtracted.unwrap_or(0.0);
    let sub_text = if sub > 0.0 {
        format!("-{sub:.1} sf")
    } else {
        "0.0 sf".to_owned()
    };
    // One decimal for areas and lengths everywhere (rows, inspector, plan
    // labels, totals); only the factor, a multiplier, keeps two.
    let details = container(
        column![
            detail_line("Base area", format!("{:.1} sf", area.base_area), false),
            detail_line("Factor", format!("{:.2}", area.factor), false),
            detail_line("Total", format!("{:.1} sf", area.total), false),
            detail_line("Subtracted", sub_text, sub > 0.0),
            detail_line("Net area", format!("{:.1} sf", area.net_area), false),
            detail_line("Perimeter", app.drawing.options.units.length(area.perimeter), false),
        ]
        .spacing(2),
    )
    .width(Length::Fill)
    .padding(Padding {
        top: 6.0,
        right: 10.0,
        bottom: 6.0,
        left: 20.0,
    })
    .style(|_| container::Style {
        background: Some(Background::Color(tokens::field())),
        border: Border {
            color: tokens::rule(),
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        ..container::Style::default()
    });

    column![header, details]
        .spacing(SPACE_1)
        .width(Length::Fill)
        .into()
}

fn detail_line<'a>(label: &'static str, value: String, alarm: bool) -> Element<'a, Message> {
    row![
        text(label)
            .font(BARLOW)
            .size(TEXT_11)
            .color(tokens::muted())
            .width(Length::Fill),
        text(value)
            .font(BSC_MEDIUM)
            .size(TEXT_12)
            .color(if alarm { tokens::flag() } else { tokens::text() }),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

// ---------------------------------------------------------------------------
// Types view
// ---------------------------------------------------------------------------

fn types_view<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let search = kit::field(
        kit::FieldProps {
            label: "",
            value: &app.ui.type_filter,
            placeholder: "Search area types",
            on_input: Some(Box::new(|s| Message::Ui(UiMsg::TypeFilterChanged(s)))),
            on_submit: None,
            focused: false,
        },
        d,
    );

    let needle = app.ui.type_filter.trim().to_lowercase();
    let model = dock::paint_palette(&app.area_codes);
    let mut list = column![].spacing(4).width(Length::Fill);

    for grp in &model.groups {
        let matching: Vec<_> = grp
            .tiles
            .iter()
            .filter(|(name, code, _)| {
                needle.is_empty()
                    || name.to_lowercase().contains(&needle)
                    || code.to_lowercase().contains(&needle)
                    || grp.label.to_lowercase().contains(&needle)
            })
            .collect();
        if matching.is_empty() {
            continue;
        }
        let collapsed = needle.is_empty() && app.workflow.collapsed.contains(&grp.group);
        list = list.push(group_toggle_row(
            &grp.label,
            &format!("{}", matching.len()),
            collapsed,
            false,
            Message::AreaWorkflow(WorkflowMsg::Group(grp.group.clone())),
            d,
        ));
        if collapsed {
            continue;
        }
        for (name, code, color_u32) in matching {
            list = list.push(type_row(app, name, code, *color_u32, false, d));
        }
    }

    if let Some((clear_name, clear_code, clear_color)) = &model.clear {
        if needle.is_empty()
            || clear_name.to_lowercase().contains(&needle)
            || clear_code.to_lowercase().contains(&needle)
            || "undefined".contains(&needle)
        {
            list = list.push(kit::group_header("Undefined", None));
            list = list.push(type_row(app, clear_name, clear_code, *clear_color, true, d));
        }
    }

    let scroll = make_scrollable(list)
        .direction(Direction::Vertical(Scrollbar::new().width(5).scroller_width(5)))
        .width(Length::Fill)
        .height(Length::Fill);

    let footer = row![
        kit::text_button("Edit types", Some(Message::OpenCodesEditor), d),
        kit::text_button(
            "New type",
            Some(Message::AreaWorkflow(WorkflowMsg::NewType)),
            d,
        ),
        kit::text_button(
            "Find undefined",
            Some(Message::AreaWorkflow(WorkflowMsg::FindUndefined)),
            d,
        ),
    ]
    .spacing(SPACE_1)
    .wrap();

    let mut col = column![search].spacing(SPACE_2).width(Length::Fill).height(Length::Fill);
    if !app.workflow.status.is_empty() {
        col = col.push(
            container(
                text(app.workflow.status.clone())
                    .font(BARLOW)
                    .size(TEXT_11)
                    .color(tokens::accent()),
            )
            .width(Length::Fill)
            .padding([4, 8])
            .style(|_| container::Style {
                background: Some(Background::Color(tokens::accent_soft())),
                border: Border {
                    radius: RADIUS_CONTROL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }),
        );
    }
    col = col.push(scroll).push(footer);
    col.into()
}

fn type_row<'a>(
    app: &'a App,
    name: &str,
    code: &str,
    color_u32: u32,
    is_clear: bool,
    d: Density,
) -> Element<'a, Message> {
    let used = app.workflow.usage.get(code).copied().unwrap_or(0);
    let armed = app.workflow.armed.as_deref() == Some(code);
    let swatch = hex_u32(color_u32);
    let drag_item = DragItem::Area {
        code: code.to_owned(),
        name: (!is_clear).then(|| name.to_owned()),
    };

    let row_h = d.row_h();
    let mut main_content = row![
        container(Space::new())
            .width(Length::Fixed(10.0))
            .height(Length::Fixed(10.0))
            .style(move |_| container::Style {
                background: Some(Background::Color(swatch)),
                border: Border {
                    color: tokens::rule(),
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..container::Style::default()
            }),
        text(name.to_owned())
            .font(if armed { BARLOW_SEMIBOLD } else { BARLOW_MEDIUM })
            .size(TEXT_13)
            .color(if is_clear { tokens::flag() } else { tokens::text() })
            .width(Length::Fill),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    if used > 0 && !is_clear {
        main_content = main_content.push(
            container(
                text(used.to_string())
                    .font(BSC_MEDIUM)
                    .size(TEXT_11)
                    .color(tokens::accent()),
            )
            .padding([1, 6])
            .style(|_| container::Style {
                background: Some(Background::Color(tokens::accent_soft())),
                border: Border {
                    radius: 999.0.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }),
        );
    }

    main_content = main_content.push(
        text(code.to_owned())
            .font(BSC_MEDIUM)
            .size(TEXT_11)
            .color(tokens::muted()),
    );

    let type_menu_msg = Message::AreaWorkflow(WorkflowMsg::TypeMenu(code.to_owned()));
    let interactive = kit::long_press(
        mouse_area(
            container(main_content)
                .height(Length::Fixed(row_h))
                .padding([0, 8])
                .align_y(Alignment::Center)
                .width(Length::Fill),
        )
        .on_press(Message::DockDragPress(drag_item.clone()))
        .on_release(Message::DockItemRelease(drag_item))
        .on_right_press(type_menu_msg.clone()),
        type_menu_msg,
    );

    let bg = if armed {
        tokens::accent_soft()
    } else {
        Color::TRANSPARENT
    };
    let tile_btn = button(interactive)
        .padding(0)
        .width(Length::Fill)
        .on_press(Message::Noop)
        .style(move |_, status| button::Style {
            background: Some(Background::Color(match status {
                button::Status::Hovered => tokens::hover(),
                button::Status::Pressed => tokens::pressed(),
                _ => bg,
            })),
            border: Border {
                color: if armed { tokens::accent() } else { Color::TRANSPARENT },
                width: if armed { 1.0 } else { 0.0 },
                radius: RADIUS_CONTROL.into(),
            },
            ..button::Style::default()
        });

    let menu_btn = kit::icon_button(
        "dots-three",
        app.workflow.menu.as_deref() == Some(code),
        Some(Message::AreaWorkflow(WorkflowMsg::TypeMenu(code.to_owned()))),
        d,
    );

    row![tile_btn, menu_btn]
        .spacing(2)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}

// ---------------------------------------------------------------------------
// Layers section / tab
// ---------------------------------------------------------------------------

fn wide_layers_section<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let filter = app.area_layer_filter();
    let count = app.layers.visible_rows(|l| filter.keeps(l)).len();
    let caret = if app.ui.layers_expanded {
        "caret-down"
    } else {
        "caret-right"
    };
    let toggle = button(
        row![
            icons::mono(caret, 12.0, tokens::muted()),
            text(format!("LAYERS ({count})"))
                .font(BSC_SEMIBOLD)
                .size(TEXT_11)
                .color(tokens::muted())
                .width(Length::Fill),
        ]
        .spacing(SPACE_1)
        .align_y(Alignment::Center)
        .padding([4, 2]),
    )
    .padding(0)
    .width(Length::Fill)
    .on_press(Message::Ui(UiMsg::ToggleLayersSection))
    .style(|_, status| button::Style {
        background: matches!(status, button::Status::Hovered | button::Status::Pressed)
            .then_some(Background::Color(tokens::hover())),
        border: Border {
            radius: RADIUS_CONTROL.into(),
            ..Border::default()
        },
        ..button::Style::default()
    });

    if !app.ui.layers_expanded {
        return column![kit::divider(), toggle]
            .spacing(SPACE_1)
            .width(Length::Fill)
            .into();
    }

    column![kit::divider(), toggle, layers_view(app, d, true)]
        .spacing(SPACE_1)
        .width(Length::Fill)
        .into()
}

fn layers_view<'a>(app: &'a App, d: Density, capped: bool) -> Element<'a, Message> {
    let filter = app.area_layer_filter();
    let rows = app.layers.visible_rows(|l| filter.keeps(l));
    let body: Element<'a, Message> = if rows.is_empty() {
        container(
            text("Closed areas create layers automatically.")
                .font(BARLOW)
                .size(TEXT_12)
                .color(tokens::muted()),
        )
        .padding([6, 4])
        .width(Length::Fill)
        .into()
    } else {
        let mut list = Column::new().spacing(2).width(Length::Fill);
        let mut active_menu: Option<LayerRow> = None;
        for r in &rows {
            if app.dock.layer_menu.as_deref() == Some(r.id.as_str()) {
                active_menu = Some(r.clone());
            }
            list = list.push(next_layer_row(app, r, d));
        }
        let mut sc = make_scrollable(list)
            .direction(Direction::Vertical(Scrollbar::new().width(5).scroller_width(5)))
            .width(Length::Fill);
        sc = if capped {
            sc.height(Length::Fixed(140.0))
        } else {
            sc.height(Length::Fill)
        };
        if let Some(menu_row) = active_menu {
            stack![sc, inline_layer_menu(&menu_row, d)]
                .width(Length::Fill)
                .into()
        } else {
            sc.into()
        }
    };

    body
}

fn next_layer_row<'a>(app: &'a App, r: &LayerRow, d: Density) -> Element<'a, Message> {
    let id = r.id.as_str();
    let dragging = app.dock.layer_drag.as_deref() == Some(id);
    let drop_mark: Option<DragPos> = if dragging {
        None
    } else {
        app.dock
            .layer_over
            .as_ref()
            .filter(|(target, _)| target != id && app.dock.layer_drag.is_some())
            .map(|(_, p)| *p)
    };

    let hovered = app.dock.layer_hover.as_deref() == Some(id);
    let renaming = app
        .dock
        .layer_rename
        .as_ref()
        .filter(|(target, _)| target == id)
        .map(|(_, buf)| buf.clone());

    let row_h: f32 = if matches!(d, Density::Touch) { 38.0 } else { 30.0 };
    let mid_h = (row_h - 12.0).max(16.0);
    let fg = if dragging {
        tokens::disabled()
    } else if !r.visible || r.locked {
        tokens::muted()
    } else {
        tokens::text()
    };

    let head: Element<'a, Message> = if r.is_group {
        let caret = if r.collapsed {
            "caret-right"
        } else {
            "caret-down"
        };
        kit::icon_button(
            caret,
            false,
            Some(Message::LayerToggleCollapsed(id.to_owned())),
            d,
        )
    } else {
        Space::new().width(Length::Fixed(16.0)).into()
    };

    let kind_icon = icons::mono(
        if r.is_group { "folder-simple" } else { "shapes" },
        14.0,
        if r.is_group {
            tokens::accent()
        } else {
            tokens::muted()
        },
    );

    let mid_content: Element<'a, Message> = if let Some(buf) = renaming {
        let id_owned = id.to_owned();
        container(
            text_input("", buf.as_str())
                .id(dock::RENAME_ID)
                .size(TEXT_12)
                .font(BARLOW)
                .padding([2, 6])
                .width(Length::Fill)
                .on_input(move |s| Message::LayerRename(id_owned.clone(), s))
                .on_submit(Message::LayerRenameCommit),
        )
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .into()
    } else {
        container(
            text(r.name.clone())
                .font(if r.active { BARLOW_SEMIBOLD } else { BARLOW_MEDIUM })
                .size(TEXT_12)
                .color(fg),
        )
        .width(Length::Fill)
        .height(Length::Fixed(mid_h))
        .align_y(Alignment::Center)
        .padding(Padding {
            left: r.depth as f32 * 12.0,
            ..Padding::ZERO
        })
        .into()
    };

    let zone = |h: f32, pos: DragPos, content: Element<'a, Message>| -> Element<'a, Message> {
        mouse_area(container(content).width(Length::Fill).height(Length::Fixed(h)))
            .on_press(Message::LayerPress(id.to_owned()))
            .on_release(Message::LayerDrop(id.to_owned(), pos))
            .on_enter(Message::LayerZoneHover(id.to_owned(), pos))
            .into()
    };
    let strip = column![
        zone(6.0, DragPos::Before, Space::new().into()),
        zone(mid_h, DragPos::Inside, mid_content),
        zone(6.0, DragPos::After, Space::new().into()),
    ]
    .width(Length::Fill);

    let tools = row![
        kit::icon_button(
            if r.visible { "eye" } else { "eye-slash" },
            !r.visible,
            Some(Message::LayerToggleVisible(id.to_owned())),
            d,
        ),
        kit::icon_button(
            if r.locked { "lock-simple" } else { "lock-simple-open" },
            r.locked,
            Some(Message::LayerToggleLocked(id.to_owned())),
            d,
        ),
        kit::icon_button(
            "dots-three",
            app.dock.layer_menu.as_deref() == Some(id),
            Some(Message::LayerRowMenu(id.to_owned())),
            d,
        ),
    ]
    .spacing(2)
    .align_y(Alignment::Center);

    let row_body = row![head, kind_icon, strip, tools]
        .spacing(4)
        .padding([0, 4])
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fixed(row_h));

    let inside = drop_mark == Some(DragPos::Inside);
    let active_row = r.active;
    let frame = container(row_body)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(if active_row {
                tokens::accent_soft()
            } else if hovered && !dragging {
                tokens::hover()
            } else {
                Color::TRANSPARENT
            })),
            border: Border {
                color: if active_row || inside {
                    tokens::accent()
                } else {
                    Color::TRANSPARENT
                },
                width: if active_row || inside { 1.0 } else { 0.0 },
                radius: RADIUS_CONTROL.into(),
            },
            ..container::Style::default()
        });

    let marked: Element<'a, Message> = match drop_mark {
        Some(DragPos::Before) => column![drop_rule(), frame].width(Length::Fill).into(),
        Some(DragPos::After) => column![frame, drop_rule()].width(Length::Fill).into(),
        _ => frame.into(),
    };

    let layer_menu_msg = Message::LayerRowMenu(id.to_owned());
    kit::long_press(
        mouse_area(marked)
            .on_enter(Message::LayerHoverRow(id.to_owned()))
            .on_exit(Message::LayerHoverRowGone(id.to_owned()))
            .on_right_press(layer_menu_msg.clone()),
        layer_menu_msg,
    )
}

fn drop_rule<'a>() -> Element<'a, Message> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(2.0))
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::accent())),
            ..container::Style::default()
        })
        .into()
}

fn inline_layer_menu<'a>(r: &LayerRow, d: Density) -> Element<'a, Message> {
    let id = r.id.clone();
    let entries = vec![
        MenuEntry::Item {
            icon: Some("text-t"),
            label: "Rename",
            shortcut: None,
            checked: false,
            focused: false,
            on_press: Some(Message::LayerStartRename(id.clone())),
        },
        MenuEntry::Item {
            icon: Some(if r.visible { "eye-slash" } else { "eye" }),
            label: if r.visible { "Hide" } else { "Show" },
            shortcut: None,
            checked: false,
            focused: false,
            on_press: Some(Message::LayerToggleVisible(id.clone())),
        },
        MenuEntry::Item {
            icon: Some(if r.locked {
                "lock-simple-open"
            } else {
                "lock-simple"
            }),
            label: if r.locked { "Unlock" } else { "Lock" },
            shortcut: None,
            checked: false,
            focused: false,
            on_press: Some(Message::LayerToggleLocked(id.clone())),
        },
        MenuEntry::Separator,
        MenuEntry::Item {
            icon: Some("export"),
            label: "Export",
            shortcut: None,
            checked: false,
            focused: false,
            on_press: Some(Message::LayerExport(id)),
        },
        MenuEntry::Item {
            icon: Some("x"),
            label: "Close menu",
            shortcut: None,
            checked: false,
            focused: false,
            on_press: Some(Message::LayerMenuClose),
        },
    ];
    container(opaque(kit::popover(kit::menu(entries, 172.0, d), RADIUS_DIALOG)))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([4, 8])
        .align_x(alignment::Horizontal::Right)
        .align_y(alignment::Vertical::Top)
        .into()
}

// ---------------------------------------------------------------------------
// Advanced / Label defaults subview (`workflow.advanced`)
// ---------------------------------------------------------------------------

fn advanced_subview<'a>(app: &'a App, d: Density, overlay: bool) -> Element<'a, Message> {
    let mut header = row![
        kit::icon_button(
            "arrow-left",
            false,
            Some(Message::AreaWorkflow(WorkflowMsg::Advanced(false))),
            d,
        ),
        text("Label defaults & area tools")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_14)
            .color(tokens::text())
            .width(Length::Fill),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    if overlay {
        header = header.push(kit::text_button(
            "Close",
            Some(Message::Ui(UiMsg::CloseOverlays)),
            d,
        ));
    }

    let mut post = app.drawing.auto_post.clone();
    post.dims = app.view.dimensions;
    let placement_in = app.view.dimension_placement == DimensionPlacement::In;

    let body = column![
        kit::group_header("AREA OPERATIONS", None),
        row![
            kit::text_button("Redefine", Some(Message::AreaWorkflow(WorkflowMsg::Redefine)), d),
            kit::text_button("Clone", Some(Message::AreaWorkflow(WorkflowMsg::Clone)), d),
            kit::text_button("Reopen", Some(Message::AreaWorkflow(WorkflowMsg::Reopen)), d),
            kit::text_button("Detect", Some(Message::AreaWorkflow(WorkflowMsg::Detect)), d),
        ]
        .spacing(SPACE_1),
        kit::divider(),
        kit::group_header("LABEL DEFAULTS", None),
        area_workflow::post_controls(&post, true),
        kit::segmented(
            vec![
                kit::Segment {
                    label: "Dims Out",
                    selected: !placement_in,
                    focused: false,
                    on_press: Some(Message::AreaWorkflow(WorkflowMsg::DefaultPlacement(
                        "Out".to_owned(),
                    ))),
                },
                kit::Segment {
                    label: "Dims In",
                    selected: placement_in,
                    focused: false,
                    on_press: Some(Message::AreaWorkflow(WorkflowMsg::DefaultPlacement(
                        "In".to_owned(),
                    ))),
                },
            ],
            d,
        ),
    ]
    .spacing(SPACE_2)
    .width(Length::Fill);

    column![
        header,
        make_scrollable(body)
            .direction(Direction::Vertical(Scrollbar::new().width(5).scroller_width(5)))
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .spacing(SPACE_2)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// ---------------------------------------------------------------------------
// Totals foot
// ---------------------------------------------------------------------------

fn totals_foot<'a>(
    app: &'a App,
    calcs_model: &calcs::Calcs,
    class: SizeClass,
    d: Density,
) -> Element<'a, Message> {
    let totals = app.calcs_totals.unwrap_or_default();
    let non_living = app
        .last_recalc
        .as_ref()
        .map(|r| (r.totals.total_all_areas - r.totals.total_gla).max(0.0))
        .unwrap_or(0.0);

    if matches!(class, SizeClass::Compact) {
        return column![
            kit::divider(),
            row![
                column![
                    text("Gross living area")
                        .font(BARLOW_MEDIUM)
                        .size(TEXT_11)
                        .color(tokens::muted()),
                    text(format!("{:.1} sf", totals.total_gla))
                        .font(BSC_SEMIBOLD)
                        .size(NUM_24)
                        .color(tokens::text()),
                ]
                .width(Length::Fill),
                kit::text_button("Show the math", Some(Message::OpenBreakdown), d),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center)
            .width(Length::Fill),
        ]
        .spacing(SPACE_1)
        .width(Length::Fill)
        .into();
    }

    let mut stats = column![
        detail_line("Building area", format!("{:.1} sf", totals.gba), false),
        detail_line("Not counted", format!("{:.1} sf", non_living), false),
    ]
    .spacing(2)
    .width(Length::Fill);
    if calcs_model.und_count > 0 {
        stats = stats.push(detail_line(
            "Needs a type",
            format!("{}", calcs_model.und_count),
            true,
        ));
    }

    column![
        kit::divider(),
        text("Gross living area")
            .font(BARLOW_MEDIUM)
            .size(TEXT_11)
            .color(tokens::muted()),
        text(format!("{:.1} sf", totals.total_gla))
            .font(BSC_SEMIBOLD)
            .size(NUM_32)
            .color(tokens::text()),
        stats,
        button(
            container(
                text("Show the math")
                    .font(BARLOW_SEMIBOLD)
                    .size(TEXT_12)
                    .color(tokens::link()),
            )
            .padding([4, 0]),
        )
        .padding(0)
        .on_press(Message::OpenBreakdown)
        .style(|_, _| button::Style::default()),
    ]
    .spacing(SPACE_1)
    .width(Length::Fill)
    .into()
}

// ---------------------------------------------------------------------------
// Colour helpers
// ---------------------------------------------------------------------------

fn hex_u32(rgb: u32) -> Color {
    Color::from_rgb8(
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
    )
}

fn code_color_rgb(entry: &AreaCode) -> Color {
    let parse = |s: &str| u32::from_str_radix(s.trim().trim_start_matches('#'), 16).ok();
    hex_u32(
        parse(&entry.color)
            .or_else(|| parse(&entry.fill_color))
            .unwrap_or(0x85_85_85),
    )
}

fn area_color(codes: &[AreaCode], code: &str) -> Color {
    codes
        .iter()
        .find(|c| c.code == code)
        .map(code_color_rgb)
        .unwrap_or_else(|| {
            if code == "UND" {
                tokens::flag()
            } else {
                tokens::accent()
            }
        })
}
