//! The Text Library manager — `showTextLibraryManager` (ui-manager.js:359):
//! a `.sidebar-manager-overlay` dialog with the label list (select size=14),
//! the label input, the alpha range rail and Add / Remove / Replace.
//!
//! Measurements (theme.css 1213-1248): the overlay dims `rgba(0,0,0,.46)`;
//! the dialog is `min(430px, 100vw-32)` wide, gap 10, padding 12, radius 5,
//! `--bg-panel`; the grid splits `minmax(0,1fr) | 52px` with a 300px min
//! height; the list and input sit on `--bg-app` with 1px `--border`; the
//! alpha rail and the action buttons are `--bg-hover` chips, the active
//! range `#D69E2E` on `#171717` with a `#F6AD55` border. A press on the
//! backdrop closes (the original's `event.target === overlay`).

use iced::widget::{
    button, column, container, mouse_area, opaque, row, scrollable, text, text_input, Space,
};
use iced::{Background, Border, Color, Element, Length};

use crate::theme::*;
use crate::Message;

/// The alpha rail, in the original's order (`manager-alpha`).
pub const RANGES: &[(&str, &str)] = &[
    ("All", "all"),
    ("ABC", "abc"),
    ("DEF", "def"),
    ("GHI", "ghi"),
    ("JKL", "jkl"),
    ("MNO", "mno"),
    ("PQR", "pqr"),
    ("STU", "stu"),
    ("VWX", "vwx"),
    ("YZ", "yz"),
];

/// The label input's widget id (focused on open, like the original's
/// `input.focus()`).
pub const INPUT_ID: &str = "text-manager-input";

/// The dialog's local state (`#text-library-manager`).
#[derive(Debug, Clone, Default)]
pub struct TextManager {
    pub open: bool,
    /// `range` — `'all'` or a lowercase letter run (`'abc'`).
    pub range: &'static str,
    /// The label input's live buffer.
    pub buffer: String,
    /// The list's selected label.
    pub selected: Option<String>,
}

/// `refresh()`'s list: the labels in the range, `localeCompare`-sorted.
pub fn visible_labels(labels: &[String], range: &str) -> Vec<String> {
    let mut items: Vec<String> = labels
        .iter()
        .filter(|label| {
            range == "all"
                || range.contains(
                    &label
                        .trim()
                        .chars()
                        .next()
                        .unwrap_or_default()
                        .to_lowercase()
                        .to_string(),
                )
        })
        .cloned()
        .collect();
    items.sort_by_key(|a| a.to_lowercase());
    items
}

impl TextManager {
    /// Open over the live library (`this.textLibraryLabels` stays the dock's
    /// `text_labels` — the dialog edits it in place).
    pub fn open(&mut self) {
        self.open = true;
        self.range = "all";
        self.buffer = String::new();
        self.selected = None;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.buffer = String::new();
        self.selected = None;
    }

    /// The dialog's own controls; the App arm returns the toasts the
    /// original's `notify()` shows.
    pub fn update(&mut self, msg: TextManagerMsg, labels: &mut Vec<String>) -> Option<String> {
        match msg {
            TextManagerMsg::Range(range) => self.range = range,
            TextManagerMsg::Input(value) => self.buffer = value,
            TextManagerMsg::Pick(label) => {
                self.selected = Some(label.clone());
                // `list.addEventListener('change', () => input.value = list.value)`.
                self.buffer = label;
            }
            TextManagerMsg::Add => {
                let value = self.buffer.trim().to_string();
                if value.is_empty() {
                    return Some("Enter a text label first.".to_string());
                }
                if labels
                    .iter()
                    .any(|l| l.to_lowercase() == value.to_lowercase())
                {
                    return Some("That label already exists.".to_string());
                }
                labels.push(value.clone());
                self.save(labels, Some(value));
            }
            TextManagerMsg::Remove => {
                let Some(value) = self.selected.clone() else {
                    return Some("Select a label to remove.".to_string());
                };
                labels.retain(|l| *l != value);
                self.buffer = String::new();
                self.save(labels, None);
            }
            TextManagerMsg::Replace => {
                let old = self.selected.clone().unwrap_or_default();
                let value = self.buffer.trim().to_string();
                if old.is_empty() || value.is_empty() {
                    return Some("Select a label and enter its replacement.".to_string());
                }
                if labels
                    .iter()
                    .any(|l| *l != old && l.to_lowercase() == value.to_lowercase())
                {
                    return Some("That label already exists.".to_string());
                }
                for l in labels.iter_mut() {
                    if *l == old {
                        *l = value.clone();
                    }
                }
                self.save(labels, Some(value));
            }
            TextManagerMsg::Open | TextManagerMsg::Close => {}
        }
        None
    }

