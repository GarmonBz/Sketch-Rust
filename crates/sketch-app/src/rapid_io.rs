//! RapidSketch (`.rs`) and ACI Sketch (`.acisketch`) in the app. One format
//! under two names (`sketch_core::rapidsketch`): a file is converted when it
//! is opened and never saved back in place, and Export writes the current
//! page as a new one.

use std::path::Path;

use iced::Task;
use sketch_core::rapidsketch::{self, RapidImport};

use crate::{App, DialogRequest, Message};

/// The name the user picked in a menu. It decides the extension and the
/// wording only; the bytes are the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    AciSketch,
    RapidSketch,
}

impl Flavor {
    pub fn label(self) -> &'static str {
        match self {
            Flavor::AciSketch => "ACI Sketch",
            Flavor::RapidSketch => "RapidSketch",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Flavor::AciSketch => "acisketch",
            Flavor::RapidSketch => "rs",
        }
    }

    /// The flavor a file name calls itself, if either.
    pub fn of(path: &Path) -> Option<Flavor> {
        let extension = path.extension()?;
        [Flavor::AciSketch, Flavor::RapidSketch]
            .into_iter()
            .find(|f| extension.eq_ignore_ascii_case(f.extension()))
    }
}

/// Convert a file's bytes into a save model.
pub fn decode(bytes: &[u8], stem: &str) -> Result<RapidImport, String> {
    rapidsketch::read(bytes)
        .map(|doc| doc.to_model(stem))
        .map_err(|e| e.to_string())
}

pub fn read_path(path: &Path) -> Result<RapidImport, String> {
    if std::fs::metadata(path).is_ok_and(|m| m.len() > rapidsketch::MAX_BYTES) {
        return Err("This sketch file is too large to open.".into());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    decode(&bytes, stem)
}

impl App {
    /// Import: the picker, then the converted open File > Open also uses.
    pub(crate) fn pick_rapid_import(&self, flavor: Flavor) -> Task<Message> {
        Task::perform(
            crate::platform::pick_file_async(
                format!("{} sketch", flavor.label()),
                vec![flavor.extension().to_owned()],
            ),
            move |path| Message::DialogPicked(DialogRequest::ImportRapid, path),
        )
    }

    pub(crate) fn pick_rapid_export(&self, flavor: Flavor) -> Task<Message> {
        Task::perform(
            crate::platform::save_file_async(
                format!("{} sketch", flavor.label()),
                vec![flavor.extension().to_owned()],
                format!("{}.{}", self.file_name, flavor.extension()),
            ),
            move |path| Message::DialogPicked(DialogRequest::ExportRapid(flavor), path),
        )
    }

    pub(crate) fn open_rapid_path(&mut self, path: &Path) -> Task<Message> {
        match read_path(path) {
            Ok(import) => {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("sketch");
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                self.open_converted(name, stem, &import.model, import.area_count, &import.notes)
            }
            Err(e) => self.toasts.show(e),
        }
    }

    /// The current page as a RapidSketch file, with anything left out.
    pub(crate) fn rapid_export_bytes(&mut self) -> Result<(Vec<u8>, Vec<String>), String> {
        let model = self.build_save_model_without_backing();
        let page = model
            .get("pages")
            .and_then(|pages| pages.get(self.pages.current))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let library = self.symbols_lib();
        let doc = rapidsketch::from_page(&page, &self.area_codes, &model["subjectInfo"], &|id| {
            library.library.symbol(id).cloned()
        })?;
        let mut notes = doc.warnings.clone();
        if self.pages.len() > 1 {
            notes.push(
                "The file holds one sketch, so only the current page was exported.".to_owned(),
            );
        }
        Ok((rapidsketch::write(&doc), notes))
    }

    pub(crate) fn export_rapid(&mut self, flavor: Flavor, path: &Path) -> Task<Message> {
        match self.rapid_export_bytes() {
            Ok((bytes, notes)) => match std::fs::write(path, bytes) {
                Ok(()) => {
                    let mut message = format!("Exported as {}.", flavor.label());
                    for note in notes {
                        message.push(' ');
                        message.push_str(&note);
                    }
                    self.toasts.show(message)
                }
                Err(e) => self
                    .toasts
                    .show(format!("Could not write {}: {e}", path.display())),
            },
            Err(e) => self
                .toasts
                .show(format!("Export as {}: {e}", flavor.label())),
        }
    }
}
