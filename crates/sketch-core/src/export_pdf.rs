//! PDF export — the native counterpart of the standalone app's print path
//! (`SketchApp.exportPDF` → `SketchReport.documentHTML`): one report page per
//! sketch page in the print layout, generated as a minimal hand-rolled PDF 1.4
//! file (the browser's output is HTML print CSS, which has
//! no direct native equivalent, so the layout constants are transcribed
//! exactly and the content is drawn as PDF operators).
//!
//! Layout (all values from `renderPage`/REPORT_CSS in the standalone HTML):
//! title top 16.4pt · subject box top 38.2pt h 72.2pt · sketch heading top
//! 110.4pt h 12pt · sketch body top 122.4pt w 540pt · lower section top
//! 122.4 + sketchHeight · page number top height−21.5. Sketch geometry is
//! fit into the sketch box through `print_layout::make_transform`
//! (labels/symbols excluded from the fit bounds).

use crate::export_model::{curve_apex, flattened_points, num_or_zero, ExportOptions, ExportPage};
use crate::geometry::Pt;
use crate::print_layout::{make_transform, PaperSize, PAGE_WIDTH, SKETCH_PAD, SKETCH_WIDTH};

/// Layout constants from the standalone's `renderPage`/REPORT_CSS.
pub const TITLE_TOP: f64 = 16.4;
const TITLE_HEIGHT: f64 = 17.0;
const SUBJECT_TOP: f64 = 38.2;
const SUBJECT_HEIGHT: f64 = 72.2;
const SKETCH_HEADING_TOP: f64 = 110.4;
const SKETCH_HEADING_HEIGHT: f64 = 12.0;
const SKETCH_BODY_TOP: f64 = 122.4;
const SECTION_ROW_HEIGHT: f64 = 9.9;
const LOWER_MARGIN: f64 = 37.0;
const TABLE_HEADER_HEIGHT: f64 = 12.0;
const TABLE_ROW_HEIGHT: f64 = 10.74;
const CALC_FOOTER_HEIGHT: f64 = 30.0;

#[derive(Default)]
struct ReportCalculations {
    summary: Vec<[String; 6]>,
    breakdown: Vec<String>,
    livable_count: usize,
    livable_total: f64,
}

fn report_calculations(
    page: &ExportPage,
    codes: &[crate::geometry::AreaCode],
) -> ReportCalculations {
    use crate::breakdown::{self, Options, Source};
    let mut out = ReportCalculations::default();
    let options = Options {
        area_precision: 1,
        ..Options::default()
    };
    for (index, p) in page
        .polygons
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.area_type.is_empty() && p.area_type != "UND" && p.points.len() >= 3)
    {
        let code = codes.iter().find(|c| c.code == p.area_type);
        let name = if p.name.is_empty() {
            code.map_or(p.area_type.as_str(), |c| c.name.as_str())
        } else {
            &p.name
        };
        let sign = if code.is_some_and(|c| !c.positive) {
            -1.0
        } else {
            1.0
        };
        let factor = p
            .effective_multiplier
            .or(code.map(|c| c.multiplier))
            .unwrap_or(1.0);
        let net = p.net.unwrap_or(p.gross * factor * sign);
        let deductions = if factor.abs() > 1e-9 {
            (p.gross - net / (factor * sign)).max(0.0)
        } else {
            0.0
        };
        out.summary.push([
            p.area_type.clone(),
            name.to_owned(),
            format!("{:.2}", factor * sign),
            format!("{:.1}", p.gross),
            format!("{:.1}", p.perimeter),
            format!("{net:.1}"),
        ]);
        let result = breakdown::compute_for_polygon(
            &Source {
                id: index as i64,
                kind: &p.area_type,
                name,
                points: &p.points,
                edges: &p.edges,
                gross: Some(p.gross),
                container_id: None,
                subtracted_from_id: None,
                subtracted: deductions,
                multiplier: factor,
                sign,
                net: Some(net),
                perimeter: Some(p.perimeter),
            },
            &options,
        );
        let mut detail = vec![format!("{name} (net {net:.1} sf)")];
        detail.extend(
            result
                .pieces
                .iter()
                .map(|piece| breakdown::piece_line(piece, &options)),
        );
        if (factor * sign - 1.0).abs() > 1e-9 || deductions > 1e-9 {
            detail.push(format!(
                "Factor {:.2}; deductions {deductions:.1}; net {net:.1}",
                factor * sign
            ));
        }
        for line in detail {
            out.breakdown.extend(wrap_text(
                Font::Regular,
                7.0,
                &line,
                SKETCH_WIDTH / 2.0 - 10.0,
            ));
        }
        if code.is_some_and(|c| c.include_in_gla) {
            out.livable_count += 1;
            out.livable_total += net;
        }
    }
    out
}

fn comment_tables_height(subject: &ReportSubject, layout: &ReportLayout) -> f64 {
    if layout.show_comment_tables
        && subject
            .comment_tables
            .iter()
            .flatten()
            .any(|s| !s.trim().is_empty())
    {
        96.0
    } else {
        0.0
    }
}

/// The raster and PDF must use the same sketch box, including space reserved
/// for long calculation tables and optional populated comment tables.
pub fn report_sketch_height(
    page: &ExportPage,
    codes: &[crate::geometry::AreaCode],
    subject: &ReportSubject,
    layout: &ReportLayout,
) -> f64 {
    let calc = report_calculations(page, codes);
    let required = 24.0
        + CALC_FOOTER_HEIGHT
        + TABLE_ROW_HEIGHT * calc.summary.len().max(calc.breakdown.len()) as f64;
    let available = layout.paper.size().1
        - SKETCH_BODY_TOP
        - LOWER_MARGIN
        - comment_tables_height(subject, layout);
    layout
        .paper
        .sketch_height()
        .min(available - required)
        .max(160.0)
}

fn calculation_capacity(
    sketch_height: f64,
    subject: &ReportSubject,
    layout: &ReportLayout,
) -> usize {
    let height = layout.paper.size().1
        - SKETCH_BODY_TOP
        - sketch_height
        - comment_tables_height(subject, layout)
        - LOWER_MARGIN;
    ((height - 24.0 - CALC_FOOTER_HEIGHT) / TABLE_ROW_HEIGHT)
        .floor()
        .max(1.0) as usize
}
/// Page number row height offset (`size.height - 21.5`).
const PAGE_NUMBER_OFFSET: f64 = 21.5;

/// Border/section gray tones from REPORT_CSS (#555 border, #d3d3d3 fill,
/// #777/#aaa row rules).
const GRAY_BORDER: f64 = 0.3333;
const GRAY_FILL: f64 = 0.8275;
const GRAY_ROW: f64 = 0.4667;

/// The subject-info strings the PDF prints (subset the report renderer
/// reads). Empty strings leave the labeled blanks.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReportSubject {
    pub file_no: String,
    pub parcel_no: String,
    pub property_address: String,
    pub city: String,
    pub county: String,
    pub state: String,
    pub zip_code: String,
    pub owner: String,
    pub client: String,
    pub client_address: String,
    pub appraiser_name: String,
    pub inspection_date: String,
    pub general_comments: String,
    /// Three comment tables of up to 10 rows each.
    pub comment_tables: Vec<Vec<String>>,
}

