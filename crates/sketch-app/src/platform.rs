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
    #[cfg(not(windows))]
    {
        let _ = (x, y);
        true
    }
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
