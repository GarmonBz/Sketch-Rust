//! Headless end-to-end tests for the Sketch Rust UI.

use crate::dialog::DialogMsg;
use crate::e2e_support::{harness_next_at, state, VIEWPORT};
use crate::input::KeyCmd;
use crate::pages_nav::NavMsg;
use crate::popup_menu::PopupMsg;
use crate::statusbar::StatusMsg;
use crate::theme::InterfaceTheme;
use crate::ui::layout::SizeClass;
use crate::ui::menu::MenuSub;
use crate::ui::tokens::Density;
use crate::ui::tray::TrayTool;
use crate::ui::Msg as UiMsg;
use crate::view_state::Mode;
use crate::{shapes::ShapesMsg, symlib, Message};

const SIZES: &[(&str, f32, f32, SizeClass)] = &[
    ("1400x860", 1400.0, 860.0, SizeClass::Wide),
    ("1280x800", 1280.0, 800.0, SizeClass::Wide),
    ("800x1280", 800.0, 1280.0, SizeClass::Medium),
    ("372x800", 372.0, 800.0, SizeClass::Compact),
];

#[test]
fn next_shell_boots_at_all_four_sizes() {
    for &(label, w, h, expected_class) in SIZES {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        assert!(
            harness.shows("Untitled"),
            "top bar doc title missing at {label}"
        );
        assert!(
            harness.shows("Page 1"),
            "top bar Page 1 tab missing at {label}"
        );
        match expected_class {
            SizeClass::Wide => assert!(harness.shows("Search commands")),
            SizeClass::Medium => assert!(harness.shows("Totals: 0 sf")),
            SizeClass::Compact => assert!(harness.shows("Totals")),
        }
        let shot = format!("p4-topbar-{label}");
        let path = harness.screenshot_in("e2e-next", &shot);
        assert!(path.exists(), "missing screenshot {}", path.display());

        let app = state(harness);
        assert_eq!(app.window_size, iced::Size::new(w, h));
        assert_eq!(SizeClass::of(app.window_size.width), expected_class);
        assert_eq!(app.window_title(), "Sketch Rust - Untitled");
    }
}

#[test]
fn p1_gallery_renders_all_three_themes_and_both_densities() {
    for &(theme, shot) in &[
        (InterfaceTheme::Dark, "gallery-dark"),
        (InterfaceTheme::Blue, "gallery-blue"),
        (InterfaceTheme::Light, "gallery-light"),
    ] {
        let mut harness = harness_next_at(VIEWPORT);
        harness.send(Message::PopupMenu(PopupMsg::Interface(theme)));
        harness.send(Message::Ui(UiMsg::SetDensity(Density::Touch)));
        harness.send(Message::Ui(UiMsg::SetDensity(Density::Comfortable)));
        harness.send(Message::Ui(UiMsg::SetGallery(true)));
        assert!(harness.shows("Sketch Rust UI Kit Gallery"));
        assert!(harness.shows("COMFORTABLE (44 / 36 / 32 / 62×58)"));
        assert!(harness.shows("TOUCH (56 / 48 / 44 / 68×68)"));

        let path = harness.screenshot_in("e2e-next", shot);
        assert!(path.exists(), "missing screenshot {}", path.display());
    }

    let mut harness = harness_next_at(VIEWPORT);
    harness.send(Message::Ui(UiMsg::SetGallery(true)));
    assert!(harness.shows("Sketch Rust UI Kit Gallery"));
    harness.send(Message::Ui(UiMsg::SetGallery(false)));
    assert!(harness.shows("Untitled"));
    assert!(harness.shows("Search commands"));
}

#[test]
fn p2_resizing_switches_size_classes_at_700_and_1100() {
    let mut harness = harness_next_at(iced::Size::new(360.0, 800.0));
    let cases = [
        (360.0, SizeClass::Compact),
        (699.0, SizeClass::Compact),
        (700.0, SizeClass::Medium),
        (1099.0, SizeClass::Medium),
        (1100.0, SizeClass::Wide),
        (1600.0, SizeClass::Wide),
    ];
    for (w, expected_class) in cases {
        harness.send(Message::WindowResized(iced::Size::new(w, 860.0)));
        assert_eq!(SizeClass::of(w), expected_class);
        match expected_class {
            SizeClass::Wide => {
                assert!(harness.shows("Search commands"), "expected wide search at {w}");
            }
            SizeClass::Medium => {
                assert!(harness.shows("Totals: 0 sf"), "expected medium two-row bar at {w}");
                assert!(!harness.shows("Search commands"), "wide search should hide at {w}");
            }
            SizeClass::Compact => {
                assert!(harness.shows("Totals"), "expected compact totals chip at {w}");
                assert!(!harness.shows("Not saved"), "compact hides save chip at {w}");
            }
        }
    }
    let app = state(harness);
    assert_eq!(app.canvas_rect().x, app.ui.density.ledger_w());
}

#[test]
fn p2_slide_overs_and_sheets_open_and_close() {
    // Medium slide-over: open button, Close button, and starting to draw.
    let mut med = harness_next_at(iced::Size::new(800.0, 1280.0));
    assert!(!med.shows("No defined areas yet."));
    med.script(&["click \"Areas\""]);
    assert!(med.shows("No defined areas yet."));
    med.script(&["click \"Close\""]);
    assert!(!med.shows("No defined areas yet."));

    med.script(&["click \"Areas\""]);
    assert!(med.shows("No defined areas yet."));
    med.send(Message::Key(KeyCmd::DrawExterior));
    assert!(!med.shows("No defined areas yet."));

    // Escape closes slide-over in Medium.
    med.script(&["click \"Areas\""]);
    assert!(med.shows("No defined areas yet."));
    med.send(Message::Ui(UiMsg::CloseOverlays));
    assert!(!med.shows("No defined areas yet."));

    // Compact bottom sheet opens via Areas and shrinks canvas height cleanly.
    let mut cmp = harness_next_at(iced::Size::new(372.0, 800.0));
    cmp.script(&["click \"Areas\""]);
    assert!(cmp.shows("No defined areas yet."));
    cmp.script(&["click \"Close\""]);
    assert!(!cmp.shows("No defined areas yet."));
    let app = state(cmp);
    assert!(!app.ui.has_open_overlays());
}

#[test]
fn p2_canvas_draw_pan_and_zoom_work_across_size_classes() {
    for &(_, w, h, _) in SIZES {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        harness.send(Message::Key(KeyCmd::DrawExterior));
        harness.send(Message::Key(KeyCmd::Enter));
        harness.send(Message::Key(KeyCmd::Arrow {
            dir: 2,
            ctrl: false,
            shift: false,
        }));
        harness.send(Message::Key(KeyCmd::Buffer("10")));
        harness.send(Message::Key(KeyCmd::Enter));
        harness.send(Message::Status(StatusMsg::ZoomIn));
        harness.send(Message::Pinch {
            factor: 1.0,
            x: 100.0,
            y: 100.0,
            dx: 24.0,
            dy: -12.0,
        });
        let app = state(harness);
        assert_eq!(app.drawing.lines.len(), 1, "expected 1 wall at {w}x{h}");
        assert!(app.view.zoom > 1.0, "expected zoom-in at {w}x{h}");
        assert!(app.view.pan.0 != 0.0 || app.view.pan.1 != 0.0, "expected pan at {w}x{h}");
        assert_eq!(app.view.mode, Mode::DrawExterior);
    }
}

#[test]
fn p2_first_touch_switches_density_to_touch() {
    let mut harness = harness_next_at(VIEWPORT);
    harness.send(Message::Ui(UiMsg::TouchDetected));
    let app = state(harness);
    assert_eq!(app.ui.density, Density::Touch);
    assert!(app.ui.touch_seen);
}

#[test]
fn p3_menu_opens_navigates_and_runs_commands_across_sizes() {
    for &(label, w, h, class) in SIZES {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        if matches!(class, SizeClass::Wide | SizeClass::Medium) {
            harness.script(&["click \"Menu\""]);
        } else {
            harness.send(Message::Ui(UiMsg::ToggleMenu));
        }
        assert!(harness.shows("Property details"), "Menu rows missing at {label}");
        assert!(harness.shows("Transform sketch"), "Transform row missing at {label}");
        // Drawing controls have their own home; the Menu no longer repeats them.
        assert!(!harness.shows("Show grid"), "Menu still lists Show grid at {label}");

        harness.send(Message::Ui(UiMsg::MenuSub(Some(MenuSub::Transform))));
        assert!(harness.shows("Flip horizontally"), "Transform fly-out missing at {label}");

        // Search reaches every command, including ones that left the Menu.
        harness.send(Message::Ui(UiMsg::MenuQuery("Metric".into())));
        assert!(
            harness.shows("Units: Metric (m / m²)"),
            "search result missing at {label}"
        );
        harness.script(&["click \"Units: Metric (m / m²)\""]);
        assert!(!harness.shows("Property details"), "Menu should close after running at {label}");
        let app = state(harness);
        assert!(app.drawing.options.units.metric);
    }

    // Keyboard: Down twice reaches Recent, Right opens it, Left closes it,
    // Enter on a fly-out row runs it, Escape closes the Menu.
    let mut kbd = harness_next_at(VIEWPORT);
    kbd.send(Message::Ui(UiMsg::ToggleMenu));
    kbd.send(Message::Ui(UiMsg::MenuMove(1)));
    kbd.send(Message::Ui(UiMsg::MenuMove(1)));
    kbd.send(Message::Ui(UiMsg::MenuSwitchGroup(1)));
    assert!(kbd.shows("Show start screen"), "Right should open Recent");
    kbd.send(Message::Ui(UiMsg::MenuSwitchGroup(-1)));
    assert!(!kbd.shows("Show start screen"), "Left should close Recent");
    kbd.script(&["type escape"]);
    assert!(!kbd.shows("Property details"));

    // Visual captures: Wide with Export open, Wide searching, Compact.
    let mut wide_shot = harness_next_at(iced::Size::new(1400.0, 860.0));
    wide_shot.script(&["click \"Menu\""]);
    wide_shot.send(Message::Ui(UiMsg::MenuSub(Some(MenuSub::Export))));
    assert!(wide_shot.shows("DXF for CAD"));
    let p_wide = wide_shot.screenshot_in("e2e-next", "p3-menu-wide");
    assert!(p_wide.exists());

    let mut search_shot = harness_next_at(iced::Size::new(1400.0, 860.0));
    search_shot.send(Message::Ui(UiMsg::ToggleMenu));
    search_shot.send(Message::Ui(UiMsg::MenuQuery("flip".into())));
    let p_search = search_shot.screenshot_in("e2e-next", "p3-menu-search-wide");
    assert!(p_search.exists());

    let mut cmp_shot = harness_next_at(iced::Size::new(372.0, 800.0));
    cmp_shot.send(Message::Ui(UiMsg::ToggleMenu));
    let p_cmp = cmp_shot.screenshot_in("e2e-next", "p3-menu-compact");
    assert!(p_cmp.exists());
}

/// Ctrl K opens the Menu with its search field; Enter runs the highlighted
/// match, and a second Ctrl K closes it. (The separate command palette was
/// retired on 2026-11-17.)
#[test]
fn p3_ctrl_k_opens_menu_search_and_runs_commands() {
    let mut harness = harness_next_at(VIEWPORT);
    harness.send(Message::Key(KeyCmd::CommandPalette));
    assert!(harness.shows("Search all commands"), "Ctrl K should open the Menu");
    harness.send(Message::Ui(UiMsg::MenuQuery("Metric".into())));
    assert!(harness.shows("Units: Metric (m / m²)"));
    harness.send(Message::Ui(UiMsg::MenuEnter));
    assert!(!harness.shows("Units: Metric (m / m²)"));

    harness.send(Message::Key(KeyCmd::CommandPalette));
    harness.send(Message::Ui(UiMsg::MenuQuery("Show grid".into())));
    assert!(harness.shows("Show grid"));
    harness.send(Message::Ui(UiMsg::MenuEnter));

    harness.send(Message::Key(KeyCmd::CommandPalette));
    harness.send(Message::Ui(UiMsg::MenuQuery("totals".into())));
    harness.send(Message::Ui(UiMsg::MenuEnter));

    // A second Ctrl K closes it again.
    harness.send(Message::Key(KeyCmd::CommandPalette));
    harness.send(Message::Key(KeyCmd::CommandPalette));
    assert!(!harness.shows("Search all commands"));

    let app = state(harness);
    assert!(app.drawing.options.units.metric);
    assert!(!app.view.grid);
    assert!(app.ui.ledger_open);
}

#[test]
fn p4_pages_rename_search_undo_redo_and_export_work_at_all_sizes() {
    for &(label, w, h, class) in SIZES {
        let mut harness = harness_next_at(iced::Size::new(w, h));

        // 1. Add pages and switch between them.
        harness.send(Message::Status(StatusMsg::PageAdd));
        // The page switcher shows the current page; pressing it opens the list.
        assert!(harness.shows("Page 2"), "page switcher missing Page 2 at {label}");
        assert!(harness.shows("2 / 2"), "page count missing at {label}");
        harness.script(&["click \"Page 2\""]);
        assert!(harness.shows("PAGES"), "switcher did not open the page list at {label}");
        harness.send(Message::PagesNav(NavMsg::Go(0)));
        harness.send(Message::PagesNav(NavMsg::Close));
        assert!(harness.shows("1 / 2"), "Go(0) did not change page at {label}");
        {
            let mut h2 = harness_next_at(iced::Size::new(w, h));
            h2.send(Message::Status(StatusMsg::PageAdd));
            h2.send(Message::PagesNav(NavMsg::Go(0)));
            let app = state(h2);
            assert_eq!(app.pages.len(), 2);
            assert_eq!(app.pages.current, 0);
        }

        // 2. Rename document via clicking the doc title.
        harness.script(&["click \"Untitled\""]);
        assert!(harness.shows("Document name:"));
        harness.send(Message::Dialog(DialogMsg::Input("Oak Street".into())));
        harness.send(Message::Dialog(DialogMsg::Ok));
        assert!(harness.shows("Oak Street"), "renamed doc title missing at {label}");

        // 3. Rename current page via tab menu.
        harness.send(Message::Ui(UiMsg::TogglePageMenu(Some(0))));
        assert!(harness.shows("Rename page"));
        assert!(harness.shows("All pages"));
        harness.script(&["click \"Rename page\""]);
        assert!(harness.shows("Page name:"));
        harness.send(Message::Dialog(DialogMsg::Input("Main Level".into())));
        harness.send(Message::Dialog(DialogMsg::Ok));
        assert!(harness.shows("Main Level"), "renamed page tab missing at {label}");

        // 4. Underlay toggle & move first/last.
        harness.send(Message::Ui(UiMsg::TogglePageMenu(Some(1))));
        harness.script(&["click \"Use as underlay\""]);
        harness.send(Message::Ui(UiMsg::TogglePageMenu(Some(1))));
        harness.script(&["click \"Move first\""]);
        harness.send(Message::PagesNav(NavMsg::Toggle));
        assert!(harness.shows("PAGES"), "All pages drawer missing at {label}");
        harness.send(Message::PagesNav(NavMsg::Close));
        assert!(!harness.shows("PAGES"));

        // 5. Delete page confirmation flow.
        harness.send(Message::Ui(UiMsg::TogglePageMenu(Some(1))));
        harness.script(&["click \"Delete page\""]);
        assert!(harness.shows("Delete page 2?"));
        harness.send(Message::Dialog(DialogMsg::Ok));

        // 6. Draw one wall, inspect Undo History, Undo, and Redo.
        harness.send(Message::Key(KeyCmd::DrawExterior));
        harness.send(Message::Key(KeyCmd::Enter));
        harness.send(Message::Key(KeyCmd::Arrow {
            dir: 2,
            ctrl: false,
            shift: false,
        }));
        harness.send(Message::Key(KeyCmd::Buffer("12")));
        harness.send(Message::Key(KeyCmd::Enter));
        harness.send(Message::UndoHistory);
        assert!(harness.shows("1. Draw Line"), "Undo history popup missing at {label}");
        harness.send(Message::Ui(UiMsg::CloseOverlays));
        harness.send(Message::Undo);
        harness.send(Message::Redo);

        // 7. The top bar's search opens the Menu with its search field.
        if matches!(class, SizeClass::Wide) {
            harness.script(&["click \"Search commands\""]);
        } else {
            harness.send(Message::Key(KeyCmd::CommandPalette));
        }
        assert!(
            harness.shows("Search all commands"),
            "Menu search missing at {label}"
        );
        harness.send(Message::Ui(UiMsg::CloseMenu));

        // 8. Export menu / Send to report flow.
        harness.send(Message::Ui(UiMsg::ToggleExportMenu));
        assert!(harness.shows("Export PNG"), "Export PNG missing at {label}");
        harness.send(Message::Ui(UiMsg::CloseOverlays));

        let app = state(harness);
        assert_eq!(app.file_name, "Oak Street");
        assert_eq!(app.pages.len(), 1);
        assert_eq!(app.drawing.lines.len(), 1);
        assert_eq!(app.window_title(), "Sketch Rust - Oak Street");
    }

    // Visual captures of tab context menu and anchored Undo history popup on Wide.
    let mut tab_shot = harness_next_at(VIEWPORT);
    tab_shot.send(Message::Status(StatusMsg::PageAdd));
    tab_shot.send(Message::Ui(UiMsg::TogglePageMenu(Some(1))));
    let p_tab = tab_shot.screenshot_in("e2e-next", "p4-tab-menu-wide");
    assert!(p_tab.exists());

    let mut hist_shot = harness_next_at(VIEWPORT);
    hist_shot.send(Message::Key(KeyCmd::DrawExterior));
    hist_shot.send(Message::Key(KeyCmd::Enter));
    hist_shot.send(Message::Key(KeyCmd::Arrow {
        dir: 2,
        ctrl: false,
        shift: false,
    }));
    hist_shot.send(Message::Key(KeyCmd::Buffer("12")));
    hist_shot.send(Message::Key(KeyCmd::Enter));
    hist_shot.send(Message::UndoHistory);
    let p_hist = hist_shot.screenshot_in("e2e-next", "p4-undo-history-wide");
    assert!(p_hist.exists());
}

