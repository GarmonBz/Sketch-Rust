//! The Area Properties dialog (screen 7, S7-005): `showSelectedAreaProperties`
//! plus `updateSelectedAreaProperties`. Every rect comes from the live original
//! at 1400x860, recorded in `refs/ui-spec/area-properties-spec.json`.
//!
//! The card is 520x456 at (440,202) with `--bg-panel`, a 1px `--border`, radius
//! 5, `0 16px 46px rgba(0,0,0,.5)` and 12px padding. The close × sits at
//! (935.3,215) 11.7x23. Each row puts its label on the left and the control at
//! x 591 (356 wide) or x 602 (334 wide). The auto-post checkboxes form a 3x2
//! grid at x 464/623.7/783.3 (152.7 wide) and y 520/543. Cancel (59.5x27) and
//! Apply (51.4x27) sit at y 618.
//!
//! The original's three controls are native `<select>`s. iced has no native
//! select, so each one is a `pick_list` styled to the same box; that is a
//! documented stand-in, the same one the Edit Tools' Line Style uses.

use iced::widget::{button, checkbox, container, pick_list, radio, text, text_input, Space};
use iced::{Background, Border, Color, Element, Length, Padding, Shadow, Vector};

use crate::area_props::AreaProperties;
use crate::snap::nudge;
use crate::theme::{bg_app, bg_panel, border, muted_alt};
use crate::theme::{bg_hover, text_main, ARIAL};
use crate::Message;

/// The card's top-left and size.
const CARD: (f32, f32, f32, f32) = (440.0, 202.0, 520.0, 456.0);
/// A pinned child inside the card lands 2px above its measured position (the
/// pin-origin quirk the command popup documents), so every placed control is
/// shifted down by this calibration. Measured against the reference: with it
/// the rows land on 221/258/298/520/543/577, the original's own pixels.
const PIN_Y: f32 = 2.0;

/// Where the label column and the control column start.
const LABEL_X: f32 = 464.0;
const CONTROL_X: f32 = 591.0;
const CONTROL_W: f32 = 356.0;
const NARROW_X: f32 = 602.0;
const NARROW_W: f32 = 334.0;

/// The Measure To options, in the original's order.
const MEASURE: [&str; 3] = ["Centerline", "Interior Face", "Exterior Face"];
fn measure_value(label: &str) -> &'static str {
    match label {
        "Interior Face" => "interior",
        "Exterior Face" => "exterior",
        _ => "center",
    }
}
fn measure_label(value: &str) -> &'static str {
    match value {
        "interior" => "Interior Face",
        "exterior" => "Exterior Face",
        _ => "Centerline",
    }
}

/// The dialog's own state: the values being edited plus the raw text fields.
#[derive(Debug, Clone, Default)]
pub struct AreaDialog {
    pub open: bool,
    /// The payload Apply sends.
    pub props: AreaProperties,
    /// The Factor field's text (`''` is factor 0 in the original).
    pub factor_text: String,
    /// The Suffix field's text.
    pub suffix_text: String,
    /// The area-code choices, `"CODE - Name"`.
    pub codes: Vec<String>,
    /// The Positive/Negative radio. Display only: the original's
    /// `updateSelectedAreaProperties` never reads it (the sign lives on the
    /// area code), so Apply ignores it — reproduced, not corrected.
    pub positive: bool,
}

/// What a control in the dialog asks for.
#[derive(Debug, Clone)]
pub enum AreaDialogMsg {
    Code(String),
    Name(String),
    Factor(String),
    Positive(bool),
    AutoSubtract(String),
    Measure(String),
    Post(PostSwitch, bool),
    Suffix(String),
    Apply,
    Cancel,
}

/// Which auto-post switch a checkbox toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostSwitch {
    Code,
    Name,
    Dims,
    Calcs,
    Base,
    Factor,
}

