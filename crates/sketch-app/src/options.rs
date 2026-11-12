//! `state.options` — the sketch's settings (state-init.js:89). The model is
//! FLAT with the original's exact keys (the JSON rides `settings.options` in
//! the save model), plus `Units` flattened in as the original's
//! `unitSystem`/`decimalPrecision`/`stripTrailingZeroes`/`areaLabelSuffix`
//! quartet. The dialog (options_dialog.rs) edits it; `applyOptions`
//! (sketch-app.js:524) clamps and mirrors on apply.

use serde::{Deserialize, Serialize};

/// Length/area formatting (metric, precision, stripping, area suffix) — the
/// canonical type lives in `format.rs` (its manual serde keeps the original's
/// flat `unitSystem`/`decimalPrecision`/`stripTrailingZeroes`/`areaLabelSuffix`
/// keys, which `#[serde(flatten)]` inlines below).
pub use crate::format::Units;

// --- The dialog key enums (collectOptions' lists) ---------------------------

/// The dialog's checkbox entries, in `booleans` order (plus the format
/// checkboxes, which map to `imageFormats`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolKey {
    PromptOnDeletion,
    InkConfirmation,
    AutoPlacement,
    SameColorAsLine,
    AlignmentLines,
    AlignmentPoints,
    TestCloseIndicators,
    GridEnabled,
    MinimumDimensionEnabled,
    AutoShade,
    SelectionHighlight,
    AutoSaveEnabled,
    AppendPageNumber,
    DisableVideoTutorial,
    AutoHealFiles,
    DimensionBold,
    DimensionItalic,
    DimensionUnderline,
    TextBold,
    TextItalic,
    TextUnderline,
    AutoPostBold,
    AutoPostItalic,
    AutoPostUnderline,
    CircleAreaCalcs,
    DisplayOnClose,
    DefineFirst,
    AutoSubtractOverlapped,
    QuickShapeDefine,
    Redefine,
    PanZoomDelete,
    SaveImageWithSketch,
    SplitWallsAtJunctions,
    WallSolids,
}

/// The format checkboxes (`formatPNG`/`formatJPG`/`formatBMP` →
/// `imageFormats`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKey {
    Png,
    Jpg,
    Bmp,
}

impl FormatKey {
    /// The lowercase `imageFormats` member.
    pub fn key(self) -> &'static str {
        match self {
            FormatKey::Png => "png",
            FormatKey::Jpg => "jpg",
            FormatKey::Bmp => "bmp",
        }
    }
}

/// The numeric fields (`numbers` + `inkRecognition`, which the dialog parses
/// off its select).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumKey {
    InkRecognition,
    DecimalPrecision,
    DimensionOption,
    OrthoAngle,
    MinimumDimension,
    AutoShadePercent,
    AutoSaveInterval,
    ImageDepth,
    DimensionFontSize,
    TextFontSize,
    AutoPostFontSize,
    TextBackgroundOpacity,
    ExteriorWidth,
    InteriorWidth,
    ExteriorThickness,
    InteriorThickness,
    LabelOpacity,
    ImageQuality,
    PointerSize,
}

/// The string fields (free text / selects / colour inputs as text).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrKey {
    InputMode,
    StepUnit,
    AreaLabelSuffix,
    AlignmentColor,
    AlignmentStyle,
    DistoDiscovery,
    DistoComPort,
    DistoRounding,
    DistoDrawingMode,
    ImageSize,
    DimensionFont,
    TextFont,
    TextColor,
    AutoPostFont,
    TextBackground,
    ExteriorColor,
    InteriorColor,
    WitnessBaseColor,
    WitnessInteriorColor,
    WitnessCurveColor,
}

// --- The flat model ---------------------------------------------------------

// Field-default helpers (serde needs a fn per default with arguments).
fn f85() -> f64 {
    8.5
}
fn f5() -> f64 {
    5.0
}
fn f12() -> f64 {
    12.0
}
fn f50() -> f64 {
    50.0
}
fn f4() -> f64 {
    4.0
}
fn f2() -> f64 {
    2.0
}
fn f16() -> f64 {
    16.0
}
fn f0() -> f64 {
    0.0
}
fn s(v: &'static str) -> String {
    v.to_string()
}
fn image_formats_default() -> Vec<String> {
    vec!["png".to_string()]
}

