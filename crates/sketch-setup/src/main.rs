//! Sketch Rust setup: a small per-user installer.
//!
//! It copies the embedded package into `%LOCALAPPDATA%\Programs\SketchRust`,
//! adds Start Menu and desktop shortcuts, and registers an entry under
//! "Installed apps" so Windows can remove it. Nothing needs administrator
//! rights and nothing is written outside the user's profile.
//!
//! Command line:
//! - no arguments: install, asking first.
//! - `/S`: install or uninstall without any message boxes.
//! - `/D=<folder>`: install there instead of the default folder.
//! - `--uninstall`: remove the installation this exe sits in (the installed
//!   copy is `uninstall.exe`).
//! - `--selftest <folder>`: install into, verify and remove from a scratch
//!   folder, with its own shortcut folders and registry key, and write
//!   `selftest.txt` there. Used by `tools/installer.ps1` to check a build.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::io::Read;
use std::path::{Path, PathBuf};

const APP_NAME: &str = "Sketch Rust";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_EXE: &str = "sketch-app.exe";
const UNINSTALLER: &str = "uninstall.exe";
/// The names this installer wrote, one per line, so a later uninstaller
/// removes files a newer payload no longer carries.
const MANIFEST: &str = "install-manifest.txt";
const REG_KEY: &str = "SketchRust";
static PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.bin"));

/// Where one installation lives. The real one uses the user's folders; the
/// self-test points every part at a scratch folder.
struct Target {
    dir: PathBuf,
    /// Folders that each get a `Sketch Rust.lnk`.
    shortcut_dirs: Vec<PathBuf>,
    /// Subkey of HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall.
    reg_key: String,
}

fn unpack(blob: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut raw = Vec::new();
    flate2::read::DeflateDecoder::new(blob)
        .read_to_end(&mut raw)
        .map_err(|e| format!("The installer's contents are damaged: {e}"))?;
    let damaged = || "The installer's contents are damaged.".to_string();
    let mut files = Vec::new();
    let mut at = 0usize;
    while at < raw.len() {
        let name_len =
            u16::from_le_bytes(raw.get(at..at + 2).ok_or_else(damaged)?.try_into().unwrap())
                as usize;
        at += 2;
        let name = std::str::from_utf8(raw.get(at..at + name_len).ok_or_else(damaged)?)
            .map_err(|_| damaged())?
            .to_string();
        at += name_len;
        let len = u64::from_le_bytes(raw.get(at..at + 8).ok_or_else(damaged)?.try_into().unwrap())
            as usize;
        at += 8;
        let bytes = raw.get(at..at + len).ok_or_else(damaged)?.to_vec();
        at += len;
        // Flat package: a name is a plain file name, never a path.
        if name.is_empty() || name.contains(['/', '\\', ':']) || name == ".." {
            return Err(damaged());
        }
        files.push((name, bytes));
    }
    Ok(files)
}

/// Writes `bytes` to `path` through a sibling temp file, so a failed write
/// never leaves a half-written program file behind.
fn write_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temp = path.with_extension("setup-new");
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })
}

