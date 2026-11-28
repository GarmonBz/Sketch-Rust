//! The Menu (`docs/ui-redesign/01-design-spec.md` §5.9, Option A of the
//! 2026-11-17 menu redesign).
//!
//! One column: a search field, then 19 rows in four sections (the file, this
//! sketch, editing, the app). Rows that hold a choice of several values open a
//! fly-out to the side: Recent, Import, Export, Print, Trace image, Select,
//! Transform sketch and Help. Ctrl K, the top bar's search field and its
//! magnifier open the Menu with the search field focused. Typing in the search field lists every command
//! in the registry (`commands::matching`), so a command that has its home
//! elsewhere (the tray, drawing controls, inspector, page tabs, Options) is
//! still one search away.
//!
//! Layout adapts by size class:
//! - `Wide` / `Medium`: a drop-down under the top bar's Menu button, with the
//!   fly-out beside the row that opened it.
//! - `Compact`: a full-height sheet; a fly-out replaces the list with a Back row.

use iced::widget::{
    button, column, container, mouse_area, opaque, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding};

use crate::icons;
use crate::ui::commands::{self, Command};
use crate::ui::fonts::{BARLOW, BARLOW_MEDIUM, BARLOW_SEMIBOLD, BSC_MEDIUM};
use crate::ui::kit;
use crate::ui::layout::SizeClass;
use crate::ui::tokens::{
    self, Density, RADIUS_CONTROL, RADIUS_DIALOG, SPACE_1, SPACE_2, SPACE_3, TEXT_11, TEXT_12,
    TEXT_13, TEXT_16,
};
use crate::ui::Msg as UiMsg;
use crate::{App, Message};

/// Widget id of the Menu's search field (focused when the Menu opens).
pub const SEARCH_ID: &str = "menu-search";

/// A fly-out of the Menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSub {
    Recent,
    Import,
    Export,
    Print,
    TraceImage,
    Select,
    Transform,
    Help,
}

impl MenuSub {
    pub fn title(self) -> &'static str {
        match self {
            MenuSub::Recent => "Recent",
            MenuSub::Import => "Import",
            MenuSub::Export => "Export",
            MenuSub::Print => "Print",
            MenuSub::TraceImage => "Trace image",
            MenuSub::Select => "Select",
            MenuSub::Transform => "Transform sketch",
            MenuSub::Help => "Help",
        }
    }

    /// The row's icon, in the same family as its commands.
    pub fn icon(self) -> (&'static str, bool) {
        match self {
            MenuSub::Recent => ("clock-counter-clockwise", true),
            MenuSub::Import => ("arrow-square-in", true),
            MenuSub::Export => ("arrow-square-out", true),
            MenuSub::Print => ("printer", true),
            MenuSub::TraceImage => ("image", false),
            MenuSub::Select => ("selection-all", false),
            MenuSub::Transform => ("arrows-left-right", false),
            MenuSub::Help => ("info", false),
        }
    }

    /// The fly-out's commands as `(command id, label shown here)`.
    fn commands(self) -> &'static [(&'static str, &'static str)] {
        match self {
            MenuSub::Recent => &[
                ("start-screen", "Show start screen"),
                ("recover-autosave", "Recover autosave…"),
            ],
            MenuSub::Import => &[
                ("import", "Sketch file…"),
                ("import-cad", "CAD drawing (DXF)…"),
                ("import-axf", "Apex AXF sketch…"),
                ("import-acisketch", "ACI Sketch…"),
                ("import-rapidsketch", "RapidSketch…"),
                ("import-layer", "Layer from another sketch…"),
            ],
            MenuSub::Export => &[
                ("export-pdf", "PDF"),
                ("export-png", "PNG image"),
                ("export-svg", "SVG drawing"),
                ("export-dxf", "DXF for CAD"),
                ("export-axf", "Apex AXF"),
                ("export-acisketch", "ACI Sketch"),
                ("export-rapidsketch", "RapidSketch"),
            ],
            MenuSub::Print => &[
                ("print", "Print…"),
                ("print-preview", "Print preview"),
                ("page-layout", "Page layout…"),
            ],
            MenuSub::TraceImage => &[
                ("photo-load", "Load trace image…"),
                ("photo-show", "Show trace image"),
                ("photo-hide", "Hide trace image"),
                ("photo-adjust", "Edit image"),
                ("photo-remove", "Remove trace image"),
                ("geo-load-map", "Load map area"),
                ("geo-address", "Search address…"),
                ("geo-coordinate", "Jump to coordinates…"),
            ],
            MenuSub::Select => &[
                ("select-all", "Everything"),
                ("select-areas", "Areas"),
                ("select-walls", "Freeform lines"),
                ("select-text", "Text labels"),
                ("select-dimensions", "Dimensions"),
                ("select-symbols", "Symbols"),
            ],
            MenuSub::Transform => &[
                ("flip-horizontal", "Flip horizontally"),
                ("flip-vertical", "Flip vertically"),
                ("rotate-sketch", "Rotate +90°"),
                ("rotate-sketch-ccw", "Rotate −90°"),
            ],
            MenuSub::Help => &[
                ("user-guide", "User guide"),
                ("shortcuts", "Keyboard shortcuts"),
                ("appraise-it-reports", "Appraise-It guide"),
                ("email", "Email support"),
                ("about", "About Sketch Rust"),
            ],
        }
    }
}

