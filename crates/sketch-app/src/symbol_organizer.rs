//! The "Organize Favorite Symbols" dialog — `showSymbolOrganizer`
//! (ui-manager.js:421): a `.sidebar-manager-overlay` dialog with a filter
//! input and a checkbox list of every symbol (sorted by label), plus
//! Restore Defaults / Save Favorites.
//!
//! The feature inventory's S8-008..021 "Manage Symbols" layer is NOT in the
//! captured build — the live original still shows the `Organize...` button
//! and this favorites editor, so that is what gets ported. `Save Favorites`
//! writes `state.symbolFavorites` (persistLibraryState), the dock's
//! Favorites category lists it, and the toast reads "Favorite symbols
//! updated.".

use iced::widget::{
    button, column, container, mouse_area, opaque, row, scrollable, text, text_input, Space,
};
use iced::{Background, Border, Color, Element, Length};

use crate::icons::ph;
use crate::theme::*;
use crate::Message;

/// The dialog's local state (`#symbol-organizer`).
#[derive(Debug, Clone, Default)]
pub struct SymbolOrganizer {
    pub open: bool,
    /// The filter input's live buffer (`label.dataset.search.includes`).
    pub filter: String,
    /// The working copy of the favorite type ids (the checkboxes).
    pub checked: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SymbolOrganizerMsg {
    Open,
    Close,
    Filter(String),
    /// A row's checkbox toggled.
    Toggle(String),
    /// Restore Defaults — the checkboxes only; nothing persists until Save.
    Defaults,
    /// Save Favorites — persists + refreshes the dock + toasts.
    Save,
}

/// The rows: every symbol sorted by label (`localeCompare`).
pub fn sorted_items(ix: &crate::symlib::SymIndex) -> Vec<crate::symlib::SymItem> {
    let mut items: Vec<crate::symlib::SymItem> = ix
        .categories
        .iter()
        .flat_map(|c| c.items.iter().cloned())
        .chain(ix.favorites.iter().cloned())
        .collect();
    // Favorites and category items can overlap; dedupe by id.
    let mut seen = std::collections::HashSet::new();
    items.retain(|i| seen.insert(i.id.clone()));
    items.sort_by_key(|i| i.label.to_lowercase());
    items
}

/// `label.dataset.search.includes(filter.value.trim().toLowerCase())`.
fn matches_filter(label: &str, filter: &str) -> bool {
    let q = filter.trim().to_lowercase();
    q.is_empty() || label.to_lowercase().contains(&q)
}

impl SymbolOrganizer {
    /// Open over the current favorites (the App's saved list, else the
    /// library defaults).
    pub fn open(&mut self, current: &[String]) {
        self.open = true;
        self.filter = String::new();
        self.checked = current.to_vec();
    }

    pub fn close(&mut self) {
        self.open = false;
        self.filter = String::new();
        self.checked.clear();
    }

