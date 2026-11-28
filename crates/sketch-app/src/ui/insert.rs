//! Insert drawer for Symbols and Room labels (`docs/ui-redesign/01-design-spec.md` §5.5, P8).
//!
//! Renders as a floating card on Wide (`~540px`) and Medium (`~420px`) screens
//! and as a full-screen sheet on Compact screens. A Wide drawer lists the symbol
//! categories down its side; the narrower ones pick them from a drop-down. Uses
//! theme tokens exclusively (`tokens::*`, `kit::*`, `icons::mono`) and
//! `symlib::render_fitted` vector previews.

use std::collections::HashSet;

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::text::Wrapping;
use iced::widget::{
    button, canvas, column, container, mouse_area, opaque, pick_list, row,
    scrollable as scroll_box, stack, text, text_input, Column, Row, Space,
};
use iced::{alignment, Alignment, Background, Border, Color, Element, Length, Padding, Rectangle};
use sketch_core::symbols::Symbol;

use crate::dock::{CategoryOption, DockMsg, DragItem, ALPHA};
use crate::icons;
use crate::symlib::{self, SymIndex, SymItem};
use crate::ui::fonts::{BARLOW, BARLOW_MEDIUM, BARLOW_SEMIBOLD};
use crate::ui::kit::{self, Segment};
use crate::ui::layout::SizeClass;
use crate::ui::symbol_library::ALL_KEY;
use crate::ui::tokens::{
    self, Density, FOCUS_RING_WIDTH, RADIUS_CONTROL, RADIUS_SHEET, RADIUS_TILE, SPACE_1, SPACE_2,
    SPACE_3, TEXT_11, TEXT_12, TEXT_13,
};
use crate::ui::tray::DrawerTab;
use crate::ui::Msg as UiMsg;
use crate::{symbol_manager, App, Message};

const SYMBOL_COLS: usize = 3;
const DRAWER_MAX_W: f32 = 420.0;
/// A Wide window has room for the category list beside the symbols.
const DRAWER_WIDE_W: f32 = 540.0;
const DRAWER_MAX_H: f32 = 560.0;
const CATEGORY_LIST_W: f32 = 164.0;
const FAVORITES_KEY: &str = "favorites";
pub(crate) const PINNED_HINT: &str = "Pinned: stays open until you close it";

/// The floating card's width on Wide and Medium screens.
fn card_width(window_w: f32) -> f32 {
    let max = if matches!(SizeClass::of(window_w), SizeClass::Wide) {
        DRAWER_WIDE_W
    } else {
        DRAWER_MAX_W
    };
    (window_w - 32.0).clamp(280.0, max)
}

/// Overlay layer for the Insert drawer (Symbols or Room labels).
pub fn layer<'a>(app: &'a App) -> Option<Element<'a, Message>> {
    let tab = app.ui.drawer?;
    let sc = SizeClass::of(app.window_size.width);
    let d = app.ui.density;

    let inner_w = match sc {
        SizeClass::Compact => (app.window_size.width - SPACE_3 * 2.0).max(260.0),
        SizeClass::Medium | SizeClass::Wide => card_width(app.window_size.width) - SPACE_3 * 2.0,
    };

    let body: Element<'a, Message> = match tab {
        DrawerTab::Symbols => symbols_body(app, d, sc),
        DrawerTab::Labels => labels_body(app, d, inner_w),
    };

    let content = column![header(app, tab, d), kit::divider(), body]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .height(Length::Fill);

    let drawer_surface: Element<'a, Message> = match sc {
        SizeClass::Compact => opaque(
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(SPACE_3)
                .style(|_| container::Style {
                    background: Some(Background::Color(tokens::panel())),
                    ..container::Style::default()
                }),
        ),
        SizeClass::Medium | SizeClass::Wide => {
            let card_w = card_width(app.window_size.width);
            let card_h = (app.window_size.height - 160.0).clamp(320.0, DRAWER_MAX_H);
            let card = opaque(
                container(content)
                    .width(Length::Fixed(card_w))
                    .height(Length::Fixed(card_h))
                    .padding(SPACE_3)
                    .style(|_| container::Style {
                        background: Some(Background::Color(tokens::panel())),
                        border: Border {
                            color: tokens::rule(),
                            width: 1.0,
                            radius: RADIUS_SHEET.into(),
                        },
                        shadow: tokens::floating_shadow(),
                        ..container::Style::default()
                    }),
            );
            let positioned = container(card)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::End)
                .align_y(Alignment::End)
                .padding(Padding {
                    top: 60.0,
                    right: 16.0,
                    bottom: 96.0,
                    left: 16.0,
                });
            if app.ui.drawer_pinned {
                positioned.into()
            } else {
                stack![
                    mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
                        .on_press(Message::Ui(UiMsg::OpenDrawer(None))),
                    positioned,
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        }
    };

    Some(drawer_surface)
}