/// One line of the Menu's main list.
#[derive(Debug, Clone, Copy)]
pub enum Entry {
    /// A command by id, with the label the Menu shows for it.
    Cmd(&'static str, &'static str),
    Sub(MenuSub),
    Sep,
}

/// The main list: about the file, about this sketch, editing, about the app.
pub const ENTRIES: &[Entry] = &[
    Entry::Cmd("new", "New sketch"),
    Entry::Cmd("open", "Open…"),
    Entry::Sub(MenuSub::Recent),
    Entry::Cmd("save", "Save"),
    Entry::Cmd("save-as", "Save as…"),
    Entry::Sub(MenuSub::Import),
    Entry::Sub(MenuSub::Export),
    Entry::Sub(MenuSub::Print),
    Entry::Sep,
    Entry::Cmd("subject-info", "Property details"),
    Entry::Cmd("area-code-settings", "Area types…"),
    Entry::Cmd("area-breakdown", "Area breakdown"),
    Entry::Sub(MenuSub::TraceImage),
    Entry::Cmd("symbol-library", "Symbol library"),
    Entry::Sep,
    Entry::Cmd("undo-history", "Undo history"),
    Entry::Cmd("version-history", "Version history…"),
    Entry::Sub(MenuSub::Select),
    Entry::Sub(MenuSub::Transform),
    Entry::Sep,
    Entry::Cmd("options", "Options…"),
    Entry::Sub(MenuSub::Help),
    Entry::Cmd("close-window", "Close window"),
];

/// The main list without separators: what Up and Down move through.
pub fn rows() -> impl Iterator<Item = Entry> {
    ENTRIES.iter().copied().filter(|e| !matches!(e, Entry::Sep))
}

