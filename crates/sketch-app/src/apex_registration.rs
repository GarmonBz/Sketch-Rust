//! Reversible redirection of the installed Apex launch path to this executable.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Debug)]
enum SetupError {
    Io(PathBuf, std::io::Error),
    Other(String),
}
impl From<String> for SetupError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}
impl From<&str> for SetupError {
    fn from(value: &str) -> Self {
        Self::Other(value.into())
    }
}
impl std::fmt::Display for SetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(path, error) => write!(f, "{}: {error}", path.display()),
            Self::Other(error) => f.write_str(error),
        }
    }
}
#[derive(Serialize, Deserialize)]
struct Registration {
    original_xml: Option<String>,
    enabled_xml: String,
    original_executable: PathBuf,
    executable: PathBuf,
}
fn config_path() -> Result<PathBuf, String> {
    Ok(
        PathBuf::from(std::env::var_os("ProgramData").ok_or("ProgramData is unavailable")?)
            .join("Apex Software/Apex Sketch/config.xml"),
    )
}
fn registration_path() -> Result<PathBuf, String> {
    Ok(
        PathBuf::from(std::env::var_os("APPDATA").ok_or("APPDATA is unavailable")?)
            .join("SketchRust/apex-compat-registration.json"),
    )
}
pub fn launch_path() -> Result<Option<PathBuf>, String> {
    let xml = match fs::read_to_string(config_path()?) {
        Ok(xml) => xml,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let (_, value, _) = scalar(&xml)?;
    let path = decode(value.trim());
    Ok((!path.is_empty()).then(|| PathBuf::from(path)))
}
fn scalar(xml: &str) -> Result<(&str, &str, &str), String> {
    let (before, rest) = xml
        .split_once("<INSTALLPATH>")
        .ok_or("Apex launch path not found")?;
    let (value, after) = rest
        .split_once("</INSTALLPATH>")
        .ok_or("Invalid Apex launch configuration")?;
    if after.contains("<INSTALLPATH>") {
        return Err("Ambiguous Apex launch configuration".into());
    }
    Ok((before, value, after))
}
fn decode(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
fn encode(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn replace_file(path: &Path, bytes: &[u8]) -> Result<(), SetupError> {
    use windows::{
        core::PCWSTR,
        Win32::Storage::FileSystem::{
            MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        },
    };
    let staging = path.with_extension("sketch-rust-new");
    fs::write(&staging, bytes).map_err(|e| SetupError::Io(staging.clone(), e))?;
    let wide = |p: &Path| {
        p.to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>()
    };
    let source = wide(&staging);
    let target = wide(path);
    unsafe {
        MoveFileExW(
            PCWSTR(source.as_ptr()),
            PCWSTR(target.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
        .map_err(|e| {
            SetupError::Io(
                path.into(),
                std::io::Error::from_raw_os_error(e.code().0 & 0xffff),
            )
        })
    }
}
pub fn configure(enable: bool) -> Result<String, String> {
    let config = config_path()?;
    let record = registration_path()?;
    if enable {
        crate::apex_compat::probe()?;
    }
    let result = configure_at(
        &config,
        &record,
        std::env::current_exe().map_err(|e| e.to_string())?,
        enable,
    );
    match result {
        Err(SetupError::Io(_, ref error))
            if error.kind() == std::io::ErrorKind::PermissionDenied
                && !std::env::args().any(|a| a == "--apex-registration-elevated") =>
        {
            elevate_registration(enable)?;
            let current = fs::read_to_string(&config).ok();
            let data: Registration =
                serde_json::from_slice(&fs::read(&record).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let expected = if enable {
                Some(data.enabled_xml)
            } else {
                data.original_xml
            };
            if current != expected {
                return Err(
                    "Administrator setup did not produce the expected launch configuration.".into(),
                );
            }
            Ok(if enable {
                "Apex-compatible host launch enabled."
            } else {
                "Original Apex launch path restored."
            }
            .into())
        }
        other => other.map_err(|e| e.to_string()),
    }
}

// Only registration is elevated. Normal sketch editing and host transfers keep
// the caller's token, and no companion executable or shell command is used.
fn elevate_registration(enable: bool) -> Result<(), String> {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::{CloseHandle, WAIT_OBJECT_0},
            System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE},
            UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW},
        },
    };
    let wide = |s: &std::ffi::OsStr| {
        s.to_string_lossy()
            .encode_utf16()
            .chain(Some(0))
            .collect::<Vec<_>>()
    };
    let exe = wide(
        std::env::current_exe()
            .map_err(|e| e.to_string())?
            .as_os_str(),
    );
    let verb = wide(std::ffi::OsStr::new("runas"));
    let args = wide(std::ffi::OsStr::new(if enable {
        "--enable-apex-compat --apex-registration-elevated"
    } else {
        "--restore-apex-launch --apex-registration-elevated"
    }));
    let mut launch = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(exe.as_ptr()),
        lpParameters: PCWSTR(args.as_ptr()),
        nShow: 0,
        ..Default::default()
    };
    unsafe {
        ShellExecuteExW(&mut launch).map_err(|e| format!("Administrator permission is required to update Apex's shared launch configuration. Setup was not completed: {e}"))?;
        if launch.hProcess.is_invalid() {
            return Err("Windows did not return the administrator setup process.".into());
        }
        let waited = WaitForSingleObject(launch.hProcess, INFINITE);
        let mut code = 1;
        let status = GetExitCodeProcess(launch.hProcess, &mut code);
        let _ = CloseHandle(launch.hProcess);
        status.map_err(|e| e.to_string())?;
        if waited != WAIT_OBJECT_0 || code != 0 {
            return Err(format!("Administrator setup failed (exit {code}); the launch configuration has not been verified."));
        }
    }
    Ok(())
}
fn configure_at(
    config: &Path,
    record: &Path,
    executable: PathBuf,
    enable: bool,
) -> Result<String, SetupError> {
    let previous_record = match fs::read(record) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(SetupError::Io(record.into(), e)),
    };
    let current = match fs::read_to_string(config) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(SetupError::Io(config.into(), error)),
    };
    if !enable {
        let bytes = previous_record
            .as_deref()
            .ok_or("No previous provider is recorded. Enable integration before using Restore.")?;
        let data: Registration = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if current == data.original_xml {
            return Ok("Original Apex launch path is already restored.".into());
        }
        if current.as_deref() != Some(data.enabled_xml.as_str()) {
            return Err(
                "Apex configuration changed after setup; its current contents were preserved."
                    .into(),
            );
        }
        if let Some(original) = data.original_xml {
            replace_file(config, original.as_bytes())?;
        } else {
            // Only remove a configuration we created, after exact-content
            // ownership verification above. Preserve later host changes.
            fs::remove_file(config).map_err(|e| e.to_string())?;
        }
        return Ok("Original Apex launch path restored.".into());
    }
    let original = if record.exists() {
        let data: Registration =
            serde_json::from_slice(&fs::read(record).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if current.as_deref() != Some(data.enabled_xml.as_str()) && current != data.original_xml {
            return Err(
                "Apex configuration changed after setup; its current contents were preserved."
                    .into(),
            );
        }
        data.original_xml
    } else {
        current.clone()
    };
    let initial = original.as_deref().unwrap_or("<?xml version=\"1.0\" encoding=\"utf-8\"?><SETTINGS><INSTALLPATH></INSTALLPATH></SETTINGS>");
    let (before, old, after) = scalar(initial)?;
    let original_executable = PathBuf::from(decode(old.trim()));
    // The old path is recovery metadata, not a program we launch or require.
    // Hosts can retain an SDK configuration after uninstalling their provider.
    if original.is_some()
        && (original_executable.as_os_str().is_empty() || original_executable == executable)
    {
        return Err(
            "The original launch configuration does not identify a distinct provider.".into(),
        );
    }
    let enabled = format!(
        "{before}<INSTALLPATH>{}</INSTALLPATH>{after}",
        encode(&executable.to_string_lossy())
    );
    let data = Registration {
        original_xml: original.clone(),
        enabled_xml: enabled.clone(),
        original_executable,
        executable,
    };
    fs::create_dir_all(record.parent().ok_or("Invalid setup path")?).map_err(|e| e.to_string())?;
    replace_file(
        record,
        &serde_json::to_vec_pretty(&data).map_err(|e| e.to_string())?,
    )?;
    let write_config = || -> Result<(), SetupError> {
        fs::create_dir_all(config.parent().ok_or("Invalid configuration path")?)
            .map_err(|e| SetupError::Io(config.into(), e))?;
        replace_file(config, enabled.as_bytes())
    };
    if let Err(error) = write_config() {
        // An unelevated upgrade must leave the old registration coherent for
        // the elevated retry. Otherwise its new record would reject the still
        // active old EXE's configuration as an unrelated host change.
        if let Some(previous) = previous_record {
            replace_file(record, &previous).map_err(|rollback| {
                SetupError::Other(format!(
                    "{error}; could not restore the previous setup record: {rollback}"
                ))
            })?;
        }
        return Err(error);
    }
    Ok("Apex-compatible host launch requests now open Sketch Rust. Original Apex launch settings are backed up.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn paths(name: &str) -> (PathBuf, PathBuf, PathBuf) {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../target/e2e/apex-registration-{name}-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        (
            root.join("settings/config.xml"),
            root.join("backup.json"),
            root.join("Sketch & Replacement.exe"),
        )
    }
    #[test]
    fn restores_exact_original_without_requiring_an_apex_program() {
        let (config, record, executable) = paths("existing");
        let original = "\u{feff}<?xml version=\"1.0\"?><SETTINGS>\r\n<INSTALLPATH>C:\\Uninstalled Apex\\Apex.exe</INSTALLPATH><OTHER>preserve &amp; me</OTHER></SETTINGS>";
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        fs::write(&config, original).unwrap();
        configure_at(&config, &record, executable.clone(), true).unwrap();
        assert!(fs::read_to_string(&config)
            .unwrap()
            .contains("Sketch &amp; Replacement.exe"));
        configure_at(&config, &record, executable.clone(), true).unwrap();
        configure_at(&config, &record, executable.clone(), false).unwrap();
        assert_eq!(fs::read_to_string(&config).unwrap(), original);
        configure_at(&config, &record, executable, false).unwrap();
        assert_eq!(fs::read_to_string(&config).unwrap(), original);
    }
    #[test]
    fn newly_created_configuration_restores_to_absence_and_preserves_host_changes() {
        let (config, record, executable) = paths("fresh");
        configure_at(&config, &record, executable.clone(), true).unwrap();
        let enabled = fs::read_to_string(&config).unwrap();
        let backup: Registration = serde_json::from_slice(&fs::read(&record).unwrap()).unwrap();
        assert!(backup.original_xml.is_none());
        fs::write(&config, "host updated this configuration").unwrap();
        assert!(configure_at(&config, &record, executable.clone(), false).is_err());
        assert!(configure_at(&config, &record, executable.clone(), true).is_err());
        assert_eq!(
            fs::read_to_string(&config).unwrap(),
            "host updated this configuration"
        );
        fs::write(&config, enabled).unwrap();
        configure_at(&config, &record, executable.clone(), false).unwrap();
        assert!(!config.exists());
        configure_at(&config, &record, executable, false).unwrap();
        assert!(!config.exists());
    }
    #[test]
    fn malformed_existing_configuration_is_preserved() {
        let (config, record, executable) = paths("malformed");
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        let original =
            "<SETTINGS><INSTALLPATH>one</INSTALLPATH><INSTALLPATH>two</INSTALLPATH></SETTINGS>";
        fs::write(&config, original).unwrap();
        assert!(configure_at(&config, &record, executable, true).is_err());
        assert_eq!(fs::read_to_string(&config).unwrap(), original);
        assert!(!record.exists());
    }
    #[test]
    fn protected_config_failure_preserves_original_and_can_be_retried() {
        let (config, record, executable) = paths("protected");
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        let original = "<SETTINGS><INSTALLPATH>C:\\Original\\Apex.exe</INSTALLPATH></SETTINGS>";
        fs::write(&config, original).unwrap();
        let original_permissions = fs::metadata(&config).unwrap().permissions();
        let mut permissions = original_permissions.clone();
        permissions.set_readonly(true);
        fs::set_permissions(&config, permissions.clone()).unwrap();
        let failed = configure_at(&config, &record, executable.clone(), true);
        // Restore fixture access before asserting, so a failing assertion does
        // not leave a protected test artifact behind.
        fs::set_permissions(&config, original_permissions).unwrap();
        assert!(matches!(failed, Err(SetupError::Io(_, ref e))
            if e.kind() == std::io::ErrorKind::PermissionDenied));
        assert_eq!(fs::read_to_string(&config).unwrap(), original);
        let backup: Registration = serde_json::from_slice(&fs::read(&record).unwrap()).unwrap();
        assert_eq!(backup.original_xml.as_deref(), Some(original));
        configure_at(&config, &record, executable.clone(), true).unwrap();
        configure_at(&config, &record, executable, false).unwrap();
        assert_eq!(fs::read_to_string(&config).unwrap(), original);
    }

    #[test]
    fn protected_upgrade_keeps_the_previous_record_for_elevated_retry() {
        let (config, record, old_exe) = paths("protected-upgrade");
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        let original = "<SETTINGS><INSTALLPATH>C:\\Original\\Apex.exe</INSTALLPATH></SETTINGS>";
        fs::write(&config, original).unwrap();
        configure_at(&config, &record, old_exe.clone(), true).unwrap();
        let previous_xml = fs::read(&config).unwrap();
        let previous_record = fs::read(&record).unwrap();
        let new_exe = old_exe.with_file_name("new-sketch.exe");
        let writable = fs::metadata(&config).unwrap().permissions();
        let mut readonly = writable.clone();
        readonly.set_readonly(true);
        fs::set_permissions(&config, readonly).unwrap();
        let failed = configure_at(&config, &record, new_exe.clone(), true);
        fs::set_permissions(&config, writable).unwrap();
        assert!(
            matches!(failed, Err(SetupError::Io(_, ref e)) if e.kind() == std::io::ErrorKind::PermissionDenied)
        );
        assert_eq!(fs::read(&config).unwrap(), previous_xml);
        assert_eq!(fs::read(&record).unwrap(), previous_record);
        configure_at(&config, &record, new_exe.clone(), true).unwrap();
        let registered: Registration = serde_json::from_slice(&fs::read(&record).unwrap()).unwrap();
        assert_eq!(registered.executable, new_exe);
        assert_eq!(registered.original_xml.as_deref(), Some(original));
        configure_at(&config, &record, new_exe, false).unwrap();
        assert_eq!(fs::read_to_string(&config).unwrap(), original);
    }
}