/// A stroked/filled drawing instruction in PDF page space (y-up points).
enum Op {
    FillRect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
    StrokeRect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    Polyline {
        points: Vec<(f64, f64)>,
        closed: bool,
        fill: bool,
    },
    Text {
        x: f64,
        y: f64,
        font: Font,
        size: f64,
        text: String,
    },
    SetLineWidth(f64),
    SetGrayFill(f64),
    SetGrayStroke(f64),
    SetDash(Vec<f64>),
    ResetDash,
    Image {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Font {
    Regular,
    Bold,
}

/// The content stream builder: ops in draw order, y-up page space.
struct Content {
    ops: Vec<Op>,
}

impl Content {
    fn new() -> Self {
        Content { ops: Vec::new() }
    }

    fn serialize(&self) -> String {
        let mut out = String::new();
        for op in &self.ops {
            match op {
                Op::SetGrayFill(g) => out += &format!("{:.4} g\n", g),
                Op::SetGrayStroke(g) => out += &format!("{:.4} G\n", g),
                Op::SetLineWidth(w) => out += &format!("{:.4} w\n", w),
                Op::SetDash(pattern) => {
                    let pattern = pattern
                        .iter()
                        .map(|v| format!("{v:.2}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    out += &format!("[{pattern}] 0 d\n");
                }
                Op::ResetDash => out += "[] 0 d\n",
                Op::Image { x, y, w, h } => {
                    out += &format!("q {w:.4} 0 0 {h:.4} {x:.4} {y:.4} cm /Sketch Do Q\n")
                }
                Op::FillRect { x, y, w, h } => {
                    out += &format!("{x:.2} {y:.2} {w:.2} {h:.2} re f\n");
                }
                Op::StrokeRect { x, y, w, h } => {
                    out += &format!("{x:.2} {y:.2} {w:.2} {h:.2} re S\n");
                }
                Op::Line { x1, y1, x2, y2 } => {
                    out += &format!("{x1:.2} {y1:.2} m {x2:.2} {y2:.2} l S\n");
                }
                Op::Polyline {
                    points,
                    closed,
                    fill,
                } => {
                    if points.is_empty() {
                        continue;
                    }
                    let mut path = String::new();
                    for (index, (x, y)) in points.iter().enumerate() {
                        path += &if index == 0 {
                            format!("{x:.2} {y:.2} m ")
                        } else {
                            format!("{x:.2} {y:.2} l ")
                        };
                    }
                    if *closed {
                        path += "h ";
                    }
                    path += if *fill { "f\n" } else { "S\n" };
                    out += &path;
                }
                Op::Text {
                    x,
                    y,
                    font,
                    size,
                    text,
                } => {
                    let name = match font {
                        Font::Regular => "/F1",
                        Font::Bold => "/F2",
                    };
                    let operand = crate::report_fonts::encode(matches!(font, Font::Bold), text)
                        .map(|hex| format!("<{hex}>"))
                        .unwrap_or_else(|| format!("({})", pdf_escape(text)));
                    out += &format!(
                        "BT {} {:.2} Tf {:.2} {:.2} Td {} Tj ET\n",
                        name, size, x, y, operand
                    );
                }
            }
        }
        out
    }
}

/// PDF string escape: backslash, parens.
fn pdf_escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '\\' => out += "\\\\",
            '(' => out += "\\(",
            ')' => out += "\\)",
            _ => {
                out.push(c);
            }
        }
    }
    out
}

/// Assemble the PDF: catalog, a page tree, the two base fonts, then one
/// page + one content stream per report page.
/// A report sketch is embedded at print resolution; report text stays vector,
/// searchable Unicode using the actual installed report font.
#[derive(Debug, Clone)]
pub struct ReportSketch {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
}

fn assemble_pdf(
    contents: &[Content],
    page_width: f64,
    page_height: f64,
    sketches: &[Option<ReportSketch>],
) -> Vec<u8> {
    use std::{collections::BTreeSet, io::Write};
    let chars: BTreeSet<_> = contents
        .iter()
        .flat_map(|c| c.ops.iter())
        .filter_map(|op| {
            if let Op::Text { text, .. } = op {
                Some(text.as_str())
            } else {
                None
            }
        })
        .flat_map(str::chars)
        .collect();
    let font_base = 5 + 2 * contents.len();
    let fonts = crate::report_fonts::font_objects(false, font_base, &chars).zip(
        crate::report_fonts::font_objects(true, font_base + 5, &chars),
    );
    let mut objects: Vec<Vec<u8>> = Vec::new();
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    objects.push(
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            (0..contents.len())
                .map(|i| format!("{} 0 R", 5 + 2 * i))
                .collect::<Vec<_>>()
                .join(" "),
            contents.len()
        )
        .into_bytes(),
    );
    let (regular, bold, extra) = if let Some(((regular, mut a), (bold, b))) = fonts {
        a.extend(b);
        (regular, bold, a)
    } else {
        ("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".into(),"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".into(),Vec::new())
    };
    objects.push(regular.into_bytes());
    objects.push(bold.into_bytes());
    let mut image_objects = Vec::new();
    for (i, content) in contents.iter().enumerate() {
        let content = content.serialize();
        let image = sketches.get(i).and_then(Option::as_ref);
        let image_id = font_base + extra.len() + image_objects.len();
        let resource = if image.is_some() {
            format!("/XObject << /Sketch {image_id} 0 R >>")
        } else {
            String::new()
        };
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {page_width:.2} {page_height:.2}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> {resource} >> /Contents {} 0 R >>",6+2*i).into_bytes());
        let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
        stream.extend(content.as_bytes());
        stream.extend(b"\nendstream");
        objects.push(stream);
        if let Some(image) = image {
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder
                .write_all(&image.rgb)
                .expect("in-memory compression");
            let bytes = encoder.finish().expect("in-memory compression");
            let mut obj=format!("<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",image.width,image.height,bytes.len()).into_bytes();
            obj.extend(bytes);
            obj.extend(b"\nendstream");
            image_objects.push(obj);
        }
    }
    objects.extend(extra);
    objects.extend(image_objects);
    let mut body = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    for (i, obj) in objects.iter().enumerate() {
        offsets.push(body.len());
        body.extend(format!("{} 0 obj\n", i + 1).as_bytes());
        body.extend(obj);
        body.extend(b"\nendobj\n");
    }
    let start = body.len();
    let count = objects.len() + 1;
    body.extend(format!("xref\n0 {count}\n0000000000 65535 f \n").as_bytes());
    for offset in offsets {
        body.extend(format!("{offset:010} 00000 n \n").as_bytes());
    }
    body.extend(
        format!("trailer\n<< /Size {count} /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n").as_bytes(),
    );
    body
}

/// The Page Layout captions (`normalizeLayout`: an empty caption falls back
/// to its default).
#[derive(Debug, Clone, PartialEq)]
pub struct ReportCaptions {
    pub sketch_page_title: String,
    pub subject_label: String,
    pub sketch_label: String,
    pub summary_label: String,
    pub comment_table_labels: [String; 3],
}

impl Default for ReportCaptions {
    fn default() -> Self {
        ReportCaptions {
            sketch_page_title: "SKETCH/AREA TABLE ADDENDUM".to_owned(),
            subject_label: "SUBJECT INFO".to_owned(),
            sketch_label: "SKETCH".to_owned(),
            summary_label: "AREA CALCULATIONS SUMMARY".to_owned(),
            comment_table_labels: [
                "COMMENT TABLE 1".to_owned(),
                "COMMENT TABLE 2".to_owned(),
                "COMMENT TABLE 3".to_owned(),
            ],
        }
    }
}

/// What `SketchReport.renderPage` reads from `layout` (canvas-engine.js:
/// 3422-3441 and 3886-3897): the paper, the captions, and the three toggles.
/// `currentForm`, `calculationAddendum` and `deedCallsAddendum` are
/// normalised by the original but never read by its renderer (the form only
/// lands in a `data-current-form` attribute), so they have no effect here
/// either.
#[derive(Debug, Clone, PartialEq)]
pub struct ReportLayout {
    pub paper: PaperSize,
    pub show_sketch_comments: bool,
    pub show_comment_tables: bool,
    pub show_page_numbers: bool,
    pub captions: ReportCaptions,
}

