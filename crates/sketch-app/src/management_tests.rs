//! End-to-end model regressions for persistent library, layer IO and recovery.
use super::*;
use serde_json::json;
use symbol_manager::Msg as S;
fn app() -> App {
    App::boot(1.).0
}
fn symbol(id: i64, kind: &str) -> placed_symbol::PlacedSymbol {
    serde_json::from_value(json!({"id":id,"symbolType":kind,"x":3.,"y":4.})).unwrap()
}
fn select(a: &mut App, id: &str) {
    let _ = a.manager_msg(S::Open);
    let _ = a.manager_msg(S::Toggle(id.into()));
}
fn first(a: &App) -> String {
    a.symbols_lib().categories[0].items[0].id.clone()
}
#[test]
fn library_rename_hide_restore_and_favorites_survive_restart() {
    persist::with_temp_store(|| {
        assert!(persist::enabled());
        assert_eq!(
            persist::write("probe.txt", "ok"),
            Some(true),
            "profile {:?}",
            persist::path("probe.txt")
        );
        let mut a = app();
        let id = first(&a);
        a.drawing.symbols.push(symbol(10, &id));
        select(&mut a, &id);
        let other = a
            .symbols_lib()
            .library
            .symbols
            .keys()
            .find(|key| *key != &id)
            .unwrap()
            .clone();
        let _ = a.manager_msg(S::Inspect(other.clone()));
        assert_eq!(a.symbol_manager.focus.as_ref(), Some(&other));
        let _ = a.manager_msg(S::Inspect(id.clone()));
        let _ = a.manager_msg(S::Name("My fixture".into()));
        let _ = a.manager_msg(S::Rename);
        let _ = a.manager_msg(S::Hide);
        assert_eq!(a.symbol_manager.confirm, 3);
        assert!(a.symbols_lib().by_id(&id).is_some());
        let _ = a.manager_msg(S::Confirm);
        assert!(a.symbols_lib().by_id(&id).is_none());
        assert!(a.symbols_lib().symbol(&id).is_some());
        assert_eq!(a.drawing.symbols.len(), 1);
        let b = app();
        assert!(b.symbols_lib().by_id(&id).is_none());
        assert_eq!(b.symbols_lib().symbol(&id).unwrap().label, "My fixture");
        let _ = a.manager_msg(S::Restore);
        assert!(a.symbols_lib().by_id(&id).is_some());
        let _ = a.manager_msg(S::Favorite);
        let favorite = a.favorite_types().contains(&id);
        assert_eq!(app().favorite_types().contains(&id), favorite);
    })
}
#[test]
fn delete_requires_two_confirmations_and_removes_all_pages_and_history() {
    symlib::with_temp_library(|| {
        persist::with_temp_store(|| {
            let mut a = app();
            let id = first(&a);
            a.drawing.symbols.push(symbol(12, &id));
            a.pages.pages.push(pages::Page {
                symbols: vec![symbol(13, &id)],
                ..Default::default()
            });
            a.push_undo("Fixture");
            select(&mut a, &id);
            let _ = a.manager_msg(S::Delete);
            let _ = a.manager_msg(S::Confirm);
            assert_eq!(a.drawing.symbols.len(), 1);
            let _ = a.manager_msg(S::Cancel);
            assert_eq!(a.drawing.symbols.len(), 1);
            let _ = a.manager_msg(S::Delete);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::Confirm);
            assert!(a.drawing.symbols.is_empty());
            assert!(a.pages.pages.iter().all(|p| p.symbols.is_empty()));
            assert!(!file_library().symbols.contains_key(&id));
            assert!(app().symbols_lib().symbol(&id).is_none());
            let mut data = layer_fixture();
            data["symbols"][0]["symbolType"] = json!(id);
            a.import_layer_data(&data).unwrap();
            assert!(a.drawing.symbols.is_empty());
            let snapshot = a.undo.undo(a.snapshot_model("Current")).unwrap();
            assert!(snapshot.symbols.is_empty());
        })
    });
}
#[test]
fn legacy_purge_preferences_stay_deleted_and_cannot_be_restored() {
    persist::with_temp_store(|| {
        let id = first(&app());
        let legacy = json!({"purged":[id.clone()],"purgedCategories":["old-empty-category"]});
        assert_eq!(
            persist::write_json("symbol-library.json", &legacy),
            Some(true)
        );
        let mut a = app();
        assert!(a.symbol_changes.deleted.contains(&id));
        assert!(a
            .symbol_changes
            .deleted_categories
            .contains("old-empty-category"));
        assert!(a.symbols_lib().symbol(&id).is_none());
        let _ = a.manager_msg(S::Open);
        let _ = a.manager_msg(S::Toggle(id.clone()));
        let _ = a.manager_msg(S::Restore);
        assert!(a.symbol_changes.deleted.contains(&id));
        assert!(app().symbols_lib().symbol(&id).is_none());
        assert!(!a.manager_symbols().iter().any(|s| s.id == id));
        let saved: serde_json::Value =
            serde_json::from_str(&persist::read("symbol-library.json").unwrap()).unwrap();
        assert!(saved.get("deleted").is_some());
        assert!(saved.get("purged").is_none());
    });
}
#[test]
fn delete_removes_the_symbol_from_the_library_file_and_everything_else() {
    symlib::with_temp_library(|| {
        persist::with_temp_store(|| {
            let mut a = app();
            let id = first(&a);
            let before = a.symbols_lib().library.symbols.len();
            a.drawing.symbols.push(symbol(14, &id));
            a.push_undo("Fixture");
            a.symbol_favorites = Some(vec![id.clone()]);
            select(&mut a, &id);
            let _ = a.manager_msg(S::Delete);
            let _ = a.manager_msg(S::Confirm);
            assert_eq!(
                a.drawing.symbols.len(),
                1,
                "one confirmation changes nothing"
            );
            let _ = a.manager_msg(S::Cancel);
            let _ = a.manager_msg(S::Delete);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::Confirm);
            assert!(a.drawing.symbols.is_empty());
            assert!(a.symbols_lib().symbol(&id).is_none());
            assert!(
                a.symbol_changes.deleted.contains(&id),
                "old documents cannot reintroduce the deleted id"
            );
            let file = sketch_core::symbols::SymbolLibrary::load(&symlib::library_path()).unwrap();
            assert_eq!(file.symbols.len(), before - 1);
            assert!(file.symbol(&id).is_none());
            assert!(file.categories.iter().all(|c| !c.symbol_ids.contains(&id)));
            assert!(!file.default_favorites.contains(&id));
            assert!(symlib::library_path()
                .with_file_name("symbols.json.bak")
                .is_file());
            assert!(app().symbols_lib().symbol(&id).is_none());
            assert!(!app().favorite_types().contains(&id));
            let snapshot = a.undo.undo(a.snapshot_model("Current")).unwrap();
            assert!(snapshot.symbols.is_empty());
        })
    })
}
#[test]
fn delete_symbol_and_then_its_whole_category() {
    symlib::with_temp_library(|| {
        persist::with_temp_store(|| {
            let mut a = app();
            let id = first(&a);
            let category = a.symbols_lib().symbol(&id).unwrap().category.clone();
            let in_category = a
                .symbols_lib()
                .library
                .symbols
                .values()
                .filter(|s| s.category == category)
                .count();
            let total = a.symbols_lib().library.symbols.len();
            select(&mut a, &id);
            let _ = a.manager_msg(S::Delete);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::Confirm);
            assert!(a.symbol_changes.deleted.contains(&id));
            let _ = a.manager_msg(S::Category(category.clone()));
            let _ = a.manager_msg(S::DeleteCategory);
            assert_eq!(a.symbol_manager.selected.len(), in_category - 1);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::Confirm);
            let file = sketch_core::symbols::SymbolLibrary::load(&symlib::library_path()).unwrap();
            assert_eq!(file.symbols.len(), total - in_category);
            assert!(!file.categories.iter().any(|c| c.key == category));
            assert!(!app()
                .symbols_lib()
                .categories
                .iter()
                .any(|c| c.key == category));
        })
    })
}
#[test]
fn delete_write_failure_changes_nothing_and_can_be_retried() {
    symlib::with_temp_library(|| {
        persist::with_temp_store(|| {
            let mut a = app();
            let id = first(&a);
            let path = symlib::library_path();
            let before = std::fs::read(&path).unwrap();
            a.drawing.symbols.push(symbol(14, &id));
            a.push_undo("Fixture");
            select(&mut a, &id);
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_readonly(true);
            std::fs::set_permissions(&path, perms.clone()).unwrap();
            let _ = a.manager_msg(S::Delete);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::Confirm);
            assert!(a.symbols_lib().symbol(&id).is_some());
            assert_eq!(a.drawing.symbols.len(), 1);
            assert!(!a.symbol_changes.deleted.contains(&id));
            assert!(a.symbol_manager.selected.contains(&id));
            assert_eq!(a.symbol_manager.confirm, 6);
            assert_eq!(std::fs::read(&path).unwrap(), before);
            assert!(a.toasts.text().unwrap().contains("Nothing was deleted"));
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            std::fs::set_permissions(&path, perms).unwrap();
            let _ = a.manager_msg(S::Confirm);
            assert!(a.drawing.symbols.is_empty());
            assert!(!file_library().symbols.contains_key(&id));
        })
    });
}
fn file_library() -> sketch_core::symbols::SymbolLibrary {
    sketch_core::symbols::SymbolLibrary::load(&symlib::library_path()).unwrap()
}
#[test]
fn duplicates_view_lists_keepers_first_and_merge_retargets_everything() {
    symlib::with_temp_library(|| {
        persist::with_temp_store(|| {
            let mut a = app();
            let groups = a.duplicate_groups();
            assert!(!groups.is_empty());
            let group = groups.iter().find(|g| g.len() == 2).unwrap().clone();
            assert!(!group[0].starts_with("favorites/") || group[1].starts_with("favorites/"));
            let _ = a.manager_msg(S::Open);
            let _ = a.manager_msg(S::Category("Duplicates".into()));
            let listed: Vec<String> = a.manager_symbols().iter().map(|s| s.id.clone()).collect();
            assert_eq!(listed.len(), groups.iter().map(Vec::len).sum::<usize>());
            let at = listed.iter().position(|id| *id == group[0]).unwrap();
            assert_eq!(
                listed[at + 1],
                group[1],
                "a group is listed together, keeper first"
            );
            // Place the copy that will be retired, keep it in history and favorites.
            a.drawing.symbols.push(symbol(30, &group[1]));
            a.push_undo("Fixture");
            a.symbol_favorites = Some(vec![group[1].clone(), group[0].clone()]);
            let before = a.symbols_lib().library.symbols.len();
            let retired = a.merge_pairs().len();
            let _ = a.manager_msg(S::MergeDuplicates);
            assert_eq!(a.symbol_manager.confirm, 10);
            assert_eq!(
                file_library().symbols.len(),
                before,
                "nothing is written before Confirm"
            );
            let _ = a.manager_msg(S::Confirm);
            assert_eq!(file_library().symbols.len(), before - retired);
            assert!(a.symbols_lib().library.symbol(&group[1]).is_none());
            assert_eq!(a.drawing.symbols[0].symbol_type, group[0]);
            assert_eq!(a.symbol_favorites.as_deref(), Some(&[group[0].clone()][..]));
            let snapshot = a.undo.undo(a.snapshot_model("Current")).unwrap();
            assert_eq!(snapshot.symbols[0].symbol_type, group[0]);
            assert!(a.merge_pairs().is_empty(), "nothing left to merge");
            assert_eq!(file_library().aliases[&group[1]], group[0]);
            // A sketch saved before the merge still finds the symbol.
            let raw = json!({"symbols":[{"id":31,"symbolType":group[1],"x":1.,"y":2.}]});
            let page = a.restore_page(&raw);
            assert_eq!(page.symbols[0].symbol_type, group[0]);
            assert!(app().symbols_lib().library.aliases.contains_key(&group[1]));
        })
    })
}
#[test]
fn move_rename_and_rename_category_edit_the_library_file() {
    symlib::with_temp_library(|| {
        persist::with_temp_store(|| {
            let mut a = app();
            let id = first(&a);
            let category = a.symbols_lib().symbol(&id).unwrap().category.clone();
            select(&mut a, &id);
            let _ = a.manager_msg(S::Move);
            assert_eq!(a.symbol_manager.confirm, 0, "Move needs a target category");
            let _ = a.manager_msg(S::Target("Tidy Things".into()));
            let _ = a.manager_msg(S::Move);
            assert_eq!(a.symbol_manager.confirm, 7);
            let _ = a.manager_msg(S::Cancel);
            assert_eq!(file_library().symbols[&id].category, category);
            let _ = a.manager_msg(S::Move);
            let _ = a.manager_msg(S::Confirm);
            let file = file_library();
            assert_eq!(file.symbols[&id].category, "tidy-things");
            assert_eq!(file.symbols[&id].category_name, "Tidy Things");
            assert_eq!(
                file.categories.last().unwrap().symbol_ids,
                std::slice::from_ref(&id)
            );
            assert!(a
                .symbols_lib()
                .categories
                .iter()
                .any(|c| c.key == "tidy-things"));
            let _ = a.manager_msg(S::Name("Neat Thing".into()));
            let _ = a.manager_msg(S::RenameInFile);
            let _ = a.manager_msg(S::Confirm);
            assert_eq!(file_library().symbols[&id].label, "Neat Thing");
            assert_eq!(a.symbols_lib().symbol(&id).unwrap().label, "Neat Thing");
            assert!(
                a.symbol_changes.renamed.is_empty(),
                "the file holds the name now"
            );
            let _ = a.manager_msg(S::Category("tidy-things".into()));
            let _ = a.manager_msg(S::Target("Tidied".into()));
            let _ = a.manager_msg(S::RenameCategoryInFile);
            let _ = a.manager_msg(S::Confirm);
            let file = file_library();
            assert_eq!(file.categories.last().unwrap().name, "Tidied");
            assert_eq!(file.symbols[&id].category_name, "Tidied");
            let _ = a.manager_msg(S::Category("Duplicates".into()));
            let _ = a.manager_msg(S::Target("Whatever".into()));
            let _ = a.manager_msg(S::RenameCategoryInFile);
            assert_eq!(a.symbol_manager.confirm, 0, "a view is not a category");
        })
    })
}
#[test]
fn largest_view_sorts_by_geometry_size() {
    persist::with_temp_store(|| {
        let mut a = app();
        let _ = a.manager_msg(S::Open);
        let _ = a.manager_msg(S::Category("Largest".into()));
        let sizes: Vec<usize> = a
            .manager_symbols()
            .iter()
            .map(|s| s.geometry.len())
            .collect();
        assert!(sizes.len() > 100);
        assert!(sizes.windows(2).all(|w| w[0] >= w[1]));
    })
}
#[test]
fn category_delete_and_hidden_bulk_actions_are_persistent() {
    symlib::with_temp_library(|| {
        persist::with_temp_store(|| {
            let mut a = app();
            let id = first(&a);
            let category = a.symbols_lib().symbol(&id).unwrap().category.clone();
            select(&mut a, &id);
            let _ = a.manager_msg(S::Hide);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::RestoreAll);
            assert!(a.symbol_changes.hidden.contains(&id));
            let _ = a.manager_msg(S::Confirm);
            assert!(!a.symbol_changes.hidden.contains(&id));
            let _ = a.manager_msg(S::Category(category.clone()));
            let _ = a.manager_msg(S::DeleteCategory);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::Confirm);
            assert!(a.symbol_changes.deleted_categories.contains(&category));
            assert!(!app()
                .symbols_lib()
                .categories
                .iter()
                .any(|c| c.key == category));
        })
    });
}
const DXF: &str =
    "0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n12\n21\n6\n0\nENDSEC\n0\nEOF\n";
