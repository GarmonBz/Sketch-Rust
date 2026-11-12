//! Installed report fonts, measured and embedded from the same TrueType face.
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

pub struct Face {
    pub bytes: Vec<u8>,
    pub index: u32,
    pub name: String,
}
pub fn face(bold: bool) -> Option<&'static Face> {
    static FONTS: OnceLock<[Option<Face>; 2]> = OnceLock::new();
    FONTS.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        [false, true].map(|bold| {
            let id = db.query(&fontdb::Query {
                families: &[
                    fontdb::Family::Name("Verdana"),
                    fontdb::Family::Name("Arial"),
                    fontdb::Family::SansSerif,
                ],
                weight: if bold {
                    fontdb::Weight::BOLD
                } else {
                    fontdb::Weight::NORMAL
                },
                ..Default::default()
            })?;
            db.with_face_data(id, |data, index| {
                let font = ttf_parser::Face::parse(data, index).ok()?;
                if data.starts_with(b"ttcf") {
                    return None;
                }
                let name = font
                    .names()
                    .into_iter()
                    .filter(|n| n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
                    .find_map(|n| n.to_string())
                    .unwrap_or_else(|| "ReportFont".into());
                Some(Face {
                    bytes: data.to_vec(),
                    index,
                    name,
                })
            })?
        })
    })[usize::from(bold)]
    .as_ref()
}

pub fn width(bold: bool, size: f64, text: &str) -> Option<f64> {
    let data = face(bold)?;
    let f = ttf_parser::Face::parse(&data.bytes, data.index).ok()?;
    Some(
        text.chars()
            .map(|c| {
                f.glyph_index(c)
                    .and_then(|g| f.glyph_hor_advance(g))
                    .unwrap_or(0) as f64
            })
            .sum::<f64>()
            * size
            / f64::from(f.units_per_em()),
    )
}

pub fn font_objects(
    bold: bool,
    base: usize,
    chars: &BTreeSet<char>,
) -> Option<(String, Vec<Vec<u8>>)> {
    let data = face(bold)?;
    let f = ttf_parser::Face::parse(&data.bytes, data.index).ok()?;
    let mut widths = BTreeMap::new();
    let mut cid = Vec::new();
    let mut unicode = Vec::new();
    for c in chars {
        let glyph = f.glyph_index(*c).unwrap_or(ttf_parser::GlyphId(0));
        let code = format!("{:04X}", glyph.0);
        cid.push(format!("<{code}> {}", glyph.0));
        let utf16 = c
            .encode_utf16(&mut [0; 2])
            .iter()
            .map(|v| format!("{v:04X}"))
            .collect::<String>();
        unicode.push(format!("<{code}> <{utf16}>"));
        widths.insert(
            glyph.0,
            f64::from(f.glyph_hor_advance(glyph).unwrap_or(0)) * 1000.0
                / f64::from(f.units_per_em()),
        );
    }
    let cmap = |entries: &[String], kind: &str| {
        let mut out="/CIDInit /ProcSet findresource begin 12 dict begin begincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> def\n/CMapName /ReportUnicode def /CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n".to_string();
        for chunk in entries.chunks(100) {
            out += &format!(
                "{} begin{kind}\n{}\nend{kind}\n",
                chunk.len(),
                chunk.join("\n")
            );
        }
        if kind == "bfchar" {
            out = out.replace("/CMapType 1", "/CMapType 2");
        }
        out += "endcmap CMapName currentdict /CMap defineresource pop end end\n";
        out
    };
    let stream = |dictionary: &str, bytes: &[u8]| {
        let mut out = format!("<< {dictionary} /Length {} >>\nstream\n", bytes.len()).into_bytes();
        out.extend(bytes);
        out.extend(b"\nendstream");
        out
    };
    use std::io::Write;
    let mut zipped = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    zipped.write_all(&data.bytes).ok()?;
    let compressed = zipped.finish().ok()?;
    let scale = 1000.0 / f64::from(f.units_per_em());
    let b = f.global_bounding_box();
    let name = data
        .name
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "");
    let font=format!("<< /Type /Font /Subtype /Type0 /BaseFont /{name} /Encoding /Identity-H /DescendantFonts [{base} 0 R] /ToUnicode {} 0 R >>",base+4);
    let desc=format!("<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{name} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor {} 0 R /CIDToGIDMap /Identity /W [{}] >>",base+1,widths.iter().map(|(g,w)|format!("{g} [{w:.3}]")).collect::<Vec<_>>().join(" "));
    let descriptor=format!("<< /Type /FontDescriptor /FontName /{name} /Flags 32 /FontBBox [{:.2} {:.2} {:.2} {:.2}] /ItalicAngle 0 /Ascent {:.2} /Descent {:.2} /CapHeight {:.2} /StemV {} /FontFile2 {} 0 R >>",f64::from(b.x_min)*scale,f64::from(b.y_min)*scale,f64::from(b.x_max)*scale,f64::from(b.y_max)*scale,f64::from(f.ascender())*scale,f64::from(f.descender())*scale,f64::from(f.capital_height().unwrap_or(f.ascender()))*scale,if bold{120}else{80},base+2);
    Some((
        font,
        vec![
            desc.into_bytes(),
            descriptor.into_bytes(),
            stream(
                &format!("/Filter /FlateDecode /Length1 {}", data.bytes.len()),
                &compressed,
            ),
            stream("", cmap(&cid, "cidchar").as_bytes()),
            stream("", cmap(&unicode, "bfchar").as_bytes()),
        ],
    ))
}

/// Two-byte glyph IDs for the embedded CID TrueType font.
pub fn encode(bold: bool, text: &str) -> Option<String> {
    let data = face(bold)?;
    let f = ttf_parser::Face::parse(&data.bytes, data.index).ok()?;
    Some(
        text.chars()
            .map(|c| {
                format!(
                    "{:04X}",
                    f.glyph_index(c).unwrap_or(ttf_parser::GlyphId(0)).0
                )
            })
            .collect(),
    )
}
