//! sketch-core — headless geometry, area calculation, CAD/PDF/SVG/PNG/DXF
//! export, and sketch file kernel for Sketch Rust.

pub mod area_calc;
pub mod breakdown;
pub mod cad_import;
pub mod disto;
pub mod export_dxf;
pub mod export_model;
pub mod export_pdf;
pub mod export_png;
pub mod export_svg;
pub mod geometry;
pub mod joins;
pub mod model;
pub mod print_layout;
mod report_fonts;
pub mod skx;
pub mod solids;
pub mod symbol_edit;
pub mod symbol_import;
pub mod symbols;
