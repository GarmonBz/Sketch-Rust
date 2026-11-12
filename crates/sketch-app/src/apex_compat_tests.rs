use crate::*;
use sketch_core::{geometry::Pt, model::WallType};

#[test]
#[ignore = "development timing probe for linked drawing input"]
fn integrated_input_profile() {
    let mut app = fixture();
    let root = output("input-profile");
    app.apex_session = Some(root.clone());
    let _ = app.update(Message::Noop);
    app.write_apex_snapshot(&root, false).unwrap();
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = app.update(Message::ApexPoll);
    }
    let idle_ms = start.elapsed().as_secs_f64() * 1000.0;
    app.drawing.polygons[0].name = "Edited input profile".into();
    let _ = app.update(Message::Noop);
    let _ = app.update(Message::ApexPoll);
    app.apex_changed_at = std::time::Instant::now() - std::time::Duration::from_secs(1);
    let start = std::time::Instant::now();
    let task = app.update(Message::ApexPoll);
    let changed_ms = start.elapsed().as_secs_f64() * 1000.0;
    finish_task(&mut app, task);
    let start = std::time::Instant::now();
    for _ in 0..100 {
        std::hint::black_box(app.view());
    }
    let view_ms = start.elapsed().as_secs_f64() * 1000.0;
    let data = serde_json::json!({"idlePolls":100,"idlePollsMs":idle_ms,"changedPollMs":changed_ms,"viewBuilds":100,"viewBuildMs":view_ms});
    let out =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/input-latency-021");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("last.json"),
        serde_json::to_vec_pretty(&data).unwrap(),
    )
    .unwrap();
    println!("INPUT_PROFILE {data}");
}
fn finish_task(app: &mut App, task: Task<Message>) {
    use iced::futures::StreamExt;
    if let Some(stream) = iced_runtime::task::into_stream(task) {
        let messages = iced::futures::executor::block_on(async {
            stream
                .filter_map(|action| async move {
                    match action {
                        iced_runtime::Action::Output(message) => Some(message),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
                .await
        });
        for message in messages {
            let next = app.update(message);
            finish_task(app, next);
        }
    }
}
fn fixture() -> App {
    let mut app = App::boot(1.).0;
    app.drawing.add_wall_shape(
        drawing::WallShape::Rectangle,
        WallType::Exterior,
        20.,
        Some(10.),
        None,
        Pt::new(10., 5.),
    );
    app.drawing.polygons[0].kind = "GLA1".into();
    app.drawing.polygons[0].name = "Compatibility first floor".into();
    app.drawing.polygons[0]
        .attributes
        .insert("fixture".into(), "Keep editable properties".into());
    app
}
fn output(name: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/e2e/apex-{name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}
#[test]
fn native_finished_basement_alias_uses_host_basement_code() {
    let mut app = fixture();
    app.drawing.polygons[0].kind = "BSMT1".into();
    app.drawing.polygons[0].name = "Lower level".into();
    let root = output("finished-basement-alias");
    app.write_apex_return(&root).unwrap();
    let packet: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("result.json")).unwrap()).unwrap();
    assert_eq!(packet["pages"][0]["areas"][0]["code"], "BSMT");
    assert_eq!(
        packet["pages"][0]["areas"][0]["name"],
        "Finished Lower level"
    );
    assert_eq!(packet["pages"][0]["areas"][0]["netArea"], 200.);
}
#[test]
fn apex_compat_real_document_exports_all_pages_numeric_areas_images_and_editable_state() {
    let mut app = fixture();
    let first_id = app.drawing.polygons[0].id;
    app.add_page();
    app.drawing.add_wall_shape(
        drawing::WallShape::Rectangle,
        WallType::Exterior,
        10.,
        Some(10.),
        None,
        Pt::new(5., 5.),
    );
    app.drawing.polygons[0].kind = "BSMT2".into();
    app.drawing.polygons[0].name = "Storage".into();
    let second_id = app.drawing.polygons[0].id;
    let root = output("return-model");
    app.write_apex_return(&root).unwrap();
    let packet: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("result.json")).unwrap()).unwrap();
    assert_eq!(packet["pages"].as_array().unwrap().len(), 2);
    assert_eq!(packet["pages"][0]["areas"][0]["netArea"], 200.);
    assert_eq!(packet["pages"][0]["areas"][0]["id"], first_id);
    assert_eq!(packet["pages"][0]["areas"][0]["code"], "GLA1");
    assert_eq!(packet["pages"][1]["areas"][0]["netArea"], 100.);
    assert_eq!(packet["pages"][1]["areas"][0]["id"], second_id);
    assert_ne!(first_id, second_id, "host IDs are stable across pages");
    assert_eq!(packet["pages"][1]["areas"][0]["code"], "BSMT");
    assert_eq!(packet["pages"][1]["areas"][0]["name"], "Unfinished Storage");
    for page in packet["pages"].as_array().unwrap() {
        let image = image::open(page["image"].as_str().unwrap()).unwrap();
        assert!(
            image.width() > 1000 && image.height() > 2800,
            "host images reserve space for summary and breakdown below the floorplan"
        );
    }
    let document =
        sketch_core::skx::read(&std::fs::read(root.join("document.skx")).unwrap()).unwrap();
    let mut reopened = App::boot(1.).0;
    let _ = reopened.apply_model(&document.model, "Round trip");
    assert_eq!(reopened.pages.pages.len(), 2);
    assert_eq!(
        reopened.drawing.polygons[0].attributes["fixture"],
        "Keep editable properties"
    );
    assert!(root.join("return-ready").is_file());
}
#[test]
fn apex_compat_calculated_factor_and_signed_deductions_survive_payload() {
    let mut app = fixture();
    app.drawing.polygons[0].factor_override = Some(-0.5);
    let root = output("signed");
    app.write_apex_return(&root).unwrap();
    let packet: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("result.json")).unwrap()).unwrap();
    assert_eq!(packet["pages"][0]["areas"][0]["netArea"], -100.);
}
#[test]
fn apex_compat_context_loads_the_report_subject_and_reopens_saved_sketch() {
    let root = output("subject");
    let mut saved = fixture();
    saved.write_apex_return(&root).unwrap();
    std::fs::write(root.join("context.json"),serde_json::to_vec(&serde_json::json!({"version":1,"hostPath":root.join("report.ax6"),"document":root.join("document.skx"),"processId":std::process::id(),"subject":{"FileNumber":"RUST-COMPAT-TEST","PropertyAddress":"Disposable fixture","City":"Test City","Borrower":"Test Borrower"}})).unwrap()).unwrap();
    let mut loaded = App::boot(1.).0;
    loaded.load_apex_context(&root).unwrap();
    assert_eq!(loaded.subject_info.file_no, "RUST-COMPAT-TEST");
    assert_eq!(loaded.subject_info.property_address, "Disposable fixture");
    assert_eq!(loaded.drawing.polygons.len(), 1);
    assert_eq!(loaded.apex_session, Some(root));
    assert!(!loaded.autosave_prompt);
}
#[test]
fn apex_compat_undefined_sketch_never_signals_a_return() {
    let mut app = fixture();
    app.drawing.polygons[0].kind = "UND".into();
    let root = output("undefined");
    assert!(app.write_apex_return(&root).is_err());
    assert!(!root.join("return-ready").exists());
}

