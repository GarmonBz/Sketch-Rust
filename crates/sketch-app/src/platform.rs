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
        Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST},
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

#[cfg(target_os = "macos")]
fn with_appkit_main<F, R>(f: F) -> R
where
    F: FnOnce(objc2::MainThreadMarker) -> R + Send,
    R: Send,
{
    use dispatch2::run_on_main;
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;

    if let Some(mtm) = MainThreadMarker::new() {
        return f(mtm);
    }
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let app = NSApplication::sharedApplication(mtm);
    if app.isRunning() {
        run_on_main(f)
    } else {
        // Headless unit tests: no run loop, so `dispatch2` would block forever.
        f(mtm)
    }
}

#[cfg(target_os = "macos")]
fn point_on_screen_macos(x: i32, y: i32) -> bool {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;
    use objc2_core_graphics::{CGDisplayBounds, CGMainDisplayID};

    with_appkit_main(|mtm: MainThreadMarker| {
        if NSScreen::screens(mtm).is_empty() {
            return false;
        }
        let main_height = CGDisplayBounds(CGMainDisplayID()).size.height;
        let (x, y) = (x as f64, y as f64);
        NSScreen::screens(mtm).iter().any(|screen| {
            cg_point_in_appkit_frame(x, y, screen.frame(), main_height)
        })
    })
}

/// Hit-test in the same global desktop space as iced/winit `WindowEvent::Moved`
/// (CG-style: origin top-left of the main display, y increasing downward).
#[cfg(target_os = "macos")]
fn cg_point_in_appkit_frame(
    x: f64,
    y: f64,
    frame: objc2_foundation::NSRect,
    main_height: f64,
) -> bool {
    let top = main_height - frame.origin.y - frame.size.height;
    let left = frame.origin.x;
    x >= left
        && x < left + frame.size.width
        && y >= top
        && y < top + frame.size.height
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

/// Ask for one existing file whose extension is in `extensions`.
pub fn pick_file(filter: &str, extensions: &[&str]) -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .add_filter(filter, extensions)
        .pick_file()
}

/// Ask for one or more existing files.
#[cfg_attr(test, allow(dead_code))]
pub fn pick_files(filter: &str, extensions: &[&str]) -> Option<Vec<std::path::PathBuf>> {
    rfd::FileDialog::new()
        .add_filter(filter, extensions)
        .pick_files()
}

/// Ask where to save, offering `default_name`.
pub fn save_file(
    filter: &str,
    extensions: &[&str],
    default_name: impl Into<String>,
) -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .add_filter(filter, extensions)
        .set_file_name(default_name)
        .save_file()
}

/// Ask where to save, offering one `(name, extension)` filter per format in
/// the order given.
#[cfg_attr(test, allow(dead_code))]
pub fn save_file_as(default_name: &str, filters: &[(&str, &str)]) -> Option<std::path::PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_file_name(default_name);
    for (name, ext) in filters {
        dialog = dialog.add_filter(*name, &[*ext]);
    }
    dialog.save_file()
}

/// `pick_file` without blocking the interface thread.
#[cfg_attr(test, allow(dead_code))]
pub async fn pick_file_async(filter: &str, extensions: &[&str]) -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter(filter, extensions)
        .pick_file()
        .await
        .map(|file| file.path().to_path_buf())
}

/// `save_file` without blocking the interface thread.
#[cfg_attr(test, allow(dead_code))]
pub async fn save_file_async(
    filter: &str,
    extensions: &[&str],
    default_name: &str,
) -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter(filter, extensions)
        .set_file_name(default_name)
        .save_file()
        .await
        .map(|file| file.path().to_path_buf())
}