#[test]
fn p5_tray_and_flyouts_enter_every_mode_and_follow_shortcuts_at_four_sizes() {
    let assert_tool = |steps: &[&str], extra: Vec<Message>, expected_mode: Mode, tool: TrayTool| {
        let mut h = harness_next_at(VIEWPORT);
        for step in steps {
            if let Err(failed) = h.step(step) {
                let shot = h.screenshot_in("e2e-next", &format!("failed-tray-{tool:?}"));
                panic!("instruction failed: {failed}; screenshot: {}", shot.display());
            }
            h.wait(15);
        }
        for msg in extra {
            h.send(msg);
        }
        let app = state(h);
        assert_eq!(
            app.view.mode, expected_mode,
            "expected {expected_mode:?} after {steps:?}"
        );
        assert!(
            tool.is_active(&app),
            "expected tray tool {tool:?} to highlight for {expected_mode:?}"
        );
    };

    // Every one of the 15 view_state::Mode values entered via the tray or fly-outs.
    assert_tool(&["click \"Select\""], vec![], Mode::Select, TrayTool::Select);
    assert_tool(
        &[],
        vec![
            Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Select))),
        ],
        Mode::Select,
        TrayTool::Select,
    );
    assert_tool(
        &["click \"Wall\""],
        vec![],
        Mode::DrawExterior,
        TrayTool::Wall,
    );
    assert_tool(
        &["click \"Inner\""],
        vec![],
        Mode::DrawInterior,
        TrayTool::Inner,
    );
    assert_tool(&["click \"Area\""], vec![], Mode::DefineArea, TrayTool::Area);
    assert_tool(&["click \"Curve\""], vec![], Mode::Curve, TrayTool::Curve);
    assert_tool(&["click \"Freeform\""], vec![], Mode::Freeform, TrayTool::Freeform);
    assert_tool(&["click \"Erase\""], vec![], Mode::Erase, TrayTool::Erase);
    assert_tool(&["click \"Measure\""], vec![], Mode::Measure, TrayTool::Measure);
    assert_tool(&["click \"Pan\""], vec![], Mode::Pan, TrayTool::Pan);
    assert_tool(
        &["click \"Shape\""],
        vec![Message::RoomShapePlace],
        Mode::PlaceShape,
        TrayTool::Shape,
    );
    assert_tool(
        &["click \"Label\"", "click \"Bed\""],
        vec![],
        Mode::PlaceText,
        TrayTool::Label,
    );

    if let Ok(ix) = symlib::library().as_ref() {
        if let Some(first) = ix.favorites.first() {
            let mut h = harness_next_at(VIEWPORT);
            h.script(&["click \"Insert\""]);
            assert!(h.shows("Symbols") && h.shows("Room labels"));
            h.script(&[&format!("click {:?}", first.label)]);
            let app = state(h);
            assert_eq!(app.view.mode, Mode::PlaceSymbol);
            assert!(TrayTool::Insert.is_active(&app));
        }
    }

    // Fly-out sub-modes & shape prompts.
    for &(label, w, h, _) in SIZES {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        for tool in TrayTool::ALL {
            assert!(
                harness.shows(tool.label()),
                "tray button {} missing at {label}",
                tool.label()
            );
        }

        harness.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Select))));
        harness.script(&["click \"Fence select\""]);

        harness.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Select))));
        harness.script(&["click \"Lasso select\""]);

        harness.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Curve))));
        harness.script(&["click \"Insert vertex\""]);

        harness.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Measure))));
        harness.script(&["click \"Chain dimension\""]);

        harness.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Pan))));
        harness.script(&["click \"Zoom to rectangle\""]);

        harness.script(&["click \"Wall\""]);
        // Clicking the active Wall button opens its fly-out too.
        harness.script(&["click \"Wall\""]);
        assert!(harness.shows("Rectangle"), "Wall flyout missing at {label}");
        harness.script(&["click \"Rectangle\""]);
        assert!(harness.shows("Width in feet"), "Shape prompt missing at {label}");
        harness.send(Message::Dialog(DialogMsg::Cancel));
        harness.send(Message::Shapes(ShapesMsg::Dismiss));

        harness.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Inner))));
        assert!(harness.shows("Circle"), "Inner flyout missing at {label}");
        harness.script(&["click \"Line\""]);

        // Shortcuts switch modes and tray highlights follow.
        harness.send(Message::Key(KeyCmd::DrawExterior));
        harness.send(Message::Key(KeyCmd::InsertVertex));
        let app = state(harness);
        assert_eq!(app.view.mode, Mode::InsertVertex, "shortcut failed at {label}");
        assert!(TrayTool::Curve.is_active(&app));
    }

    // Verify all fly-out sub-modes explicitly on App::view.mode & TrayTool::is_active.
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Select))));
        h.script(&["click \"Fence select\""]);
        let app = state(h);
        assert_eq!(app.view.mode, Mode::Fence);
        assert!(TrayTool::Select.is_active(&app));
    }
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Select))));
        h.script(&["click \"Lasso select\""]);
        let app = state(h);
        assert_eq!(app.view.mode, Mode::Lasso);
        assert!(TrayTool::Select.is_active(&app));
    }
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Curve))));
        h.script(&["click \"Insert vertex\""]);
        let app = state(h);
        assert_eq!(app.view.mode, Mode::InsertVertex);
        assert!(TrayTool::Curve.is_active(&app));
    }
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Measure))));
        h.script(&["click \"Chain dimension\""]);
        let app = state(h);
        assert_eq!(app.view.mode, Mode::ChainDim);
        assert!(TrayTool::Measure.is_active(&app));
    }
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Pan))));
        h.script(&["click \"Zoom to rectangle\""]);
        let app = state(h);
        assert_eq!(app.view.mode, Mode::ZoomFence);
        assert!(TrayTool::Pan.is_active(&app));
    }

    // Visual captures of Wide tray, Wall fly-out, and Compact scrollable bar.
    let mut wide_shot = harness_next_at(VIEWPORT);
    wide_shot.script(&["click \"Wall\""]);
    let p1 = wide_shot.screenshot_in("e2e-next", "p5-tray-wide");
    assert!(p1.exists());

    let mut fly_shot = harness_next_at(VIEWPORT);
    fly_shot.script(&["click \"Wall\""]);
    fly_shot.script(&["click \"Wall\""]);
    let p2 = fly_shot.screenshot_in("e2e-next", "p5-tray-flyout-wide");
    assert!(p2.exists());

    let mut cmp_shot = harness_next_at(iced::Size::new(372.0, 800.0));
    cmp_shot.script(&["click \"Area\""]);
    let p3 = cmp_shot.screenshot_in("e2e-next", "p5-tray-compact");
    assert!(p3.exists());
}

fn draw_closed_square(
    harness: &mut crate::e2e_support::Harness<
        impl iced_test::program::Program<State = crate::App, Message = Message, Theme = iced::Theme>
            + 'static,
    >,
) {
    harness.send(Message::Key(KeyCmd::DrawExterior));
    harness.send(Message::Key(KeyCmd::Enter));
    for dir in [2, 4, 6, 0] {
        harness.send(Message::Key(KeyCmd::Buffer("1")));
        harness.send(Message::Key(KeyCmd::Buffer("0")));
        harness.send(Message::Key(KeyCmd::Arrow {
            dir,
            ctrl: false,
            shift: false,
        }));
        harness.send(Message::Key(KeyCmd::Enter));
    }
}

fn drawn_square_center_on_screen(w: f32, h: f32) -> (f32, f32) {
    let mut tmp = harness_next_at(iced::Size::new(w, h));
    draw_closed_square(&mut tmp);
    let app = state(tmp);
    let poly = &app.drawing.polygons[0];
    let cx = poly.points.iter().map(|p| p.x).sum::<f64>() / poly.points.len() as f64;
    let cy = poly.points.iter().map(|p| p.y).sum::<f64>() / poly.points.len() as f64;
    let (sx, sy) = app
        .view
        .to_screen(crate::view_state::World { x: cx, y: cy });
    let r = app.canvas_rect();
    (r.x + sx as f32, r.y + sy as f32)
}

/// Closing the first area on a page fits the sketch into the drawing, so the
/// outline sits in the middle of the view instead of a corner of it.
#[test]
fn first_closed_area_is_centred_in_the_view_across_sizes() {
    for &(label, w, h, _) in SIZES {
        let mut h_app = harness_next_at(iced::Size::new(w, h));
        draw_closed_square(&mut h_app);
        let app = state(h_app);
        let poly = &app.drawing.polygons[0];
        let n = poly.points.len() as f64;
        let cx = poly.points.iter().map(|p| p.x).sum::<f64>() / n;
        let cy = poly.points.iter().map(|p| p.y).sum::<f64>() / n;
        let (sx, sy) = app
            .view
            .to_screen(crate::view_state::World { x: cx, y: cy });
        let r = app.canvas_rect();
        let (mx, my) = (f64::from(r.width) / 2.0, f64::from(r.height) / 2.0);
        assert!(
            (sx - mx).abs() < f64::from(r.width) * 0.1 && (sy - my).abs() < f64::from(r.height) * 0.1,
            "the first closed area is centred at {label}: centre ({sx:.0}, {sy:.0}) in {:.0} x {:.0}",
            r.width,
            r.height
        );
    }
}

#[test]
fn canvas_context_menu_closes_on_an_outside_press_and_on_escape() {
    let r = {
        let mut peek = harness_next_at(VIEWPORT);
        draw_closed_square(&mut peek);
        state(peek).canvas_rect()
    };
    let at = iced::Point::new(r.x + 100.0, r.y + 100.0);

    let mut h = harness_next_at(VIEWPORT);
    draw_closed_square(&mut h);
    h.send(Message::RightPress(at));
    // A press on empty canvas, away from the menu, dismisses it.
    let (px, py) = (r.x + r.width - 40.0, r.y + r.height - 40.0);
    h.script(&[&format!("press ({px}, {py})"), &format!("release ({px}, {py})")]);
    assert!(state(h).context.open.is_none(), "an outside press must close the menu");

    let mut h = harness_next_at(VIEWPORT);
    draw_closed_square(&mut h);
    h.send(Message::RightPress(at));
    h.send(Message::ContextClose);
    assert!(state(h).context.open.is_none());
}

/// A closed square plus one new folder named "Layer": the combined tree's
/// starting point for the tests below.
macro_rules! tree_harness {
    ($h:ident) => {
        let mut $h = harness_next_at(VIEWPORT);
        draw_closed_square(&mut $h);
        $h.send(Message::LayerNewFolder);
        $h.send(Message::LayerRenameCommit);
    };
}

/// (the area's layer id, the folder's id, the polygon id) of the tree above.
fn tree_ids() -> (String, String, i64) {
    tree_harness!(p);
    let app = state(p);
    let area = app
        .layers
        .items
        .iter()
        .find(|l| l.area_poly_id.is_some())
        .expect("the area has a layer");
    let folder = app.layers.items.iter().find(|l| l.is_group).expect("the new folder");
    (area.id.clone(), folder.id.clone(), app.drawing.polygons[0].id)
}

#[test]
fn areas_and_layers_show_as_one_tree_with_a_folder_button() {
    tree_harness!(h);
    // The area row: its name, and its type and size on a second line.
    assert!(h.shows("Undefined Area"));
    assert!(h.shows("Needs a type · 100.0 sf"));
    // The folder row, and the plain layer that holds loose work.
    assert!(h.shows("Layer"));
    assert!(h.shows("Layer 1"));
    // There is no second Layers list or tab any more.
    assert!(!h.shows("LAYERS (1)"));
    let app = state(h);
    let folder = app.layers.items.iter().find(|l| l.is_group).unwrap();
    assert_eq!(folder.name, "Layer");
    assert!(folder.parent_id.is_none());
}

#[test]
fn clicking_an_area_row_selects_the_area_and_leaves_the_active_layer_alone() {
    let (_, _, poly) = tree_ids();
    tree_harness!(h);
    let before = {
        let mut p = harness_next_at(VIEWPORT);
        draw_closed_square(&mut p);
        state(p).layers.active.clone()
    };
    h.script(&["click \"Needs a type · 100.0 sf\""]);
    assert!(h.shows("Base area"), "the click opens the area's numbers");
    let app = state(h);
    assert!(app.selection.polys.contains(&poly));
    assert_eq!(app.layers.active, before);
}

#[test]
fn a_folders_eye_hides_the_areas_inside_it() {
    let (area, folder, poly) = tree_ids();
    tree_harness!(h);
    h.send(Message::LayerMoveTo(area.clone(), Some(folder.clone())));
    h.send(Message::LayerToggleVisible(folder.clone()));
    let app = state(h);
    assert_eq!(app.layers.layer(&area).unwrap().parent_id.as_deref(), Some(folder.as_str()));
    assert!(app.layer_gates().hidden.contains(&poly), "the area is hidden with its folder");
}

#[test]
fn move_to_layer_reparents_and_top_level_brings_it_back() {
    let (area, folder, _) = tree_ids();
    tree_harness!(h);
    h.send(Message::LayerRowMenu(area.clone()));
    assert!(h.shows("Move to layer"));
    h.send(Message::LayerMoveTo(area.clone(), Some(folder.clone())));
    h.send(Message::LayerRowMenu(area.clone()));
    assert!(h.shows("Top level"));
    h.send(Message::LayerMoveTo(area.clone(), None));
    let app = state(h);
    assert!(app.layers.layer(&area).unwrap().parent_id.is_none());
    // The row menu never made the area layer the active one.
    assert_ne!(app.layers.active, area);
}

#[test]
fn dragging_a_row_reorders_it_or_drops_it_into_a_folder() {
    use crate::layers::DragPos;
    let (area, folder, _) = tree_ids();
    tree_harness!(h);
    // Into the folder.
    h.send(Message::LayerPress(area.clone()));
    h.send(Message::LayerZoneHover(folder.clone(), DragPos::Inside));
    h.send(Message::LayerDrop(folder.clone(), DragPos::Inside));
    // Then above the first plain layer, which also takes it out again.
    h.send(Message::LayerPress(area.clone()));
    h.send(Message::LayerZoneHover("layer-1".into(), DragPos::Before));
    h.send(Message::LayerDrop("layer-1".into(), DragPos::Before));
    let app = state(h);
    assert!(app.layers.layer(&area).unwrap().parent_id.is_none());
    let order: Vec<&str> = app
        .layers
        .flatten_layer_order()
        .iter()
        .map(|l| l.id.as_str())
        .collect();
    let pos = |id: &str| order.iter().position(|x| *x == id).unwrap();
    assert!(pos(&area) < pos("layer-1"), "{order:?}");
}

#[test]
fn dragging_a_rows_grip_over_another_row_moves_it_there() {
    let (area, folder, _) = tree_ids();
    tree_harness!(h);
    // Rows now read: Layer 1, the folder, the area inside it.
    h.send(Message::LayerMoveTo(area.clone(), Some(folder.clone())));
    // The grip of the Layer 1 row (left edge, first row), released over the
    // middle of the area row: Layer 1 lands after it, in the same folder.
    h.script(&["press (21, 147)", "move (120, 212)", "release (120, 212)"]);
    let app = state(h);
    assert_eq!(
        app.layers.layer("layer-1").unwrap().parent_id.as_deref(),
        Some(folder.as_str()),
        "dropping on a row inside a folder joins that folder"
    );
    let order: Vec<&str> = app
        .layers
        .flatten_layer_order()
        .iter()
        .map(|l| l.id.as_str())
        .collect();
    let pos = |id: &str| order.iter().position(|x| *x == id).unwrap();
    assert!(pos(&area) < pos("layer-1"), "{order:?}");
}

/// Two "Kitchen" labels placed on an empty sketch, and their positions on
/// the canvas.
macro_rules! two_labels {
    ($h:ident) => {
        let mut $h = harness_next_at(VIEWPORT);
        for x in [500.0_f32, 760.0] {
            $h.send(Message::Dock(crate::dock::DockMsg::TextPick("Kitchen".into())));
            $h.script(&[&format!("click ({x:.1}, 430.0)")]);
            $h.wait(450);
        }
    };
}

fn two_label_spots() -> [(f32, f32); 2] {
    two_labels!(h);
    let app = state(h);
    assert_eq!(app.drawing.texts.len(), 2);
    let spot = |i: usize| {
        let t = &app.drawing.texts[i];
        let (x, y) = app.view.to_screen(crate::view_state::World { x: t.x, y: t.y });
        (x as f32, y as f32)
    };
    [spot(0), spot(1)]
}

#[test]
fn ctrl_click_adds_and_removes_labels_from_the_selection() {
    let [a, b] = two_label_spots();
    two_labels!(h);
    macro_rules! click {
        ($h:ident, $at:expr, $ctrl:expr) => {
            $h.send(Message::CanvasClick { x: $at.0, y: $at.1, double: false, ctrl: $ctrl });
        };
    }
    click!(h, a, false);
    click!(h, b, true);
    let both = state(h).selection.texts.len();
    assert_eq!(both, 2, "Ctrl+click adds the second label");

    two_labels!(h);
    click!(h, a, false);
    click!(h, b, true);
    click!(h, b, true);
    let app = state(h);
    assert_eq!(app.selection.texts.len(), 1, "Ctrl+click on a selected label removes it");

    two_labels!(h);
    click!(h, a, false);
    click!(h, b, false);
    assert_eq!(state(h).selection.texts.len(), 1, "a plain click replaces the selection");
}

#[test]
fn apply_to_all_labels_gives_every_label_the_selected_ones_style() {
    let [a, _] = two_label_spots();
    two_labels!(h);
    h.send(Message::CanvasClick { x: a.0, y: a.1, double: false, ctrl: false });
    h.send(Message::ToggleSelectedTextStyle("bold".into()));
    h.send(Message::AdjustSelectedTextSize(4));
    h.send(Message::SetSelectedTextStyle("color".into(), "#dc2626".into()));
    h.send(Message::ApplyTextStyleToAll);
    let app = state(h);
    assert_eq!(app.drawing.texts.len(), 2);
    let first = app.drawing.texts[0].clone();
    assert!(first.bold && first.font_size > 12.0, "the selected label was styled");
    for t in &app.drawing.texts {
        assert_eq!(t.bold, first.bold);
        assert_eq!(t.font_size, first.font_size);
        assert_eq!(t.color, first.color);
        assert_eq!(t.italic, first.italic);
    }
}

#[test]
fn arrow_keys_nudge_by_the_snap_pitch() {
    let moved = |pitch: Option<(&'static str, f64)>, snap: bool, shift_step: f64| {
        let mut h = harness_next_at(VIEWPORT);
        draw_closed_square(&mut h);
        let before = {
            let mut p = harness_next_at(VIEWPORT);
            draw_closed_square(&mut p);
            state(p).drawing.lines[0].start.x
        };
        if let Some((label, value)) = pitch {
            h.send(Message::Status(StatusMsg::Pitch(crate::statusbar::PitchOption { label, value })));
        }
        if !snap {
            h.send(Message::Status(StatusMsg::ToggleSnap));
        }
        h.send(Message::SelectAll);
        h.send(Message::Key(KeyCmd::Nudge(shift_step, 0.0)));
        state(h).drawing.lines[0].start.x - before
    };
    let inch = 1.0 / 12.0;
    assert!((moved(Some(("1\"", inch)), true, 1.0) - inch).abs() < 1e-9, "one 1-inch pitch");
    assert!((moved(Some(("1/2'", 0.5)), true, 1.0) - 0.5).abs() < 1e-9, "one half-foot pitch");
    assert!((moved(Some(("1\"", inch)), true, -1.0) + inch).abs() < 1e-9, "left");
    assert!((moved(Some(("1\"", inch)), true, 0.1) - 10.0 * inch).abs() < 1e-9, "Shift is ten pitches");
    assert!((moved(None, false, 1.0) - 1.0).abs() < 1e-9, "snap off keeps the 1 ft step");
}

#[test]
fn deleting_a_folder_keeps_the_areas_that_were_in_it() {
    let (area, folder, _) = tree_ids();
    tree_harness!(h);
    h.send(Message::LayerMoveTo(area.clone(), Some(folder.clone())));
    h.send(Message::LayerDelete(folder.clone()));
    let app = state(h);
    assert!(app.layers.layer(&folder).is_none());
    assert!(app.layers.layer(&area).unwrap().parent_id.is_none());
}

#[test]
fn areas_and_layers_tree_screenshots() {
    let (area, folder, _) = tree_ids();
    for (label, density) in [("comfortable", Density::Comfortable), ("touch", Density::Touch)] {
        tree_harness!(h);
        h.send(Message::Ui(UiMsg::SetDensity(density)));
        h.send(Message::LayerMoveTo(area.clone(), Some(folder.clone())));
        h.script(&["click \"Needs a type · 100.0 sf\""]);
        let shot = h.screenshot_in("e2e-next", &format!("areas-layers-tree-{label}"));
        assert!(shot.exists());
    }
}