    /// `save(selected)`: dedupe + sort, persist, and keep the picked label.
    fn save(&mut self, labels: &mut Vec<String>, selected: Option<String>) {
        let mut seen = std::collections::HashSet::new();
        labels.retain(|l| seen.insert(l.to_lowercase()));
        labels.sort_by_key(|a| a.to_lowercase());
        self.selected = selected;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextManagerMsg {
    Open,
    Close,
    Range(&'static str),
    Input(String),
    Pick(String),
    Add,
    Remove,
    Replace,
}

/// The `.manager-*` chip button (the alpha rail + the action row share it).
fn chip(label_s: &'static str, active: bool, msg: Message) -> Element<'static, Message> {
    button(
        container(text(label_s).font(SEGOE).size(11.0).color(if active {
            hex(0x171717)
        } else {
            text_main()
        }))
        .padding([5, 8]),
    )
    .on_press(msg)
    .style(move |_, status| button::Style {
        background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => bg_hover(),
            _ if active => hex(0xD69E2E),
            _ => bg_hover(),
        })),
        border: if active {
            Border {
                color: hex(0xF6AD55),
                width: 1.0,
                radius: 2.0.into(),
            }
        } else {
            Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            }
        },
        ..button::Style::default()
    })
    .into()
}

/// The dialog overlay, or None when closed. The labels are the live
/// library (the dock's `text_labels` — the dialog edits it in place).
pub fn layer(d: &TextManager, labels: &[String]) -> Option<Element<'static, Message>> {
    if !d.open {
        return None;
    }
    let title = row![
        text("Manage Text Library")
            .font(SEGOE_SEMIBOLD)
            .size(13.0)
            .color(text_main()),
        Space::new().width(Length::Fill),
        button(text("x").font(SEGOE).size(16.0).color(muted()))
            .padding(2)
            .on_press(Message::TextLibraryManager(TextManagerMsg::Close)),
    ]
    .spacing(6);

    // The list (`select size=14`): the labels in the range, sorted.
    let items = visible_labels(labels, d.range);
    let list_rows: Vec<Element<'static, Message>> = items
        .iter()
        .map(|label| {
            let label = label.clone();
            let picked = d.selected.as_deref() == Some(label.as_str());
            let label_owned = label.clone();
            let row_text = text(label_owned).font(SEGOE).size(12.0).color(if picked {
                WHITE
            } else {
                text_main()
            });
            button(container(row_text).padding([3, 6]).width(Length::Fill))
                .width(Length::Fill)
                .on_press(Message::TextLibraryManager(TextManagerMsg::Pick(
                    label.clone(),
                )))
                .style(move |_, status| button::Style {
                    background: Some(Background::Color(match status {
                        button::Status::Hovered | button::Status::Pressed => bg_hover(),
                        _ if picked => accent(),
                        _ => Color::TRANSPARENT,
                    })),
                    ..button::Style::default()
                })
                .into()
        })
        .collect();
    let list = list_frame_rows(list_rows);

    let input = text_input("Label text", d.buffer.as_str())
        .size(12.0)
        .padding([6, 8])
        .id(INPUT_ID)
        .on_input(|s| Message::TextLibraryManager(TextManagerMsg::Input(s)));

    let main = column![list, input]
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Fill);

    // The alpha rail (52px column).
    let rail = column(
        RANGES
            .iter()
            .map(|(label_s, key)| {
                chip(
                    label_s,
                    d.range == *key,
                    Message::TextLibraryManager(TextManagerMsg::Range(key)),
                )
            })
            .collect::<Vec<Element<'static, Message>>>(),
    )
    .spacing(4)
    .width(52);

    let grid = row![main, rail].spacing(8).height(300.0);

    let actions = row![
        chip(
            "Add",
            false,
            Message::TextLibraryManager(TextManagerMsg::Add)
        ),
        chip(
            "Remove",
            false,
            Message::TextLibraryManager(TextManagerMsg::Remove)
        ),
        chip(
            "Replace",
            false,
            Message::TextLibraryManager(TextManagerMsg::Replace)
        ),
        Space::new().width(Length::Fill),
    ]
    .spacing(8);

    let card = container(column![title, grid, actions].spacing(10).padding(12))
        .width(430)
        .max_height(620)
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
        Message::TextLibraryManager(TextManagerMsg::Close),
    )))
}