fn header<'a>(app: &'a App, tab: DrawerTab, d: Density) -> Element<'a, Message> {
    let tabs = kit::segmented(
        vec![
            Segment {
                label: "Symbols",
                selected: tab == DrawerTab::Symbols,
                focused: false,
                on_press: Some(Message::Ui(UiMsg::OpenDrawer(Some(DrawerTab::Symbols)))),
            },
            Segment {
                label: "Room labels",
                selected: tab == DrawerTab::Labels,
                focused: false,
                on_press: Some(Message::Ui(UiMsg::OpenDrawer(Some(DrawerTab::Labels)))),
            },
        ],
        d,
    );

    let pin_label = if app.ui.drawer_pinned {
        "Pinned"
    } else {
        "Pin"
    };
    let pin_btn = pin_button(pin_label, app.ui.drawer_pinned, d);
    let close_btn = kit::icon_button("x", false, Some(Message::Ui(UiMsg::OpenDrawer(None))), d);

    row![tabs, Space::new().width(Length::Fill), pin_btn, close_btn]
        .spacing(SPACE_1)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}

/// The line under the list. While pinned it says what that means.
fn footer_hint(app: &App, unpinned: &'static str) -> &'static str {
    if app.ui.drawer_pinned {
        PINNED_HINT
    } else {
        unpinned
    }
}

fn pin_button<'a>(label: &'a str, pinned: bool, d: Density) -> Element<'a, Message> {
    let fg = if pinned {
        tokens::accent()
    } else {
        tokens::text()
    };
    let h = if matches!(d, Density::Touch) {
        36.0
    } else {
        28.0
    };
    button(
        container(
            row![
                icons::mono("push-pin", 14.0, fg),
                text(label)
                    .font(if pinned {
                        BARLOW_SEMIBOLD
                    } else {
                        BARLOW_MEDIUM
                    })
                    .size(TEXT_12)
                    .color(fg),
            ]
            .spacing(SPACE_1)
            .align_y(Alignment::Center),
        )
        .height(Length::Fill)
        .align_y(Alignment::Center),
    )
    .height(Length::Fixed(h))
    .padding([0, 10])
    .on_press(Message::Ui(UiMsg::ToggleDrawerPin))
    .style(move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if pinned {
                tokens::accent_soft()
            } else if hovered {
                tokens::hover()
            } else {
                tokens::field()
            })),
            text_color: fg,
            border: Border {
                color: if pinned {
                    tokens::accent()
                } else {
                    tokens::rule()
                },
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..button::Style::default()
        }
    })
    .into()
}

// ---------------------------------------------------------------------------
// Symbols mode
// ---------------------------------------------------------------------------

