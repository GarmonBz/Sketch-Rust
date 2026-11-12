//! Regression cases for preserving area identity and properties during repair.
use crate::*;
use sketch_core::{
    geometry::Pt,
    model::{Polygon, WallType},
};

#[test]
fn native_pending_boundaries_keep_their_definition_before_three_vertices_exist() {
    for count in 0..=2 {
        let (app, mut original) = fixture();
        original.points.truncate(count);
        original.edges.truncate(count);
        if let Some(last) = original.edges.last_mut() {
            last.line_id = None;
        }
        original
            .attributes
            .insert("apex.calculation".into(), "net".into());
        let mut candidate = original.clone();
        candidate.id = 100;
        candidate.points = vec![Pt::new(0., 0.), Pt::new(10., 0.), Pt::new(0., 10.)];
        let lines = if count == 2 {
            vec![app.drawing.lines[0].clone()]
        } else {
            vec![]
        };
        candidate.edges = Polygon::build_edges(&candidate.points, &lines);
        let mut page:pages::Page=serde_json::from_value(serde_json::json!({"lines":lines,"polygons":[candidate],"reopenedAreas":[{"original":original,"firstReplacementId":100}]})).unwrap();
        assert_eq!(page.reopened_areas.len(), 1);
        area_repair::restore(
            &mut page.lines,
            &mut page.polygons,
            &mut page.reopened_areas,
        );
        assert!(
            page.reopened_areas.is_empty(),
            "pending native boundary with {count} vertices"
        );
        assert_eq!(page.polygons[0].id, original.id);
        assert_eq!(page.polygons[0].name, original.name);
        assert_eq!(page.polygons[0].attributes["note"], "Keep this property");
    }
}
fn fixture() -> (App, Polygon) {
    let mut app = App::boot(1.).0;
    app.drawing.add_wall_shape(
        drawing::WallShape::Rectangle,
        WallType::Exterior,
        10.,
        Some(10.),
        None,
        Pt::new(5., 5.),
    );
    let p = &mut app.drawing.polygons[0];
    p.kind = "GLA1".into();
    p.name = "Original kitchen".into();
    p.factor_override = Some(0.8);
    p.factor = 0.8;
    p.subtract_mode = "never".into();
    p.measure_to = "center".into();
    p.attributes.insert("yearBuilt".into(), "1998".into());
    p.attributes
        .insert("note".into(), "Keep this property".into());
    p.fill_color = Some("#dbeafe".into());
    p.fill_pattern = Some("diagonal".into());
    p.fill_opacity = Some(0.35);
    p.line_color = Some("#2563eb".into());
    p.line_style = Some("dashed".into());
    p.line_thickness = Some(2.5);
    p.label_rotation = Some(17.);
    p.area_size_rotation = Some(-11.);
    p.label_position = Some(Pt::new(4., 6.));
    p.area_size_position = Some(Pt::new(6., 4.));
    p.label_scale = Some(1.2);
    p.area_size_scale = Some(0.9);
    p.label_font_size = Some(14.);
    p.label_font_family = Some("Cambria".into());
    p.label_bold = Some(true);
    p.label_italic = Some(true);
    p.label_underline = Some(true);
    p.label_color = Some("#123456".into());
    p.auto_post = sketch_core::model::AutoPost {
        code: true,
        name: true,
        dims: false,
        calcs: true,
        base: true,
        factor: true,
        suffix: "sq ft".into(),
    };
    app.ensure_area_layers();
    let original = app.drawing.polygons[0].clone();
    let mut group = layers::Layer::new("repair-test-group", "Original group");
    group.is_group = true;
    app.layers.items.push(group);
    app.layers
        .items
        .iter_mut()
        .find(|l| Some(&l.id) == original.layer_id.as_ref())
        .unwrap()
        .parent_id = Some("repair-test-group".into());
    (app, original)
}
fn delete_boundary(app: &mut App, ids: &[i64]) {
    app.selection.clear();
    app.selection.lines.extend(ids.iter().copied());
    let _ = app.update(Message::Delete);
    assert!(app.drawing.polygons.is_empty());
    assert_eq!(app.drawing.reopened_areas.len(), 1);
}
fn typed_repair(app: &mut App) {
    use input::KeyCmd;
    let _ = app.update(Message::Key(KeyCmd::DrawExterior));
    app.drawing.cursor = Pt::new(0., 0.);
    for key in [
        KeyCmd::Enter,
        KeyCmd::Buffer("1"),
        KeyCmd::Buffer("0"),
        KeyCmd::Arrow {
            dir: 2,
            ctrl: false,
            shift: false,
        },
        KeyCmd::Enter,
    ] {
        let _ = app.update(Message::Key(key));
    }
    let _ = app.update(Message::Noop);
}
fn draw_points(app: &mut App, points: &[Pt]) {
    let _ = app.set_mode(view_state::Mode::DrawExterior);
    let ctx = drawing::DrawCtx {
        snap: false,
        ..app.draw_ctx()
    };
    app.drawing.pen_down_at(points[0], ctx);
    for point in &points[1..] {
        let before = app.drawing.lines.last().map(|l| l.id);
        app.drawing.click(*point, ctx);
        app.finish_wall_commit(before);
    }
    let _ = app.update(Message::Noop);
}
fn assert_properties(app: &App, original: &Polygon) {
    let actual = app
        .drawing
        .polygons
        .iter()
        .find(|p| p.id == original.id)
        .expect("the original area identity must be restored");
    let mut expected = original.clone();
    expected.points = actual.points.clone();
    expected.edges = actual.edges.clone();
    expected.gross = actual.gross;
    expected.area = actual.area;
    assert_eq!(
        actual, &expected,
        "only repaired geometry and computed area may change"
    );
    assert!(app.drawing.reopened_areas.is_empty());
    assert!(actual.edges.iter().filter_map(|e| e.line_id).all(|id| app
        .drawing
        .lines
        .iter()
        .find(|l| l.id == id)
        .unwrap()
        .layer_id
        == original.layer_id));
    assert_eq!(
        app.layers
            .layer(original.layer_id.as_deref().unwrap())
            .unwrap()
            .parent_id
            .as_deref(),
        Some("repair-test-group")
    );
}
#[test]
fn area_repair_restores_all_properties_identity_and_layer_after_typed_redraw() {
    let (mut app, original) = fixture();
    delete_boundary(&mut app, &[1]);
    typed_repair(&mut app);
    assert_properties(&app, &original);
    assert!((app.drawing.polygons[0].gross - 100.).abs() < 1e-6);
}
#[test]
fn area_repair_preserves_properties_with_changed_geometry_and_define_first() {
    let (mut app, original) = fixture();
    delete_boundary(&mut app, &[1]);
    app.drawing.options.define_first = true;
    app.drawing.selected_area_type = Some("BAS".into());
    app.drawing.selected_area_name = Some("Other type".into());
    draw_points(
        &mut app,
        &[Pt::new(0., 0.), Pt::new(5., -2.), Pt::new(10., 0.)],
    );
    assert_properties(&app, &original);
    assert!((app.drawing.polygons[0].gross - 110.).abs() < 1e-6);
}
#[test]
fn area_repair_undo_redo_carries_pending_metadata_and_restored_properties() {
    let (mut app, original) = fixture();
    delete_boundary(&mut app, &[1]);
    let _ = app.update(Message::Undo);
    assert_properties(&app, &original);
    let _ = app.update(Message::Redo);
    assert!(app.drawing.polygons.is_empty());
    assert_eq!(app.drawing.reopened_areas.len(), 1);
    typed_repair(&mut app);
    assert_properties(&app, &original);
    let _ = app.update(Message::Undo);
    assert!(app.drawing.polygons.is_empty());
    assert_eq!(app.drawing.reopened_areas[0].original, original);
    let _ = app.update(Message::Redo);
    assert_properties(&app, &original);
}
#[test]
fn area_repair_autosave_recovery_preserves_open_boundary_metadata() {
    persist::with_temp_store(|| {
        let (mut app, original) = fixture();
        delete_boundary(&mut app, &[1]);
        let _ = app.autosave_tick(true);
        assert!(app.autosave.path.is_some());
        let mut recovered = App::boot(1.).0;
        assert!(recovered.autosave_prompt);
        let _ = recovered.recover_autosave(true);
        assert_eq!(recovered.drawing.reopened_areas[0].original, original);
        typed_repair(&mut recovered);
        assert_properties(&recovered, &original);
    })
}
#[test]
fn area_repair_is_page_local_and_survives_skx_round_trip() {
    let (mut app, original) = fixture();
    delete_boundary(&mut app, &[1]);
    app.save_current_page();
    app.pages.add_page();
    app.load_page_data();
    assert!(app.drawing.reopened_areas.is_empty());
    app.save_current_page();
    app.pages.go_to(0);
    app.load_page_data();
    assert_eq!(app.drawing.reopened_areas.len(), 1);
    let model = app.build_save_model();
    let bytes = sketch_core::skx::write(&model, &Default::default()).unwrap();
    let model = sketch_core::skx::read(&bytes).unwrap().model;
    let mut loaded = App::boot(1.).0;
    let _ = loaded.apply_model(&model, "Saved repair");
    typed_repair(&mut loaded);
    assert_properties(&loaded, &original);
    assert!(loaded.build_save_model()["pages"][0]
        .get("reopenedAreas")
        .is_none());
}
#[test]
fn area_repair_explicit_area_delete_does_not_restore_old_properties() {
    let (mut app, original) = fixture();
    app.selection.polys.insert(original.id);
    let _ = app.update(Message::Delete);
    assert!(app.drawing.reopened_areas.is_empty());
    assert!(app.drawing.lines.is_empty());
    draw_points(
        &mut app,
        &[
            Pt::new(0., 0.),
            Pt::new(10., 0.),
            Pt::new(10., 10.),
            Pt::new(0., 10.),
            Pt::new(0., 0.),
        ],
    );
    assert_eq!(app.drawing.polygons.len(), 1);
    assert_eq!(app.drawing.polygons[0].kind, "UND");
    assert_ne!(app.drawing.polygons[0].name, original.name);
}
#[test]
fn area_repair_does_not_transfer_properties_to_neighbor_on_other_side() {
    let (mut app, original) = fixture();
    delete_boundary(&mut app, &[1, 2, 3]);
    draw_points(
        &mut app,
        &[
            Pt::new(0., 0.),
            Pt::new(-10., 0.),
            Pt::new(-10., 10.),
            Pt::new(0., 10.),
        ],
    );
    assert_eq!(app.drawing.polygons.len(), 1);
    assert_eq!(app.drawing.polygons[0].kind, "UND");
    assert_eq!(app.drawing.reopened_areas.len(), 1);
    draw_points(
        &mut app,
        &[
            Pt::new(0., 0.),
            Pt::new(10., 0.),
            Pt::new(10., 10.),
            Pt::new(0., 10.),
        ],
    );
    assert_eq!(app.drawing.polygons.len(), 2);
    assert_properties(&app, &original);
}
#[test]
fn area_repair_full_redraw_restores_properties_when_entire_boundary_was_deleted() {
    let (mut app, original) = fixture();
    delete_boundary(&mut app, &[1, 2, 3, 4]);
    draw_points(
        &mut app,
        &[
            Pt::new(0., 0.),
            Pt::new(10., 0.),
            Pt::new(10., 10.),
            Pt::new(0., 10.),
            Pt::new(0., 0.),
        ],
    );
    assert_properties(&app, &original);
}
#[test]
fn area_repair_invalid_optional_history_does_not_discard_live_geometry() {
    let (mut app, _) = fixture();
    let mut model = app.build_save_model();
    model["pages"][0]["reopenedAreas"] = serde_json::json!([{"invalid":true}]);
    let mut loaded = App::boot(1.).0;
    let _ = loaded.apply_model(&model, "Invalid history");
    assert_eq!(loaded.drawing.lines.len(), 4);
    assert_eq!(loaded.drawing.polygons.len(), 1);
    assert!(loaded.drawing.reopened_areas.is_empty());
}

