# P16. Android Feasibility Spike Findings

Status: **Written finding (spike, not a port)** — evaluates the 10 open questions in [`07-touch-and-android.md` §5](07-touch-and-android.md#5-android-what-to-verify-p16) against the current `ui-redesign` codebase and the pinned Cargo dependency graph (`iced 0.14.0`, `iced_winit 0.14.1`, `winit 0.30.13`, `wgpu 27.0.1`, `rfd 0.15.4`, `btleplug 0.11.8`, `rusqlite 0.37.0`).

---

## 1. Executive Verdict

- **Responsive UI & core domain engine: Ready.** The `ui-redesign` layout already scales down to `360 x 560` logical px (`window_state.rs`), implements Touch density, the floating/docked Pad (`ui/pad.rs`), touch-friendly workflows, bundled Barlow font bytes (`ui/fonts.rs`), and pure-Rust geometry/SKX/PDF export in `sketch-core` with zero C dependencies.
- **Immediate Android build on this host: Blocked (toolchain absent).** No Android SDK, Android NDK, `adb`, or Android `rustup` targets (`aarch64-linux-android` / `armv7-linux-androideabi`) are installed on this workstation (only `x86_64-pc-windows-msvc` is installed; `ANDROID_HOME`, `ANDROID_SDK_ROOT`, and `ANDROID_NDK_HOME` are unset; `JAVA_HOME` is OpenJ9 JDK 17). No compiled APK or device run could be executed in this spike.
- **Upstream runtime verdict: Conditional Go, gated on `iced_winit` + `platform` extraction.** `iced 0.14.0` (`iced_winit 0.14.1`) **does not support Android out of the box** — its event-loop bootstrap does not accept `AndroidApp`, exposes neither `winit`'s `android-native-activity` nor `android-game-activity` features, and lacks Android `Suspended` / `Resumed` surface-recreation handling. In addition, `rfd 0.15.4` has no Android backend, `persist.rs` relies on `APPDATA`, `symbols.json` is resolved beside `current_exe()`, `report_fonts.rs` queries OS system fonts, and custom frameless window controls (`decorations: false`, resize borders, `window_controls`) must be disabled on Android.

---

## 2. Host Toolchain Audit

Checked on the current Windows development machine before planning a compile check:

| Requirement | Observed State | Notes |
|---|---|---|
| Rust target (`aarch64-linux-android`) | **Missing** | Only `x86_64-pc-windows-msvc` installed in `rustup` |
| Android SDK (`ANDROID_HOME` / `ANDROID_SDK_ROOT`) | **Unset / missing** | Needed for platform JARs, build-tools, APK packaging |
| Android NDK (`ANDROID_NDK_HOME`) | **Unset / missing** | Needed by `libsqlite3-sys` (`rusqlite` C build) and native linking |
| `adb` | **Not found** | No device / emulator deployment possible |
| `JAVA_HOME` | `C:\Program Files\Semeru\jdk-17.0.2.8-openj9\` | Present (Gradle / Android Java glue compatibility with Semeru OpenJ9 vs HotSpot JDK 17 would still need verification) |

Because neither the Android NDK nor Android Rust targets are present on this host, cross-compilation and device verification could not be executed and remain the first physical prerequisite.

---

## 3. Question-by-Question Assessment (`07-touch-and-android.md` §5)

### 3.1 Does `iced 0.14` (`winit` + `wgpu`) run acceptably on Android?

- **Finding:** **Not out of the box.**
  - `iced_winit-0.14.1/src/lib.rs:79-81` builds the event loop directly via `EventLoop::with_user_event().build()` with no `#[cfg(target_os = "android")]` path.
  - `iced_winit` does not accept `winit::platform::android::activity::AndroidApp` (via `.with_android_app(app)`) from an `#[no_mangle] fn android_main(app: AndroidApp)` entry point, nor does it expose `winit`'s `android-native-activity` / `android-game-activity` Cargo features.
  - `iced_winit-0.14.1/src/lib.rs:178-188` only queries system theme inside `resumed()` and lacks Android pause/resume (`Suspended` / `Resumed`) `wgpu` surface teardown and re-creation handling.
- **Implication:** Before a hello-world prototype can render on Android, `iced_winit` (either via a maintained upstream release/PR or a workspace patch/fork) must add `AndroidApp` event-loop initialization, `android_main` entry integration, and proper `wgpu` surface lifecycle handling across Android background/foreground transitions. Frame pacing (`wgpu` Vulkan/GLES3 backend on Android) can only be measured after that hello-world prototype builds and runs on hardware.

### 3.2 Text input and the soft keyboard

- **Finding:** **Partial IME wiring exists in `iced_winit`/`winit`, but viewport/inset avoidance is unhandled.**
  - `iced_winit-0.14.1/src/window.rs:283-299` and `conversion.rs:321-327` already connect `set_ime_allowed(true/false)`, `set_ime_cursor_area`, `set_ime_purpose`, and `Ime::{Enabled, Preedit, Commit, Disabled}`.
  - Underneath, `winit-0.30.13/src/platform_impl/android/mod.rs:918-920` calls `self.app.show_soft_input(true)`.
  - However, `crates/sketch-app/src/ui/layout.rs` has no awareness of Android soft-keyboard (`WindowInsets.ime()`) height, so `text_input` controls positioned near the bottom of dialogs or sheets would be covered unless `layout.rs` / bottom sheets lift or scroll into view.
  - Numeric drawing/wall entry (`ui/pad.rs`) already works completely without invoking the OS soft keyboard.

### 3.3 File open and save (`rfd 0.15` vs Storage Access Framework)

- **Finding:** **Blocked — `rfd` has zero Android support and is called directly outside `platform.rs`.**
  - Inspecting `rfd-0.15.4/Cargo.toml` and `rfd-0.15.4/src/lib.rs` confirms backends only for Windows, macOS, Linux, and WASM32.
  - While `crates/sketch-app/src/file_dialogs.rs:9,36` wraps some calls, direct `rfd::FileDialog` / `AsyncFileDialog` calls remain scattered in:
    - `crates/sketch-app/src/ui/dialogs.rs:3397,3406`
    - `crates/sketch-app/src/layer_io.rs:61`
    - `crates/sketch-app/src/main.rs:3341,3370,3385,3396,3413,4201,4398,4423,7694,8753`
    - `crates/sketch-app/src/ui/symbol_library.rs:2654,2757,2773`
- **Action needed:** Consolidate every file open/save/folder picker behind an async `platform` abstraction backed by Android Storage Access Framework (`ACTION_OPEN_DOCUMENT`, `ACTION_CREATE_DOCUMENT`, reading/writing `content://` streams via JNI rather than POSIX filesystem paths).

### 3.4 Bluetooth laser (`btleplug 0.11` in `disto_transport.rs`)

- **Finding:** **Supported upstream in `btleplug 0.11.8`, requires JNI initialization + Gradle Java sources + runtime permissions.**
  - `btleplug-0.11.8/Cargo.toml:136-143` enables `#[cfg(target_os = "android")] mod droidplug;` (using `jni`, `jni-utils`, `once_cell`) and ships Java glue (`Adapter.java`, `Peripheral.java`, `ScanFilter.java`, `AndroidManifest.xml`).
  - `droidplug/mod.rs:10-20` requires calling `btleplug::platform::init(env: &JNIEnv)` at startup — without it, `global_adapter()` panics.
  - Because `btleplug` depends on those Java helper classes being packaged into the APK, Android builds will require a Gradle project (not pure bare-metal `cargo-apk` without Gradle glue) plus manifest & runtime permission prompts for `BLUETOOTH_SCAN`, `BLUETOOTH_CONNECT`, and `ACCESS_FINE_LOCATION` (depending on API level).

### 3.5 Printing and PDF viewing (`print.rs`, `sketch-core`)

- **Finding:** **WinRT PDF preview is already gated cleanly; font resolution and `xdg-open` are the gaps.**
  - `crates/sketch-app/src/print.rs:70-147` already gates WinRT `Data::Pdf::PdfDocument` behind `#[cfg(windows)]` and provides a non-Windows stub error.
  - Pure-Rust `sketch_core::export_pdf` is cross-platform, **except** `crates/sketch-core/src/report_fonts.rs:12-47` queries OS system fonts (`Verdana`, `Arial`, `SansSerif`) through `fontdb::Database::load_system_fonts()`. On Android, desktop fonts like `Verdana` / `Arial` are absent; bundling an embedded fallback TTF (or sharing bundled Barlow fonts) avoids silent missing/substituted report text.
  - `print::open_with_os` (`print.rs:172`) calls `platform::open_target` (`platform.rs:27-35`), which invokes `xdg-open` on `#[cfg(not(windows))]`. On Android this must be replaced with an Android `Intent` (`ACTION_VIEW` + `FileProvider`) or an in-app PDF preview via Android `PdfRenderer` / pure-Rust renderer.

### 3.6 Per-user files (`persist.rs`, `main.rs`, `autosave.rs`, `print.rs`)

- **Finding:** **Silently broken on Android today because of `APPDATA` checks.**
  - `crates/sketch-app/src/persist.rs:38-48` resolves the storage directory via `std::env::var("APPDATA")`, and `main.rs:7096,7109,7188,7496` also read `APPDATA` directly (`main.rs:10518` sets `SKETCH_PERSIST=1`). On Android `APPDATA` is unset (`None`), which disables persisted settings, theme, recent files, and autosave without crashing.
  - `print.rs:150-169` writes temporary preview PDFs using `std::env::temp_dir()`, which needs to point at the app's cache dir (`Context::getCacheDir()` / app-private cache path) on Android.
- **Action needed:** Route all state/autosave/cache paths through `platform` and initialize them from `AndroidApp::internal_data_path()` (or `Context::getFilesDir()` / `getCacheDir()`).

### 3.7 Symbol library location (`assets/symbols.json`, `symbols.previous.json`)

- **Finding:** **Current executable-relative resolution will fail inside an Android APK.**
  - `crates/sketch-core/src/symbols.rs:63-86` searches for `assets/symbols.json` (`11.9 MB`) and `symbols.previous.json` (`9.2 MB`) relative to `std::env::current_exe()` and `std::env::current_dir()`.
  - Furthermore, `sketch_core::symbol_edit` (called from `crates/sketch-app/src/ui/symbol_library.rs:2624-2629`) edits symbols in place using `.tmp` / `.bak` sibling files.
  - Inside an Android APK, `assets/` is read-only inside the zip container and not directly addressed by `current_exe()`. (By contrast, UI fonts in `ui/fonts.rs:11-28` already embed all 5 Barlow TTFs via `include_bytes!`.)
- **Action needed:** Package `symbols.json` (and `symbols.previous.json` if needed) as APK assets (or compressed embedded bytes) and copy them into writable `internal_data_path` on first launch so user symbol edits and `.tmp`/`.bak` writes continue to work.

### 3.8 SQLite store format (`rusqlite` + `bundled` in `sketchy/store.rs`)

- **Finding:** **Supported cleanly once Android NDK C toolchain is configured.**
  - `crates/sketch-app/Cargo.toml` uses `rusqlite 0.37` with `features = ["bundled"]` (`libsqlite3-sys 0.35.0`).
  - `sketch-core` has zero C dependencies (`serde`, `flate2`, `crc32fast`, `geo`, `fontdb`, `ttf-parser`, `tiny-skia`).
  - Compiling `libsqlite3-sys` for `aarch64-linux-android` only requires pointing Cargo/`cc-rs` at the Android NDK `clang`/`ar` (`CC_aarch64_linux_android`, `AR_aarch64_linux_android`, or driving the build with `cargo-ndk` / `xbuild`).

### 3.9 Appraise-It Pro integration (`appraise_it.rs`)

- **Finding:** **Already cleanly isolated.**
  - Windows COM integration (`appraise_it.rs`) and its callers are already gated behind `#[cfg(windows)]` across `crates/sketch-app/Cargo.toml:24-25`, `main.rs:9,1655,4039,5588,10298`, `guide.rs:14..357`, `ui/commands.rs:1490..1510`, `ui/dialogs.rs:3114`, and `ui/start.rs:230..294`.
  - Similarly, `platform::work_area` (`main.rs:1654-1666`) already falls back to `window::monitor_size(id)` under `#[cfg(not(windows))]`, and `platform::activate_existing` (`platform.rs:135-139`) returns `false` on non-Windows.

### 3.10 Custom window frame (`decorations: false` and custom chrome)

- **Finding:** **Must be gated off on Android, plus safe-area insets added.**
  - `main.rs:10370` configures `decorations: false` (custom frameless desktop window).
  - `main.rs:9892-9937` renders custom edge resize overlays (`resize_layer()`).
  - `crates/sketch-app/src/ui/topbar.rs:88,756-818` unconditionally renders `window_controls` (Minimize / Maximize / Close).
- **Action needed:** Hide `window_controls` and skip resize borders on Android (`#[cfg(not(target_os = "android"))]`), use standard fullscreen/edge-to-edge surface settings on mobile, and add top/bottom system-bar / display-cutout safe-area padding in `ui/layout.rs`.

---

## 4. Recommended Path Before Starting an Android Port PR

1. **Provision Android toolchain on a build host:** Install Android SDK, NDK, `rustup target add aarch64-linux-android`, and `cargo-ndk` (plus a Gradle wrapper template so `btleplug` Java classes can compile).
2. **Prove `iced 0.14` / `iced_winit 0.14.1` hello-world on device:** Validate `AndroidApp` bootstrap, `Suspended`/`Resumed` `wgpu` surface rebuilds, touch scrolling, soft keyboard open/close, and Vulkan/GLES3 frame pacing before porting `sketch-app`.
3. **Eliminate remaining direct desktop OS assumptions in `sketch-app`:**
   - Move all remaining `rfd` calls (`main.rs`, `layer_io.rs`, `ui/dialogs.rs`, `ui/symbol_library.rs`) into `platform.rs`.
   - Replace direct `APPDATA` reads in `main.rs` and `persist.rs` with `platform` data/cache directory helpers.
   - Add first-run extraction/asset loading for `symbols.json` in `sketch-core` and bundle a report fallback font in `report_fonts.rs`.
   - Gate `window_controls`, `resize_layer()`, and `decorations: false` off on Android, and plumb safe-area + IME insets into `ui/layout.rs`.
   - Wire `btleplug::platform::init` + Android BLE runtime permission requests for Disto laser pairing.