#[test]
fn apex_compat_corrupt_recovery_never_binds_a_blank_host_session() {
    let root = output("corrupt-context");
    let state = root.join("invalid.skx");
    std::fs::write(&state, b"not an editable sketch").unwrap();
    std::fs::write(
        root.join("context.json"),
        serde_json::to_vec(&serde_json::json!({
            "version":1,"hostPath":root.join("report.ax7"),"document":state,
            "processId":std::process::id(),"subject":{}
        }))
        .unwrap(),
    )
    .unwrap();
    let mut app = fixture();
    let polygon = app.drawing.polygons[0].clone();
    assert!(app.load_apex_context(&root).is_err());
    assert!(app.apex_session.is_none());
    assert_eq!(app.drawing.polygons[0], polygon);
}

#[test]
fn apex_compat_rejects_recovery_with_no_document_pages() {
    let root = output("invalid-pages");
    let state = root.join("invalid.skx");
    std::fs::write(
        &state,
        sketch_core::skx::write(&serde_json::json!({"pages":[]}), &Default::default()).unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("context.json"),
        serde_json::to_vec(&serde_json::json!({
            "version":1,"hostPath":root.join("report.ax7"),"document":state,
            "processId":std::process::id(),"subject":{}
        }))
        .unwrap(),
    )
    .unwrap();
    let mut app = fixture();
    assert!(app.load_apex_context(&root).is_err());
    assert!(app.apex_session.is_none());
}

fn host_command(
    app: &mut App,
    root: &std::path::Path,
    op: &str,
    argument: serde_json::Value,
) -> serde_json::Value {
    let id = "0123456789abcdef0123456789abcdef";
    app.apex_session = Some(root.to_owned());
    std::fs::write(
        root.join("command.json"),
        serde_json::to_vec(&serde_json::json!({"id":id,"op":op,"argument":argument})).unwrap(),
    )
    .unwrap();
    let task = app.apex_poll();
    finish_task(app, task);
    serde_json::from_slice(&std::fs::read(root.join(format!("response-{id}.json"))).unwrap())
        .unwrap()
}

