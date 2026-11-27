//! A synthetic sketch (no private data) through write, read and both
//! conversions.

use super::*;
use crate::geometry::{area_codes, GeometryEngine};

const NOWHERE: Pt = Pt { x: 0.0, y: 0.0 };

fn near_pt(a: Pt, b: Pt, tolerance: f64) -> bool {
    (a.x - b.x).abs() < tolerance && (a.y - b.y).abs() < tolerance
}

/// A 20 x 10 ft room whose far wall bows out, stored with its walls out of
/// order and one reversed, plus an unfinished basement, an interior wall, a
/// note and a symbol.
fn sample_page() -> serde_json::Value {
    let line = |id: i64, a: (f64, f64), b: (f64, f64), bow: f64| {
        serde_json::json!({
            "id": id, "start": {"x": a.0, "y": a.1}, "end": {"x": b.0, "y": b.1},
            "type": "exterior", "bowAngle": bow
        })
    };
    serde_json::json!({
        "lines": [
            line(1, (0.0, 0.0), (20.0, 0.0), 0.0),
            line(2, (20.0, 0.0), (20.0, 10.0), 0.0),
            line(3, (20.0, 10.0), (0.0, 10.0), -90.0),
            line(4, (0.0, 10.0), (0.0, 0.0), 0.0),
            line(9, (5.0, 0.0), (5.0, 6.0), 0.0),
        ],
        "polygons": [{
            "id": 20,
            "points": [{"x":0,"y":0},{"x":20,"y":0},{"x":20,"y":10},{"x":0,"y":10}],
            "edges": [{"lineId":1,"bowAngle":0},{"lineId":2,"bowAngle":0},
                      {"lineId":3,"bowAngle":-90},{"lineId":4,"bowAngle":0}],
            "type": "GLA2", "name": "Second Floor", "factor": 0.5,
            "fillColor": "#FF8000", "fillOpacity": 20
        }],
        "labels": [{
            "id": 30, "text": "½ Bath", "x": 3.0, "y": 4.0, "rotation": 315.0,
            "fontFamily": "Verdana", "fontSize": 18.3, "italic": true
        }],
        "symbols": [{
            "id": 40, "symbolType": "test/door", "x": 8.0, "y": 0.0,
            "rotation": 90.0, "scale": 2.0, "flipH": true
        }]
    })
}

/// A leaf and its swing: 36 x 36 inches, drawn from the hinge.
fn door() -> crate::symbols::Symbol {
    serde_json::from_value(serde_json::json!({
        "id": "test/door", "category": "doors", "categoryName": "Doors", "name": "Swing",
        "geometry": [
            {"entityType": "LINE", "startPoint": {"x": 0, "y": 0}, "endPoint": {"x": 0, "y": 36}},
            {"entityType": "ARC", "center": {"x": 0, "y": 0}, "axisPoint": {"x": 36, "y": 0},
             "startAngle": 0, "endAngle": 90}
        ]
    }))
    .unwrap()
}

fn exported() -> Document {
    let subject = serde_json::json!({
        "fileNo": "F-1", "propertyAddress": "1 Test Way", "owner": "A. Sample",
        "rapidSketchCaseNumber": "C-9"
    });
    from_page(
        &sample_page(),
        &area_codes::default_area_codes(),
        &subject,
        &|id| (id == "test/door").then(door),
    )
    .unwrap()
}