/// The `.manager-list` frame: 1px border on `--bg-app`, scrollable.
fn list_frame_rows(rows: Vec<Element<'static, Message>>) -> Element<'static, Message> {
    let list: Element<'static, Message> = if rows.is_empty() {
        Space::new().height(Length::Fill).into()
    } else {
        scrollable(column(rows).width(Length::Fill))
            .height(Length::Fill)
            .into()
    };
    container(list)
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
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> Vec<String> {
        ["Living Room", "kitchen", "Half Bath", "Basement"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn the_range_filters_by_the_first_letter_and_sorts() {
        let all = visible_labels(&labels(), "all");
        // `localeCompare` is case-insensitive at these boundaries.
        assert_eq!(all, vec!["Basement", "Half Bath", "kitchen", "Living Room"]);
        let k = visible_labels(&labels(), "k");
        assert_eq!(k, vec!["kitchen"]);
        // The trim precedes the letter test (the original trims then reads
        // charAt(0)).
        let h = visible_labels(&labels(), "h");
        assert_eq!(h, vec!["Half Bath"]);
    }

    #[test]
    fn add_trims_refuses_blank_and_duplicates() {
        let mut m = TextManager {
            open: true,
            ..TextManager::default()
        };
        let mut lib = labels();
        // Blank → the toast, nothing added.
        m.buffer = "   ".to_string();
        assert_eq!(
            m.update(TextManagerMsg::Add, &mut lib),
            Some("Enter a text label first.".to_string())
        );
        assert_eq!(lib.len(), 4);
        // A case-insensitive duplicate is refused...
        m.buffer = "KITCHEN".to_string();
        assert_eq!(
            m.update(TextManagerMsg::Add, &mut lib),
            Some("That label already exists.".to_string())
        );
        // ...a fresh label lands trimmed, sorted, selected.
        m.buffer = "  Pantry  ".to_string();
        assert_eq!(m.update(TextManagerMsg::Add, &mut lib), None);
        assert_eq!(m.selected.as_deref(), Some("Pantry"));
        assert_eq!(
            visible_labels(&lib, "all"),
            vec!["Basement", "Half Bath", "kitchen", "Living Room", "Pantry"]
        );
    }

    #[test]
    fn replace_maps_and_remove_deletes() {
        let mut m = TextManager {
            open: true,
            ..TextManager::default()
        };
        let mut lib = labels();
        // Remove without a selection toasts.
        assert_eq!(
            m.update(TextManagerMsg::Remove, &mut lib),
            Some("Select a label to remove.".to_string())
        );
        // Pick (the change handler syncs the input) then remove.
        m.update(TextManagerMsg::Pick("Basement".to_string()), &mut lib);
        assert_eq!(m.buffer, "Basement");
        assert_eq!(m.update(TextManagerMsg::Remove, &mut lib), None);
        assert!(!lib.contains(&"Basement".to_string()));
        assert!(m.buffer.is_empty());
        // Replace needs a selection AND a value.
        assert_eq!(
            m.update(TextManagerMsg::Replace, &mut lib),
            Some("Select a label and enter its replacement.".to_string())
        );
        // Pick again; the replacement maps the old label (trimmed, deduped,
        // sorted) and keeps it selected.
        m.update(TextManagerMsg::Pick("kitchen".to_string()), &mut lib);
        m.buffer = "  Pantry  ".to_string();
        assert_eq!(m.update(TextManagerMsg::Replace, &mut lib), None);
        assert_eq!(m.selected.as_deref(), Some("Pantry"));
        assert!(lib.contains(&"Pantry".to_string()) && !lib.contains(&"kitchen".to_string()));
    }
}