#[test]
fn p6_ledger_drawn_types_layers_totals_and_resize_across_sizes() {
    for &(label, w, h, class) in SIZES {
        let mut h_app = harness_next_at(iced::Size::new(w, h));
        // Open-chain live row ("Drawing an area") appears while pen is down.
        h_app.send(Message::Key(KeyCmd::DrawExterior));
        h_app.send(Message::Key(KeyCmd::Enter));
        h_app.send(Message::Key(KeyCmd::Buffer("1")));
        h_app.send(Message::Key(KeyCmd::Buffer("0")));
        h_app.send(Message::Key(KeyCmd::Arrow {
            dir: 2,
            ctrl: false,
            shift: false,
        }));
        h_app.send(Message::Key(KeyCmd::Enter));
        if !matches!(class, SizeClass::Wide) {
            h_app.script(&["click \"Areas\""]);
        }
        assert!(
            h_app.shows("Drawing an area"),
            "live open-chain row missing at {label}"
        );
        if !matches!(class, SizeClass::Wide) {
            h_app.script(&["click \"Close\""]);
        }

        // Complete the remaining 3 legs to form a 10x10 UND polygon (100 sf).
        for dir in [4, 6, 0] {
            h_app.send(Message::Key(KeyCmd::Buffer("1")));
            h_app.send(Message::Key(KeyCmd::Buffer("0")));
            h_app.send(Message::Key(KeyCmd::Arrow {
                dir,
                ctrl: false,
                shift: false,
            }));
            h_app.send(Message::Key(KeyCmd::Enter));
        }
        if !matches!(class, SizeClass::Wide) {
            h_app.script(&["click \"Areas\""]);
        }
        assert!(
            h_app.shows("Needs a type · 100.0 sf"),
            "undefined area row missing at {label}"
        );

        // Find undefined areas toggle.
        h_app.script(&["click \"Types\""]);
        assert!(
            h_app.shows("First Floor"),
            "Types view missing GLA1 at {label}"
        );
        if !matches!(class, SizeClass::Compact) {
            h_app.script(&["click \"Find undefined\""]);
            assert!(
                h_app.shows("Undefined outlines highlighted. Classification is optional."),
                "Find undefined banner missing at {label}"
            );
        }

        // Arm First Floor and click inside the outline on the canvas.
        h_app.script(&["click \"First Floor\""]);
        assert!(
            h_app.shows("Click an outline to assign First Floor. Escape stops."),
            "arming First Floor did not update workflow status at {label}"
        );
        if !matches!(class, SizeClass::Wide) {
            // Close Medium slide-over / Compact bottom sheet so the outline click hits canvas.
            h_app.script(&["click \"Close\""]);
        }
        let app_rect = drawn_square_center_on_screen(w, h);
        h_app.script(&[&format!("click ({:.1}, {:.1})", app_rect.0, app_rect.1)]);

        if !matches!(class, SizeClass::Wide) {
            h_app.script(&["click \"Areas\""]);
        }
        assert!(
            h_app.shows("Assigned. Click another outline, or Escape to stop."),
            "canvas click at ({:.1}, {:.1}) did not assign First Floor at {label}",
            app_rect.0,
            app_rect.1
        );
        h_app.script(&["click \"Drawn\""]);
        assert!(
            h_app.shows("GLA1 · 100.0 sf"),
            "typed area row missing after defining GLA1 at {label}"
        );
        assert!(
            h_app.shows("100.0 sf"),
            "Totals GLA 100.0 sf missing at {label}"
        );

        // Expand area row to see the 6 calculation figures.
        h_app.script(&["click \"First Floor\""]);
        assert!(h_app.shows("Base area"), "Base area detail missing at {label}");
        assert!(h_app.shows("Factor"), "Factor detail missing at {label}");
        assert!(h_app.shows("Subtracted"), "Subtracted detail missing at {label}");
        assert!(h_app.shows("Perimeter"), "Perimeter detail missing at {label}");

        // "Show the math" opens breakdown modal.
        h_app.script(&["click \"Show the math\""]);
        assert!(
            h_app.shows("Breakdown Details"),
            "Breakdown modal should open at {label}"
        );
        h_app.send(Message::Breakdown(crate::breakdown_dialog::BreakdownMsg::Close));

        // Deleting a type turns its areas back into Undefined.
        h_app.send(Message::AreaWorkflow(crate::area_workflow::Msg::ConfirmDelete(
            "GLA1".into(),
        )));
        assert!(
            h_app.shows("Needs a type · 100.0 sf"),
            "Deleting GLA1 should revert area to Needs a type at {label}"
        );

        // Layer tree: hide the area layer cleanly.
        let layer_id = {
            let mut tmp = harness_next_at(iced::Size::new(w, h));
            draw_closed_square(&mut tmp);
            state(tmp)
                .layers
                .items
                .iter()
                .find(|l| l.area_poly_id.is_some())
                .map(|l| l.id.clone())
                .expect("area layer should exist")
        };
        h_app.send(Message::LayerToggleVisible(layer_id.clone()));
        let app = state(h_app);
        assert!(
            !app.layers.layer(&layer_id).unwrap().visible,
            "Layer should become hidden at {label}"
        );
    }

    // Wide-only tests: drag type onto outline, overflow "..." menu, and width drag handle.
    {
        let mut h = harness_next_at(VIEWPORT);
        draw_closed_square(&mut h);
        h.script(&["click \"Types\""]);
        // Search filters down to Garage so we can drag it onto the canvas outline.
        h.send(Message::Ui(UiMsg::TypeFilterChanged("GAR".into())));
        let (drop_x, drop_y) = drawn_square_center_on_screen(VIEWPORT.width, VIEWPORT.height);
        h.send(Message::DockDragPress(crate::dock::DragItem::Area {
            code: "GAR".into(),
            name: Some("Garage".into()),
        }));
        h.send(Message::DockDragMove(
            iced::Point::new(120.0, 220.0),
            VIEWPORT.width,
        ));
        h.send(Message::DockDragMove(
            iced::Point::new(drop_x, drop_y),
            VIEWPORT.width,
        ));
        h.send(Message::DockDragRelease);
        h.send(Message::Ui(UiMsg::TypeFilterChanged(String::new())));
        h.script(&["click \"Drawn\""]);
        assert!(h.shows("GAR · 100.0 sf"));
        assert!(h.shows("Garage"));

        // Dragging Wide ledger resize handle updates ledger_width and canvas_rect().
        let handle_x = Density::Comfortable.ledger_w();
        h.script(&[
            &format!("press ({handle_x:.1}, 300)"),
            "move (340, 300)",
            "release (340, 300)",
        ]);

        // Overflow "..." menu opens Expand all / Collapse all / Label defaults.
        h.send(Message::Ui(UiMsg::ToggleLedgerMenu));
        assert!(h.shows("Label defaults & area tools…"));
        h.script(&["click \"Label defaults & area tools…\""]);
        assert!(h.shows("AREA OPERATIONS"));
        h.send(Message::AreaWorkflow(crate::area_workflow::Msg::Advanced(false)));

        let app = state(h);
        assert_eq!(app.ui.ledger_width, Some(340.0));
        assert_eq!(app.canvas_rect().x, 340.0);
        assert_eq!(app.drawing.polygons[0].kind, "GAR");
    }

    // Visual captures of Wide expanded Ledger and Compact sheet.
    {
        let mut h = harness_next_at(VIEWPORT);
        draw_closed_square(&mut h);
        let pt = drawn_square_center_on_screen(VIEWPORT.width, VIEWPORT.height);
        h.script(&["click \"Types\"", "click \"First Floor\""]);
        h.script(&[&format!("click ({:.1}, {:.1})", pt.0, pt.1)]);
        h.script(&["click \"Drawn\"", "click \"First Floor\""]);
        let shot = h.screenshot_in("e2e-next", "p6-ledger-wide");
        assert!(shot.exists());
    }

    {
        let mut h = harness_next_at(iced::Size::new(372.0, 800.0));
        draw_closed_square(&mut h);
        h.script(&["click \"Areas\""]);
        let shot = h.screenshot_in("e2e-next", "p6-ledger-compact");
        assert!(shot.exists());
    }
}

fn drawn_square_wall_mid_on_screen(w: f32, h: f32) -> (f32, f32) {
    let mut tmp = harness_next_at(iced::Size::new(w, h));
    draw_closed_square(&mut tmp);
    let app = state(tmp);
    let line = &app.drawing.lines[0];
    let mx = (line.start.x + line.end.x) * 0.5;
    let my = (line.start.y + line.end.y) * 0.5;
    let (sx, sy) = app
        .view
        .to_screen(crate::view_state::World { x: mx, y: my });
    let r = app.canvas_rect();
    (r.x + sx as f32, r.y + sy as f32)
}

fn drawn_square_chain_mid_on_screen(w: f32, h: f32) -> (f32, f32) {
    let mut tmp = harness_next_at(iced::Size::new(w, h));
    draw_closed_square(&mut tmp);
    let wall_mid = drawn_square_wall_mid_on_screen(w, h);
    tmp.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Measure))));
    tmp.script(&["click \"Chain dimension\""]);
    tmp.script(&[&format!("click ({:.1}, {:.1})", wall_mid.0, wall_mid.1)]);
    let app = state(tmp);
    let chain = &app.drawing.dimension_chains[0];
    let computed = app.drawing.compute_chain(chain).expect("computed chain");
    let run_len = sketch_core::geometry::distance(computed.start, computed.end);
    let tiers = crate::chains::ChainTiers::solve(computed, chain);
    let mid = tiers.point_at(run_len * 0.5, 0.0);
    let (sx, sy) = app
        .view
        .to_screen(crate::view_state::World { x: mid.x, y: mid.y });
    let r = app.canvas_rect();
    (r.x + sx as f32, r.y + sy as f32)
}

#[test]
fn p7_inspector_controls_across_selection_kinds_and_sizes() {
    use crate::area_dialog::AreaDialogMsg;

    // 1. Wall inspector across all 4 responsive sizes.
    for &(label, w, h, class) in SIZES {
        let mut h_app = harness_next_at(iced::Size::new(w, h));
        draw_closed_square(&mut h_app);
        h_app.script(&["click \"Select\""]);
        let (wx, wy) = drawn_square_wall_mid_on_screen(w, h);
        h_app.script(&[&format!("click ({wx:.1}, {wy:.1})")]);

        if !matches!(class, SizeClass::Wide) {
            assert!(
                h_app.shows("Inspect"),
                "Inspect trigger missing at {label} with wall selected"
            );
            h_app.script(&["click \"Inspect\""]);
        }
        assert!(h_app.shows("DIMENSIONS"), "wall section missing at {label}");
        assert!(h_app.shows("APPEARANCE"), "appearance section missing at {label}");
        assert!(h_app.shows("CURVE"), "curve section missing at {label}");
        if matches!(class, SizeClass::Wide) {
            assert!(h_app.shows("Copy"), "wall edit actions missing at {label}");
        }

        h_app.send(Message::LineLengthInput("126\"".into()));
        h_app.script(&["click \"Set length\""]);
        h_app.script(&["click \"Dashed\""]);
        h_app.send(Message::Ui(UiMsg::WallBowStep(1.0)));
        h_app.script(&["click \"Toggle dimension\""]);

        let app = state(h_app);
        let wall = app.drawing.first_selected_line(&app.selection).unwrap();
        assert!(
            (sketch_core::geometry::distance(wall.start, wall.end) - 10.5).abs() < 1e-6,
            "wall length in inches was not applied at {label}"
        );
        assert!(
            app.drawing
                .lines
                .iter()
                .any(|l| l.line_style.as_deref() == Some("dashed")),
            "wall line style not updated at {label}"
        );
        assert!(
            app.drawing.lines.iter().any(|l| l.bow_angle.abs() > 0.0),
            "wall curve bow not updated via stepper at {label}"
        );
        assert!(
            app.drawing.lines.iter().any(|l| l.dimension_hidden),
            "wall dimension toggle failed at {label}"
        );
        if matches!(class, SizeClass::Wide) {
            let expected_cw = w - app.ui.effective_ledger_width() - Density::Comfortable.inspector_w();
            assert!((app.canvas_rect().width - expected_cw).abs() < 1.0);
        }
    }

    // 2. Area inspector (inline AreaDialog fields, switches, clone, deduct-from, walls list).
    {
        let area_vp = iced::Size::new(VIEWPORT.width, 1320.0);
        let mut h = harness_next_at(area_vp);
        draw_closed_square(&mut h);
        h.script(&["click \"Select\""]);
        let (cx, cy) = drawn_square_center_on_screen(area_vp.width, area_vp.height);
        h.script(&[&format!("click ({cx:.1}, {cy:.1})")]);
        h.send(Message::AreaProps);
        assert!(h.shows("Counts toward"));
        assert!(h.shows("CALCULATION") && h.shows("APPEARANCE") && h.shows("EDIT"));

        h.send(Message::AreaDialog(AreaDialogMsg::Name("Sunroom".into())));
        h.script(&["click \"Apply\""]);
        h.script(&["click \"Subtracts (-)\"", "click \"Code\"", "click \"Clone\""]);

        let mut app = state(h);
        assert!(
            app.drawing.polygons.iter().any(|p| p.name == "Sunroom"),
            "area inline name Apply failed"
        );
        assert!(
            app.drawing
                .polygons
                .iter()
                .any(|p| p.factor_override.unwrap_or(p.factor) < 0.0),
            "area sign toggle failed"
        );
        assert_eq!(app.drawing.polygons.len(), 2, "area clone failed");
        app.invalidate_canvas();
    }

    // 2a. Wide area screenshot at standard VIEWPORT.
    {
        let mut h = harness_next_at(VIEWPORT);
        draw_closed_square(&mut h);
        h.script(&["click \"Select\""]);
        let (cx, cy) = drawn_square_center_on_screen(VIEWPORT.width, VIEWPORT.height);
        h.script(&[&format!("click ({cx:.1}, {cy:.1})")]);
        let shot = h.screenshot_in("e2e-next", "p7-inspector-wide-area");
        assert!(shot.exists());
    }

    // 2b. Area Deduct-from + Walls list navigation.
    {
        let area_vp = iced::Size::new(VIEWPORT.width, 1680.0);
        let mut h = harness_next_at(area_vp);
        draw_closed_square(&mut h);
        h.script(&["click \"Select\""]);
        let (cx, cy) = drawn_square_center_on_screen(area_vp.width, area_vp.height);
        h.script(&[&format!("click ({cx:.1}, {cy:.1})")]);
        h.script(&["click \"Clone\""]);
        let poly0_id = {
            let mut h2 = harness_next_at(area_vp);
            draw_closed_square(&mut h2);
            state(h2).drawing.polygons[0].id
        };
        h.send(Message::Ui(UiMsg::SetAreaSubtractTarget(Some(poly0_id))));
        assert!(h.shows("Wall 1 · 10.0'"));
        let app = state(h);
        assert!(app
            .drawing
            .polygons
            .iter()
            .any(|p| p.subtract_target_id == Some(poly0_id)));
    }

    // 3. Text label inspector.
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Dock(crate::dock::DockMsg::TextPick("Kitchen".into())));
        let (cx, cy) = (700.0_f32, 430.0_f32);
        h.script(&[&format!("click ({cx:.1}, {cy:.1})")]);
        // Offset horizontally so the selection click stays inside the label hitbox
        // without coalescing into a double-click.
        h.script(&[&format!("click ({:.1}, {cy:.1})", cx + 12.0)]);
        assert!(h.shows("TEXT LABEL"));
        assert!(h.shows("Copy"));
        h.script(&["click \"Bold\""]);
        h.send(Message::AdjustSelectedTextSize(2));
        let app = state(h);
        assert_eq!(app.selection.texts.len(), 1);
        assert!(app.drawing.texts[0].bold);
        assert!(app.drawing.texts[0].font_size > 12.0);
    }

    // 4. Symbol + hosted wall opening inspector.
    {
        let lib = symlib::library();
        let door_id = lib
            .as_ref()
            .as_ref()
            .expect("symbols loaded")
            .favorites[0]
            .id
            .clone();
        let mut h = harness_next_at(VIEWPORT);
        draw_closed_square(&mut h);
        let (wx, wy) = drawn_square_wall_mid_on_screen(VIEWPORT.width, VIEWPORT.height);
        h.send(Message::Dock(crate::dock::DockMsg::SymbolPick(door_id)));
        h.script(&[&format!("click ({wx:.1}, {wy:.1})")]);
        assert!(h.shows("SYMBOL"));
        assert!(h.shows("WALL OPENING"));
        assert!(h.shows("Copy"));
        h.script(&["click \"Flip swing\""]);
        h.send(Message::OpeningWider);
        h.send(Message::ScaleSelected(1.1));
        let app = state(h);
        let sym = &app.drawing.symbols[0];
        assert!(sym.wall_id.is_some());
        assert!(sym.wall_side_flip);
        assert!(sym.scale > 1.0);
    }

    // 5. Chain dimension inspector.
    {
        let mut h = harness_next_at(VIEWPORT);
        draw_closed_square(&mut h);
        let (wx, wy) = drawn_square_wall_mid_on_screen(VIEWPORT.width, VIEWPORT.height);
        let (chx, chy) = drawn_square_chain_mid_on_screen(VIEWPORT.width, VIEWPORT.height);
        h.send(Message::Ui(UiMsg::ToggleTrayFlyout(Some(TrayTool::Measure))));
        h.script(&["click \"Chain dimension\""]);
        h.script(&[&format!("click ({wx:.1}, {wy:.1})")]);
        h.send(Message::Rail(crate::view_state::RailTool::Select));
        h.script(&[&format!("click ({chx:.1}, {chy:.1})")]);
        assert!(h.shows("CHAIN DIMENSION"));
        h.script(&["click \"Flip side\""]);
        h.script(&["click \"Show overall tier\""]);
        h.send(Message::Ui(UiMsg::ChainOffsetStep(0.5)));
        let app = state(h);
        let c = &app.drawing.dimension_chains[0];
        assert_ne!(c.side, 1);
        assert!(!c.show_overall);
        assert!(c.offset > 2.5);
    }

    // 6. Trace image inspector (menu Adjust / edit mode).
    {
        let mut raw = Vec::new();
        image::RgbaImage::from_pixel(8, 8, image::Rgba([200, 30, 30, 255]))
            .write_to(&mut std::io::Cursor::new(&mut raw), image::ImageFormat::Png)
            .unwrap();
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::PhotoBytesLoaded(raw, 8, 8));
        h.send(Message::Ui(UiMsg::InspectorAdjustImage));
        assert!(h.shows("TRACE IMAGE"));
        h.script(&["click \"Move on canvas\""]);
        assert!(h.shows("Done moving"));
        h.script(&["click \"Done moving\""]);
        assert!(h.shows("TRACE IMAGE") && h.shows("Move on canvas"));
        h.script(&["click \"Hide\""]);
        h.script(&["click \"Rotate R\""]);
        let app = state(h);
        assert!(!app.bg_image.visible);
        assert!((app.bg_image.rotation - 90.0).abs() < 1e-6);
    }

    // 7. Multiple selection kinds at once (Select All => lines + polygon).
    {
        let mut h = harness_next_at(VIEWPORT);
        draw_closed_square(&mut h);
        h.send(Message::Key(KeyCmd::SelectAll));
        assert!(h.shows("SELECTION ACTIONS"));
        h.script(&["click \"Rotate +90°\""]);
        let app = state(h);
        assert!(!app.selection.lines.is_empty() && !app.selection.polys.is_empty());
    }

    // 8. Compact wall inspector screenshot.
    {
        let cmp = iced::Size::new(372.0, 800.0);
        let mut h = harness_next_at(cmp);
        draw_closed_square(&mut h);
        h.script(&["click \"Select\""]);
        let (wx, wy) = drawn_square_wall_mid_on_screen(cmp.width, cmp.height);
        h.script(&[&format!("click ({wx:.1}, {wy:.1})")]);
        h.script(&["click \"Inspect\""]);
        let shot = h.screenshot_in("e2e-next", "p7-inspector-compact-wall");
        assert!(shot.exists());
    }
}

#[test]
fn p8_insert_drawer_symbols_and_labels_across_sizes() {
        let lib = symlib::library();
        let ix = (*lib).as_ref().expect("symbol library loads");
        let first_fav = ix.favorites.first().expect("default favorites exist").clone();
        let non_fav = ix
            .categories
            .iter()
            .flat_map(|c| c.items.iter())
            .find(|item| !ix.favorites.iter().any(|f| f.id == item.id))
            .expect("non-favorite symbol exists")
            .clone();

    for &(size_label, w, h, _) in SIZES {
        let cx = w * 0.40;
        let cy = h * 0.42;

        // 1. Open Insert drawer (Symbols mode), verify categories, unpinned pick + canvas place closes drawer.
        {
            let mut harness = harness_next_at(iced::Size::new(w, h));
            harness.send(Message::Ui(UiMsg::OpenDrawer(Some(
                crate::ui::tray::DrawerTab::Symbols,
            ))));
            assert!(
                harness.shows("Symbols") && harness.shows("Room labels"),
                "mode switcher missing at {size_label}"
            );
            assert!(harness.shows("Pin"), "pin control missing at {size_label}");
            assert!(
                harness.shows("Edit library"),
                "edit library action missing at {size_label}"
            );
            assert!(
                harness.shows(&format!("Favorites ({})", ix.favorites.len())),
                "Favorites count missing at {size_label}"
            );
            let total_syms: usize = ix.categories.iter().map(|c| c.items.len()).sum();
            assert!(
                harness.shows(&format!("All {total_syms}")),
                "All symbols chip missing at {size_label}"
            );
            for cat in &ix.categories {
                let chip_label = format!("{} ({})", cat.name, cat.items.len());
                assert!(
                    harness.shows(&chip_label),
                    "category chip '{chip_label}' missing at {size_label}"
                );
            }

            harness.script(&[&format!("click {:?}", first_fav.label)]);
            harness.script(&[&format!("click ({cx:.1}, {cy:.1})")]);
            let app = state(harness);
            assert_eq!(
                app.drawing.symbols.len(),
                1,
                "unpinned symbol should place on canvas at {size_label}"
            );
            assert!(
                app.ui.drawer.is_none(),
                "unpinned drawer should close after placing at {size_label}"
            );
        }

        // 2. Pinning keeps drawer open after picking AND placing on canvas.
        {
            let mut harness = harness_next_at(iced::Size::new(w, h));
            harness.send(Message::Ui(UiMsg::OpenDrawer(Some(
                crate::ui::tray::DrawerTab::Symbols,
            ))));
            harness.script(&["click \"Pin\""]);
            assert!(harness.shows("Pinned"), "pin label should update at {size_label}");
            harness.script(&[&format!("click {:?}", first_fav.label)]);
            if !matches!(SizeClass::of(w), SizeClass::Compact) {
                let cx2 = cx * 0.6;
                harness.script(&[&format!("click ({cx2:.1}, {cy:.1})")]);
                let app = state(harness);
                assert!(app.ui.drawer_pinned);
                assert_eq!(app.drawing.symbols.len(), 1);
                assert_eq!(
                    app.ui.drawer,
                    Some(crate::ui::tray::DrawerTab::Symbols),
                    "pinned drawer must stay open after placing at {size_label}"
                );
            } else {
                let app = state(harness);
                assert!(app.ui.drawer_pinned);
                assert_eq!(app.view.mode, Mode::PlaceSymbol);
                assert_eq!(app.ui.drawer, Some(crate::ui::tray::DrawerTab::Symbols));
            }
        }

        // 3. Room labels: filter by letter range, custom label Place, and preset pick.
        {
            let mut harness = harness_next_at(iced::Size::new(w, h));
            harness.send(Message::Ui(UiMsg::OpenDrawer(Some(
                crate::ui::tray::DrawerTab::Labels,
            ))));
            assert!(
                harness.shows("All") && harness.shows("ABC") && harness.shows("Place"),
                "room labels controls missing at {size_label}"
            );
            harness.script(&["click \"ABC\""]);
            assert!(harness.shows("Bath"), "ABC filter should include Bath at {size_label}");
            assert!(
                !harness.shows("Kitchen"),
                "ABC filter should exclude Kitchen at {size_label}"
            );
            harness.script(&["click \"All\""]);
            assert!(harness.shows("Kitchen"), "All filter restores Kitchen at {size_label}");

            harness.send(Message::Dock(crate::dock::DockMsg::TextSearch(
                "Sunroom".into(),
            )));
            harness.script(&["click \"Place\""]);
            harness.script(&[&format!("click ({cx:.1}, {cy:.1})")]);

            harness.send(Message::Dock(crate::dock::DockMsg::TextSearch(
                String::new(),
            )));
            harness.send(Message::Ui(UiMsg::OpenDrawer(Some(
                crate::ui::tray::DrawerTab::Labels,
            ))));
            harness.script(&["click \"Bed\""]);
            let cx3 = cx + 36.0;
            let cy3 = cy + 36.0;
            harness.script(&[&format!("click ({cx3:.1}, {cy3:.1})")]);

            let app = state(harness);
            assert!(
                app.drawing.texts.iter().any(|t| t.text == "Sunroom"),
                "custom room label should place at {size_label}"
            );
            assert!(
                app.drawing.texts.iter().any(|t| t.text == "Bed"),
                "preset room label should place at {size_label}"
            );
            assert!(app.ui.drawer.is_none());
        }
    }

    // 4. Tray open + Context Favorite toggle updates Favorites list live.
    {
        let mut h = harness_next_at(VIEWPORT);
        h.script(&["click \"Insert\""]);
        h.send(Message::SymbolManager(
            crate::symbol_manager::Msg::Context(non_fav.id.clone()),
        ));
        assert!(h.shows("Symbol actions"));
        h.script(&["click \"Favorite\""]);
        assert!(
            h.shows(&format!("Favorites ({})", ix.favorites.len() + 1)),
            "Favorites count chip should update live"
        );
        let app = state(h);
        assert!(
            app.symbol_favorites
                .as_ref()
                .is_some_and(|favs| favs.contains(&non_fav.id)),
            "Favorite context action should add symbol to custom favorites"
        );
    }

    // 5. Visual verification screenshots.
    {
        let mut h = harness_next_at(VIEWPORT);
        h.script(&["click \"Insert\"", "click \"Pin\""]);
        let shot = h.screenshot_in("e2e-next", "p8-insert-symbols-wide");
        assert!(shot.exists());
    }
    {
        let mut h = harness_next_at(VIEWPORT);
        h.script(&["click \"Label\""]);
        let shot = h.screenshot_in("e2e-next", "p8-insert-labels-wide");
        assert!(shot.exists());
    }
    {
        let mut h = harness_next_at(iced::Size::new(390.0, 844.0));
        h.send(Message::Ui(UiMsg::OpenDrawer(Some(
            crate::ui::tray::DrawerTab::Symbols,
        ))));
        let shot = h.screenshot_in("e2e-next", "p8-insert-compact");
        assert!(shot.exists());
    }
}