/// A row of a fly-out or of the search results.
pub struct SubAction {
    pub label: String,
    pub detail: &'static str,
    pub icon: Option<(&'static str, bool)>,
    pub shortcut: &'static str,
    pub checked: bool,
    pub enabled: bool,
    pub on_press: Option<Message>,
}

fn cmd_action(app: &App, cmd: &'static Command, label: &str, detail: &'static str) -> SubAction {
    let enabled = (cmd.enabled)(app);
    SubAction {
        label: label.to_owned(),
        detail,
        icon: commands::icon_for(cmd.id),
        shortcut: cmd.shortcut,
        checked: cmd.checked.is_some_and(|f| f(app)),
        enabled,
        on_press: enabled.then_some(Message::Ui(UiMsg::MenuRunId(cmd.id))),
    }
}

/// Fly-out rows for `sub` against the current `app` state.
pub fn sub_actions(app: &App, sub: MenuSub) -> Vec<SubAction> {
    let mut out = Vec::new();
    if sub == MenuSub::Recent {
        for (i, rec) in app.recent.iter().enumerate() {
            out.push(SubAction {
                label: rec.name.clone(),
                detail: "",
                icon: Some(("file", false)),
                shortcut: "",
                checked: false,
                enabled: true,
                on_press: Some(Message::Ui(UiMsg::MenuRunRecent(i))),
            });
        }
        if app.recent.is_empty() {
            out.push(SubAction {
                label: "No recent sketches".to_owned(),
                detail: "",
                icon: None,
                shortcut: "",
                checked: false,
                enabled: false,
                on_press: None,
            });
        }
    }
    for &(id, label) in sub.commands() {
        if let Some(cmd) = commands::find(id) {
            out.push(cmd_action(app, cmd, label, ""));
        }
    }
    if sub == MenuSub::Recent && !app.recent.is_empty() {
        out.push(SubAction {
            label: "Clear recent sketches".to_owned(),
            detail: "",
            icon: Some(("trash", false)),
            shortcut: "",
            checked: false,
            enabled: true,
            on_press: Some(Message::Ui(UiMsg::MenuClearRecent)),
        });
    }
    out
}

/// Every registered command matching the search text, labelled as the
/// registry labels it, with its group as a hint.
pub fn search_actions(app: &App) -> Vec<SubAction> {
    commands::matching(&app.ui.menu_query)
        .into_iter()
        .map(|cmd| cmd_action(app, cmd, cmd.label, cmd.group.label()))
        .collect()
}

fn searching(app: &App) -> bool {
    !app.ui.menu_query.trim().is_empty()
}

/// Moves the keyboard row by `d`, wrapping around `count` rows.
pub fn step(active: usize, count: usize, d: i32) -> usize {
    if count == 0 {
        return active;
    }
    ((active as i64 + d as i64).rem_euclid(count as i64)) as usize
}

/// Number of rows Up and Down move through right now.
pub fn active_row_count(app: &App) -> usize {
    if searching(app) {
        search_actions(app).len()
    } else if let Some(sub) = app.ui.menu_sub {
        sub_actions(app, sub).len()
    } else {
        rows().count()
    }
}

/// The message Enter sends for the focused row, if any.
pub fn enter_message(app: &App) -> Option<Message> {
    let row = app.ui.menu_row;
    if searching(app) {
        return search_actions(app)
            .into_iter()
            .nth(row)
            .and_then(|a| a.on_press);
    }
    if let Some(sub) = app.ui.menu_sub {
        return sub_actions(app, sub)
            .into_iter()
            .nth(row)
            .and_then(|a| a.on_press);
    }
    match rows().nth(row)? {
        Entry::Sub(sub) => Some(Message::Ui(UiMsg::MenuSub(Some(sub)))),
        Entry::Cmd(id, _) => {
            let cmd = commands::find(id)?;
            (cmd.enabled)(app).then_some(Message::Ui(UiMsg::MenuRunId(id)))
        }
        Entry::Sep => None,
    }
}

/// Renders the Menu overlay when `app.ui.menu_open` is true.
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

/// Row height in the Menu: a little tighter than list rows with a mouse so
/// the whole Menu fits a 760 px tall window; full touch height otherwise.
fn row_h(d: Density) -> f32 {
    match d {
        Density::Touch => d.row_h(),
        Density::Comfortable => d.row_h().min(32.0),
    }
}

const SEP_H: f32 = 9.0;
const MAIN_W: f32 = 300.0;
const SUB_W: f32 = 270.0;
const SEARCH_W: f32 = 420.0;

fn search_field<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let input = text_input("Search all commands", &app.ui.menu_query)
        .id(SEARCH_ID)
        .font(BARLOW)
        .size(TEXT_13)
        .padding([((d.field_h() - 18.0).max(4.0)) * 0.5, SPACE_1])
        .width(Length::Fill)
        .on_input(|q| Message::Ui(UiMsg::MenuQuery(q)))
        .style(|_, _| iced::widget::text_input::Style {
            background: Background::Color(Color::TRANSPARENT),
            border: Border::default(),
            icon: tokens::muted(),
            placeholder: tokens::muted(),
            value: tokens::text(),
            selection: tokens::accent_soft(),
        });
    container(
        row![
            icons::mono("magnifying-glass", 15.0, tokens::muted()),
            input,
        ]
        .spacing(SPACE_1)
        .align_y(Alignment::Center),
    )
    .padding([0.0, SPACE_2])
    .height(Length::Fixed(d.field_h()))
    .align_y(Alignment::Center)
    .style(|_| container::Style {
        background: Some(Background::Color(tokens::input_bg())),
        border: Border {
            color: tokens::accent(),
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn panel_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(tokens::bar())),
        border: Border {
            color: tokens::rule(),
            width: 1.0,
            radius: RADIUS_DIALOG.into(),
        },
        shadow: tokens::floating_shadow(),
        ..container::Style::default()
    }
}

/// The main list as rows, with hover opening fly-outs on a mouse.
fn main_list<'a>(app: &'a App, d: Density, hover_opens: bool) -> Element<'a, Message> {
    let mut col = column![].width(Length::Fill);
    let mut idx = 0usize;
    let keyboard_on_main = app.ui.menu_sub.is_none();
    for entry in ENTRIES {
        match *entry {
            Entry::Sep => {
                col = col.push(
                    container(kit::divider())
                        .height(Length::Fixed(SEP_H))
                        .align_y(Alignment::Center),
                );
            }
            Entry::Cmd(id, label) => {
                let focused = keyboard_on_main && app.ui.menu_row == idx;
                if let Some(cmd) = commands::find(id) {
                    let action = cmd_action(app, cmd, label, "");
                    let btn = action_row(action, d, false, focused, false);
                    col = col.push(if hover_opens {
                        mouse_area(btn)
                            .on_enter(Message::Ui(UiMsg::MenuSub(None)))
                            .into()
                    } else {
                        btn
                    });
                }
                idx += 1;
            }
            Entry::Sub(sub) => {
                let open = app.ui.menu_sub == Some(sub);
                let focused = keyboard_on_main && app.ui.menu_row == idx;
                let action = SubAction {
                    label: sub.title().to_owned(),
                    detail: "",
                    icon: Some(sub.icon()),
                    shortcut: "",
                    checked: false,
                    enabled: true,
                    on_press: Some(Message::Ui(UiMsg::MenuSub(Some(sub)))),
                };
                let btn = action_row(action, d, open, focused, true);
                col = col.push(if hover_opens {
                    mouse_area(btn)
                        .on_enter(Message::Ui(UiMsg::MenuSub(Some(sub))))
                        .into()
                } else {
                    btn
                });
                idx += 1;
            }
        }
    }
    col.into()
}