    /// The dialog's own controls. `defaults` = the library's
    /// `defaultFavorites` ids; the returned bool is the Save flag (the App
    /// persists + toasts).
    pub fn update(&mut self, msg: SymbolOrganizerMsg, defaults: &[String]) -> bool {
        match msg {
            SymbolOrganizerMsg::Filter(value) => self.filter = value,
            SymbolOrganizerMsg::Toggle(id) => {
                if let Some(pos) = self.checked.iter().position(|t| *t == id) {
                    self.checked.remove(pos);
                } else {
                    self.checked.push(id);
                }
            }
            SymbolOrganizerMsg::Defaults => {
                self.checked = defaults.to_vec();
            }
            SymbolOrganizerMsg::Save => {
                // `symbolFavorites = [...checked]` — dedupe, keep order.
                let mut seen = std::collections::HashSet::new();
                self.checked.retain(|t| seen.insert(t.clone()));
                return true;
            }
            SymbolOrganizerMsg::Open | SymbolOrganizerMsg::Close => {}
        }
        false
    }
}

/// The row's checkbox glyph: a 13px box on `--bg-app`; the checked state
/// fills `--bg-active` with a white check (the native dark checkbox).
fn check_glyph(checked: bool) -> Element<'static, Message> {
    container(if checked {
        text("\u{2713}").font(SEGOE).size(11.0).color(WHITE)
    } else {
        text("").font(SEGOE).size(11.0)
    })
    .width(13)
    .height(13)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_| container::Style {
        background: Some(Background::Color(if checked { accent() } else { bg_app() })),
        border: Border {
            color: border(),
            width: 1.0,
            radius: 2.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

/// The dialog overlay, or None when closed.
pub fn layer(
    d: &SymbolOrganizer,
    ix: &crate::symlib::SymIndex,
) -> Option<Element<'static, Message>> {
    if !d.open {
        return None;
    }
    let title = row![
        text("Organize Favorite Symbols")
            .font(SEGOE_SEMIBOLD)
            .size(13.0)
            .color(text_main()),
        Space::new().width(Length::Fill),
        button(text("x").font(SEGOE).size(16.0).color(muted()))
            .padding(2)
            .on_press(Message::SymbolOrganizer(SymbolOrganizerMsg::Close)),
    ]
    .spacing(6);

    let filter = text_input("Filter symbols...", d.filter.as_str())
        .size(12.0)
        .padding([6, 8])
        .on_input(|s| Message::SymbolOrganizer(SymbolOrganizerMsg::Filter(s)));

    let items = sorted_items(ix);
    let rows: Vec<Element<'static, Message>> = items
        .iter()
        .filter(|i| matches_filter(&i.label, &d.filter))
        .map(|i| {
            let checked = d.checked.contains(&i.id);
            let icon: &'static str = i.icon;
            let label_text = i.label.clone();
            let body = row![
                check_glyph(checked),
                ph(icon, 14.0, ICON_BLUE),
                text(label_text).font(SEGOE).size(12.0).color(text_main()),
                Space::new().width(Length::Fill),
            ]
            .spacing(8)
            .align_y(iced::alignment::Vertical::Center);
            container(body).padding([3, 4]).width(Length::Fill).into()
        })
        .collect();
    let list: Element<'static, Message> = if rows.is_empty() {
        Space::new().height(Length::Fill).into()
    } else {
        scrollable(column(rows).width(Length::Fill))
            .height(Length::Fill)
            .into()
    };
    let list_frame = container(list)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .style(|_| container::Style {
            background: Some(Background::Color(bg_app())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    let chip = |label_s: &'static str, msg: Message| -> Element<'static, Message> {
        button(container(text(label_s).font(SEGOE).size(11.0).color(text_main())).padding([5, 8]))
            .on_press(msg)
            .style(move |_, status| button::Style {
                background: Some(Background::Color(match status {
                    button::Status::Hovered | button::Status::Pressed => bg_hover(),
                    _ => bg_hover(),
                })),
                border: Border {
                    color: border(),
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..button::Style::default()
            })
            .into()
    };

    let actions = row![
        chip(
            "Restore Defaults",
            Message::SymbolOrganizer(SymbolOrganizerMsg::Defaults)
        ),
        Space::new().width(Length::Fill),
        chip(
            "Save Favorites",
            Message::SymbolOrganizer(SymbolOrganizerMsg::Save)
        ),
    ]
    .spacing(8);

    let card = container(
        column![title, filter, list_frame, actions]
            .spacing(10)
            .padding(12),
    )
    .width(430)
    .height(480)
    .style(|_| container::Style {
        background: Some(Background::Color(bg_panel())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..container::Style::default()
    });

    // The card swallows its own presses; the backdrop closes (the original's
    // `event.target === overlay` check).
    let centred = container(opaque(card))
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    let backdrop = container(centred)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.46).into()),
            ..container::Style::default()
        });
    Some(opaque(mouse_area(backdrop).on_press(
        Message::SymbolOrganizer(SymbolOrganizerMsg::Close),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, label: &str) -> crate::symlib::SymItem {
        crate::symlib::SymItem {
            id: id.to_string(),
            label: label.to_string(),
            icon: "couch",
        }
    }

    fn ix() -> crate::symlib::SymIndex {
        let mut ix = crate::symlib::SymIndex {
            hidden: Default::default(),
            duplicates: Default::default(),
            library: serde_json::from_value(serde_json::json!({
                "symbols": {}, "categories": [], "defaultFavorites": []
            }))
            .unwrap(),
            categories: vec![crate::symlib::SymCategory {
                key: "bath".to_string(),
                name: "Bath".to_string(),
                items: vec![item("b1", "Water Closet"), item("a1", "Bathtub")],
            }],
            favorites: vec![item("f1", "Air Conditioner")],
        };
        ix.categories.push(crate::symlib::SymCategory {
            key: "fav-dup".to_string(),
            name: "FavDup".to_string(),
            items: vec![item("f1", "Air Conditioner")],
        });
        ix
    }

    #[test]
    fn rows_sort_by_label_and_dedupe() {
        let rows = sorted_items(&ix());
        let ids: Vec<&str> = rows.iter().map(|i| i.id.as_str()).collect();
        // f1 appears in favorites AND a category once.
        assert_eq!(
            ids,
            vec!["f1", "a1", "b1"],
            "Air Conditioner < Bathtub < Water Closet"
        );
    }

    #[test]
    fn the_filter_trims_and_matches_case_insensitively() {
        assert!(matches_filter("Water Closet", "  CLO"));
        assert!(!matches_filter("Water Closet", "bath"));
        assert!(matches_filter("Water Closet", "   "));
    }

    #[test]
    fn toggle_defaults_and_save() {
        let mut o = SymbolOrganizer::default();
        o.open(&["f1".to_string()]);
        assert_eq!(o.checked, vec!["f1".to_string()]);
        o.update(SymbolOrganizerMsg::Toggle("a1".to_string()), &[]);
        o.update(SymbolOrganizerMsg::Toggle("f1".to_string()), &[]);
        assert_eq!(o.checked, vec!["a1".to_string()]);
        // Restore Defaults resets the CHECKBOXES only (nothing persists).
        o.update(SymbolOrganizerMsg::Defaults, &["b1".to_string()]);
        assert_eq!(o.checked, vec!["b1".to_string()]);
        // Save persists: the update returns the Save flag and dedupes.
        o.checked.push("b1".to_string());
        assert!(o.update(SymbolOrganizerMsg::Save, &[]));
        assert_eq!(o.checked, vec!["b1".to_string()]);
    }
}