#[test]
fn caddillo_dxf_manager_import_keeps_size_metadata_and_survives_reopen() {
    persist::with_temp_store(|| {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../refs/caddillo/fixtures");
        let mut a = app();
        for name in [
            "DOOR-SWING-36",
            "WINDOW-SINGLE-36",
            "WASHER-27",
            "TUB",
            "BED-KING",
            "STAIR-SPIRAL-60-LH",
        ] {
            let body = std::fs::read_to_string(root.join(format!("{name}.dxf"))).unwrap();
            a.import_symbol(&body, name, "dxf").unwrap();
        }
        a.rebuild_symbols();
        let _ = a.manager_msg(S::Restore); // same store_symbols path as import completion
        assert_eq!(a.symbol_changes.custom.len(), 6);
        let reopened = app();
        assert_eq!(reopened.symbol_changes.custom.len(), 6);
        for (id, original) in &a.symbol_changes.custom {
            let restored = &reopened.symbol_changes.custom[id];
            assert_eq!(restored.name, original.name);
            assert_eq!(restored.geometry.len(), original.geometry.len());
            assert_eq!(restored.wall_snap, original.wall_snap);
            assert!((restored.wall_gap_width - original.wall_gap_width).abs() < 1e-9);
            assert!((restored.bounds.max_x - original.bounds.max_x).abs() < 1e-9);
            assert!((restored.bounds.max_y - original.bounds.max_y).abs() < 1e-9);
        }
        let door = a
            .symbol_changes
            .custom
            .values()
            .find(|s| s.name == "DOOR-SWING-36")
            .unwrap()
            .clone();
        assert_eq!(door.wall_gap_width, 36.0);
        assert_eq!(door.wall_snap, sketch_core::symbols::WallSnap::Side);
        let washer = a
            .symbol_changes
            .custom
            .values()
            .find(|s| s.name == "WASHER-27")
            .unwrap();
        assert_eq!((washer.bounds.max_x - washer.bounds.min_x) / 12., 2.25);
        let id = door.id.clone();
        a.drawing.symbols.push(symbol(99, &id));
        let save = a.build_save_model();
        let mut portable = app();
        let _ = portable.apply_model(&save, "Imported residential symbols");
        assert_eq!(
            portable.symbols_lib().symbol(&id).unwrap().geometry,
            door.geometry
        );
    });
}