/// Copies the package into place. Returns notes for the closing message.
/// `retry` is asked when a file is locked (Sketch still running).
fn install(
    target: &Target,
    files: &[(String, Vec<u8>)],
    retry: &dyn Fn(&str) -> bool,
) -> Result<Vec<String>, String> {
    if !files.iter().any(|(name, _)| name == APP_EXE) {
        return Err("This installer was built without its program files.".into());
    }
    let mut notes = Vec::new();
    std::fs::create_dir_all(&target.dir)
        .map_err(|e| format!("Could not create {}: {e}", target.dir.display()))?;
    for (name, bytes) in files {
        let path = target.dir.join(name);
        // The symbol library is edited in place by Manage symbols: keep the
        // user's copy beside the new one rather than silently replacing it.
        if name == "symbols.json" {
            if let Ok(existing) = std::fs::read(&path) {
                if existing != *bytes {
                    let backup = format!("symbols.before-{VERSION}.json");
                    std::fs::write(target.dir.join(&backup), existing)
                        .map_err(|e| format!("Could not back up symbols.json: {e}"))?;
                    notes.push(format!(
                        "Your previous symbol library was kept as {backup} in the install folder."
                    ));
                }
            }
        }
        loop {
            match write_file(&path, bytes) {
                Ok(()) => break,
                Err(e) => {
                    let message = format!(
                        "Could not write {name}: {e}\n\nIf Sketch is running, close it and try again."
                    );
                    if !retry(&message) {
                        return Err(format!("Could not write {name}: {e}"));
                    }
                }
            }
        }
    }
    let mut names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    names.push(UNINSTALLER);
    names.push(MANIFEST);
    std::fs::write(target.dir.join(MANIFEST), names.join("\r\n"))
        .map_err(|e| format!("Could not write {MANIFEST}: {e}"))?;
    let me = std::env::current_exe().map_err(|e| e.to_string())?;
    let uninstaller = target.dir.join(UNINSTALLER);
    if me != uninstaller {
        std::fs::copy(&me, &uninstaller)
            .map_err(|e| format!("Could not write {UNINSTALLER}: {e}"))?;
    }
    let exe = target.dir.join(APP_EXE);
    for dir in &target.shortcut_dirs {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        platform::shortcut(&dir.join(format!("{APP_NAME}.lnk")), &exe, &target.dir)
            .map_err(|e| format!("Could not create a shortcut in {}: {e}", dir.display()))?;
    }
    let size_kb = files.iter().map(|(_, b)| b.len() as u64).sum::<u64>() / 1024;
    platform::register(&target.reg_key, &target.dir, size_kb)
        .map_err(|e| format!("Could not register the uninstaller: {e}"))?;
    Ok(notes)
}

/// Removes what `install` wrote. The running uninstaller cannot delete
/// itself; the caller deals with that file and the then-empty folder.
/// Anything else in the folder (library backups the user made) is left.
fn uninstall(target: &Target) -> Result<(), String> {
    for dir in &target.shortcut_dirs {
        let _ = std::fs::remove_file(dir.join(format!("{APP_NAME}.lnk")));
    }
    platform::unregister(&target.reg_key);
    let listed = std::fs::read_to_string(target.dir.join(MANIFEST)).unwrap_or_default();
    let me = std::env::current_exe().ok();
    let mut failed = Vec::new();
    for name in listed.lines().map(str::trim).filter(|n| !n.is_empty()) {
        if name.contains(['/', '\\', ':']) {
            continue;
        }
        let path = target.dir.join(name);
        if me.as_deref() == Some(path.as_path()) || name == MANIFEST {
            continue;
        }
        if path.exists() && std::fs::remove_file(&path).is_err() {
            failed.push(name.to_string());
        }
    }
    if !failed.is_empty() {
        return Err(format!(
            "Could not remove: {}.\n\nIf Sketch is running, close it and uninstall again.",
            failed.join(", ")
        ));
    }
    let _ = std::fs::remove_file(target.dir.join(MANIFEST));
    Ok(())
}

fn default_target(dir_override: Option<PathBuf>) -> Result<Target, String> {
    let dir = match dir_override {
        Some(dir) => dir,
        None => PathBuf::from(
            std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is not set for this user.")?,
        )
        .join("Programs")
        .join("SketchRust"),
    };
    Ok(Target {
        dir,
        shortcut_dirs: vec![platform::programs_folder()?, platform::desktop_folder()?],
        reg_key: REG_KEY.into(),
    })
}

