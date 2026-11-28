//! RapidSketch (`.rs`) and ACI Sketch (`.acisketch`) files.
//!
//! The two are the same format: the object graph of `RapidSketch.Sketch`
//! written by .NET's `BinaryFormatter` (see [`crate::nrbf`]). Nothing in it is
//! encrypted or compressed. See `docs/RAPIDSKETCH.md` for the layout.
//!
//! [`Document`] keeps the file's own units: screen coordinates (y down),
//! [`UNITS_PER_FOOT`] to the foot, angles clockwise on screen.

mod convert;
mod write;

pub use convert::{from_page, RapidImport};
pub use write::write;

use crate::geometry::Pt;
use crate::nrbf::{self, Graph, Node, NrbfError};

/// Drawing units in one foot.
pub const UNITS_PER_FOOT: f64 = 7.0;
/// The version string written into exported files.
pub const APP_VERSION: &str = "2.8.0.250";
/// Largest file read.
pub const MAX_BYTES: u64 = 32 * 1024 * 1024;
/// How close two wall ends must be to join, in drawing units.
const JOIN_TOLERANCE: f64 = 0.5;
/// Pen width of the strokes inside a symbol.
pub(crate) const SYMBOL_PEN: f64 = 0.2;
/// Deepest nesting of symbols inside symbols.
const MAX_SYMBOL_DEPTH: usize = 8;
/// Most strokes one placed symbol may expand to.
const MAX_SYMBOL_SEGMENTS: usize = 20_000;
/// Most strokes all symbols together may expand to.
const MAX_TOTAL_SYMBOL_SEGMENTS: usize = 200_000;
/// Most walls one area may have.
const MAX_AREA_LINES: usize = 5_000;
/// Most top-level items, and most walls in all, one sketch may hold. A file can
/// list one object any number of times, so its size alone bounds neither.
const MAX_CONTROLS: usize = 20_000;
const MAX_TOTAL_WALLS: usize = 100_000;
/// Longest text one label or name keeps, and all notes and dimensions together.
const MAX_TEXT: usize = 4_000;
const MAX_TOTAL_TEXT: usize = 4_000_000;
/// Coordinates beyond this are damage, not drawings.
const MAX_COORDINATE: f64 = 1.0e7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RapidError {
    NotRapidSketch,
    Damaged(String),
    Empty,
}

impl std::fmt::Display for RapidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RapidError::NotRapidSketch => {
                write!(f, "This is not a RapidSketch or ACI Sketch file.")
            }
            RapidError::Damaged(why) => write!(f, "This sketch file could not be read: {why}."),
            RapidError::Empty => write!(f, "No sketch geometry was found in this file."),
        }
    }
}

impl std::error::Error for RapidError {}

impl From<NrbfError> for RapidError {
    fn from(e: NrbfError) -> Self {
        RapidError::Damaged(e.to_string())
    }
}

/// A straight or circular wall stroke.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub start: Pt,
    pub end: Pt,
    /// `ArcHeight`: the arc's rise from the chord. Zero is straight. The apex
    /// is `mid + arc * (dy, -dx) / chord`.
    pub arc: f64,
}

impl Segment {
    /// The highest point of the arc (the chord's middle when straight).
    pub fn apex(&self) -> Pt {
        let (dx, dy) = (self.end.x - self.start.x, self.end.y - self.start.y);
        let chord = dx.hypot(dy);
        let mid = Pt::new(
            (self.start.x + self.end.x) / 2.0,
            (self.start.y + self.end.y) / 2.0,
        );
        if chord < 1e-9 {
            return mid;
        }
        Pt::new(mid.x + self.arc * dy / chord, mid.y - self.arc * dx / chord)
    }

    /// The segment through `start`, `apex` and `end`.
    pub(crate) fn through(start: Pt, apex: Pt, end: Pt) -> Segment {
        let (dx, dy) = (end.x - start.x, end.y - start.y);
        let chord = dx.hypot(dy);
        let arc = if chord < 1e-9 {
            0.0
        } else {
            let (ox, oy) = (
                apex.x - (start.x + end.x) / 2.0,
                apex.y - (start.y + end.y) / 2.0,
            );
            (ox * dy - oy * dx) / chord
        };
        Segment {
            start,
            end,
            // Rounding noise on a straight stroke is not an arc.
            arc: if arc.abs() < 0.01 { 0.0 } else { arc },
        }
    }

