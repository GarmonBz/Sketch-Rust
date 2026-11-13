//! The in-app User Guide (Help > User Guide / Appraise-It Reports).
//!
//! Not in the original web app: a topic list on the left, the chosen topic's
//! text on the right, in the same card/backdrop dress as the Keyboard
//! Shortcuts sheet. The text is static and lives in this file so it ships
//! inside the exe and always matches the build.

use iced::widget::{button, column, container, mouse_area, opaque, row, scrollable, text, Space};
use iced::{Background, Border, Color, Element, Length, Task};

use crate::theme::*;
use crate::Message;

#[cfg(windows)]
use crate::appraise_it::{self, AreaTarget, AreaTransfer, Report, ReportStatus, UnitAreaTotals};
#[cfg(windows)]
use iced::widget::image::Handle;

/// One piece of a topic's body.
pub enum Block {
    /// A section heading inside the topic.
    Heading(&'static str),
    /// A paragraph.
    Para(&'static str),
    /// Numbered steps, done in order.
    Steps(&'static [&'static str]),
    /// Unordered points.
    Points(&'static [&'static str]),
    /// A caution or limit, set apart from the body text.
    Note(&'static str),
}

pub struct Topic {
    pub title: &'static str,
    pub blocks: &'static [Block],
}

/// The index of the Appraise-It topic, opened directly by Help >
/// Appraise-It Reports.
pub const INTEGRATION_TOPIC: usize = 6;

use Block::{Heading, Note, Para, Points, Steps};

pub const TOPICS: &[Topic] = &[
    Topic {
        title: "Getting started",
        blocks: &[
            Para("Sketch draws floor plans to scale and calculates their areas. A sketch is made of walls, the areas they enclose, and symbols and text placed on top."),
            Heading("The window"),
            Points(&[
                "Title bar: Save, Undo, Redo and the command search, with a pill that reads Unsaved or Saved. The caret opens New, Open, Save As and Options.",
                "Tabs: File opens the file menu. Home, View, Tools, Reference Image and Help each show a ribbon of commands. Edit Tools appears while something is selected.",
                "Left rail: the tools - Select, drawing walls, Measure and the rest. Hover a tool to see its name.",
                "Right dock: panels for defining areas, calculations, symbols, text and layers.",
                "Status bar: the current page, zoom and live totals.",
            ]),
            Heading("A first sketch"),
            Steps(&[
                "Press E to draw exterior walls.",
                "Click a starting point, then type a length and press an arrow key for the direction - for example 40 then Right arrow draws 40 feet to the right.",
                "Carry on around the building. Press A to close the shape back to the start.",
                "Choose what the area is (First Floor, Garage and so on) in the Define Area panel (F4).",
                "Open the Calculations panel (F6) to see the totals.",
                "Save with Ctrl + S.",
            ]),
            Para("Help > Keyboard Shortcuts (F1) lists every key."),
        ],
    },
    Topic {
        title: "Drawing walls",
        blocks: &[
            Para("Press E for exterior walls or I for interior walls. Walls can be drawn with the mouse, the keyboard, or both."),
            Heading("With the mouse"),
            Points(&[
                "Click to set each corner. The wall preview follows the pointer and snaps to existing corners.",
                "Dashed guide lines appear when the pointer lines up with another corner.",
                "Press Esc to stop drawing.",
            ]),
            Heading("With the keyboard"),
            Points(&[
                "Type a length, then press an arrow key: 12 then Up arrow draws 12 feet up.",
                "10x5 draws a diagonal that rises 10 and runs 5.",
                "10<45 draws 10 feet at 45 degrees.",
                "10l or 10r turns left or right from the last wall (90 degrees unless you give an angle).",
                "Tab types a decimal point; Backspace erases the last character typed.",
                "A closes the area back to its starting point. J jumps the cursor to the nearest corner.",
            ]),
            Heading("Changing walls afterwards"),
            Points(&[
                "Press P for the Select tool, then click a wall or area. The Edit Tools tab shows what can be changed: length, line style, thickness and color.",
                "Drag a corner to move it. Insert adds a corner in the middle of a wall.",
                "Arrow keys nudge the selection; Delete removes it; Ctrl + Z undoes.",
            ]),
        ],
    },
    Topic {
        title: "Areas and calculations",
        blocks: &[
            Para("A closed run of exterior walls becomes an area. Each area has an area code - such as First Floor, Second Floor, Garage or Porch - that decides how it counts in the totals."),
            Heading("Defining an area"),
            Steps(&[
                "Open the Define Area panel (F4).",
                "Pick the area code, either before you draw or afterwards by dragging the code onto a drawn area.",
                "The area takes the code's name, color and fill, and joins the calculations.",
            ]),
            Heading("Reading the numbers"),
            Points(&[
                "The Calculations panel (F6) lists each area with its size and perimeter, grouped by code, with the living-area total.",
                "Tools > Area Breakdown shows how each area was calculated, piece by piece.",
                "Tools > Area Codes edits the list of codes: names, colors, and whether a code counts toward living area.",
            ]),
            Heading("Subtracting one area from another"),
            Para("Select an area and switch on Edit Tools > Auto Subtract to take it out of the area around it - for example an open-to-below space inside a second floor."),
            Note("An unclosed run of walls is not an area and is left out of the totals. Close it with A, or by clicking back on its first corner."),
        ],
    },
    Topic {
        title: "Symbols and text",
        blocks: &[
            Heading("Symbols"),
            Points(&[
                "Press S to open the symbol library. Pick a category, or search by name.",
                "Click a symbol, then click the drawing to place it. You can also drag it from the panel onto the drawing.",
                "Doors and windows snap onto a wall and cut an opening in it. For a door, the side of the wall you click on is the side it swings to.",
                "Select a placed symbol to move, rotate or resize it with its handles. Resizing a door or window resizes its opening too.",
                "Manage symbols chooses your Favorites and renames, hides, moves and deletes library symbols.",
            ]),
            Heading("Text"),
            Points(&[
                "Press T to open the text library, pick a label and click the drawing to place it.",
                "Select a label to change its wording, font, size and color on the Edit Tools tab.",
                "Drag the handle above a selected label to rotate it.",
            ]),
        ],
    },
    Topic {
        title: "Pages, printing and export",
        blocks: &[
            Heading("Pages"),
            Para("A sketch can have several pages, for example one per floor. Use the page buttons on the View tab or the status bar to add, delete and move between pages."),
            Heading("Printing"),
            Points(&[
                "File > Print Preview shows the pages as they will print, with the area summary and breakdown tables beneath the drawing.",
                "File > Page Layout sets the paper size and what is printed.",
                "File > Print sends the previewed pages to the printer.",
            ]),
            Heading("Export and import"),
            Points(&[
                "File > Export as PDF, PNG, SVG or DXF writes the drawing in that format.",
                "File > Email opens a message to support in your mail program.",
                "File > Import CAD brings in a DXF or DWG drawing.",
                "The Reference Image tab loads a picture - a scan or an old sketch - behind the drawing so it can be traced. Set Scale matches it to real measurements.",
            ]),
        ],
    },
    Topic {
        title: "Saving and recovery",
        blocks: &[
            Points(&[
                "Ctrl + S saves to the current file. The first save asks where to put it; Save As (F12) saves a copy under a new name and carries on in that copy.",
                "The pill in the title bar reads Unsaved while there are changes that are not in the file. Closing with unsaved changes asks whether to save them.",
                "File > Recent Files reopens a sketch you worked on lately.",
                "Drop a sketch file on the window to open it. Each window holds one sketch; opening a sketch that is already open brings its window forward.",
            ]),
            Heading("Autosave"),
            Para("While you work, Sketch keeps a recovery copy of the open sketch. If Sketch closes unexpectedly, the next start offers to restore it; File > Recover Autosave does the same at any time. Autosave is switched on, and its interval set, in Options."),
            Note("The recovery copy is a safety net, not a save. It is removed once you save or deliberately discard your changes."),
        ],
    },
    Topic {
        title: "Appraise-It reports",
        blocks: &[
            Para("Keep the editable drawing in a Sketchy or SKX file. On Windows, use the controls above to send sketch pages to UAD 3.6 reports and legacy reports that support Sketch addenda."),
            Heading("Export a sketch for your report"),
            Steps(&[
                "Use File > Export as PNG for a sketch image or File > Export as PDF for a document with calculations.",
                "Attach the image or PDF using the controls available for your report type in Appraise-It Pro. Review the sketch and enter the area totals in the appropriate report fields.",
            ]),
            Heading("If you used a previous preview with Appraise-It Pro"),
            Para("Ask SFREP or Apex support to restore the original sketch provider if the report button still points to Sketch Rust. Open existing AXF workfiles in the licensed Apex app."),
        ],
    },
];

/// The guide overlay's state.
#[derive(Debug, Clone, Default)]
pub struct Guide {
    pub open: bool,
    pub topic: usize,
    #[cfg(windows)]
    report_busy: bool,
    #[cfg(windows)]
    report_status: Option<String>,
    #[cfg(windows)]
    report_target: Option<Report>,
    #[cfg(windows)]
    sketch_preview: Vec<(usize, Handle)>,
    #[cfg(windows)]
    area_target: Option<Result<AreaTarget, String>>,
    #[cfg(windows)]
    area_preview: Option<Result<AreaTransfer, String>>,
    #[cfg(windows)]
    include_areas: bool,
}

#[derive(Debug, Clone)]
pub enum GuideMsg {
    /// Open at a topic (the Help tab's two buttons).
    Open(usize),
    /// Switch topic from the list.
    Topic(usize),
    Close,
    #[cfg(windows)]
    CheckReport,
    #[cfg(windows)]
    ReportChecked(Result<ReportStatus, String>),
    #[cfg(windows)]
    PreviewSketch,
    #[cfg(windows)]
    SketchRendered(Result<Vec<(usize, Handle)>, String>),
    #[cfg(windows)]
    IncludeAreas(bool),
    #[cfg(windows)]
    SendSketch,
    #[cfg(windows)]
    SketchSent(Result<(), String>, bool, &'static str),
}

impl Guide {
    #[cfg(windows)]
    pub fn preview_sketch(
        &mut self,
        pages: Vec<(usize, String)>,
        totals: Result<UnitAreaTotals, String>,
    ) -> Task<Message> {
        let Some(report) = self.report_target.as_ref() else {
            return Task::none();
        };
        if self.report_busy {
            return Task::none();
        }
        if pages.is_empty() {
            self.report_status = Some(format!(
                "{}\nDraw or open a sketch to preview it for transfer.",
                report.summary()
            ));
            return Task::none();
        }
        self.report_busy = true;
        self.report_status = Some(report.summary());
        self.include_areas = false;
        self.area_preview = self.area_target.clone().map(|target| {
            target.and_then(|target| totals.map(|totals| AreaTransfer::new(target, totals)))
        });
        Task::perform(appraise_it::render_pages(pages), |result| {
            Message::Guide(GuideMsg::SketchRendered(result))
        })
    }