#[test]
fn apex_compat_live_snapshot_supports_a_new_empty_document() {
    let mut app = App::boot(1.).0;
    let root = output("live-empty");
    assert_eq!(
        host_command(&mut app, &root, "snapshot", serde_json::Value::Null)["ok"],
        true
    );
    assert!(root.join("model.json").exists());
    assert!(root.join("snapshot.json").exists());
    assert!(!root.join("return-ready").exists());
}

#[test]
fn apex_compat_host_area_delete_updates_geometry_and_can_be_undone() {
    let mut app = fixture();
    let root = output("live-delete");
    let id = app.drawing.polygons[0].id;
    assert_eq!(
        host_command(&mut app, &root, "zoom", serde_json::json!({"id":id}))["ok"],
        true
    );
    assert_eq!(
        host_command(&mut app, &root, "delete", serde_json::json!({"id":id}))["ok"],
        true
    );
    assert!(app.drawing.polygons.is_empty());
    assert!(app.drawing.lines.is_empty());
    let _ = app.undo_step();
    assert_eq!(app.drawing.polygons[0].id, id);
}

#[test]
fn apex_compat_load_accepts_a_converted_native_model_and_rejects_outside_paths() {
    let mut source = fixture();
    let root = output("live-load");
    let model = source.build_save_model();
    let path = root.join("native-open.json");
    std::fs::write(&path, serde_json::to_vec(&model).unwrap()).unwrap();
    let mut app = App::boot(1.).0;
    assert_eq!(
        host_command(&mut app, &root, "load", serde_json::json!({"path":path}))["ok"],
        true
    );
    assert_eq!(app.drawing.polygons[0].name, "Compatibility first floor");
    assert_eq!(
        host_command(
            &mut app,
            &root,
            "load",
            serde_json::json!({"path":root.join("../outside.json")})
        )["ok"],
        false
    );
    let mut invalid = model.clone();
    invalid["pages"][0]["symbols"] =
        serde_json::json!([{"id":500,"type":"wrong-schema","x":0,"y":0}]);
    std::fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
    assert_eq!(
        host_command(&mut app, &root, "load", serde_json::json!({"path":path}))["ok"],
        false
    );
    assert_eq!(app.drawing.polygons[0].name, "Compatibility first floor");
}

#[test]
fn apex_compat_clearing_public_tags_preserves_native_area_fields() {
    let mut app = fixture();
    let area = &mut app.drawing.polygons[0];
    area.attributes
        .insert("apex.description".into(), "Native room description".into());
    area.attributes
        .insert("apex.averageWallHeight".into(), "9".into());
    let id = area.id;
    let root = output("clear-public-tags");
    assert_eq!(
        host_command(
            &mut app,
            &root,
            "tag",
            serde_json::json!({"id":id,"operation":"clear"})
        )["ok"],
        true
    );
    let attributes = &app.drawing.polygons[0].attributes;
    assert_eq!(attributes.len(), 3);
    assert_eq!(attributes["apex.calculation"], "net");
    assert_eq!(attributes["apex.description"], "Native room description");
    assert_eq!(attributes["apex.averageWallHeight"], "9");
    let snapshot: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("snapshot.json")).unwrap()).unwrap();
    assert_eq!(
        snapshot["pages"][0]["areas"][0]["description"],
        "Native room description"
    );
    assert_eq!(snapshot["pages"][0]["areas"][0]["averageWallHeight"], 9.);
    assert_eq!(
        snapshot["pages"][0]["areas"][0]["tags"],
        serde_json::json!({})
    );
}

#[test]
fn apex_compat_native_units_and_host_defaults_survive_skx_without_changing_extended_data() {
    let mut app = fixture();
    app.extended_data = "Opaque suite data".into();
    app.native_apex = serde_json::json!({"units":2,"hostCodes":[{"code":"GLA1","description":"Host-defined room","nativeProperties":{"wallHeight":3.0,"smartLabelCode":"ROOM"}}]});
    let model = app.build_save_model();
    assert_eq!(
        app.drawing.polygons[0].attributes["apex.description"],
        "Host-defined room"
    );
    assert!(
        (app.drawing.polygons[0].attributes["apex.averageWallHeight"]
            .parse::<f64>()
            .unwrap()
            - 3. / 0.3048)
            .abs()
            < 1e-8
    );
    let bytes = sketch_core::skx::write(&model, &Default::default()).unwrap();
    let mut restored = App::boot(1.).0;
    let _ = restored.apply_model(
        &sketch_core::skx::read(&bytes).unwrap().model,
        "native provenance",
    );
    assert_eq!(restored.native_apex, app.native_apex);
    assert_eq!(restored.extended_data, "Opaque suite data");
}