#[test]
fn measurement_format_applies_to_the_sketch_and_saves_with_it() {
    let mut harness = harness_next_at(VIEWPORT);
    draw_closed_square(&mut harness);
    harness.script(&["click \"Show\"", "click \"Feet and inches\""]);
    let mut app = state(harness);
    assert_eq!(app.drawing.options.units.length(10.5), "10' 6\"");
    let _ = app.update(Message::Status(StatusMsg::PageAdd));
    assert_eq!(app.drawing.options.units.length(10.5), "10' 6\"");

    let saved = app.build_save_model();
    let _ = app.update(Message::Status(StatusMsg::LengthFormat { feet_and_inches: false }));
    assert_eq!(app.drawing.options.units.length(10.5), "10.5'");
    app.apply_settings(saved.get("settings"));
    assert_eq!(app.drawing.options.units.length(10.5), "10' 6\"");
}

#[test]
fn zoom_fence_button_zooms_to_a_dragged_rectangle() {
    for &(label, w, h, _) in SIZES {
        let size = iced::Size::new(w, h);
        let initial = state(harness_next_at(size));
        let canvas = crate::ui::layout::canvas_rect(&initial);
        let mut harness = harness_next_at(size);

        // The magnifier is the icon between plus and Fit in the top-left bar.
        let button = format!("click ({}, {})", canvas.x + 125.0, canvas.y + 24.0);
        let center = canvas.center() + iced::Vector::new(35.0, 25.0);
        let selected = initial
            .view
            .to_world(f64::from(center.x - canvas.x), f64::from(center.y - canvas.y));
        let end = format!("({}, {})", center.x + 50.0, center.y + 40.0);
        harness.script(&[
            &button,
            &format!("press ({}, {})", center.x - 50.0, center.y - 40.0),
            &format!("move {end}"),
            &format!("release {end}"),
        ]);

        let app = state(harness);
        assert!(app.view.zoom > 1.0, "rectangle did not zoom at {label}");
        let at_center = app.view.to_world(app.view.canvas.0 / 2.0, app.view.canvas.1 / 2.0);
        assert!((at_center.x - selected.x).abs() < 1e-6, "x center at {label}");
        assert!((at_center.y - selected.y).abs() < 1e-6, "y center at {label}");
        assert_eq!(app.view.mode, Mode::Select);
        assert!(app.view.fence.is_none());
    }
}

#[test]
fn drawing_chrome_laid_out_controls_never_overlap_the_tray() {
    use crate::e2e_support::scroll_areas;
    let mut h = harness_next_at(VIEWPORT);
    h.send(Message::Status(StatusMsg::PageAdd));
    draw_closed_square(&mut h);
    let mut app = state(h);
    let line = app.drawing.lines[0].id;
    app.view.mode = Mode::Pan;
    // Includes the reported desktop layout, logical sizes at 125%/150%
    // Windows scaling, a very wide bottom dock, and a narrow canvas with
    // both the resized ledger and inspector taking space.
    for (w, height) in [(2600.0, 1100.0), (2048.0, 1218.0), (1638.4, 974.4),
        (1920.0, 1080.0), (1280.0, 800.0), (1100.0, 800.0), (800.0, 1000.0), (372.0, 800.0)] {
        for density in [Density::Comfortable, Density::Touch] {
            for inspect in [false, true] {
                app.window_size = iced::Size::new(w, height);
                app.ui.density = density;
                app.ui.ledger_width = Some(if inspect { 400.0 } else { density.ledger_w() });
                app.selection.clear();
                if inspect { app.selection.lines.insert(line); }
                let canvas = crate::ui::layout::canvas_rect(&app);
                let areas = scroll_areas(&app, app.window_size);
                let find = |name: &str| areas.iter().find(|a| a.id.as_ref().is_some_and(|id| id.contains(name)))
                    .unwrap_or_else(|| panic!("missing {name}: {w} {density:?} inspector={inspect}"));
                let zoom = find("drawing-zoom").bounds;
                let snap = find("drawing-snap").bounds;
                let compact = SizeClass::of(w) == SizeClass::Compact;
                let tray = find(if compact { "drawing-compact-tray" } else { "drawing-tool-tray" }).bounds;
                let expanded = |r: iced::Rectangle, pad: f32| iced::Rectangle {
                    x: r.x - pad, y: r.y - pad,
                    width: r.width + pad * 2.0, height: r.height + pad * 2.0,
                };
                let tray_card = expanded(tray, if compact { 0.0 } else { 6.0 });
                for control in [zoom, snap] {
                    let card = expanded(control, 4.0);
                    let overlaps = card.x < tray_card.x + tray_card.width
                        && card.x + card.width > tray_card.x
                        && card.y < tray_card.y + tray_card.height
                        && card.y + card.height > tray_card.y;
                    assert!(!overlaps, "overlap at {w} {density:?} inspector={inspect}: {card:?} vs {tray_card:?}");
                    assert!(control.x >= canvas.x && control.x + control.width <= canvas.x + canvas.width + 0.5,
                        "control outside canvas at {w}: {control:?} vs {canvas:?}");
                }
                assert!(zoom.intersection(&snap).is_none(), "top controls overlap at {w}");
                assert!(tray.x >= canvas.x && tray.x + tray.width <= canvas.x + canvas.width + 0.5,
                    "tray outside canvas at {w}: {tray:?}");
            }
        }
    }
}

#[test]
fn drawing_chrome_buttons_and_popovers_work_after_reflow() {
    for (name, w, height, density) in [
        ("reported", 1536.0, 914.0, Density::Comfortable),
        ("wide", 2200.0, 1080.0, Density::Comfortable),
        ("touch", 1280.0, 800.0, Density::Touch),
    ] {
        let mut h = harness_next_at(iced::Size::new(w, height));
        h.send(Message::Ui(UiMsg::SetDensity(density)));
        h.script(&["click \"Freeform\"", "click \"Erase\"", "click \"Pan\"",
            "click \"Grid\"", "click \"Snap\"", "click \"Dims: Out\""]);
        h.script(&["click \"100%\""]);
        assert!(h.shows("Zoom & Scale"));
        h.send(Message::Ui(UiMsg::CloseOverlays));
        h.script(&["click \"Show\""]);
        assert!(h.shows("Dimensions"));
        h.send(Message::Ui(UiMsg::CloseOverlays));
        // Clicking active Pan opens its flyout at the tray, clear of the
        // drawing controls in both the top and bottom arrangements.
        h.script(&["click \"Pan\""]);
        assert!(h.shows("Zoom to rectangle"));
        let shot = h.screenshot_in("e2e-next", &format!("chrome-{name}-flyout"));
        assert!(shot.exists());
        let app = state(h);
        assert!(!app.view.grid && !app.view.snap);
        assert_eq!(app.ui.tray_flyout, Some(TrayTool::Pan));
    }
}

#[test]
fn p9_controls_length_strip_and_area_chooser_across_sizes() {
    for &(label, w, h, _) in SIZES {
        // 1. Zoom, scale, fit, grid, snap, pitch/angle-step, show, dims, north, underlay, minimap, touch.
        {
            let mut harness = harness_next_at(iced::Size::new(w, h));
            assert!(harness.shows("100%"), "zoom pct missing at {label}");
            assert!(harness.shows("Grid"), "Grid pill missing at {label}");
            assert!(harness.shows("Snap"), "Snap pill missing at {label}");
            assert!(harness.shows("Show"), "Show pill missing at {label}");
            assert!(harness.shows("Dims: Out"), "Dims pill missing at {label}");
            assert!(harness.shows("0°"), "North pill missing at {label}");

            // Zoom popover + slider + Scale submit.
            harness.script(&["click \"100%\""]);
            assert!(harness.shows("Zoom & Scale"), "Zoom popover missing at {label}");
            harness.send(Message::Status(StatusMsg::ZoomSlider(150.0)));
            assert!(harness.shows("150%"), "Zoom pct did not update at {label}");
            harness.send(Message::Status(StatusMsg::ScaleEdit("20".into())));
            harness.script(&["click \"Apply\""]);
            {
                let app = state(harness);
                assert!((app.view.drawing_scale - 20.0).abs() < 1e-6);
                assert!(app.ui.controls_popover.is_none());
            }

            let mut harness = harness_next_at(iced::Size::new(w, h));
            harness.send(Message::Ui(UiMsg::ToggleControlsPopover(
                crate::ui::ControlsPopover::Fit,
            )));
            assert!(harness.shows("Center & Fit"));
            harness.script(&["click \"Zoom Fence\""]);
            assert_eq!(state(harness).view.mode, Mode::ZoomFence);

            // Add Page 2 first so Underlay chip appears and per-page grid starts true.
            let mut harness = harness_next_at(iced::Size::new(w, h));
            harness.send(Message::Status(StatusMsg::PageAdd));
            assert!(harness.shows("Underlay: None"));
            harness.script(&["click \"Underlay: None\""]);
            assert!(harness.shows("Underlay page"));
            harness.script(&["click \"Underlay: Page 1\""]);
            assert!(harness.shows("Underlay: Page 1"));

            // Grid, Snap, Snap popover (Pitch + Area snap + Angle step).
            assert!(harness.shows("Grid"));
            harness.script(&["click \"Grid\""]);
            harness.script(&["click \"Snap\""]);
            harness.script(&["click \"1'\""]);
            assert!(harness.shows("Snap Pitch"));
            assert!(harness.shows("Angle Step"));
            harness.script(&["click \"1/4'\""]);
            harness.script(&["click \"Area snap\""]);
            harness.script(&["click \"45°\""]);
            assert!(harness.shows("1/4'"));
            harness.send(Message::Ui(UiMsg::CloseOverlays));

            // Show popover (6 filters) & Dimensions cycle.
            harness.script(&["click \"Show\""]);
            assert!(harness.shows("Show in sketch"));
            harness.script(&["click \"Symbols\""]);
            harness.send(Message::Ui(UiMsg::CloseOverlays));
            harness.script(&["click \"Dims: Out\""]);
            assert!(harness.shows("Dims: In"));
            harness.script(&["click \"Dims: In\""]);
            assert!(harness.shows("Dims: Off"));

            // North compass popover.
            harness.script(&["click \"0°\""]);
            assert!(harness.shows("North angle (°)"));
            harness.send(Message::Ui(UiMsg::NorthEdit("35".into())));
            harness.script(&["click \"Set\""]);
            assert!(harness.shows("35°"));

            // Mini-map display (window screenshots don't complete in headless harness).
            let sw = w as u32;
            let sh = h as u32;
            harness.send(Message::MiniMapShot(iced::window::Screenshot::new(
                vec![230u8; (sw * sh * 4) as usize],
                iced::Size::new(sw, sh),
                1.0,
            )));
            assert!(!harness.shows("Pad"));

            // Touch extras: Pad + Laser.
            harness.send(Message::Ui(UiMsg::SetDensity(Density::Touch)));
            assert!(harness.shows("Pad"));
            assert!(harness.shows("Laser"));
            harness.script(&["click \"Pad\""]);

            let app = state(harness);
            assert!(!app.view.grid);
            assert!(!app.view.snap);
            assert!(app.view.area_snap);
            assert!((app.drawing.options.ortho_angle - 45.0).abs() < 1e-6);
            assert!(!app.view.filters[5]);
            assert!(!app.view.dimensions);
            assert!((app.pages.page().north_angle - 35.0).abs() < 1e-6);
            assert_eq!(app.pages.underlay_label(), "Underlay: Page 1");
            assert!(app.mini_map.is_some());
            assert!(app.touch_panel_open);
        }

        // 2. Length Strip in Wall mode and Measure mode.
        {
            let mut harness = harness_next_at(iced::Size::new(w, h));
            harness.send(Message::Key(KeyCmd::DrawExterior));
            assert!(harness.shows("pen up"), "wall strip missing at {label}");
            harness.send(Message::Key(KeyCmd::Enter));
            assert!(harness.shows("pen down"), "pen down indicator missing at {label}");
            harness.send(Message::Key(KeyCmd::Buffer("1")));
            harness.send(Message::Key(KeyCmd::Buffer("2")));
            assert!(harness.shows("12'"), "typed buffer length missing in strip at {label}");
            harness.send(Message::Key(KeyCmd::Arrow {
                dir: 2,
                ctrl: false,
                shift: false,
            }));
            assert!(harness.shows("12.0'"), "moved length missing in strip at {label}");

            // Measure mode strip & actions.
            harness.send(Message::Rail(crate::view_state::RailTool::Measure));
            assert!(harness.shows("Finish"));
            assert!(harness.shows("Close"));
            assert!(harness.shows("Undo point"));
            assert!(harness.shows("Clear"));
            assert!(harness.shows("Copy"));
            let (cx, cy) = (w * 0.5, h * 0.5);
            harness.script(&[
                &format!("click ({:.1}, {:.1})", cx - 40.0, cy),
                &format!("click ({:.1}, {:.1})", cx + 40.0, cy),
                &format!("click ({:.1}, {:.1})", cx + 40.0, cy + 40.0),
            ]);
            assert_eq!(state_peek_measure_pts(w, h), 3);
            harness.script(&["click \"Undo point\""]);
            harness.script(&["click \"Close\""]);
            harness.script(&["click \"Finish\""]);
            harness.script(&["click \"Copy\""]);
            harness.script(&["click \"Clear\""]);
            let app = state(harness);
            assert!(app.drawing.measure_points.is_empty());
            assert!(app.drawing.measure_runs.is_empty());
        }

        // 3. Area Chooser opens in Area mode, lists all types cleanly, filters & applies.
        {
            let (cx, cy) = drawn_square_center_on_screen(w, h);
            let mut harness = harness_next_at(iced::Size::new(w, h));
            draw_closed_square(&mut harness);
            harness.send(Message::Rail(crate::view_state::RailTool::DefineArea));
            harness.script(&[&format!("click ({cx:.1}, {cy:.1})")]);
            assert!(
                harness.shows("What is this area?"),
                "Area Chooser header missing at {label}"
            );
            assert!(harness.shows("All"), "All group chip missing at {label}");
            assert!(harness.shows("Apply"), "Apply button missing at {label}");
            // Both GLA and non-GLA area types visible when search is empty.
            assert!(harness.shows("First Floor"), "First Floor type missing at {label}");
            assert!(harness.shows("Garage"), "Garage type missing at {label}");

            harness.script(&["click \"Label switches ▸\""]);
            assert!(harness.shows("Name"));
            assert!(harness.shows("Use for the next areas too"));
            harness.script(&["click \"Use for the next areas too\""]);
            harness.send(Message::AreaWorkflow(crate::area_workflow::Msg::Submit));
            let app = state(harness);
            assert!(app.workflow.chooser.is_none());
            assert_ne!(app.drawing.polygons[0].kind, "UND");
        }
    }

    // Visual screenshots for P9 inspection.
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Key(KeyCmd::DrawExterior));
        h.send(Message::Key(KeyCmd::Enter));
        h.send(Message::Key(KeyCmd::Buffer("1")));
        h.send(Message::Key(KeyCmd::Buffer("2")));
        h.send(Message::Key(KeyCmd::Arrow {
            dir: 2,
            ctrl: false,
            shift: false,
        }));
        h.send(Message::MiniMapShot(iced::window::Screenshot::new(
            vec![230u8; (VIEWPORT.width as u32 * VIEWPORT.height as u32 * 4) as usize],
            iced::Size::new(VIEWPORT.width as u32, VIEWPORT.height as u32),
            1.0,
        )));
        let shot = h.screenshot_in("e2e-next", "p9-controls-tape-wide");
        assert!(shot.exists());
    }
    {
        let mut h = harness_next_at(VIEWPORT);
        h.script(&["click \"100%\""]);
        let shot = h.screenshot_in("e2e-next", "p9-controls-popovers-wide");
        assert!(shot.exists());
    }
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Rail(crate::view_state::RailTool::Measure));
        h.script(&["click (660.0, 420.0)", "click (760.0, 420.0)"]);
        let shot = h.screenshot_in("e2e-next", "p9-measure-wide");
        assert!(shot.exists());
    }
    {
        let (cx, cy) = drawn_square_center_on_screen(VIEWPORT.width, VIEWPORT.height);
        let mut h = harness_next_at(VIEWPORT);
        draw_closed_square(&mut h);
        h.send(Message::Rail(crate::view_state::RailTool::DefineArea));
        h.script(&[&format!("click ({cx:.1}, {cy:.1})")]);
        let shot = h.screenshot_in("e2e-next", "p9-chooser-wide");
        assert!(shot.exists());
    }
    {
        let mut h = harness_next_at(iced::Size::new(372.0, 800.0));
        h.send(Message::Status(StatusMsg::PageAdd));
        let shot = h.screenshot_in("e2e-next", "p9-controls-compact");
        assert!(shot.exists());
    }
}