    pub fn update(&mut self, msg: GuideMsg) -> Task<Message> {
        match msg {
            GuideMsg::Open(topic) => {
                self.open = true;
                self.topic = topic.min(TOPICS.len() - 1);
            }
            GuideMsg::Topic(topic) => self.topic = topic.min(TOPICS.len() - 1),
            GuideMsg::Close => {
                self.open = false;
                #[cfg(windows)]
                {
                    self.report_target = None;
                    self.sketch_preview.clear();
                    self.area_target = None;
                    self.area_preview = None;
                    self.include_areas = false;
                    if !self.report_busy {
                        self.report_status = None;
                    }
                }
            }
            #[cfg(windows)]
            GuideMsg::CheckReport => {
                if !self.report_busy {
                    self.report_busy = true;
                    self.report_target = None;
                    self.sketch_preview.clear();
                    self.area_target = None;
                    self.area_preview = None;
                    self.include_areas = false;
                    self.report_status = None;
                    return Task::perform(appraise_it::check_report(), |result| {
                        Message::Guide(GuideMsg::ReportChecked(result))
                    });
                }
            }
            #[cfg(windows)]
            GuideMsg::ReportChecked(result) => {
                self.report_busy = false;
                if self.open && self.topic == INTEGRATION_TOPIC {
                    self.report_status = Some(match result {
                        Ok(ReportStatus::Open(report, area_target)) => {
                            let summary = report.summary();
                            match report.transfer_availability() {
                                Ok(()) => {
                                    self.report_target = Some(report);
                                    self.area_target = Some(area_target);
                                    summary
                                }
                                Err(reason) => format!("{summary}\n{reason}"),
                            }
                        }
                        Ok(status) => status.summary(),
                        Err(error) => error,
                    });
                }
            }
            #[cfg(windows)]
            GuideMsg::PreviewSketch => {}
            #[cfg(windows)]
            GuideMsg::SketchRendered(result) => {
                self.report_busy = false;
                if let Some(report) = &self.report_target {
                    match result {
                        Ok(pages) => self.sketch_preview = pages,
                        Err(error) => {
                            self.report_status = Some(format!("{}\n{error}", report.summary()));
                        }
                    }
                }
            }
            #[cfg(windows)]
            GuideMsg::IncludeAreas(include) => {
                self.include_areas = include && matches!(self.area_preview, Some(Ok(_)));
            }
            #[cfg(windows)]
            GuideMsg::SendSketch => {
                if !self.report_busy && !self.sketch_preview.is_empty() {
                    if let Some(target) = self.report_target.take() {
                        let images = std::mem::take(&mut self.sketch_preview);
                        let preview = self.area_preview.take();
                        let areas = if self.include_areas {
                            preview.and_then(Result::ok)
                        } else {
                            None
                        };
                        let included = areas.is_some();
                        let destination = target.sketch_destination();
                        self.include_areas = false;
                        self.report_busy = true;
                        self.report_status = Some("Sending sketch to Appraise-It Pro…".into());
                        return Task::perform(
                            appraise_it::send_sketch(target, images, areas),
                            move |result| {
                                Message::Guide(GuideMsg::SketchSent(result, included, destination))
                            },
                        );
                    }
                }
            }
            #[cfg(windows)]
            GuideMsg::SketchSent(result, included, destination) => {
                self.report_busy = false;
                self.report_status = Some(result.map_or_else(
                    |error| error,
                    |()| if included {
                        "Sketch and area totals added. Review Additional Sketches and the living unit's area fields in Appraise-It Pro, then save.".into()
                    } else {
                        format!("Sketch added. Review {destination} in Appraise-It Pro and save the report.")
                    },
                ));
            }
        }
        Task::none()
    }
}

const BODY: f32 = 12.5;

fn para(s: &'static str, color: Color) -> Element<'static, Message> {
    text(s)
        .font(SEGOE)
        .size(BODY)
        .line_height(iced::widget::text::LineHeight::Relative(1.45))
        .color(color)
        .width(Length::Fill)
        .into()
}

/// A list row: a fixed-width marker, then the wrapped text.
fn item(marker: String, s: &'static str) -> Element<'static, Message> {
    row![
        text(marker)
            .font(SEGOE_SEMIBOLD)
            .size(BODY)
            .line_height(iced::widget::text::LineHeight::Relative(1.45))
            .color(muted())
            .width(22),
        para(s, text_main()),
    ]
    .into()
}

fn block(b: &'static Block) -> Element<'static, Message> {
    match b {
        Heading(s) => container(text(*s).font(SEGOE_SEMIBOLD).size(13.5).color(text_main()))
            .padding(iced::Padding {
                top: 8.0,
                ..iced::Padding::ZERO
            })
            .into(),
        Para(s) => para(s, text_main()),
        Steps(steps) => column(
            steps
                .iter()
                .enumerate()
                .map(|(i, s)| item(format!("{}.", i + 1), s)),
        )
        .spacing(5)
        .into(),
        Points(points) => column(points.iter().map(|s| item("\u{2022}".into(), s)))
            .spacing(5)
            .into(),
        Note(s) => container(para(s, text_main()))
            .padding([8, 10])
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(bg_app())),
                border: Border {
                    color: border(),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..container::Style::default()
            })
            .into(),
    }
}

/// The guide's layer, or None when closed. `size` is the window's, so the
/// card fits a small window.
pub fn layer(guide: &Guide, size: iced::Size) -> Option<Element<'static, Message>> {
    if !guide.open {
        return None;
    }
    let current = guide.topic.min(TOPICS.len() - 1);
    let topic = &TOPICS[current];

    let mut list = column![text("USER GUIDE")
        .font(SEGOE_SEMIBOLD)
        .size(11.0)
        .color(muted())]
    .spacing(2)
    .width(190);
    list = list.push(Space::new().height(4));
    for (i, t) in TOPICS.iter().enumerate() {
        let active = i == current;
        list = list.push(
            button(
                container(text(t.title).font(SEGOE).size(12.5).color(if active {
                    WHITE
                } else {
                    text_main()
                }))
                .padding([6, 10])
                .width(Length::Fill),
            )
            .padding(0)
            .width(Length::Fill)
            .on_press(Message::Guide(GuideMsg::Topic(i)))
            .style(move |_, status| button::Style {
                background: if active {
                    Some(Background::Color(accent()))
                } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                    Some(Background::Color(bg_hover()))
                } else {
                    None
                },
                border: Border {
                    radius: 3.0.into(),
                    ..Border::default()
                },
                ..button::Style::default()
            }),
        );
    }

