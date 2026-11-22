//! Open desktop documents and URLs without a command-shell console.
use std::ffi::OsStr;

pub fn open_target(target: &OsStr) {
    if cfg!(test) {
        return;
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::{
            core::{w, PCWSTR},
            Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
        };
        let target: Vec<u16> = target.encode_wide().chain(Some(0)).collect();
        unsafe {
            let _ = ShellExecuteW(
                None,
                w!("open"),
                PCWSTR(target.as_ptr()),
                None,
                None,
                SW_SHOWNORMAL,
            );
        }
    }
    #[cfg(not(windows))]
    {
        let program = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let _ = std::process::Command::new(program).arg(target).spawn();
    }
}

/// The work area (monitor minus the taskbar and other docked bars) of the
/// monitor holding `window`, in OS-logical pixels (physical / DPI scale).
#[cfg(target_os = "macos")]
#[allow(dead_code)] // `main.rs` still uses `monitor_size` on macOS until wired in a later task.
pub fn work_area(window: &dyn iced::window::Window) -> Option<(f32, f32)> {
    use iced::window::raw_window_handle::RawWindowHandle;
    use objc2::rc::Retained;
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSView;

    let handle = window.window_handle().ok()?;
    let RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        return None;
    };
    let ns_view = appkit.ns_view.as_ptr() as usize;
    with_appkit_main(move |_mtm: MainThreadMarker| {
        // SAFETY: `ns_view` is the live view iced handed us for this call.
        let ns_view = ns_view as *mut NSView;
        let view = unsafe { Retained::retain(ns_view)? };
        let window = view.window()?;
        let screen = window.screen()?;
        let visible = screen.visibleFrame();
        Some((visible.size.width as f32, visible.size.height as f32))
    })
}

/// The work area (monitor minus the taskbar and other docked bars) of the
/// monitor holding `window`, in OS-logical pixels (physical / DPI scale).
#[cfg(windows)]
pub fn work_area(window: &dyn iced::window::Window) -> Option<(f32, f32)> {
    use iced::window::raw_window_handle::RawWindowHandle;
    use windows::Win32::{
        Foundation::HWND,
        Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        },
        UI::HiDpi::GetDpiForWindow,
    };
    let RawWindowHandle::Win32(handle) = window.window_handle().ok()?.as_raw() else {
        return None;
    };
    let hwnd = HWND(handle.hwnd.get() as *mut _);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..MONITORINFO::default()
    };
    // SAFETY: `hwnd` is the live window iced handed us for this call, and
    // `info` is a correctly sized MONITORINFO.
    let dpi = unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return None;
        }
        GetDpiForWindow(hwnd)
    };
    let scale = if dpi == 0 { 1.0 } else { dpi as f32 / 96.0 };
    let work = info.rcWork;
    Some((
        (work.right - work.left) as f32 / scale,
        (work.bottom - work.top) as f32 / scale,
    ))
}

/// Whether a physical-pixel desktop point lies on a connected monitor.
pub fn point_on_screen(x: i32, y: i32) -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::{
            Foundation::POINT,
            Graphics::Gdi::{MonitorFromPoint, MONITOR_DEFAULTTONULL},
        };
        // SAFETY: a plain query with no pointers.
        !unsafe { MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONULL) }.is_invalid()
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        let _ = (x, y);
        true
    }
    #[cfg(target_os = "macos")]
    {
        point_on_screen_macos(x, y)
    }
}

/// Run `f` on AppKit's real main thread. Never fabricates a
/// [`MainThreadMarker`] on a worker — that still executes AppKit off-thread.
#[cfg(target_os = "macos")]
pub(crate) fn with_appkit_main<F, R>(f: F) -> R
where
    F: FnOnce(objc2::MainThreadMarker) -> R + Send,
    R: Send,
{
    use dispatch2::run_on_main;
    use objc2::MainThreadMarker;

    if let Some(mtm) = MainThreadMarker::new() {
        f(mtm)
    } else {
        // iced owns the process main queue while the app runs. Unit tests that
        // need AppKit must not call this without a main run loop (`run_on_main`
        // would block forever); prefer CoreGraphics-only paths in those tests.
        run_on_main(f)
    }
}