impl Default for ReportLayout {
    fn default() -> Self {
        ReportLayout {
            paper: PaperSize::Legal,
            show_sketch_comments: true,
            show_comment_tables: true,
            show_page_numbers: true,
            captions: ReportCaptions::default(),
        }
    }
}

impl ReportLayout {
    /// `normalizeLayout`'s `||` fallbacks for blank captions.
    pub fn normalized(mut self) -> ReportLayout {
        let d = ReportCaptions::default();
        let fix = |s: &mut String, default: &str| {
            if s.is_empty() {
                *s = default.to_owned();
            }
        };
        fix(&mut self.captions.sketch_page_title, &d.sketch_page_title);
        fix(&mut self.captions.subject_label, &d.subject_label);
        fix(&mut self.captions.sketch_label, &d.sketch_label);
        fix(&mut self.captions.summary_label, &d.summary_label);
        for i in 0..3 {
            fix(
                &mut self.captions.comment_table_labels[i],
                &d.comment_table_labels[i],
            );
        }
        self
    }
}

/// Helvetica advance widths (1/1000 em) for ASCII 32..=126.
const HELVETICA: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556,
    556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667,
    611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667,
    667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500,
    222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];

/// Helvetica-Bold advance widths for ASCII 32..=126.
const HELVETICA_BOLD: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556,
    556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611, 975, 722, 722, 722, 722, 667,
    611, 778, 722, 278, 556, 722, 611, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667,
    667, 611, 333, 278, 333, 584, 556, 333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556,
    278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584,
];

/// The width of `text` in points (what `getTextWidth` would say).
fn text_width(font: Font, size: f64, text: &str) -> f64 {
    if let Some(width) = crate::report_fonts::width(font == Font::Bold, size, text) {
        return width;
    }
    let table = match font {
        Font::Regular => &HELVETICA,
        Font::Bold => &HELVETICA_BOLD,
    };
    let units: u32 = text
        .chars()
        .map(|c| {
            let code = c as u32;
            if (32..=126).contains(&code) {
                table[(code - 32) as usize] as u32
            } else {
                556
            }
        })
        .sum();
    units as f64 * size / 1000.0
}

/// `overflow: hidden` on a single line: drop trailing characters until the
/// text fits `max` points.
fn clip_text(font: Font, size: f64, text: &str, max: f64) -> String {
    let mut out: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    while !out.is_empty() && text_width(font, size, &out) > max {
        out.pop();
    }
    out
}

/// Greedy word wrap into lines no wider than `max` points (the sketch
/// comments are a block of wrapping text).
fn wrap_text(font: Font, size: f64, text: &str, max: f64) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            if !line.is_empty() && text_width(font, size, &candidate) > max {
                lines.push(std::mem::take(&mut line));
                line = word.to_owned();
            } else {
                line = candidate;
            }
        }
        lines.push(line);
    }
    lines
}

/// Space reserved below report geometry for readable general comments.
pub fn sketch_comment_inset(
    subject: &ReportSubject,
    layout: &ReportLayout,
    sketch_height: f64,
) -> f64 {
    if !layout.show_sketch_comments || subject.general_comments.is_empty() {
        return 0.0;
    }
    (wrap_text(
        Font::Regular,
        6.0,
        &subject.general_comments,
        SKETCH_WIDTH - 10.0,
    )
    .len() as f64
        * 6.9
        + 9.0)
        .min((sketch_height - 80.0).max(0.0))
}

/// `exportPdf` - the one-page report PDF bytes with the default Page Layout
/// (kept for callers that only choose the paper; `paper` defaults to legal
/// through `normalize_paper_size`).
pub fn export_pdf(
    page: &ExportPage,
    options: &ExportOptions,
    area_codes: &[crate::geometry::AreaCode],
    subject: &ReportSubject,
    paper: PaperSize,
) -> Vec<u8> {
    let layout = ReportLayout {
        paper,
        ..ReportLayout::default()
    };
    export_pdf_report(
        std::slice::from_ref(page),
        options,
        area_codes,
        subject,
        &layout,
    )
}

/// `SketchReport.render(model)` for every sketch page: one report page per
/// entry of `pages` (an empty list renders one blank page, like the
/// original), all sharing the subject block, the comment tables and the
/// layout.
pub fn export_pdf_report(
    pages: &[ExportPage],
    options: &ExportOptions,
    area_codes: &[crate::geometry::AreaCode],
    subject: &ReportSubject,
    layout: &ReportLayout,
) -> Vec<u8> {
    export_pdf_report_with_sketches(pages, options, area_codes, subject, layout, &[])
}

struct ReportPagePosition {
    number: usize,
    rich_sketch: bool,
    calculation_part: usize,
}

pub fn export_pdf_report_with_sketches(
    pages: &[ExportPage],
    options: &ExportOptions,
    area_codes: &[crate::geometry::AreaCode],
    subject: &ReportSubject,
    layout: &ReportLayout,
    sketches: &[Option<ReportSketch>],
) -> Vec<u8> {
    let layout = layout.clone().normalized();
    let paper = crate::print_layout::normalize_paper_size(Some(layout.paper));
    let (page_width, page_height) = paper.size();
    let blank = ExportPage::default();
    let list: Vec<&ExportPage> = if pages.is_empty() {
        vec![&blank]
    } else {
        pages.iter().collect()
    };
    let mut contents = Vec::new();
    let mut report_sketches = Vec::new();
    for (index, page) in list.iter().enumerate() {
        let calc = report_calculations(page, area_codes);
        let height = report_sketch_height(page, area_codes, subject, &layout);
        let capacity = calculation_capacity(height, subject, &layout);
        let parts = calc
            .summary
            .len()
            .max(calc.breakdown.len())
            .div_ceil(capacity)
            .max(1);
        for part in 0..parts {
            contents.push(report_page_content(
                page,
                options,
                area_codes,
                subject,
                &layout,
                ReportPagePosition {
                    number: contents.len() + 1,
                    rich_sketch: sketches.get(index).is_some_and(Option::is_some),
                    calculation_part: part,
                },
            ));
            report_sketches.push(sketches.get(index).cloned().flatten());
        }
    }
    assemble_pdf(&contents, page_width, page_height, &report_sketches)
}