    let mut body = column![text(topic.title)
        .font(SEGOE_SEMIBOLD)
        .size(18.0)
        .color(text_main())]
    .spacing(9)
    .padding(iced::Padding {
        right: 16.0,
        ..iced::Padding::ZERO
    });
    #[cfg(windows)]
    if current == INTEGRATION_TOPIC {
        body = body.push(para(
            "Select an Appraise-It Pro session to choose the destination report.",
            text_main(),
        ));
        body = body.push(
            button(text("Check open report").font(SEGOE).size(BODY)).on_press_maybe(
                (!guide.report_busy).then_some(Message::Guide(GuideMsg::CheckReport)),
            ),
        );
        if let Some(status) = &guide.report_status {
            body = body.push(
                text(status.clone())
                    .font(SEGOE)
                    .size(BODY)
                    .color(text_main())
                    .width(Length::Fill),
            );
        }
        if guide.report_target.is_some() && guide.sketch_preview.is_empty() {
            let label = if guide.report_busy {
                "Rendering sketch…"
            } else {
                "Preview sketch"
            };
            body = body.push(button(text(label).font(SEGOE).size(BODY)).on_press_maybe(
                (!guide.report_busy).then_some(Message::Guide(GuideMsg::PreviewSketch)),
            ));
        }
        if let Some(report) = guide
            .report_target
            .as_ref()
            .filter(|_| !guide.sketch_preview.is_empty())
        {
            body = body.push(
                text(format!(
                    "Sending appends these pages to {}.",
                    report.sketch_destination()
                ))
                .font(SEGOE)
                .size(BODY)
                .color(text_main())
                .width(Length::Fill),
            );
            for (number, image) in &guide.sketch_preview {
                body = body
                    .push(text(format!("Sketch page {number}")).font(SEGOE).size(BODY))
                    .push(
                        iced::widget::image(image.clone())
                            .width(Length::Fill)
                            .height(260),
                    );
            }
            match &guide.area_preview {
                Some(Ok(preview)) => {
                    body = body.push(
                        iced::widget::checkbox(guide.include_areas)
                            .label("Include area totals")
                            .on_toggle(|value| Message::Guide(GuideMsg::IncludeAreas(value))),
                    );
                    if guide.include_areas {
                        body = body.push(para(
                            "Review the replacements for the subject living unit. Mark above-grade outlines as GLA only when they qualify as standard finished area.",
                            text_main(),
                        )).push(text(preview.summary()).font(SEGOE).size(BODY).width(Length::Fill));
                    }
                }
                Some(Err(reason)) => {
                    body = body.push(
                        text(reason.clone())
                            .font(SEGOE)
                            .size(BODY)
                            .width(Length::Fill),
                    );
                }
                None => {}
            }
            let send_label = if guide.include_areas {
                "Send sketch and area totals"
            } else {
                "Send sketch images"
            };
            body = body.push(
                button(text(send_label).font(SEGOE).size(BODY))
                    .on_press(Message::Guide(GuideMsg::SendSketch)),
            );
        }
    }
    for b in topic.blocks {
        body = body.push(block(b));
    }