/// Hit-test with CoreGraphics only (no AppKit / main-thread requirement).
/// Uses the same global desktop space as iced/winit `WindowEvent::Moved`
/// (origin at the top-left of the main display, y increasing downward).
#[cfg(target_os = "macos")]
fn point_on_screen_macos(x: i32, y: i32) -> bool {
    use objc2_core_graphics::{
        CGDisplayBounds, CGError, CGGetActiveDisplayList, CGDirectDisplayID,
    };

    let mut count = 0u32;
    // SAFETY: null `active_displays` is allowed when only querying the count.
    let err = unsafe { CGGetActiveDisplayList(0, std::ptr::null_mut(), &mut count) };
    if err != CGError::Success || count == 0 {
        return false;
    }
    let mut displays = vec![0 as CGDirectDisplayID; count as usize];
    // SAFETY: buffer length matches `count` from the prior query.
    let err = unsafe { CGGetActiveDisplayList(count, displays.as_mut_ptr(), &mut count) };
    if err != CGError::Success {
        return false;
    }
    let (x, y) = (x as f64, y as f64);
    displays[..count as usize].iter().any(|&id| {
        let bounds = CGDisplayBounds(id);
        x >= bounds.origin.x
            && x < bounds.origin.x + bounds.size.width
            && y >= bounds.origin.y
            && y < bounds.origin.y + bounds.size.height
    })
}

/// The name of the event an instance holds while `path` is its open file.
#[cfg(windows)]
fn instance_event_name(path: &std::path::Path) -> Vec<u16> {
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let key = path.to_string_lossy().to_lowercase();
    format!(
        "Local\\SketchRust-open-{}",
        sketch_core::skx::sha256_hex(key.as_bytes())
    )
    .encode_utf16()
    .chain(Some(0))
    .collect()
}

/// If another running instance has `path` open, bring its window forward and
/// return true: two windows on one file would overwrite each other's saves.
pub fn activate_existing(path: &std::path::Path) -> bool {
    if cfg!(test) {
        return false;
    }
    #[cfg(windows)]
    {
        use windows::{
            core::PCWSTR,
            Win32::{
                Foundation::CloseHandle,
                System::Threading::{OpenEventW, SetEvent, EVENT_MODIFY_STATE},
                UI::WindowsAndMessaging::{AllowSetForegroundWindow, ASFW_ANY},
            },
        };
        let name = instance_event_name(path);
        // SAFETY: `name` is a live NUL-terminated buffer; the handle is
        // closed before returning.
        unsafe {
            let Ok(event) = OpenEventW(EVENT_MODIFY_STATE, false, PCWSTR(name.as_ptr())) else {
                return false;
            };
            // Let the other instance take the foreground from us.
            let _ = AllowSetForegroundWindow(ASFW_ANY);
            let signalled = SetEvent(event).is_ok();
            let _ = CloseHandle(event);
            signalled
        }
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}

/// Holds the "this file is open here" event for `path` for as long as the
/// stream lives, and yields each time another instance asks for the window.
#[allow(clippy::ptr_arg)]
pub fn activation_stream(path: &std::path::PathBuf) -> impl iced::futures::Stream<Item = ()> {
    let (tx, rx) = iced::futures::channel::mpsc::unbounded::<()>();
    #[cfg(windows)]
    {
        let name = instance_event_name(path);
        std::thread::spawn(move || {
            use windows::{
                core::PCWSTR,
                Win32::{
                    Foundation::{CloseHandle, WAIT_OBJECT_0},
                    System::Threading::{CreateEventW, WaitForSingleObject},
                },
            };
            // SAFETY: `name` outlives the call; the handle is owned by this
            // thread and closed when the stream is dropped.
            unsafe {
                let Ok(event) = CreateEventW(None, false, false, PCWSTR(name.as_ptr())) else {
                    return;
                };
                while !tx.is_closed() {
                    if WaitForSingleObject(event, 500) == WAIT_OBJECT_0
                        && tx.unbounded_send(()).is_err()
                    {
                        break;
                    }
                }
                let _ = CloseHandle(event);
            }
        });
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        // Keep the sender alive so the stream stays pending.
        std::mem::forget(tx);
    }
    rx
}

#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    #[test]
    fn point_on_screen_accepts_origin_of_main_screen() {
        // Top-left of the main display in winit/iced desktop coordinates is on-screen.
        assert!(
            crate::platform::point_on_screen(0, 0),
            "expected (0,0) in screen space to hit a display on CI/dev Macs"
        );
    }

    #[test]
    fn point_on_screen_rejects_far_away_coordinate() {
        assert!(!crate::platform::point_on_screen(i32::MAX / 2, i32::MAX / 2));
    }
}