    pub(crate) fn reversed(&self) -> Segment {
        Segment {
            start: self.end,
            end: self.start,
            arc: -self.arc,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    pub name: String,
    /// Points.
    pub size: f64,
    /// `FontStyle` bits: 1 bold, 2 italic, 4 underline.
    pub style: i64,
}

/// A text object: an area's name or size line, a wall's dimension, or a note.
#[derive(Debug, Clone, PartialEq)]
pub struct TextBlock {
    pub text: String,
    /// The centre of the text.
    pub at: Pt,
    /// Degrees clockwise.
    pub rotation: f64,
    pub visible: bool,
    pub font: Font,
    /// `0xAARRGGBB`.
    pub color: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Wall {
    pub segment: Segment,
    pub dimension: TextBlock,
    /// `DashStyle`: 0 solid, 1 dash, 2 and up dotted.
    pub dash: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Area {
    pub name: String,
    pub living: bool,
    pub multiplier: f64,
    /// `0xAARRGGBB`.
    pub fill: u32,
    pub walls: Vec<Wall>,
    pub label: TextBlock,
    pub size_label: TextBlock,
}

/// A run of joined strokes inside a symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub segments: Vec<Segment>,
    pub closed: bool,
}

/// A placed symbol (a `ContainerControl` and its graphic).
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub category: String,
    pub name: String,
    pub center: Pt,
    pub width: f64,
    pub height: f64,
    /// Degrees clockwise.
    pub rotation: f64,
    pub flip_h: bool,
    pub flip_v: bool,
    /// Relative to the centre, at the placed size, before flips and rotation.
    pub shapes: Vec<Shape>,
}

/// `SketchInformation`: the subject property.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Information {
    pub case_number: String,
    pub file_number: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub borrower: String,
    pub county: String,
    pub client: String,
    pub client_address: String,
    pub appraiser_name: String,
    pub appraiser_address: String,
}

/// Everything read from, or to be written to, one file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    pub areas: Vec<Area>,
    /// Walls to be written that belong to no closed area, as open runs.
    pub open_walls: Vec<Vec<Wall>>,
    /// `FreeformLine`s: the interior walls, drawn on the sketch itself and
    /// part of no area. They are straight and carry no pen or dimension.
    pub interior_walls: Vec<Segment>,
    pub notes: Vec<TextBlock>,
    pub symbols: Vec<Symbol>,
    pub information: Information,
    /// Things the user should be told about.
    pub warnings: Vec<String>,
}

/// True if `bytes` look like a RapidSketch / ACI Sketch file.
pub fn recognizes(bytes: &[u8]) -> bool {
    const MARK: &[u8] = b"RapidSketch.Sketch";
    nrbf::has_header(bytes)
        && bytes[..bytes.len().min(4096)]
            .windows(MARK.len())
            .any(|w| w == MARK)
}

/// Read a `.rs` / `.acisketch` file's bytes.
pub fn read(bytes: &[u8]) -> Result<Document, RapidError> {
    if !recognizes(bytes) {
        return Err(RapidError::NotRapidSketch);
    }
    let graph = nrbf::parse(bytes)?;
    decode(&graph)
}

fn coordinate(v: Option<f64>) -> Result<f64, RapidError> {
    v.filter(|v| v.is_finite() && v.abs() < MAX_COORDINATE)
        .ok_or_else(|| RapidError::Damaged("a coordinate is missing or out of range".into()))
}

/// `text`, cut to [`MAX_TEXT`] characters.
fn clipped(text: &str) -> String {
    text.chars().take(MAX_TEXT).collect()
}