impl AreaDialog {
    /// Open with the first selected area's values (`showSelectedAreaProperties`).
    pub fn open_with(props: AreaProperties, codes: Vec<String>) -> AreaDialog {
        let factor_text = props
            .factor
            .map(|f| {
                if (f - f.round()).abs() < 1e-9 {
                    format!("{}", f.round() as i64)
                } else {
                    format!("{f}")
                }
            })
            .unwrap_or_default();
        let suffix_text = props
            .auto_post
            .as_ref()
            .map(|p| p.suffix.clone())
            .unwrap_or_else(|| "sf".into());
        AreaDialog {
            open: true,
            props,
            factor_text,
            suffix_text,
            codes,
            positive: true,
        }
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Apply a control's change to the pending payload.
    pub fn update(&mut self, msg: AreaDialogMsg) {
        match msg {
            AreaDialogMsg::Code(code) => self.props.code = code,
            AreaDialogMsg::Name(name) => self.props.name = name,
            AreaDialogMsg::Factor(text) => {
                self.factor_text = text.clone();
                // The original coerces the field with `Number(...)`: an empty
                // field is 0, junk is NaN (which clears the override).
                self.props.factor = if text.trim().is_empty() {
                    Some(0.0)
                } else {
                    text.trim().parse::<f64>().ok()
                };
            }
            AreaDialogMsg::Positive(positive) => self.positive = positive,
            AreaDialogMsg::AutoSubtract(label) => self.props.auto_subtract = label == "Automatic",
            AreaDialogMsg::Measure(label) => {
                self.props.measure_to = Some(measure_value(&label).to_string())
            }
            AreaDialogMsg::Post(which, on) => {
                let post = self.props.auto_post.get_or_insert_with(Default::default);
                match which {
                    PostSwitch::Code => post.code = on,
                    PostSwitch::Name => post.name = on,
                    PostSwitch::Dims => post.dims = on,
                    PostSwitch::Calcs => post.calcs = on,
                    PostSwitch::Base => post.base = on,
                    PostSwitch::Factor => post.factor = on,
                }
            }
            AreaDialogMsg::Suffix(text) => {
                self.suffix_text = text.clone();
                let post = self.props.auto_post.get_or_insert_with(Default::default);
                post.suffix = text;
            }
            AreaDialogMsg::Apply | AreaDialogMsg::Cancel => {}
        }
    }

    /// The payload Apply sends.
    pub fn payload(&self) -> AreaProperties {
        let mut props = self.props.clone();
        if let Some(post) = props.auto_post.as_mut() {
            post.suffix = self.suffix_text.clone();
        }
        props
    }
}

fn field<'a>(
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    w: f32,
) -> Element<'a, Message> {
    text_input("", value)
        .on_input(on_input)
        .font(ARIAL)
        .size(13.33)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 8.0,
            right: 8.0,
        })
        .width(w)
        .style(|_, _| iced::widget::text_input::Style {
            background: Background::Color(bg_app()),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            },
            icon: muted_alt(),
            placeholder: muted_alt(),
            value: text_main(),
            selection: crate::dialog::SELECTION,
        })
        .into()
}

fn row_label<'a>(label: &'a str, y: f32) -> Element<'a, Message> {
    crate::theme::at(
        LABEL_X - CARD.0,
        y - CARD.1 + PIN_Y,
        nudge(
            0.5,
            0.0,
            text(label).font(ARIAL).size(12.5).color(muted_alt()),
        ),
    )
}

/// A group caption with the rule that runs to the card's right edge
/// ("Calculation Adjustments", "Automatic labels").
fn section<'a>(label: &'a str, y: f32) -> Element<'a, Message> {
    let caption = text(label).font(ARIAL).size(12.0).color(muted_alt());
    // The rule starts after the caption and runs to x 947 (12px inset).
    let rule_len = (947.0 - (LABEL_X + 8.0 + label.len() as f32 * 6.2)).max(0.0);
    let rule = container(Space::new())
        .width(rule_len)
        .height(1.0)
        .style(|_| container::Style {
            background: Some(Background::Color(border())),
            ..container::Style::default()
        });
    let line = iced::widget::row![caption, rule]
        .spacing(8.0)
        .align_y(iced::Alignment::Center);
    place(LABEL_X, y, line)
}

fn place<'a>(x: f32, y: f32, el: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    crate::theme::at(x - CARD.0, y - CARD.1 + PIN_Y, el)
}

