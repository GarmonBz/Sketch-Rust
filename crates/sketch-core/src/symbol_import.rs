//! Shared DXF symbol importer, used by the manager and catalog builder.
use crate::{
    cad_import,
    symbols::{Bounds, Entity, EntityType, Point, Symbol, Vertex, WallSnap},
};

pub fn from_dxf(body: &str, name: &str) -> Result<Symbol, String> {
    let pairs = cad_import::parse_dxf_pairs(body);
    let ranges = cad_import::section_ranges(&pairs);
    let caddillo = pairs
        .iter()
        .any(|p| p.code == 1 && p.value.contains("caddillo.com"));
    let block_name = cad_import::read_entities(&pairs, &ranges)
        .into_iter()
        .find(|e| e.type_name == "INSERT")
        .and_then(|e| e.name)
        .unwrap_or_else(|| name.into());
    let mut source = cad_import::expanded_entities(&pairs, &ranges)?;
    if caddillo {
        // The source's optional advertising label is not part of the fixture.
        source.retain(|e| !e.layer.eq_ignore_ascii_case("CADDILLO"));
    }
    if source.iter().any(|e| e.type_name == "INSERT") {
        return Err("DXF symbol references a missing block.".into());
    }
    let extent = cad_import::extent_of(&source).ok_or("DXF has no usable symbol geometry.")?;
    let unit = cad_import::pick_scale(cad_import::header_insunits(&pairs, &ranges), Some(extent));
    let factor = unit.scale * 12.0;
    let (min_x, min_y, max_x, max_y) = extent;
    let mut gap = 0.0;
    let mut snap = WallSnap::None;
    let mut pivot = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
    if caddillo && (block_name.starts_with("DOOR-") || block_name.starts_with("WINDOW-")) {
        gap = block_name
            .rsplit('-')
            .next()
            .and_then(|n| n.parse::<f64>().ok())
            .unwrap_or(0.0);
        if gap > 0.0 {
            // These blocks use the left jamb as their origin. A side-snapped
            // opening is cut from the symbol's origin across its width (the
            // original's hinge-origin doors), so a door keeps the jamb as its
            // pivot; a centre-snapped window is cut about its midpoint.
            if block_name.starts_with("WINDOW-") {
                snap = WallSnap::Center;
                pivot = (gap / 2.0, 0.0);
            } else {
                snap = WallSnap::Side;
                pivot = (0.0, 0.0);
            }
        }
    }
    let point = |x: f64, y: f64| Point {
        x: (x - pivot.0) * factor,
        y: (y - pivot.1) * factor,
    };
    let mut geometry = Vec::new();
    for e in source {
        let entity = match e.type_name.as_str() {
            "LINE" => Entity {
                start_point: Some(point(e.x, e.y)),
                end_point: Some(point(e.x2.unwrap_or(e.x), e.y2.unwrap_or(e.y))),
                ..Default::default()
            },
            "LWPOLYLINE" | "POLYLINE" if e.vertices.len() >= 2 => Entity {
                entity_type: EntityType::Polyline,
                closed: Some(e.flag & 1 != 0),
                vertices: Some(
                    e.vertices
                        .iter()
                        .map(|v| {
                            let p = point(v.x, v.y);
                            Vertex {
                                x: p.x,
                                y: p.y,
                                b: None,
                            }
                        })
                        .collect(),
                ),
                fill_color: e.filled.then_some(0),
                ..Default::default()
            },
            "TEXT" | "MTEXT" if !e.text.trim().is_empty() => Entity {
                entity_type: EntityType::Text,
                insert_point: Some(point(e.x, e.y)),
                text_string: Some(e.text),
                font_size: Some(e.radius.unwrap_or(6.0) * factor),
                rotation: e.start_angle,
                ..Default::default()
            },
            _ => continue,
        };
        geometry.push(entity);
    }
    if geometry.is_empty() {
        return Err("DXF has no usable symbol geometry.".into());
    }
    if ![min_x, min_y, max_x, max_y, factor]
        .iter()
        .all(|v| v.is_finite())
        || factor <= 0.0
    {
        return Err("DXF symbol has invalid coordinates or units.".into());
    }
    let lo = point(min_x, min_y);
    let hi = point(max_x, max_y);
    Ok(Symbol {
        name: name.into(),
        label: name.into(),
        units: "inches".into(),
        bounds: Bounds {
            min_x: lo.x,
            min_y: lo.y,
            max_x: hi.x,
            max_y: hi.y,
        },
        geometry,
        wall_snap: snap,
        wall_gap_width: gap * factor,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};
    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }
    fn source(name: &str) -> Symbol {
        from_dxf(
            &fs::read_to_string(root().join(format!("refs/caddillo/fixtures/{name}.dxf"))).unwrap(),
            name,
        )
        .unwrap()
    }
    #[test]
    fn real_caddillo_blocks_import_geometry_in_inches_without_advertising() {
        for (name, w, h) in [
            ("TUB", 60., 30.),
            ("WASHER-27", 27., 27.),
            ("BED-KING", 76., 80.),
            ("COUCH-84", 84., 36.),
        ] {
            let s = source(name);
            assert!((s.bounds.max_x - s.bounds.min_x - w).abs() < 1e-6, "{name}");
            assert!((s.bounds.max_y - s.bounds.min_y - h).abs() < 1e-6, "{name}");
            assert!(s.geometry.len() > 4);
            assert_eq!(s.units, "inches");
            assert_eq!(s.wall_snap, WallSnap::None);
            assert_eq!(s.wall_gap_width, 0.0);
            assert!(!serde_json::to_string(&s.geometry)
                .unwrap()
                .contains("caddillo.com"));
        }
        let spiral = source("STAIR-SPIRAL-60-LH");
        assert!(spiral.geometry.len() > 12);
    }
    #[test]
    fn doors_pivot_on_the_jamb_windows_on_the_midpoint_with_nominal_openings() {
        for (name, width, snap) in [
            ("DOOR-SWING-36", 36., WallSnap::Side),
            ("DOOR-DOUBLE-72", 72., WallSnap::Side),
            ("WINDOW-SINGLE-36", 36., WallSnap::Center),
        ] {
            let s = source(name);
            assert_eq!(s.wall_gap_width, width);
            assert_eq!(s.wall_snap, snap);
            // The wall is cut from the origin across the width (side snap)
            // or about the origin (centre snap); either way the symbol must
            // sit centred on that cut, with its frame overhanging equally.
            let cut_centre = if snap == WallSnap::Side { width / 2.0 } else { 0.0 };
            assert!(
                ((s.bounds.min_x + s.bounds.max_x) / 2.0 - cut_centre).abs() < 1e-6,
                "{name}"
            );
            assert!(s.geometry.iter().all(|e| e
                .text_string
                .as_ref()
                .is_none_or(|t| !t.contains("caddillo.com"))));
        }
        let window = source("WINDOW-SINGLE-36");
        assert_eq!(window.bounds.min_x, -18.0);
        assert_eq!(window.bounds.max_x, 18.0);
    }
    #[test]
    fn catalog_preserves_every_old_definition_and_all_categories_resolve() {
        let previous: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root().join("assets/symbols.previous.json")).unwrap(),
        )
        .unwrap();
        let current: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root().join("assets/symbols.json")).unwrap())
                .unwrap();
        for (id, symbol) in previous["symbols"].as_object().unwrap() {
            assert_eq!(&current["symbols"][id], symbol, "{id}");
        }
        let lib: crate::symbols::SymbolLibrary = serde_json::from_value(current).unwrap();
        assert_eq!(
            lib.symbols
                .keys()
                .filter(|id| id.starts_with("caddillo/"))
                .count(),
            177
        );
        for c in &lib.categories {
            for id in &c.symbol_ids {
                assert!(lib.symbols.contains_key(id), "{id}");
            }
        }
        for id in &lib.default_favorites {
            assert!(lib.symbols.contains_key(id));
        }
        for s in lib
            .symbols
            .values()
            .filter(|s| s.id.starts_with("caddillo/"))
        {
            assert!(!s.geometry.is_empty());
            assert!([
                s.bounds.min_x,
                s.bounds.min_y,
                s.bounds.max_x,
                s.bounds.max_y
            ]
            .iter()
            .all(|v| v.is_finite()));
            assert!(s.bounds.max_x >= s.bounds.min_x && s.bounds.max_y >= s.bounds.min_y);
            assert!(!serde_json::to_string(&s.geometry)
                .unwrap()
                .contains("caddillo.com"));
        }
    }
    #[test]
    fn insert_base_translation_rotation_mirroring_and_nested_scale() {
        let body="0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n1\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nINNER\n10\n2\n20\n3\n0\nLINE\n10\n2\n20\n3\n11\n5\n21\n3\n0\nENDBLK\n0\nBLOCK\n2\nOUTER\n0\nINSERT\n2\nINNER\n10\n10\n20\n20\n41\n2\n42\n1\n50\n90\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nOUTER\n10\n5\n20\n7\n41\n-1\n42\n3\n0\nENDSEC\n0\nEOF\n";
        for text in [body.to_owned(), body.replace('\n', "\r\n")] {
            let parsed = cad_import::parse_dxf(&text, None, (0., 0.)).unwrap();
            assert_eq!(parsed.lines.len(), 1);
            let (a, b) = parsed.lines[0];
            assert!((a.x * 12. + 5.).abs() < 1e-8 && (a.y * 12. - 67.).abs() < 1e-8);
            assert!((b.x * 12. + 5.).abs() < 1e-8 && (b.y * 12. - 85.).abs() < 1e-8);
        }
    }
    #[test]
    fn recursive_and_missing_blocks_fail_symbol_import_cleanly() {
        let body="0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nLOOP\n0\nINSERT\n2\nLOOP\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nLOOP\n0\nENDSEC\n";
        assert!(from_dxf(body, "loop").unwrap_err().contains("nesting"));
        assert!(from_dxf(
            "0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nMISSING\n0\nENDSEC\n",
            "missing"
        )
        .unwrap_err()
        .contains("missing block"));
    }

    #[test]
    fn caddillo_rotated_scaled_mirrored_geometry_fits_image_with_margins() {
        use crate::export_model::{ExportPage, ExportSymbol};
        let lib = crate::symbols::SymbolLibrary::load(&root().join("assets/symbols.json")).unwrap();
        for rotation in [0., 45., 90., 270.] {
            for flip_h in [false, true] {
                let s = ExportSymbol {
                    x: 17.,
                    y: -9.,
                    symbol_type: "caddillo/door-pocket-32".into(),
                    rotation,
                    scale: 2.,
                    flip_h,
                    ..Default::default()
                };
                let page = ExportPage {
                    symbols: vec![s.clone()],
                    ..Default::default()
                };
                let fit =
                    crate::export_png::fit_page_with_library(&page, (2048, 1536), None, Some(&lib));
                for p in crate::symbols::world_corners(
                    lib.symbol(&s.symbol_type).unwrap().bounds,
                    s.x,
                    s.y,
                    rotation,
                    2.,
                    flip_h,
                ) {
                    let (x, y) = fit.point(p);
                    assert!(
                        (59.9..=1988.1).contains(&x) && (59.9..=1476.1).contains(&y),
                        "{rotation} {flip_h}: {x},{y}"
                    );
                }
            }
        }
    }
}