/// A text object's lines as one string, cut to about [`MAX_TEXT`].
fn joined(lines: &[&str]) -> String {
    let mut out = String::new();
    for (i, line) in lines.iter().take(200).enumerate() {
        if out.len() >= MAX_TEXT {
            break;
        }
        if i > 0 {
            out.push('\n');
        }
        out.extend(line.chars().take(MAX_TEXT));
    }
    out
}

/// A `System.Drawing.PointF` member.
fn point(node: Node, member: &str) -> Result<Pt, RapidError> {
    let p = node
        .child(member)
        .ok_or_else(|| RapidError::Damaged(format!("a {member} is missing")))?;
    Ok(Pt::new(
        coordinate(p.number("x"))?,
        coordinate(p.number("y"))?,
    ))
}

fn angle(node: Node, member: &str) -> f64 {
    node.number(member)
        .filter(|v| v.is_finite())
        .map_or(0.0, |v| v.rem_euclid(360.0))
}

/// A `System.Drawing.Color` member as `0xAARRGGBB`.
fn color(node: Node, member: &str, fallback: u32) -> u32 {
    let Some(c) = node.child(member) else {
        return fallback;
    };
    match c.int("state") {
        // A named color: an index into .NET's KnownColor table.
        Some(1) => c
            .int("knownColor")
            .and_then(known_color)
            .unwrap_or(fallback),
        Some(0) | None => fallback,
        Some(_) => c.int("value").map_or(fallback, |v| v as u32),
    }
}

/// .NET `KnownColor` 27 (Transparent) through 167 (YellowGreen).
fn known_color(index: i64) -> Option<u32> {
    #[rustfmt::skip]
    const NAMED: [u32; 140] = [
        0xF0F8FF, 0xFAEBD7, 0x00FFFF, 0x7FFFD4, 0xF0FFFF, 0xF5F5DC, 0xFFE4C4, 0x000000,
        0xFFEBCD, 0x0000FF, 0x8A2BE2, 0xA52A2A, 0xDEB887, 0x5F9EA0, 0x7FFF00, 0xD2691E,
        0xFF7F50, 0x6495ED, 0xFFF8DC, 0xDC143C, 0x00FFFF, 0x00008B, 0x008B8B, 0xB8860B,
        0xA9A9A9, 0x006400, 0xBDB76B, 0x8B008B, 0x556B2F, 0xFF8C00, 0x9932CC, 0x8B0000,
        0xE9967A, 0x8FBC8B, 0x483D8B, 0x2F4F4F, 0x00CED1, 0x9400D3, 0xFF1493, 0x00BFFF,
        0x696969, 0x1E90FF, 0xB22222, 0xFFFAF0, 0x228B22, 0xFF00FF, 0xDCDCDC, 0xF8F8FF,
        0xFFD700, 0xDAA520, 0x808080, 0x008000, 0xADFF2F, 0xF0FFF0, 0xFF69B4, 0xCD5C5C,
        0x4B0082, 0xFFFFF0, 0xF0E68C, 0xE6E6FA, 0xFFF0F5, 0x7CFC00, 0xFFFACD, 0xADD8E6,
        0xF08080, 0xE0FFFF, 0xFAFAD2, 0xD3D3D3, 0x90EE90, 0xFFB6C1, 0xFFA07A, 0x20B2AA,
        0x87CEFA, 0x778899, 0xB0C4DE, 0xFFFFE0, 0x00FF00, 0x32CD32, 0xFAF0E6, 0xFF00FF,
        0x800000, 0x66CDAA, 0x0000CD, 0xBA55D3, 0x9370DB, 0x3CB371, 0x7B68EE, 0x00FA9A,
        0x48D1CC, 0xC71585, 0x191970, 0xF5FFFA, 0xFFE4E1, 0xFFE4B5, 0xFFDEAD, 0x000080,
        0xFDF5E6, 0x808000, 0x6B8E23, 0xFFA500, 0xFF4500, 0xDA70D6, 0xEEE8AA, 0x98FB98,
        0xAFEEEE, 0xDB7093, 0xFFEFD5, 0xFFDAB9, 0xCD853F, 0xFFC0CB, 0xDDA0DD, 0xB0E0E6,
        0x800080, 0xFF0000, 0xBC8F8F, 0x4169E1, 0x8B4513, 0xFA8072, 0xF4A460, 0x2E8B57,
        0xFFF5EE, 0xA0522D, 0xC0C0C0, 0x87CEEB, 0x6A5ACD, 0x708090, 0xFFFAFA, 0x00FF7F,
        0x4682B4, 0xD2B48C, 0x008080, 0xD8BFD8, 0xFF6347, 0x40E0D0, 0xEE82EE, 0xF5DEB3,
        0xFFFFFF, 0xF5F5F5, 0xFFFF00, 0x9ACD32,
    ];
    match index {
        27 => Some(0x00FF_FFFF),
        28..=167 => NAMED
            .get((index - 28) as usize)
            .map(|rgb| 0xFF00_0000 | rgb),
        _ => None,
    }
}