fn symbols_body<'a>(app: &'a App, d: Density, sc: SizeClass) -> Element<'a, Message> {
    let ix = app.symbols_lib();
    let search = search_row(
        "Find in all symbols…",
        &app.dock.symbol_search,
        |s| Message::Dock(DockMsg::SymbolSearch(s)),
        None,
        Message::Dock(DockMsg::SymbolSearch(String::new())),
        d,
    );

    let fav_count = match app.symbol_favorites.as_deref() {
        Some(list) => list.iter().filter(|id| ix.by_id(id).is_some()).count(),
        None => ix.favorites.len(),
    };
    let total_count: usize = ix.categories.iter().map(|c| c.items.len()).sum();

    let mut cats = Vec::with_capacity(ix.categories.len() + 2);
    cats.push(CategoryEntry::new(FAVORITES_KEY, "Favorites", fav_count));
    cats.push(CategoryEntry::new(ALL_KEY, "All symbols", total_count));
    for cat in &ix.categories {
        cats.push(CategoryEntry::new(&cat.key, &cat.name, cat.items.len()));
    }
    // A search covers every category, so while one runs "All symbols" is lit.
    let active = if app.dock.symbol_search.trim().is_empty() {
        app.dock.symbol_category.as_str()
    } else {
        ALL_KEY
    };

    let items = app.dock.symbol_items(ix, app.symbol_favorites.as_deref());
    let fav_ids: HashSet<String> = match app.symbol_favorites.as_deref() {
        Some(list) => list.iter().cloned().collect(),
        None => ix.favorites.iter().map(|i| i.id.clone()).collect(),
    };

    let grid_content: Element<'a, Message> = if items.is_empty() {
        column![
            text("No matching symbols")
                .font(BARLOW_MEDIUM)
                .size(TEXT_13)
                .color(tokens::muted()),
            row![
                kit::text_button(
                    "Show all symbols",
                    Some(choose_category(ALL_KEY, "All symbols")),
                    d,
                ),
                kit::text_button(
                    "Edit library",
                    Some(Message::Dock(DockMsg::OrganizeSymbols)),
                    d,
                ),
            ]
            .spacing(SPACE_2),
        ]
        .spacing(SPACE_2)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .padding([SPACE_3, 0.0])
        .into()
    } else {
        let mut grid = Column::new().spacing(SPACE_2).width(Length::Fill);
        for chunk in items.chunks(SYMBOL_COLS) {
            let mut r = Row::new().spacing(SPACE_2).width(Length::Fill);
            for item in chunk {
                let selected = app.placing_symbol.as_deref() == Some(item.id.as_str())
                    || app.dock.preview.as_deref() == Some(item.id.as_str());
                let is_fav = fav_ids.contains(&item.id);
                r = r.push(symbol_tile(ix, item, selected, is_fav, d));
            }
            for _ in chunk.len()..SYMBOL_COLS {
                r = r.push(Space::new().width(Length::FillPortion(1)));
            }
            grid = grid.push(r);
        }
        grid.into()
    };

    let list_scroll = scroll_box(
        container(grid_content)
            .padding(Padding {
                right: 12.0,
                ..Padding::ZERO
            })
            .width(Length::Fill),
    )
    .direction(Direction::Vertical(
        Scrollbar::new().width(5).scroller_width(5),
    ))
    .width(Length::Fill)
    .height(Length::Fill);

    let mut footer = row![
        text(footer_hint(app, "Tap to arm, or drag onto plan"))
            .font(BARLOW)
            .size(TEXT_11)
            .color(tokens::muted()),
        Space::new().width(Length::Fill),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    if !ix.hidden.is_empty() {
        footer = footer.push(kit::text_button(
            format!("Restore ({})", ix.hidden.len()),
            Some(Message::SymbolManager(symbol_manager::Msg::RestoreAll)),
            d,
        ));
    }
    footer = footer.push(kit::text_button(
        "Edit library",
        Some(Message::Dock(DockMsg::OrganizeSymbols)),
        d,
    ));

    let browse: Element<'a, Message> = if matches!(sc, SizeClass::Wide) {
        row![
            category_list(&cats, active, d),
            container(Space::new())
                .width(Length::Fixed(1.0))
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Background::Color(tokens::rule())),
                    ..container::Style::default()
                }),
            list_scroll,
        ]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        column![
            category_picker(&cats, active, d),
            kit::divider(),
            list_scroll
        ]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    };

    column![search, browse, kit::divider(), footer]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// One entry of the category list or drop-down: a view (Favorites, All) or one
/// of the library's categories.
struct CategoryEntry {
    key: String,
    name: String,
    count: usize,
}

impl CategoryEntry {
    fn new(key: &str, name: &str, count: usize) -> Self {
        CategoryEntry {
            key: key.to_owned(),
            name: name.to_owned(),
            count,
        }
    }
}

fn choose_category(key: &str, name: &str) -> Message {
    Message::Dock(DockMsg::SymbolCategory(CategoryOption {
        key: key.to_owned(),
        name: name.to_owned(),
    }))
}

/// The categories down the side of the Wide drawer, each one click away.
fn category_list<'a>(cats: &[CategoryEntry], active: &str, d: Density) -> Element<'a, Message> {
    let mut list = Column::new().spacing(1.0).width(Length::Fill);
    for (i, entry) in cats.iter().enumerate() {
        // Favorites and All come first, apart from the library's categories.
        if i == 2 {
            list = list.push(container(kit::divider()).padding([2, 0]));
        }
        list = list.push(category_line(entry, active == entry.key, d));
    }
    scroll_box(
        container(list)
            .padding(Padding {
                right: SPACE_2,
                ..Padding::ZERO
            })
            .width(Length::Fill),
    )
    .direction(Direction::Vertical(
        Scrollbar::new().width(5).scroller_width(5),
    ))
    .width(Length::Fixed(CATEGORY_LIST_W))
    .height(Length::Fill)
    .into()
}