#[test]
fn a_written_file_reads_back() {
    let doc = exported();
    assert!(doc.warnings.is_empty(), "{:?}", doc.warnings);
    let bytes = write(&doc);
    assert!(recognizes(&bytes));
    let back = read(&bytes).unwrap();

    assert_eq!(back.information, doc.information);
    // The interior wall travels as an open area.
    assert_eq!(back.areas.len(), 2);
    let area = &back.areas[0];
    assert_eq!((area.name.as_str(), area.living), ("Second Floor", true));
    assert_eq!(area.multiplier, 0.5);
    assert_eq!(area.fill, 0x33FF_8000);
    assert_eq!(area.walls.len(), 4);
    for (got, want) in area.walls.iter().zip(&doc.areas[0].walls) {
        assert!(near_pt(got.segment.start, want.segment.start, 1e-3));
        assert!(near_pt(got.segment.apex(), want.segment.apex(), 1e-3));
        assert_eq!(got.dimension.text, want.dimension.text);
    }
    assert_eq!(area.walls[0].dimension.text, "20'");
    assert_eq!(back.areas[1].walls.len(), 1);
    assert!(!back.areas[1].label.visible);
    assert_eq!(back.notes.len(), 1);
    assert_eq!(back.notes[0].text, "½ Bath");
    assert_eq!(back.notes[0].font.size, 8.0);
    assert_eq!(back.notes[0].font.style, 2);

    let (got, want) = (&back.symbols[0], &doc.symbols[0]);
    assert_eq!(
        (got.name.as_str(), got.category.as_str()),
        ("Swing", "Doors")
    );
    assert!(near_pt(got.center, want.center, 1e-3));
    assert!(got.flip_h && !got.flip_v && got.rotation == 90.0);
    // Reading stretches the graphic to the stated size: it must come back as
    // it was written, not shifted or rescaled.
    let strokes = |s: &Symbol| -> Vec<Pt> {
        s.shapes
            .iter()
            .flat_map(|shape| shape.segments.iter())
            .flat_map(|s| [s.start, s.apex(), s.end])
            .collect()
    };
    let (got, want) = (strokes(got), strokes(want));
    assert_eq!(got.len(), want.len());
    for (g, w) in got.iter().zip(&want) {
        assert!(near_pt(*g, *w, 0.01), "{g:?} {w:?}");
    }
}

#[test]
fn a_round_trip_keeps_areas_arcs_and_symbols() {
    let engine = GeometryEngine::new();
    let page = sample_page();
    let original: crate::model::Polygon =
        serde_json::from_value(page["polygons"][0].clone()).unwrap();
    let want_area = engine.calculate_area(&original.points, &original.edges);

    let import = read(&write(&exported())).unwrap().to_model("trip");
    assert_eq!(import.area_count, 1);
    let page = &import.model["pages"][0];
    let polygon: crate::model::Polygon =
        serde_json::from_value(page["polygons"][0].clone()).unwrap();
    assert_eq!(polygon.kind, "GLA2");
    assert_eq!(polygon.factor, 0.5);
    assert_eq!(polygon.fill_color.as_deref(), Some("#FF8000"));
    assert_eq!(polygon.fill_opacity, Some(20.0));
    assert!(
        (polygon.gross - want_area).abs() < 1e-2,
        "{}",
        polygon.gross
    );
    let bowed = polygon.edges.iter().filter(|e| e.bow_angle != 0.0);
    let bows: Vec<f64> = bowed.map(|e| e.bow_angle).collect();
    assert_eq!(bows.len(), 1);
    assert!((bows[0] + 90.0).abs() < 0.05, "{bows:?}");
    assert_eq!(page["lines"].as_array().unwrap().len(), 5);
    assert_eq!(page["labels"][0]["text"], "½ Bath");

    // The symbol's corners land where they were.
    let placed = &page["symbols"][0];
    let id = placed["symbolType"].as_str().unwrap();
    let def: crate::symbols::Symbol =
        serde_json::from_value(import.model["settings"]["customSymbols"][id].clone()).unwrap();
    let corners = |bounds, x, y, rotation, scale, flip| {
        crate::symbols::world_corners(bounds, x, y, rotation, scale, flip)
    };
    let door_bounds = crate::symbols::Bounds {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 36.0,
        max_y: 36.0,
    };
    let before = corners(door_bounds, 8.0, 0.0, 90.0, 2.0, true);
    let after = corners(
        def.bounds,
        placed["x"].as_f64().unwrap(),
        placed["y"].as_f64().unwrap(),
        placed["rotation"].as_f64().unwrap(),
        1.0,
        placed["flipH"].as_bool().unwrap(),
    );
    for (a, b) in before.iter().zip(&after) {
        assert!(near_pt(*a, *b, 0.01), "{before:?} {after:?}");
    }
}