/// Pixel offset of `sub`'s row from the top of the main list, so its
/// fly-out lines up with it. Computed from the same row and separator
/// heights the list is laid out with.
fn sub_offset(sub: MenuSub, d: Density) -> f32 {
    let mut y = 0.0;
    for entry in ENTRIES {
        match *entry {
            Entry::Sub(s) if s == sub => return y,
            Entry::Sep => y += SEP_H,
            _ => y += row_h(d) + 2.0,
        }
    }
    y
}

/// Laid-out height of a fly-out with `rows` rows (rows, their 2 px gaps and
/// the panel padding), matching `actions_list` and the fly-out container.
fn fly_out_height(rows: usize, d: Density) -> f32 {
    let n = rows as f32;
    n * row_h(d) + (n - 1.0).max(0.0) * 2.0 + SPACE_2 * 2.0
}

/// Where a fly-out starts: level with its row, or higher when that would run
/// it past the bottom of the space below the top bar (never above the top).
fn fly_out_top(row_top: f32, fly_h: f32, avail_h: f32) -> f32 {
    row_top.min(avail_h - fly_h).max(0.0)
}

fn actions_list<'a>(actions: Vec<SubAction>, d: Density, row: usize) -> Element<'a, Message> {
    let mut col = column![].spacing(2).width(Length::Fill);
    for (i, a) in actions.into_iter().enumerate() {
        col = col.push(action_row(a, d, false, row == i, false));
    }
    col.into()
}

