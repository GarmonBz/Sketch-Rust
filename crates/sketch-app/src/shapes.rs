//! The wall-shape menu (UIManager.showCommandPopup + showWallShapeMenu,
//! theme.css `.command-popup`): a 190px popup under the Draw Exterior /
//! Draw Interior rail button (its caret, or a right-click) offering Line /
//! Rectangle / Circle / Polygon. The shapes prompt for their size, chaining
//! "Width in feet" (20) -> "Height in feet" (20) -> "Number of sides" (8),
//! and then call `addWallShape`.
//!
//! Measured from the reference: the popup is 190x128 at (8, 317) for the
//! exterior button (anchor bottom + 2), padding 5, 1px border, shadow
//! `0 8px 22px rgba(0,0,0,.42)`, rows 29.5px tall with a 16px icon at x=24
//! and the label 9px after it, hover `--bg-hover`.

use iced::widget::{button, container, mouse_area, row, stack, Space};
use iced::{alignment, Background, Border, Color, Element, Length, Padding, Shadow, Vector};

use crate::drawing::WallShape;
use crate::icons::icon;
use crate::snap::nudge;
use crate::theme::*;
use sketch_core::model::WallType;

/// `.command-popup`'s row height (measured pitch).
const ROW_H: f32 = 29.0;
/// `.command-popup { padding: 5px }` — the horizontal inset.
const PAD: f32 = 5.0;
/// Where the first row's top sits inside the panel. CSS puts it at
/// `border + padding` = 6px, but a `pin`ed stack inside the panel renders 7px
/// higher than its content box, so this offset is calibrated against the
/// original's own screenshot (row text lands on the reference pixel rows).
pub(crate) const ROW_Y0: f32 = 12.0;

/// The popup's page position for a rail tool (`anchor.bottom + 2`, left 8).
pub fn popup_origin(tool_y: f32) -> Vector {
    Vector::new(8.0, tool_y + 34.0)
}

/// Which prompt the chain is asking for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prompt {
    Width,
    Height,
    Sides,
}

impl Prompt {
    /// `shape === 'circle' ? 'Diameter' : 'Width'`.
    pub fn title(self, shape: WallShape) -> &'static str {
        match self {
            Prompt::Width if shape == WallShape::Circle => "Diameter in feet",
            Prompt::Width => "Width in feet",
            Prompt::Height => "Height in feet",
            Prompt::Sides => "Number of sides",
        }
    }

    pub fn default_value(self, shape: WallShape) -> &'static str {
        match self {
            Prompt::Width | Prompt::Height => "20",
            Prompt::Sides if shape == WallShape::Circle => "36",
            Prompt::Sides => "8",
        }
    }
}

/// A shape's size as it is being entered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pending {
    pub shape: WallShape,
    pub kind: WallType,
    pub width: f64,
    pub height: Option<f64>,
    /// The typed side count (polygon only).
    pub sides: Option<f64>,
    pub prompt: Prompt,
}

/// The menu and its prompt chain.
#[derive(Debug, Clone, Default)]
pub struct Shapes {
    /// The open popup: which wall type, and its page position.
    pub open: Option<(WallType, Vector)>,
    /// The prompt chain in progress.
    pub pending: Option<Pending>,
}

/// What the menu needs from the app.
pub struct ShapesView<'a> {
    pub shapes: &'a Shapes,
}

#[derive(Debug, Clone)]
pub enum ShapesMsg {
    /// The caret (or a right-click) on a rail tool at `anchor_y`.
    Open(WallType, f32),
    /// The Line row: plain drawing in that wall type.
    Line(WallType),
    /// A shape row.
    Choose(WallShape),
    /// A press outside the popup.
    Dismiss,
    /// A prompt's OK, with the typed value.
    Value(String),
}

impl Shapes {
    pub fn open_at(&mut self, kind: WallType, anchor_y: f32) {
        self.open = Some((kind, popup_origin(anchor_y)));
    }

    pub fn dismiss(&mut self) {
        self.open = None;
        self.pending = None;
    }

    /// A shape row: start the prompt chain (Line is handled by the caller,
    /// which just sets the mode).
    pub fn choose(&mut self, shape: WallShape, kind: WallType) {
        self.open = None;
        self.pending = Some(Pending {
            shape,
            kind,
            width: 20.0,
            height: None,
            sides: None,
            prompt: Prompt::Width,
        });
    }

    /// The prompt on screen (`title`, `default`).
    pub fn prompt(&self) -> Option<(&'static str, &'static str)> {
        self.pending
            .as_ref()
            .map(|p| (p.prompt.title(p.shape), p.prompt.default_value(p.shape)))
    }

    /// A prompt's OK: advance the chain, and on the last step report the
    /// shape to add. `Number(await promptDialog(...))` — a falsy entry
    /// abandons the shape.
    pub fn submit(&mut self, value: &str) -> Option<Pending> {
        let pending = self.pending?;
        let number = value
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|n| *n != 0.0 && n.is_finite());
        let Some(number) = number else {
            self.pending = None;
            return None;
        };
        match pending.prompt {
            // A circle takes its diameter once: it is also the height, and
            // its side count is fixed at 36.
            Prompt::Width if pending.shape == WallShape::Circle => {
                self.pending = None;
                Some(Pending {
                    shape: pending.shape,
                    kind: pending.kind,
                    width: number,
                    height: Some(number),
                    sides: None,
                    prompt: Prompt::Sides,
                })
            }
            Prompt::Width => {
                self.pending = Some(Pending {
                    width: number,
                    prompt: Prompt::Height,
                    ..pending
                });
                None
            }
            // Only a polygon asks for its side count; a rectangle finishes
            // after the height.
            Prompt::Height if pending.shape == WallShape::Polygon => {
                self.pending = Some(Pending {
                    height: Some(number),
                    prompt: Prompt::Sides,
                    ..pending
                });
                None
            }
            Prompt::Height => {
                self.pending = None;
                Some(Pending {
                    height: Some(number),
                    ..pending
                })
            }
            Prompt::Sides => {
                self.pending = None;
                Some(Pending {
                    height: Some(pending.height.unwrap_or(pending.width)),
                    sides: Some(number),
                    ..pending
                })
            }
        }
    }
}