fn text_block(node: Node) -> Result<TextBlock, RapidError> {
    let font = node.child("Font");
    let size = font
        .and_then(|f| f.number("Size"))
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(8.0)
        .min(200.0);
    Ok(TextBlock {
        text: joined(&node.strings("Lines")),
        at: Pt::new(coordinate(node.number("X"))?, coordinate(node.number("Y"))?),
        rotation: angle(node, "Rotation"),
        visible: node.flag("Visible").unwrap_or(true),
        font: Font {
            name: font.map_or_else(String::new, |f| clipped(f.string("Name"))),
            size,
            style: font
                .and_then(|f| f.child("Style"))
                .and_then(|s| s.int("value__"))
                .unwrap_or(0),
        },
        color: color(node, "Color", 0xFF00_0000),
    })
}

fn segment(line: Node) -> Result<Segment, RapidError> {
    let end_point = |member: &str| -> Result<Pt, RapidError> {
        let p = line
            .child(member)
            .ok_or_else(|| RapidError::Damaged("a wall has no end point".into()))?;
        point(p, "Location")
    };
    let (start, end) = (end_point("Start")?, end_point("End")?);
    let arc = line
        .number("ArcHeight")
        .filter(|a| a.is_finite() && a.abs() < MAX_COORDINATE)
        .unwrap_or(0.0);
    Ok(Segment { start, end, arc })
}

fn wall(line: Node) -> Result<Wall, RapidError> {
    let segment = segment(line)?;
    let dimension = match line.child("Label") {
        Some(label) => text_block(label)?,
        None => return Err(RapidError::Damaged("a wall has no dimension label".into())),
    };
    Ok(Wall {
        segment,
        dimension,
        dash: line
            .child("DashStyle")
            .and_then(|d| d.int("value__"))
            .unwrap_or(0),
    })
}

fn near(a: Pt, b: Pt) -> bool {
    (a.x - b.x).abs() <= JOIN_TOLERANCE && (a.y - b.y).abs() <= JOIN_TOLERANCE
}

/// A chain of segment indices, each with whether it runs backwards.
pub(crate) struct Chain {
    pub(crate) links: Vec<(usize, bool)>,
    pub(crate) closed: bool,
}