fn desktop_dropdown<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let avail_h = app.window_size.height - d.bar_h() - SPACE_3 * 2.0;

    if searching(app) {
        let results = search_actions(app);
        let count = results.len();
        let list = actions_list(results, d, app.ui.menu_row);
        let header = text(match count {
            0 => "No matching command.".to_owned(),
            1 => "1 command".to_owned(),
            _ => format!("{count} commands"),
        })
        .font(BARLOW_MEDIUM)
        .size(TEXT_12)
        .color(tokens::muted());
        let panel = container(
            column![
                search_field(app, d),
                container(header).padding([SPACE_1, SPACE_1]),
                scrollable(list).height(Length::Shrink),
            ]
            .spacing(SPACE_1),
        )
        .padding(SPACE_2)
        .width(Length::Fixed(
            SEARCH_W.min(app.window_size.width - SPACE_3 * 2.0),
        ))
        .max_height(avail_h)
        .style(panel_style);
        return mouse_area(panel).on_press(Message::Ui(UiMsg::Noop)).into();
    }

    let main = container(
        column![
            search_field(app, d),
            container(main_list(app, d, true)).padding(Padding {
                top: SPACE_1,
                ..Padding::ZERO
            }),
        ]
        .spacing(SPACE_1),
    )
    .padding(SPACE_2)
    .width(Length::Fixed(MAIN_W))
    .style(panel_style);

    // The whole Menu fits a 760 px tall window; below that it scrolls rather
    // than being cut off.
    let main: Element<'a, Message> = scrollable(main).height(Length::Shrink).into();

    let mut layout = row![main].spacing(SPACE_1);
    if let Some(sub) = app.ui.menu_sub {
        let actions = sub_actions(app, sub);
        let fly_h = fly_out_height(actions.len(), d);
        let fly = container(scrollable(actions_list(actions, d, app.ui.menu_row)))
            .padding(SPACE_2)
            .width(Length::Fixed(SUB_W))
            .max_height(avail_h)
            .style(panel_style);
        // Search field + its gap + panel padding, then the row's own offset;
        // raised so a fly-out near the bottom (Help, Recent) stays inside the
        // window instead of losing its last rows (Help > About) off the edge.
        let row_top = SPACE_2 + d.field_h() + SPACE_1 * 2.0 + sub_offset(sub, d) - SPACE_2;
        let top = fly_out_top(row_top, fly_h, avail_h);
        layout = layout.push(column![Space::new().height(Length::Fixed(top)), fly]);
    }

    mouse_area(layout).on_press(Message::Ui(UiMsg::Noop)).into()
}

