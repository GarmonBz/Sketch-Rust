//! Print/PDF layout — clean-room port of the standalone HTML's
//! `SketchReport` renderer (`window.SKETCH_REPORT` section of
//! `00) sketch standalone.html`, lines ~10105–10600): paper sizes, sketch
//! heights, the fixed 540pt sketch box, and the min-fit transform with the
//! y-down mapping. Fit bounds come from line endpoints + flattened polygon
//! rings ONLY — labels/symbols are intentionally excluded so a stray label
//! never shrinks the fit.

use crate::export_model::{ExportPage, ExportSolid};
use crate::geometry::Pt;

/// PDF points per inch page constants (`PAGE_WIDTH`/`PAGE_HEIGHT` = legal).
pub const PAGE_WIDTH: f64 = 612.0;
pub const PAGE_HEIGHT: f64 = 1008.0;
/// The sketch box is 540pt wide with a 20pt interior pad (`SKETCH_WIDTH`,
/// `SKETCH_PAD`).
pub const SKETCH_WIDTH: f64 = 540.0;
pub const SKETCH_HEIGHT: f64 = 392.0;
pub const SKETCH_PAD: f64 = 20.0;
/// Lower-section bottom margin (`LOWER_MARGIN`).
pub const LOWER_MARGIN: f64 = 37.0;

/// Sketch body height per paper size (`SKETCH_HEIGHTS`).
pub const SKETCH_HEIGHTS: [(PaperSize, f64); 3] = [
    (PaperSize::Letter, 392.0),
    (PaperSize::Legal, 608.0),
    (PaperSize::A4, 442.0),
];

/// Paper sizes (`PAGE_SIZES`): width/height in pt plus the CSS `@page` size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperSize {
    Legal,
    Letter,
    A4,
}

impl PaperSize {
    pub fn size(self) -> (f64, f64) {
        match self {
            PaperSize::Legal => (612.0, 1008.0),
            PaperSize::Letter => (612.0, 792.0),
            PaperSize::A4 => (595.28, 841.89),
        }
    }

    /// The `@page{size:…}` CSS value.
    pub fn css(self) -> &'static str {
        match self {
            PaperSize::Legal => "legal portrait",
            PaperSize::Letter => "letter portrait",
            PaperSize::A4 => "A4 portrait",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PaperSize::Legal => "Legal",
            PaperSize::Letter => "Letter",
            PaperSize::A4 => "A4",
        }
    }

    /// The sketch body height (`SKETCH_HEIGHTS` lookup; default legal).
    pub fn sketch_height(self) -> f64 {
        for (size, height) in SKETCH_HEIGHTS {
            if size == self {
                return height;
            }
        }
        SKETCH_HEIGHT
    }
}

/// `normalizeLayout` — an unknown paper size falls back to legal.
pub fn normalize_paper_size(paper: Option<PaperSize>) -> PaperSize {
    paper.unwrap_or(PaperSize::Legal)
}

/// Fit transform for the sketch box (`getBounds` + `makeTransform`):
/// y-down mapping `y_screen = offsetY + (maxY - world.y) * scale`, min-fit
/// scale centered in both axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SketchFit {
    pub bounds_min: Pt,
    pub bounds_max: Pt,
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

impl SketchFit {
    /// Screen point for a world point, inside the sketch box (pt units).
    pub fn point(&self, world: Pt) -> (f64, f64) {
        (
            self.offset_x + (world.x - self.bounds_min.x) * self.scale,
            self.offset_y + (self.bounds_max.y - world.y) * self.scale,
        )
    }
}