fn category_line<'a>(entry: &CategoryEntry, selected: bool, d: Density) -> Element<'a, Message> {
    let fg = if selected {
        tokens::accent()
    } else {
        tokens::text()
    };
    let count_fg = if selected {
        tokens::accent()
    } else {
        tokens::muted()
    };
    let h = if matches!(d, Density::Touch) {
        36.0
    } else {
        26.0
    };
    let mut line = Row::new().spacing(SPACE_1).align_y(Alignment::Center);
    if entry.key == FAVORITES_KEY {
        line = line.push(icons::mono("star", 11.0, fg));
    }
    line = line
        .push(
            container(
                text(entry.name.clone())
                    .font(if selected {
                        BARLOW_SEMIBOLD
                    } else {
                        BARLOW_MEDIUM
                    })
                    .size(TEXT_12)
                    .color(fg)
                    .wrapping(Wrapping::None),
            )
            .width(Length::Fill)
            .clip(true),
        )
        .push(
            text(entry.count.to_string())
                .font(BARLOW)
                .size(TEXT_11)
                .color(count_fg),
        );
    button(
        container(line)
            .width(Length::Fill)
            .height(Length::Fixed(h))
            .padding([0, 6])
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(0)
    .on_press(choose_category(&entry.key, &entry.name))
    .style(move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: if selected {
                Some(Background::Color(tokens::accent_soft()))
            } else if hovered {
                Some(Background::Color(tokens::hover()))
            } else {
                None
            },
            text_color: fg,
            border: Border {
                radius: RADIUS_CONTROL.into(),
                ..Border::default()
            },
            ..button::Style::default()
        }
    })
    .into()
}

