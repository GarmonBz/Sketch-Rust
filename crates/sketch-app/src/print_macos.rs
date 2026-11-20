//! Rasterize report PDF pages with PDFKit for in-app print preview.

use objc2::MainThreadMarker;
use objc2::AnyThread;
use objc2_app_kit::NSImage;
use objc2_foundation::{NSData, NSSize};
use objc2_pdf_kit::{PDFDisplayBox, PDFDocument, PDFPage};

pub fn render_pages(pdf: &[u8], max_edge: u32) -> Result<Vec<Vec<u8>>, String> {
    with_appkit_main(|_mtm| render_pages_inner(pdf, max_edge))
}

fn render_pages_inner(pdf: &[u8], max_edge: u32) -> Result<Vec<Vec<u8>>, String> {
    let data = NSData::with_bytes(pdf);
    let doc = unsafe { PDFDocument::initWithData(PDFDocument::alloc(), &data) }
        .ok_or_else(|| "Could not open PDF for preview.".to_string())?;
    let count = unsafe { doc.pageCount() } as usize;
    if count == 0 {
        return Err("PDF has no pages.".into());
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let page = unsafe { doc.pageAtIndex(i) }
            .ok_or_else(|| format!("Missing PDF page {i}"))?;
        out.push(render_page_png(&page, max_edge)?);
    }
    Ok(out)
}

fn render_page_png(page: &PDFPage, max_edge: u32) -> Result<Vec<u8>, String> {
    let bounds = unsafe { page.boundsForBox(PDFDisplayBox::MediaBox) };
    let w = bounds.size.width;
    let h = bounds.size.height;
    if w <= 0.0 || h <= 0.0 {
        return Err("PDF page has empty bounds.".into());
    }
    let longest = w.max(h);
    let scale = max_edge as f64 / longest;
    let thumb_size = NSSize::new((w * scale).max(1.0), (h * scale).max(1.0));
    let image = unsafe { page.thumbnailOfSize_forBox(thumb_size, PDFDisplayBox::MediaBox) };
    png_bytes_from_nsimage(&image)
}

fn png_bytes_from_nsimage(image: &NSImage) -> Result<Vec<u8>, String> {
    let tiff = image
        .TIFFRepresentation()
        .ok_or_else(|| "Could not rasterize PDF page.".to_string())?;
    let raw = tiff.to_vec();
    let decoded = ::image::load_from_memory(&raw)
        .map_err(|e| format!("Could not decode PDF raster: {e}"))?;
    let mut output = std::io::Cursor::new(Vec::new());
    decoded
        .write_to(&mut output, ::image::ImageFormat::Png)
        .map_err(|e| format!("Could not encode PNG: {e}"))?;
    Ok(output.into_inner())
}

fn with_appkit_main<F, R>(f: F) -> R
where
    F: FnOnce(MainThreadMarker) -> R + Send,
    R: Send,
{
    use dispatch2::run_on_main;
    use objc2_app_kit::NSApplication;

    if let Some(mtm) = MainThreadMarker::new() {
        return f(mtm);
    }
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let app = NSApplication::sharedApplication(mtm);
    if app.isRunning() {
        run_on_main(f)
    } else {
        // Headless unit tests: no run loop, so `dispatch2` would block forever.
        f(mtm)
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use sketch_core::export_model::{ExportLine, ExportOptions, ExportPage, ExportPolygon};
    use sketch_core::export_pdf::{export_pdf, ReportSubject};
    use sketch_core::geometry::{Edge, Pt};
    use sketch_core::print_layout::PaperSize;

    use super::*;

    fn minimal_export_pdf() -> Vec<u8> {
        let page = ExportPage {
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
        };
        export_pdf(
            &page,
            &ExportOptions::default(),
            &[],
            &ReportSubject::default(),
            PaperSize::Letter,
        )
    }

    #[test]
    fn render_pages_returns_at_least_one_png_for_minimal_pdf() {
        let pdf = minimal_export_pdf();
        let pages = render_pages(&pdf, 512);
        assert!(pages.is_ok(), "{pages:?}");
        let pages = pages.unwrap();
        assert!(!pages.is_empty());
        assert_eq!(&pages[0][0..8], b"\x89PNG\r\n\x1a\n");
    }
}