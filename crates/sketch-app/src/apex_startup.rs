//! Native command-line options shared by SDK and file launches.
use std::path::{Path, PathBuf};

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Options {
    pub integration: Option<PathBuf>,
    pub legacy: Option<String>,
    pub code_table: Option<String>,
    pub files: Vec<PathBuf>,
}
pub(crate) fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut index = 1;
    while index < args.len() {
        let value = &args[index];
        if value.eq_ignore_ascii_case("/integration")
            || value.eq_ignore_ascii_case("/areacodetableurl")
        {
            let next = args
                .get(index + 1)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| format!("Missing value for {value}"))?;
            if value.eq_ignore_ascii_case("/integration") {
                options.integration = Some(next.into());
            } else {
                options.code_table = Some(next.clone());
            }
            index += 2;
            continue;
        }
        if ["-TOTALN", "-TOTALS", "-WINTOTALN", "-WINTOTALS"]
            .iter()
            .any(|s| value.eq_ignore_ascii_case(s))
        {
            options.legacy = Some(value.clone());
        } else if Path::new(value).extension().is_some_and(|ext| {
            ["ax2", "ax3", "ax4", "ax5", "ax6", "ax7"]
                .iter()
                .any(|s| ext.eq_ignore_ascii_case(s))
        }) {
            options.files.push(value.into());
        }
        index += 1;
    }
    Ok(options)
}

/// The sketch files (`.skx`, `.json`, ...) named on the command line, as a
/// double-click on an associated file or "Open with" passes them.
pub(crate) fn sketch_files(args: &[String]) -> Vec<PathBuf> {
    args.iter()
        .skip(1)
        .filter(|value| {
            Path::new(value).extension().is_some_and(|ext| {
                crate::SKETCH_OPEN_EXTENSIONS
                    .iter()
                    .any(|s| ext.eq_ignore_ascii_case(s))
            })
        })
        .map(PathBuf::from)
        .collect()
}

/// Each native file gets an independent document/session in the same program.
/// This prevents a save in one window from overwriting another input file.
#[cfg(windows)]
pub(crate) fn open_additional(options: &Options) -> Result<(), String> {
    launch_files(options, 1, false).map(|_| ())
}

#[cfg(windows)]
pub(crate) fn launch_files(
    options: &Options,
    start: usize,
    probe: bool,
) -> Result<Vec<std::process::Child>, String> {
    use std::os::windows::process::CommandExt;
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut children = Vec::new();
    for file in options.files.iter().skip(start) {
        let mut command = std::process::Command::new(&executable);
        if probe {
            command.arg("--check-apex-legacy");
        }
        command.arg(file).creation_flags(0x0800_0000);
        if let Some(url) = &options.code_table {
            command.arg("/areacodetableurl").arg(url);
        }
        children.push(
            command
                .spawn()
                .map_err(|e| format!("Could not open {}: {e}", file.display()))?,
        );
    }
    Ok(children)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_startup_keeps_multiple_files_separate_from_option_values() {
        let args = [
            "sketch.exe",
            "C:\\first sketch.ax7",
            "/AreaCodeTableUrl",
            "https://host/table.ax7",
            "D:\\second.ax6",
        ]
        .map(str::to_owned);
        let options = parse(&args).unwrap();
        assert_eq!(
            options.files,
            vec![
                PathBuf::from("C:\\first sketch.ax7"),
                PathBuf::from("D:\\second.ax6")
            ]
        );
        assert_eq!(
            options.code_table.as_deref(),
            Some("https://host/table.ax7")
        );
        assert!(parse(&["sketch.exe".into(), "/integration".into()]).is_err());
        // Sketch files are a separate list: they open without a host session.
        assert!(sketch_files(&args).is_empty());
        let args = ["sketch.exe", "C:\\a b\\House.SKX", "notes.txt", "plan.json"].map(str::to_owned);
        assert_eq!(
            sketch_files(&args),
            vec![
                PathBuf::from("C:\\a b\\House.SKX"),
                PathBuf::from("plan.json")
            ]
        );
    }
}