/// The dialog's layer, or None when closed.
pub fn layer<'a>(d: &AreaDialog, codes: &'a [String]) -> Option<Element<'a, Message>> {
    if !d.open {
        return None;
    }
    let send = |m: AreaDialogMsg| Message::AreaDialog(m);
    let post = d.props.auto_post.clone().unwrap_or_default();

    let mut body = iced::widget::stack![]
        .width(Length::Fill)
        .height(Length::Fill);

    // Title.
    body = body.push(place(
        464.0,
        214.0,
        text("Area Properties")
            .font(ARIAL)
            .size(14.0)
            .color(text_main()),
    ));
    // Close ×.
    body = body.push(place(
        935.0,
        212.0,
        button(text("\u{d7}").font(ARIAL).size(16.0).color(muted_alt()))
            .on_press(send(AreaDialogMsg::Cancel))
            .padding(0)
            .width(12.0)
            .height(23.0)
            .style(|_, _| button::Style {
                background: None,
                border: Border::default(),
                text_color: muted_alt(),
                ..button::Style::default()
            }),
    ));

    // Area (code select).
    body = body.push(row_label("Area", 258.0));
    body = body.push(place(
        CONTROL_X,
        248.0,
        pick_list(
            codes.to_vec(),
            codes
                .iter()
                .find(|c| c.starts_with(&format!("{} - ", d.props.code)))
                .cloned(),
            |v: String| {
                Message::AreaDialog(AreaDialogMsg::Code(
                    v.split(" - ").next().unwrap_or("").to_string(),
                ))
            },
        )
        .width(CONTROL_W)
        .text_size(13.0)
        .font(ARIAL)
        .padding(Padding {
            top: 5.0,
            bottom: 5.0,
            left: 8.0,
            right: 8.0,
        })
        .style(pick_style),
    ));

    // Name.
    body = body.push(row_label("Name", 297.0));
    body = body.push(place(
        CONTROL_X,
        289.0,
        field(
            &d.props.name,
            move |v| send(AreaDialogMsg::Name(v)),
            CONTROL_W,
        ),
    ));

    // Group caption: the reference puts "Calculation Adjustments" at y 332.
    body = body.push(section("Calculation Adjustments", 328.0));

    // Factor.
    body = body.push(row_label("Factor", 361.0));
    body = body.push(place(
        NARROW_X,
        353.0,
        field(
            &d.factor_text,
            move |v| send(AreaDialogMsg::Factor(v)),
            NARROW_W,
        ),
    ));

    // Positive / Negative.
    body = body.push(place(
        594.0,
        387.0,
        radio("Positive", true, Some(d.positive), move |v| {
            send(AreaDialogMsg::Positive(v))
        })
        .text_size(12.0)
        .font(ARIAL)
        .size(13.0),
    ));
    body = body.push(place(
        672.0,
        387.0,
        radio("Negative", false, Some(!d.positive), move |v| {
            send(AreaDialogMsg::Positive(!v))
        })
        .text_size(12.0)
        .font(ARIAL)
        .size(13.0),
    ));

    // Auto-Subtract.
    body = body.push(row_label("Auto-Subtract", 422.0));
    body = body.push(place(
        NARROW_X,
        412.0,
        pick_list(
            vec!["Automatic".to_string(), "None".to_string()],
            Some(if d.props.auto_subtract {
                "Automatic".to_string()
            } else {
                "None".to_string()
            }),
            move |v: String| send(AreaDialogMsg::AutoSubtract(v)),
        )
        .width(NARROW_W)
        .text_size(13.0)
        .font(ARIAL)
        .style(pick_style),
    ));

    // Measure To.
    body = body.push(row_label("Measure To", 453.0));
    body = body.push(place(
        NARROW_X,
        443.0,
        pick_list(
            MEASURE.iter().map(|m| m.to_string()).collect::<Vec<_>>(),
            Some(measure_label(d.props.measure_to.as_deref().unwrap_or("center")).to_string()),
            move |v: String| send(AreaDialogMsg::Measure(v)),
        )
        .width(NARROW_W)
        .text_size(13.0)
        .font(ARIAL)
        .style(pick_style),
    ));

    // Group caption: "Automatic labels" at y 499.
    body = body.push(section("Automatic labels", 495.0));

    // The 3x2 auto-post grid.
    let boxes: [(PostSwitch, &str, f32, f32, bool); 6] = [
        (PostSwitch::Code, "Code", 464.0, 516.0, post.code),
        (PostSwitch::Name, "Name", 623.7, 516.0, post.name),
        (PostSwitch::Dims, "Dims", 783.3, 516.0, post.dims),
        (PostSwitch::Calcs, "Calcs", 464.0, 539.0, post.calcs),
        (PostSwitch::Base, "Base", 623.7, 539.0, post.base),
        (PostSwitch::Factor, "Factor", 783.3, 539.0, post.factor),
    ];
    for (which, label, x, y, on) in boxes {
        body = body.push(place(
            x,
            y,
            checkbox(on)
                .label(label)
                .on_toggle(move |v| send(AreaDialogMsg::Post(which, v)))
                .text_size(12.0)
                .font(ARIAL)
                .size(13.0),
        ));
    }

    // Suffix.
    body = body.push(row_label("Suffix", 576.0));
    body = body.push(place(
        NARROW_X,
        568.0,
        field(
            &d.suffix_text,
            move |v| send(AreaDialogMsg::Suffix(v)),
            NARROW_W,
        ),
    ));

    // Cancel / Apply.
    body = body.push(place(
        828.0,
        618.0,
        dialog_button("Cancel", false, send(AreaDialogMsg::Cancel)),
    ));
    body = body.push(place(
        895.0,
        618.0,
        dialog_button("Apply", true, send(AreaDialogMsg::Apply)),
    ));

    // No container padding: every child is placed at its measured absolute
    // position (a padded container would shift the whole pinned stack, the
    // same pin-origin quirk the command popup documents).
    let card = container(body)
        .width(CARD.2)
        .height(CARD.3)
        .style(|_| container::Style {
            background: Some(Background::Color(bg_panel())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 5.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: Vector::new(0.0, 16.0),
                blur_radius: 23.0,
            },
            ..container::Style::default()
        });
    // The backdrop catches presses outside the card (Escape-equivalent).
    let backdrop = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.46))),
            ..container::Style::default()
        });
    Some(
        iced::widget::stack![
            iced::widget::opaque(
                iced::widget::mouse_area(backdrop).on_press(send(AreaDialogMsg::Cancel))
            ),
            crate::theme::at(CARD.0, CARD.1, card)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    )
}