/// One report page (`renderPage`).
fn report_page_content(
    page: &ExportPage,
    options: &ExportOptions,
    area_codes: &[crate::geometry::AreaCode],
    subject: &ReportSubject,
    layout: &ReportLayout,
    position: ReportPagePosition,
) -> Content {
    let ReportPagePosition {
        number: page_number,
        rich_sketch,
        calculation_part,
    } = position;
    let paper = layout.paper;
    let (page_width, page_height) = paper.size();
    let sketch_height = report_sketch_height(page, area_codes, subject, layout);
    let lower_top = SKETCH_BODY_TOP + sketch_height + comment_tables_height(subject, layout);
    let lower_height = (page_height - lower_top - LOWER_MARGIN).max(120.0);
    // Non-letter pages shift the 612pt column to center it.
    let horizontal_shift = (page_width - PAGE_WIDTH) / 2.0;

    let mut content = Content::new();
    // PDF y-up: top-left layout y -> page_height - y.
    let y = |top: f64| page_height - top;
    // A centred bold section title inside a `width`-wide band at `x`.
    let centred = |content: &mut Content, x: f64, width: f64, top: f64, size: f64, text: &str| {
        let w = text_width(Font::Bold, size, text);
        content.ops.push(Op::Text {
            x: x + (width - w) / 2.0,
            y: y(top),
            font: Font::Bold,
            size,
            text: text.to_owned(),
        });
    };

    // `.report-section-title`: a 12pt #d3d3d3 band with a .45 #555 rule
    // under it and the caption centred in bold 8pt.
    let band = |content: &mut Content, x: f64, top: f64, width: f64, text: &str| {
        content.ops.push(Op::SetGrayFill(GRAY_FILL));
        content.ops.push(Op::FillRect {
            x,
            y: y(top + 12.0),
            w: width,
            h: 12.0,
        });
        content.ops.push(Op::SetGrayStroke(GRAY_BORDER));
        content.ops.push(Op::SetLineWidth(0.45));
        content.ops.push(Op::Line {
            x1: x,
            y1: y(top + 12.0),
            x2: x + width,
            y2: y(top + 12.0),
        });
        content.ops.push(Op::SetGrayFill(0.0));
        centred(content, x, width, top + 9.0, 8.0, text);
    };

    // White page background.
    content.ops.push(Op::SetGrayFill(1.0));
    content.ops.push(Op::FillRect {
        x: 0.0,
        y: 0.0,
        w: page_width,
        h: page_height,
    });

    let x0 = 36.0 + horizontal_shift;

    // `.report-title`: the Sketch Page Title caption, centred over 540pt.
    content.ops.push(Op::SetGrayFill(0.0));
    centred(
        &mut content,
        x0,
        SKETCH_WIDTH,
        TITLE_TOP + TITLE_HEIGHT - 4.0,
        12.0,
        &layout.captions.sketch_page_title,
    );

    // `.report-subject`: a 540 x 72.2pt box (.45 #555); under the title band
    // come six 9.9pt grid rows (7.02pt text, 4.6pt side padding, a .35 #777
    // rule under all but the last). The column widths are the original's
    // `two` / `four` / `client` / `appraiser` templates, the last column
    // taking the rest; a label is followed by 4pt, then its value, and each
    // cell clips at its column edge (`overflow:hidden`).
    content.ops.push(Op::SetGrayStroke(GRAY_BORDER));
    content.ops.push(Op::SetLineWidth(0.45));
    content.ops.push(Op::StrokeRect {
        x: x0,
        y: y(SUBJECT_TOP + SUBJECT_HEIGHT),
        w: SKETCH_WIDTH,
        h: SUBJECT_HEIGHT,
    });
    band(
        &mut content,
        x0,
        SUBJECT_TOP,
        SKETCH_WIDTH,
        &layout.captions.subject_label,
    );
    const BORDER_W: f64 = 0.45;
    const ROW_PAD: f64 = 4.6;
    let inner_x = x0 + BORDER_W + ROW_PAD;
    let inner_w = SKETCH_WIDTH - 2.0 * BORDER_W - 2.0 * ROW_PAD;
    // (fixed column widths, up to four (label, value) cells, cell count).
    #[allow(clippy::type_complexity)]
    let subject_rows: [(&[f64], [(&str, &str); 4], usize); 6] = [
        (
            &[105.0],
            [
                ("File No.:", &subject.file_no),
                ("Parcel No.:", &subject.parcel_no),
                ("", ""),
                ("", ""),
            ],
            2,
        ),
        (
            &[],
            [
                ("Property Address:", &subject.property_address),
                ("", ""),
                ("", ""),
                ("", ""),
            ],
            1,
        ),
        (
            &[59.0, 103.0, 97.0],
            [
                ("City:", &subject.city),
                ("County:", &subject.county),
                ("State:", &subject.state),
                ("ZipCode:", &subject.zip_code),
            ],
            4,
        ),
        (
            &[],
            [("Owner:", &subject.owner), ("", ""), ("", ""), ("", "")],
            1,
        ),
        (
            &[98.0],
            [
                ("Client:", &subject.client),
                ("Client Address:", &subject.client_address),
                ("", ""),
                ("", ""),
            ],
            2,
        ),
        (
            &[136.0],
            [
                ("Appraiser Name:", &subject.appraiser_name),
                ("Inspection Date:", &subject.inspection_date),
                ("", ""),
                ("", ""),
            ],
            2,
        ),
    ];
    let rows_top = SUBJECT_TOP + BORDER_W + 12.0;
    for (index, (fixed, cells, count)) in subject_rows.iter().enumerate() {
        let top = rows_top + index as f64 * SECTION_ROW_HEIGHT;
        let mut col_x = inner_x;
        let mut used = 0.0;
        for (c, (label, value)) in cells.iter().take(*count).enumerate() {
            let width = if c < fixed.len() {
                fixed[c]
            } else {
                inner_w - used
            };
            let label_w = text_width(Font::Regular, 7.02, label);
            content.ops.push(Op::SetGrayFill(0.0));
            content.ops.push(Op::Text {
                x: col_x,
                y: y(top + 7.2),
                font: Font::Regular,
                size: 7.02,
                text: clip_text(Font::Regular, 7.02, label, width),
            });
            let room = width - label_w - 4.0;
            if room > 0.0 && !value.is_empty() {
                content.ops.push(Op::Text {
                    x: col_x + label_w + 4.0,
                    y: y(top + 7.2),
                    font: Font::Regular,
                    size: 7.02,
                    text: clip_text(Font::Regular, 7.02, value, room),
                });
            }
            col_x += width;
            used += width;
        }
        if index + 1 < subject_rows.len() {
            content.ops.push(Op::SetGrayStroke(GRAY_ROW));
            content.ops.push(Op::SetLineWidth(0.35));
            content.ops.push(Op::Line {
                x1: x0 + BORDER_W,
                y1: y(top + SECTION_ROW_HEIGHT),
                x2: x0 + SKETCH_WIDTH - BORDER_W,
                y2: y(top + SECTION_ROW_HEIGHT),
            });
        }
    }

    // `.report-sketch-heading`: the band between left and right .45 rules.
    band(
        &mut content,
        x0,
        SKETCH_HEADING_TOP,
        SKETCH_WIDTH,
        &layout.captions.sketch_label,
    );
    content.ops.push(Op::SetGrayStroke(GRAY_BORDER));
    content.ops.push(Op::SetLineWidth(0.45));
    for x in [x0, x0 + SKETCH_WIDTH] {
        content.ops.push(Op::Line {
            x1: x,
            y1: y(SKETCH_HEADING_TOP),
            x2: x,
            y2: y(SKETCH_HEADING_TOP + SKETCH_HEADING_HEIGHT),
        });
    }

    // Sketch body box + geometry.
    content.ops.push(Op::SetGrayStroke(GRAY_BORDER));
    content.ops.push(Op::SetLineWidth(0.45));
    content.ops.push(Op::StrokeRect {
        x: x0,
        y: y(SKETCH_BODY_TOP + sketch_height),
        w: SKETCH_WIDTH,
        h: sketch_height,
    });
    if rich_sketch {
        content.ops.push(Op::Image {
            x: x0,
            y: y(SKETCH_BODY_TOP + sketch_height),
            w: SKETCH_WIDTH,
            h: sketch_height,
        });
    } else {
        draw_sketch_geometry(
            &mut content,
            page,
            options,
            // The box's top edge in PDF y-up space: the fit transform measures
            // downward from it.
            y(SKETCH_BODY_TOP),
            sketch_height,
            x0,
            horizontal_shift,
        );
    }

    // `layout.showSketchComments`: the general comments, a wrapping 6pt
    // block pinned 4pt above the sketch box's bottom edge, 5pt in from the
    // sides; lines that would rise above the box are clipped.
    if layout.show_sketch_comments && !subject.general_comments.is_empty() {
        const SIZE: f64 = 6.0;
        const LINE: f64 = 6.9;
        let mut lines = wrap_text(
            Font::Regular,
            SIZE,
            &subject.general_comments,
            SKETCH_WIDTH - 10.0,
        );
        let inset = sketch_comment_inset(subject, layout, sketch_height);
        lines.truncate(((inset - 9.0).max(0.0) / LINE).round() as usize);
        let bottom = SKETCH_BODY_TOP + sketch_height - 4.0;
        let first_top = bottom - lines.len() as f64 * LINE;
        content.ops.push(Op::SetGrayFill(0.2));
        for (i, line) in lines.iter().enumerate() {
            let top = first_top + i as f64 * LINE;
            if top < SKETCH_BODY_TOP || line.is_empty() {
                continue;
            }
            content.ops.push(Op::Text {
                x: x0 + 5.0,
                y: y(top + 5.4),
                font: Font::Regular,
                size: SIZE,
                text: line.clone(),
            });
        }
    }

    // Populated comment tables have their own strip above the calculation pair.
    // Empty default comment grids must not displace the Breakdown section.
    let comment_height = comment_tables_height(subject, layout);
    if comment_height > 0.0 {
        let top = SKETCH_BODY_TOP + sketch_height;
        for index in 0..3 {
            let cx = x0 + index as f64 * 180.0;
            content.ops.push(Op::SetGrayStroke(GRAY_BORDER));
            content.ops.push(Op::StrokeRect {
                x: cx,
                y: y(top + comment_height),
                w: 180.0,
                h: comment_height,
            });
            band(
                &mut content,
                cx,
                top,
                180.0,
                &layout.captions.comment_table_labels[index],
            );
            if let Some(table) = subject.comment_tables.get(index) {
                for (row, value) in table.iter().take(10).enumerate() {
                    content.ops.push(Op::Text {
                        x: cx + 3.0,
                        y: y(top + 20.0 + row as f64 * 7.5),
                        font: Font::Regular,
                        size: 6.5,
                        text: clip_text(Font::Regular, 6.5, value, 174.0),
                    });
                }
            }
        }
    }

    // Equal Summary and Breakdown boxes, centered across the printable column.
    let calc = report_calculations(page, area_codes);
    let capacity = calculation_capacity(sketch_height, subject, layout);
    let offset = calculation_part * capacity;
    let half = SKETCH_WIDTH / 2.0;
    content.ops.push(Op::SetGrayStroke(GRAY_BORDER));
    content.ops.push(Op::SetLineWidth(0.45));
    content.ops.push(Op::StrokeRect {
        x: x0,
        y: y(lower_top + lower_height),
        w: SKETCH_WIDTH,
        h: lower_height,
    });
    content.ops.push(Op::Line {
        x1: x0 + half,
        y1: y(lower_top),
        x2: x0 + half,
        y2: y(lower_top + lower_height),
    });
    let summary_title = if calculation_part == 0 {
        layout.captions.summary_label.clone()
    } else {
        format!("{} (continued)", layout.captions.summary_label)
    };
    band(&mut content, x0, lower_top, half, &summary_title);
    band(
        &mut content,
        x0 + half,
        lower_top,
        half,
        if calculation_part == 0 {
            "AREA CALCULATIONS BREAKDOWN"
        } else {
            "AREA CALCULATIONS BREAKDOWN (continued)"
        },
    );
    let table_top = lower_top + 12.0;
    content.ops.push(Op::SetGrayFill(GRAY_FILL));
    content.ops.push(Op::FillRect {
        x: x0,
        y: y(table_top + TABLE_HEADER_HEIGHT),
        w: SKETCH_WIDTH,
        h: TABLE_HEADER_HEIGHT,
    });
    content.ops.push(Op::SetGrayFill(0.0));
    const COLS: [f64; 6] = [26.0, 92.0, 30.0, 39.0, 39.0, 44.0];
    let mut col_x = [0.0f64; 7];
    for i in 0..6 {
        col_x[i + 1] = col_x[i] + COLS[i];
    }
    for (index, header) in [
        "Code",
        "Description",
        "Factor",
        "Net Size",
        "Perimeter",
        "Net Totals",
    ]
    .iter()
    .enumerate()
    {
        let x = if index < 2 {
            x0 + col_x[index] + 3.0
        } else {
            x0 + col_x[index + 1] - 3.0 - text_width(Font::Bold, 6.2, header)
        };
        content.ops.push(Op::Text {
            x,
            y: y(table_top + 8.5),
            font: Font::Bold,
            size: 6.2,
            text: (*header).into(),
        });
    }
    content.ops.push(Op::Text {
        x: x0 + half + 5.0,
        y: y(table_top + 8.5),
        font: Font::Bold,
        size: 6.5,
        text: "Name / calculation".into(),
    });
    content.ops.push(Op::SetGrayStroke(GRAY_ROW));
    content.ops.push(Op::Line {
        x1: x0,
        y1: y(table_top + TABLE_HEADER_HEIGHT),
        x2: x0 + SKETCH_WIDTH,
        y2: y(table_top + TABLE_HEADER_HEIGHT),
    });
    for (index, values) in calc.summary.iter().skip(offset).take(capacity).enumerate() {
        let top = table_top + TABLE_HEADER_HEIGHT + index as f64 * TABLE_ROW_HEIGHT;
        for (column, value) in values.iter().enumerate() {
            let shown = clip_text(Font::Regular, 6.6, value, COLS[column] - 6.0);
            let x = if column < 2 {
                x0 + col_x[column] + 3.0
            } else {
                x0 + col_x[column + 1] - 3.0 - text_width(Font::Regular, 6.6, &shown)
            };
            content.ops.push(Op::Text {
                x,
                y: y(top + 8.0),
                font: Font::Regular,
                size: 6.6,
                text: shown,
            });
        }
    }
    for (index, line) in calc
        .breakdown
        .iter()
        .skip(offset)
        .take(capacity)
        .enumerate()
    {
        content.ops.push(Op::Text {
            x: x0 + half + 5.0,
            y: y(table_top + TABLE_HEADER_HEIGHT + index as f64 * TABLE_ROW_HEIGHT + 8.0),
            font: Font::Regular,
            size: 7.0,
            text: line.clone(),
        });
    }
    if calc.summary.is_empty() {
        for x in [x0 + 5.0, x0 + half + 5.0] {
            content.ops.push(Op::Text {
                x,
                y: y(table_top + 22.0),
                font: Font::Regular,
                size: 7.0,
                text: "No defined areas".into(),
            });
        }
    }
    // Keep a dedicated footer below all rows, including on continuation pages.
    let footer = lower_top + lower_height - 17.0;
    let rounded = format_thousands(calc.livable_total.round() as i64);
    for left in [x0, x0 + half] {
        content.ops.push(Op::SetGrayFill(0.0));
        content.ops.push(Op::Text {
            x: left + 5.0,
            y: y(footer),
            font: Font::Bold,
            size: 7.0,
            text: "Net LIVABLE".into(),
        });
        content.ops.push(Op::Text {
            x: left + half - 5.0 - text_width(Font::Bold, 7.0, &rounded),
            y: y(footer),
            font: Font::Bold,
            size: 7.0,
            text: rounded.clone(),
        });
        content.ops.push(Op::Text {
            x: left + 5.0,
            y: y(footer + 10.0),
            font: Font::Regular,
            size: 6.5,
            text: "cnt".into(),
        });
        content.ops.push(Op::Text {
            x: left + 20.0,
            y: y(footer + 10.0),
            font: Font::Regular,
            size: 6.5,
            text: calc.livable_count.to_string(),
        });
        content.ops.push(Op::Text {
            x: left + half - 5.0 - text_width(Font::Regular, 6.5, "(rounded)"),
            y: y(footer + 10.0),
            font: Font::Regular,
            size: 6.5,
            text: "(rounded)".into(),
        });
    }

    // `layout.showPageNumbers`: `Page N`, right edge on the 540pt column,
    // 21.5pt above the bottom.
    if layout.show_page_numbers {
        let label = format!("Page {page_number}");
        content.ops.push(Op::SetGrayFill(0.0));
        content.ops.push(Op::Text {
            x: x0 + SKETCH_WIDTH - text_width(Font::Regular, 6.0, &label),
            y: y(page_height - PAGE_NUMBER_OFFSET - 5.0),
            font: Font::Regular,
            size: 6.0,
            text: label,
        });
    }

    content
}