/// `state.options` — flat, the original's keys verbatim
/// (`#[serde(flatten)]` on `units` keeps the saved JSON flat too).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Options {
    // General
    pub input_mode: String, // pen-ink | pen-mouse | touch
    pub prompt_on_deletion: bool,
    pub ink_confirmation: bool,
    pub ink_recognition: f64, // 0.1 | 0.25 | 0.5
    #[serde(flatten)]
    pub units: Units,
    pub step_unit: String, // tenths | half | whole
    pub dimension_option: f64,
    pub ortho_angle: f64,
    pub auto_placement: bool,
    pub same_color_as_line: bool,

    // Display
    pub alignment_lines: bool,
    pub alignment_points: bool,
    pub test_close_indicators: bool,
    pub alignment_color: String,
    pub alignment_style: String, // dash | solid
    pub grid_background: String,
    pub major_grid_color: String,
    pub minor_grid_color: String,
    pub minimum_dimension_enabled: bool,
    pub minimum_dimension: f64,
    pub auto_shade: bool,
    pub auto_shade_percent: f64,
    pub selection_highlight: bool,
    pub grid_enabled: bool,

    // Hardware
    pub disto_discovery: String, // automatic | manual
    pub disto_com_port: String,
    pub disto_rounding: String, // none | nearestHalf | nearestHundredth | nearestTenth | nearestWhole
    pub disto_drawing_mode: String, // standard | continuous

    // System
    pub auto_save_enabled: bool,
    pub auto_save_interval: f64,
    pub image_size: String,
    pub image_depth: f64, // 8 | 24
    pub append_page_number: bool,
    pub image_formats: Vec<String>,
    pub disable_video_tutorial: bool,
    pub auto_heal_files: bool,

    // Fonts
    pub dimension_font: String,
    pub dimension_font_size: f64,
    pub dimension_bold: bool,
    pub dimension_italic: bool,
    pub dimension_underline: bool,
    pub text_font: String,
    pub text_font_size: f64,
    pub text_bold: bool,
    pub text_italic: bool,
    pub text_underline: bool,
    pub text_color: String,
    pub auto_post_font: String,
    pub auto_post_font_size: f64,
    pub auto_post_bold: bool,
    pub auto_post_italic: bool,
    pub auto_post_underline: bool,
    pub circle_area_calcs: bool,
    pub text_background: String,
    pub text_background_opacity: f64,

    // Drawing
    pub exterior_color: String,
    pub interior_color: String,
    pub exterior_width: f64,
    pub interior_width: f64,
    pub wall_solids: bool,
    pub exterior_thickness: f64,
    pub interior_thickness: f64,
    pub witness_base_color: String,
    pub witness_interior_color: String,
    pub witness_curve_color: String,
    pub label_opacity: f64,
    pub display_on_close: bool,
    pub define_first: bool,
    pub auto_subtract_overlapped: bool,
    pub quick_shape_define: bool,
    pub redefine: bool,
    pub split_walls_at_junctions: bool,
    pub pan_zoom_delete: bool,
    pub save_image_with_sketch: bool,
    pub image_quality: f64,
    pub pointer_size: f64, // 128 | 64 | 32
    pub suppress_minimize_confirmation: bool,

    /// `curveMode` — this port keeps only the 'height' behaviour toggle
    /// (Shift+wheel); the original stores the whole state key elsewhere.
    pub curve_mode_height: bool,

    /// PDF/print paper size (this port's addition; never saved).
    #[serde(skip)]
    pub paper_size: sketch_core::print_layout::PaperSize,
}

/// The dialog's lists — fonts and font sizes, in the original's order.
pub const FONTS: &[&str] = &[
    "Arial",
    "Calibri",
    "Georgia",
    "Segoe UI",
    "Times New Roman",
    "Verdana",
];
pub const FONT_SIZES: &[f64] = &[
    8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 28.0, 36.0, 48.0, 72.0,
];