/// `getBounds(page, options)` — the report fit bounds. Line endpoints and
/// flattened polygon rings only; wall solids and dimension chains stretch
/// past the centerlines so they are included when present. Labels and
/// symbols are excluded. Degenerate spans widen to 1 foot; empty defaults to
/// {0,40,0,28}.
pub fn get_bounds(page: &ExportPage) -> (Pt, Pt) {
    let mut min = Pt::new(f64::INFINITY, f64::INFINITY);
    let mut max = Pt::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut expand = |p: Pt| {
        if p.x.is_finite() && p.y.is_finite() {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
    };
    for line in &page.lines {
        expand(line.start);
        expand(line.end);
    }
    let engine = crate::geometry::GeometryEngine::new();
    for poly in &page.polygons {
        for p in engine.flatten_ring(&poly.points, &poly.edges, 0.02) {
            expand(p);
        }
    }
    for solid in &page.wall_solids {
        for piece in &solid.pieces {
            for p in piece {
                expand(*p);
            }
        }
    }
    if min.x.is_infinite() || min.y.is_infinite() {
        return (Pt::new(0.0, 0.0), Pt::new(40.0, 28.0));
    }
    if max.x - min.x < 1.0 {
        min.x -= 0.5;
        max.x += 0.5;
    }
    if max.y - min.y < 1.0 {
        min.y -= 0.5;
        max.y += 0.5;
    }
    (min, max)
}

/// `makeTransform(page, sketchHeight, options)` — the min-fit scale centered
/// in both axes with the 20pt pad.
pub fn make_transform(page: &ExportPage, sketch_height: f64) -> SketchFit {
    let (bounds_min, bounds_max) = get_bounds(page);
    let pad_x = SKETCH_PAD;
    let pad_y = SKETCH_PAD;
    let scale = ((SKETCH_WIDTH - pad_x * 2.0) / (bounds_max.x - bounds_min.x))
        .min((sketch_height - pad_y * 2.0) / (bounds_max.y - bounds_min.y));
    let drawn_width = (bounds_max.x - bounds_min.x) * scale;
    let drawn_height = (bounds_max.y - bounds_min.y) * scale;
    SketchFit {
        bounds_min,
        bounds_max,
        scale,
        offset_x: (SKETCH_WIDTH - drawn_width) / 2.0,
        offset_y: (sketch_height - drawn_height) / 2.0,
    }
}

/// `calculationRows` — the grouped AREA CALCULATIONS SUMMARY rows: group code
/// on the first row of each group, running group total on the last.
#[derive(Debug, Clone, PartialEq)]
pub struct CalculationRow {
    pub code: String,
    pub description: String,
    pub factor: f64,
    pub gross: f64,
    pub perimeter: f64,
    /// `None` when the row is not its group's total row.
    pub total: Option<f64>,
}

pub fn calculation_rows(page: &ExportPage, area_codes: &[AreaCode2]) -> Vec<CalculationRow> {
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for (index, poly) in page.polygons.iter().enumerate() {
        if poly.area_type.is_empty() || poly.area_type == "UND" || poly.points.len() < 3 {
            continue;
        }
        let group = area_codes
            .iter()
            .find(|c| c.code == poly.area_type)
            .map(|c| c.group.clone())
            .unwrap_or_else(|| poly.area_type.clone());
        match groups.iter_mut().find(|(key, _)| *key == group) {
            Some((_, items)) => items.push(index),
            None => groups.push((group, vec![index])),
        }
    }
    let mut rows = Vec::new();
    for (group, items) in &groups {
        let total: f64 = items
            .iter()
            .map(|i| page.polygons[*i].net.unwrap_or(0.0))
            .sum();
        for (position, index) in items.iter().enumerate() {
            let poly = &page.polygons[*index];
            let definition = area_codes.iter().find(|c| c.code == poly.area_type);
            let last = position == items.len() - 1;
            rows.push(CalculationRow {
                code: if position == 0 {
                    group.clone()
                } else {
                    String::new()
                },
                description: if poly.name.is_empty() {
                    definition
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| poly.area_type.clone())
                } else {
                    poly.name.clone()
                },
                factor: poly
                    .effective_multiplier
                    .or(definition.map(|c| c.multiplier))
                    .unwrap_or(1.0),
                gross: poly.gross,
                perimeter: poly.perimeter,
                total: if last { Some(total) } else { None },
            });
        }
    }
    rows
}

/// A minimal area-code definition the report renderer reads (subset of
/// `AreaCode` the calculation rows consume).
#[derive(Debug, Clone, PartialEq)]
pub struct AreaCode2 {
    pub code: String,
    pub name: String,
    pub group: String,
    pub multiplier: f64,
}

/// Convert the full `AreaCode` into the report subset.
pub fn area_code_subset(code: &crate::geometry::AreaCode) -> AreaCode2 {
    AreaCode2 {
        code: code.code.clone(),
        name: code.name.clone(),
        group: code.group.clone(),
        multiplier: code.multiplier,
    }
}