fn pick_style(_: &iced::Theme, _: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: text_main(),
        placeholder_color: muted_alt(),
        handle_color: muted_alt(),
        background: Background::Color(bg_app()),
        border: Border {
            color: border(),
            width: 1.0,
            radius: 2.0.into(),
        },
    }
}

/// The dialog's action buttons: both are `#3E3E42` with `#ccc` text in the
/// original — this dialog has no accent primary.
fn dialog_button<'a>(label: &'a str, _primary: bool, on_press: Message) -> Element<'a, Message> {
    button(text(label).font(ARIAL).size(13.33).color(text_main()))
        .on_press(on_press)
        .padding(Padding {
            top: 5.0,
            bottom: 5.0,
            left: 12.0,
            right: 12.0,
        })
        .height(27.0)
        .style(move |_, _| button::Style {
            background: Some(Background::Color(bg_hover())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            },
            text_color: text_main(),
            ..button::Style::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dialog() -> AreaDialog {
        AreaDialog::open_with(
            AreaProperties {
                code: "GLA1".into(),
                name: "First Floor".into(),
                factor: Some(1.0),
                auto_subtract: true,
                measure_to: Some("center".into()),
                auto_post: Some(Default::default()),
            },
            vec!["GLA1 - First Floor".into(), "GLA2 - Second Floor".into()],
        )
    }

    #[test]
    fn the_dialog_opens_with_the_areas_values() {
        let d = dialog();
        assert!(d.open);
        assert_eq!(d.props.code, "GLA1");
        assert_eq!(d.factor_text, "1", "a whole factor prints without decimals");
        assert_eq!(d.suffix_text, "sf");
        assert!(d.positive);
    }

    #[test]
    fn an_empty_factor_field_is_zero_as_in_the_original() {
        let mut d = dialog();
        d.update(AreaDialogMsg::Factor(String::new()));
        assert_eq!(d.props.factor, Some(0.0), "Number('') === 0");
        d.update(AreaDialogMsg::Factor("abc".into()));
        assert_eq!(d.props.factor, None, "junk clears the override");
        d.update(AreaDialogMsg::Factor("1.25".into()));
        assert_eq!(d.props.factor, Some(1.25));
        assert_eq!(d.payload().factor, Some(1.25));
    }

    #[test]
    fn the_controls_reach_the_payload() {
        let mut d = dialog();
        d.update(AreaDialogMsg::Code("GLA2".into()));
        d.update(AreaDialogMsg::Name("Second Floor".into()));
        d.update(AreaDialogMsg::AutoSubtract("None".into()));
        d.update(AreaDialogMsg::Measure("Interior Face".into()));
        d.update(AreaDialogMsg::Post(PostSwitch::Code, true));
        d.update(AreaDialogMsg::Post(PostSwitch::Dims, false));
        d.update(AreaDialogMsg::Suffix("SF".into()));
        let p = d.payload();
        assert_eq!(p.code, "GLA2");
        assert_eq!(p.name, "Second Floor");
        assert!(!p.auto_subtract, "'None' is autoSubtract false");
        assert_eq!(p.measure_to.as_deref(), Some("interior"));
        let post = p.auto_post.expect("autoPost");
        assert!(post.code);
        assert!(!post.dims);
        assert_eq!(post.suffix, "SF");
        // The radio is display-only in the original.
        d.update(AreaDialogMsg::Positive(false));
        assert!(!d.positive);
        assert_eq!(d.payload().code, "GLA2");
    }
}