impl Default for Options {
    fn default() -> Self {
        // state-init.js: values in file order.
        Self {
            input_mode: s("pen-ink"),
            prompt_on_deletion: false,
            ink_confirmation: true,
            ink_recognition: 0.1,
            units: Units::default(),
            step_unit: s("whole"),
            dimension_option: f85(),
            ortho_angle: f5(),
            auto_placement: true,
            same_color_as_line: false,
            alignment_lines: true,
            alignment_points: true,
            test_close_indicators: true,
            alignment_color: s("#808080"),
            alignment_style: s("dash"),
            grid_background: s("#FFFFFF"),
            major_grid_color: s("#334155"),
            minor_grid_color: s("#1f2937"),
            minimum_dimension_enabled: false,
            minimum_dimension: f12(),
            auto_shade: false,
            auto_shade_percent: f50(),
            selection_highlight: true,
            grid_enabled: true,
            disto_discovery: s("automatic"),
            disto_com_port: String::new(),
            disto_rounding: s("none"),
            disto_drawing_mode: s("standard"),
            auto_save_enabled: true,
            auto_save_interval: f5(),
            image_size: s("2048x1536"),
            image_depth: 24.0,
            append_page_number: true,
            image_formats: image_formats_default(),
            disable_video_tutorial: false,
            auto_heal_files: true,
            dimension_font: s("Arial"),
            dimension_font_size: f12(),
            dimension_bold: false,
            dimension_italic: false,
            dimension_underline: false,
            text_font: s("Arial"),
            text_font_size: f12(),
            text_bold: false,
            text_italic: false,
            text_underline: false,
            text_color: s("#e2e8f0"),
            auto_post_font: s("Arial"),
            auto_post_font_size: f16(),
            auto_post_bold: false,
            auto_post_italic: false,
            auto_post_underline: false,
            circle_area_calcs: false,
            text_background: s("#111111"),
            text_background_opacity: f0(),
            exterior_color: s("#2B6CB0"),
            interior_color: s("#63B3ED"),
            exterior_width: f4(),
            interior_width: f2(),
            wall_solids: false, // see the doc comment on `wall_solids_enabled`
            exterior_thickness: 6.0,
            interior_thickness: 4.0,
            witness_base_color: s("#ef4444"),
            witness_interior_color: s("#2563eb"),
            witness_curve_color: s("#22c55e"),
            label_opacity: f50(),
            display_on_close: false,
            define_first: false,
            auto_subtract_overlapped: true,
            quick_shape_define: false,
            redefine: false,
            split_walls_at_junctions: false, // inert in the original (off by default)
            pan_zoom_delete: true,
            save_image_with_sketch: false,
            image_quality: 85.0,
            pointer_size: 32.0,
            suppress_minimize_confirmation: false,
            curve_mode_height: false,
            paper_size: sketch_core::print_layout::PaperSize::Legal,
        }
    }
}

impl Options {
    /// `wallSolidsEnabled()`: the option is on, at least one thickness is
    /// positive, and the WallSolids module is present (always, for us).
    pub fn wall_solids_enabled(&self) -> bool {
        self.wall_solids && (self.exterior_thickness > 0.0 || self.interior_thickness > 0.0)
    }

    // --- Keyed access for the dialog ---------------------------------------

    pub fn get_bool(&self, key: BoolKey) -> bool {
        match key {
            BoolKey::PromptOnDeletion => self.prompt_on_deletion,
            BoolKey::InkConfirmation => self.ink_confirmation,
            BoolKey::AutoPlacement => self.auto_placement,
            BoolKey::SameColorAsLine => self.same_color_as_line,
            BoolKey::AlignmentLines => self.alignment_lines,
            BoolKey::AlignmentPoints => self.alignment_points,
            BoolKey::TestCloseIndicators => self.test_close_indicators,
            BoolKey::GridEnabled => self.grid_enabled,
            BoolKey::MinimumDimensionEnabled => self.minimum_dimension_enabled,
            BoolKey::AutoShade => self.auto_shade,
            BoolKey::SelectionHighlight => self.selection_highlight,
            BoolKey::AutoSaveEnabled => self.auto_save_enabled,
            BoolKey::AppendPageNumber => self.append_page_number,
            BoolKey::DisableVideoTutorial => self.disable_video_tutorial,
            BoolKey::AutoHealFiles => self.auto_heal_files,
            BoolKey::DimensionBold => self.dimension_bold,
            BoolKey::DimensionItalic => self.dimension_italic,
            BoolKey::DimensionUnderline => self.dimension_underline,
            BoolKey::TextBold => self.text_bold,
            BoolKey::TextItalic => self.text_italic,
            BoolKey::TextUnderline => self.text_underline,
            BoolKey::AutoPostBold => self.auto_post_bold,
            BoolKey::AutoPostItalic => self.auto_post_italic,
            BoolKey::AutoPostUnderline => self.auto_post_underline,
            BoolKey::CircleAreaCalcs => self.circle_area_calcs,
            BoolKey::DisplayOnClose => self.display_on_close,
            BoolKey::DefineFirst => self.define_first,
            BoolKey::AutoSubtractOverlapped => self.auto_subtract_overlapped,
            BoolKey::QuickShapeDefine => self.quick_shape_define,
            BoolKey::Redefine => self.redefine,
            BoolKey::PanZoomDelete => self.pan_zoom_delete,
            BoolKey::SaveImageWithSketch => self.save_image_with_sketch,
            BoolKey::SplitWallsAtJunctions => self.split_walls_at_junctions,
            BoolKey::WallSolids => self.wall_solids,
        }
    }