#[test]
fn caddillo_export_contact_sheet_contains_residential_geometry() {
    persist::with_temp_store(|| {
        let mut a = app();
        let names = [
            "door-swing-36",
            "window-single-36",
            "door-double-72",
            "door-pocket-32",
            "tub",
            "toilet",
            "shower-60x32",
            "sink-bath-rect",
            "fridge-36",
            "range-30",
            "washer-27",
            "dryer-27",
            "bed-king",
            "couch-84",
            "dining-table",
            "stair-spiral-60-lh",
        ];
        for (i, name) in names.iter().enumerate() {
            a.place_symbol_at(
                sketch_core::geometry::Pt::new((i % 4) as f64 * 10., -((i / 4) as f64) * 10.),
                format!("caddillo/{name}"),
            );
        }
        let bytes = a.png_export_bytes().unwrap();
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/caddillo-024");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("residential-contact-sheet.png"), bytes).unwrap();
        assert_eq!(a.drawing.symbols.len(), 16);
    });
}
#[test]
fn custom_dxf_import_is_portable_and_rejects_invalid_or_dwg() {
    persist::with_temp_store(|| {
        let mut a = app();
        a.import_symbol(DXF, "Fixture", "dxf").unwrap();
        a.rebuild_symbols();
        let id = a.symbol_manager.selected.first().unwrap().clone();
        assert!(a.symbols_lib().symbol(&id).is_some());
        a.drawing.symbols.push(symbol(9, &id));
        let save = a.build_save_model();
        let mut b = app();
        let _ = b.apply_model(&save, "Fixture");
        assert!(b.symbols_lib().symbol(&id).is_some());
        assert_eq!(b.drawing.symbols[0].symbol_type, id);
        let count = a.symbol_changes.custom.len();
        assert!(a.import_symbol("garbage", "Bad", "dxf").is_err());
        assert!(a.import_symbol("{}", "Bad", "json").is_err());
        assert!(a
            .import_symbol("AC1027", "Bad", "dwg")
            .unwrap_err()
            .contains("DXF"));
        assert_eq!(a.symbol_changes.custom.len(), count);
    })
}
fn layer_fixture() -> serde_json::Value {
    json!({"version":"Sketch-Layer","layers":[{"id":"group","name":"Group","isGroup":true},{"id":"leaf","name":"Leaf","parentId":"group"}],"lines":[{"id":20,"start":{"x":1.,"y":2.},"end":{"x":11.,"y":2.},"type":"exterior","layerId":"leaf"}],"labels":[{"id":21,"x":2.,"y":3.,"text":"Label","layerId":"leaf"}],"symbols":[{"id":22,"x":3.,"y":4.,"symbolType":"fixture","wallId":20,"layerId":"leaf"}],"polygons":[],"dimensionChains":[{"id":23,"seedLineId":20}]})
}
#[test]
fn layer_import_remaps_tree_geometry_openings_and_chains_and_undo() {
    let mut a = app();
    let n = a.layers.items.len();
    a.import_layer_data(&layer_fixture()).unwrap();
    assert_eq!(a.layers.items.len(), n + 2);
    let leaf = a.layers.items.last().unwrap();
    assert_ne!(leaf.id, "leaf");
    assert_eq!(
        leaf.parent_id.as_deref(),
        Some(a.layers.items[n].id.as_str())
    );
    assert_eq!(
        a.drawing.lines[0].layer_id.as_deref(),
        Some(leaf.id.as_str())
    );
    assert_eq!(a.drawing.texts[0].layer_id, (n + 2) as i64);
    assert_eq!(a.drawing.symbols[0].wall_id, Some(a.drawing.lines[0].id));
    assert_eq!(
        a.drawing.dimension_chains[0].seed_line_id,
        a.drawing.lines[0].id
    );
    assert_eq!(a.drawing.lines[0].start.x, 1.);
    let snap = a.undo.undo(a.snapshot_model("Current")).unwrap();
    a.restore_snapshot(&snap);
    assert_eq!(a.layers.items.len(), n);
    assert!(a.drawing.lines.is_empty());
}
#[test]
fn layer_export_import_round_trip_preserves_named_entity_layers() {
    let mut a = app();
    a.import_layer_data(&layer_fixture()).unwrap();
    let id = a.layers.items[a.layers.items.len() - 2].id.clone();
    let data = a.export_layer_data(&id);
    assert_eq!(data["labels"].as_array().unwrap().len(), 1);
    assert!(data["labels"][0]["layerId"].is_string());
    assert!(data["symbols"][0]["layerId"].is_string());
    assert!(data["lines"][0]["layerId"].is_string());
    let mut b = app();
    b.import_layer_data(&data).unwrap();
    assert_eq!(b.drawing.lines.len(), 1);
    assert_eq!(b.drawing.texts[0].text, "Label");
    assert_eq!(
        b.drawing.dimension_chains[0].seed_line_id,
        b.drawing.lines[0].id
    );
    let save = b.build_save_model();
    assert!(save["pages"][0]["lines"][0]["layerId"].is_string());
}
#[test]
fn invalid_layer_files_never_mutate_document_or_undo() {
    let mut a = app();
    let before = a.layers.items.clone();
    let mut cycle = layer_fixture();
    cycle["layers"][0]["parentId"] = json!("leaf");
    let mut duplicate = layer_fixture();
    duplicate["layers"][1]["id"] = json!("group");
    let mut malformed = layer_fixture();
    malformed["symbols"] = json!("bad");
    for data in [json!({}), cycle, duplicate, malformed] {
        assert!(a.import_layer_data(&data).is_err());
        assert_eq!(a.layers.items, before);
        assert!(!a.undo.can_undo());
    }
    let mut data = layer_fixture();
    data["lines"][0]["start"] = json!("invalid");
    assert!(a.import_layer_data(&data).is_err());
    assert_eq!(a.layers.items, before);
    assert!(!a.undo.can_undo());
}
#[test]
fn autosave_skips_unchanged_and_disabled_then_recovers_all_pages() {
    persist::with_temp_store(|| {
        let mut a = app();
        let _ = a.autosave_tick(true);
        assert!(a.autosave.path.is_none());
        a.file_name = "Recovery fixture".into();
        a.extended_data = "unsaved work".into();
        a.pages.pages.push(pages::Page {
            north_angle: 90.,
            ..Default::default()
        });
        a.drawing.options.auto_save_enabled = false;
        let _ = a.autosave_tick(true);
        assert!(a.autosave.path.is_none());
        a.drawing.options.auto_save_enabled = true;
        let _ = a.autosave_tick(true);
        let path = a
            .autosave
            .path
            .clone()
            .expect("Autosave should write a valid snapshot");
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(
            sketch_core::skx::read(&bytes).unwrap().model["extendedData"],
            "unsaved work"
        );
        let _ = a.autosave_tick(true);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let mut b = app();
        assert!(b.autosave_prompt);
        let _ = b.recover_autosave(false);
        assert_eq!(b.file_name, "Untitled");
        assert!(path.exists());
        b.autosave_clean();
        assert!(
            path.exists(),
            "saving a different sketch must preserve a recovery deferred with Later"
        );
        b.autosave.recovery = autosave::latest();
        let _ = b.recover_autosave(true);
        assert_eq!(b.file_name, "Recovery fixture");
        assert_eq!(b.pages.pages.len(), 2);
        assert_eq!(b.extended_data, "unsaved work");
        b.autosave_clean();
        assert!(!path.exists());
    })
}
#[test]
fn autosave_failure_preserves_snapshot_and_corrupt_recovery_is_skipped() {
    persist::with_temp_store(|| {
        let mut a = app();
        a.file_name = "Safe".into();
        let _ = a.autosave_tick(true);
        let path = a.autosave.path.clone().unwrap();
        let old = std::fs::read(&path).unwrap();
        let bad = persist::path("autosave/bad.skx").unwrap();
        std::fs::write(&bad, b"corrupt").unwrap();
        assert_eq!(autosave::latest().unwrap().1["fileName"], "Safe");
        a.autosave.path = Some(path.join("invalid.skx"));
        a.file_name = "Changed".into();
        let _ = a.autosave_tick(true);
        assert_eq!(std::fs::read(&path).unwrap(), old);
        assert!(a.toasts.text().unwrap().contains("Autosave failed"));
    })
}
#[test]
fn opening_or_new_document_preserves_previous_recovery() {
    persist::with_temp_store(|| {
        let mut a = app();
        a.file_name = "Old".into();
        let _ = a.autosave_tick(true);
        let old = a.autosave.path.clone().unwrap();
        a.reset_to_blank();
        assert!(old.exists());
        a.file_name = "New".into();
        let _ = a.autosave_tick(true);
        assert_ne!(a.autosave.path.as_ref(), Some(&old));
        assert!(old.exists());
    })
}