// ---------------------------------------------------------------------------
// File pickers. Every native file dialog goes through here so a port only
// has to replace this block (Android: the Storage Access Framework).
// ---------------------------------------------------------------------------

/// Runs a blocking native dialog on a `file-dialog` thread of its own and
/// hands the result to the returned future. The interface thread never
/// pumps or blocks: it awaits, and the iced event loop keeps running.
///
/// The pump architecture this replaces (0.3.8-0.3.10) polled the dialog
/// thread from the interface thread while hand-pumping its Windows message
/// queue; nested inside a winit dispatch, that pump wedged the Shell's
/// dialog initialization (the dialog thread stuck in `NtUserGetMessage`
/// inside windows.storage/SHCore, no dialog window ever created) and the
/// app froze. A spawned thread running the sync dialog — awaited, never
/// pumped for — is the known-good architecture (the D repo ran it this
/// way).
fn dialog_on_thread<T: Send + 'static>(
    show: impl FnOnce() -> Option<T> + Send + 'static,
) -> impl std::future::Future<Output = Option<T>> {
    let (sender, receiver) = iced::futures::channel::oneshot::channel();
    // A failed spawn drops `sender`, which resolves the future to `None`.
    let _ = std::thread::Builder::new()
        .name("file-dialog".into())
        .spawn(move || {
            let _ = sender.send(show());
        });
    async move { receiver.await.unwrap_or_default() }
}

/// A known-local folder the native file dialogs start in. Left unset, the
/// Windows file dialog restores its own last-visited location; when that is
/// a network share the Shell's folder initialization can stall indefinitely
/// (SMB enumeration) and the dialog window is never created — the
/// app appears frozen the moment a dialog opens (File > Open in 0.3.8,
/// Save in 0.3.9). An explicit directory keeps the dialog off remembered
/// network locations.
fn dialog_start_dir() -> Option<std::path::PathBuf> {
    let profile = std::path::PathBuf::from(std::env::var_os("USERPROFILE")?);
    let documents = profile.join("Documents");
    Some(if documents.is_dir() {
        documents
    } else {
        profile
    })
}

/// A native dialog builder that accepts an explicit start folder. The trait
/// exists so tests can pin the anchoring behaviour without opening a real
/// dialog.
trait NativeDialog {
    fn set_directory(self, dir: std::path::PathBuf) -> Self;
}

impl NativeDialog for rfd::FileDialog {
    fn set_directory(self, dir: std::path::PathBuf) -> Self {
        rfd::FileDialog::set_directory(self, dir)
    }
}

/// Anchor a dialog to an explicit start folder, so Windows never restores
/// its own remembered location at dialog init. `None` leaves the dialog
/// exactly as built.
fn with_start_dir<D: NativeDialog>(dialog: D, dir: Option<std::path::PathBuf>) -> D {
    match dir {
        Some(dir) => dialog.set_directory(dir),
        None => dialog,
    }
}

/// The only construction point of an rfd dialog: anchored to an explicit
/// local start folder (see `dialog_start_dir`).
fn sync_dialog() -> rfd::FileDialog {
    with_start_dir(rfd::FileDialog::new(), dialog_start_dir())
}

/// Ask for one existing file whose extension is in `extensions`.
pub async fn pick_file_async(
    filter: String,
    extensions: Vec<String>,
) -> Option<std::path::PathBuf> {
    dialog_on_thread(move || sync_dialog().add_filter(filter, &extensions).pick_file()).await
}