fn run_install(silent: bool, dir_override: Option<PathBuf>) -> Result<(), String> {
    let files = unpack(PAYLOAD)?;
    let target = default_target(dir_override)?;
    let updating = target.dir.join(APP_EXE).is_file();
    if !silent
        && !platform::ask(&format!(
            "{} {APP_NAME} {VERSION} for this Windows user?\n\nFolder: {}\n\nNo administrator permission is needed. Your sketches, settings and recent files are not touched.",
            if updating { "Update to" } else { "Install" },
            target.dir.display()
        ))
    {
        return Ok(());
    }
    let retry = |message: &str| !silent && platform::retry(message);
    let notes = install(&target, &files, &retry)?;
    if silent {
        return Ok(());
    }
    let mut message = format!(
        "{APP_NAME} {VERSION} is installed.\n\nStart it from the Start Menu or the desktop shortcut. Remove it from Windows Settings > Apps > Installed apps."
    );
    for note in &notes {
        message.push_str("\n\n");
        message.push_str(note);
    }
    message.push_str("\n\nIf you use Appraise-It integration, open Help > Appraise-It Integration in this copy and click Enable integration, then restart Appraise-It Pro.\n\nStart Sketch now?");
    if platform::ask(&message) {
        std::process::Command::new(target.dir.join(APP_EXE))
            .current_dir(&target.dir)
            .spawn()
            .map_err(|e| format!("Installed, but Sketch could not be started: {e}"))?;
    }
    Ok(())
}

fn run_uninstall(silent: bool) -> Result<(), String> {
    let me = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = me.parent().ok_or("No install folder.")?.to_path_buf();
    if !dir.join(MANIFEST).is_file() {
        return Err(format!(
            "{} is not a {APP_NAME} installation.",
            dir.display()
        ));
    }
    if !silent
        && !platform::ask(&format!(
            "Remove {APP_NAME} from this computer?\n\nFolder: {}\n\nYour sketches, settings and recent files are kept.\n\nIf Appraise-It integration is enabled for this copy, cancel, open Help > Appraise-It Integration and click Restore previous provider first.",
            dir.display()
        ))
    {
        return Ok(());
    }
    let target = default_target(Some(dir.clone()))?;
    uninstall(&target)?;
    platform::remove_after_exit(&me, &dir);
    if !silent {
        platform::info(&format!("{APP_NAME} was removed."));
    }
    Ok(())
}

