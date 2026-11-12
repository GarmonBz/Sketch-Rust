//! Persistent user overrides for the external symbol library.
use crate::{persist, symlib, App, Message};
use iced::widget::{
    button, checkbox, column, container, opaque, row, scrollable, text, text_input,
};
use iced::{Element, Length, Task};
use serde::{Deserialize, Serialize};
use sketch_core::symbols::{Symbol, SymbolCategory};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Changes {
    pub renamed: BTreeMap<String, String>,
    pub hidden: BTreeSet<String>,
    // Read old purge records as deletion tombstones; they are never restorable.
    #[serde(alias = "purged")]
    pub deleted: BTreeSet<String>,
    #[serde(alias = "purgedCategories")]
    pub deleted_categories: BTreeSet<String>,
    pub custom: BTreeMap<String, Symbol>,
}
#[derive(Debug, Clone, Default)]
pub struct Manager {
    pub open: bool,
    pub filter: String,
    pub selected: BTreeSet<String>,
    pub focus: Option<String>,
    pub name: String,
    pub category: String,
    pub confirm: u8,
    pub enlarged: bool,
    pub context: bool,
    pub pending_category: Option<String>,
    /// Category name typed for Move / Rename category.
    pub target: String,
}
#[derive(Debug, Clone)]
pub enum Msg {
    Focus(String),
    ViewerClose,
    Inspect(String),
    Context(String),
    Place,
    RestoreAll,
    Open,
    Close,
    Filter(String),
    Toggle(String),
    Name(String),
    Category(String),
    Rename,
    SelectAll,
    Favorite,
    Hide,
    Restore,
    /// Remove the selection from `symbols.json` itself (two confirmations).
    Delete,
    Confirm,
    Cancel,
    DeleteCategory,
    Target(String),
    /// Rename the one selected symbol in `symbols.json` (Rename only overlays).
    RenameInFile,
    /// File the selection under the `target` category in `symbols.json`.
    Move,
    /// Rename the filtered category to `target` in `symbols.json`.
    RenameCategoryInFile,
    /// Fold every group of identical symbols into its first member.
    MergeDuplicates,
    Import,
    View,
    Favorites,
}
impl App {
    pub(crate) fn portable_custom_symbols(&self) -> BTreeMap<String, Symbol> {
        self.symbol_changes
            .custom
            .keys()
            .filter_map(|id| {
                self.symbols_lib()
                    .symbol(id)
                    .cloned()
                    .map(|s| (id.clone(), s))
            })
            .collect()
    }
    pub(crate) fn rebuild_symbols(&mut self) {
        let base = symlib::library();
        let Ok(base) = base.as_ref() else { return };
        let mut lib = base.library.clone();
        lib.symbols.extend(self.symbol_changes.custom.clone());
        lib.symbols
            .retain(|id, _| !self.symbol_changes.deleted.contains(id));
        for (id, name) in &self.symbol_changes.renamed {
            if let Some(s) = lib.symbols.get_mut(id) {
                s.label = name.clone();
                s.name = name.clone();
            }
        }
        for s in lib.symbols.values() {
            if !lib.categories.iter().any(|c| c.key == s.category) {
                lib.categories.push(SymbolCategory {
                    key: s.category.clone(),
                    name: s.category_name.clone(),
                    symbol_ids: vec![],
                });
            }
        }
        for c in &mut lib.categories {
            c.symbol_ids = lib
                .symbols
                .values()
                .filter(|s| s.category == c.key && !self.symbol_changes.hidden.contains(&s.id))
                .map(|s| s.id.clone())
                .collect();
        }
        lib.default_favorites
            .retain(|id| !self.symbol_changes.hidden.contains(id) && lib.symbols.contains_key(id));
        lib.categories.retain(|c| {
            !self.symbol_changes.deleted_categories.contains(&c.key) || !c.symbol_ids.is_empty()
        });
        lib.symbol_count = lib.symbols.len();
        let mut index = symlib::SymIndex::build(lib);
        index.hidden = self.symbol_changes.hidden.clone();
        self.symbols = Arc::new(Ok(index));
        // Deleted categories must disappear from both pickers, not remain as a
        // selected value that no longer exists in their options.
        if !self.symbol_manager.category.is_empty()
            && !VIEWS.contains(&self.symbol_manager.category.as_str())
            && !self.category_is_real(&self.symbol_manager.category)
        {
            self.symbol_manager.category.clear();
            self.symbol_manager.filter.clear();
        }
        if self.dock.symbol_category != "favorites"
            && !self.category_is_real(&self.dock.symbol_category)
        {
            self.dock.symbol_category = "favorites".into();
        }
        self.sync_manager_focus();
        if self.placing_symbol.as_ref().is_some_and(|id| {
            self.symbol_changes.hidden.contains(id) || self.symbol_changes.deleted.contains(id)
        }) {
            self.placing_symbol = None;
            self.dock.preview = None;
        }
    }
    fn sync_manager_focus(&mut self) {
        let shown = self.manager_symbols();
        let focus_matches = shown
            .iter()
            .any(|symbol| Some(&symbol.id) == self.symbol_manager.focus.as_ref());
        let next_focus = shown.first().map(|symbol| symbol.id.clone());
        if !focus_matches {
            self.symbol_manager.focus = next_focus;
        }
    }
    fn store_symbols(&mut self) -> Task<Message> {
        self.rebuild_symbols();
        if persist::write_json("symbol-library.json", &self.symbol_changes) == Some(false) {
            return self
                .toasts
                .show("Could not save symbol library changes.".into());
        }
        Task::none()
    }
    pub(crate) fn manager_msg(&mut self, m: Msg) -> Task<Message> {
        if matches!(
            &m,
            Msg::Toggle(_)
                | Msg::Category(_)
                | Msg::Filter(_)
                | Msg::Delete
                | Msg::DeleteCategory
                | Msg::RenameInFile
                | Msg::Move
                | Msg::RenameCategoryInFile
                | Msg::MergeDuplicates
                | Msg::Hide
                | Msg::RestoreAll
        ) {
            self.symbol_manager.confirm = 0;
            self.symbol_manager.pending_category = None;
        }
        match m {
            Msg::Focus(id) => {
                self.symbol_manager.focus = Some(id);
            }
            Msg::ViewerClose => self.symbol_manager.enlarged = false,
            Msg::Inspect(id) => {
                self.symbol_manager.focus = Some(id);
                self.symbol_manager.enlarged = true;
            }
            Msg::Context(id) => {
                self.symbol_manager = Manager {
                    open: true,
                    context: true,
                    selected: BTreeSet::from([id.clone()]),
                    focus: Some(id.clone()),
                    name: self
                        .symbols_lib()
                        .symbol(&id)
                        .map(|s| s.label.clone())
                        .unwrap_or_default(),
                    ..Manager::default()
                };
            }
            Msg::Place => {
                if let Some(id) = self.symbol_manager.selected.first().cloned() {
                    self.symbol_manager.open = false;
                    self.placing_symbol = Some(id.clone());
                    self.dock.preview = Some(id);
                    return self.set_mode(crate::view_state::Mode::PlaceSymbol);
                }
            }
            Msg::RestoreAll => {
                self.symbol_manager.open = true;
                self.symbol_manager.context = false;
                self.symbol_manager.selected = self.symbol_changes.hidden.clone();
                self.symbol_manager.confirm = 4;
            }
            Msg::Open => {
                if self.symbol_manager.context {
                    self.symbol_manager.context = false;
                } else {
                    self.symbol_manager = Manager {
                        open: true,
                        category: if self.dock.active == crate::dock::DockTab::Symbols
                            && self.category_is_real(&self.dock.symbol_category)
                        {
                            self.dock.symbol_category.clone()
                        } else {
                            String::new()
                        },
                        ..Manager::default()
                    };
                }
            }
            Msg::Close => {
                self.symbol_manager.open = false;
                self.symbol_manager.enlarged = false;
            }
            Msg::Filter(v) => self.symbol_manager.filter = v,
            Msg::Name(v) => self.symbol_manager.name = v,
            Msg::Category(v) => self.symbol_manager.category = v,
            Msg::Toggle(id) => {
                self.symbol_manager.focus = Some(id.clone());
                if !self.symbol_manager.selected.remove(&id) {
                    self.symbol_manager.selected.insert(id.clone());
                }
                if let Some(s) = self.symbols_lib().symbol(&id) {
                    self.symbol_manager.name = s.label.clone();
                }
                self.symbol_manager.confirm = 0;
            }
            Msg::SelectAll => {
                self.symbol_manager.selected = self
                    .manager_symbols()
                    .into_iter()
                    .map(|s| s.id.clone())
                    .collect()
            }
            Msg::Rename => {
                let name = self.symbol_manager.name.trim().to_string();
                if self.symbol_manager.selected.len() == 1 && !name.is_empty() {
                    let id = self.symbol_manager.selected.first().unwrap().clone();
                    self.symbol_changes.renamed.insert(id, name);
                    return self.store_symbols();
                }
            }
            Msg::Favorite => {
                let mut ids = self.favorite_types();
                for id in &self.symbol_manager.selected {
                    if let Some(p) = ids.iter().position(|x| x == id) {
                        ids.remove(p);
                    } else {
                        ids.push(id.clone());
                    }
                }
                self.symbol_favorites = Some(ids);
                self.persist_library();
            }
            Msg::Hide => {
                if !self.symbol_manager.selected.is_empty() {
                    self.symbol_manager.confirm = 3;
                }
            }
            Msg::Restore => {
                for id in self.symbol_manager.selected.clone() {
                    self.symbol_changes.hidden.remove(&id);
                }
                return self.store_symbols();
            }
            Msg::Delete => {
                if !self.symbol_manager.selected.is_empty() {
                    self.symbol_manager.confirm = 5;
                }
            }
            Msg::DeleteCategory => {
                let category = self.symbol_manager.category.trim().to_string();
                if self.category_is_real(&category) {
                    self.symbol_manager.selected = self
                        .symbols_lib()
                        .library
                        .symbols
                        .values()
                        .filter(|s| s.category == category)
                        .map(|s| s.id.clone())
                        .collect();
                    self.symbol_manager.pending_category = Some(category);
                    self.symbol_manager.confirm = 5;
                }
            }
            Msg::Target(v) => self.symbol_manager.target = v,
            Msg::RenameInFile => {
                if self.symbol_manager.selected.len() == 1
                    && !self.symbol_manager.name.trim().is_empty()
                {
                    self.symbol_manager.confirm = 8;
                }
            }
            Msg::Move => {
                if !self.symbol_manager.selected.is_empty()
                    && !self.symbol_manager.target.trim().is_empty()
                {
                    self.symbol_manager.confirm = 7;
                }
            }
            Msg::RenameCategoryInFile => {
                if self.category_is_real(self.symbol_manager.category.trim())
                    && !self.symbol_manager.target.trim().is_empty()
                {
                    self.symbol_manager.confirm = 9;
                }
            }
            Msg::MergeDuplicates => {
                if !self.merge_pairs().is_empty() {
                    self.symbol_manager.confirm = 10;
                }
            }
            Msg::Cancel => {
                self.symbol_manager.confirm = 0;
                self.symbol_manager.pending_category = None;
            }
            Msg::Confirm => {
                if self.symbol_manager.confirm == 3 {
                    self.symbol_changes
                        .hidden
                        .extend(self.symbol_manager.selected.clone());
                    self.symbol_manager.confirm = 0;
                    if self.symbol_manager.context {
                        self.symbol_manager.open = false;
                    }
                    return self.store_symbols();
                } else if self.symbol_manager.confirm == 4 {
                    for id in &self.symbol_manager.selected {
                        self.symbol_changes.hidden.remove(id);
                    }
                    self.symbol_manager.confirm = 0;
                    return self.store_symbols();
                } else if (7..=10).contains(&self.symbol_manager.confirm) {
                    return self.edit_library_file();
                } else if self.symbol_manager.confirm == 5 {
                    self.symbol_manager.confirm = 6;
                } else if self.symbol_manager.confirm == 6 {
                    let ids = self.symbol_manager.selected.clone();
                    let category = self.symbol_manager.pending_category.clone();
                    let file_category =
                        symlib::library()
                            .as_ref()
                            .as_ref()
                            .ok()
                            .is_some_and(|base| {
                                category.as_ref().is_some_and(|key| {
                                    base.library.categories.iter().any(|c| &c.key == key)
                                })
                            });
                    let base: BTreeSet<String> = match symlib::library().as_ref() {
                        Ok(base) => ids
                            .iter()
                            .filter(|id| base.library.symbols.contains_key(*id))
                            .cloned()
                            .collect(),
                        Err(_) => BTreeSet::new(),
                    };
                    let mut notice = format!(
                        "Deleted {} symbol{} permanently.",
                        ids.len(),
                        plural(ids.len())
                    );
                    if !base.is_empty() || file_category {
                        let result = if file_category {
                            symlib::edit_library(|path| {
                                sketch_core::symbol_edit::delete_category(
                                    path,
                                    category.as_deref().unwrap(),
                                )
                            })
                        } else {
                            symlib::delete_from_library(&base)
                        };
                        if let Err(error) = result {
                            // A failed deletion leaves the document, history and selection intact.
                            return self.toasts.show(format!("Could not delete symbols ({error}). Nothing was deleted. Check that the library file is writable and try again."));
                        }
                        notice.push_str(" Previous library saved as symbols.json.bak.");
                    }
                    self.forget_symbols(&ids);
                    self.symbol_changes.deleted.extend(ids);
                    if let Some(category) = category {
                        self.symbol_changes.deleted_categories.insert(category);
                    }
                    self.symbol_manager.pending_category = None;
                    self.symbol_manager.confirm = 0;
                    self.symbol_manager.selected.clear();

                    let task = self.store_symbols();
                    return task.chain(self.toasts.show(notice));
                }
            }
            Msg::Import => {
                #[cfg(not(test))]
                if let Some(paths) = rfd::FileDialog::new()
                    .add_filter("Symbol or DXF", &["json", "dxf", "dwg"])
                    .pick_files()
                {
                    let mut added = 0;
                    let mut errors = vec![];
                    for path in paths {
                        let result = if path
                            .extension()
                            .is_some_and(|s| s.eq_ignore_ascii_case("dwg"))
                        {
                            Err("DWG requires conversion to DXF.".to_string())
                        } else {
                            std::fs::read_to_string(&path)
                                .map_err(|e| e.to_string())
                                .and_then(|body| {
                                    self.import_symbol(
                                        &body,
                                        path.file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("Imported"),
                                        path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                                    )
                                })
                        };
                        match result {
                            Ok(()) => added += 1,
                            Err(e) => errors.push(format!(
                                "{}: {e}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            )),
                        }
                    }
                    let task = self.store_symbols();
                    if errors.is_empty() {
                        return task.chain(self.toasts.show(format!("Added {added} symbols.")));
                    }
                    return task.chain(
                        self.toasts
                            .show(format!("Added {added} symbols. {}", errors.join("; "))),
                    );
                }
            }

            Msg::View => self.symbol_manager.enlarged = self.symbol_manager.focus.is_some(),
            Msg::Favorites => {
                self.symbol_manager.open = false;
                return self
                    .symbol_organizer_msg(crate::symbol_organizer::SymbolOrganizerMsg::Open);
            }
        }
        if self.symbol_manager.open && !self.symbol_manager.context {
            self.sync_manager_focus();
        }
        Task::none()
    }
    /// What the manager list shows, narrowed by the category and search: the
    /// live library, or one of the views over it (groups of
    /// Duplicates keeper first, the Largest first).
    pub(crate) fn manager_symbols(&self) -> Vec<&Symbol> {
        let lib = self.symbols_lib();
        let mut list: Vec<&Symbol> = match self.symbol_manager.category.as_str() {
            "Duplicates" => self
                .duplicate_groups()
                .iter()
                .flatten()
                .filter_map(|id| lib.symbol(id))
                .collect(),
            _ => lib.library.symbols.values().collect(),
        };
        list.retain(|s| self.symbol_matches(s));
        if self.symbol_manager.category == "Largest" {
            list.sort_by_key(|s| std::cmp::Reverse(s.geometry.len()));
        }
        list
    }
    /// A category the manager can delete or rename: not one of the views.
    fn category_is_real(&self, category: &str) -> bool {
        !category.is_empty()
            && !VIEWS.contains(&category)
            && self
                .symbols_lib()
                .categories
                .iter()
                .any(|c| c.key == category)
    }
    /// Groups of identical symbols that live in `symbols.json` and are still in
    /// the library, keeper first. Imported symbols are not in the
    /// file, so they never take part.
    pub(crate) fn duplicate_groups(&self) -> Vec<Vec<String>> {
        let base = symlib::library();
        let Ok(base) = base.as_ref() else {
            return vec![];
        };
        let lib = self.symbols_lib();
        base.duplicates()
            .iter()
            .map(|group| {
                group
                    .iter()
                    .filter(|id| lib.library.symbols.contains_key(*id))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .filter(|group| group.len() > 1)
            .collect()
    }
    /// `(retired, kept)` for every duplicate that Merge duplicates would fold.
    pub(crate) fn merge_pairs(&self) -> Vec<(String, String)> {
        self.duplicate_groups()
            .into_iter()
            .flat_map(|group| {
                let kept = group[0].clone();
                group.into_iter().skip(1).map(move |id| (id, kept.clone()))
            })
            .collect()
    }
    /// The ids of `ids` that `symbols.json` holds (the rest are imported).
    fn file_ids(&self, ids: &BTreeSet<String>) -> BTreeSet<String> {
        match symlib::library().as_ref() {
            Ok(base) => ids
                .iter()
                .filter(|id| base.library.symbols.contains_key(*id))
                .cloned()
                .collect(),
            Err(_) => BTreeSet::new(),
        }
    }
    /// Runs the file edit the manager just confirmed (stage 7 Move, 8 Rename,
    /// 9 Rename category, 10 Merge duplicates) and reports it.
    fn edit_library_file(&mut self) -> Task<Message> {
        let stage = self.symbol_manager.confirm;
        let ids = self.symbol_manager.selected.clone();
        let target = self.symbol_manager.target.trim().to_string();
        let label = self.symbol_manager.name.trim().to_string();
        let category = self.symbol_manager.category.trim().to_string();
        self.symbol_manager.confirm = 0;
        let in_file = self.file_ids(&ids);
        let result: Result<String, String> = match stage {
            7 => {
                let (key, name) = match self.symbols_lib().library.categories.iter().find(|c| {
                    c.key.eq_ignore_ascii_case(&target) || c.name.eq_ignore_ascii_case(&target)
                }) {
                    Some(c) => (c.key.clone(), c.name.clone()),
                    None => (category_key(&target), target.clone()),
                };
                symlib::edit_library(|path| {
                    sketch_core::symbol_edit::move_to_category(path, &in_file, &key, &name)
                })
                .map(|_| {
                    for id in &ids {
                        if let Some(s) = self.symbol_changes.custom.get_mut(id) {
                            s.category = key.clone();
                            s.category_name = name.clone();
                        }
                    }
                    format!("Moved {} symbol{} to {name}.", ids.len(), plural(ids.len()))
                })
            }
            8 => {
                let id = ids.first().cloned().unwrap_or_default();
                let labels = BTreeMap::from([(id.clone(), label.clone())]);
                symlib::edit_library(|path| sketch_core::symbol_edit::rename(path, &labels)).map(
                    |_| {
                        self.symbol_changes.renamed.remove(&id);
                        if let Some(s) = self.symbol_changes.custom.get_mut(&id) {
                            s.label = label.clone();
                            s.name = label.clone();
                        }
                        format!("Renamed to {label}.")
                    },
                )
            }
            9 => symlib::edit_library(|path| {
                sketch_core::symbol_edit::rename_category(path, &category, &target)
            })
            .map(|_| {
                for s in self.symbol_changes.custom.values_mut() {
                    if s.category == category {
                        s.category_name = target.clone();
                    }
                }
                format!("Renamed category {category} to {target}.")
            }),
            _ => {
                let pairs = self.merge_pairs();
                symlib::edit_library(|path| sketch_core::symbol_edit::merge(path, &pairs)).map(
                    |_| {
                        let map: BTreeMap<String, String> = pairs.iter().cloned().collect();
                        self.remap_symbols(&map);
                        let kept: BTreeSet<&String> = map.values().collect();
                        format!(
                            "Merged {} duplicates into {} symbols. Placed instances now use the kept symbol.",
                            map.len(),
                            kept.len()
                        )
                    },
                )
            }
        };
        match result {
            Ok(notice) => {
                let task = self.store_symbols();
                let mut selected = std::mem::take(&mut self.symbol_manager.selected);
                selected.retain(|id| self.symbols_lib().library.symbols.contains_key(id));
                self.symbol_manager.selected = selected;
                task.chain(self.toasts.show(notice))
            }
            Err(e) => self.toasts.show(format!(
                "Could not edit the symbol library ({e}). Nothing changed."
            )),
        }
    }
    /// Points every placed instance of a retired symbol at the symbol that
    /// replaced it: the document, every page, undo/redo, the clipboard and the
    /// user's favorites.
    fn remap_symbols(&mut self, map: &BTreeMap<String, String>) {
        self.save_current_page();
        let fix = |s: &mut crate::placed_symbol::PlacedSymbol| {
            if let Some(to) = map.get(&s.symbol_type) {
                s.symbol_type = to.clone();
            }
        };
        self.pages
            .pages
            .iter_mut()
            .flat_map(|p| p.symbols.iter_mut())
            .for_each(fix);
        self.drawing.symbols.iter_mut().for_each(fix);
        self.undo.remap_symbols(map);
        if let Some(clipboard) = &mut self.clipboard {
            clipboard.symbols.iter_mut().for_each(fix);
        }
        if let Some(favorites) = &mut self.symbol_favorites {
            let mut seen = BTreeSet::new();
            *favorites = favorites
                .iter()
                .map(|id| map.get(id).unwrap_or(id).clone())
                .filter(|id| seen.insert(id.clone()))
                .collect();
        }
        for id in map.keys() {
            self.symbol_changes.hidden.remove(id);
            self.symbol_changes.renamed.remove(id);
        }
        self.persist_library();
    }
    /// A placed symbol whose id was retired by a merge takes the id that
    /// replaced it (`aliases` in `symbols.json`), for sketches saved before.
    pub(crate) fn resolve_symbol_aliases(
        &self,
        symbols: &mut [crate::placed_symbol::PlacedSymbol],
    ) {
        let lib = self.symbols_lib();
        for s in symbols {
            if !lib.library.symbols.contains_key(&s.symbol_type) {
                if let Some(to) = lib.library.aliases.get(&s.symbol_type) {
                    s.symbol_type = to.clone();
                }
            }
        }
    }
    /// Takes `ids` out of the document and everything that could bring them
    /// back: placed instances on every page, undo/redo history, the clipboard,
    /// the selection, and the library overrides and favorites.
    fn forget_symbols(&mut self, ids: &BTreeSet<String>) {
        self.save_current_page();
        for page in &mut self.pages.pages {
            page.symbols.retain(|s| !ids.contains(&s.symbol_type));
        }
        self.drawing
            .symbols
            .retain(|s| !ids.contains(&s.symbol_type));
        self.undo.purge_symbols(ids);
        if let Some(clipboard) = &mut self.clipboard {
            clipboard.symbols.retain(|s| !ids.contains(&s.symbol_type));
        }
        self.selection.clear();
        for id in ids {
            self.symbol_changes.hidden.remove(id);
            self.symbol_changes.custom.remove(id);
            self.symbol_changes.renamed.remove(id);
            self.symbol_changes.deleted.remove(id);
        }
        if let Some(favorites) = &mut self.symbol_favorites {
            favorites.retain(|id| !ids.contains(id));
        }
        self.persist_library();
    }
    fn symbol_matches(&self, s: &Symbol) -> bool {
        let m = &self.symbol_manager;
        (m.category.trim().is_empty()
            || m.category == "All"
            || (VIEWS.contains(&m.category.as_str()) && m.category != "Hidden")
            || (m.category == "Hidden" && self.symbol_changes.hidden.contains(&s.id))
            || s.category == m.category.trim())
            && (s
                .label
                .to_lowercase()
                .contains(&m.filter.trim().to_lowercase())
                || s.id
                    .to_lowercase()
                    .contains(&m.filter.trim().to_lowercase()))
    }
    pub(crate) fn import_symbol(
        &mut self,
        body: &str,
        name: &str,
        extension: &str,
    ) -> Result<(), String> {
        if extension.eq_ignore_ascii_case("dwg") {
            return Err("DWG is binary. Export or convert it to DXF before importing.".into());
        }
        let mut s: Symbol = if extension.eq_ignore_ascii_case("dxf") {
            sketch_core::symbol_import::from_dxf(body, name)?
        } else {
            serde_json::from_str(body).map_err(|e| format!("Invalid symbol JSON: {e}"))?
        };
        if s.geometry.is_empty()
            || [
                s.bounds.min_x,
                s.bounds.min_y,
                s.bounds.max_x,
                s.bounds.max_y,
            ]
            .iter()
            .any(|n| !n.is_finite())
            || s.bounds.max_x < s.bounds.min_x
            || s.bounds.max_y < s.bounds.min_y
        {
            return Err("Symbol requires valid geometry and bounds.".into());
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        s.id = format!("custom/{stamp}");
        if s.label.trim().is_empty() {
            s.label = name.into();
        }
        s.name = s.label.clone();
        if s.category.trim().is_empty() {
            s.category = "custom".into();
            s.category_name = "Custom".into();
        }
        if s.units.is_empty() {
            s.units = "inches".into();
        }
        self.symbol_manager.focus = Some(s.id.clone());
        self.symbol_manager.selected = BTreeSet::from([s.id.clone()]);
        self.symbol_manager.name = s.label.clone();
        self.symbol_changes.custom.insert(s.id.clone(), s);
        Ok(())
    }
    pub(crate) fn manager_layer(&self) -> Element<'_, Message> {
        if !self.symbol_manager.open {
            return iced::widget::Space::new().into();
        }
        let m = &self.symbol_manager;
        let action = |label: &'static str, msg| {
            button(text(label).size(12)).on_press(Message::SymbolManager(msg))
        };
        let mut rows = column![].spacing(3);
        if m.context {
            let confirmation = if m.confirm > 0 {
                column![
                    text(confirm_text(m.confirm)),
                    row![
                        action("Confirm", Msg::Confirm),
                        action("Cancel", Msg::Cancel)
                    ]
                    .spacing(8)
                ]
            } else {
                column![]
            };
            let card = container(
                column![
                    text("Symbol actions").size(18),
                    text(&m.name),
                    row![
                        action("Place", Msg::Place),
                        action("Favorite", Msg::Favorite),
                        action("Hide", Msg::Hide),
                        action("Delete", Msg::Delete)
                    ]
                    .spacing(8),
                    row![
                        action("Manage / Rename…", Msg::Open),
                        action("Close", Msg::Close)
                    ]
                    .spacing(8),
                    confirmation
                ]
                .spacing(12)
                .padding(20),
            )
            .style(|_| container::Style {
                background: Some(crate::theme::bg_panel().into()),
                text_color: Some(crate::theme::text_main()),
                ..Default::default()
            });
            return opaque(
                container(opaque(card))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            );
        }
        let width = (self.window_size.width - 40.).clamp(300., 1280.);
        let height = (self.window_size.height - 40.).clamp(280., 940.);
        let preview_width = if width < 850. { 200. } else { 300. };
        let focused = m
            .focus
            .as_deref()
            .and_then(|id| self.symbols_lib().symbol(id));
        let merge_pairs = self.merge_pairs();
        let keepers: BTreeSet<String> = if m.category == "Duplicates" {
            self.duplicate_groups()
                .into_iter()
                .map(|group| group[0].clone())
                .collect()
        } else {
            BTreeSet::new()
        };
        let action = |label: &'static str, msg: Msg| {
            let enabled = match &msg {
                Msg::Rename => m.selected.len() == 1 && !m.name.trim().is_empty(),
                Msg::Favorite | Msg::Hide => !m.selected.is_empty(),
                Msg::Delete => !m.selected.is_empty(),
                Msg::Move => !m.selected.is_empty() && !m.target.trim().is_empty(),
                Msg::RenameInFile => m.selected.len() == 1 && !m.name.trim().is_empty(),
                Msg::RenameCategoryInFile => {
                    self.category_is_real(m.category.trim()) && !m.target.trim().is_empty()
                }
                Msg::MergeDuplicates => !merge_pairs.is_empty(),
                Msg::Restore => m
                    .selected
                    .iter()
                    .any(|id| self.symbol_changes.hidden.contains(id)),
                Msg::DeleteCategory => self.category_is_real(m.category.trim()),
                Msg::RestoreAll => !self.symbol_changes.hidden.is_empty(),
                Msg::View => focused.is_some(),
                _ => true,
            };
            manager_button(label, enabled.then_some(msg))
        };
        let mut categories: Vec<String> = VIEWS.iter().map(|v| v.to_string()).collect();
        categories.extend(self.symbols_lib().categories.iter().map(|c| c.key.clone()));
        let mut shown = 0;
        for symbol in self.manager_symbols() {
            shown += 1;
            let id = symbol.id.clone();
            let select_id = id.clone();
            let view_id = id.clone();
            let is_focused = m.focus.as_ref() == Some(&id);
            let hidden = if m.category == "Duplicates" {
                if keepers.contains(&id) {
                    " · keep"
                } else {
                    " · duplicate"
                }
            } else if self.symbol_changes.hidden.contains(&id) {
                " · hidden"
            } else {
                ""
            };
            let details = if width >= 1000. {
                format!(
                    "{} · {:.0} in · {} parts{}",
                    symbol.category,
                    (symbol.bounds.max_x - symbol.bounds.min_x)
                        .max(symbol.bounds.max_y - symbol.bounds.min_y),
                    symbol.geometry.len(),
                    hidden
                )
            } else {
                hidden.trim_start_matches(" · ").to_string()
            };
            let focus_button = button(
                container(
                    text(symbol.label.clone())
                        .size(13)
                        .wrapping(iced::widget::text::Wrapping::None),
                )
                .clip(true)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .padding([5, 4])
            .on_press(Message::SymbolManager(Msg::Focus(id)))
            .style(|_, _| button::Style {
                background: None,
                text_color: crate::theme::text_main(),
                ..Default::default()
            });
            rows = rows.push(
                container(
                    row![
                        checkbox(m.selected.contains(&select_id))
                            .size(16)
                            .on_toggle(move |_| Message::SymbolManager(Msg::Toggle(
                                select_id.clone()
                            ))),
                        crate::icons::ph(
                            symlib::category_icon(&symbol.category),
                            16.,
                            crate::theme::ICON_BLUE
                        ),
                        focus_button,
                        container(
                            text(details)
                                .size(11)
                                .color(crate::theme::muted())
                                .wrapping(iced::widget::text::Wrapping::None)
                        )
                        .width(if width >= 1000. { 220. } else { 0. })
                        .clip(true),
                        action("View", Msg::Inspect(view_id))
                    ]
                    .spacing(7)
                    .align_y(iced::Alignment::Center),
                )
                .padding([2, 5])
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(
                        if is_focused {
                            crate::theme::bg_hover()
                        } else {
                            crate::theme::bg_app()
                        }
                        .into(),
                    ),
                    border: iced::Border {
                        color: if is_focused {
                            crate::theme::accent()
                        } else {
                            crate::theme::border()
                        },
                        width: 1.,
                        radius: 2.into(),
                    },
                    ..Default::default()
                }),
            );
        }
        let preview: Element<'_, Message> = if let Some(symbol) = focused {
            let metadata = self.symbol_metadata(symbol);
            column![
                container(
                    iced::widget::canvas(symlib::Preview { symbol })
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
                .height(Length::FillPortion(3))
                .width(Length::Fill)
                .style(panel_style),
                scrollable(metadata)
                    .width(Length::Fill)
                    .height(Length::FillPortion(2)),
                action("Enlarge", Msg::View).width(Length::Fill)
            ]
            .spacing(10)
            .into()
        } else {
            container(text("Select a symbol to preview").size(13))
                .center(Length::Fill)
                .into()
        };
        let content = row![
            container(
                scrollable(rows.padding(iced::Padding {
                    right: 14.,
                    ..Default::default()
                }))
                .height(Length::Fill)
            )
            .padding(8)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(panel_style),
            container(preview)
                .padding(12)
                .width(preview_width)
                .height(Length::Fill)
                .style(panel_style)
        ]
        .spacing(12)
        .height(Length::Fill);
        let confirmation: Element<'_, Message> = if m.confirm > 0 && m.pending_category.is_none() {
            column![
                text(confirm_text(m.confirm)).size(12),
                row![
                    action("Confirm", Msg::Confirm),
                    action("Cancel", Msg::Cancel)
                ]
                .spacing(8)
            ]
            .spacing(6)
            .into()
        } else {
            iced::widget::Space::new().into()
        };
        let footer = column![
            row![
                text_input("New label (one selected)", &m.name)
                    .size(13)
                    .on_input(|v| Message::SymbolManager(Msg::Name(v))),
                action("Rename", Msg::Rename),
                action("Rename in file", Msg::RenameInFile)
            ]
            .spacing(8),
            row![
                text_input("Category name (for Move or Rename category)", &m.target)
                    .size(13)
                    .on_input(|v| Message::SymbolManager(Msg::Target(v))),
                action("Move to category", Msg::Move),
                action("Rename category", Msg::RenameCategoryInFile)
            ]
            .spacing(8),
            row![
                action("Add DXF / JSON", Msg::Import),
                action("Favorite", Msg::Favorite),
                action("Hide", Msg::Hide),
                action("Restore", Msg::Restore),
                action("Delete", Msg::Delete),
                action("Close", Msg::Close)
            ]
            .spacing(8)
            .wrap(),
            row![
                action("Edit favorites…", Msg::Favorites),
                action("Merge duplicates…", Msg::MergeDuplicates),
                action("Restore hidden…", Msg::RestoreAll)
            ]
            .spacing(8)
            .wrap(),
            confirmation
        ]
        .spacing(8);
        let card = container(
            column![
                row![
                    text("Manage Symbols").size(21),
                    iced::widget::Space::new().width(Length::Fill),
                    action("×", Msg::Close)
                ]
                .align_y(iced::Alignment::Center),
                row![
                    text_input("Search labels or IDs", &m.filter)
                        .size(13)
                        .on_input(|v| Message::SymbolManager(Msg::Filter(v))),
                    iced::widget::pick_list(
                        categories,
                        Some(if m.category.is_empty() {
                            "All".into()
                        } else {
                            m.category.clone()
                        }),
                        |v| Message::SymbolManager(Msg::Category(v))
                    )
                    .width(Length::Fill)
                ]
                .spacing(10),
                row![
                    action("Select shown", Msg::SelectAll),
                    text(format!("{shown} shown · {} selected", m.selected.len())).size(12),
                    iced::widget::Space::new().width(Length::Fill),
                    action("Delete category", Msg::DeleteCategory)
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
                content,
                footer
            ]
            .spacing(12)
            .padding(16),
        )
        .width(width)
        .height(height)
        .style(|_| container::Style {
            background: Some(crate::theme::bg_panel().into()),
            text_color: Some(crate::theme::text_main()),
            border: iced::Border {
                color: crate::theme::border(),
                width: 1.,
                radius: 5.into(),
            },
            ..Default::default()
        });
        let base = opaque(
            iced::widget::mouse_area(
                container(opaque(card))
                    .center(Length::Fill)
                    .style(scrim_style),
            )
            .on_press(Message::SymbolManager(Msg::Close)),
        );
        if let Some(category) = m
            .pending_category
            .as_ref()
            .filter(|_| [5, 6].contains(&m.confirm))
        {
            let category_name = self
                .symbols_lib()
                .categories
                .iter()
                .find(|c| &c.key == category)
                .map(|c| c.name.as_str())
                .unwrap_or(category);
            let final_step = m.confirm == 6;
            let detail = format!(
                "{}{} category {} and all {} symbol{}? Placed instances on every page and their undo history will also be removed.{}",
                if final_step { "Final confirmation: " } else { "" },
                "Delete",
                category_name, m.selected.len(), plural(m.selected.len()),
                " This permanently updates the library file; the previous file is saved as symbols.json.bak.",
            );
            let card = container(
                column![
                    text(format!("Delete category: {category_name}")).size(20),
                    text(detail).size(14),
                    row![
                        iced::widget::Space::new().width(Length::Fill),
                        manager_button("Cancel", Some(Msg::Cancel)),
                        manager_button(
                            if final_step {
                                "Confirm deletion"
                            } else {
                                "Continue"
                            },
                            Some(Msg::Confirm)
                        ),
                    ]
                    .spacing(8),
                ]
                .spacing(16)
                .padding(20),
            )
            .width(width.min(540.))
            .style(panel_style);
            return iced::widget::stack![
                base,
                opaque(
                    iced::widget::mouse_area(
                        container(opaque(card))
                            .center(Length::Fill)
                            .style(scrim_style)
                    )
                    .on_press(Message::SymbolManager(Msg::Cancel))
                )
            ]
            .into();
        }
        if m.enlarged {
            if let Some(symbol) = focused {
                let viewer = container(
                    column![
                        row![
                            text("Symbol Viewer").size(20),
                            iced::widget::Space::new().width(Length::Fill),
                            action("Close viewer", Msg::ViewerClose)
                        ]
                        .align_y(iced::Alignment::Center),
                        iced::widget::canvas(symlib::Preview { symbol })
                            .width(Length::Fill)
                            .height(Length::Fill),
                        scrollable(self.symbol_metadata(symbol))
                            .width(Length::Fill)
                            .height(160)
                    ]
                    .spacing(12)
                    .padding(18),
                )
                .width(width.min(900.))
                .height(height)
                .style(panel_style);
                return iced::widget::stack![
                    base,
                    opaque(
                        iced::widget::mouse_area(
                            container(opaque(viewer))
                                .center(Length::Fill)
                                .style(scrim_style)
                        )
                        .on_press(Message::SymbolManager(Msg::ViewerClose))
                    )
                ]
                .into();
            }
        }
        base
    }
    fn symbol_metadata<'a>(&self, symbol: &'a Symbol) -> iced::widget::Column<'a, Message> {
        column![
            text(symbol.label.clone()).size(15),
            text(format!("Type: {}", symbol.id)).size(11),
            text(format!("Category: {}", symbol.category_name)).size(12),
            text(format!(
                "Size: {:.1} × {:.1} in",
                symbol.bounds.max_x - symbol.bounds.min_x,
                symbol.bounds.max_y - symbol.bounds.min_y
            ))
            .size(12),
            text(format!("Wall snap: {:?}", symbol.wall_snap)).size(12),
            text(format!("Placed: {}", self.symbol_placed_count(&symbol.id))).size(12)
        ]
        .spacing(5)
        .width(Length::Fill)
    }
    fn symbol_placed_count(&self, id: &str) -> usize {
        self.drawing
            .symbols
            .iter()
            .filter(|s| s.symbol_type == id)
            .count()
            + self
                .pages
                .pages
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != self.pages.current)
                .map(|(_, p)| p.symbols.iter().filter(|s| s.symbol_type == id).count())
                .sum::<usize>()
    }
}
/// What the confirmation line asks for each `Manager::confirm` stage.
/// The category-picker entries that are views over the library, not categories.
pub(crate) const VIEWS: [&str; 4] = ["All", "Hidden", "Duplicates", "Largest"];
fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}
/// The category key a new category name files under: `Window Treatments` ->
/// `window-treatments`.
fn category_key(name: &str) -> String {
    let key: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    key.split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
fn confirm_text(confirm: u8) -> &'static str {
    match confirm {
        3 => "Hide selected definitions? Placed instances keep rendering.",
        4 => "Restore all hidden symbols?",
        7 => "Move the selected symbols to this category in symbols.json? Their ids stay, so placed instances keep working. The previous file is kept as symbols.json.bak.",
        8 => "Rename this symbol in symbols.json? The previous file is kept as symbols.json.bak.",
        9 => "Rename this category in symbols.json? The previous file is kept as symbols.json.bak.",
        10 => "Merge every group of identical symbols into its first member? The others are removed from symbols.json, placed instances switch to the kept symbol, and old sketches still find it. The previous file is kept as symbols.json.bak.",
        5 => "Delete selected symbols? The symbols are removed from symbols.json along with every placed instance. This cannot be undone.",
        _ => "Final confirmation: permanently delete these symbols from the library file?",
    }
}
fn manager_button(label: &'static str, msg: Option<Msg>) -> iced::widget::Button<'static, Message> {
    button(text(label).size(12))
        .padding([5, 9])
        .on_press_maybe(msg.map(Message::SymbolManager))
        .style(|_, status| {
            let disabled = matches!(status, button::Status::Disabled);
            button::Style {
                background: Some(
                    if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                        crate::theme::bg_hover()
                    } else {
                        crate::theme::bg_app()
                    }
                    .into(),
                ),
                text_color: if disabled {
                    crate::theme::muted()
                } else {
                    crate::theme::text_main()
                },
                border: iced::Border {
                    color: crate::theme::border(),
                    width: 1.,
                    radius: 3.into(),
                },
                ..Default::default()
            }
        })
}
fn panel_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(crate::theme::bg_app().into()),
        text_color: Some(crate::theme::text_main()),
        border: iced::Border {
            color: crate::theme::border(),
            width: 1.,
            radius: 2.into(),
        },
        ..Default::default()
    }
}
fn scrim_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Color::from_rgba(0., 0., 0., 0.5).into()),
        ..Default::default()
    }
}