/// Wall-solid pieces flattened for the fit bounds (`ExportSolid` alias for
/// callers that build solids directly).
pub type FitSolid = ExportSolid;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_model::{ExportLine, ExportPage, ExportPolygon};
    use crate::geometry::Edge;

    fn rect_page(x0: f64, y0: f64, x1: f64, y1: f64) -> ExportPage {
        ExportPage {
            lines: vec![ExportLine {
                id: 1,
                start: Pt::new(x0, y0),
                end: Pt::new(x1, y1),
                wall_type: "exterior".to_owned(),
                color: None,
                line_thickness: 4.0,
                line_style: None,
                bow_angle: 0.0,
                openings: vec![],
            }],
            polygons: vec![ExportPolygon {
                points: vec![
                    Pt::new(x0, y0),
                    Pt::new(x1, y0),
                    Pt::new(x1, y1),
                    Pt::new(x0, y1),
                ],
                edges: vec![Edge::default(); 4],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn paper_size_constants_match_the_standalone() {
        assert_eq!(PaperSize::Legal.size(), (612.0, 1008.0));
        assert_eq!(PaperSize::Letter.size(), (612.0, 792.0));
        assert_eq!(PaperSize::A4.size(), (595.28, 841.89));
        assert_eq!(PaperSize::Legal.css(), "legal portrait");
        assert_eq!(PaperSize::Letter.css(), "letter portrait");
        assert_eq!(PaperSize::A4.css(), "A4 portrait");
        assert_eq!(SKETCH_WIDTH, 540.0);
        assert_eq!(SKETCH_PAD, 20.0);
        assert_eq!(SKETCH_HEIGHT, 392.0);
        assert_eq!(PaperSize::Legal.sketch_height(), 608.0);
        assert_eq!(PaperSize::Letter.sketch_height(), 392.0);
        assert_eq!(PaperSize::A4.sketch_height(), 442.0);
        assert_eq!(normalize_paper_size(None), PaperSize::Legal);
    }

    #[test]
    fn fit_scale_known_rectangle_expected_screen_box() {
        // A 40x20 ft rectangle in a legal sketch box:
        // scale = min(500/40, 568/20) = 12.5
        let page = rect_page(0.0, 0.0, 40.0, 20.0);
        let fit = make_transform(&page, PaperSize::Legal.sketch_height());
        assert!((fit.scale - 12.5).abs() < 1e-9);
        // Drawn 500x250; legal box height 608 -> centered offsets.
        assert!((fit.offset_x - 20.0).abs() < 1e-9);
        assert!((fit.offset_y - (608.0 - 250.0) / 2.0).abs() < 1e-9);
        // y-down mapping: world top (y=20) maps to the small screen y.
        let (top_x, top_y) = fit.point(Pt::new(0.0, 20.0));
        assert!((top_x - 20.0).abs() < 1e-9);
        assert!((top_y - fit.offset_y).abs() < 1e-9);
        let (bottom_x, bottom_y) = fit.point(Pt::new(40.0, 0.0));
        assert!((bottom_x - 520.0).abs() < 1e-9);
        assert!((bottom_y - (fit.offset_y + 250.0)).abs() < 1e-9);
    }

    #[test]
    fn fit_bounds_exclude_labels_and_symbols() {
        let mut page = rect_page(0.0, 0.0, 10.0, 8.0);
        page.labels.push(crate::export_model::ExportLabel {
            x: 500.0,
            y: 500.0,
            text: "stray".to_owned(),
            ..Default::default()
        });
        page.symbols.push(crate::export_model::ExportSymbol {
            x: -300.0,
            y: -300.0,
            symbol_type: "shapes/circle".to_owned(),
            ..Default::default()
        });
        let (min, max) = get_bounds(&page);
        assert!((min.x - 0.0).abs() < 1e-9 && (max.x - 10.0).abs() < 1e-9);
        assert!((min.y - 0.0).abs() < 1e-9 && (max.y - 8.0).abs() < 1e-9);
    }

    #[test]
    fn fit_bounds_empty_default_and_degenerate_span() {
        let (min, max) = get_bounds(&ExportPage::default());
        assert_eq!(min, Pt::new(0.0, 0.0));
        assert_eq!(max, Pt::new(40.0, 28.0));
        let (dmin, dmax) = get_bounds(&rect_page(5.0, 5.0, 5.0, 5.0));
        // Degenerate spans widen to 1 foot.
        assert!((dmax.x - dmin.x - 1.0).abs() < 1e-9);
        assert!((dmax.y - dmin.y - 1.0).abs() < 1e-9);
    }

    #[test]
    fn calculation_rows_group_code_and_totals() {
        let mut page = rect_page(0.0, 0.0, 10.0, 8.0);
        page.polygons[0].area_type = "LIV".to_owned();
        page.polygons[0].name = "Living Area".to_owned();
        page.polygons[0].net = Some(80.0);
        let codes = vec![AreaCode2 {
            code: "LIV".to_owned(),
            name: "Living Area".to_owned(),
            group: "LIV".to_owned(),
            multiplier: 1.0,
        }];
        let rows = calculation_rows(&page, &codes);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].code, "LIV");
        assert_eq!(rows[0].description, "Living Area");
        assert_eq!(rows[0].total, Some(80.0));
    }
}
