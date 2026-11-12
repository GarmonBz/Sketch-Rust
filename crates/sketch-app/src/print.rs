//! Print and Print Preview (S11-054/055) — `exportPDF` and
//! `UIManager.showPrintPreview` (ui-manager.js:1526).
//!
//! The preview renders every page of the exact PDF it prints through Windows'
//! native PDF engine. The cached PDF is retained until the preview closes.

use iced::widget::{
    button, column, container, image, mouse_area, opaque, row, scrollable, text, Space,
};
use iced::{Background, Border, Color, Element, Length};
use sketch_core::print_layout::PaperSize;

use crate::theme::*;
use crate::Message;

/// The preview sheet's width on screen, in px.
const SHEET_W: f32 = 520.0;

#[derive(Debug, Clone, Default)]
pub struct PrintPreview {
    pub open: bool,
    paper: Option<PaperSize>,
    rasters: Vec<image::Handle>,
    pub pdf: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewMsg {
    Close,
    /// `Print / Save PDF` - the same path as File > Print.
    Print,
}

/// The `paperName` the toolbar prints.
pub fn paper_name(paper: PaperSize) -> &'static str {
    match paper {
        PaperSize::Legal => "Legal",
        PaperSize::Letter => "Letter",
        PaperSize::A4 => "A4",
    }
}

/// `1 page · Legal portrait` / `2 pages · A4 portrait`.
pub fn summary(pages: usize, paper: PaperSize) -> String {
    format!(
        "{} {} \u{b7} {} portrait",
        pages,
        if pages == 1 { "page" } else { "pages" },
        paper_name(paper)
    )
}

impl PrintPreview {
    /// The exact PDF is rendered by the platform PDF engine, once per page.
    pub fn open(&mut self, paper: PaperSize, pdf: Vec<u8>) -> Result<(), String> {
        let pages = render_pages(&pdf, 1040)?;
        self.rasters = pages.into_iter().map(image::Handle::from_bytes).collect();
        self.pdf = pdf;
        self.paper = Some(paper);
        self.open = true;
        Ok(())
    }

    pub fn close(&mut self) {
        self.open = false;
        self.rasters.clear();
        self.pdf.clear();
    }
    pub fn page_count(&self) -> usize {
        self.rasters.len()
    }

    pub fn summary(&self) -> String {
        summary(self.page_count(), self.paper.unwrap_or(PaperSize::Legal))
    }
}

/// Render each PDF page in memory. No viewer, process or temporary file is used.
#[cfg(windows)]
pub fn render_pages(pdf: &[u8], width: u32) -> Result<Vec<Vec<u8>>, String> {
    let bytes = pdf.to_vec();
    std::thread::spawn(move || -> windows::core::Result<Vec<Vec<u8>>> {
        use windows::{
            Data::Pdf::{PdfDocument, PdfPageRenderOptions},
            Storage::Streams::{DataReader, DataWriter, InMemoryRandomAccessStream},
            Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
        };
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED)?;
        }
        struct Apartment;
        impl Drop for Apartment {
            fn drop(&mut self) {
                unsafe {
                    RoUninitialize();
                }
            }
        }
        let _apartment = Apartment;
        let input = InMemoryRandomAccessStream::new()?;
        let writer = DataWriter::CreateDataWriter(&input)?;
        writer.WriteBytes(&bytes)?;
        writer.StoreAsync()?.get()?;
        writer.DetachStream()?;
        input.Seek(0)?;
        let document = PdfDocument::LoadFromStreamAsync(&input)?.get()?;
        let mut images = Vec::new();
        for i in 0..document.PageCount()? {
            let page = document.GetPage(i)?;
            let output = InMemoryRandomAccessStream::new()?;
            let options = PdfPageRenderOptions::new()?;
            options.SetDestinationWidth(width)?;
            let size = page.Size()?;
            options.SetDestinationHeight(
                (width as f64 * size.Height as f64 / size.Width as f64).round() as u32,
            )?;
            page.RenderWithOptionsToStreamAsync(&output, &options)?
                .get()?;
            let len = output.Size()? as u32;
            let reader = DataReader::CreateDataReader(&output.GetInputStreamAt(0)?)?;
            reader.LoadAsync(len)?.get()?;
            let mut bytes = vec![0; len as usize];
            reader.ReadBytes(&mut bytes)?;
            page.Close()?;
            // Windows applies the monitor's DPI scale to requested dimensions.
            // Normalize to pixel dimensions so preview memory and sharpness are stable.
            let decoded = ::image::load_from_memory(&bytes).map_err(|_| {
                windows::core::Error::from_hresult(windows::core::HRESULT(0x80004005u32 as i32))
            })?;
            if decoded.width() != width {
                let height = (width as f64 * size.Height as f64 / size.Width as f64).round() as u32;
                let resized =
                    decoded.resize_exact(width, height, ::image::imageops::FilterType::Lanczos3);
                let mut output = std::io::Cursor::new(Vec::new());
                resized
                    .write_to(&mut output, ::image::ImageFormat::Png)
                    .map_err(|_| {
                        windows::core::Error::from_hresult(windows::core::HRESULT(
                            0x80004005u32 as i32,
                        ))
                    })?;
                bytes = output.into_inner();
            }
            images.push(bytes);
        }
        Ok(images)
    })
    .join()
    .map_err(|_| "PDF preview renderer stopped.".to_string())?
    .map_err(|e| format!("Could not render the report preview: {e}"))
}