#[test]
fn area_repair_preserves_properties_with_a_remaining_curved_boundary() {
    let (mut app, _) = fixture();
    assert!(app.drawing.set_line_bow(4, 90.));
    let original = app.drawing.polygons[0].clone();
    delete_boundary(&mut app, &[1]);
    typed_repair(&mut app);
    assert_properties(&app, &original);
    assert!(app.drawing.polygons[0]
        .edges
        .iter()
        .any(|edge| edge.line_id == Some(4) && edge.bow_angle.abs() > 89.));
}
#[test]
fn area_repair_multiple_boundary_deletions_keep_the_first_original_record() {
    let (mut app, original) = fixture();
    delete_boundary(&mut app, &[1]);
    delete_boundary(&mut app, &[2]);
    draw_points(
        &mut app,
        &[Pt::new(0., 0.), Pt::new(10., 0.), Pt::new(10., 10.)],
    );
    assert_properties(&app, &original);
}

#[test]
fn area_repair_layer_export_import_keeps_open_area_metadata_with_remapped_ids() {
    let (mut app, original) = fixture();
    delete_boundary(&mut app, &[1]);
    let layer = app.export_layer_data("repair-test-group");
    assert_eq!(layer["reopenedAreas"].as_array().unwrap().len(), 1);
    let mut loaded = App::boot(1.).0;
    loaded.import_layer_data(&layer).unwrap();
    assert_eq!(loaded.drawing.reopened_areas.len(), 1);
    let imported = loaded.drawing.reopened_areas[0].original.clone();
    assert_ne!(imported.id, original.id);
    typed_repair(&mut loaded);
    let actual = loaded
        .drawing
        .polygons
        .iter()
        .find(|p| p.id == imported.id)
        .unwrap();
    let mut expected = imported.clone();
    expected.points = actual.points.clone();
    expected.edges = actual.edges.clone();
    expected.gross = actual.gross;
    expected.area = actual.area;
    assert_eq!(actual, &expected);
    assert_eq!(actual.name, original.name);
    assert_eq!(actual.kind, original.kind);
    assert_eq!(actual.attributes, original.attributes);
    assert!(loaded.drawing.reopened_areas.is_empty());
}