/// Join segments end to end. A file stores an area's walls in the order they
/// were drawn or edited, not necessarily around the outline.
pub(crate) fn chains(segments: &[Segment]) -> Vec<Chain> {
    let mut used = vec![false; segments.len()];
    let mut out = Vec::new();
    for first in 0..segments.len() {
        if used[first] {
            continue;
        }
        used[first] = true;
        let mut links = std::collections::VecDeque::from([(first, false)]);
        let mut head = segments[first].start;
        let mut tail = segments[first].end;
        let mut closed = false;
        // Grow from the tail, then (if still open) from the head.
        let find = |used: &[bool], at: Pt, from_start: bool| {
            let matches = |i: &usize, forward: bool| {
                let s = &segments[*i];
                let end = if forward == from_start {
                    s.start
                } else {
                    s.end
                };
                !used[*i] && near(end, at)
            };
            (0..segments.len())
                .find(|i| matches(i, true))
                .map(|i| (i, false))
                .or_else(|| {
                    (0..segments.len())
                        .find(|i| matches(i, false))
                        .map(|i| (i, true))
                })
        };
        loop {
            if links.len() >= 2 && near(tail, head) {
                closed = true;
                break;
            }
            let Some((i, reversed)) = find(&used, tail, true) else {
                break;
            };
            used[i] = true;
            tail = if reversed {
                segments[i].start
            } else {
                segments[i].end
            };
            links.push_back((i, reversed));
        }
        while !closed {
            let Some((i, reversed)) = find(&used, head, false) else {
                break;
            };
            used[i] = true;
            head = if reversed {
                segments[i].end
            } else {
                segments[i].start
            };
            links.push_front((i, reversed));
            closed = links.len() >= 2 && near(tail, head);
        }
        let links: Vec<_> = links.into_iter().collect();
        // Two straight walls over the same span enclose nothing.
        let closed =
            closed && (links.len() >= 3 || links.iter().any(|(i, _)| segments[*i].arc != 0.0));
        out.push(Chain { links, closed });
    }
    out
}

/// Items left out of an import, so the user is told rather than left to notice.
#[derive(Default)]
struct Skipped {
    count: usize,
    kinds: std::collections::BTreeSet<String>,
}

impl Skipped {
    fn add(&mut self, kind: &str) {
        self.count += 1;
        // A file chooses these names, so both their number and length are cut.
        if self.kinds.len() < 8 {
            self.kinds.insert(kind.chars().take(40).collect());
        }
    }

    fn warning(&self) -> Option<String> {
        let kinds: Vec<&str> = self.kinds.iter().map(String::as_str).collect();
        (self.count > 0).then(|| {
            format!(
                "{} item{} Sketch Rust does not support {} not imported: {}.",
                self.count,
                if self.count == 1 { "" } else { "s" },
                if self.count == 1 { "was" } else { "were" },
                kinds.join(", "),
            )
        })
    }
}

/// An area's walls. Besides them it holds only its pieces of text.
fn lines_of<'a>(area: Node<'a>, skipped: &mut Skipped) -> Result<Vec<Node<'a>>, RapidError> {
    let mut lines = Vec::new();
    for control in area.children("Controls") {
        match control.short_class() {
            "Line" => lines.push(control),
            "Label" | "Note" => {}
            other => skipped.add(other),
        }
    }
    if lines.len() > MAX_AREA_LINES {
        return Err(RapidError::Damaged("an area has too many walls".into()));
    }
    Ok(lines)
}

fn area(node: Node, skipped: &mut Skipped) -> Result<Area, RapidError> {
    let walls = lines_of(node, skipped)?
        .into_iter()
        .map(wall)
        .collect::<Result<Vec<_>, _>>()?;
    let block = |member: &str| -> Result<TextBlock, RapidError> {
        match node.child(member) {
            Some(n) => text_block(n),
            None => Err(RapidError::Damaged("an area has no label".into())),
        }
    };
    Ok(Area {
        name: clipped(node.string("Name")),
        living: node.flag("LivingArea").unwrap_or(false),
        multiplier: node
            .number("Multiplier")
            .filter(|m| m.is_finite())
            .unwrap_or(1.0),
        fill: color(node, "FillColor", 0),
        walls,
        label: block("Label")?,
        size_label: block("DimensionLabel")?,
    })
}

#[derive(Clone, Copy)]
struct Bounds {
    min: Pt,
    max: Pt,
}

impl Bounds {
    const EMPTY: Bounds = Bounds {
        min: Pt {
            x: f64::MAX,
            y: f64::MAX,
        },
        max: Pt {
            x: f64::MIN,
            y: f64::MIN,
        },
    };

    fn add(&mut self, p: Pt, margin: f64) {
        self.min.x = self.min.x.min(p.x - margin);
        self.min.y = self.min.y.min(p.y - margin);
        self.max.x = self.max.x.max(p.x + margin);
        self.max.y = self.max.y.max(p.y + margin);
    }
}