#[cfg(not(windows))]
pub fn render_pages(_: &[u8], _: u32) -> Result<Vec<Vec<u8>>, String> {
    Err("The native PDF preview renderer requires Windows.".into())
}

/// Where Print writes the report: `%TEMP%\SketchRust\<name>.pdf`.
pub fn temp_report_path(file_name: &str) -> std::path::PathBuf {
    let safe: String = file_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | ' ' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stem = if safe.trim().is_empty() {
        "sketch".to_string()
    } else {
        safe
    };
    std::env::temp_dir()
        .join("SketchRust")
        .join(format!("{stem}.pdf"))
}

/// Hand a file to the OS handler (the browser's `window.open`).
pub fn open_with_os(path: &std::path::Path) {
    crate::platform::open_target(path.as_os_str());
}

/// The preview overlay, or None when closed.
pub fn layer(p: &PrintPreview) -> Option<Element<'_, Message>> {
    if !p.open {
        return None;
    }
    let paper = p.paper.unwrap_or(PaperSize::Legal);
    let (pw, ph) = paper.size();
    let sheet_h = SHEET_W * (ph / pw) as f32;

    let tool_btn = |label: &'static str, msg: PreviewMsg, primary: bool| {
        button(container(text(label).font(SEGOE).size(12.0).color(WHITE)).padding([5, 12]))
            .on_press(Message::PrintPreview(msg))
            .style(move |_, status| button::Style {
                background: Some(Background::Color(match (primary, status) {
                    (true, _) => accent(),
                    (false, button::Status::Hovered) => bg_hover(),
                    (false, _) => bg_ribbon(),
                })),
                border: Border {
                    color: border(),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..button::Style::default()
            })
    };

    let toolbar = container(
        row![
            tool_btn("Print / Save PDF", PreviewMsg::Print, true),
            tool_btn("Close Preview", PreviewMsg::Close, false),
            text(p.summary()).font(SEGOE).size(12.0).color(muted()),
        ]
        .spacing(10)
        .align_y(iced::alignment::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([8, 14])
    .style(|_| container::Style {
        background: Some(Background::Color(bg_panel())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    });

    let mut sheets = column![].spacing(20);
    for raster in &p.rasters {
        sheets = sheets.push(
            image(raster.clone())
                .width(SHEET_W)
                .height(sheet_h)
                .content_fit(iced::ContentFit::Contain),
        );
    }
    let body = scrollable(
        container(sheets)
            .width(Length::Fill)
            .padding(20)
            .center_x(Length::Fill),
    )
    .height(Length::Fill);

    let page = container(column![toolbar, body, Space::new().height(0)])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x3a, 0x3a, 0x3c))),
            ..container::Style::default()
        });
    Some(opaque(mouse_area(page)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_toolbar_summary_matches_the_original() {
        assert_eq!(summary(1, PaperSize::Legal), "1 page \u{b7} Legal portrait");
        assert_eq!(summary(2, PaperSize::A4), "2 pages \u{b7} A4 portrait");
        assert_eq!(
            summary(1, PaperSize::Letter),
            "1 page \u{b7} Letter portrait"
        );
    }

    #[test]
    fn temp_path_sanitises_the_name() {
        let p = temp_report_path("a/b:c");
        assert!(p.ends_with("a_b_c.pdf"));
        assert!(temp_report_path("  ").ends_with("sketch.pdf"));
    }
}