fn compact_sheet<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let header = row![
        text("Menu")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_16)
            .color(tokens::text()),
        Space::new().width(Length::Fill),
        kit::icon_button("x", false, Some(Message::Ui(UiMsg::CloseMenu)), d),
    ]
    .align_y(Alignment::Center)
    .padding([SPACE_1, SPACE_3]);

    let body: Element<'a, Message> = if searching(app) && search_actions(app).is_empty() {
        text("No matching command.")
            .font(BARLOW_MEDIUM)
            .size(TEXT_13)
            .color(tokens::muted())
            .into()
    } else if searching(app) {
        actions_list(search_actions(app), d, app.ui.menu_row)
    } else if let Some(sub) = app.ui.menu_sub {
        let back = action_row(
            SubAction {
                label: sub.title().to_owned(),
                detail: "",
                icon: Some(("caret-left", true)),
                shortcut: "",
                checked: false,
                enabled: true,
                on_press: Some(Message::Ui(UiMsg::MenuSub(None))),
            },
            d,
            true,
            false,
            false,
        );
        column![
            back,
            kit::divider(),
            actions_list(sub_actions(app, sub), d, app.ui.menu_row)
        ]
        .spacing(SPACE_1)
        .into()
    } else {
        main_list(app, d, false)
    };

    let sheet = container(column![
        header,
        container(search_field(app, d)).padding([SPACE_1, SPACE_3]),
        scrollable(
            container(body)
                .padding([SPACE_1, SPACE_3])
                .width(Length::Fill)
        )
        .height(Length::Fill)
        .width(Length::Fill),
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

/// One Menu row: tinted icon, label, optional hint, shortcut or fly-out caret.
fn action_row<'a>(
    a: SubAction,
    d: Density,
    open: bool,
    focused: bool,
    has_fly: bool,
) -> Element<'a, Message> {
    let fg = if a.enabled {
        tokens::text()
    } else {
        tokens::disabled()
    };
    let mut r = row![].spacing(SPACE_2).align_y(Alignment::Center);
    r = r.push(match (a.checked, a.icon) {
        (true, _) => icons::mono("check", 15.0, tokens::accent()),
        (false, Some((name, fill))) if a.enabled => icons::tinted(name, fill, 15.0, fg),
        (false, Some((name, fill))) => icons::mono_icon(name, fill, 15.0, fg),
        (false, None) => Space::new().width(Length::Fixed(15.0)).into(),
    });
    r = r.push(
        text(a.label)
            .font(if open { BARLOW_SEMIBOLD } else { BARLOW_MEDIUM })
            .size(TEXT_13)
            .color(fg)
            .width(Length::Fill),
    );
    if !a.detail.is_empty() {
        r = r.push(
            text(a.detail)
                .font(BARLOW)
                .size(TEXT_11)
                .color(tokens::muted()),
        );
    }
    if has_fly {
        r = r.push(icons::mono("caret-right", 12.0, tokens::muted()));
    } else if !a.shortcut.is_empty() {
        r = r.push(
            text(crate::input::localize_shortcut(a.shortcut))
                .font(BSC_MEDIUM)
                .size(TEXT_12)
                .color(if a.enabled {
                    tokens::muted()
                } else {
                    tokens::disabled()
                }),
        );
    }
    menu_row_button(r, d, open, focused, a.on_press)
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
            .height(Length::Fixed(row_h(density)))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_list_has_nineteen_rows_and_every_command_exists() {
        assert_eq!(rows().count(), 20);
        for entry in ENTRIES {
            if let Entry::Cmd(id, _) = entry {
                assert!(commands::find(id).is_some(), "unknown command {id}");
            }
        }
        for sub in [
            MenuSub::Recent,
            MenuSub::Import,
            MenuSub::Export,
            MenuSub::Print,
            MenuSub::TraceImage,
            MenuSub::Select,
            MenuSub::Transform,
            MenuSub::Help,
        ] {
            for (id, _) in sub.commands() {
                assert!(
                    commands::find(id).is_some(),
                    "{sub:?} cites unknown command {id}"
                );
            }
            let (name, fill) = sub.icon();
            let _ = icons::mono_icon(name, fill, 14.0, Color::WHITE);
        }
    }
}

#[cfg(test)]
mod fly_out_tests {
    use super::*;

    #[test]
    fn help_fly_out_stays_inside_short_windows() {
        let d = Density::Comfortable;
        let help = MenuSub::Help.commands().len();
        assert_eq!(MenuSub::Help.commands().last().unwrap().0, "about");
        let fly_h = fly_out_height(help, d);
        let row_top =
            SPACE_2 + d.field_h() + SPACE_1 * 2.0 + sub_offset(MenuSub::Help, d) - SPACE_2;
        for window_h in [600.0_f32, 700.0, 760.0, 900.0] {
            let avail_h = window_h - d.bar_h() - SPACE_3 * 2.0;
            let top = fly_out_top(row_top, fly_h, avail_h);
            assert!(
                top + fly_h <= avail_h + 0.5,
                "About cut off at {window_h} px"
            );
            assert!(top <= row_top);
        }
        // Tall enough: it still lines up with its row.
        assert_eq!(fly_out_top(row_top, fly_h, 5000.0), row_top);
    }
}