#[test]
fn p10_touch_input_pad_and_gestures_across_sizes() {
    use crate::ui::PadPopover;

    // 1. Auto-show on TouchDetected in Wall tool, hardware key auto-hide, touch re-show,
    //    Compact bottom docking, and drawing a 3-area floor plan via Pad across all 4 sizes.
    for &(label, w, h, class) in SIZES {
        // 1a. Hardware key auto-hide + touch re-show + Compact canvas height shrink.
        {
            let mut harness = harness_next_at(iced::Size::new(w, h));
            harness.send(Message::Ui(UiMsg::TouchDetected));
            assert!(!harness.shows("Turn L"), "Pad should not auto-show in Select mode at {label}");
            harness.send(Message::Rail(crate::view_state::RailTool::DrawExterior));
            assert!(harness.shows("Pen: Up"), "Pad header missing at {label}");
            assert!(harness.shows("Turn L"), "Pad Turn L key missing at {label}");
            assert!(harness.shows("Close area"), "Pad Close area key missing at {label}");

            harness.send(Message::Ui(UiMsg::HardwareKeyPressed));
            assert!(!harness.shows("Turn L"), "Hardware key must hide Pad at {label}");
            harness.send(Message::Ui(UiMsg::TouchDetected));
            assert!(harness.shows("Turn L"), "Touch after hardware key must re-show Pad at {label}");

            if matches!(class, SizeClass::Compact) {
                harness.script(&["click \"Pad\""]);
                assert!(!harness.shows("Close area"), "Pad chip toggle must close Pad in Compact");
                harness.send(Message::Ui(UiMsg::MenuRunId("touch-panel")));
                assert!(harness.shows("Close area"), "Menu > View > Pad must re-open Pad");
                let app = state(harness);
                let expected_open_h = crate::ui::layout::canvas_rect_next(
                    w,
                    h,
                    Density::Touch,
                    0.0,
                    false,
                    true,
                )
                .height;
                assert!(
                    (app.canvas_rect().height - expected_open_h).abs() < 1.0,
                    "Compact Pad must dock above bottom toolbar and shrink canvas height"
                );
            } else {
                let app = state(harness);
                assert!(
                    matches!(app.ui.density, Density::Touch),
                    "first touch must switch density to Touch at {label}"
                );
                assert!(
                    app.touch_panel_open,
                    "Pad must auto-show in Touch density on DrawExterior at {label}"
                );
                assert!(app.canvas_rect().width > 0.0);
            }
        }

        // 1b. Draw a 3-area floor plan using Pad buttons & taps only.
        {
            let mut harness = harness_next_at(iced::Size::new(w, h));
            harness.send(Message::Ui(UiMsg::TouchDetected));
            harness.send(Message::Rail(crate::view_state::RailTool::DrawExterior));

            // Area 1 (10'6" x 10') using Pad buttons: Pen: Up -> digits + "'" + arrow -> Close area.
            harness.script(&["click \"Pen: Up\""]);
            assert!(harness.shows("Pen: Down"));
            harness.script(&["click \"1\"", "click \"0\"", "click \"'\"", "click \"6\""]);
            harness.send(Message::Ui(UiMsg::PadArrow(2))); // 10'6" East
            harness.script(&["click \"1\"", "click \"0\""]);
            harness.send(Message::Ui(UiMsg::PadArrow(4))); // 10' South
            harness.script(&["click \"1\"", "click \"0\"", "click \"'\"", "click \"6\""]);
            harness.send(Message::Ui(UiMsg::PadArrow(6))); // 10'6" West
            harness.script(&["click \"Close area\""]);

            // Area 2 to the east (29' East pen-up jump) using Pen toggle & Turn R (9' x 8').
            // Use digits 7/8/9 so Selector::find matches Pad buttons rather than Ledger area counts.
            harness.script(&["click \"2\"", "click \"9\""]);
            harness.send(Message::Ui(UiMsg::PadArrow(2)));
            harness.script(&["click \"Pen: Up\""]);
            harness.script(&["click \"9\""]);
            harness.send(Message::Ui(UiMsg::PadArrow(2)));
            harness.script(&["click \"8\""]);
            harness.send(Message::Ui(UiMsg::PadTurn(false))); // Turn R (South)
            harness.script(&["click \"9\""]);
            harness.send(Message::Ui(UiMsg::PadTurn(false))); // Turn R (West)
            harness.script(&["click \"Close area\""]);

            // Area 3 to the south (9.5' + 9.5' = 19' South pen-up jump, 8' x 9').
            harness.script(&["click \"9\"", "click \".\"", "click \"5\""]);
            harness.send(Message::Ui(UiMsg::PadArrow(4)));
            harness.script(&["click \"9\"", "click \".\"", "click \"5\""]);
            harness.send(Message::Ui(UiMsg::PadArrow(4)));
            harness.script(&["click \"Pen: Up\""]);
            harness.script(&["click \"8\""]);
            harness.send(Message::Ui(UiMsg::PadArrow(2)));
            harness.script(&["click \"9\""]);
            harness.send(Message::Ui(UiMsg::PadArrow(4)));
            harness.script(&["click \"8\""]);
            harness.send(Message::Ui(UiMsg::PadArrow(6)));
            harness.script(&["click \"Close area\""]);

            let app = state(harness);
            assert_eq!(
                app.drawing.polygons.len(),
                3,
                "Pad drawing must close 3 areas at {label}"
            );
            assert!(
                app.drawing.lines.iter().any(|l| {
                    let dx = l.end.x - l.start.x;
                    let dy = l.end.y - l.start.y;
                    (dx.hypot(dy) - 10.5).abs() < 1e-3
                }),
                "Feet-inches 10'6\" wall missing at {label}"
            );
        }
    }

    // 2. Pad Diagonals & More popovers + Laser integration.
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Ui(UiMsg::TouchDetected));
        h.send(Message::Rail(crate::view_state::RailTool::DrawExterior));
        h.send(Message::Ui(UiMsg::PadPopover(Some(PadPopover::Diagonals))));
        assert!(h.shows("NW") && h.shows("NE") && h.shows("SW") && h.shows("SE"));
        h.send(Message::Ui(UiMsg::PadPopover(Some(PadPopover::More))));
        assert!(
            h.shows("Bay mirror")
                && h.shows("Jump endpoint")
                && h.shows("Rise × run (x)")
                && h.shows("Angle (<)")
                && h.shows("Commit")
        );
        h.send(Message::Ui(UiMsg::PadPopover(None)));

        // Disto session & measurement populates buffer and Laser key re-applies it.
        h.script(&["click \"Pen: Up\""]);
        h.script(&["click \"Laser\""]);
        h.send(Message::Disto(crate::disto::DistoMsg::ScannedFor(
            1,
            Ok(vec![crate::disto::Device {
                id: "d2".into(),
                name: Some("Disto D2".into()),
            }]),
        )));
        h.send(Message::Disto(crate::disto::DistoMsg::Pick(0)));
        h.send(Message::Disto(crate::disto::DistoMsg::Choose));
        h.send(Message::Disto(crate::disto::DistoMsg::ConnectedFor(
            1,
            Ok("Disto D2".into()),
        )));
        h.send(Message::Disto(crate::disto::DistoMsg::Radio(
            1,
            crate::disto::RadioEvent::Measurement(3.048),
        )));
        h.send(Message::Key(KeyCmd::BufferBackspace));
        h.send(Message::Key(KeyCmd::BufferBackspace));
        h.send(Message::Ui(UiMsg::PadLaser));
        let app = state(h);
        assert!(
            app.drawing.buffer.starts_with("10"),
            "Disto/Pad Laser measurement should populate drawing buffer, got {:?} (session={}, phase={:?}, conn={:?}, last={:?}, pen={}, mode={:?})",
            app.drawing.buffer,
            app.disto.session,
            app.disto.phase,
            app.disto.connected,
            app.last_disto_meters,
            app.drawing.pen_down,
            app.view.mode,
        );
    }

    // 3. Touch Area workflow: 1st tap previews region & size, 2nd tap opens Chooser.
    {
        let (cx, cy) = drawn_square_center_on_screen(VIEWPORT.width, VIEWPORT.height);
        let r = {
            let mut peek = harness_next_at(VIEWPORT);
            draw_closed_square(&mut peek);
            state(peek).canvas_rect()
        };
        let (rx, ry) = (cx - r.x, cy - r.y);

        // First touch tap: preview + size status without opening chooser.
        {
            let mut h = harness_next_at(VIEWPORT);
            draw_closed_square(&mut h);
            h.send(Message::Ui(UiMsg::TouchDetected));
            h.send(Message::Rail(crate::view_state::RailTool::DefineArea));
            h.send(Message::CanvasTouchTap { x: rx, y: ry });
            let app = state(h);
            assert!(
                app.workflow.status.contains("Preview:") && app.workflow.status.contains("100.0 sf"),
                "first touch tap in DefineArea must preview size: {:?}",
                app.workflow.status
            );
            assert!(
                app.workflow.chooser.is_none(),
                "first touch tap must preview without immediately opening chooser"
            );
        }

        // Second touch tap on same region opens Area Chooser and assigns GLA1.
        {
            let mut h = harness_next_at(VIEWPORT);
            draw_closed_square(&mut h);
            h.send(Message::Ui(UiMsg::TouchDetected));
            h.send(Message::Rail(crate::view_state::RailTool::DefineArea));
            h.send(Message::CanvasTouchTap { x: rx, y: ry });
            h.send(Message::CanvasTouchTap { x: rx, y: ry });
            h.script(&["click \"First Floor\""]);
            h.send(Message::AreaWorkflow(crate::area_workflow::Msg::Submit));
            let app = state(h);
            assert_eq!(app.drawing.polygons[0].kind, "GLA1");
        }
    }

    // 4. Select more toggle + 48 px handle tolerance + RightPress context menu + pen-ink flag.
    {
        let (sx0, sy0, sx1, sy1, r) = {
            let mut peek = harness_next_at(VIEWPORT);
            draw_closed_square(&mut peek);
            let app = state(peek);
            let r = app.canvas_rect();
            let (sx0, sy0) = app.view.to_screen(crate::view_state::World { x: 5.0, y: 0.0 });
            let (sx1, sy1) = app.view.to_screen(crate::view_state::World { x: 10.0, y: -5.0 });
            (sx0 as f32, sy0 as f32, sx1 as f32, sy1 as f32, r)
        };

        let mut h = harness_next_at(VIEWPORT);
        draw_closed_square(&mut h);
        h.send(Message::Ui(UiMsg::TouchDetected));
        h.send(Message::Rail(crate::view_state::RailTool::Select));
        assert!(h.shows("Select more"), "Select more pill missing in Select mode");
        h.script(&["click \"Select more\""]);
        h.send(Message::CanvasTouchTap { x: sx0, y: sy0 });
        h.send(Message::CanvasTouchTap { x: sx1, y: sy1 });
        h.send(Message::RightPress(iced::Point::new(r.x + 100.0, r.y + 100.0)));
        h.send(Message::Options(crate::options_dialog::OptMsg::Open));
        h.send(Message::Options(crate::options_dialog::OptMsg::Str(
            crate::options::StrKey::InputMode,
            "pen-ink".into(),
        )));
        h.send(Message::Options(crate::options_dialog::OptMsg::Ok));

        let app = state(h);
        assert!(app.ui.select_more);
        assert_eq!(
            app.selection.lines.len(),
            2,
            "Select more must accumulate selections across taps"
        );
        let line = app.drawing.lines[0].clone();
        let offset_world = sketch_core::geometry::Pt::new(
            line.start.x + 18.0 / (sketch_core::geometry::PIXELS_PER_FOOT * app.view.zoom),
            line.start.y,
        );
        assert!(
            matches!(
                app.vertex_handle(offset_world),
                Some((crate::handles::VertexHandle::WallEnd { id, start: true }, _)) if id == line.id
            ),
            "Touch density endpoint handle must hit within 24 px radius (48 px diameter)"
        );
        assert!(app.context.open.is_some(), "RightPress must open canvas context menu");
        assert_eq!(app.drawing.options.input_mode, "pen-ink");
    }

    // 5. Visual verification screenshots across Wide, Medium, and Compact.
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Ui(UiMsg::TouchDetected));
        h.send(Message::Rail(crate::view_state::RailTool::DrawExterior));
        h.script(&["click \"Pen: Up\"", "click \"1\"", "click \"2\""]);
        h.send(Message::Ui(UiMsg::PadArrow(2)));
        let shot = h.screenshot_in("e2e-next", "p10-pad-wide");
        assert!(shot.exists());
    }
    {
        let mut h = harness_next_at(iced::Size::new(800.0, 1280.0));
        h.send(Message::Ui(UiMsg::TouchDetected));
        h.send(Message::Rail(crate::view_state::RailTool::DrawExterior));
        h.script(&["click \"Pen: Up\"", "click \"1\"", "click \"4\""]);
        h.send(Message::Ui(UiMsg::PadArrow(4)));
        let shot = h.screenshot_in("e2e-next", "p10-pad-tablet-800x1280");
        assert!(shot.exists());
    }
    {
        let mut h = harness_next_at(iced::Size::new(372.0, 800.0));
        h.send(Message::Ui(UiMsg::TouchDetected));
        h.send(Message::Rail(crate::view_state::RailTool::DrawExterior));
        h.script(&["click \"Pen: Up\"", "click \"8\""]);
        let shot = h.screenshot_in("e2e-next", "p10-pad-compact");
        assert!(shot.exists());
    }
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Ui(UiMsg::TouchDetected));
        h.send(Message::Rail(crate::view_state::RailTool::DrawExterior));
        h.send(Message::Ui(UiMsg::PadPopover(Some(PadPopover::Diagonals))));
        let shot = h.screenshot_in("e2e-next", "p10-pad-diagonals-wide");
        assert!(shot.exists());
    }
}

fn state_peek_measure_pts(w: f32, h: f32) -> usize {
    let mut tmp = harness_next_at(iced::Size::new(w, h));
    tmp.send(Message::Rail(crate::view_state::RailTool::Measure));
    let (cx, cy) = (w * 0.5, h * 0.5);
    tmp.script(&[
        &format!("click ({:.1}, {:.1})", cx - 40.0, cy),
        &format!("click ({:.1}, {:.1})", cx + 40.0, cy),
        &format!("click ({:.1}, {:.1})", cx + 40.0, cy + 40.0),
    ]);
    state(tmp).drawing.measure_points.len()
}

#[test]
fn area_type_palette_paints_boundaries_and_fills_on_canvas() {
    use sketch_core::geometry::Pt;
    use sketch_core::model::{Polygon, SketchLine};

    let areas = [
        ("DK", 0xc05621),
        ("GLA1", 0x2b6cb0),
        ("OTH", 0xa0aec0),
        ("PORCH", 0xd69e2e),
        ("BAL", 0xb7791f),
        ("GLA2", 0x0987a0),
    ];
    let mut fixture = state(harness_next_at(VIEWPORT));
    fixture.drawing.options.grid_enabled = false;
    let mut samples = Vec::new();
    for (index, (code, rgb)) in areas.into_iter().enumerate() {
        let x = 16.0 * (index % 3) as f64;
        let y = 16.0 * (index / 3) as f64;
        let points = [
            Pt::new(x, y), Pt::new(x + 12.0, y),
            Pt::new(x + 12.0, y + 12.0), Pt::new(x, y + 12.0),
        ];
        for (edge, (start, end)) in points.iter().zip(points.iter().cycle().skip(1)).enumerate() {
            let id = fixture.drawing.gen_id();
            let line_color = if code == "DK" && edge == 2 {
                "#DB2777"
            } else {
                "#2B6CB0"
            };
            let line: SketchLine = serde_json::from_value(serde_json::json!({
                "id": id, "start": start, "end": end, "color": line_color,
            }))
            .unwrap();
            fixture.drawing.lines.push(line);
        }
        let id = fixture.drawing.gen_id();
        let edges = Polygon::build_edges(&points, &fixture.drawing.lines);
        let poly = serde_json::from_value(serde_json::json!({
            "id": id, "type": code, "points": points, "edges": edges,
            "name": fixture.area_codes.iter().find(|area| area.code == code).unwrap().name,
        }))
        .unwrap();
        fixture.drawing.polygons.push(poly);
        samples.push((code, rgb, Pt::new(x + 12.0, y + 4.0), Pt::new(x + 2.0, y + 9.0)));
    }
    let file = std::env::temp_dir().join(format!("sketch-rust-canvas-palette-{}.skx", std::process::id()));
    let bytes = sketch_core::skx::write(&fixture.build_save_model(), &Default::default()).unwrap();
    std::fs::write(&file, bytes).unwrap();

    let mut harness = harness_next_at(VIEWPORT);
    harness.send(Message::FileDropped(file.clone()));
    std::fs::remove_file(file).unwrap();
    harness.send(Message::PopupMenu(PopupMsg::Canvas(crate::canvas_theme::CanvasTheme::Dark)));
    harness.send(Message::Status(StatusMsg::CenterFit));
    harness.send(Message::Status(StatusMsg::ZoomOut));
    let screenshot = image::open(harness.screenshot_in("e2e-next", "canvas-area-colors"))
        .unwrap()
        .into_rgb8();
    let app = state(harness);
    assert_eq!(app.drawing.polygons.len(), areas.len());
    let origin = app.canvas_origin();
    let pixel = |at: Pt| {
        let (x, y) = app.view.to_screen(crate::view_state::World { x: at.x, y: at.y });
        let px = (x + f64::from(origin.x)).round() as u32;
        let py = (y + f64::from(origin.y)).round() as u32;
        &screenshot.get_pixel(px, py).0
    };
    let background = pixel(Pt::new(14.0, 5.0));
    for (code, rgb, edge, inside) in samples {
        let color = [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8];
        for (actual, expected) in pixel(edge).iter().zip(color) {
            assert!(actual.abs_diff(expected) <= 3, "{code} boundary {:?}", pixel(edge));
        }
        for ((actual, background), source) in pixel(inside).iter().zip(background).zip(color) {
            let blended = (f32::from(*background) * 0.9 + f32::from(source) * 0.1).round() as u8;
            assert!(actual.abs_diff(blended) <= 5, "{code} fill {:?}", pixel(inside));
        }
    }
    assert_eq!(pixel(Pt::new(4.0, 12.0)), &[219, 39, 119], "custom wall colour");
}