    let close =
        button(container(text("Close").font(SEGOE).size(12.0).color(WHITE)).padding([5, 12]))
            .padding(0)
            .on_press(Message::Guide(GuideMsg::Close))
            .style(move |_, _| button::Style {
                background: Some(Background::Color(accent())),
                ..button::Style::default()
            });

    let reading = column![
        scrollable(body).width(Length::Fill).height(Length::Fill),
        row![Space::new().width(Length::Fill), close],
    ]
    .spacing(10);

    let divider =
        container(Space::new().width(1).height(Length::Fill)).style(|_| container::Style {
            background: Some(Background::Color(border())),
            ..container::Style::default()
        });

    let card = container(row![list, divider, reading].spacing(14).padding(14))
        .width(860.0_f32.min((size.width - 32.0).max(320.0)))
        .height(620.0_f32.min((size.height - 48.0).max(240.0)))
        .style(|_| container::Style {
            background: Some(Background::Color(bg_panel())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        });

    // The card swallows its own presses; the backdrop closes.
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
    Some(opaque(
        mouse_area(backdrop).on_press(Message::Guide(GuideMsg::Close)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_integration_shortcut_points_at_the_integration_topic() {
        assert_eq!(TOPICS[INTEGRATION_TOPIC].title, "Appraise-It reports");
    }

    #[test]
    fn every_topic_has_a_title_and_non_empty_text() {
        for topic in TOPICS {
            assert!(!topic.title.is_empty());
            assert!(!topic.blocks.is_empty(), "{}", topic.title);
            for b in topic.blocks {
                match b {
                    Heading(s) | Para(s) | Note(s) => assert!(!s.trim().is_empty()),
                    Steps(items) | Points(items) => {
                        assert!(!items.is_empty());
                        assert!(items.iter().all(|s| !s.trim().is_empty()));
                    }
                }
            }
        }
    }

    #[test]
    fn open_picks_the_topic_and_out_of_range_is_clamped() {
        let mut guide = Guide::default();
        let _ = guide.update(GuideMsg::Open(INTEGRATION_TOPIC));
        assert!(guide.open);
        assert_eq!(guide.topic, INTEGRATION_TOPIC);
        let _ = guide.update(GuideMsg::Topic(999));
        assert_eq!(guide.topic, TOPICS.len() - 1);
        let _ = guide.update(GuideMsg::Close);
        assert!(!guide.open);
    }
}
