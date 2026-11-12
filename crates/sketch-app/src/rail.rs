//! Left tool rail (0,129 40x703, `--bg-panel`, 1px `--border` right):
//! 17 `.tool-btn`s (32x32 at x=4, 2px apart) in five groups; groups are
//! separated by a 14px gap carrying a 1px `--border` line 8px above the
//! padding box of the group's first button, inset 8px each side (i.e. 7px
//! above / 9px inside its border box). The active mode's tool carries
//! `.active`: accent border, 20% accent fill, white text (inherited-colour
//! icons turn white; icons with their own colour keep it).

use iced::widget::{button, container, stack};
use iced::{Background, Border, Color, Element, Length};

use crate::icons::{icon, rotated};
use crate::theme::*;
use crate::view_state::Mode;
use crate::Message;
use sketch_core::model::WallType;

/// Rail origin on the page.
const TOP: f32 = 129.0;

/// `.tool-btn` ids, top to bottom (`#tool-<id>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailTool {
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
}

impl RailTool {
    /// The mode whose `#tool-<mode>` button this is (setMode highlights it).
    pub fn mode(self) -> Option<Mode> {
        Some(match self {
            RailTool::Select => Mode::Select,
            RailTool::DefineArea => Mode::DefineArea,
            RailTool::Fence => Mode::Fence,
            RailTool::Lasso => Mode::Lasso,
            RailTool::Pan => Mode::Pan,
            RailTool::DrawExterior => Mode::DrawExterior,
            RailTool::DrawInterior => Mode::DrawInterior,
            RailTool::InsertVertex => Mode::InsertVertex,
            RailTool::Curve => Mode::Curve,
            RailTool::Measure => Mode::Measure,
            RailTool::ChainDim => Mode::ChainDim,
            _ => return None,
        })
    }
}

struct Tool {
    id: RailTool,
    y: f32,
    /// Icon and its own colour; `None` colour = inherits the button's text
    /// colour (`--text-main`, white while `.active`).
    icon: Option<(&'static str, Option<Color>)>,
    /// Draw Exterior / Interior: pencil badge + shape caret.
    draw_badge: bool,
    /// The wall type whose shape menu the caret opens (the two draw tools).
    caret: Option<WallType>,
    active: bool,
}

const fn tool(id: RailTool, y: f32, name: &'static str, color: Option<Color>) -> Tool {
    Tool {
        id,
        y,
        icon: Some((name, color)),
        draw_badge: false,
        caret: None,
        active: false,
    }
}

fn tools() -> Vec<Tool> {
    use RailTool::*;
    vec![
        tool(Select, 133.0, "cursor", None),
        tool(Fence, 167.0, "selection-background", None),
        tool(Lasso, 201.0, "lasso", None),
        tool(Pan, 235.0, "arrows-out-cardinal", None),
        Tool {
            draw_badge: true,
            caret: Some(WallType::Exterior),
            ..tool(DrawExterior, 283.0, "polygon", Some(ICON_BLUE))
        },
        Tool {
            draw_badge: true,
            caret: Some(WallType::Interior),
            ..tool(
                DrawInterior,
                317.0,
                "square-split-vertical",
                Some(ICON_SLATE),
            )
        },
        tool(InsertVertex, 351.0, "line-segments", None),
        tool(Curve, 385.0, "bezier-curve", None),
        tool(Measure, 433.0, "ruler", None),
        tool(ChainDim, 467.0, "arrows-horizontal", None),
        tool(Shapes, 515.0, "square", None),
        tool(DefineArea, 549.0, "bounding-box", Some(ICON_RED)),
        tool(Text, 583.0, "text-t", None),
        tool(Symbols, 617.0, "couch", None),
        tool(Touch, 665.0, "hand-pointing", Some(ICON_RED)),
        tool(Geo, 699.0, "globe-hemisphere-west", Some(ICON_BLUE)),
        // Pages navigator: its `ph-files` glyph is absent from the
        // original's icon font subset, so the button renders empty there.
        Tool {
            id: Pages,
            y: 733.0,
            icon: None,
            draw_badge: false,
            caret: None,
            active: false,
        },
    ]
}

fn tool_button<'a>(t: &Tool) -> Element<'a, Message> {
    let mut body = stack![].width(32).height(32);
    if let Some((name, color)) = t.icon {
        let inherited = if t.active { WHITE } else { text_main() };
        body = body.push(at(
            7.0,
            7.0,
            icon(name, false, 18.0, color.unwrap_or(inherited)),
        ));
    }
    if t.draw_badge {
        // 12px fill pencil, right/bottom -4px of the 18px icon box,
        // rotated -45deg about its center.
        body = body
            .push(at(
                17.0,
                17.0,
                rotated("pencil-simple", true, 12.0, ICON_ORANGE, -45.0),
            ))
            .push(at(12.0, 23.0, label("▸", 8.0, SYMBOL, muted(), 8.0)));
    }
    // The caret (`.tool-caret`) opens the wall-shape menu instead of the
    // tool: the original stops the click's propagation. It sits on top of
    // the button's own press area, so it must be added last.
    if let Some(kind) = t.caret {
        // The mouse area must be INSIDE the pin: a pin's own bounds are the
        // whole parent, so wrapping it would swallow the button's presses.
        body = body.push(at(
            11.0,
            22.0,
            iced::widget::mouse_area(iced::widget::container(
                iced::widget::Space::new().width(9.0).height(9.0),
            ))
            .on_press(Message::Shapes(crate::shapes::ShapesMsg::Open(kind, t.y))),
        ));
    }
    let active = t.active;
    let title = crate::tooltips::rail_title(t.id);
    crate::tooltips::tip_right(
        button(body)
            .width(32)
            .height(32)
            .padding(0)
            .on_press(Message::Rail(t.id))
            .style(move |_, status| {
                if active {
                    button::Style {
                        background: Some(Background::Color(over(accent(), 0.2, bg_panel()))),
                        border: Border {
                            color: accent(),
                            width: 1.0,
                            radius: 0.0.into(),
                        },
                        ..button::Style::default()
                    }
                } else {
                    button::Style {
                        background: match status {
                            button::Status::Hovered | button::Status::Pressed => {
                                Some(bg_hover().into())
                            }
                            _ => None,
                        },
                        ..button::Style::default()
                    }
                }
            }),
        title,
    )
}

/// `mode`: setMode puts `.active` on `#tool-<mode>` (none for modes without
/// a rail tool, e.g. the zoom fence).
pub fn view<'a>(mode: Mode, pages_open: bool) -> Element<'a, Message> {
    let mut layers = stack![at(39.0, 0.0, rect_fill_h(1.0, border()))]
        .width(40)
        .height(Length::Fill);
    for y in [283.0, 433.0, 515.0, 665.0] {
        layers = layers.push(at(13.0, y - 7.0 - TOP, rect(14.0, 1.0, border())));
    }
    for mut t in tools() {
        // `body.pages-nav-open .tool-btn#tool-pages` is highlighted too.
        t.active = t.id.mode() == Some(mode) || (t.id == RailTool::Pages && pages_open);
        layers = layers.push(at(4.0, t.y - TOP, tool_button(&t)));
    }
    fill_bg(layers, bg_panel())
        .width(40)
        .height(Length::Fill)
        .into()
}

/// A full-height vertical bar `w` wide.
fn rect_fill_h<'a>(w: f32, color: Color) -> Element<'a, Message> {
    container(iced::widget::Space::new().width(w).height(Length::Fill))
        .style(move |_| container::Style {
            background: Some(Background::Color(color)),
            ..container::Style::default()
        })
        .into()
}