#[test]
fn p11_area_type_editor_and_default_colors_across_sizes() {
    use crate::color_dialog::{ColorMsg, ColorTarget};
    use crate::ui::type_editor::Msg as TypeMsg;

    // 1. Fresh boot has updated GLA2..4 colors and preserves existing .skx custom codes.
    {
        let mut h = harness_next_at(VIEWPORT);
        {
            let tmp_dir = std::env::temp_dir().join("sketch_rust_p11_preserve");
            let _ = std::fs::create_dir_all(&tmp_dir);
            let skx_path = tmp_dir.join("custom_codes.skx");
            h.send(Message::Key(KeyCmd::Buffer("10")));
            h.send(Message::Key(KeyCmd::Enter));
            h.send(Message::Rail(crate::view_state::RailTool::DrawExterior));
            let mut app = state(h);
            assert_eq!(app.area_codes.len(), 26);
            assert_eq!(app.area_codes[1].code, "GLA2");
            assert_eq!(app.area_codes[1].color, "#0987A0");
            assert_eq!(app.area_codes[2].color, "#B83280");
            assert_eq!(app.area_codes[3].color, "#C53030");

            // Give GLA2 a custom name + color, save .skx, reopen into fresh App -> must preserve!
            app.area_codes[1].color = "#112233".to_string();
            app.area_codes[1].name = "Custom Upper".to_string();
            let model = app.build_save_model();
            let bytes = sketch_core::skx::write(&model, &Default::default()).unwrap();
            std::fs::write(&skx_path, bytes).unwrap();
            let _ = app.open_sketch_path(skx_path.clone());
            assert_eq!(app.area_codes[1].color, "#112233");
            assert_eq!(app.area_codes[1].name, "Custom Upper");
            let _ = std::fs::remove_dir_all(&tmp_dir);
        }
    }

    // 2. Entry points, field edits, ColorDialog integration, duplicate, reorder, cancel confirm & save.
    {
        let mut h = harness_next_at(VIEWPORT);
        h.script(&["click \"Types\"", "click \"Edit types\""]);
        assert!(h.shows("Area types"));
        assert!(h.shows("Built in"));
        assert!(h.shows("Count in gross living area"));

        // Edit fields on selected GLA1 and open ColorDialog for Line & Fill.
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::Name("Main Living".into()))));
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::Description("Primary GLA".into()))));
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::Multiplier("1.25".into()))));
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::Sign(false))));
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::IncludeGla(false))));
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::FillStyle("Crosshatch".into()))));
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::FillOpacity(35.0))));
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::PickLineColor)));
        h.send(Message::Color(ColorMsg::Preset("#E53E3E")));
        h.send(Message::Color(ColorMsg::Ok));
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::PickFillColor)));
        h.send(Message::Color(ColorMsg::Preset("#38A169")));
        h.send(Message::Color(ColorMsg::Ok));

        // Duplicate + MoveUp + Cancel dirty confirmation + Save.
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::Duplicate)));
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::MoveUp(1))));
        h.send(Message::Ui(UiMsg::TypeEditor(TypeMsg::RequestCancel)));
        assert!(h.shows("Discard changes"));
        h.script(&["click \"Keep editing\""]);
        h.script(&["click \"Save\""]);
        assert!(h.shows("Main Living"));
        assert!(h.shows("GLA1_COPY"));

        let app = state(h);
        assert!(!app.ui.type_editor.open);
        assert_eq!(app.color_target, ColorTarget::DrawColor);
        assert_eq!(app.area_codes[0].code, "GLA1_COPY");
        let gla1 = app.area_codes.iter().find(|c| c.code == "GLA1").unwrap();
        assert_eq!(gla1.name, "Main Living");
        assert_eq!(gla1.description, "Primary GLA");
        assert!((gla1.multiplier - 1.25).abs() < 1e-6);
        assert!(!gla1.positive);
        assert!(!gla1.include_in_gla);
        assert_eq!(gla1.fill_style, "crosshatch");
        assert!((gla1.fill_opacity - 35.0).abs() < 1e-6);
        assert_eq!(gla1.color, "#E53E3E");
        assert_eq!(gla1.fill_color, "#38A169");
    }

    // 3. Rename & Delete across 2 pages + Undo/Redo + UND protection + Import/Export/Restore defaults.
    {
        let (mut app, _) = crate::App::boot(1.0);
        let draw_rect_and_define = |app: &mut crate::App, code: &str, name: &str| {
            let _ = app.update(Message::Key(KeyCmd::DrawExterior));
            let _ = app.update(Message::Key(KeyCmd::Enter));
            for dir in [2u8, 4, 6, 0] {
                let _ = app.update(Message::Key(KeyCmd::Buffer("1")));
                let _ = app.update(Message::Key(KeyCmd::Buffer("0")));
                let _ = app.update(Message::Key(KeyCmd::Arrow {
                    dir,
                    ctrl: false,
                    shift: false,
                }));
                let _ = app.update(Message::Key(KeyCmd::Enter));
            }
            assert!(!app.drawing.polygons.is_empty());
            app.drawing.polygons[0].kind = code.to_string();
            app.drawing.polygons[0].name = name.to_string();
        };

        // Page 0: GAR, Page 1: GAR
        draw_rect_and_define(&mut app, "GAR", "Garage");
        let _ = app.update(Message::Status(StatusMsg::PageAdd));
        draw_rect_and_define(&mut app, "GAR", "Garage");

        // Open EditType("GAR") (session = 1), rename to GARX and Save.
        let _ = app.update(Message::AreaWorkflow(crate::area_workflow::Msg::EditType(
            "GAR".into(),
        )));
        assert!(app.ui.type_editor.open);
        assert_eq!(
            app.ui
                .type_editor
                .usage_for_entry(&app.ui.type_editor.drafts[app.ui.type_editor.selected]),
            2
        );
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::Code("GARX".into()))));
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::Save)));
        assert!(!app.ui.type_editor.open);
        assert_eq!(app.pages.pages[0].polygons[0].kind, "GARX");
        assert_eq!(app.drawing.polygons[0].kind, "GARX");

        // Undo reverts BOTH pages back to GAR; Redo restores GARX.
        let _ = app.update(Message::Undo);
        assert!(app.area_codes.iter().any(|c| c.code == "GAR"));
        assert_eq!(app.pages.pages[0].polygons[0].kind, "GAR");
        assert_eq!(app.drawing.polygons[0].kind, "GAR");

        let _ = app.update(Message::Redo);
        assert!(app.area_codes.iter().any(|c| c.code == "GARX"));
        assert_eq!(app.pages.pages[0].polygons[0].kind, "GARX");
        assert_eq!(app.drawing.polygons[0].kind, "GARX");

        // Delete in-use GARX asks confirmation -> becomes UND -> Undo restores GARX + polygons.
        let _ = app.update(Message::AreaWorkflow(crate::area_workflow::Msg::EditType(
            "GARX".into(),
        )));
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::RequestDelete)));
        assert_eq!(
            app.ui.type_editor.confirm,
            Some(crate::ui::type_editor::ConfirmKind::Delete("GARX".into()))
        );
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::ConfirmDelete)));
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::Save)));
        assert!(!app.area_codes.iter().any(|c| c.code == "GARX"));
        assert_eq!(app.pages.pages[0].polygons[0].kind, "UND");
        assert_eq!(app.drawing.polygons[0].kind, "UND");

        let _ = app.update(Message::Undo);
        assert!(app.area_codes.iter().any(|c| c.code == "GARX"));
        assert_eq!(app.pages.pages[0].polygons[0].kind, "GARX");
        assert_eq!(app.drawing.polygons[0].kind, "GARX");

        // Test Export / Import / Restore defaults / UND protection.
        let _ = app.update(Message::AreaWorkflow(crate::area_workflow::Msg::EditType(
            "UND".into(),
        )));
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::RequestDelete)));
        assert!(
            app.ui.type_editor.drafts.iter().any(|d| d.area.code == "UND"),
            "UND must not be deletable"
        );
        let tmp_dir = std::env::temp_dir().join("sketch_rust_p11_io");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let export_path = tmp_dir.join("types.json");
        let session = app.ui.type_editor.session;
        let body = sketch_core::geometry::area_codes::to_json(
            app.ui.type_editor.validated_codes().unwrap(),
        )
        .unwrap();
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::ExportChosen {
            session,
            body,
            path: Some(export_path.clone()),
        })));
        assert!(export_path.exists());
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::ImportChosen(
            session,
            Some(export_path.clone()),
        ))));
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::RequestRestoreDefaults)));
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::ConfirmRestoreDefaults)));
        let _ = app.update(Message::Ui(UiMsg::TypeEditor(TypeMsg::Save)));
        assert_eq!(app.area_codes.len(), 26);
        assert!(app.area_codes.iter().any(|c| c.code == "GAR"));
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    // 4. Screenshots across all 4 viewport sizes (including Compact list -> form).
    for &(label, w, h) in &[
        ("wide", 1400.0, 860.0),
        ("1280x800", 1280.0, 800.0),
        ("tablet", 800.0, 1280.0),
    ] {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        harness.send(Message::OpenCodesEditor);
        assert!(harness.shows("Area types"));
        let shot = harness.screenshot_in("e2e-next", &format!("p11-type-editor-{label}"));
        assert!(shot.exists());
    }
    {
        let mut harness = harness_next_at(iced::Size::new(372.0, 800.0));
        harness.send(Message::OpenCodesEditor);
        assert!(harness.shows("First Floor"));
        let shot_list = harness.screenshot_in("e2e-next", "p11-type-editor-compact-list");
        assert!(shot_list.exists());
    }
    {
        let mut harness = harness_next_at(iced::Size::new(372.0, 800.0));
        harness.send(Message::AreaWorkflow(crate::area_workflow::Msg::EditType(
            "GAR".into(),
        )));
        assert!(harness.shows("< Back"));
        let shot_form = harness.screenshot_in("e2e-next", "p11-type-editor-compact-form");
        assert!(shot_form.exists());
    }
}

#[test]
fn p12_symbol_library_and_shape_editing_across_sizes() {
    use crate::ui::symbol_library::{
        import_dxf_paths, import_library_file, CompactPage, ConfirmAction, Msg as SymMsg,
        ALL_KEY,
    };
    use sketch_core::geometry::Pt;
    use sketch_core::symbols::{WallSnap, world_corners};

    // 1. Missing symbols.original.json leaves original_library_path() empty (disables Restore).
    symlib::with_temp_library_opts(false, || {
        crate::persist::with_temp_store(|| {
            assert!(
                symlib::original_library_path().is_none(),
                "Restore original must be disabled when symbols.original.json is absent"
            );
        });
    });

    // 2. Full functional round-trip on an isolated library + profile store.
    symlib::with_temp_library(|| {
        crate::persist::with_temp_store(|| {
            assert!(symlib::original_library_path().is_some());
            let (mut app, _) = crate::App::boot(1.0);

            // Open via the "symbol-library" command.
            let cmd = crate::ui::commands::find("symbol-library").unwrap();
            let _ = app.update((cmd.run)(&app));
            assert!(app.ui.symbol_library.open);
            assert_eq!(app.symbols_lib().library.symbols.len(), 499);
            assert_eq!(app.symbols_lib().categories.len(), 18);

            let first_sym_id = app.ui.symbol_library.focused.clone().unwrap();

            // Edit every property of the focused built-in symbol and save.
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::EditName(
                "Edited Swing 36".into(),
            ))));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::EditCategory(
                "plumbing".into(),
            ))));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::EditDescription(
                "Updated interior door".into(),
            ))));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::EditWallSnap(
                WallSnap::Center,
            ))));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::EditWallGap(
                "42".into(),
            ))));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::EditWallAlign(
                "6".into(),
            ))));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::EditWallCap(
                "3".into(),
            ))));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::SaveProperties)));

            // Verify persisted in library reload + new App.
            let (reloaded, _) = crate::App::boot(1.0);
            let sym = reloaded.symbols_lib().symbol(&first_sym_id).unwrap();
            assert_eq!(sym.name, "Edited Swing 36");
            assert_eq!(sym.category, "plumbing");
            assert_eq!(sym.description, "Updated interior door");
            assert_eq!(sym.wall_snap, WallSnap::Center);
            assert_eq!(sym.wall_gap_width, 42.0);
            assert_eq!(sym.wall_gap_alignment, 6.0);
            assert_eq!(sym.wall_cap_length, 3.0);

            // Duplicate focused symbol + place an instance on page before reordering.
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::DuplicateFocused)));
            let dup_id = app.ui.symbol_library.focused.clone().unwrap();
            assert_ne!(dup_id, first_sym_id);

            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Close)));
            app.place_symbol_at(Pt::new(2.0, 2.0), dup_id.clone());
            assert_eq!(app.drawing.symbols.len(), 1);

            // Reorder category and move symbol within category; placed instance unchanged.
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Open(Some(
                dup_id.clone(),
            )))));
            let cat0 = app.symbols_lib().categories[0].key.clone();
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::MoveCategoryDown(
                cat0.clone(),
            ))));
            assert_eq!(app.symbols_lib().categories[1].key, cat0);
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::MoveSymbolEarlier(
                dup_id.clone(),
            ))));
            assert_eq!(app.drawing.symbols[0].symbol_type, dup_id);
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Close)));

            // Make symbol from selection: draw a 4ft x 2ft rectangle + text label, convert, place.
            let _ = app.update(Message::Key(KeyCmd::DrawExterior));
            let _ = app.update(Message::Key(KeyCmd::Enter));
            for (len, dir) in [("4", 2u8), ("2", 4), ("4", 6), ("2", 0)] {
                let _ = app.update(Message::Key(KeyCmd::Buffer(len)));
                let _ = app.update(Message::Key(KeyCmd::Arrow {
                    dir,
                    ctrl: false,
                    shift: false,
                }));
                let _ = app.update(Message::Key(KeyCmd::Enter));
            }
            assert!(app.drawing.lines.len() >= 4);
            app.selection.clear();
            for line in &app.drawing.lines {
                app.selection.lines.insert(line.id);
            }
            assert!(crate::symbol_shape::selection_convertible(
                &app.drawing,
                &app.selection
            ));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(
                SymMsg::OpenMakeFromSelection,
            )));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::PromptText(
                "Custom 4x2 Desk".into(),
            ))));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::SubmitPrompt)));
            let desk_id = app.ui.symbol_library.focused.clone().unwrap();
            let desk_sym = app.symbols_lib().symbol(&desk_id).unwrap().clone();
            let corners = world_corners(desk_sym.bounds, 10.0, 10.0, 0.0, 1.0, false);
            let min_x = corners.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
            let max_x = corners.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
            let min_y = corners.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
            let max_y = corners.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
            assert!(((max_x - min_x) - 4.0).abs() < 1e-3);
            assert!(((max_y - min_y) - 2.0).abs() < 1e-3);

            // Shape editing mode: edit geometry + Cancel leaves unchanged, Save updates bounds while keeping page & undo intact!
            let page_lines_before = app.drawing.lines.len();
            let undo_before_can_undo = app.undo.can_undo();
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::EditShapeFocused)));
            assert!(app.symbol_edit_session.is_some());
            assert_eq!(app.drawing.lines.len(), 4);
            app.drawing.lines[0].end.x += 2.0; // widen by 24 inches
            let _ = app.update(Message::CancelSymbolShape);
            assert!(app.symbol_edit_session.is_none());
            assert_eq!(app.drawing.lines.len(), page_lines_before);
            assert_eq!(app.undo.can_undo(), undo_before_can_undo);
            assert!((app.symbols_lib().symbol(&desk_id).unwrap().bounds.max_x - 24.0).abs() < 1e-3);

            // Re-enter shape editing, stretch, and SaveShape.
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Open(Some(
                desk_id.clone(),
            )))));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::EditShapeFocused)));
            app.drawing.lines[0].end.x += 2.0;
            let _ = app.update(Message::SaveSymbolShape);
            assert!(app.symbol_edit_session.is_none());
            assert_eq!(app.drawing.lines.len(), page_lines_before);
            assert_eq!(app.undo.can_undo(), undo_before_can_undo);
            let updated_bounds = app.symbols_lib().symbol(&desk_id).unwrap().bounds;
            assert!((updated_bounds.max_x - updated_bounds.min_x - 72.0).abs() < 1e-3);

            // DXF import + subset export + library import with conflict reporting.
            let tmp_dir = std::env::temp_dir().join("sketch_rust_p12_io");
            let _ = std::fs::create_dir_all(&tmp_dir);
            let dxf_path = tmp_dir.join("bench.dxf");
            std::fs::write(
                &dxf_path,
                "0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n36\n21\n0\n0\nENDSEC\n0\nEOF\n",
            )
            .unwrap();
            let _ = import_dxf_paths(&mut app, &[dxf_path]);
            assert!(app
                .symbols_lib()
                .library
                .symbols
                .values()
                .any(|s| s.name.eq_ignore_ascii_case("bench")));

            let export_path = tmp_dir.join("subset.json");
            let subset = std::collections::BTreeSet::from([desk_id.clone()]);
            let exported = sketch_core::symbol_edit::export(
                &symlib::library_path(),
                &export_path,
                Some(&subset),
            )
            .unwrap();
            assert_eq!(exported, 1);

            // Re-importing same subset reports conflicts cleanly (0 added).
            let _ = import_library_file(&mut app, &export_path);
            let report = sketch_core::symbol_edit::import_with_report(
                &symlib::library_path(),
                &export_path,
            )
            .unwrap();
            assert_eq!(report.added, 0);
            assert!(report.conflicts.contains(&desk_id));
            let _ = std::fs::remove_dir_all(&tmp_dir);

            // Delete a category (writes symbols.json.bak) then Restore original library -> 499 symbols.
            let del_cat = app.symbols_lib().categories[0].key.clone();
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Open(None))));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(
                SymMsg::RequestDeleteCategory(del_cat.clone()),
            )));
            assert_eq!(
                app.ui.symbol_library.confirm,
                Some(ConfirmAction::DeleteCategory(del_cat.clone()))
            );
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Confirm)));
            assert!(!app.symbols_lib().categories.iter().any(|c| c.key == del_cat));
            assert!(symlib::library_path().with_file_name("symbols.json.bak").exists());

            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(
                SymMsg::RequestRestoreOriginal,
            )));
            let _ = app.update(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Confirm)));
            assert_eq!(app.symbols_lib().library.symbols.len(), 499);
            assert_eq!(app.symbols_lib().categories.len(), 18);
            assert_eq!(app.ui.symbol_library.category, ALL_KEY);
        });
    });

    // 3. Visual screenshots across all 4 viewport sizes + Medium editor slide-over + Shape edit mode.
    for &(label, w, h) in &[
        ("wide", 1400.0, 860.0),
        ("1280x800", 1280.0, 800.0),
        ("tablet", 800.0, 1280.0),
    ] {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        harness.send(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Open(None))));
        assert!(harness.shows("Symbol library"));
        let shot = harness.screenshot_in("e2e-next", &format!("p12-symbol-library-{label}"));
        assert!(shot.exists());
    }
    {
        let mut harness = harness_next_at(iced::Size::new(800.0, 1280.0));
        harness.send(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Open(None))));
        harness.send(Message::Ui(UiMsg::SymbolLibrary(SymMsg::FocusSymbol(
            "caddillo/door-swing-36".into(),
        ))));
        assert!(harness.shows("Edit symbol"));
        let shot = harness.screenshot_in("e2e-next", "p12-symbol-library-tablet-editor");
        assert!(shot.exists());
    }
    {
        let mut harness = harness_next_at(iced::Size::new(372.0, 800.0));
        harness.send(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Open(None))));
        let shot = harness.screenshot_in("e2e-next", "p12-symbol-library-compact-cats");
        assert!(shot.exists());
    }
    {
        let mut harness = harness_next_at(iced::Size::new(372.0, 800.0));
        harness.send(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Open(None))));
        harness.send(Message::Ui(UiMsg::SymbolLibrary(SymMsg::SelectCategory(
            ALL_KEY.into(),
        ))));
        let shot = harness.screenshot_in("e2e-next", "p12-symbol-library-compact-grid");
        assert!(shot.exists());
        assert_eq!(
            state(harness).ui.symbol_library.compact_page,
            CompactPage::Grid
        );
    }
    {
        let mut harness = harness_next_at(iced::Size::new(372.0, 800.0));
        harness.send(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Open(Some(
            "caddillo/door-swing-36".into(),
        )))));
        assert!(harness.shows("Edit shape"));
        let shot = harness.screenshot_in("e2e-next", "p12-symbol-library-compact-editor");
        assert!(shot.exists());
    }
    {
        let mut harness = harness_next_at(iced::Size::new(1400.0, 860.0));
        harness.send(Message::Ui(UiMsg::SymbolLibrary(SymMsg::Open(Some(
            "caddillo/door-swing-36".into(),
        )))));
        harness.send(Message::Ui(UiMsg::SymbolLibrary(SymMsg::EditShapeFocused)));
        assert!(harness.shows("Save shape"));
        let shot = harness.screenshot_in("e2e-next", "p12-symbol-shape-edit-wide");
        assert!(shot.exists());
    }
}