#[test]
fn autosave_obeys_elapsed_interval_and_keeps_atomic_replacements_valid() {
    persist::with_temp_store(|| {
        let mut a = app();
        a.file_name = "First snapshot".into();
        a.drawing.options.auto_save_interval = 1.;
        let _ = a.autosave_tick(false);
        assert!(a.autosave.path.is_none());
        a.autosave.last = std::time::Instant::now() - std::time::Duration::from_secs(61);
        let _ = a.autosave_tick(false);
        let path = a.autosave.path.clone().unwrap();
        a.file_name = "Updated snapshot".into();
        a.autosave.last = std::time::Instant::now() - std::time::Duration::from_secs(61);
        let _ = a.autosave_tick(false);
        assert_eq!(
            sketch_core::skx::read(&std::fs::read(path).unwrap())
                .unwrap()
                .model["fileName"],
            "Updated snapshot"
        );
    })
}

#[test]
fn layer_import_preserves_area_membership_edges_and_subtraction_targets() {
    let mut a = app();
    let mut data = layer_fixture();
    data["layers"][1]["areaPolyId"] = json!(24);
    data["polygons"] = json!([
        {"id":24,"points":[{"x":1.,"y":2.},{"x":11.,"y":2.},{"x":11.,"y":12.}],"layerId":"leaf","edges":[{"lineId":20},{},{}]},
        {"id":25,"points":[{"x":2.,"y":3.},{"x":3.,"y":3.},{"x":3.,"y":4.}],"layerId":"leaf","subtractTargetId":24}]);
    a.import_layer_data(&data).unwrap();
    let first = &a.drawing.polygons[0];
    let second = &a.drawing.polygons[1];
    assert_ne!(first.id, 24);
    assert_eq!(second.subtract_target_id, Some(first.id));
    assert_eq!(first.edges[0].line_id, Some(a.drawing.lines[0].id));
    assert_eq!(
        a.layers.items.last().unwrap().area_poly_id,
        Some(first.id.to_string())
    );
}