/// The narrower drawers keep Favorites and All as buttons and put the library's
/// categories in one drop-down.
fn category_picker<'a>(cats: &[CategoryEntry], active: &str, d: Density) -> Element<'a, Message> {
    let fav_count = cats.first().map_or(0, |c| c.count);
    let total_count = cats.get(1).map_or(0, |c| c.count);
    let options: Vec<CategoryOption> = cats
        .iter()
        .skip(2)
        .map(|c| CategoryOption {
            key: c.key.clone(),
            name: format!("{} ({})", c.name, c.count),
        })
        .collect();
    let selected = options.iter().find(|o| o.key == active).cloned();
    let pad = ((d.field_h() - 16.0).max(4.0)) * 0.5;
    let drop_down = pick_list(options, selected, |o: CategoryOption| {
        Message::Dock(DockMsg::SymbolCategory(o))
    })
    .placeholder("Choose a category")
    .width(Length::Fill)
    .text_size(TEXT_12)
    .font(BARLOW)
    .padding(Padding {
        top: pad,
        bottom: pad,
        left: 8.0,
        right: 8.0,
    })
    .menu_style(|_| iced::overlay::menu::Style {
        background: Background::Color(tokens::panel()),
        border: Border {
            color: tokens::rule(),
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        text_color: tokens::text(),
        selected_text_color: tokens::on_accent(),
        selected_background: Background::Color(tokens::accent()),
        shadow: tokens::floating_shadow(),
    })
    .style(|_, _| pick_list::Style {
        text_color: tokens::text(),
        placeholder_color: tokens::muted(),
        handle_color: tokens::muted(),
        background: Background::Color(tokens::input_bg()),
        border: Border {
            color: tokens::rule(),
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
    });
    row![
        kit::chip(
            format!("Favorites ({fav_count})"),
            active == FAVORITES_KEY,
            Some(choose_category(FAVORITES_KEY, "Favorites")),
            d,
        ),
        kit::chip(
            format!("All {total_count}"),
            active == ALL_KEY,
            Some(choose_category(ALL_KEY, "All symbols")),
            d,
        ),
        drop_down,
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn symbol_tile<'a>(
    ix: &'a SymIndex,
    item: &SymItem,
    selected: bool,
    is_fav: bool,
    d: Density,
) -> Element<'a, Message> {
    let tile_h = if matches!(d, Density::Touch) {
        98.0
    } else {
        90.0
    };
    let bg = if selected {
        tokens::accent_soft()
    } else {
        tokens::field()
    };
    let border_col = if selected {
        tokens::accent()
    } else {
        tokens::rule()
    };

    let preview: Element<'a, Message> = if let Some(def) = ix.symbol(&item.id) {
        canvas(SymbolThumb {
            symbol: def,
            stroke: tokens::text(),
            bg,
        })
        .width(Length::Fixed(48.0))
        .height(Length::Fixed(44.0))
        .into()
    } else {
        icons::mono(item.icon, 24.0, tokens::muted())
    };

    let label_el = text(item.label.clone())
        .font(if selected {
            BARLOW_SEMIBOLD
        } else {
            BARLOW_MEDIUM
        })
        .size(TEXT_11)
        .color(tokens::text())
        .align_x(alignment::Horizontal::Center)
        .wrapping(Wrapping::WordOrGlyph);

    let tile_body = container(
        column![preview, label_el]
            .spacing(SPACE_1)
            .align_x(Alignment::Center)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fixed(tile_h))
    .padding(Padding {
        top: 8.0,
        right: 6.0,
        bottom: 6.0,
        left: 6.0,
    })
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border_col,
            width: if selected { 1.5 } else { 1.0 },
            radius: RADIUS_TILE.into(),
        },
        ..container::Style::default()
    });

    let id = item.id.clone();
    let sym_ctx = Message::SymbolManager(symbol_manager::Msg::Context(id.clone()));
    let trigger_area = kit::long_press(
        mouse_area(tile_body)
            .on_press(Message::DockDragPress(DragItem::Symbol(id.clone())))
            .on_release(Message::DockItemRelease(DragItem::Symbol(id.clone())))
            .on_right_press(sym_ctx.clone()),
        sym_ctx,
    );

    let menu_btn = container(
        button(
            container(icons::mono("dots-three", 13.0, tokens::muted()))
                .width(Length::Fixed(22.0))
                .height(Length::Fixed(22.0))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .padding(0)
        .on_press(Message::SymbolManager(symbol_manager::Msg::Context(id)))
        .style(|_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: if hovered {
                    Some(Background::Color(tokens::hover()))
                } else {
                    None
                },
                border: Border {
                    radius: RADIUS_CONTROL.into(),
                    ..Border::default()
                },
                ..button::Style::default()
            }
        }),
    )
    .width(Length::Fill)
    .padding(Padding {
        top: 2.0,
        right: 2.0,
        ..Padding::ZERO
    })
    .align_x(Alignment::End);

    let overlay_items: Element<'a, Message> = if is_fav {
        let star = container(icons::mono("star", 10.0, tokens::accent())).padding(Padding {
            top: 5.0,
            left: 6.0,
            ..Padding::ZERO
        });
        stack![trigger_area, star, menu_btn].into()
    } else {
        stack![trigger_area, menu_btn].into()
    };

    container(overlay_items)
        .width(Length::FillPortion(1))
        .into()
}

struct SymbolThumb<'a> {
    symbol: &'a Symbol,
    stroke: Color,
    bg: Color,
}

impl<M> canvas::Program<M> for SymbolThumb<'_> {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        _: &iced::Theme,
        bounds: Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        symlib::render_fitted(
            &mut frame,
            self.symbol,
            bounds.width,
            bounds.height,
            6.0,
            self.stroke,
            self.bg,
        );
        vec![frame.into_geometry()]
    }
}

// ---------------------------------------------------------------------------
// Room labels mode
// ---------------------------------------------------------------------------