#[test]
fn p13_start_screen_dialogs_and_guide_across_sizes() {
    use crate::attr_dialog::AttrMsg;
    use crate::color_dialog::{ColorMsg, Tab as ColorTab};
    use crate::guide::{GuideMsg, INTEGRATION_TOPIC, TOPICS};
    use crate::options_dialog::{OptMsg, Tab as OptTab};
    use crate::page_layout::LayoutMsg;
    use crate::recent::RecentFile;
    use crate::shortcuts::{ShortcutsMsg, GROUPS as SHORTCUT_GROUPS};
    use crate::subject_info::{SubjectMsg, Tab as SubjectTab};
    use crate::text_manager::TextManagerMsg;
    use crate::ui::start::{Msg as StartMsg, ReportStatus, StartState};

    // 1. Vocabulary rewrite verification: Guide & Shortcuts must use Next shell terms.
    for &(group_title, entries) in SHORTCUT_GROUPS {
        for &(keys, action) in entries {
            let joined = format!("{group_title} {keys} {action}");
            assert!(
                !joined.contains(concat!("Rib", "bon"))
                    && !joined.contains(concat!("rib", "bon"))
                    && !joined.contains("Dock")
                    && !joined.contains("dock"),
                "Shortcuts still reference Classic vocabulary: {joined}"
            );
        }
    }
    let guide_text_all: String = TOPICS.iter().map(|t| t.title).collect::<Vec<_>>().join(" ");
    assert!(guide_text_all.contains("Appraise-It reports"));
    assert!(TOPICS.iter().any(|t| t.title == "Getting started"));

    // 2. Recent sketch outline thumbnail extraction + StartState actions.
    let sample_sketch = serde_json::json!({
        "pages": [{
            "lines": [
                {"x1": 0.0, "y1": 0.0, "x2": 200.0, "y2": 0.0},
                {"x1": 200.0, "y1": 0.0, "x2": 200.0, "y2": 150.0},
                {"x1": 200.0, "y1": 150.0, "x2": 0.0, "y2": 150.0},
                {"x1": 0.0, "y1": 150.0, "x2": 0.0, "y2": 0.0},
            ],
            "polygons": [{
                "type": "GLA1",
                "area": 300.0,
                "points": [
                    {"x": 0.0, "y": 0.0},
                    {"x": 200.0, "y": 0.0},
                    {"x": 200.0, "y": 150.0},
                    {"x": 0.0, "y": 150.0}
                ]
            }]
        }]
    })
    .to_string();
    let rec = RecentFile {
        name: "Maple St.skx".into(),
        last_opened: "2026-11-15T12:00:00Z".into(),
        data: sample_sketch,
        path: None,
    };
    let mut start_state = StartState::default();
    start_state.sync_thumbs(std::slice::from_ref(&rec));
    let thumb = start_state.thumb_for(&rec);
    assert!(thumb.has_geometry);
    assert_eq!(thumb.lines.len(), 4);
    assert_eq!(thumb.polys.len(), 1);
    assert_eq!(thumb.living_sf, Some(300.0));

    // 3. Start screen across all 4 viewport sizes (empty recent + Appraise-It open report CTA).
    for &(label, w, h, _) in SIZES {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        harness.send(Message::Ui(UiMsg::Start(StartMsg::Open)));
        harness.send(Message::Ui(UiMsg::Start(StartMsg::ReportChecked(
            ReportStatus::Open("124 Maple St · File #441".into()),
        ))));
        assert!(harness.shows("Start a sketch"), "start screen header missing at {label}");
        assert!(harness.shows("New sketch"));
        assert!(harness.shows("Draw the sketch for this report"));
        assert!(harness.shows("Recent sketches"));
        let shot = harness.screenshot_in("e2e-next", &format!("p13-start-{label}"));
        assert!(shot.exists());
    }

    // Clicking "New sketch" closes Start screen.
    {
        let mut harness = harness_next_at(VIEWPORT);
        harness.send(Message::Ui(UiMsg::Start(StartMsg::Open)));
        assert!(harness.shows("Start a sketch"));
        harness.send(Message::Ui(UiMsg::Start(StartMsg::NewSketch)));
        assert!(!state(harness).ui.start.open);
    }

    // 4. Preferences (Options) all 6 tabs across Wide + Compact, plus Restore Defaults.
    for &(label, w, h) in &[("wide", 1400.0, 860.0), ("compact", 372.0, 800.0)] {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        harness.send(Message::Options(OptMsg::Open));
        assert!(harness.shows("Preferences"));
        for tab in OptTab::ALL {
            harness.send(Message::Options(OptMsg::Tab(*tab)));
        }
        harness.send(Message::Options(OptMsg::Tab(OptTab::General)));
        let shot = harness.screenshot_in("e2e-next", &format!("p13-options-{label}"));
        assert!(shot.exists());
    }

    // 5. Property Details (Subject Info) all 3 tabs across Wide + Tablet.
    for &(label, w, h) in &[("wide", 1400.0, 860.0), ("tablet", 800.0, 1280.0)] {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        harness.send(Message::OpenSubjectInfo);
        assert!(harness.shows("Property Details"));
        harness.send(Message::SubjectInfo(SubjectMsg::Tab(SubjectTab::Comments)));
        assert!(harness.shows("Save Default"));
        harness.send(Message::SubjectInfo(SubjectMsg::Tab(SubjectTab::Tables)));
        assert!(harness.shows("Comment Table 1"));
        harness.send(Message::SubjectInfo(SubjectMsg::Tab(SubjectTab::Parcel)));
        let shot = harness.screenshot_in("e2e-next", &format!("p13-subject-{label}"));
        assert!(shot.exists());
    }

    // 6. User Guide + Appraise-It report integration across Wide + Compact.
    for &(label, w, h) in &[("wide", 1400.0, 860.0), ("compact", 372.0, 800.0)] {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        harness.send(Message::Guide(GuideMsg::Open(0)));
        assert!(harness.shows("User Guide"));
        assert!(harness.shows("Getting started"));
        harness.send(Message::Guide(GuideMsg::Topic(INTEGRATION_TOPIC)));
        assert!(harness.shows("Appraise-It reports"));
        let shot = harness.screenshot_in("e2e-next", &format!("p13-guide-{label}"));
        assert!(shot.exists());
    }

    // 7. Keyboard Shortcuts across Wide + Compact.
    for &(label, w, h) in &[("wide", 1400.0, 860.0), ("compact", 372.0, 800.0)] {
        let mut harness = harness_next_at(iced::Size::new(w, h));
        harness.send(Message::Shortcuts(ShortcutsMsg::Open));
        assert!(harness.shows("Keyboard Shortcuts"));
        assert!(harness.shows("Ctrl + K"));
        let shot = harness.screenshot_in("e2e-next", &format!("p13-shortcuts-{label}"));
        assert!(shot.exists());
    }

    // 8. Remaining dialogs open cleanly across size classes: Page Layout, Color, Room Shapes,
    // Room Labels, Area Attributes, Breakdown, Rename Prompt, Undo History, Geo Menu.
    {
        let mut harness = harness_next_at(VIEWPORT);
        harness.send(Message::PageLayout(LayoutMsg::Open));
        assert!(harness.shows("Page Layout"));
        let shot = harness.screenshot_in("e2e-next", "p13-pagelayout-wide");
        assert!(shot.exists());
    }
    {
        let mut harness = harness_next_at(VIEWPORT);
        harness.send(Message::Color(ColorMsg::Open("#3182ce".into())));
        harness.send(Message::Color(ColorMsg::Tab(ColorTab::Custom)));
        assert!(harness.shows("Colour"));
        let shot = harness.screenshot_in("e2e-next", "p13-color-wide");
        assert!(shot.exists());
    }
    {
        let mut harness = harness_next_at(VIEWPORT);
        harness.send(Message::Key(KeyCmd::DrawExterior));
        harness.send(Message::Key(KeyCmd::Enter));
        for dir in [2, 4, 6, 0] {
            harness.send(Message::Key(KeyCmd::Buffer("1")));
            harness.send(Message::Key(KeyCmd::Buffer("0")));
            harness.send(Message::Key(KeyCmd::Arrow {
                dir,
                ctrl: false,
                shift: false,
            }));
            harness.send(Message::Key(KeyCmd::Enter));
        }
        harness.send(Message::Key(KeyCmd::SelectAll));
        harness.send(Message::AreaWorkflow(crate::area_workflow::Msg::AssignSelected(
            "GLA1".into(),
        )));
        harness.send(Message::OpenBreakdown);
        assert!(harness.shows("Breakdown Details"));
        let shot = harness.screenshot_in("e2e-next", "p13-breakdown-wide");
        assert!(shot.exists());
    }
    {
        let mut harness = harness_next_at(iced::Size::new(372.0, 800.0));
        harness.send(Message::Key(KeyCmd::DrawExterior));
        harness.send(Message::Key(KeyCmd::Enter));
        for dir in [2, 4, 6, 0] {
            harness.send(Message::Key(KeyCmd::Buffer("1")));
            harness.send(Message::Key(KeyCmd::Buffer("0")));
            harness.send(Message::Key(KeyCmd::Arrow {
                dir,
                ctrl: false,
                shift: false,
            }));
            harness.send(Message::Key(KeyCmd::Enter));
        }
        harness.send(Message::Key(KeyCmd::SelectAll));
        harness.send(Message::AreaWorkflow(crate::area_workflow::Msg::AssignSelected(
            "GLA1".into(),
        )));

        harness.send(Message::RoomShapeOpen);
        assert!(harness.shows("Insert Room Shape"));
        harness.send(Message::RoomShapeCancel);
        assert!(!harness.shows("Insert Room Shape"));

        harness.send(Message::TextLibraryManager(TextManagerMsg::Open));
        assert!(harness.shows("Room Labels"));
        harness.send(Message::TextLibraryManager(TextManagerMsg::Close));

        harness.send(Message::OpenAreaAttributes(true));
        assert!(harness.shows("Area Attributes"));
        harness.send(Message::AreaAttributes(AttrMsg::Cancel));

        harness.send(Message::Ui(UiMsg::PromptRenameDocument));
        assert!(harness.shows("Document name:"));
        let shot = harness.screenshot_in("e2e-next", "p13-prompt-compact");
        assert!(shot.exists());
    }
}

#[test]
fn p14_coverage_audit_themes_and_menu_workflows_across_sizes() {
    use crate::canvas_theme::CanvasTheme;
    use crate::disto::DistoMsg;
    use crate::theme::InterfaceTheme;
    use crate::ui::menu::MenuSub;

    // 1. Live Menu submenus + Theme switching (Light / Blue / Dark) & screenshots at 1400x860.
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Key(KeyCmd::DrawExterior));
        h.send(Message::Key(KeyCmd::Enter));
        for dir in [2, 4, 6, 0] {
            h.send(Message::Key(KeyCmd::Buffer("1")));
            h.send(Message::Key(KeyCmd::Buffer("0")));
            h.send(Message::Key(KeyCmd::Arrow {
                dir,
                ctrl: false,
                shift: false,
            }));
            h.send(Message::Key(KeyCmd::Enter));
        }

        // Open Undo History modal & close.
        h.send(Message::UndoHistory);
        assert!(h.shows("Undo History"));
        h.send(Message::UndoHistory);

        // Open Visual Area Type Editor via OpenCodesEditor, NewCode dialog, and Disto picker.
        h.send(Message::OpenCodesEditor);
        assert!(h.shows("Area types"));
        h.send(Message::Ui(UiMsg::CloseOverlays));

        h.send(Message::OpenNewCode);
        assert!(h.shows("Area types"));
        h.send(Message::Ui(UiMsg::CloseOverlays));

        h.send(Message::OpenDefEditor);
        assert!(h.shows("Area types"));
        h.send(Message::Ui(UiMsg::CloseOverlays));
        assert!(!h.shows("Area types"));

        h.send(Message::Disto(DistoMsg::Connect));
        assert!(h.shows("Select Bluetooth Laser"));
        h.send(Message::Disto(DistoMsg::Cancel));

        // Geo Coordinate prompt opens cleanly & closes.
        h.send(Message::GeoCoordinate);
        assert!(h.shows("Cancel"));
        h.send(Message::Dialog(crate::dialog::DialogMsg::Cancel));

        // Menu > View > Interface theme -> Light + Canvas Light.
        h.send(Message::PopupMenu(PopupMsg::Interface(InterfaceTheme::Light)));
        h.send(Message::PopupMenu(PopupMsg::Canvas(CanvasTheme::Light)));
        assert_eq!(crate::theme::interface_theme(), InterfaceTheme::Light);
        let shot_light = h.screenshot_in("e2e-next", "p14-theme-light-wide");
        assert!(shot_light.exists());
    }
    {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::Key(KeyCmd::DrawExterior));
        h.send(Message::Key(KeyCmd::Enter));
        for dir in [2, 4, 6, 0] {
            h.send(Message::Key(KeyCmd::Buffer("1")));
            h.send(Message::Key(KeyCmd::Buffer("0")));
            h.send(Message::Key(KeyCmd::Arrow {
                dir,
                ctrl: false,
                shift: false,
            }));
            h.send(Message::Key(KeyCmd::Enter));
        }
        h.send(Message::PopupMenu(PopupMsg::Interface(InterfaceTheme::Blue)));
        h.send(Message::PopupMenu(PopupMsg::Canvas(CanvasTheme::Blue)));
        assert_eq!(crate::theme::interface_theme(), InterfaceTheme::Blue);
        let shot_blue = h.screenshot_in("e2e-next", "p14-theme-blue-wide");
        assert!(shot_blue.exists());

        // Restore default Dark UI + Light canvas before finishing.
        h.send(Message::PopupMenu(PopupMsg::Interface(InterfaceTheme::Dark)));
        h.send(Message::PopupMenu(PopupMsg::Canvas(CanvasTheme::Light)));
    }

    // 2. Menu submenus, filters, page underlay, sketch transforms across all 4 viewport sizes.
    for (size, label) in [
        (iced::Size::new(1400.0, 860.0), "1400x860"),
        (iced::Size::new(1280.0, 800.0), "1280x800"),
        (iced::Size::new(800.0, 1280.0), "800x1280"),
        (iced::Size::new(372.0, 800.0), "372x800"),
    ] {
        let mut h = harness_next_at(size);
        h.send(Message::Key(KeyCmd::DrawExterior));
        h.send(Message::Key(KeyCmd::Enter));
        for dir in [2, 4, 6, 0] {
            h.send(Message::Key(KeyCmd::Buffer("1")));
            h.send(Message::Key(KeyCmd::Buffer("0")));
            h.send(Message::Key(KeyCmd::Arrow {
                dir,
                ctrl: false,
                shift: false,
            }));
            h.send(Message::Key(KeyCmd::Enter));
        }

        // Verify the Select fly-out, filters, pitch, units, flip and underlay at every size.
        h.send(Message::Ui(UiMsg::ToggleMenu));
        h.send(Message::Ui(UiMsg::MenuSub(Some(MenuSub::Select))));
        assert!(h.shows("Everything"), "Select fly-out missing at {label}");
        h.send(Message::Ui(UiMsg::MenuRunId("select-all")));
        {
            let app = state(h);
            assert!(
                !app.selection.lines.is_empty(),
                "Select > Everything should select walls at {label}"
            );
            h = harness_next_at(size);
        }

        h.send(Message::Status(crate::statusbar::StatusMsg::Filter(0)));
        h.send(Message::Ui(UiMsg::MenuRunId("pitch-half")));
        h.send(Message::Ui(UiMsg::MenuRunId("units-metric")));
        h.send(Message::Ui(UiMsg::MenuRunId("units-imperial")));
        h.send(Message::RotateSketchBy(90.0));
        h.send(Message::RotateSketchBy(-90.0));
        h.send(Message::Ui(UiMsg::MenuRunId("flip-horizontal")));
        h.send(Message::Status(crate::statusbar::StatusMsg::PageAdd));
        h.send(Message::Status(crate::statusbar::StatusMsg::Underlay(Some(0))));

        let app = state(h);
        assert_eq!(app.pages.pages.len(), 2, "second page added at {label}");
        assert_eq!(app.pages.underlay, Some(0), "underlay active at {label}");
    }
}

/// Canvas-local screen point `t` of the way along the first wall of the
/// closed square, plus the pixel length of one foot.
fn drawn_square_wall_point(w: f32, h: f32, t: f64) -> ((f32, f32), f32) {
    let mut tmp = harness_next_at(iced::Size::new(w, h));
    draw_closed_square(&mut tmp);
    let app = state(tmp);
    let line = &app.drawing.lines[0];
    let wx = line.start.x + (line.end.x - line.start.x) * t;
    let wy = line.start.y + (line.end.y - line.start.y) * t;
    let (sx, sy) = app.view.to_screen(crate::view_state::World { x: wx, y: wy });
    let (ox, _) = app.view.to_screen(crate::view_state::World { x: wx + 1.0, y: wy });
    ((sx as f32, sy as f32), (ox - sx).abs() as f32)
}

