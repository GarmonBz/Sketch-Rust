//! The in-app User Guide (Help > User Guide / Appraise-It Reports).
//!
//! Not in the original web app: a topic list on the left, the chosen topic's
//! text on the right, in the same card/backdrop dress as the Keyboard
//! Shortcuts sheet. The text is static and lives in this file so it ships
//! inside the exe and always matches the build.

use iced::Task;

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
            Para("Sketch draws floor plans to scale and calculates their areas. A sketch is made of exterior and interior walls, the areas they enclose, and symbols and room labels placed on top."),
            Heading("The workspace"),
            Points(&[
                "Top bar: Menu, document name and Save status pill, Page tabs, Undo/Redo with history, Command search (Ctrl + K), and Send to report or Export.",
                "Areas drawer (F4 / F6): Drawn areas grouped by type with live totals and GLA, the area Types palette, and Layers.",
                "Tool tray: Floating bottom bar with Select, Wall, Inner, Area, Shape, Curve, Measure, Label, Insert, and Pan. Click a tool twice or tap its chevron for fly-outs (Circle, Cutout, Bay window).",
                "Inspector: Appears on the right whenever something is selected so you can adjust lengths, line styles, area properties, labels, symbols, or background images.",
                "Canvas controls: Zoom bar bottom-left, compass top-right, and Grid, Snap, step size, Show filters, and Dimension controls bottom-right.",
            ]),
            Heading("A first sketch"),
            Steps(&[
                "Press E or select Wall in the tool tray.",
                "Click a starting point, then type a length in the live length strip (or tap the Pad) and press an arrow key - for example 40 then Right arrow draws 40 feet to the right.",
                "Carry on around the building. Press A (or Close area) to close the shape back to the start.",
                "Choose what the area is (Living space / First Floor, Garage, Porch, and so on) in the chooser that opens or in the Areas drawer (F4).",
                "Review totals at the bottom of the Areas drawer or open Area breakdown (Tools > Area breakdown).",
                "Save with Ctrl + S.",
            ]),
            Para("Help > Keyboard Shortcuts (F1) lists every key."),
        ],
    },
    Topic {
        title: "Drawing walls",
        blocks: &[
            Para("Press E for exterior walls (Wall) or I for interior walls (Inner). Walls can be drawn with the mouse, keyboard, touch, or the on-screen Pad."),
            Para("Choose Freeform in the tool tray and drag to draw unsnapped lines without dimensions. Choose Erase, then click or drag across lines, room labels, or symbols to remove them. Each stroke is one Undo step (Ctrl+Z). Freeform lines are saved and included in printed and exported sketches."),
            Heading("With the pointer"),
            Points(&[
                "Click or tap to set each corner. The live preview snaps to grid steps and existing corners.",
                "Dashed alignment guides appear when the pointer lines up with another corner.",
                "Press Esc to stop drawing.",
            ]),
            Heading("With the keyboard or Pad"),
            Points(&[
                "Type a length into the floating length strip, then press an arrow key: 12 then Up arrow draws 12 feet up.",
                "10x5 draws a diagonal that rises 10 and runs 5.",
                "10<45 draws 10 feet at 45 degrees.",
                "10l or 10r turns left or right from the last wall (90 degrees unless you give an angle).",
                "Tab types a decimal point; Backspace erases the last character typed.",
                "A closes the area back to its starting point. J jumps the cursor to the nearest corner. D toggles curve height vs angle mode.",
            ]),
            Heading("Editing walls afterwards"),
            Points(&[
                "Press P (or pick Select) and click a wall or area. The Inspector opens with Length, Line style, Thickness, Colour, and Remove.",
                "Drag a corner to move it, or switch to Wall > Insert vertex (Insert) to add a corner along a wall.",
                "Arrow keys nudge the selection by the snap pitch (or 1 ft with snap off); Delete removes it; Ctrl + Z undoes.",
            ]),
            Heading("Changing a measurement"),
            Points(&[
                "With Select, click a line made by the Measure tool. Drag an end or corner to change it, or drag the line to move it.",
                "Delete, right-click or the Inspector removes it, and Ctrl + Z undoes.",
                "Measurements are saved with the sketch, each on its own page. Prints, and exports to picture, drawing or other apps' files, leave them out.",
                "While measuring, Esc or Delete drops only the line you are still taping. Clear removes every measurement on the page, and Ctrl + Z brings them back.",
            ]),
        ],
    },
    Topic {
        title: "Areas and calculations",
        blocks: &[
            Para("A closed run of exterior walls becomes an area. Each area has an area type (such as GLA1, GAR, POR) that controls its name, colour, fill, and whether it counts toward Living space (GLA)."),
            Heading("Defining an area"),
            Steps(&[
                "Close a perimeter or open the Areas drawer (F4).",
                "Pick the area type in the chooser modal, from the Inspector when selected, or from the Types tab in the Areas drawer.",
                "The area takes the type's name, colour and fill, and updates the Totals footer immediately.",
            ]),
            Heading("Reading the numbers"),
            Points(&[
                "The Drawn view in the Areas drawer lists each area with its type and net sf, on the layer folder it belongs to, plus live GLA and Non-GLA totals. Use the eye to hide an area or a folder, the lock to protect it, drag a row to reorder it or move it into a folder (the top row draws in front), and the folder button to add a layer folder.",
                "Tools > Area breakdown shows the rectangle and bow math behind every area.",
                "Click Edit area types in the Types tab (or Menu > Areas > Area types) to customize codes, names, colours, multipliers, and calculation rules.",
            ]),
            Heading("Subtracting one area from another"),
            Para("Select an area and turn on Subtract from surrounding area in the Inspector (or assign a negative type) to deduct open-to-below or inner voids from the enclosing area."),
            Note("An unclosed run of walls is not counted as an area. Close it with A, Close area, or by clicking back on its starting corner."),
        ],
    },
    Topic {
        title: "Symbols and labels",
        blocks: &[
            Heading("Symbols"),
            Points(&[
                "Press S or tap Insert in the tool tray to open the Insert drawer.",
                "Browse Favorites or categories, click a symbol, then click the drawing to place it (or drag straight from Favorites onto the canvas).",
                "A wide window lists the categories down the side of the drawer; a narrower one has them in a drop-down. Search covers every category.",
                "Pin keeps the drawer open while you work, until you close it or unpin it.",
                "Doors and windows snap onto a wall and cut an opening automatically; the side of the wall you click sets the door swing.",
                "Select a placed symbol to drag, rotate, flip, resize, or open Edit shape in the Inspector.",
                "Open Symbol library from the Insert drawer or Menu > Insert to manage categories, edit symbol properties, import DXF blocks, or draw custom symbol shapes.",
            ]),
            Heading("Room labels"),
            Points(&[
                "Press T (or switch the Insert drawer to Room labels) to pick a label and place it on the canvas.",
                "Select any label to edit its text, font, size, colour, or rotation in the Inspector.",
                "Open Manage room labels to add, rename, or remove reusable presets.",
            ]),
        ],
    },
    Topic {
        title: "Pages, printing and export",
        blocks: &[
            Heading("Pages"),
            Para("A sketch can have several pages, such as Main floor, Upper floor, and Site. Use the page tabs in the top bar to switch pages, add pages with +, or open the page menu to rename, duplicate, delete, or set a floor underlay."),
            Heading("Printing"),
            Points(&[
                "Menu > File > Print preview shows rendered pages with summary tables before printing or saving a PDF.",
                "Menu > File > Page layout sets paper size (Legal, Letter, A4), addendum form layout, and custom headings.",
                "Menu > File > Print sends the report to the printer.",
            ]),
            Heading("Export and import"),
            Points(&[
                "Top bar Export (or Menu > File > Export as PNG, SVG, DXF, PDF) saves images, CAD vectors, or formatted addenda.",
                "Menu > File > Import CAD brings in a DXF or DWG floor plan.",
                "Menu > File > Import and Export read and write RapidSketch (.rs) and ACI Sketch (.acisketch) files. An imported file opens as a new sketch; check its area types. Export writes the current page only.",
                "Menu > Image loads a scanned floor plan or photo behind the sketch; use Set scale and Opacity in the Inspector to trace accurately.",
            ]),
        ],
    },
    Topic {
        title: "Saving and recovery",
        blocks: &[
            Points(&[
                "Ctrl + S saves to the current file. The first save asks where to put it; Save As (F12) saves a copy under a new name and carries on in that copy.",
                "Sketches are saved as .sketchy files. A .sketchy file keeps every save as a version, so earlier states of the sketch are never lost (see Version history below).",
                "The pill next to the title reads Unsaved while there are changes that are not in the file, and Saved when there are none. A .sketchy file adds its version number: Saved · v3, Final · v3, Read-only · v3, or Viewing v1. Clicking the file title renames the sketch.",
                "The Start screen and Menu > File > Recent sketches reopen recent work with live outline thumbnails.",
                "Drop a sketch file anywhere on the window (or Start screen) to open it. Each window holds one sketch; opening a sketch that is already open brings its window forward.",
            ]),
            Heading("Version history"),
            Points(&[
                "File > Version History shows a panel listing every version of the sketch, newest first, with its date and note.",
                "Open shows an old version read-only, under a banner. Nothing can be edited there; Back to latest returns to the current sketch.",
                "Restore this version (on the banner) brings the old version back as a new latest version. Nothing is deleted, so you can go back again.",
                "Save a copy writes any version out as a separate .sketchy file and leaves the original alone.",
                "Mark as Final records the current sketch as a signed-off version with an optional note. A final version is never changed; saving more work adds a new version after it.",
                "Saving without making changes adds no new version.",
            ]),
            Heading("Autosave"),
            Para("While you work, Sketch Rust writes atomic recovery snapshots automatically. If the app closes unexpectedly, the next launch offers to restore your recovery snapshot, or use Menu > File > Recover autosave at any time. Configure intervals in Menu > Tools > Preferences."),
            Note("Recovery snapshots are cleared once you explicitly save or discard changes."),
        ],
    },
    Topic {
        title: "Appraise-It reports",
        blocks: &[
            Para("Keep your editable source in a .skx file. On Windows, use Send to report in the top bar (and the controls below) to check an open Appraise-It Pro report, preview pages, optionally transfer the living unit's area totals (all six URAR area-breakdown fields), and append sketch addenda."),
            Heading("Export a sketch for your report"),
            Steps(&[
                "Use Send to report when Appraise-It Pro is open, or Export > Export PNG / Export PDF for manual attachment.",
                "Verify area totals against the Inspector and Area breakdown before finalising report fields.",
            ]),
            Heading("If you used a previous preview with Appraise-It Pro"),
            Para("Ask SFREP or Apex support to restore the original sketch provider if needed, and open legacy AXF workfiles in Apex."),
        ],
    },
];

