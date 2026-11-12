//! Regenerate Windows/window icons from the original HTML logo SVG.
use std::{fs, path::Path};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let tree =
        resvg::usvg::Tree::from_data(&fs::read(assets.join("app-icon.svg"))?, &Default::default())?;
    let sizes = [16u32, 20, 24, 32, 40, 48, 64, 128, 256];
    let mut images = Vec::new();
    for size in sizes {
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).ok_or("invalid icon size")?;
        let scale = size as f32 / tree.size().width();
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(scale, scale),
            &mut pixmap.as_mut(),
        );
        let png = pixmap.encode_png()?;
        if size == 256 {
            fs::write(assets.join("app-icon.png"), &png)?;
        }
        images.push((size, png));
    }
    let mut ico = vec![0, 0, 1, 0];
    ico.extend_from_slice(&(images.len() as u16).to_le_bytes());
    let mut offset = 6 + images.len() * 16;
    for (size, png) in &images {
        ico.extend_from_slice(&[*size as u8, *size as u8, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
        ico.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += png.len();
    }
    for (_, png) in images {
        ico.extend_from_slice(&png);
    }
    fs::write(assets.join("app-icon.ico"), ico)?;
    Ok(())
}