#[test]
fn a_positive_arc_height_stands_left_of_travel_with_y_up() {
    // East along the file's x axis; with y flipped, "left" is north, which is
    // negative y in the file.
    let s = Segment {
        start: NOWHERE,
        end: Pt::new(14.0, 0.0),
        arc: 3.0,
    };
    assert!(near_pt(s.apex(), Pt::new(7.0, -3.0), 1e-9));
    let back = Segment::through(s.start, s.apex(), s.end);
    assert!((back.arc - 3.0).abs() < 1e-9);
}

#[test]
fn walls_join_in_any_order_and_direction() {
    let seg = |a: (f64, f64), b: (f64, f64)| Segment {
        start: Pt::new(a.0, a.1),
        end: Pt::new(b.0, b.1),
        arc: 0.0,
    };
    let joined = chains(&[
        seg((10.0, 10.0), (0.0, 10.0)),
        seg((0.0, 0.0), (10.0, 0.0)),
        seg((0.0, 0.0), (0.0, 10.0)),
        seg((10.0, 0.0), (10.0, 10.0)),
        // A separate open run, given middle first.
        seg((30.0, 0.0), (40.0, 0.0)),
        seg((20.0, 0.0), (30.0, 0.0)),
    ]);
    assert_eq!(joined.len(), 2);
    assert!(joined[0].closed && joined[0].links.len() == 4);
    assert!(!joined[1].closed && joined[1].links == [(5, false), (4, false)]);
}

#[test]
fn other_files_and_damaged_files_are_errors() {
    assert_eq!(read(b"not a sketch"), Err(RapidError::NotRapidSketch));
    let bytes = write(&exported());
    for end in (0..bytes.len()).step_by(97) {
        assert!(read(&bytes[..end]).is_err());
    }
    // Damage anywhere must never panic.
    for at in (17..bytes.len()).step_by(13) {
        for bad in [0u8, 9, 0x7f, 0xff] {
            let mut copy = bytes.clone();
            copy[at] = bad;
            let _ = read(&copy).map(|d| d.to_model("x"));
        }
    }
}

/// Set `RAPIDSKETCH_SAMPLE` to a real file to check it reads, converts and
/// survives a round trip. Real sketches are private, so none is committed.
#[test]
fn a_real_sample_round_trips_when_one_is_given() {
    let Ok(path) = std::env::var("RAPIDSKETCH_SAMPLE") else {
        return;
    };
    let doc = read(&std::fs::read(&path).unwrap()).unwrap();
    let import = doc.to_model("sample");
    let page = &import.model["pages"][0];
    let count = |key: &str| page[key].as_array().unwrap().len();
    println!(
        "areas {} polygons {} walls {} labels {} symbols {} notes {:?}",
        doc.areas.len(),
        count("polygons"),
        count("lines"),
        count("labels"),
        count("symbols"),
        import.notes
    );
    for p in page["polygons"].as_array().unwrap() {
        println!(
            "  {} [{}] {:.2}",
            p["name"],
            p["type"],
            p["area"].as_f64().unwrap()
        );
    }
    let definitions = import.model["settings"]["customSymbols"].clone();
    let again = from_page(
        page,
        &area_codes::default_area_codes(),
        &import.model["subjectInfo"],
        &|id| serde_json::from_value(definitions[id].clone()).ok(),
    )
    .unwrap();
    let bytes = write(&again);
    if let Ok(out) = std::env::var("RAPIDSKETCH_OUT") {
        std::fs::write(out, &bytes).unwrap();
    }
    let back = read(&bytes).unwrap().to_model("sample");
    assert_eq!(back.area_count, import.area_count);
    let totals = |m: &serde_json::Value| -> f64 {
        m["pages"][0]["polygons"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["area"].as_f64().unwrap())
            .sum()
    };
    assert!((totals(&back.model) - totals(&import.model)).abs() < 0.1);
}