/// The guide overlay's state.
#[derive(Debug, Clone, Default)]
pub struct Guide {
    pub open: bool,
    pub topic: usize,
    #[cfg(windows)]
    pub(crate) report_busy: bool,
    #[cfg(windows)]
    pub(crate) report_status: Option<String>,
    #[cfg(windows)]
    pub(crate) report_target: Option<Report>,
    #[cfg(windows)]
    pub(crate) sketch_preview: Vec<(usize, Handle)>,
    #[cfg(windows)]
    pub(crate) area_target: Option<Result<AreaTarget, String>>,
    #[cfg(windows)]
    pub(crate) area_preview: Option<Result<AreaTransfer, String>>,
    #[cfg(windows)]
    pub(crate) include_areas: bool,
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
    pub(crate) fn report_process_id(&self) -> Option<i32> {
        self.report_target
            .as_ref()
            .map(appraise_it::Report::process_id)
    }

    #[cfg(windows)]
    pub(crate) fn set_checked_report(
        &mut self,
        report: Report,
        area_target: Result<AreaTarget, String>,
    ) {
        self.sketch_preview.clear();
        self.area_preview = None;
        self.include_areas = false;
        let summary = report.summary();
        self.report_status = Some(match report.transfer_availability() {
            Ok(()) => {
                self.report_target = Some(report);
                self.area_target = Some(area_target);
                summary
            }
            Err(reason) => {
                self.report_target = None;
                self.area_target = None;
                format!("{summary}\n{reason}")
            }
        });
    }