#[test]
fn category_deletion_fire_resets_filter_and_persists() {
    symlib::with_temp_library(|| {
        persist::with_temp_store(|| {
            let mut a = app();
            let ids: Vec<_> = a
                .symbols_lib()
                .library
                .symbols
                .values()
                .filter(|s| s.category == "fire")
                .map(|s| s.id.clone())
                .collect();
            assert_eq!(ids.len(), 119);
            let _ = a.manager_msg(S::Open);
            let _ = a.manager_msg(S::Category("fire".into()));
            let _ = a.manager_msg(S::DeleteCategory);
            assert_eq!(a.symbol_manager.selected.len(), 119);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::Cancel);
            assert!(ids.iter().all(|id| a.symbols_lib().symbol(id).is_some()));
            let _ = a.manager_msg(S::Filter("fire".into()));
            let _ = a.manager_msg(S::DeleteCategory);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::Confirm);
            assert!(a.symbol_changes.deleted_categories.contains("fire"));
            assert!(a.symbol_manager.category.is_empty());
            assert!(a.symbol_manager.filter.is_empty());
            assert!(!a.manager_symbols().is_empty());
            assert!(ids.iter().all(|id| a.symbols_lib().symbol(id).is_none()));
            assert!(!app()
                .symbols_lib()
                .categories
                .iter()
                .any(|c| c.key == "fire"));
        });
    });
}
#[test]
fn category_deletion_starts_in_the_dock_category() {
    let mut a = app();
    a.dock.active = dock::DockTab::Symbols;
    a.dock.symbol_category = "fire".into();
    let _ = a.manager_msg(S::Open);
    assert_eq!(a.symbol_manager.category, "fire");
}
#[test]
fn category_deletion_handles_empty_categories() {
    symlib::with_temp_library(|| {
        persist::with_temp_store(|| {
            let path = symlib::library_path();
            let mut data: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            data["categories"]
                .as_array_mut()
                .unwrap()
                .push(json!({"key":"empty-fixture","name":"Empty fixture","symbolIds":[]}));
            std::fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
            let mut a = app();
            let _ = a.manager_msg(S::Open);
            let _ = a.manager_msg(S::Category("empty-fixture".into()));
            let _ = a.manager_msg(S::DeleteCategory);
            assert_eq!(a.symbol_manager.confirm, 5);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::Confirm);
            assert!(!app()
                .symbols_lib()
                .categories
                .iter()
                .any(|c| c.key == "empty-fixture"));
        })
    });
}

