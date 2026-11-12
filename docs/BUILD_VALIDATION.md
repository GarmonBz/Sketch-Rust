# Build validation — 0.1.29 native compatibility preview

0.1.29 User Guide and installer (2026-11-11):

- Help > User Guide and Help > Integration Guide open an in-app guide (text in
  crates/sketch-app/src/guide.rs). Headless screenshots of the Help tab and
  two topics were reviewed; the guide was not clicked through in the running
  app. The integration steps follow the behavior documented in
  APEX-COMPATIBILITY.md and were not re-run against Appraise-It Pro.
- Automated: 626 app and 178 core tests passed, 0 failed.
- Installer: tools/installer.ps1 builds SketchRust-<version>-x64.msi (WiX 5.0.2).
  Its -Test installs silently into a scratch folder, checks the program files,
  both shortcuts and the Windows Installer registration with the app icon, then
  uninstalls and checks nothing is left. The dialogs were stepped through once
  by hand (install, then the finish-page launch); upgrade over an older MSI,
  folder change, desktop-shortcut opt-out and removal through Settings > Apps
  were not exercised.

0.1.28 Recent Files crash (2026-11-11):

- The Recent Files row used a fill-weight "file" icon that was not in the
  embedded icon table, and the lookup panicked. The icon is added; a release
  build now substitutes an empty icon for a missing one. A scan of literal
  icon uses in the source found no others missing.
- Automated: 622 app tests passed, 0 failed (before the version bump),
  including a new test that builds the list with an entry. Not clicked through
  in the running app.

0.1.27 frame presentation (2026-11-11):

- Vsync is off by default (iced Settings.vsync = false); ICED_PRESENT_MODE
  still overrides it. Measured beforehand in a release build with about 30
  walls: a pointer move costs 0.04 ms and a full UI rebuild and layout 1 ms, so
  the lag was in presentation, not app logic. The user confirmed the lag is
  gone with ICED_PRESENT_MODE=immediate on the 0.1.26 build.
- Checked live: the release exe starts and responds. The full test suite was
  not re-run for this settings change; the Apex tests were, after the bump.

0.1.26 window and desktop integration (2026-11-11):

- Maximize/Restore replaces full screen; remembered placement (window.json),
  800 x 560 minimum, edge/corner resize zones, unsaved-changes tracking with a
  save prompt on close, file drop and command-line open, bring-forward for an
  already open file, quick-access menu, pinch zoom and two-finger pan.
- Automated: `cargo test -p sketch-app` — 619 passed, 0 failed, 2 ignored
  (run on the 0.1.25 tree with these changes, before the version bump).
- CADdillo doors pivot on the jamb (catalog regenerated; only those 16
  definitions changed) and Help > About shows the version. Automated after
  this change: 621 app and 178 core tests passed, 0 failed. Neither was
  checked on screen.
- Checked live: the release exe starts, closes cleanly on an unchanged document
  and restores the stored window size and position.
- Not checked on screen or hardware: edge resizing with a mouse, maximized
  restore, title-bar double-click, the quick-access menu and three-button close
  dialog rendering, Explorer file drop, launch-with-file and bring-forward, and
  touchscreen pinch.

0.1.25 permanent symbol deletion (2026-10-31):

- Removed Purge, Purge hidden, the Purged view, its export option and the
  duplicate Delete category forever action. Delete forever is labeled Delete
  in the manager and right-click actions. Delete category uses the same permanent
  operation; Hide/Restore remains a separate visibility setting.
- A successful deletion updates symbols.json and removes placed instances on
  every page, undo/redo history, clipboard entries, favorites and overrides.
  The previous file is retained as symbols.json.bak. Deleted IDs also block
  reintroduction through old documents and layer imports.
- Library writes happen before document removal. A write failure leaves the
  definition, placed instances, history, selection and confirmation intact;
  after the file becomes writable, the same confirmation can retry successfully.
  There is no fallback to reversible removal.
- Legacy purged/purgedCategories preferences deserialize as deleted IDs/category
  records. These remain excluded and cannot be restored by Hide/Restore actions.
  New preference writes use deleted/deletedCategories.