#[test]
fn unchanged_host_reads_reuse_images_and_preserve_canvas() {
    let mut app = fixture();
    let root = output("unchanged-refresh");
    app.apex_session = Some(root.clone());
    let _ = app.update(Message::Noop);
    app.write_apex_snapshot(&root, false).unwrap();
    let packet = std::fs::read(root.join("snapshot.json")).unwrap();
    let epoch = app.apex_snapshot_epoch;
    for _ in 0..5 {
        assert_eq!(
            host_command(&mut app, &root, "snapshot", serde_json::Value::Null)["ok"],
            true
        );
        let _ = app.update(Message::ApexPoll);
    }
    assert_eq!(std::fs::read(root.join("snapshot.json")).unwrap(), packet);
    assert_eq!(app.apex_snapshot_epoch, epoch);
    assert!(app.pointer_scene_ready);
    assert!(!app.apex_snapshot_busy);
}
#[test]
fn background_refresh_keeps_input_live_and_cannot_replace_a_newer_save() {
    let mut app = fixture();
    let root = output("background-freshness");
    app.apex_session = Some(root.clone());
    let _ = app.update(Message::Noop);
    app.drawing.polygons[0].name = "Background revision".into();
    app.apex_changed_at = std::time::Instant::now() - std::time::Duration::from_secs(1);
    let task = app.update(Message::ApexPoll);
    assert!(
        app.apex_snapshot_busy,
        "image generation runs after update returns"
    );
    let _ = app.update(Message::CanvasHover(50., 60.));
    let expected = app.world(50., 60.);
    assert_eq!(app.drawing.pointer, expected);
    assert!(app.pointer_scene_ready);
    app.drawing.polygons[0].name = "Explicit saved revision".into();
    app.write_apex_snapshot(&root, false).unwrap();
    let saved = std::fs::read(root.join("snapshot.json")).unwrap();
    finish_task(&mut app, task);
    assert_eq!(std::fs::read(root.join("snapshot.json")).unwrap(), saved);
    let model: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("model.json")).unwrap()).unwrap();
    assert_eq!(
        model["pages"][0]["polygons"][0]["name"],
        "Explicit saved revision"
    );
}
#[test]
fn edits_during_background_refresh_publish_the_latest_revision_next() {
    let mut app = fixture();
    let root = output("background-edit");
    app.apex_session = Some(root.clone());
    let _ = app.update(Message::Noop);
    app.apex_changed_at = std::time::Instant::now() - std::time::Duration::from_secs(1);
    let task = app.update(Message::ApexPoll);
    assert!(app.apex_snapshot_busy);
    app.drawing.polygons[0].name = "Latest live edit".into();
    let _ = app.update(Message::Noop);
    finish_task(&mut app, task);
    assert!(app.apex_check_pending);
    app.apex_changed_at = std::time::Instant::now() - std::time::Duration::from_secs(1);
    let task = app.update(Message::ApexPoll);
    finish_task(&mut app, task);
    let model: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("model.json")).unwrap()).unwrap();
    assert_eq!(model["pages"][0]["polygons"][0]["name"], "Latest live edit");
    assert!(!app.apex_check_pending);
}

#[test]
fn keyboard_input_reads_the_latest_coalesced_mouse_position() {
    let mut app = App::boot(1.).0;
    app.view.mode = view_state::Mode::DrawExterior;
    app.canvas_pointer.set(Some((100., 120.)));
    let expected = app.world(100., 120.);
    let _ = app.update(Message::Key(input::KeyCmd::Buffer("2")));
    assert_eq!(app.drawing.pointer, expected);
    assert_eq!(app.drawing.buffer, "2");
}