/// `Number.prototype.toLocaleString('en-US')` for an integer.
fn format_thousands(value: i64) -> String {
    let digits = value.abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if value < 0 {
        format!("-{out}")
    } else {
        out
    }
}

/// Draw the sketch geometry into the sketch box (PDF y-up space). The fit
/// transform is the report's min-fit with labels/symbols excluded from the
/// bounds; wall widths follow the print rules: exterior max(1.3,
/// thickness||2), interior max(0.65, thickness||1); dashed 8 5, dotted
/// 1.5 3.5.
fn draw_sketch_geometry(
    content: &mut Content,
    page: &ExportPage,
    options: &ExportOptions,
    box_top_pdf_y: f64,
    sketch_height: f64,
    x0: f64,
    _horizontal_shift: f64,
) {
    let fit = make_transform(page, sketch_height);
    // Report-space y-down point -> PDF y-up point in the box.
    let to_pdf = |world: Pt| -> (f64, f64) {
        let (sx, sy) = fit.point(world);
        (x0 + sx, box_top_pdf_y - sy)
    };
    // Polygon fills + outlines.
    for poly in &page.polygons {
        if poly.points.len() < 3 {
            continue;
        }
        let ring: Vec<(f64, f64)> = flattened_points(poly).iter().map(|p| to_pdf(*p)).collect();
        content.ops.push(Op::Polyline {
            points: ring.clone(),
            closed: true,
            fill: false,
        });
        let _ = options;
    }
    // Wall strokes.
    for line in &page.lines {
        let thickness = num_or_zero(line.line_thickness);
        let width = if line.wall_type == "exterior" {
            (if thickness != 0.0 { thickness } else { 2.0 }).max(1.3)
        } else {
            (if thickness != 0.0 { thickness } else { 1.0 }).max(0.65)
        };
        content.ops.push(Op::SetLineWidth(width));
        match line.line_style.as_deref() {
            Some("dashed") => content.ops.push(Op::SetDash(vec![8.0, 5.0])),
            Some("dotted") => content.ops.push(Op::SetDash(vec![1.5, 3.5])),
            _ => content.ops.push(Op::ResetDash),
        }
        let apex = curve_apex(line);
        let points: Vec<(f64, f64)> = if apex.is_some() {
            // Flattened arc, matching the report renderer's linePath.
            let faux_poly = crate::export_model::ExportPolygon {
                points: vec![line.start, line.end],
                edges: vec![
                    crate::geometry::Edge {
                        line_id: Some(line.id),
                        bow_angle: line.bow_angle,
                    },
                    crate::geometry::Edge::default(),
                ],
                ..Default::default()
            };
            flattened_points(&faux_poly)
                .iter()
                .map(|p| to_pdf(*p))
                .collect()
        } else {
            let runs: Vec<(f64, f64)> = if line.openings.is_empty() {
                vec![(0.0, 1.0)]
            } else {
                line.openings.clone()
            };
            runs.iter()
                .flat_map(|(start, end)| {
                    vec![
                        to_pdf(Pt::new(
                            line.start.x + (line.end.x - line.start.x) * start,
                            line.start.y + (line.end.y - line.start.y) * start,
                        )),
                        to_pdf(Pt::new(
                            line.start.x + (line.end.x - line.start.x) * end,
                            line.start.y + (line.end.y - line.start.y) * end,
                        )),
                    ]
                })
                .collect()
        };
        if points.len() >= 2 {
            content.ops.push(Op::Polyline {
                points,
                closed: false,
                fill: false,
            });
        }
    }
    content.ops.push(Op::ResetDash);
    // Area labels: name and net size, 9.72pt, centered on the label anchor.
    content.ops.push(Op::SetGrayFill(0.0));
    for poly in &page.polygons {
        if poly.area_type.is_empty() || poly.area_type == "UND" || poly.points.len() < 3 {
            continue;
        }
        let anchor = poly.label_position.unwrap_or_else(|| {
            let engine = crate::geometry::GeometryEngine::new();
            engine.area_label_anchor(&flattened_points(poly))
        });
        let (x, y) = to_pdf(anchor);
        content.ops.push(Op::Text {
            x: x - 20.0,
            y: y + 2.0,
            font: Font::Regular,
            size: 9.72,
            text: if poly.name.is_empty() {
                poly.area_type.clone()
            } else {
                poly.name.clone()
            },
        });
        let net = num_or_zero(poly.net.unwrap_or(poly.area));
        content.ops.push(Op::Text {
            x: x - 20.0,
            y: y - 12.5,
            font: Font::Regular,
            size: 9.72,
            text: format!("{net:.1} sf"),
        });
    }
    let _ = SKETCH_PAD;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_model::{ExportLine, ExportPage, ExportPolygon};
    use crate::geometry::Edge;

    fn rect_page() -> ExportPage {
        ExportPage {
            lines: vec![ExportLine {
                id: 1,
                start: Pt::new(0.0, 0.0),
                end: Pt::new(20.0, 0.0),
                wall_type: "exterior".to_owned(),
                color: None,
                line_thickness: 4.0,
                line_style: None,
                bow_angle: 0.0,
                openings: vec![],
            }],
            polygons: vec![ExportPolygon {
                points: vec![
                    Pt::new(0.0, 0.0),
                    Pt::new(20.0, 0.0),
                    Pt::new(20.0, 10.0),
                    Pt::new(0.0, 10.0),
                ],
                edges: vec![Edge::default(); 4],
                area_type: "LIV".to_owned(),
                name: "Living Area".to_owned(),
                net: Some(200.0),
                area: 200.0,
                gross: 200.0,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn pdf_to_string(bytes: &[u8]) -> String {
        // Decode the actual CID text operators through the emitted ToUnicode maps.
        let raw: String = bytes.iter().map(|b| *b as char).collect();
        let maps: Vec<std::collections::HashMap<String, String>> = raw
            .split("/CMapName /ReportUnicode")
            .skip(1)
            .filter_map(|chunk| {
                let cmap = chunk.split("endcmap").next()?;
                if !cmap.contains("beginbfchar") {
                    return None;
                }
                let mut map = std::collections::HashMap::new();
                for line in cmap.lines() {
                    let parts: Vec<_> = line.split_whitespace().collect();
                    if parts.len() == 2 && parts[0].starts_with('<') && parts[1].starts_with('<') {
                        let code = parts[0].trim_matches(['<', '>']);
                        let value = parts[1].trim_matches(['<', '>']);
                        let units: Vec<_> = (0..value.len())
                            .step_by(4)
                            .filter_map(|i| u16::from_str_radix(value.get(i..i + 4)?, 16).ok())
                            .collect();
                        map.insert(code.to_owned(), String::from_utf16_lossy(&units));
                    }
                }
                Some(map)
            })
            .collect();
        raw.lines()
            .map(|line| {
                if line.starts_with("BT ") && line.ends_with(" Tj ET") {
                    if let Some(start) = line.find('<') {
                        let end = line[start..].find('>').unwrap() + start;
                        let map = maps.get(usize::from(line.contains("/F2 "))).unwrap();
                        let hex = &line[start + 1..end];
                        let text: String = (0..hex.len())
                            .step_by(4)
                            .filter_map(|i| map.get(&hex[i..i + 4]))
                            .cloned()
                            .collect();
                        return format!(
                            "{}({}){}",
                            &line[..start],
                            pdf_escape(&text),
                            &line[end + 1..]
                        );
                    }
                }
                line.to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn pdf_page_size_constants_in_mediabox() {
        for (paper, width, height) in [
            (PaperSize::Legal, 612.0, 1008.0),
            (PaperSize::Letter, 612.0, 792.0),
            (PaperSize::A4, 595.28, 841.89),
        ] {
            let bytes = export_pdf(
                &rect_page(),
                &ExportOptions::default(),
                &[],
                &ReportSubject::default(),
                paper,
            );
            let text = pdf_to_string(&bytes);
            let expected = format!("/MediaBox [0 0 {width:.2} {height:.2}]");
            assert!(text.contains(&expected), "missing {expected} for {paper:?}");
        }
    }

    #[test]
    fn pdf_header_and_structure() {
        let bytes = export_pdf(
            &rect_page(),
            &ExportOptions::default(),
            &[],
            &ReportSubject::default(),
            PaperSize::Legal,
        );
        let text = pdf_to_string(&bytes);
        assert!(text.starts_with("%PDF-1.4"));
        assert!(text.contains("/Type /Catalog"));
        assert!(text.contains("/Type /Page"));
        assert!(text.contains("startxref"));
        assert!(text.trim_end().ends_with("%%EOF"));
    }

    #[test]
    fn pdf_print_layout_sections_present() {
        let subject = ReportSubject {
            file_no: "12-345".to_owned(),
            property_address: "100 Main St".to_owned(),
            comment_tables: vec![vec!["Line one".to_owned()], Vec::new(), Vec::new()],
            ..Default::default()
        };
        let bytes = export_pdf(
            &rect_page(),
            &ExportOptions::default(),
            &[],
            &subject,
            PaperSize::Legal,
        );
        let text = pdf_to_string(&bytes);
        assert!(text.contains("SKETCH/AREA TABLE ADDENDUM"));
        assert!(text.contains("SUBJECT INFO"));
        assert!(text.contains("SKETCH"));
        assert!(text.contains("AREA CALCULATIONS SUMMARY"));
        assert!(text.contains("COMMENT TABLE 1"));
        assert!(text.contains("COMMENT TABLE 3"));
        assert!(text.contains("Page 1"));
        assert!(text.contains("100 Main St"));
        assert!(text.contains("Line one"));
        assert!(text.contains("Living Area"));
        assert!(text.contains("200.0 sf"));
    }

    fn report(pages: &[ExportPage], subject: &ReportSubject, layout: &ReportLayout) -> String {
        let codes = crate::geometry::area_codes::default_area_codes();
        pdf_to_string(&export_pdf_report(
            pages,
            &ExportOptions::default(),
            &codes,
            subject,
            layout,
        ))
    }

    #[test]
    fn print_summary_and_breakdown_match_the_sample_on_every_paper() {
        let mut page = ExportPage::default();
        for (height, code, name) in [
            (30.0, "GLA1", "First Floor"),
            (22.0, "GLA2", "Second Floor"),
            (26.0, "BSMT", "Finished BSMT"),
        ] {
            let mut p = rect_page().polygons.remove(0);
            p.points = vec![
                Pt::new(0.0, 0.0),
                Pt::new(50.0, 0.0),
                Pt::new(50.0, height),
                Pt::new(0.0, height),
            ];
            p.area_type = code.into();
            p.name = name.into();
            p.gross = 50.0 * height;
            p.area = p.gross;
            p.net = Some(p.gross);
            p.perimeter = 2.0 * (50.0 + height);
            page.polygons.push(p);
        }
        for paper in [PaperSize::Letter, PaperSize::Legal, PaperSize::A4] {
            let layout = ReportLayout {
                paper,
                ..Default::default()
            };
            let text = report(&[page.clone()], &ReportSubject::default(), &layout);
            for expected in [
                "AREA CALCULATIONS SUMMARY",
                "AREA CALCULATIONS BREAKDOWN",
                "GLA1",
                "GLA2",
                "BSMT",
                "50.0 x 30.0 = 1500.0",
                "50.0 x 22.0 = 1100.0",
                "50.0 x 26.0 = 1300.0",
                "2,600",
            ] {
                assert!(text.contains(expected), "{paper:?}: {expected}");
            }
            assert!(text.contains("/Count 1"));
            assert!(!text.contains("COMMENT TABLE"));
            let printed_text = text
                .lines()
                .filter(|line| line.starts_with("BT "))
                .collect::<Vec<_>>()
                .join("\n");
            for forbidden in ["Copyright", "Apex Sketch", "Starcap", "Sketch by", "©"] {
                assert!(
                    !printed_text.contains(forbidden),
                    "unexpected branding: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn print_overflow_retains_all_area_calculations_and_keeps_footers_clear() {
        let mut page = rect_page();
        page.polygons[0].area_type = "GLA1".into();
        let template = page.polygons[0].clone();
        page.polygons = (0..85)
            .map(|i| {
                let mut p = template.clone();
                p.name = format!("Area {i:03}");
                p
            })
            .collect();
        for paper in [PaperSize::Letter, PaperSize::Legal, PaperSize::A4] {
            let layout = ReportLayout {
                paper,
                ..Default::default()
            };
            let text = report(&[page.clone()], &ReportSubject::default(), &layout);
            for i in 0..85 {
                assert!(
                    text.contains(&format!("Area {i:03} \\(net 200.0 sf\\)")),
                    "missing area {i} on {paper:?}"
                );
            }
            assert!(text.contains("BREAKDOWN \\(continued\\)"));
            assert!(!text.contains("/Count 1 "));
            let codes = crate::geometry::area_codes::default_area_codes();
            let content = report_page_content(
                &page,
                &ExportOptions::default(),
                &codes,
                &ReportSubject::default(),
                &layout,
                ReportPagePosition {
                    number: 1,
                    rich_sketch: false,
                    calculation_part: 0,
                },
            );
            for op in content.ops {
                if let Op::Text {
                    x,
                    y,
                    font,
                    size,
                    text,
                } = op
                {
                    assert!(y > 10.0 && y < paper.size().1, "text outside page: {text}");
                    assert!(
                        x >= 20.0 && x + text_width(font, size, &text) <= paper.size().0 - 20.0,
                        "text outside margins: {text}"
                    );
                }
            }
        }
    }

    #[test]
    fn print_breakdown_includes_triangles_curves_factors_and_deductions() {
        let mut page = rect_page();
        let p = &mut page.polygons[0];
        p.area_type = "GLA1".into();
        p.points = vec![Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), Pt::new(0.0, 10.0)];
        p.edges = vec![Edge::default(); 3];
        p.gross = 50.0;
        p.net = Some(20.0);
        p.effective_multiplier = Some(0.5);
        let text = report(
            &[page.clone()],
            &ReportSubject::default(),
            &ReportLayout::default(),
        );
        assert!(text.contains("0.5 x"));
        assert!(text.contains("Factor 0.50; deductions 10.0; net 20.0"));
        page.polygons[0].edges[0].bow_angle = 30.0;
        let text = report(&[page], &ReportSubject::default(), &ReportLayout::default());
        assert!(text.contains("arc r="));
    }

    #[test]
    fn the_report_has_one_page_per_sketch_page_with_numbers() {
        let text = report(
            &[rect_page(), rect_page()],
            &ReportSubject::default(),
            &ReportLayout::default(),
        );
        assert!(text.contains("/Count 2"));
        assert!(text.contains("(Page 1) Tj") && text.contains("(Page 2) Tj"));
        // No pages still renders one blank report page, like the original.
        let blank = report(&[], &ReportSubject::default(), &ReportLayout::default());
        assert!(blank.contains("/Count 1"));
    }

    #[test]
    fn captions_replace_the_defaults_and_blanks_fall_back() {
        let mut layout = ReportLayout::default();
        layout.captions.sketch_page_title = "MY ADDENDUM".into();
        layout.captions.subject_label = "THE SUBJECT".into();
        layout.captions.sketch_label = "PLAN VIEW".into();
        layout.captions.summary_label = "TOTALS".into();
        layout.captions.comment_table_labels = ["ONE".into(), String::new(), "THREE".into()];
        let subject = ReportSubject {
            comment_tables: vec![vec!["Caption test".into()]],
            ..Default::default()
        };
        let text = report(&[rect_page()], &subject, &layout);
        for shown in [
            "(MY ADDENDUM)",
            "(THE SUBJECT)",
            "(PLAN VIEW)",
            "(TOTALS)",
            "(ONE)",
            "(THREE)",
        ] {
            assert!(text.contains(shown), "{shown}");
        }
        assert!(!text.contains("(SKETCH/AREA TABLE ADDENDUM)"));
        assert!(!text.contains("(AREA CALCULATIONS SUMMARY)"));
        // A blank caption is `||`-defaulted.
        assert!(text.contains("(COMMENT TABLE 2)"));
    }

    #[test]
    fn the_toggles_remove_their_sections() {
        let subject = ReportSubject {
            general_comments: "Walkthrough note".into(),
            comment_tables: vec![vec!["Table line".into()], vec![], vec![]],
            ..Default::default()
        };
        let on = report(&[rect_page()], &subject, &ReportLayout::default());
        assert!(on.contains("(Walkthrough note)"));
        assert!(on.contains("(COMMENT TABLE 1)") && on.contains("(Table line)"));
        assert!(on.contains("(Page 1)"));

        let off = ReportLayout {
            show_sketch_comments: false,
            show_comment_tables: false,
            show_page_numbers: false,
            ..ReportLayout::default()
        };
        let text = report(&[rect_page()], &subject, &off);
        assert!(!text.contains("(Walkthrough note)"), "sketch comments off");
        assert!(!text.contains("(COMMENT TABLE 1)"), "comment tables off");
        assert!(!text.contains("(Table line)"));
        assert!(!text.contains("(Page 1)"), "page numbers off");
        // The summary keeps its title and now spans the whole lower box.
        assert!(text.contains("(AREA CALCULATIONS SUMMARY)"));
    }

    #[test]
    fn long_sketch_comments_wrap_inside_the_box() {
        let long = "word ".repeat(200);
        let subject = ReportSubject {
            general_comments: long,
            ..Default::default()
        };
        let text = report(&[rect_page()], &subject, &ReportLayout::default());
        let lines = text.matches("(word word").count();
        assert!(lines >= 2, "wrapped onto several lines, got {lines}");
    }

    #[test]
    fn net_livable_counts_the_gla_areas() {
        let mut page = rect_page();
        page.polygons[0].area_type = "GLA1".into();
        let text = report(&[page], &ReportSubject::default(), &ReportLayout::default());
        assert!(text.contains("(Net LIVABLE) Tj"));
        assert!(text.contains("(cnt) Tj"));
        assert!(text.contains(r"(\(rounded\)) Tj"));
        assert!(text.contains("(200) Tj"), "the rounded total");
    }

    #[test]
    fn the_subject_rows_use_the_originals_column_grid() {
        // x0 = 36 on legal; text starts 0.45 border + 4.6 padding in, so the
        // second column of the `two` row sits at 41.05 + 105 and the fourth
        // of the `four` row at 41.05 + 59 + 103 + 97.
        let subject = ReportSubject {
            file_no: "12-345".into(),
            parcel_no: "P-9".into(),
            zip_code: "98370".into(),
            property_address: "X".repeat(400),
            ..Default::default()
        };
        let text = report(&[rect_page()], &subject, &ReportLayout::default());
        assert!(text.contains("146.05"), "Parcel No. column");
        assert!(text.contains("300.05"), "ZipCode column");
        assert!(text.contains("(Parcel No.:) Tj") && text.contains("(P-9) Tj"));
        // The address clips at the row's edge instead of running off.
        let longest = text
            .lines()
            .filter(|l| l.contains("XXXX"))
            .map(|l| l.matches('X').count())
            .max()
            .unwrap();
        assert!(longest > 50 && longest < 400, "clipped to {longest}");
        // Labels are regular weight (`.report-subject-row b{font-weight:normal}`).
        assert!(text.contains("BT /F1 7.02 Tf"));
    }

    #[test]
    fn report_font_metrics_and_clipping() {
        let regular = crate::report_fonts::face(false).expect("a report font is installed");
        assert!(
            regular.name.contains("Verdana"),
            "native report uses Verdana: {}",
            regular.name
        );
        let face = ttf_parser::Face::parse(&regular.bytes, regular.index).unwrap();
        let expected = "Hello"
            .chars()
            .map(|c| {
                f64::from(
                    face.glyph_hor_advance(face.glyph_index(c).unwrap())
                        .unwrap(),
                )
            })
            .sum::<f64>()
            * 10.0
            / f64::from(face.units_per_em());
        assert!((text_width(Font::Regular, 10.0, "Hello") - expected).abs() < 1e-9);
        let clipped = clip_text(Font::Regular, 10.0, "Hello", 13.0);
        assert!(text_width(Font::Regular, 10.0, &clipped) <= 13.0);
        let next = "Hello".chars().nth(clipped.chars().count()).unwrap();
        assert!(text_width(Font::Regular, 10.0, &format!("{clipped}{next}")) > 13.0);
        for line in wrap_text(Font::Regular, 10.0, "aa bb cc", 20.0) {
            assert!(text_width(Font::Regular, 10.0, &line) <= 20.0);
        }
        assert_eq!(format_thousands(1234567), "1,234,567");
        assert_eq!(format_thousands(999), "999");
        assert_eq!(format_thousands(-1500), "-1,500");
    }

    #[test]
    fn pdf_escape_parens_and_backslash() {
        assert_eq!(pdf_escape("a(b)\\c"), "a\\(b\\)\\\\c");
    }

    #[test]
    fn pdf_wall_widths_follow_print_rules() {
        let mut page = rect_page();
        page.lines[0].line_thickness = 0.0;
        page.lines.push(ExportLine {
            id: 2,
            start: Pt::new(0.0, 0.0),
            end: Pt::new(0.0, 10.0),
            wall_type: "interior".to_owned(),
            color: None,
            line_thickness: 0.0,
            line_style: None,
            bow_angle: 0.0,
            openings: vec![],
        });
        page.lines.push(ExportLine {
            id: 3,
            start: Pt::new(0.0, 0.0),
            end: Pt::new(5.0, 0.0),
            wall_type: "exterior".to_owned(),
            color: None,
            line_thickness: 0.0,
            line_style: Some("dashed".to_owned()),
            bow_angle: 0.0,
            openings: vec![],
        });
        let bytes = export_pdf(
            &page,
            &ExportOptions::default(),
            &[],
            &ReportSubject::default(),
            PaperSize::Legal,
        );
        let text = pdf_to_string(&bytes);
        // Exterior default thickness 0 -> max(1.3, 2) = 2.0.
        assert!(text.contains("2.0000 w"));
        // Interior default -> max(0.65, 1) = 1.0.
        assert!(text.contains("1.0000 w"));
        // Dashed -> [8.00 5.00] 0 d.
        assert!(text.contains("[8.00 5.00] 0 d"));
    }
}