    #[cfg(windows)]
    pub fn preview_sketch(
        &mut self,
        pages: Vec<(usize, String)>,
        totals: Result<UnitAreaTotals, String>,
    ) -> Task<Message> {
        self.open = true;
        self.topic = INTEGRATION_TOPIC;
        if self.report_busy {
            return Task::none();
        }
        let Some(report) = self.report_target.as_ref() else {
            self.report_status =
                Some("Check for an open Appraise-It Pro report before previewing.".into());
            return Task::none();
        };
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
                self.open = true;
                self.topic = INTEGRATION_TOPIC;
                if !self.report_busy {
                    self.report_busy = true;
                    self.report_target = None;
                    self.sketch_preview.clear();
                    self.area_target = None;
                    self.area_preview = None;
                    self.include_areas = false;
                    self.report_status = Some("Checking Appraise-It Pro session…".into());
                    return Task::perform(appraise_it::check_report(), |result| {
                        Message::Guide(GuideMsg::ReportChecked(result))
                    });
                }
            }
            #[cfg(windows)]
            GuideMsg::ReportChecked(result) => {
                self.report_busy = false;
                match result {
                    Ok(ReportStatus::Open(report, area_target)) => {
                        self.set_checked_report(report, area_target);
                    }
                    Ok(status) => {
                        self.report_target = None;
                        self.area_target = None;
                        self.report_status = Some(status.summary());
                    }
                    Err(error) => {
                        self.report_target = None;
                        self.area_target = None;
                        self.report_status = Some(error);
                    }
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
                if !self.report_busy {
                    if self.report_target.is_none() {
                        self.open = true;
                        self.topic = INTEGRATION_TOPIC;
                        self.report_status =
                            Some("Check for an open Appraise-It Pro report before sending.".into());
                    } else if self.sketch_preview.is_empty() {
                        self.open = true;
                        self.topic = INTEGRATION_TOPIC;
                        self.report_status =
                            Some("Preview the sketch before sending it to Appraise-It Pro.".into());
                    } else if let Some(target) = self.report_target.take() {
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
    fn saving_topic_describes_sketchy_versions() {
        let topic = TOPICS
            .iter()
            .find(|t| t.title == "Saving and recovery")
            .expect("topic");
        let text: String = topic
            .blocks
            .iter()
            .flat_map(|b| match b {
                Heading(s) | Para(s) | Note(s) => vec![*s],
                Steps(items) | Points(items) => items.to_vec(),
            })
            .collect::<Vec<_>>()
            .join("\n");
        for needle in [
            ".sketchy",
            "Version History",
            "read-only",
            "Restore this version",
            "Save a copy",
            "Mark as Final",
            "Final · v3",
            "Read-only · v3",
        ] {
            assert!(text.contains(needle), "guide lacks {needle:?}");
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