/// Installs into, checks and removes from a scratch folder. Shortcuts go to
/// folders inside it and the registry entry gets its own key, so the user's
/// Start Menu, desktop and real entry are untouched.
fn run_selftest(root: &Path) -> Result<String, String> {
    let files = unpack(PAYLOAD)?;
    let target = Target {
        dir: root.join("app"),
        shortcut_dirs: vec![root.join("menu"), root.join("desktop")],
        reg_key: format!("{REG_KEY}-selftest"),
    };
    let never = |_: &str| false;
    install(&target, &files, &never)?;
    for (name, bytes) in &files {
        let written = std::fs::read(target.dir.join(name)).map_err(|e| format!("{name}: {e}"))?;
        if written != *bytes {
            return Err(format!("{name} differs after install"));
        }
    }
    for dir in &target.shortcut_dirs {
        let link = dir.join(format!("{APP_NAME}.lnk"));
        if std::fs::metadata(&link).map(|m| m.len()).unwrap_or(0) == 0 {
            return Err(format!("no shortcut at {}", link.display()));
        }
    }
    if !target.dir.join(UNINSTALLER).is_file() {
        return Err("no uninstaller copied".into());
    }
    let registered = platform::registered_location(&target.reg_key);
    if registered.as_deref() != Some(target.dir.as_path()) {
        return Err(format!("registry InstallLocation is {registered:?}"));
    }
    // A second install over the first, with an edited library: the edit must
    // survive as a backup and the new library must be in place.
    let has_library = files.iter().any(|(name, _)| name == "symbols.json");
    if has_library {
        std::fs::write(target.dir.join("symbols.json"), b"{\"edited\":true}")
            .map_err(|e| e.to_string())?;
        let notes = install(&target, &files, &never)?;
        let backup = target.dir.join(format!("symbols.before-{VERSION}.json"));
        if notes.len() != 1 || std::fs::read(&backup).ok().as_deref() != Some(b"{\"edited\":true}")
        {
            return Err("an edited symbol library was not backed up on update".into());
        }
        let _ = std::fs::remove_file(backup);
    }
    uninstall(&target)?;
    let _ = std::fs::remove_file(target.dir.join(UNINSTALLER));
    let left: Vec<String> = std::fs::read_dir(&target.dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    if !left.is_empty() {
        return Err(format!("left behind after uninstall: {left:?}"));
    }
    for dir in &target.shortcut_dirs {
        if dir.join(format!("{APP_NAME}.lnk")).exists() {
            return Err("a shortcut survived uninstall".into());
        }
    }
    if platform::registered_location(&target.reg_key).is_some() {
        return Err("the registry entry survived uninstall".into());
    }
    Ok(format!(
        "ok: {} files, version {VERSION}, library update {}",
        files.len(),
        if has_library { "checked" } else { "not in payload" }
    ))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let silent = args.iter().any(|a| a.eq_ignore_ascii_case("/S"));
    if let Some(i) = args.iter().position(|a| a == "--selftest") {
        let Some(root) = args.get(i + 1).map(PathBuf::from) else {
            std::process::exit(2);
        };
        let result = run_selftest(&root);
        let _ = std::fs::write(
            root.join("selftest.txt"),
            match &result {
                Ok(text) => text.clone(),
                Err(text) => format!("FAILED: {text}"),
            },
        );
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }
    let dir_override = args
        .iter()
        .find_map(|a| a.strip_prefix("/D=").or_else(|| a.strip_prefix("/d=")))
        .map(PathBuf::from);
    let result = if args.iter().any(|a| a == "--uninstall") {
        run_uninstall(silent)
    } else {
        run_install(silent, dir_override)
    };
    if let Err(message) = result {
        if !silent {
            platform::error(&message);
        }
        std::process::exit(1);
    }
}

#[cfg(windows)]
mod platform {
    use super::{APP_EXE, APP_NAME, UNINSTALLER, VERSION};
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use windows::core::{w, Interface, GUID, PCWSTR};
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegGetValueW, RegSetValueExW, HKEY,
        HKEY_CURRENT_USER, KEY_WRITE, REG_DWORD, REG_OPTION_NON_VOLATILE, REG_SZ, REG_VALUE_TYPE,
        RRF_RT_REG_SZ,
    };
    use windows::Win32::UI::Shell::{
        FOLDERID_Desktop, FOLDERID_Programs, IShellLinkW, SHGetKnownFolderPath, ShellLink,
        KF_FLAG_DEFAULT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        MessageBoxW, IDOK, IDRETRY, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION,
        MB_ICONWARNING, MB_OK, MB_RETRYCANCEL, MB_YESNO, MESSAGEBOX_STYLE,
    };

    const UNINSTALL_ROOT: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\";

    fn wide(s: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
        s.as_ref().encode_wide().chain(Some(0)).collect()
    }

    fn message(text: &str, style: MESSAGEBOX_STYLE) -> i32 {
        let text = wide(text);
        let title = wide(format!("{APP_NAME} {VERSION} Setup"));
        // SAFETY: both buffers are NUL-terminated and outlive the call.
        unsafe { MessageBoxW(None, PCWSTR(text.as_ptr()), PCWSTR(title.as_ptr()), style).0 }
    }
    pub fn ask(text: &str) -> bool {
        message(text, MB_YESNO | MB_ICONQUESTION) == IDYES.0
    }
    pub fn retry(text: &str) -> bool {
        message(text, MB_RETRYCANCEL | MB_ICONWARNING) == IDRETRY.0
    }
    pub fn info(text: &str) {
        let _ = message(text, MB_OK | MB_ICONINFORMATION) == IDOK.0;
    }
    pub fn error(text: &str) {
        let _ = message(text, MB_OK | MB_ICONERROR);
    }

    fn known_folder(id: &GUID) -> Result<PathBuf, String> {
        // SAFETY: the returned buffer is a NUL-terminated string owned by
        // the shell until freed with CoTaskMemFree, which happens here.
        unsafe {
            let path = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).map_err(|e| e.to_string())?;
            let folder = PathBuf::from(String::from_utf16_lossy(path.as_wide()));
            CoTaskMemFree(Some(path.0 as *const core::ffi::c_void));
            Ok(folder)
        }
    }
    pub fn programs_folder() -> Result<PathBuf, String> {
        known_folder(&FOLDERID_Programs)
    }
    pub fn desktop_folder() -> Result<PathBuf, String> {
        known_folder(&FOLDERID_Desktop)
    }

    /// Writes a `.lnk` at `link` that starts `exe` in `folder`.
    pub fn shortcut(link: &Path, exe: &Path, folder: &Path) -> Result<(), String> {
        let (link, exe, folder) = (wide(link), wide(exe), wide(folder));
        // SAFETY: COM is initialized on this thread first (a repeat call is
        // harmless); every string is NUL-terminated and outlives its call.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let shell_link: IShellLinkW =
                CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).map_err(|e| e.to_string())?;
            shell_link.SetPath(PCWSTR(exe.as_ptr())).map_err(|e| e.to_string())?;
            shell_link
                .SetWorkingDirectory(PCWSTR(folder.as_ptr()))
                .map_err(|e| e.to_string())?;
            shell_link
                .SetDescription(w!("Appraisal Sketch"))
                .map_err(|e| e.to_string())?;
            shell_link
                .SetIconLocation(PCWSTR(exe.as_ptr()), 0)
                .map_err(|e| e.to_string())?;
            let file: IPersistFile = shell_link.cast().map_err(|e| e.to_string())?;
            file.Save(PCWSTR(link.as_ptr()), true).map_err(|e| e.to_string())
        }
    }

    fn set(key: HKEY, name: &str, kind: REG_VALUE_TYPE, data: &[u8]) -> Result<(), String> {
        let name = wide(name);
        // SAFETY: `key` is an open key; the buffers outlive the call.
        let status = unsafe { RegSetValueExW(key, PCWSTR(name.as_ptr()), None, kind, Some(data)) };
        if status == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!("registry error {}", status.0))
        }
    }
    fn set_text(key: HKEY, name: &str, value: &str) -> Result<(), String> {
        let bytes: Vec<u8> = wide(value).iter().flat_map(|u| u.to_le_bytes()).collect();
        set(key, name, REG_SZ, &bytes)
    }

    /// The "Installed apps" entry, under the current user only.
    pub fn register(reg_key: &str, dir: &Path, size_kb: u64) -> Result<(), String> {
        let subkey = wide(format!("{UNINSTALL_ROOT}{reg_key}"));
        let mut key = HKEY::default();
        // SAFETY: `subkey` is NUL-terminated; `key` receives the handle and
        // is closed below.
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                None,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                None,
                &mut key,
                None,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!("registry error {}", status.0));
        }
        let uninstaller = dir.join(UNINSTALLER);
        let result = (|| {
            set_text(key, "DisplayName", APP_NAME)?;
            set_text(key, "DisplayVersion", VERSION)?;
            set_text(key, "Publisher", APP_NAME)?;
            set_text(key, "InstallLocation", &dir.to_string_lossy())?;
            set_text(key, "DisplayIcon", &dir.join(APP_EXE).to_string_lossy())?;
            set_text(
                key,
                "UninstallString",
                &format!("\"{}\" --uninstall", uninstaller.display()),
            )?;
            set_text(
                key,
                "QuietUninstallString",
                &format!("\"{}\" --uninstall /S", uninstaller.display()),
            )?;
            set(key, "NoModify", REG_DWORD, &1u32.to_le_bytes())?;
            set(key, "NoRepair", REG_DWORD, &1u32.to_le_bytes())?;
            set(
                key,
                "EstimatedSize",
                REG_DWORD,
                &(size_kb.min(u32::MAX as u64) as u32).to_le_bytes(),
            )
        })();
        // SAFETY: `key` was opened above.
        unsafe {
            let _ = RegCloseKey(key);
        }
        result
    }

    pub fn unregister(reg_key: &str) {
        let subkey = wide(format!("{UNINSTALL_ROOT}{reg_key}"));
        // SAFETY: `subkey` is NUL-terminated. Only this app's own subkey is
        // named, never the Uninstall root.
        unsafe {
            let _ = RegDeleteTreeW(HKEY_CURRENT_USER, PCWSTR(subkey.as_ptr()));
            // RegDeleteTreeW empties the key; this removes the key itself.
            let _ = windows::Win32::System::Registry::RegDeleteKeyW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
            );
        }
    }

    /// The registered InstallLocation, if the entry exists.
    pub fn registered_location(reg_key: &str) -> Option<PathBuf> {
        let subkey = wide(format!("{UNINSTALL_ROOT}{reg_key}"));
        let mut buffer = [0u16; 1024];
        let mut size = (buffer.len() * 2) as u32;
        // SAFETY: `buffer` is `size` bytes long and receives a NUL-terminated
        // string of at most that many bytes.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                w!("InstallLocation"),
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&mut size),
            )
        };
        if status != ERROR_SUCCESS {
            return None;
        }
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        Some(PathBuf::from(String::from_utf16_lossy(&buffer[..len])))
    }

    /// A running exe cannot delete itself: hand the last two steps (this
    /// file, then the folder if it is empty) to a hidden command prompt
    /// that waits for this process to end. `rmdir` without `/S` never
    /// removes a folder that still holds something.
    pub fn remove_after_exit(exe: &Path, dir: &Path) {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let script = format!(
            "/C \"ping -n 3 127.0.0.1 >NUL & del /F /Q \"{}\" & rmdir \"{}\"\"",
            exe.display(),
            dir.display()
        );
        let _ = std::process::Command::new("cmd.exe")
            .raw_arg(script)
            .current_dir(std::env::temp_dir())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
}