    pub fn set_bool(&mut self, key: BoolKey, value: bool) {
        match key {
            BoolKey::PromptOnDeletion => self.prompt_on_deletion = value,
            BoolKey::InkConfirmation => self.ink_confirmation = value,
            BoolKey::AutoPlacement => self.auto_placement = value,
            BoolKey::SameColorAsLine => self.same_color_as_line = value,
            BoolKey::AlignmentLines => self.alignment_lines = value,
            BoolKey::AlignmentPoints => self.alignment_points = value,
            BoolKey::TestCloseIndicators => self.test_close_indicators = value,
            BoolKey::GridEnabled => self.grid_enabled = value,
            BoolKey::MinimumDimensionEnabled => self.minimum_dimension_enabled = value,
            BoolKey::AutoShade => self.auto_shade = value,
            BoolKey::SelectionHighlight => self.selection_highlight = value,
            BoolKey::AutoSaveEnabled => self.auto_save_enabled = value,
            BoolKey::AppendPageNumber => self.append_page_number = value,
            BoolKey::DisableVideoTutorial => self.disable_video_tutorial = value,
            BoolKey::AutoHealFiles => self.auto_heal_files = value,
            BoolKey::DimensionBold => self.dimension_bold = value,
            BoolKey::DimensionItalic => self.dimension_italic = value,
            BoolKey::DimensionUnderline => self.dimension_underline = value,
            BoolKey::TextBold => self.text_bold = value,
            BoolKey::TextItalic => self.text_italic = value,
            BoolKey::TextUnderline => self.text_underline = value,
            BoolKey::AutoPostBold => self.auto_post_bold = value,
            BoolKey::AutoPostItalic => self.auto_post_italic = value,
            BoolKey::AutoPostUnderline => self.auto_post_underline = value,
            BoolKey::CircleAreaCalcs => self.circle_area_calcs = value,
            BoolKey::DisplayOnClose => self.display_on_close = value,
            BoolKey::DefineFirst => self.define_first = value,
            BoolKey::AutoSubtractOverlapped => self.auto_subtract_overlapped = value,
            BoolKey::QuickShapeDefine => self.quick_shape_define = value,
            BoolKey::Redefine => self.redefine = value,
            BoolKey::PanZoomDelete => self.pan_zoom_delete = value,
            BoolKey::SaveImageWithSketch => self.save_image_with_sketch = value,
            BoolKey::SplitWallsAtJunctions => self.split_walls_at_junctions = value,
            BoolKey::WallSolids => self.wall_solids = value,
        }
    }

    pub fn get_num(&self, key: NumKey) -> f64 {
        match key {
            NumKey::InkRecognition => self.ink_recognition,
            NumKey::DecimalPrecision => self.units.decimal_precision,
            NumKey::DimensionOption => self.dimension_option,
            NumKey::OrthoAngle => self.ortho_angle,
            NumKey::MinimumDimension => self.minimum_dimension,
            NumKey::AutoShadePercent => self.auto_shade_percent,
            NumKey::AutoSaveInterval => self.auto_save_interval,
            NumKey::ImageDepth => self.image_depth,
            NumKey::DimensionFontSize => self.dimension_font_size,
            NumKey::TextFontSize => self.text_font_size,
            NumKey::AutoPostFontSize => self.auto_post_font_size,
            NumKey::TextBackgroundOpacity => self.text_background_opacity,
            NumKey::ExteriorWidth => self.exterior_width,
            NumKey::InteriorWidth => self.interior_width,
            NumKey::ExteriorThickness => self.exterior_thickness,
            NumKey::InteriorThickness => self.interior_thickness,
            NumKey::LabelOpacity => self.label_opacity,
            NumKey::ImageQuality => self.image_quality,
            NumKey::PointerSize => self.pointer_size,
        }
    }