- All 25 management regressions passed, including physical single/category/empty
  deletion, two confirmations, Cancel, restart, legacy settings, layer-import
  blocking, undo cleanup and read-only file failure/retry. The full application
  regression run passed all 327 cases, with two timing probes ignored.
- All ten focused UI cases passed. They exercise the manager Delete button,
  right-click Delete,
  removal of the old controls, category picker contents, Fire category deletion,
  Cancel/Escape, small-window layouts, preview, favorites and actual dock symbol
  placement. Workspace all-target Clippy and formatting passed. Evidence is under
  target/symbol-delete-025/{regression-final,ui-final,category-ui,clippy-final}.txt
  and target/e2e/*025.png.
  Tests use disposable libraries/profiles; the shipped CADdillo catalog and its
  original-library backup retain their pre-test hashes.
- The optimized build retains the embedded host integration and GUI subsystem;
  no companion executable or console window is introduced.

0.1.24 CADdillo residential library (2026-10-31):

- Downloaded the official archive and verified its SHA256. It contains 413 DXFs;
  the selection manifest imports 177 residential blocks into ten new categories.
  The 462 original definitions remain verbatim, including unknown fields and
  decimal literals. The combined catalog contains 639 definitions. A separate
  byte-identical original catalog is included as symbols.previous.json.
- Four pre-update library/profile files were backed up under
  backups/symbols-before-caddillo-0.1.23; every backup hash was verified. Private
  profile backups are not packaged. Existing personal settings are preserved.
- The manager and catalog builder share the same Rust DXF importer. Tests cover
  actual vendor files, inch conversion, BLOCK/INSERT base points, nested scale,
  translation, rotation and reflection, and CRLF input. Ellipse and solid
  conversion are available through the same importer. Missing
  and recursive blocks fail cleanly. Vendor promotional text is excluded.
- Verified the 27-inch washer, 60x30-inch tub, 76x80-inch king bed, 84x36-inch
  couch, nominal door openings and centered window pivots. All catalog IDs and
  categories resolve; original definitions match their backup exactly.
- Actual headless widget/canvas events verify wall placement on horizontal,
  vertical and diagonal walls; both door sides; freely placed fixtures; corner
  scaling and the selection-scale action; matching wall gaps, Undo and reopening.
  Import/restart also uses disposable profiles and portable document definitions.
- PNG/SVG fitting and print/report bounds include full rotated/scaled symbols;
  mirrored symbols retain their footprint. A 16-symbol exported contact sheet
  was visually inspected without edge cropping. Center/Fit includes symbols.
- All 327 application and 178 core regression cases passed; two diagnostic timing
  probes remain ignored. Seven focused actual-widget UI cases passed: four new
  placement/scale/preview cases and three existing symbol-manager cases including
  small windows. Workspace all-target Clippy and formatting passed. Evidence:
  target/caddillo-024/{app-final,core-final,caddillo-tests,manager-ui,clippy-final}.txt,
  target/caddillo-024/residential-contact-sheet.png and target/e2e/caddillo-*.png.
- No new host service or companion executable is introduced; integration stays
  inside Sketch. Report/print images receive no vendor logo or copyright notice.
  Verification uses CARGO_INCREMENTAL=0 and workspace TEMP/TMP directories.

0.1.23 symbol-category deletion (2026-10-31):

- Reproduced four failures before the fix: the dock category was lost on Open,
  Fire deletion retained a stale filter, empty categories did nothing, and the
  category confirmation was not presented as a centered named prompt.
- Manage symbols inherits the active real dock category. Delete category and
  Delete category forever freeze the complete category selection, independent
  of search. A centered modal identifies its name, count and placed-instance
  consequences. Continue advances to the final confirmation. Cancel/Escape
  abort without deleting, and the manager stays open.
- Rebuilding the library resets removed category choices in the manager and
  dock, clears the stale search filter, and chooses a preview from live rows.
  Empty categories work for both ordinary purge and physical-file deletion;
  physical deletion retains the previous file as symbols.json.bak.
- All 325 application and 172 core tests passed, with two diagnostic timing
  probes ignored and 272 GUI cases excluded from the broad run. Five focused
  actual-widget checks passed: Fire deletion including Cancel/Escape and list
  refresh; 800x600 confirmation; existing manager actions; live/separate preview;
  and narrow-window footer layout. Workspace all-target Clippy and formatting
  passed. Preview focus is checked against visible rows in Purged and other views.
- The core file-edit tests use a workspace TEMP/TMP directory; replacement in
  the default system temporary directory was denied by the sandbox. All 172
  core cases passed with that writable temporary root, including physical
  category deletion, backups, rename, move and merge operations.
- Tests use disposable symbol libraries and profiles. The user's Fire category
  and live library are not modified while testing. Evidence is under
  target/category-delete-023/{regression-final,core-final,ui-final}.txt and
  target/e2e/fire-category-delete-*.png. Verification used CARGO_INCREMENTAL=0.

0.1.22 integrated close choices (2026-10-30):

- A linked document's close dialog offers Save to report, Discard changes and
  Cancel. Save uses the existing full return pipeline, and exits only after
  a positive host receipt. Stale receipts are removed before each return.
- Discard cancels the linked session without publishing a new return/save
  signal. Cancel/Escape preserve the open sketch. A repeated close cannot
  interrupt a pending transfer. Standalone confirmation remains unchanged.
- Windows close requests (including Alt+F4) now route through the same Close
  message instead of bypassing the prompt. The native auto-close flag is off.
- Save/return acknowledgment waits use a dedicated thread and Iced oneshot,
  matching the actual Iced executor without requiring a Tokio timer runtime.
- All 314 application and 167 core regressions passed (270 GUI tests excluded
  from the broad run; two diagnostic probes ignored). All six focused close
  checks passed, plus the existing standalone close widget test. All-target
  workspace Clippy and formatting checks passed.
- Actual widget input clicks Cancel, Save to report and Discard changes and
  exercises Escape. Save/Discard I/O errors keep the widget-based editor open.
  Disposable-file tests verify successful discard, current 200 sf measurements,
  stale receipt removal, positive/negative host acknowledgment, duplicate-close
  guards, preparation failures, timeout and closed-host errors. A return emits
  the exit action only after a successful receipt. Host receipts are staged in
  workspace fixtures; no live Appraise-It report was modified in this release.
- The rendered dialog was inspected: title, text and all three buttons fit.
  Evidence: target/report-close-022/{focused-final,regression-final}.txt and
  target/e2e/report-close-options.png.

0.1.21 console and input responsiveness (2026-10-30):

- The embedded apex-compat.dll is compiled as a managed Windows GUI entry
  assembly, preserving ExecuteAssembly/bootstrap support without a console.
  Build-time PE checks and the packaged in-process probe reject a console
  subsystem in the embedded runtime; the outer EXE is checked separately.
- Report-image preparation runs on a dedicated worker with frozen document
  state. Only the UI completion publishes metadata; session/epoch checks
  reject late renders after an explicit Save/Return or document switch.
  Unchanged host getters reuse the published packet and image paths. Read-only
  polls and background completions preserve the floorplan cache.
- Rapid drawing moves paint the pointer, snapping preview and alignment guides
  from the physical cursor. Application/HUD updates coalesce within 16 ms and
  flush the last move on a scheduled redraw. Clicks retain their own coordinates;
  keyboard input reads the latest coalesced cursor before drawing commands.
- PDF and email open through ShellExecuteW, removing cmd.exe from those paths.
- All 309 application and 167 core regressions passed (269 GUI tests excluded
  from the broad run; two diagnostic timing probes ignored). Three focused
  actual-widget checks passed: cached/uncached drawing image parity, typed
  entry/arrow cursor, and area-definition hover/label/Escape behavior. Workspace
  all-target Clippy and formatting passed.
- The isolated SDK test passed 66 component checks plus a real embedded server
  session: subject transfer, two pages, 200 sf, EMF export, save/close callbacks
  and editable-state byte persistence. Local IPC requires running this test
  outside the filesystem sandbox. It used the debug-only test entry point;
  release packaging independently probes the embedded runtime in-process.
- The same unoptimized one-area diagnostic measured a changed idle-poll update
  at 10,266.7 ms before and 47.7 ms after; 100 unchanged idle polls fell from
  676.0 to 47.2 ms. Image rendering completes separately after update returns.
  These are update-handler timings, not end-to-end cursor latency or frame rate.
  100 full view builds measured 336.0 ms before and 508.7 ms after; those builds
  still cost time, so high-frequency moves now coalesce full application updates.
- This release does not claim a new live Appraise-It report-transfer test.
  Existing report aspect and calculation-box rendering remain covered by the
  regression suite. The recovered interactive sketch was left untouched.
- Evidence and precise validation results: target/input-latency-021/.

0.1.20 integration setup buttons (2026-10-30):

- Help > Appraise-It Integration opens a native panel showing this EXE, the
  current host launch path, Enable integration, Restore previous provider and
  Close. The registration backend is the same one used by the existing CLI.
  A dedicated worker thread keeps the UI responsive during filesystem work
  and Windows elevation; no separate bridge or setup EXE is introduced.
- Duplicate operations and panel dismissal are blocked while setup is pending.
  Escape/Close dismiss when idle, and drawing shortcuts and DISTO input are
  gated while the panel is open. Actions stack vertically on narrow windows.
- Fixed protected upgrades: if the shared configuration cannot yet be written,
  restore the old registration record before requesting elevation. A new
  disposable-file regression checks the failed upgrade, intact old XML/record,
  successful retry to the new EXE and exact restoration of the original Apex
  launch configuration. Existing new/malformed/changed-config tests remain.
- Headless actual-widget checks click Help, the Integration entry, Enable,
  Restore and Close, exercise Escape, and check success/permission-error text.
  Backend outcomes are staged for GUI tests; actual shared host settings and
  Windows permission prompts are not changed or operated by those tests.
  Native backend tests use disposable workspace files. This is not a fresh-PC
  Appraise-It installation qualification or a new live report-transfer test.
- Screenshots: target/e2e/integration-setup-{open,error,small}.png.
  Broad regression evidence: target/integration-020/regression.txt.
- All 303 application regressions and 167 core tests passed (269 GUI tests
  excluded from that broad run; one timing probe ignored). All three focused
  setup checks passed, including pending-job guards, and all five disposable
  registration-backend tests passed. The control audit covers 130 ribbon
  instances and 17 File items with zero missing actions. Workspace/all-target
  Clippy and formatting checks passed. Existing print and report-transfer
  implementations remain unchanged from the previous build.

0.1.19 print calculations and pointer visibility (2026-10-30):

- Print Preview, File Print and PDF export use the same report generator.
  Equal Summary/Breakdown boxes occupy the bottom of the printable column on
  Letter, Legal and A4. Geometry and print raster share the same computed box
  height, preserving proportions when long tables need more space.
- Summary shows each defined area's actual code/name, signed factor, gross
  size, perimeter and net total. Breakdown uses the existing geometry engine's
  rectangles, triangles and circular segments, plus factors/deductions and
  Net LIVABLE totals. Undefined/stub areas are excluded from calculations.
- Long tables continue on numbered pages with repeated headings and clear
  footer space. Populated comment tables move to a strip above the pair;
  empty comment grids are omitted. Existing caption/toggle settings remain.
- Native PDF output was independently rendered with Poppler and inspected on
  all three papers. The sample shows 1,500 sf First Floor, 1,100 sf Second Floor,
  1,300 sf Finished BSMT and 2,600 sf Net LIVABLE. Both tables are centered and
  readable, with no appended copyright notice or logo. PDF text extraction
  independently confirms the numerical rows and calculation formulas.
- Area-definition mode previously hid the Windows cursor without painting the
  supplied SVG; a completed keyboard chain could also suppress the mouse when
  its keyboard preview was no longer visible. Both modes now keep a drawing
  pointer visible, while live keyboard previews retain pointer ownership.
- All three headless Print widget tests passed. The new check opens a sample
  through File Open, clicks File Print Preview, clicks its Print / Save PDF
  button and then File Print in a separate session. Both printed PDFs are
  identical and Preview Print matches the displayed PDF bytes exactly.
  The native PDF harness was stopped after its successful result because it
  remained alive during teardown; the application on the desktop was untouched.
- All 301 application regressions and 167 core tests passed (267 GUI tests
  excluded from the broad run; one development timing probe ignored).
  Workspace/all-target Clippy and formatting checks passed. Calculation tests
  cover all papers, 85-area overflow without missing rows, triangles, curved
  edges, factors/deductions and comment/caption settings.
- Evidence is under target/print-019/ and is excluded from the portable ZIP.
  Printer hardware and external Appraise-It reports are outside this patch's
  checks; the tests do not submit a physical print job or launch a PDF viewer.

0.1.18 drawing pointer and Save As verification (2026-10-30):

- The supplied floorplan_crosshair_cursor.svg is embedded unchanged in the EXE.
  Source and embedded asset SHA-256:
  D74757EA73A0BE5C93D9B08B94CFAB918BE9D385E384081954CCE8E86D9C8F7F.
  Mouse and keyboard drawing share the SVG, centered on its 16,16 hotspot.
  New settings default to 32 pixels; stored 32/64/128 preferences still apply.
- Raster reference checks cover transparent corners, centered dark hotspot and
  all three sizes. Headless screenshots verify the actual canvas pointer on
  both light and dark backgrounds. The harness suppresses the physical mouse
  cursor in screenshots, so these use the keyboard drawing pointer, which
  calls the same renderer. No desktop pointer or user document is touched.
- Pointer hover skips layer sorting, calculation-signature serialization,
  workflow catalogue refresh and visibility-gate reconstruction when the scene
  has not changed. Static floorplan geometry is cached; live guides, drawing
  previews and cursor remain separate. Document/view changes clear the cache.
  Cached and uncached complete canvas screenshots compare pixel for pixel.
- Update-only development timing probe (unoptimized, 500 hover messages):
  0 rooms: 300.86 ms before / 1.04 ms after; 10 rooms: 399.38 / 0.48 ms;
  40 rooms: 743.03 / 0.46 ms. This measures application update bookkeeping,
  excluding GPU drawing, event delivery and display latency; it is not an FPS
  or end-to-end cursor-latency measurement. Raw evidence is under
  target/pointer-performance/{before,after}.json.
- Save As matrix: SKX-to-SKX, SKX-to-JSON, JSON-to-SKX, JSON-to-JSON and identical
  filenames in different folders. Real File-menu Save/Save As widgets select
  the target through staged native-picker results, then File, toolbar and
  keyboard-handler Save each write an additional edit to the new destination.
  All 15 subsequent saves show zero additional pickers. All five originals
  remain byte-for-byte unchanged, independently verified with SHA-256 hashes.
  Every final copy retains GLA1 and its latest rotation. Cancellation and write
  failures also retain the previous valid target. Fixtures and hash evidence:
  target/direct-save-017/save-as-matrix/ (excluded from the package).
- All 20 focused save/recovery checks and all 300 application regressions
  passed (266 GUI tests excluded from the regression run; one timing probe
  ignored). Workspace/all-target Clippy passed with warnings denied.
  All 20 focused area-workflow widget tests passed, plus typed/arrow drawing,
  Center/Fit, curved-wall dragging, and drawn-wall undo/redo widget checks.
  Pointer screenshots: target/e2e/crosshair-{light,dark}.png and
  target/e2e/pointer-{cached,uncached}.png. The full GUI gallery, physical DISTO
  and external Appraise-It reports were not retested for this patch.

0.1.17 direct saving (2026-10-30):

- Save writes atomically to the bound path without showing a picker. First
  save and Save As use one native file dialog. Host-linked Save still uses its
  existing embedded host API.
- Successful Open and Save/Save As retain the destination and container format.
  JSON remains JSON on direct save. Save As commits its new target only after
  a successful atomic write; errors and cancellation retain the prior target.
- New and recovery clear the file binding. New Recent entries store their path
  in local history and reopen the latest file from disk. Older cache-only or
  missing-file Recent snapshots remain unbound and request a save destination.
  No absolute session destination is embedded in portable SKX documents.
- Real-widget tests click Open, first Save, direct File/QAT Save and Save As,
  verify the latest rotation was written, test SKX and JSON, and check cancelled
  or failed Save As followed by Save. New-document tests verify the old file
  remains unchanged. Native picker results are staged in the headless harness;
  the native Windows dialog itself was tested in the 0.1.15 baseline.
- Additional checks cover Recent restart metadata and fresh disk content,
  the Ctrl+S dispatch, recovery, failed Open, and failed direct-save retention
  of the current target, drawing, and recovery snapshot.
- All 19 focused save/recovery checks passed. Disposable fixtures are under
  target/direct-save-017/ and are excluded from the package.
- All 299 application regressions passed (264 GUI/e2e tests excluded from that
  run). Workspace/all-target Clippy passed with warnings denied. The full GUI
  screenshot suite and external Appraise-It reports were not retested.

0.1.16 baseline: single-dialog saving (2026-10-30):

- Removed the first-save and Save As naming prompts. The native Windows picker
  is the only save dialog. Host-linked Save still calls the existing host API.
- The selected path supplies the saved document name, title and recent-file
  label only after the atomic write succeeds. Cancellation and write failure
  retain the previous name and drawing.
- Headless real-widget tests stage the native picker result rather than opening
  Windows dialogs on the user's desktop; they check one picker invocation, no
  internal prompt, real SKX writes/read-back, cancellation and write failure.
  Native Windows dialog interaction itself was verified in the 0.1.15 baseline.
- All 13 focused save/recovery tests passed, including saving an already named
  sketch, direct Save As from command search, and preservation of area codes.
  Fixtures are in target/single-save-016/; these are not shipped in the package.
- All 296 app regressions passed (261 GUI/e2e tests excluded from that run),
  including the wiring guards and image-aspect checks. Workspace/all-target
  Clippy passed with warnings denied. The full GUI suite was not rerun.

0.1.15 baseline: control-wiring correction (2026-10-30):

- All 13 audited controls now invoke the existing native handlers. Home shares
  clipboard/selection/Delete handlers with Edit Tools; Rotate Sketch is mapped;
  two-line captions normalize before lookup; File Open/Save/Save As are mapped.
- Wiring guards cover 129 ribbon button/field instances and 17 File-menu items.
  Actual widget tests exercise the repaired Home, View and Tools buttons with
  selected walls and defined areas, plus File Save/Save As naming prompts.
- 296 application regressions passed (260 GUI/e2e tests excluded), and all six
  focused real-widget tests passed. Workspace/all-target Clippy passed with
  warnings denied. The full screenshot suite was not rerun.
- Actual optimized Windows EXE: clicked File Open, selected the disposable SKX
  through the native dialog, and verified its 10 x 10 First Floor / 100 sf view.
  File Save wrote saved.skx through the native save dialog. File Save As accepted
  a new name and wrote ControlButtons015-Copy.skx. Reading both SKX containers
  back confirmed all page data matched the input exactly: four walls, one
  polygon, dimensions and area properties preserved. The new file name matched.
  Evidence: target/control-buttons-015/native-file-results.json. Computer use
  was stopped by the user after these file checks; no later UI actions occurred.
- Prior Center/Fit and Pro image-aspect corrections remain in this build.
  Appraise-It reports and DISTO hardware are outside this patch's test scope.

0.1.14 baseline: view-control correction (2026-10-30):

- Center and Center & Fit previously passed a placeholder `None` instead of
  drawing bounds. Both buttons and C / Shift+C now use the current page's bounds.
- Targeted application tests cover preserving zoom and geometry, fitting after
  window resize, empty pages, curved-wall extents, and excluding page underlays.
- The headless GUI harness draws and pans a real sketch, then clicks both actual
  buttons and types C. All three actions center the sketch; Fit adjusts zoom
  and retains the canvas margin. Shift+C is covered through the shortcut mapping
  and application handler tests.
- This patch changes view positioning only. Pro image rendering and transfer
  code retain the verified 0.1.13 behavior below; Pro report tests were not repeated.
- 294 application regression tests passed (254 GUI/e2e tests excluded from that
  run), plus the focused GUI test above. Workspace/all-target Clippy passed with
  warnings denied. The full screenshot suite was not rerun for this view-only fix.

0.1.13 baseline: aspect-ratio correction verified in actual Pro reports (2026-10-30):

- Reproduced the 0.1.12 distortion: a native 10 x 10 square measured 1351 x 1646
  pixels in the stored legacy JPEG and 842 x 1026 pixels in Pro's rendered
  Sketch addendum. The earlier centering checks did not verify shape proportions.
- The native helper unconditionally converts EMFs onto a 2615 x 4202 canvas.
  Integration pages now match that aspect before transfer. Additional height
  goes to the floorplan region; crowded pages expand the outer width as well,
  retaining all table rows and uniformly scaled geometry. Print forms stay separate.
- Verified 0.1.13 legacy stored square: 1352 x 1351 px. Pro's actual rendered
  report page: 842 x 841 px. Verified UAD stored square: exactly 777 x 777 px.
  These measurements come from colored outline pixels, not declared image sizes.
- The 20 x 10 fixture also retains its approximately 2:1 ratio (pixel strokes
  and JPEG resampling account for differences under 1%). Measurements remain
  300 sf living area, 60 sf finished basement and 40 sf unfinished basement.
- Both pages retain centered compact Summary/Breakdown sections with no appended
  copyright notice or branding logo. Both Pro report images were read back;
  the legacy report itself was additionally rendered through Pro's API.
- Evidence: target/pro-native-tests/013-aspect-verification.json,
  013-legacy-report.json, 013-uad-final-report.json and images-013/.
- Both disposable Pro reports were saved after the corrected transfers.
- 291 app regressions and 164 core tests passed (253 GPU/e2e app tests excluded), including a new
  pixel-level square test after the host canvas resize; 66 embedded API/frame
  contract checks and Clippy passed. Full GPU screenshots were not rerun.

0.1.12 baseline (centering, not aspect-ratio certification):

Final actual Appraise-It Pro 3.8.8 checks (2026-10-30):

- The native UAD 3.6 Transfer sketches dialog was completed with both pages
  assigned to the existing Dwelling / Unit. Merely receiving the SDK return
  receipt was not counted as a completed Pro image transfer.
- Read back both newly stored UAD JPEGs through Pro's data-store API, then
  visually inspected them: each has the floorplan, compact Summary and Breakdown
  boxes, complete totals, and no appended copyright notice or branding logo.
- UAD images are 836 x 1344. Pixel measurements of the table header margins:
  page 1 left/right 19/20 px; page 2 19/21 px. The 1–2 px difference is consistent
  with JPEG compression. Both report image hashes changed from the older layout.
- Legacy stored JPEGs are 1453 x 2335. Both have equal 33/33 px table margins,
  both calculation sections, and no copyright notice or bottom branding logo.
- UAD unit fields remain 300 sf above-grade finished, 60 sf below-grade
  finished, 40 sf below-grade unfinished, with exactly two unit sketch images.
  Both disposable reports were saved. Legacy retains 300 sf living area,
  100 sf basement / 60% finished, its 120 sf garage row and two sketch images.
- Fixed asymmetric metafile padding caused by DPI-virtualized display bounds.
  Physical desktop bounds now drive the recorded image frame. Twelve added
  contract checks verify all four page edges in GDI, GDI+ and dual EMFs.
- 290 app tests and 164 core tests passed, with 253 GPU/e2e app tests filtered; 66 embedded host contract
  checks passed; Clippy with warnings denied passed. Full GPU screenshots were
  not rerun. Final package-smoke results accompany packaging.
- Evidence: target/pro-native-tests/012-uad-final-report.json and
  012-legacy-final-report.json, plus the JPEGs under images-012/. These probes
  are development-only and excluded from the package. The integration code
  stays in sketch-app.exe; no bridge executable or service is shipped.

Earlier layout and transfer baseline:

0.1.11 replaces floorplan-only host images with full pages containing Summary
and Breakdown tables. A fixture matching the supplied sample verifies 1500 sf
first floor, 1100 sf second floor, 1300 sf basement and 2600 sf Net LIVABLE.
Its actual transfer-page renderer produces target/host-report-preview/sample.png.
The generated SVG and 300-DPI PNG were visually inspected. The image has no
appended copyright notice or branding logo. Automatic report-label spacing is
in physical points so fitting a large drawing cannot collapse name/size spacing.
290 app tests passed (253 GPU/e2e tests filtered); the sample preview was rerun
after the spacing correction.
The following 0.1.10 results are the earlier host baseline. The final 0.1.12
stored-image verification is documented above.

Actual Appraise-It Pro 3.8.8 desktop transfer checks (2026-10-30):

- Disposable legacy FNMA-1004-0911 report: 300 sf living area, 100 sf basement,
  60% finished, a 120 sf garage measurement row, and two nonempty sketch images.
- Disposable UAD 3.6 FNMA-URAR-0924 report: living-unit fields contain 300 sf
  above-grade finished, 60 sf below-grade finished and 40 sf below-grade
  unfinished, with two unit sketch images. The sample's existing property image
  remains separate. UAD vehicle-storage field mapping was not independently verified.
- Both were reopened through Pro's native Apex integration helper and returned
  a second time with the same area totals and two images (no duplicate pages).
- Legacy launch was exercised using Pro's Apex ribbon button. Further runs used
  its installed native helper against the actual report PID. Return packets were
  frozen from the real Rust model and signalled through the embedded session
  interface; returned fields were read through Pro's transaction/data-store API.
  These are live report checks, not a report save/reopen or universal host certification.
- Fixed native finished-basement alias BSMT1 transfer and SDK shutdown callbacks
  that unregister synchronously. Both final host return receipts report ok=true.
- The shared original Pro Sample.ax6 SHA256 stayed
  7e1d9cefc99d4f3f1bdc9e9072dab255e6633c705c6332dce0d668cf900e822c.
- Development-only report probes/fixture generators remain under target and are
  excluded from the package. No integration bridge EXE or service is shipped.
- 289 app tests passed (253 GPU/e2e tests filtered); 164 core tests passed.
  The 54 embedded contract checks passed. The additional full SDK script was
  attempted against the release binary and timed out because its server probe
  command is debug-only; it is not counted as a new full live-SDK pass.

The broader native-file and SDK results below were established for 0.1.9;
native file/embedded contract checks were rerun during 0.1.10 preparation.

The requested geometry/calculation, unfinished-draft, code-table URL and
multiple-file startup changes are implemented inside sketch-app.exe. Actual
host certification remains open; see APEX-COMPATIBILITY.md.

Verified in this workspace:

- 164 core tests passed, including signed recursive native factors and totals.
- 287 app tests passed in the 0.1.9 baseline with the e2e screenshot suite excluded (253 filtered).
  This run does not claim a new GPU screenshot/full-suite result.
- 54 embedded component contract checks passed.
- 121 generated/edited AXF fixtures were accepted by installed Apex 7.1.16.0.
  These include native layouts 1–38, unfinished layouts 1–38 and all 462 symbols.
- The genuine drawing engine independently confirmed six fixture cases:
  transformed native symbols, all library symbols, zero area for an unfinished
  boundary, metric area 200, measured-face area 144, and nested/factored nets
  107.5 / -7.5 / 20. The reference engine is development-only.
- Live SDK IPC against the real Rust model passed native Save/Save As/Base64,
  Load, subjects/tags, georeferences, deletion/zoom, breakdown, images/forms,
  UI Save notification, repeated New and Close. The expanded checks passed
  native metric-unit return/save, point metadata after Rust serialization,
  recursive factors and face measurement restoration.
- A loopback HTTP fixture and file URL were fetched by the sketch EXE. Native
  XML nesting, signed factors, colors, calculation types and wall defaults
  were checked; external XML entities were rejected.
- Two separate native input files launched independent sketch EXE sessions,
  each editing/returning its own disposable file without loading SDK transport.
- Native drafts in layouts 1, 23 and 38 closed through actual Rust wall drawing
  and saved with their original name/code/factor/identity. Each output was then
  reopened and verified to contain a closed area rather than an open draft.

Formatting, Clippy with warnings denied, optimized build and package checks
are run before packaging. Package verification checks every manifest hash,
current release-EXE identity, Windows GUI subsystem, symbol loading, native
view/icon decoding, embedded same-process compatibility and SKX round trip.

The manifest and adjacent .sha256 file contain the executable/archive hashes.
The installation checks do not activate provider setup or modify reports.

Physical DISTO testing remains deferred. Appraise-It Pro live report results
are described above. TOTAL/WinTOTAL and other host report-field/registration
round trips remain unverified; those older host COM programs are not installed here.

Source logs: target/apex-019-core-final.log, apex-019-app-tests-final.log,
apex-019-files-final.log, apex-019-live-release-check.log and the corresponding
release/package logs. Test clients, fixture servers and reference-inspection
artifacts under target are excluded from the distribution.
