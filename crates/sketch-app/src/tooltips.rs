//! Hover tooltips: the original's `title` attributes (refs/sidebar-redesign.txt
//! markup), shown the way the browser shows them - a small system-styled box
//! at the pointer after a short hover delay. Only the controls the original
//! titles get one: every rail tool, the quick-access and window buttons, a
//! handful of ribbon buttons, and the status bar's icon buttons.

use std::time::Duration;

use iced::widget::{container, text, tooltip};
use iced::{Background, Border, Color, Element};

use crate::rail::RailTool;
use crate::ribbon::Body;
use crate::theme::*;
use crate::Message;

/// A browser tooltip appears after about half a second of hovering.
const DELAY: Duration = Duration::from_millis(500);

/// `content` with the tooltip `title` at the pointer (an empty title adds
/// nothing). iced puts a following tooltip above the pointer, which suits
/// the status bar along the window's bottom edge.
pub fn tip<'a>(
    content: impl Into<Element<'a, Message>>,
    title: impl Into<String>,
) -> Element<'a, Message> {
    tip_at(content, title, tooltip::Position::FollowCursor)
}

/// `tip` below the control (the ribbon, title bar).
pub fn tip_below<'a>(
    content: impl Into<Element<'a, Message>>,
    title: impl Into<String>,
) -> Element<'a, Message> {
    tip_at(content, title, tooltip::Position::Bottom)
}

/// `tip` beside the control (the rail).
pub fn tip_right<'a>(
    content: impl Into<Element<'a, Message>>,
    title: impl Into<String>,
) -> Element<'a, Message> {
    tip_at(content, title, tooltip::Position::Right)
}

fn tip_at<'a>(
    content: impl Into<Element<'a, Message>>,
    title: impl Into<String>,
    position: tooltip::Position,
) -> Element<'a, Message> {
    let content = content.into();
    let title: String = title.into();
    if title.is_empty() {
        return content;
    }
    let body = container(text(title).font(SEGOE).size(12.0).color(Color::BLACK))
        .padding([2, 6])
        .style(|_| container::Style {
            // Chrome on Windows: an off-white box with a thin grey border.
            background: Some(Background::Color(hex(0xF2F2F2))),
            border: Border {
                color: hex(0x767676),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });
    tooltip(content, body, position).gap(4).delay(DELAY).into()
}

/// `#tool-<id>`'s `title`.
pub fn rail_title(tool: RailTool) -> &'static str {
    match tool {
        RailTool::Select => "Select/Edit",
        RailTool::Fence => "Fence",
        RailTool::Lasso => "Lasso",
        RailTool::Pan => "Pan",
        RailTool::DrawExterior => "Draw Exterior",
        RailTool::DrawInterior => "Draw Interior",
        RailTool::InsertVertex => "Insert Vertex",
        RailTool::Curve => "Curve Wall",
        RailTool::Measure => "Measure: chain legs, close for area, Shift locks 45\u{b0}",
        RailTool::ChainDim => "Chain Dimensions",
        RailTool::Shapes => "Insert Room Shape",
        RailTool::DefineArea => "Define Areas: click inside an outline",
        RailTool::Text => "Text Library",
        RailTool::Symbols => "Symbol Library",
        RailTool::Touch => "Touch Panel",
        RailTool::Geo => "Sketch Geo-Referencing",
        RailTool::Pages => "Pages navigator",
    }
}

/// A ribbon button's `title`, by the caption it shows (the original titles
/// only these; the rest carry no tooltip).
pub fn ribbon_title(body: Body, caption: &str) -> &'static str {
    let caption = caption.replace('\n', " ");
    match (body, caption.as_str()) {
        (Body::Home | Body::EditTools, "Undo") => "Undo (Ctrl+Z)",
        (Body::Home | Body::EditTools, "Redo") => "Redo (Ctrl+Y)",
        (Body::View, "Background") => "Choose the drawing background: White, Dark, or Blue",
        (Body::View, "Interface") => "Switch the interface between Light, Dark, and Blue",
        (_, "Keyboard Shortcuts") => "Keyboard Shortcuts (F1)",
        (Body::Help, "User Guide") => "User Guide: how to draw, define areas, print and save",
        (Body::Help, "Integration Guide") => "How to set up and use Appraise-It integration",
        (Body::Help, "About") => "About Sketch Rust: version of this build",
        (Body::EditTools, "Bold") => "Bold",
        (Body::EditTools, "Italic") => "Italic",
        (Body::EditTools, "Underline") => "Underline",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rail_tool_has_the_originals_title() {
        use RailTool::*;
        for tool in [
            Select,
            Fence,
            Lasso,
            Pan,
            DrawExterior,
            DrawInterior,
            InsertVertex,
            Curve,
            Measure,
            ChainDim,
            Shapes,
            DefineArea,
            Text,
            Symbols,
            Touch,
            Geo,
            Pages,
        ] {
            assert!(!rail_title(tool).is_empty());
        }
        assert_eq!(rail_title(Select), "Select/Edit");
        assert!(rail_title(Measure).ends_with("45\u{b0}"));
    }

    #[test]
    fn only_the_originals_titled_ribbon_buttons_get_a_tooltip() {
        assert_eq!(ribbon_title(Body::Home, "Undo"), "Undo (Ctrl+Z)");
        assert_eq!(ribbon_title(Body::EditTools, "Redo"), "Redo (Ctrl+Y)");
        assert_eq!(
            ribbon_title(Body::Help, "Keyboard\nShortcuts"),
            "Keyboard Shortcuts (F1)"
        );
        assert_eq!(ribbon_title(Body::Home, "Cut"), "");
        assert_eq!(ribbon_title(Body::Home, "Bold"), "");
    }
}