fn ring_area(points: &[sketch_core::geometry::Pt]) -> f64 {
    let n = points.len();
    let twice: f64 = (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum();
    (twice / 2.0).abs()
}

/// Grabbing a wall in Select and dragging it stretches its neighbours and
/// resizes the area: on a fresh press, and on a wall that is already selected.
#[test]
fn select_drag_on_a_wall_resizes_the_area_across_sizes() {
    for &(label, w, h, _) in SIZES {
        let ((x, y), foot) = drawn_square_wall_point(w, h, 0.25);
        for preselect in [false, true] {
            let mut h_app = harness_next_at(iced::Size::new(w, h));
            draw_closed_square(&mut h_app);
            h_app.send(Message::Key(KeyCmd::Select));
            if preselect {
                // Click the wall first, as a user does, then grab it.
                h_app.send(Message::CanvasPress(x, y));
                h_app.send(Message::CanvasDragEnd);
            }
            h_app.send(Message::CanvasPress(x, y));
            h_app.send(Message::CanvasDrag(x, y + foot, false));
            h_app.send(Message::CanvasDrag(x, y + 1.8 * foot, false));
            h_app.send(Message::CanvasDragEnd);
            let app = state(h_app);
            assert_eq!(app.drawing.lines.len(), 4, "four walls remain at {label}");
            assert_eq!(app.drawing.polygons.len(), 1, "one area remains at {label}");
            let area = ring_area(&app.drawing.polygons[0].points);
            assert!(
                (area - 120.0).abs() < 0.01 || (area - 80.0).abs() < 0.01,
                "dragging a wall 2 ft resizes the 100 sf area at {label} \
                 (preselect {preselect}): got {area}"
            );
        }
    }
}

/// Dragging the endpoint dot of a selected wall moves that corner, and the
/// walls that meet there follow it.
#[test]
fn select_drag_on_a_wall_endpoint_moves_the_corner_across_sizes() {
    for &(label, w, h, _) in SIZES {
        let ((qx, qy), foot) = drawn_square_wall_point(w, h, 0.25);
        let ((ex, ey), _) = drawn_square_wall_point(w, h, 0.0);
        let mut h_app = harness_next_at(iced::Size::new(w, h));
        draw_closed_square(&mut h_app);
        h_app.send(Message::Key(KeyCmd::Select));
        h_app.send(Message::CanvasPress(qx, qy));
        h_app.send(Message::CanvasDragEnd);
        h_app.send(Message::CanvasPress(ex, ey));
        h_app.send(Message::CanvasDrag(ex, ey + foot, false));
        h_app.send(Message::CanvasDrag(ex, ey + 1.8 * foot, false));
        h_app.send(Message::CanvasDragEnd);
        let app = state(h_app);
        assert_eq!(app.drawing.lines.len(), 4, "four walls remain at {label}");
        assert_eq!(app.drawing.polygons.len(), 1, "one area remains at {label}");
        let area = ring_area(&app.drawing.polygons[0].points);
        assert!(
            ((area - 100.0).abs() - 10.0).abs() < 0.01,
            "moving a corner 2 ft changes the 100 sf area by 10 sf at {label}: got {area}"
        );
    }
}

#[test]
fn selected_area_corner_moves_connected_walls_and_can_be_undone() {
    use sketch_core::geometry::Pt;

    let (x, y) = drawn_square_screen(VIEWPORT.width, VIEWPORT.height, |app| {
        let corner = app.drawing.polygons[0].points[0];
        (corner.x, corner.y)
    });
    let (cx, cy) = drawn_square_center_on_screen(VIEWPORT.width, VIEWPORT.height);
    for snap in [true, false] {
        let mut harness = harness_next_at(VIEWPORT);
        draw_closed_square(&mut harness);
        harness.script(&["click \"Select\"", &format!("click ({cx:.1}, {cy:.1})")]);
        harness.send(Message::Status(StatusMsg::Pitch(crate::statusbar::PitchOption {
            label: "1/2'",
            value: 0.5,
        })));
        if !snap {
            harness.send(Message::Status(StatusMsg::ToggleSnap));
        }
        harness.send(Message::CanvasPress(x, y));
        harness.send(Message::CanvasDrag(x + 24.0, y - 12.0, false));
        harness.send(Message::CanvasDragEnd);

        let mut app = state(harness);
        let area_id = app.drawing.polygons[0].id;
        assert!(app.selection.is_poly_selected(area_id));
        let moved = app.drawing.polygons[0].points[0];
        assert_eq!(
            app.drawing.lines.iter().filter(|wall| wall.start == moved || wall.end == moved).count(),
            2,
            "both walls should follow the corner"
        );
        let _ = app.update(Message::Undo);
        let original = app.drawing.polygons[0].points[0];
        let k = sketch_core::geometry::PIXELS_PER_FOOT * app.view.zoom;
        let (dx, dy) = if snap { (1.0, 0.5) } else { (24.0 / k, 12.0 / k) };
        let expected = Pt::new(original.x + dx, original.y + dy);
        assert!(sketch_core::geometry::distance(moved, expected) < 1e-5);
    }
}

/// "Select more" replaces holding Ctrl, so the pill is for touch only.
#[test]
fn select_more_pill_is_only_on_screen_for_touch() {
    let mut h = harness_next_at(VIEWPORT);
    draw_closed_square(&mut h);
    h.send(Message::Rail(crate::view_state::RailTool::Select));
    assert!(!h.shows("Select more"), "the pill must not show with a mouse");
    h.send(Message::Ui(UiMsg::TouchDetected));
    assert!(h.shows("Select more"), "the pill must show for touch");
}

/// With Snap on, dragging a wall's end handle and moving a selection both land
/// on the grid, at every pitch; with Snap off they do not.
#[test]
fn wall_end_drags_and_selection_moves_land_on_the_grid_when_snap_is_on() {
    let on_grid = |v: f64, pitch: f64| ((v / pitch).round() * pitch - v).abs() < 1e-6;
    for (label, pitch) in [("1'", 1.0), ("1/2'", 0.5), ("1\"", 1.0 / 12.0)] {
        for snap in [true, false] {
            let pick = crate::statusbar::PitchOption { label, value: pitch };
            let prepare = || {
                let mut h = harness_next_at(VIEWPORT);
                draw_closed_square(&mut h);
                h.send(Message::Key(KeyCmd::Select));
                h.send(Message::Status(StatusMsg::Pitch(pick.clone())));
                if !snap {
                    h.send(Message::Status(StatusMsg::ToggleSnap));
                }
                h
            };

            // Wall end handle: select the wall, then drag its start handle.
            let ((mx, my), _) = drawn_square_wall_point(VIEWPORT.width, VIEWPORT.height, 0.5);
            let ((sx, sy), _) = drawn_square_wall_point(VIEWPORT.width, VIEWPORT.height, 0.0);
            let mut h = prepare();
            h.send(Message::CanvasPress(mx, my));
            h.send(Message::CanvasDragEnd);
            h.send(Message::CanvasPress(sx, sy));
            h.send(Message::CanvasDrag(sx + 23.3, sy - 11.7, false));
            h.send(Message::CanvasDragEnd);
            let app = state(h);
            let p = app.drawing.lines[0].start;
            assert_eq!(
                on_grid(p.x, pitch) && on_grid(p.y, pitch),
                snap,
                "wall end at pitch {label}, snap {snap}: ({}, {})",
                p.x,
                p.y
            );

            // Selection move: press inside the area, drag by an off-grid amount.
            let (cx, cy) = {
                let mut tmp = harness_next_at(VIEWPORT);
                draw_closed_square(&mut tmp);
                let app = state(tmp);
                let poly = &app.drawing.polygons[0];
                let n = poly.points.len() as f64;
                let wx = poly.points.iter().map(|p| p.x).sum::<f64>() / n;
                let wy = poly.points.iter().map(|p| p.y).sum::<f64>() / n;
                let (x, y) = app.view.to_screen(crate::view_state::World { x: wx, y: wy });
                (x as f32, y as f32)
            };
            let mut h = prepare();
            h.send(Message::CanvasPress(cx, cy));
            h.send(Message::CanvasDrag(cx + 10.0, cy - 5.0, false));
            h.send(Message::CanvasDrag(cx + 23.3, cy - 11.7, false));
            h.send(Message::CanvasDragEnd);
            let app = state(h);
            let pts = &app.drawing.polygons[0].points;
            let min_x = pts.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
            let min_y = pts.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
            assert_eq!(
                on_grid(min_x, pitch) && on_grid(min_y, pitch),
                snap,
                "moved area origin at pitch {label}, snap {snap}: ({min_x}, {min_y})"
            );
        }
    }
}

/// Canvas-local screen position of a world point of the closed square.
fn drawn_square_screen(w: f32, h: f32, pick: impl Fn(&crate::App) -> (f64, f64)) -> (f32, f32) {
    let mut tmp = harness_next_at(iced::Size::new(w, h));
    draw_closed_square(&mut tmp);
    let app = state(tmp);
    let (x, y) = pick(&app);
    let (sx, sy) = app.view.to_screen(crate::view_state::World { x, y });
    (sx as f32, sy as f32)
}

/// Dragging the middle handle of a selected wall bows it, and the area follows.
#[test]
fn select_drag_on_the_curve_handle_bows_the_wall_across_sizes() {
    for &(label, w, h, _) in SIZES {
        let ((qx, qy), foot) = drawn_square_wall_point(w, h, 0.25);
        let ((mx, my), _) = drawn_square_wall_point(w, h, 0.5);
        let mut h_app = harness_next_at(iced::Size::new(w, h));
        draw_closed_square(&mut h_app);
        h_app.send(Message::Key(KeyCmd::Select));
        h_app.send(Message::CanvasPress(qx, qy));
        h_app.send(Message::CanvasDragEnd);
        h_app.send(Message::CanvasPress(mx, my));
        h_app.send(Message::CanvasDrag(mx, my + foot, false));
        h_app.send(Message::CanvasDrag(mx, my + 1.8 * foot, false));
        h_app.send(Message::CanvasDragEnd);
        let app = state(h_app);
        let wall = &app.drawing.lines[0];
        assert!(
            (crate::drawing::arc_height_of(wall).abs() - 2.0).abs() < 0.01,
            "the curve height snaps to 2 ft at {label}"
        );
        assert!(
            (wall.end.x - wall.start.x).hypot(wall.end.y - wall.start.y) > 9.99,
            "the wall's ends stay put at {label}"
        );
    }
}

/// Dragging from empty canvas across the sketch selects what the rectangle encloses.
#[test]
fn select_drag_on_empty_canvas_marquee_selects_across_sizes() {
    for &(label, w, h, _) in SIZES {
        let bounds = |app: &crate::App, pad: f64, hi: bool| {
            let pts = &app.drawing.polygons[0].points;
            let pick = |f: fn(f64, f64) -> f64, init: f64, get: fn(&sketch_core::geometry::Pt) -> f64| {
                pts.iter().map(get).fold(init, f)
            };
            if hi {
                (
                    pick(f64::max, f64::NEG_INFINITY, |p| p.x) + pad,
                    pick(f64::max, f64::NEG_INFINITY, |p| p.y) + pad,
                )
            } else {
                (
                    pick(f64::min, f64::INFINITY, |p| p.x) - pad,
                    pick(f64::min, f64::INFINITY, |p| p.y) - pad,
                )
            }
        };
        let (ax, ay) = drawn_square_screen(w, h, |app| bounds(app, 1.5, false));
        let (bx, by) = drawn_square_screen(w, h, |app| bounds(app, 1.5, true));
        let mut h_app = harness_next_at(iced::Size::new(w, h));
        draw_closed_square(&mut h_app);
        h_app.send(Message::Key(KeyCmd::Select));
        h_app.send(Message::CanvasPress(ax, ay));
        h_app.send(Message::CanvasDrag((ax + bx) / 2.0, (ay + by) / 2.0, false));
        h_app.send(Message::CanvasDrag(bx, by, false));
        h_app.send(Message::CanvasDragEnd);
        let app = state(h_app);
        assert_eq!(app.selection.lines.len(), 4, "marquee selects the four walls at {label}");
        assert!(app.view.fence.is_none(), "marquee ends at {label}");
    }
}

/// Dragging the rotation handle of a selected area turns it in 15 degree steps.
#[test]
fn select_drag_on_the_rotate_handle_turns_the_area_across_sizes() {
    for &(label, w, h, _) in SIZES {
        let centre = |app: &crate::App| {
            let pts = &app.drawing.polygons[0].points;
            let n = pts.len() as f64;
            (
                pts.iter().map(|p| p.x).sum::<f64>() / n,
                pts.iter().map(|p| p.y).sum::<f64>() / n,
            )
        };
        let (cx, cy) = drawn_square_screen(w, h, centre);
        // Where the handle sits once the area is selected.
        let (hx, hy, pcx, pcy) = {
            let mut tmp = harness_next_at(iced::Size::new(w, h));
            draw_closed_square(&mut tmp);
            tmp.send(Message::Key(KeyCmd::Select));
            tmp.send(Message::CanvasPress(cx, cy));
            tmp.send(Message::CanvasDragEnd);
            let app = state(tmp);
            let area = crate::handles::selected_area_box(
                &app.drawing,
                &app.selection,
                app.view.pan,
                app.view.zoom,
            )
            .expect("a selected area shows its rotation handle");
            let (px, py) = app.view.to_screen(crate::view_state::World {
                x: area.center_world.x,
                y: area.center_world.y,
            });
            (area.handle.0 as f32, area.handle.1 as f32, px as f32, py as f32)
        };
        let (dx, dy) = (hx - pcx, hy - pcy);
        let (s, c) = std::f32::consts::FRAC_PI_4.sin_cos();
        let (tx, ty) = (pcx + dx * c - dy * s, pcy + dx * s + dy * c);

        let mut h_app = harness_next_at(iced::Size::new(w, h));
        draw_closed_square(&mut h_app);
        h_app.send(Message::Key(KeyCmd::Select));
        h_app.send(Message::CanvasPress(cx, cy));
        h_app.send(Message::CanvasDragEnd);
        h_app.send(Message::CanvasPress(hx, hy));
        h_app.send(Message::CanvasDrag((hx + tx) / 2.0, (hy + ty) / 2.0, false));
        h_app.send(Message::CanvasDrag(tx, ty, false));
        h_app.send(Message::CanvasDragEnd);
        let app = state(h_app);
        let pts = &app.drawing.polygons[0].points;
        let width = pts.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max)
            - pts.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        assert!(
            (width - 200.0_f64.sqrt()).abs() < 0.05,
            "a 45 degree turn makes the 10 ft square 14.14 ft wide at {label}: got {width}"
        );
        assert!((ring_area(pts) - 100.0).abs() < 0.01, "area is kept at {label}");
    }
}

/// Capture the unified area and wall inspector at desktop and tablet sizes.
#[test]
fn unified_inspector_renders_on_desktop_and_tablet() {
    for (name, w, h, touch) in [("wide", 1400.0, 860.0, false), ("tablet", 1280.0, 800.0, true)] {
        let (cx, cy) = drawn_square_center_on_screen(w, h);
        let (wx, wy) = drawn_square_wall_mid_on_screen(w, h);
        for (kind, x, y) in [("area", cx, cy), ("wall", wx, wy)] {
            let mut app = harness_next_at(iced::Size::new(w, h));
            if touch {
                app.send(Message::Ui(UiMsg::SetDensity(Density::Touch)));
            }
            draw_closed_square(&mut app);
            app.script(&["click \"Select\"", &format!("click ({x:.1}, {y:.1})")]);
            let shot = app.screenshot_in("e2e-next", &format!("inspector-{kind}-{name}"));
            assert!(shot.exists());
        }
    }
}

/// Every dialog, and each of its tabs, fits a 1920 x 1080 window without its
/// form scrolling (owner requirement, 2026-11-17). Lists of the user's own
/// data may scroll inside their own box; those scroll areas carry an id that
/// ends in "-list" and are skipped. Scroll areas that exist with no dialog
/// open (the ledger and so on) are the baseline and are skipped too.
/// Opens every dialog and tab at `size` and returns the scroll areas that
/// overflow (lists of the user's own data, ids ending in "-list", excepted).
fn dialogs_that_scroll_at(size: iced::Size, tag: &str) -> Vec<String> {
    use crate::color_dialog::{ColorMsg, Tab as ColorTab};
    use crate::disto::DistoMsg;
    use crate::e2e_support::{scroll_areas, ScrollArea};
    use crate::guide::{GuideMsg, TOPICS};
    use crate::options_dialog::{OptMsg, Tab as OptTab};
    use crate::page_layout::LayoutMsg;
    use crate::shortcuts::ShortcutsMsg;
    use crate::subject_info::{SubjectMsg, Tab as SubjectTab};
    use crate::text_manager::TextManagerMsg;
    use crate::ui::start::Msg as StartMsg;

    let full_hd = size;

    let prepared = || {
        let mut h = harness_next_at(full_hd);
        draw_closed_square(&mut h);
        h.send(Message::Key(KeyCmd::SelectAll));
        h.send(Message::AreaWorkflow(crate::area_workflow::Msg::AssignSelected(
            "GLA1".into(),
        )));
        h.send(Message::Key(KeyCmd::SelectAll));
        h
    };
    let same = |a: &ScrollArea, b: &ScrollArea| {
        (a.bounds.x - b.bounds.x).abs() < 0.5
            && (a.bounds.y - b.bounds.y).abs() < 0.5
            && (a.bounds.width - b.bounds.width).abs() < 0.5
            && (a.bounds.height - b.bounds.height).abs() < 0.5
    };
    let baseline = scroll_areas(&state(prepared()), full_hd);

    let mut cases: Vec<(String, Vec<Message>)> = Vec::new();
    for tab in OptTab::ALL {
        cases.push((
            format!("options-{}", tab.label()),
            vec![Message::Options(OptMsg::Open), Message::Options(OptMsg::Tab(*tab))],
        ));
    }
    for tab in [SubjectTab::Parcel, SubjectTab::Comments, SubjectTab::Tables] {
        cases.push((
            format!("property-details-{tab:?}"),
            vec![Message::OpenSubjectInfo, Message::SubjectInfo(SubjectMsg::Tab(tab))],
        ));
    }
    for tab in [ColorTab::Standard, ColorTab::Custom] {
        cases.push((
            format!("colour-{tab:?}"),
            vec![
                Message::Color(ColorMsg::Open("#3182ce".into())),
                Message::Color(ColorMsg::Tab(tab)),
            ],
        ));
    }
    for (i, _) in TOPICS.iter().enumerate() {
        cases.push((format!("guide-{i}"), vec![Message::Guide(GuideMsg::Open(i))]));
    }
    for (name, steps) in [
        ("page-layout", vec![Message::PageLayout(LayoutMsg::Open)]),
        ("print-preview", vec![Message::PrintPreviewLayoutFixture]),
        ("breakdown", vec![Message::OpenBreakdown]),
        ("area-attributes", vec![Message::OpenAreaAttributes(true)]),
        ("area-properties", vec![Message::AreaProps]),
        ("room-shapes", vec![Message::RoomShapeOpen]),
        ("laser", vec![Message::Disto(DistoMsg::Connect)]),
        ("room-labels", vec![Message::TextLibraryManager(TextManagerMsg::Open)]),
        ("undo-history", vec![Message::UndoHistory]),
        ("shortcuts", vec![Message::Shortcuts(ShortcutsMsg::Open)]),
        ("map", vec![Message::GeoLoadMap]),
        ("area-types", vec![Message::OpenCodesEditor]),
        (
            "symbol-library",
            vec![Message::Ui(UiMsg::SymbolLibrary(
                crate::ui::symbol_library::Msg::Open(None),
            ))],
        ),
        ("start", vec![Message::Ui(UiMsg::Start(StartMsg::Open))]),
        ("about", vec![Message::About]),
    ] {
        cases.push((name.to_owned(), steps));
    }

    let mut failures = Vec::new();
    for (name, steps) in cases {
        eprintln!("fit-{tag}: {name}");
        let mut h = prepared();
        for m in steps {
            h.send(m);
        }
        let _ = h.screenshot_in(&format!("e2e-next/fit-{tag}"), &name);
        let app = state(h);
        for area in scroll_areas(&app, full_hd) {
            let is_list = area.id.as_deref().is_some_and(|id| id.contains("-list"));
            let in_baseline = baseline.iter().any(|b| same(b, &area));
            if area.overflows() && !is_list && !in_baseline {
                failures.push(format!(
                    "{tag} {name}: {:.0} x {:.0} area holds {:.0} x {:.0} content (id {:?})",
                    area.bounds.width,
                    area.bounds.height,
                    area.content.width,
                    area.content.height,
                    area.id
                ));
            }
        }
    }
    // The area chooser opens by clicking an untyped outline with the Area tool.
    {
        let (cx, cy) = drawn_square_center_on_screen(full_hd.width, full_hd.height);
        let mut h = harness_next_at(full_hd);
        draw_closed_square(&mut h);
        h.send(Message::Rail(crate::view_state::RailTool::DefineArea));
        h.script(&[&format!("click ({cx:.1}, {cy:.1})")]);
        assert!(h.shows("What is this area?"), "chooser did not open");
        let _ = h.screenshot_in(&format!("e2e-next/fit-{tag}"), "area-chooser");
        let app = state(h);
        for area in scroll_areas(&app, full_hd) {
            let is_list = area.id.as_deref().is_some_and(|id| id.contains("-list"));
            let in_baseline = baseline.iter().any(|b| same(b, &area));
            if area.overflows() && !is_list && !in_baseline {
                failures.push(format!(
                    "{tag} area-chooser: {:.0} x {:.0} area holds {:.0} x {:.0} content",
                    area.bounds.width, area.bounds.height, area.content.width, area.content.height
                ));
            }
        }
    }

    failures
}

/// Every dialog, and each of its tabs, fits without its form scrolling
/// (owner requirement, 2026-11-17): at 1920 x 1080, and at 1382 x 816 (the
/// owner's 3456 x 2160 screen at 250% scaling, maximized above the taskbar).
#[test]
fn dialogs_fit_1920x1080_without_scrolling() {
    let mut failures = dialogs_that_scroll_at(iced::Size::new(1920.0, 1080.0), "1920");
    failures.extend(dialogs_that_scroll_at(iced::Size::new(1382.0, 816.0), "1382"));
    assert!(
        failures.is_empty(),
        "dialogs that scroll:
{}",
        failures.join("
")
    );
}

/// The Dark interface on the Dark drawing background (the field precision
/// CAD palette, 2026-11-17): a typed area, selected and not.
#[test]
fn readme_floor_plan_screenshot() {
    use crate::area_workflow::Msg as AreaMsg;
    use sketch_core::geometry::Pt;

    fn walls(app: &mut crate::App, mode: Mode, pts: &[(f64, f64)]) {
        let _ = app.set_mode(mode);
        let ctx = crate::drawing::DrawCtx {
            snap: false,
            ..app.draw_ctx()
        };
        // Plan coordinates run down the page; the drawing's y runs up.
        app.drawing.pen_down_at(Pt::new(pts[0].0, -pts[0].1), ctx);
        for &(x, y) in &pts[1..] {
            let before = app.drawing.lines.last().map(|l| l.id);
            app.drawing.click(Pt::new(x, -y), ctx);
            app.finish_wall_commit(before);
        }
        let _ = app.update(Message::Key(KeyCmd::Escape));
    }

    // A small ranch house in feet: an L-shaped living area, an attached
    // garage and a front porch, with rooms and furniture.
    let (mut app, _) = crate::App::boot(1.0);
    let areas: [(&str, &[(f64, f64)]); 3] = [
        (
            "GLA1",
            &[(0., 0.), (44., 0.), (44., 28.), (24., 28.), (24., 38.), (0., 38.), (0., 0.)],
        ),
        ("GAR", &[(44., 0.), (68., 0.), (68., 24.), (44., 24.), (44., 0.)]),
        ("OFP", &[(24., 28.), (44., 28.), (44., 36.), (24., 36.), (24., 28.)]),
    ];
    for (code, pts) in areas {
        let known: std::collections::BTreeSet<i64> =
            app.drawing.polygons.iter().map(|p| p.id).collect();
        walls(&mut app, Mode::DrawExterior, pts);
        let new = app
            .drawing
            .polygons
            .iter()
            .map(|p| p.id)
            .find(|id| !known.contains(id))
            .unwrap_or_else(|| panic!("{code} closes"));
        app.selection.clear();
        app.selection.polys.insert(new);
        let _ = app.update(Message::AreaWorkflow(AreaMsg::AssignSelected(code.into())));
    }
    for room in [
        &[(14., 0.), (14., 16.), (0., 16.)][..],
        &[(14., 10.), (24., 10.), (24., 0.)][..],
        &[(14., 16.), (14., 38.)][..],
    ] {
        walls(&mut app, Mode::DrawInterior, room);
    }
    for (id, x, y) in [
        ("caddillo/bed", 7.0, 6.5),
        ("caddillo/tub", 19.0, 1.4),
        ("caddillo/toilet", 22.5, 8.5),
        ("caddillo/sink-bath-rect", 16.2, 8.8),
        ("caddillo/fridge-36", 42.2, 1.5),
        ("caddillo/range-30", 37.0, 1.3),
        ("caddillo/sink-kitchen-dbl", 31.0, 1.2),
        ("caddillo/dining-table-96x42", 32.0, 12.0),
        ("caddillo/sectional-l-96", 7.0, 26.0),
        ("caddillo/car-plan-6x16", 49.5, 11.0),
        ("caddillo/washer-27", 66.5, 2.0),
        ("caddillo/dryer-27", 66.5, 4.6),
        ("caddillo/door-swing-36", 34.0, 28.0),
        ("caddillo/door-garage-108", 50.0, 24.0),
    ] {
        assert!(app.symbols_lib().symbol(id).is_some(), "{id}");
        app.place_symbol_at(Pt::new(x, -y), id.into());
    }
    app.selection.clear();
    let model = app.build_save_model();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/e2e-next");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Sample House.json");
    std::fs::write(&file, serde_json::to_vec(&model).unwrap()).unwrap();

    let mut h = harness_next_at(iced::Size::new(1600.0, 900.0));
    h.send(Message::FileDropped(file));
    h.send(Message::Key(KeyCmd::CenterFit));
    h.send(Message::Status(StatusMsg::ZoomOut)); // clear of the tool tray
    h.send(Message::Key(KeyCmd::Escape));
    h.wait(5000); // let the "opened" toast go
    assert!(h.screenshot_in("e2e-next", "readme-floor-plan").exists());
    let opened = state(h);
    let mut codes: Vec<&str> = opened.drawing.polygons.iter().map(|p| p.kind.as_str()).collect();
    codes.sort();
    assert_eq!(codes, ["GAR", "GLA1", "OFP"]);
    assert_eq!(opened.drawing.symbols.len(), 14);
}

#[test]
fn dark_palette_on_dark_drawing_background_screenshots() {
    use crate::canvas_theme::{self, CanvasTheme};

    // One screenshot per test window (an `iced_test` limit), so the
    // selected and plain views each get their own.
    for selected in [true, false] {
        let mut h = harness_next_at(VIEWPORT);
        h.send(Message::PopupMenu(PopupMsg::Interface(InterfaceTheme::Dark)));
        h.send(Message::PopupMenu(PopupMsg::Canvas(CanvasTheme::Dark)));
        draw_closed_square(&mut h);
        h.send(Message::Key(KeyCmd::SelectAll));
        h.send(Message::AreaWorkflow(crate::area_workflow::Msg::AssignSelected(
            "GLA1".into(),
        )));
        h.send(Message::Key(KeyCmd::SelectAll));
        if !selected {
            h.send(Message::Key(KeyCmd::Escape));
        }
        assert_eq!(canvas_theme::current(), CanvasTheme::Dark);
        assert_eq!(
            CanvasTheme::Dark.bg(),
            iced::Color::from_rgb8(0x0f, 0x14, 0x1a),
            "the dark drawing background is #0F141A"
        );
        let name = if selected {
            "p14-dark-canvas-selected-wide"
        } else {
            "p14-dark-canvas-wide"
        };
        assert!(h.screenshot_in("e2e-next", name).exists());
    }
}