    pub fn set_num(&mut self, key: NumKey, value: f64) {
        match key {
            NumKey::InkRecognition => self.ink_recognition = value,
            NumKey::DecimalPrecision => self.units.decimal_precision = value,
            NumKey::DimensionOption => self.dimension_option = value,
            NumKey::OrthoAngle => self.ortho_angle = value,
            NumKey::MinimumDimension => self.minimum_dimension = value,
            NumKey::AutoShadePercent => self.auto_shade_percent = value,
            NumKey::AutoSaveInterval => self.auto_save_interval = value,
            NumKey::ImageDepth => self.image_depth = value,
            NumKey::DimensionFontSize => self.dimension_font_size = value,
            NumKey::TextFontSize => self.text_font_size = value,
            NumKey::AutoPostFontSize => self.auto_post_font_size = value,
            NumKey::TextBackgroundOpacity => self.text_background_opacity = value,
            NumKey::ExteriorWidth => self.exterior_width = value,
            NumKey::InteriorWidth => self.interior_width = value,
            NumKey::ExteriorThickness => self.exterior_thickness = value,
            NumKey::InteriorThickness => self.interior_thickness = value,
            NumKey::LabelOpacity => self.label_opacity = value,
            NumKey::ImageQuality => self.image_quality = value,
            NumKey::PointerSize => self.pointer_size = value,
        }
    }

    pub fn get_str(&self, key: StrKey) -> String {
        match key {
            StrKey::InputMode => self.input_mode.clone(),
            StrKey::StepUnit => self.step_unit.clone(),
            StrKey::AreaLabelSuffix => self.units.area_label_suffix.clone(),
            StrKey::AlignmentColor => self.alignment_color.clone(),
            StrKey::AlignmentStyle => self.alignment_style.clone(),
            StrKey::DistoDiscovery => self.disto_discovery.clone(),
            StrKey::DistoComPort => self.disto_com_port.clone(),
            StrKey::DistoRounding => self.disto_rounding.clone(),
            StrKey::DistoDrawingMode => self.disto_drawing_mode.clone(),
            StrKey::ImageSize => self.image_size.clone(),
            StrKey::DimensionFont => self.dimension_font.clone(),
            StrKey::TextFont => self.text_font.clone(),
            StrKey::TextColor => self.text_color.clone(),
            StrKey::AutoPostFont => self.auto_post_font.clone(),
            StrKey::TextBackground => self.text_background.clone(),
            StrKey::ExteriorColor => self.exterior_color.clone(),
            StrKey::InteriorColor => self.interior_color.clone(),
            StrKey::WitnessBaseColor => self.witness_base_color.clone(),
            StrKey::WitnessInteriorColor => self.witness_interior_color.clone(),
            StrKey::WitnessCurveColor => self.witness_curve_color.clone(),
        }
    }

    pub fn set_str(&mut self, key: StrKey, value: String) {
        match key {
            StrKey::InputMode => self.input_mode = value,
            StrKey::StepUnit => self.step_unit = value,
            StrKey::AreaLabelSuffix => self.units.area_label_suffix = value,
            StrKey::AlignmentColor => self.alignment_color = value,
            StrKey::AlignmentStyle => self.alignment_style = value,
            StrKey::DistoDiscovery => self.disto_discovery = value,
            StrKey::DistoComPort => self.disto_com_port = value,
            StrKey::DistoRounding => self.disto_rounding = value,
            StrKey::DistoDrawingMode => self.disto_drawing_mode = value,
            StrKey::ImageSize => self.image_size = value,
            StrKey::DimensionFont => self.dimension_font = value,
            StrKey::TextFont => self.text_font = value,
            StrKey::TextColor => self.text_color = value,
            StrKey::AutoPostFont => self.auto_post_font = value,
            StrKey::TextBackground => self.text_background = value,
            StrKey::ExteriorColor => self.exterior_color = value,
            StrKey::InteriorColor => self.interior_color = value,
            StrKey::WitnessBaseColor => self.witness_base_color = value,
            StrKey::WitnessInteriorColor => self.witness_interior_color = value,
            StrKey::WitnessCurveColor => self.witness_curve_color = value,
        }
    }

    /// `formatPNG`/`formatJPG`/`formatBMP` checkboxes ↔ `imageFormats`.
    pub fn format_enabled(&self, key: FormatKey) -> bool {
        self.image_formats.iter().any(|f| f == key.key())
    }

    pub fn set_format(&mut self, key: FormatKey, on: bool) {
        let owned = key.key().to_string();
        if on {
            if !self.format_enabled(key) {
                self.image_formats.push(owned);
            }
        } else {
            self.image_formats.retain(|f| *f != owned);
        }
    }
}