fn labels_body<'a>(app: &'a App, d: Density, inner_w: f32) -> Element<'a, Message> {
    let custom_ready = !app.dock.text_search.trim().is_empty();
    let search = search_row(
        "Find or type a label…",
        &app.dock.text_search,
        |s| Message::Dock(DockMsg::TextSearch(s)),
        Some(Message::Dock(DockMsg::TextSubmit)),
        Message::Dock(DockMsg::TextSearch(String::new())),
        d,
    );
    let top_row = row![
        container(search).width(Length::Fill),
        kit::primary_button(
            "Place",
            custom_ready.then_some(Message::Dock(DockMsg::TextSubmit)),
            d,
        ),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let alpha_chips = wrapped_alpha_chips(app.dock.text_range, inner_w, d);
    let items = app.dock.text_items();

    let list_body: Element<'a, Message> = if items.is_empty() {
        container(
            text("No matching labels — tap Place to use as typed")
                .font(BARLOW)
                .size(TEXT_12)
                .color(tokens::muted()),
        )
        .padding([SPACE_3, SPACE_1])
        .width(Length::Fill)
        .into()
    } else {
        let mut col = Column::new().spacing(SPACE_1).width(Length::Fill);
        for lbl in items {
            let active = app.placing_text.as_deref() == Some(lbl)
                || app.dock.text_active.as_deref() == Some(lbl);
            col = col.push(label_row(lbl, active, d));
        }
        col.into()
    };

    let scroll = scroll_box(
        container(list_body)
            .padding(Padding {
                right: 12.0,
                ..Padding::ZERO
            })
            .width(Length::Fill),
    )
    .direction(Direction::Vertical(
        Scrollbar::new().width(5).scroller_width(5),
    ))
    .width(Length::Fill)
    .height(Length::Fill);

    let footer = row![
        text(footer_hint(app, "Tap a label or type your own"))
            .font(BARLOW)
            .size(TEXT_11)
            .color(tokens::muted()),
        Space::new().width(Length::Fill),
        kit::text_button(
            "Edit list",
            Some(Message::Dock(DockMsg::EditTextLibrary)),
            d,
        ),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    column![
        top_row,
        alpha_chips,
        kit::divider(),
        scroll,
        kit::divider(),
        footer
    ]
    .spacing(SPACE_2)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn wrapped_alpha_chips<'a>(active_range: &str, inner_w: f32, d: Density) -> Element<'a, Message> {
    let mut rows_col = Column::new().spacing(SPACE_1).width(Length::Fill);
    let mut current_row = Row::new().spacing(SPACE_1).align_y(Alignment::Center);
    let mut used = 0.0_f32;
    let mut has_item = false;

    for &(label, range) in ALPHA {
        let est_w = label.len() as f32 * 7.2 + 24.0;
        if has_item && used + est_w > inner_w {
            rows_col = rows_col.push(current_row);
            current_row = Row::new().spacing(SPACE_1).align_y(Alignment::Center);
            used = 0.0;
        }
        current_row = current_row.push(kit::chip(
            label,
            active_range == range,
            Some(Message::Dock(DockMsg::TextRange(range))),
            d,
        ));
        used += est_w + SPACE_1;
        has_item = true;
    }
    if has_item {
        rows_col = rows_col.push(current_row);
    }

    rows_col.into()
}

fn label_row<'a>(label: &str, active: bool, d: Density) -> Element<'a, Message> {
    let fg = if active {
        tokens::accent()
    } else {
        tokens::text()
    };
    let bg = if active {
        tokens::accent_soft()
    } else {
        tokens::field()
    };
    let border_col = if active {
        tokens::accent()
    } else {
        tokens::rule()
    };
    let row_content = row![
        icons::mono(
            "text-t",
            14.0,
            if active {
                tokens::accent()
            } else {
                tokens::muted()
            }
        ),
        text(label.to_string())
            .font(if active {
                BARLOW_SEMIBOLD
            } else {
                BARLOW_MEDIUM
            })
            .size(TEXT_13)
            .color(fg),
        Space::new().width(Length::Fill),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let box_el = container(row_content)
        .width(Length::Fill)
        .height(Length::Fixed(d.row_h()))
        .padding([0, 10])
        .align_y(Alignment::Center)
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: border_col,
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..container::Style::default()
        });

    let owned = label.to_string();
    mouse_area(box_el)
        .on_press(Message::DockDragPress(DragItem::Text(owned.clone())))
        .on_release(Message::DockItemRelease(DragItem::Text(owned)))
        .into()
}

// ---------------------------------------------------------------------------
// Shared search input
// ---------------------------------------------------------------------------

fn search_row<'a>(
    placeholder: &'a str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Option<Message>,
    on_clear: Message,
    d: Density,
) -> Element<'a, Message> {
    let mut input = text_input(placeholder, value)
        .on_input(on_input)
        .size(TEXT_13)
        .font(BARLOW)
        .padding([6, 10])
        .width(Length::Fill)
        .style(move |_, status| {
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
        });
    if let Some(submit) = on_submit {
        input = input.on_submit(submit);
    }

    let mut r = row![
        icons::mono("magnifying-glass", 14.0, tokens::muted()),
        input,
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    if !value.is_empty() {
        r = r.push(kit::icon_button("x", false, Some(on_clear), d));
    }

    r.into()
}
