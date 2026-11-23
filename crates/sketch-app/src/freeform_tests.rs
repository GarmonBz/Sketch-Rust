use crate::{
    drawing::WallShape,
    input::KeyCmd,
    view_state::{Mode, RailTool},
    App, Message,
};
use sketch_core::{geometry::Pt, model::WallType};

fn send(app: &mut App, message: Message) {
    let _ = app.update(message);
}

#[test]
fn freeform_strokes_are_unsnapped_saved_and_undone_as_one_edit() {
    let mut app = App::boot(1.).0;
    send(&mut app, Message::Rail(RailTool::Freeform));
    let start = app.world(201.3, 203.7);
    send(&mut app, Message::CanvasPress(201.3, 203.7));
    send(&mut app, Message::CanvasDrag(260.5, 220.9, false));
    send(&mut app, Message::CanvasDrag(300.2, 190.3, false));
    send(&mut app, Message::CanvasDragEnd);
    assert_eq!(app.drawing.lines.len(), 2);
    assert_eq!(app.drawing.lines[0].start, start);
    assert!(app
        .drawing
        .lines
        .iter()
        .all(|l| l.kind == WallType::Interior && l.dimension_hidden));
    assert!(app.drawing.polygons.is_empty());
    assert!(app
        .drawing
        .lines
        .iter()
        .all(|l| app.drawing.wall_thickness_of(l) == 0.0));
    assert!(app.view.drag.is_none());
    let strokes = app.drawing.lines.clone();
    let page = crate::pages::Page {
        lines: strokes.clone(),
        ..Default::default()
    };
    let json = serde_json::to_string(&page).unwrap();
    let saved: crate::pages::Page = serde_json::from_str(&json).unwrap();
    assert_eq!(saved.lines, strokes);
    send(&mut app, Message::Key(KeyCmd::Undo));
    assert!(app.drawing.lines.is_empty());
    send(&mut app, Message::Key(KeyCmd::Redo));
    assert_eq!(app.drawing.lines, strokes);
}

#[test]
fn eraser_sweeps_between_events_and_undo_restores_every_segment() {
    let mut app = App::boot(1.).0;
    send(&mut app, Message::Rail(RailTool::Freeform));
    for x in [200., 300.] {
        send(&mut app, Message::CanvasPress(x, 150.));
        send(&mut app, Message::CanvasDrag(x, 250., false));
        send(&mut app, Message::CanvasDragEnd);
    }
    let strokes = app.drawing.lines.clone();
    send(&mut app, Message::Rail(RailTool::Erase));
    send(&mut app, Message::CanvasPress(100., 200.));
    send(&mut app, Message::CanvasDrag(400., 200., false));
    send(&mut app, Message::CanvasDragEnd);
    assert!(app.drawing.lines.is_empty());
    send(&mut app, Message::Key(KeyCmd::Undo));
    assert_eq!(app.drawing.lines, strokes);
    send(&mut app, Message::Key(KeyCmd::Redo));
    assert!(app.drawing.lines.is_empty());
}

#[test]
fn eraser_preserves_empty_area_interiors_and_protected_layers() {
    let mut app = App::boot(1.).0;
    app.drawing.add_wall_shape(
        WallShape::Rectangle,
        WallType::Exterior,
        10.,
        Some(10.),
        None,
        Pt::new(5., 5.),
    );
    let screen = app
        .view
        .to_screen(crate::view_state::World { x: 5., y: 5. });
    send(&mut app, Message::Rail(RailTool::Erase));
    let lines = app.drawing.lines.clone();
    let areas = app.drawing.polygons.clone();
    send(
        &mut app,
        Message::CanvasPress(screen.0 as f32, screen.1 as f32),
    );
    send(&mut app, Message::CanvasDragEnd);
    assert_eq!(app.drawing.lines, lines);
    assert_eq!(app.drawing.polygons, areas);
    let layer = app.drawing.lines[0]
        .layer_id
        .clone()
        .unwrap_or_else(|| app.layers.active.clone());
    app.layers.set_locked(&layer, true);
    app.layers.set_active(layer.clone());
    let screen = app
        .view
        .to_screen(crate::view_state::World { x: 5., y: 0. });
    send(
        &mut app,
        Message::CanvasPress(screen.0 as f32, screen.1 as f32),
    );
    send(&mut app, Message::CanvasDragEnd);
    assert_eq!(app.drawing.lines, lines);
    send(&mut app, Message::Rail(RailTool::Freeform));
    send(&mut app, Message::CanvasPress(200., 200.));
    send(&mut app, Message::CanvasDrag(300., 250., false));
    send(&mut app, Message::Rail(RailTool::Select));
    assert_eq!(app.drawing.lines, lines);
    assert_eq!(app.view.mode, Mode::Select);
    assert!(app.view.drag.is_none());
}