/// Ask for one or more existing files.
#[cfg_attr(test, allow(dead_code))]
pub async fn pick_files_async(
    filter: String,
    extensions: Vec<String>,
) -> Option<Vec<std::path::PathBuf>> {
    dialog_on_thread(move || sync_dialog().add_filter(filter, &extensions).pick_files()).await
}

/// Ask where to save, offering `default_name`.
pub async fn save_file_async(
    filter: String,
    extensions: Vec<String>,
    default_name: String,
) -> Option<std::path::PathBuf> {
    dialog_on_thread(move || {
        sync_dialog()
            .add_filter(filter, &extensions)
            .set_file_name(default_name)
            .save_file()
    })
    .await
}

/// Ask where to save, offering one `(name, extension)` filter per format in
/// the order given.
#[cfg_attr(test, allow(dead_code))]
pub async fn save_file_as_async(
    default_name: String,
    filters: Vec<(String, String)>,
) -> Option<std::path::PathBuf> {
    dialog_on_thread(move || {
        let mut dialog = sync_dialog().set_file_name(default_name);
        for (name, ext) in &filters {
            dialog = dialog.add_filter(name, &[ext.as_str()]);
        }
        dialog.save_file()
    })
    .await
}

#[cfg(test)]
mod dialog_tests {
    use super::*;

    /// A dialog stand-in that records the folder it was anchored to, so the
    /// anchoring behaviour is testable without opening a real dialog.
    struct RecordingDialog(Option<std::path::PathBuf>);

    impl NativeDialog for RecordingDialog {
        fn set_directory(self, dir: std::path::PathBuf) -> Self {
            RecordingDialog(Some(dir))
        }
    }

    #[test]
    fn with_start_dir_applies_the_given_local_directory() {
        let dir = std::env::temp_dir();
        let anchored = with_start_dir(RecordingDialog(None), Some(dir.clone()));
        assert_eq!(
            anchored.0,
            Some(dir),
            "a provided directory must reach the dialog builder"
        );
        let untouched = with_start_dir(RecordingDialog(None), None);
        assert_eq!(
            untouched.0, None,
            "without a directory the dialog must be left exactly as built"
        );
    }

    #[test]
    fn dialogs_never_pump_the_interface_thread() {
        // Only the production half of this file is under contract.
        let production = include_str!("platform.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the test module sits after the production code");
        assert!(
            !production.contains("PeekMessageW"),
            "the interface thread must never hand-pump Windows messages while a \
             dialog is open: the nested pump wedged the Shell's dialog \
             initialization (dialog thread stuck in NtUserGetMessage inside \
             windows.storage/SHCore, no dialog window created) and froze the \
             app in 0.3.8-0.3.10. The iced event loop owns message dispatch; a \
             dialog is awaited, never pumped for."
        );
        assert!(
            !production.contains("fn on_dialog_thread"),
            "dialogs must not be waited on from the interface thread; the \
             runtime awaits a spawned dialog thread while the event loop runs"
        );
    }

    #[test]
    fn every_native_dialog_is_anchored_to_an_explicit_start_directory() {
        // Only the production half of this file is under contract; this
        // module's own probe strings must not count towards it.
        let production = include_str!("platform.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the test module sits after the production code");
        // The only legal construction of an rfd dialog is inside
        // `sync_dialog`, which anchors through `with_start_dir`. Counting
        // constructions (rather than call formatting) keeps the oracle
        // stable under rustfmt.
        let sync_built = production.matches("rfd::FileDialog::new()").count();
        assert_eq!(
            sync_built, 1,
            "rfd::FileDialog::new() may only appear inside sync_dialog(), which \
             anchors it to an explicit start folder so Windows never restores its \
             remembered location at dialog init: a remembered network share wedges \
             the Shell's folder initialization and froze the app (0.3.8 File > Open, \
             0.3.9 Save)"
        );
        assert!(
            !production.contains("AsyncFileDialog"),
            "every native dialog runs on the `file-dialog` thread through \
             dialog_on_thread(); rfd's AsyncFileDialog is gone, and the sync \
             builder it replaced is anchored the same way (see dialog_start_dir)"
        );
    }
}