#[test]
fn category_deletion_forever_removes_an_empty_category_from_the_file() {
    symlib::with_temp_library(|| {
        persist::with_temp_store(|| {
            let path = symlib::library_path();
            let mut data: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            data["categories"].as_array_mut().unwrap().push(
                json!({"key":"empty-file-category","name":"Empty file category","symbolIds":[]}),
            );
            std::fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
            let mut a = app();
            let _ = a.manager_msg(S::Open);
            let _ = a.manager_msg(S::Category("empty-file-category".into()));
            let _ = a.manager_msg(S::DeleteCategory);
            assert_eq!(a.symbol_manager.confirm, 5);
            let _ = a.manager_msg(S::Confirm);
            let _ = a.manager_msg(S::Confirm);
            let saved: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            assert!(!saved["categories"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["key"] == "empty-file-category"));
            assert!(a.symbol_manager.category.is_empty());
            assert!(path.with_file_name("symbols.json.bak").exists());
        })
    });
}

/// Which resize direction a left press at `at` (window px) starts, by
/// running the press through the real resize layer.
fn resize_zone_at(app: &App, at: (f32, f32)) -> Option<String> {
    use iced::advanced::renderer::Headless;
    use iced_runtime::user_interface::{Cache, UserInterface};
    let mut renderer = png_overlay::ready(<iced::Renderer as Headless>::new(
        theme::SEGOE,
        iced::Pixels(16.0),
        Some("tiny-skia"),
    ))
    .flatten()
    .expect("a software renderer");
    let mut ui = UserInterface::build(
        app.resize_layer(),
        app.window_size,
        Cache::default(),
        &mut renderer,
    );
    let mut messages = Vec::new();
    let point = Point::new(at.0, at.1);
    let _ = ui.update(
        &[
            Event::Mouse(mouse::Event::CursorMoved { position: point }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        ],
        mouse::Cursor::Available(point),
        &mut renderer,
        &mut iced::advanced::clipboard::Null,
        &mut messages,
    );
    messages.into_iter().find_map(|m| match m {
        Message::ResizeDrag(direction) => Some(format!("{direction:?}")),
        _ => None,
    })
}
#[test]
fn window_edges_and_corners_start_a_resize_unless_maximized() {
    // The frameless window's own resize zones: within 4px of an edge, or
    // the 10px corner squares; a press just inside them does nothing.
    let mut app = app();
    for (at, expected) in [
        ((700.0, 1.0), Some("North")),
        ((700.0, 858.0), Some("South")),
        ((1.0, 400.0), Some("West")),
        ((1398.0, 400.0), Some("East")),
        ((2.0, 2.0), Some("NorthWest")),
        ((1.0, 8.0), Some("NorthWest")),
        ((1397.0, 857.0), Some("SouthEast")),
        ((1394.0, 2.0), Some("NorthEast")),
        ((2.0, 852.0), Some("SouthWest")),
        ((700.0, 400.0), None),
        ((8.0, 400.0), None),
        ((700.0, 850.0), None),
    ] {
        assert_eq!(
            resize_zone_at(&app, at).as_deref(),
            expected,
            "press at {at:?}"
        );
    }
    let _ = app.update(Message::Maximized(true));
    assert_eq!(resize_zone_at(&app, (700.0, 1.0)), None);
    assert_eq!(resize_zone_at(&app, (2.0, 2.0)), None);
}
fn exits(task: Task<Message>) -> bool {
    use iced::futures::StreamExt;
    let actions: Vec<iced_runtime::Action<Message>> = iced_runtime::task::into_stream(task)
        .map(|stream| iced::futures::executor::block_on(stream.collect()))
        .unwrap_or_default();
    actions
        .iter()
        .any(|a| matches!(a, iced_runtime::Action::Exit))
}
#[test]
fn unsaved_changes_are_tracked_against_the_last_save() {
    let mut a = app();
    assert!(!a.dirty);
    assert_eq!(a.save_state(), "New document");
    // An edit that never touches the undo stack is still seen.
    a.extended_data = "a note".into();
    a.refresh_dirty();
    assert!(a.dirty);
    assert_eq!(a.save_state(), "Unsaved");
    // Putting it back reads as clean again: the check is on content.
    a.extended_data.clear();
    a.refresh_dirty();
    assert!(!a.dirty);
    // Interface state saved with the file is not the user's work.
    a.dock.open = !a.dock.open;
    a.dock.pinned = !a.dock.pinned;
    a.refresh_dirty();
    assert!(!a.dirty, "toggling the dock is not an edit");
    // Saving makes the new content the baseline.
    a.extended_data = "kept".into();
    let dir = std::env::temp_dir().join(format!("sketch-dirty-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Dirty.skx");
    let _ = file_dialogs::with_save_path(Some(file.clone()), || a.update(Message::SaveFile));
    assert!(file.exists());
    assert!(!a.dirty);
    assert_eq!(a.save_state(), "Saved");
    a.extended_data = "changed again".into();
    a.refresh_dirty();
    assert_eq!(a.save_state(), "Unsaved");
    // File > New starts clean.
    a.reset_to_blank();
    assert_eq!(a.save_state(), "New document");
    let _ = std::fs::remove_dir_all(dir);
}
#[test]
fn close_asks_only_when_there_is_unsaved_work() {
    let mut clean = app();
    assert!(exits(clean.update(Message::Close)), "nothing to lose");
    assert!(clean.dialog.is_none());

    let mut a = app();
    a.extended_data = "unsaved".into();
    assert!(!exits(a.update(Message::Close)));
    let d = a.dialog.clone().expect("the save prompt opens");
    assert_eq!(d.on_ok, dialog::Action::SaveAndExit);
    // Cancel keeps the sketch and the app.
    assert!(!exits(a.update(Message::Dialog(dialog::DialogMsg::Cancel))));
    assert!(a.dialog.is_none());
    assert_eq!(a.extended_data, "unsaved");
    // Save with the picker cancelled: nothing was written, so stay open.
    let _ = a.update(Message::Close);
    let (task, _) = file_dialogs::with_save_path(None, || {
        a.update(Message::Dialog(dialog::DialogMsg::Ok))
    });
    assert!(!exits(task), "a cancelled save must not close the app");
    // Don't save closes.
    let _ = a.update(Message::Close);
    assert!(exits(a.update(Message::Dialog(dialog::DialogMsg::Discard))));
}
#[test]
fn a_dropped_sketch_opens_and_other_files_are_refused() {
    let dir = std::env::temp_dir().join(format!("sketch-drop-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Dropped.skx");
    let mut source = app();
    source.extended_data = "from the dropped file".into();
    let _ = file_dialogs::with_save_path(Some(file.clone()), || source.update(Message::SaveFile));

    let mut a = app();
    let _ = a.update(Message::FileDropped(dir.join("photo.png")));
    assert!(a.current_file.is_none(), "not a sketch file");
    let _ = a.update(Message::FileDropped(file.clone()));
    assert_eq!(a.extended_data, "from the dropped file");
    assert_eq!(a.save_state(), "Saved");
    // With unsaved work the drop asks before replacing it.
    a.extended_data = "edited".into();
    let _ = a.update(Message::FileDropped(file.clone()));
    assert_eq!(a.extended_data, "edited", "nothing replaced yet");
    assert_eq!(
        a.dialog.as_ref().map(|d| d.on_ok.clone()),
        Some(dialog::Action::OpenDropped(file.clone()))
    );
    let _ = a.update(Message::Dialog(dialog::DialogMsg::Ok));
    assert_eq!(a.extended_data, "from the dropped file");
    let _ = std::fs::remove_dir_all(dir);
}
#[test]
fn the_window_placement_keeps_the_restored_bounds_while_maximized() {
    persist::with_temp_store(|| {
        let mut a = app();
        let _ = a.update(Message::WindowMoved(Point::new(120., 80.)));
        let _ = a.update(Message::WindowResized(Size::new(1000., 700.)));
        let _ = a.update(Message::Maximized(false));
        // Maximizing reports the monitor's bounds; they are not the
        // restored ones.
        let _ = a.update(Message::WindowMoved(Point::new(0., 0.)));
        let _ = a.update(Message::WindowResized(Size::new(1920., 1040.)));
        let _ = a.update(Message::Maximized(true));
        // A minimize is ignored outright.
        let _ = a.update(Message::WindowMoved(Point::new(-32000., -32000.)));
        let _ = a.update(Message::WindowResized(Size::new(0., 0.)));
        assert_eq!(
            a.placement,
            Some(window_state::Placement {
                pos: Some((120., 80.)),
                width: 1000.,
                height: 700.,
                maximized: true,
            })
        );
        a.store_placement();
        let reopened = app();
        assert!(reopened.maximized);
        assert_eq!(reopened.window_size, Size::new(1000., 700.));
    })
}
#[test]
fn a_pinch_zooms_about_its_centre_and_pans_with_it() {
    let mut a = app();
    let (zoom, pan) = (a.view.zoom, a.view.pan);
    let _ = a.update(Message::Pinch {
        factor: 2.0,
        x: pan.0 as f32,
        y: pan.1 as f32,
        dx: 10.0,
        dy: -5.0,
    });
    assert!((a.view.zoom - zoom * 2.0).abs() < 1e-9);
    // Zooming about a point 10px right / 5px up of the moved origin.
    assert!((a.view.pan.0 - (pan.0 + 20.0)).abs() < 1e-6);
    assert!((a.view.pan.1 - (pan.1 - 10.0)).abs() < 1e-6);
}