#[cfg(not(windows))]
mod platform {
    use std::path::{Path, PathBuf};
    const WINDOWS_ONLY: &str = "The installer only runs on Windows.";
    pub fn ask(_: &str) -> bool {
        false
    }
    pub fn retry(_: &str) -> bool {
        false
    }
    pub fn info(_: &str) {}
    pub fn error(text: &str) {
        eprintln!("{text}");
    }
    pub fn programs_folder() -> Result<PathBuf, String> {
        Err(WINDOWS_ONLY.into())
    }
    pub fn desktop_folder() -> Result<PathBuf, String> {
        Err(WINDOWS_ONLY.into())
    }
    pub fn shortcut(_: &Path, _: &Path, _: &Path) -> Result<(), String> {
        Err(WINDOWS_ONLY.into())
    }
    pub fn register(_: &str, _: &Path, _: u64) -> Result<(), String> {
        Err(WINDOWS_ONLY.into())
    }
    pub fn unregister(_: &str) {}
    pub fn registered_location(_: &str) -> Option<PathBuf> {
        None
    }
    pub fn remove_after_exit(_: &Path, _: &Path) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn pack(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut raw = Vec::new();
        for (name, bytes) in files {
            raw.extend_from_slice(&(name.len() as u16).to_le_bytes());
            raw.extend_from_slice(name.as_bytes());
            raw.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            raw.extend_from_slice(bytes);
        }
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&raw).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn unpack_round_trips_the_build_scripts_layout() {
        let blob = pack(&[("sketch-app.exe", b"MZ"), ("README.txt", b""), ("a b.json", b"{}")]);
        let files = unpack(&blob).unwrap();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0], ("sketch-app.exe".to_string(), b"MZ".to_vec()));
        assert_eq!(files[1].1, b"");
        assert_eq!(files[2].0, "a b.json");
    }

    #[test]
    fn unpack_refuses_paths_and_truncated_blobs() {
        for name in ["..\\evil.exe", "sub/file.txt", "C:evil", ""] {
            assert!(unpack(&pack(&[(name, b"x")])).is_err(), "{name:?}");
        }
        let mut raw = Vec::new();
        raw.extend_from_slice(&4u16.to_le_bytes());
        raw.extend_from_slice(b"a.tx");
        raw.extend_from_slice(&99u64.to_le_bytes());
        raw.extend_from_slice(b"short");
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&raw).unwrap();
        assert!(unpack(&encoder.finish().unwrap()).is_err());
        assert!(unpack(b"not deflate at all").is_err());
    }

    #[test]
    fn an_installer_built_without_a_payload_says_so() {
        let target = Target {
            dir: std::env::temp_dir().join("sketch-setup-test-never-created"),
            shortcut_dirs: Vec::new(),
            reg_key: "SketchRust-unit-test".into(),
        };
        let error = install(&target, &[], &|_| false).unwrap_err();
        assert!(error.contains("without its program files"));
        assert!(!target.dir.exists());
    }
}