/// How a `ContainerControl` places its graphic in its parent.
struct Placement {
    center: Pt,
    width: f64,
    height: f64,
    rotation: f64,
    flip_h: bool,
    flip_v: bool,
}

fn placement(node: Node) -> Result<Placement, RapidError> {
    let size = node
        .child("Size")
        .ok_or_else(|| RapidError::Damaged("a symbol has no size".into()))?;
    Ok(Placement {
        center: point(node, "Location")?,
        width: coordinate(size.number("width"))?.abs(),
        height: coordinate(size.number("height"))?.abs(),
        rotation: angle(node, "Rotation"),
        flip_h: node.flag("HorizontalFlip").unwrap_or(false),
        flip_v: node.flag("VerticalFlip").unwrap_or(false),
    })
}

impl Placement {
    /// A point of the fitted graphic (centred on the origin) in the parent.
    fn place(&self, p: Pt) -> Pt {
        let x = if self.flip_h { -p.x } else { p.x };
        let y = if self.flip_v { -p.y } else { p.y };
        let (sin, cos) = self.rotation.to_radians().sin_cos();
        Pt::new(
            self.center.x + x * cos - y * sin,
            self.center.y + x * sin + y * cos,
        )
    }

    fn corners(&self) -> [Pt; 4] {
        let (w, h) = (self.width / 2.0, self.height / 2.0);
        [(-w, -h), (w, -h), (w, h), (-w, h)].map(|(x, y)| self.place(Pt::new(x, y)))
    }
}

/// The strokes of a symbol graphic, stretched to `width` x `height` and
/// centred on the origin. `budget` bounds the work one file can cause.
fn graphic_shapes(
    graphic: Node,
    width: f64,
    height: f64,
    depth: usize,
    budget: &mut usize,
    skipped: &mut Skipped,
) -> Result<Vec<Shape>, RapidError> {
    if depth > MAX_SYMBOL_DEPTH {
        return Err(RapidError::Damaged("symbols are nested too deeply".into()));
    }
    let mut shapes = Vec::new();
    let mut bounds = Bounds::EMPTY;
    for control in graphic.children("Controls") {
        match control.short_class() {
            "Area" => {
                let lines = lines_of(control, skipped)?;
                let mut segments = Vec::with_capacity(lines.len());
                for line in lines {
                    *budget = budget
                        .checked_sub(1)
                        .ok_or_else(|| RapidError::Damaged("the symbols are too complex".into()))?;
                    let s = segment(line)?;
                    let margin = line
                        .number("Width")
                        .filter(|w| w.is_finite() && *w >= 0.0)
                        .unwrap_or(SYMBOL_PEN)
                        / 2.0;
                    for p in [s.start, s.apex(), s.end] {
                        bounds.add(p, margin);
                    }
                    segments.push(s);
                }
                for chain in chains(&segments) {
                    shapes.push(Shape {
                        segments: chain
                            .links
                            .iter()
                            .map(|&(i, reversed)| {
                                if reversed {
                                    segments[i].reversed()
                                } else {
                                    segments[i]
                                }
                            })
                            .collect(),
                        closed: chain.closed,
                    });
                }
            }
            "ContainerControl" => {
                let Some(inner) = control.child("Graphic") else {
                    continue;
                };
                let place = placement(control)?;
                for corner in place.corners() {
                    bounds.add(corner, 0.0);
                }
                let inner =
                    graphic_shapes(inner, place.width, place.height, depth + 1, budget, skipped)?;
                shapes.extend(map_shapes(inner, |p| place.place(p)));
            }
            other => skipped.add(other),
        }
    }
    if shapes.is_empty() {
        return Ok(shapes);
    }
    let center = Pt::new(
        (bounds.min.x + bounds.max.x) / 2.0,
        (bounds.min.y + bounds.max.y) / 2.0,
    );
    let stretch = |size: f64, drawn: f64| if drawn > 1e-6 { size / drawn } else { 1.0 };
    let sx = stretch(width, bounds.max.x - bounds.min.x);
    let sy = stretch(height, bounds.max.y - bounds.min.y);
    Ok(map_shapes(shapes, |p| {
        Pt::new((p.x - center.x) * sx, (p.y - center.y) * sy)
    }))
}