fn close_task_actions(task: Task<Message>) -> Vec<iced_runtime::Action<Message>> {
    use iced::futures::StreamExt;
    iced_runtime::task::into_stream(task)
        .map(|stream| iced::futures::executor::block_on(stream.collect()))
        .unwrap_or_default()
}
fn close_task_exits(task: Task<Message>) -> bool {
    close_task_actions(task)
        .into_iter()
        .any(|a| matches!(a, iced_runtime::Action::Exit))
}
#[test]
fn report_close_cancel_and_escape_keep_report_and_sketch_unchanged() {
    for message in [dialog::DialogMsg::Cancel, dialog::DialogMsg::Dismiss] {
        let mut app = fixture();
        let root = output(&format!("report-close-cancel-{message:?}"));
        app.apex_session = Some(root.clone());
        let _ = app.update(Message::Noop); // settle ordinary area/layer normalization
        let original = app.drawing.polygons.clone();
        let _ = app.update(Message::Close);
        assert_eq!(
            app.dialog.as_ref().unwrap().on_ok,
            dialog::Action::ReturnToReport
        );
        assert!(!close_task_exits(app.update(Message::Dialog(message))));
        assert!(app.dialog.is_none());
        assert_eq!(app.drawing.polygons, original);
        assert!(!root.join("return-ready").exists());
        assert!(!root.join("cancel").exists());
    }
}
#[test]
fn report_close_discard_closes_without_signaling_a_measurement_transfer() {
    let mut app = fixture();
    let root = output("report-close-discard");
    let report = root.join("original.ax7");
    std::fs::write(&report, b"previous report sketch").unwrap();
    app.apex_session = Some(root.clone());
    let _ = app.update(Message::Close);
    assert!(close_task_exits(
        app.update(Message::Dialog(dialog::DialogMsg::Discard))
    ));
    assert!(root.join("cancel").exists());
    assert!(!root.join("return-ready").exists());
    assert!(!root.join("save-ready").exists());
    assert_eq!(std::fs::read(report).unwrap(), b"previous report sketch");
}
#[test]
fn report_close_save_transfers_current_measurements_and_only_exits_after_success() {
    for (accepted, suffix) in [(true, "accepted"), (false, "rejected")] {
        let mut app = fixture();
        let root = output(&format!("report-close-save-{suffix}"));
        app.apex_session = Some(root.clone());
        // A stale successful receipt must not close a newly edited document.
        std::fs::write(root.join("return-receipt.json"), br#"{"ok":true}"#).unwrap();
        app.drawing.polygons[0].name = "Current closing revision".into();
        let response_root = root.clone();
        let host = std::thread::spawn(move || {
            let start = std::time::Instant::now();
            while !response_root.join("return-ready").exists() {
                assert!(start.elapsed().as_secs() < 60);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert!(!response_root.join("return-receipt.json").exists());
            std::thread::sleep(std::time::Duration::from_millis(150));
            std::fs::write(
                response_root.join("return-receipt.json"),
                serde_json::to_vec(
                    &serde_json::json!({"ok":accepted,"error":"Report rejected transfer"}),
                )
                .unwrap(),
            )
            .unwrap();
        });
        let _ = app.update(Message::Close);
        let task = app.update(Message::Dialog(dialog::DialogMsg::Ok));
        assert!(app.apex_returning);
        assert!(app.dialog.is_none());
        let packet: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("result.json")).unwrap()).unwrap();
        assert_eq!(packet["pages"][0]["areas"][0]["netArea"], 200.);
        assert_eq!(
            packet["pages"][0]["areas"][0]["name"],
            "Current closing revision"
        );
        let _ = app.update(Message::Close);
        assert!(
            app.dialog.is_none(),
            "a second close cannot discard a pending transfer"
        );
        let actions = close_task_actions(task);
        assert!(!actions
            .iter()
            .any(|a| matches!(a, iced_runtime::Action::Exit)));
        let mut replied = false;
        for action in actions {
            if let iced_runtime::Action::Output(message @ Message::ApexReturned(_)) = action {
                replied = true;
                assert_eq!(close_task_exits(app.update(message)), accepted);
            }
        }
        host.join().unwrap();
        assert!(replied);
        assert!(!app.apex_returning);
        if !accepted {
            assert_eq!(app.toasts.text(), Some("Report rejected transfer"));
            assert_eq!(app.drawing.polygons[0].name, "Current closing revision");
            assert!(!root.join("cancel").exists());
        }
    }
}
#[test]
fn report_close_preparation_error_and_busy_guard_keep_sketch_open() {
    let mut app = fixture();
    let root = output("report-close-prepare-error");
    let blocked = root.join("not-a-directory");
    std::fs::write(&blocked, b"blocked").unwrap();
    app.apex_session = Some(blocked);
    let _ = app.update(Message::Close);
    assert!(!close_task_exits(
        app.update(Message::Dialog(dialog::DialogMsg::Ok))
    ));
    assert!(!app.apex_returning);
    assert_eq!(app.drawing.polygons.len(), 1);
    assert!(app.toasts.text().is_some());
    app.apex_returning = true;
    let _ = app.update(Message::Close);
    assert!(app.dialog.is_none());
    assert!(!close_task_exits(
        app.update(Message::Dialog(dialog::DialogMsg::Discard))
    ));
}