fn menu_row<'a>(text: &str, glyph: &str, on_press: crate::Message) -> Element<'a, crate::Message> {
    menu_row_sized(text, glyph, 13.33, on_press)
}

/// A command-popup row at an explicit label size (the undo history's rows are
/// 11.5px, `.command-popup.undo-history-popup button span`).
pub(crate) fn menu_row_sized<'a>(
    text: &str,
    glyph: &str,
    size: f32,
    on_press: crate::Message,
) -> Element<'a, crate::Message> {
    let body = row![
        // `.command-popup button i`: 13.33px, the same size as the label.
        icon(glyph, false, 13.34, muted()),
        // The label sits at a fractional x inside the row; nudge it onto the
        // pixel like the rest of the chrome's text.
        nudge(0.5, 0.0, label(text, size, ARIAL, text_main(), 16.0)),
    ]
    .spacing(9)
    .align_y(alignment::Vertical::Center);
    button(body)
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: 10.0,
            right: 10.0,
        })
        .width(Length::Fill)
        .height(ROW_H)
        .on_press(on_press)
        .style(|_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: hovered.then_some(Background::Color(bg_hover())),
                border: Border::default(),
                text_color: text_main(),
                ..button::Style::default()
            }
        })
        .into()
}

/// The popup layer (`position:fixed; z-index:3000`), or None when closed.
pub fn layer<'a>(v: &ShapesView<'a>) -> Option<Element<'a, crate::Message>> {
    let (kind, at) = v.shapes.open?;
    let rows = stack![
        crate::theme::at(
            PAD,
            ROW_Y0,
            menu_row(
                "Line",
                "line-segment",
                crate::Message::Shapes(ShapesMsg::Line(kind))
            )
        ),
        crate::theme::at(
            PAD,
            ROW_Y0 + ROW_H,
            menu_row(
                "Rectangle",
                "rectangle",
                crate::Message::Shapes(ShapesMsg::Choose(WallShape::Rectangle))
            )
        ),
        crate::theme::at(
            PAD,
            ROW_Y0 + ROW_H * 2.0,
            menu_row(
                "Circle",
                "circle",
                crate::Message::Shapes(ShapesMsg::Choose(WallShape::Circle))
            )
        ),
        crate::theme::at(
            PAD,
            ROW_Y0 + ROW_H * 3.0,
            menu_row(
                "Polygon",
                "polygon",
                crate::Message::Shapes(ShapesMsg::Choose(WallShape::Polygon))
            )
        ),
    ]
    .width(Length::Fill)
    .height(Length::Fill);
    let panel = container(rows)
        .width(190.0)
        .height(128.0)
        .style(|_| container::Style {
            background: Some(Background::Color(bg_panel())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.42),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 11.0,
            },
            ..container::Style::default()
        });
    // A press outside the popup dismisses it; the panel sits on top.
    let backdrop = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(crate::Message::Shapes(ShapesMsg::Dismiss));
    Some(
        stack![backdrop, crate::theme::at(at.x, at.y, panel)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prompt_chain_matches_the_original() {
        // showWallShapeMenu: Rectangle asks Width -> Height (both default
        // 20); Circle asks Diameter once (its height is the diameter);
        // Polygon asks Width -> Height -> Number of sides (8).
        let mut s = Shapes::default();
        s.choose(WallShape::Rectangle, WallType::Exterior);
        assert_eq!(s.prompt(), Some(("Width in feet", "20")));
        assert!(s.submit("20").is_none());
        assert_eq!(s.prompt(), Some(("Height in feet", "20")));
        let done = s
            .submit("20")
            .expect("a rectangle completes after the height");
        assert_eq!(
            (done.shape, done.kind, done.width, done.height),
            (WallShape::Rectangle, WallType::Exterior, 20.0, Some(20.0))
        );

        let mut c = Shapes::default();
        c.choose(WallShape::Circle, WallType::Interior);
        assert_eq!(c.prompt(), Some(("Diameter in feet", "20")));
        let done = c.submit("10").expect("a circle completes after one prompt");
        assert_eq!(
            (done.width, done.height),
            (10.0, Some(10.0)),
            "the diameter is the height"
        );

        // A polygon is the only shape that asks for sides.
        let mut p = Shapes::default();
        p.choose(WallShape::Polygon, WallType::Exterior);
        assert!(p.submit("12").is_none());
        assert!(
            p.submit("8").is_none(),
            "the polygon continues to the sides"
        );
        assert_eq!(p.prompt(), Some(("Number of sides", "8")));
        let done = p.submit("6").expect("polygon completes");
        assert_eq!(
            (done.width, done.height, done.sides),
            (12.0, Some(8.0), Some(6.0))
        );

        // A falsy or unparsable entry abandons the shape.
        let mut a = Shapes::default();
        a.choose(WallShape::Rectangle, WallType::Exterior);
        assert!(a.submit("0").is_none() && a.prompt().is_none());
        let mut b = Shapes::default();
        b.choose(WallShape::Rectangle, WallType::Exterior);
        assert!(b.submit("").is_none() && b.prompt().is_none());
    }
}