fn map_shapes(shapes: Vec<Shape>, f: impl Fn(Pt) -> Pt) -> Vec<Shape> {
    shapes
        .into_iter()
        .map(|shape| Shape {
            segments: shape
                .segments
                .iter()
                .map(|s| Segment::through(f(s.start), f(s.apex()), f(s.end)))
                .collect(),
            closed: shape.closed,
        })
        .collect()
}

fn decode(graph: &Graph) -> Result<Document, RapidError> {
    let sketch = graph
        .root()
        .filter(|r| r.class() == "RapidSketch.Sketch")
        .ok_or(RapidError::NotRapidSketch)?;
    let mut doc = Document::default();
    let mut budget = MAX_TOTAL_SYMBOL_SEGMENTS;
    let mut skipped = Skipped::default();
    let (mut walls, mut text) = (0usize, 0usize);
    let controls = sketch.children("Controls");
    if controls.len() > MAX_CONTROLS {
        return Err(RapidError::Damaged("it holds too many items".into()));
    }
    for control in controls {
        if walls > MAX_TOTAL_WALLS || text > MAX_TOTAL_TEXT {
            return Err(RapidError::Damaged("it holds too much to open".into()));
        }
        match control.short_class() {
            "Area" => {
                let area = area(control, &mut skipped)?;
                walls += area.walls.len();
                text += area
                    .walls
                    .iter()
                    .map(|w| w.dimension.text.len())
                    .sum::<usize>();
                doc.areas.push(area);
            }
            // An interior wall: two ends and nothing else.
            "FreeformLine" => {
                walls += 1;
                let wall = segment(control)?;
                if (wall.start.x - wall.end.x).hypot(wall.start.y - wall.end.y) > 1e-6 {
                    doc.interior_walls.push(wall);
                }
            }
            "Note" | "Label" => {
                let note = text_block(control)?;
                text += note.text.len();
                if note.visible && !note.text.trim().is_empty() {
                    doc.notes.push(note);
                }
            }
            "ContainerControl" => {
                let Some(graphic) = control.child("Graphic") else {
                    skipped.add("symbol without a drawing");
                    continue;
                };
                let place = placement(control)?;
                let mut one = MAX_SYMBOL_SEGMENTS.min(budget);
                let allowed = one;
                let shapes = graphic_shapes(
                    graphic,
                    place.width,
                    place.height,
                    0,
                    &mut one,
                    &mut skipped,
                )?;
                budget -= allowed - one;
                if shapes.is_empty() {
                    skipped.add("symbol without a drawing");
                    continue;
                }
                doc.symbols.push(Symbol {
                    category: clipped(graphic.string("Category")),
                    name: clipped(graphic.string("Name")),
                    center: place.center,
                    width: place.width,
                    height: place.height,
                    rotation: place.rotation,
                    flip_h: place.flip_h,
                    flip_v: place.flip_v,
                    shapes,
                });
            }
            other => skipped.add(other),
        }
    }
    doc.warnings.extend(skipped.warning());
    if sketch.child("PhotoMetricsImage").is_some() {
        doc.warnings
            .push("The background picture was not imported.".into());
    }
    if let Some(info) = sketch.child("SketchInformation") {
        let text = |member: &str| clipped(info.string(member));
        doc.information = Information {
            case_number: text("CaseNumber"),
            file_number: text("FileNumber"),
            address: text("Address"),
            city: text("City"),
            state: text("State"),
            zip: text("Zip"),
            borrower: text("Borrower"),
            county: text("County"),
            client: text("LenderOrClient"),
            client_address: text("LenderOrClientAddress"),
            appraiser_name: text("AppraiserName"),
            appraiser_address: text("AppraiserAddress"),
        };
    }
    if doc.areas.is_empty()
        && doc.interior_walls.is_empty()
        && doc.notes.is_empty()
        && doc.symbols.is_empty()
    {
        return Err(RapidError::Empty);
    }
    Ok(doc)
}

#[cfg(test)]
mod tests;
