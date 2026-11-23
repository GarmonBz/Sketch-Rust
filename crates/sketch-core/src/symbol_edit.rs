//! In-place edits of the external library file (`symbols.json`).
//!
//! Every edit works on the raw JSON, so fields this port does not model
//! survive, and goes through [`rewrite`]: the file as it was is kept beside it
//! as `<name>.bak`, the new one is written whole and swapped in, so a failed
//! write leaves the library as it was. A read-only target is refused before
//! any swap (Unix can otherwise rename over mode `0444`). An edit that
//! changes nothing writes nothing.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::symbols::{
    Bounds, Entity, Symbol, SymbolLibraryError, WallSnap, SYMBOL_LIBRARY_FILENAME,
};

type Edit = Result<usize, SymbolLibraryError>;

/// Refuses a read-only library file. On Unix, `rename` can replace a mode
/// `0444` file in a writable directory; Windows often cannot. Checking here
/// keeps "library not writable" behaviour the same on every host.
fn ensure_library_writable(path: &Path) -> std::io::Result<()> {
    let meta = std::fs::metadata(path)?;
    if meta.permissions().readonly() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("{} is not writable", path.display()),
        ));
    }
    Ok(())
}

/// Runs `edit` on the library document at `path`; when it reports a non-zero
/// count the document is saved (see the module docs) and the count returned.
fn rewrite(path: &Path, edit: impl FnOnce(&mut Value) -> usize) -> Edit {
    let original = std::fs::read_to_string(path)?;
    let mut doc: Value = serde_json::from_str(&original)?;
    let changed = edit(&mut doc);
    if changed == 0 {
        return Ok(0);
    }
    ensure_library_writable(path)?;
    if let Some(count) = doc
        .get("symbols")
        .and_then(Value::as_object)
        .map(|s| s.len())
    {
        doc["symbolCount"] = Value::from(count);
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| SYMBOL_LIBRARY_FILENAME.into());
    std::fs::write(path.with_file_name(format!("{name}.bak")), &original)?;
    let tmp = path.with_file_name(format!("{name}.tmp"));
    std::fs::write(&tmp, serde_json::to_vec(&doc)?)?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.into());
    }
    Ok(changed)
}

fn categories(doc: &mut Value) -> Option<&mut Vec<Value>> {
    doc.get_mut("categories").and_then(Value::as_array_mut)
}

/// Takes `ids` out of every category list, dropping a category that this
/// leaves empty.
fn unlist(doc: &mut Value, ids: &BTreeSet<String>) {
    let keep = |v: &Value| v.as_str().is_none_or(|id| !ids.contains(id));
    if let Some(categories) = categories(doc) {
        categories.retain_mut(|category| {
            let Some(list) = category.get_mut("symbolIds").and_then(Value::as_array_mut) else {
                return true;
            };
            let before = list.len();
            list.retain(keep);
            before == list.len() || !list.is_empty()
        });
    }
}

/// Removes `ids` from the symbols, the category lists and the default
/// favorites, and any alias that led to them. Returns how many symbols went.
fn remove(doc: &mut Value, ids: &BTreeSet<String>) -> usize {
    let Some(symbols) = doc.get_mut("symbols").and_then(Value::as_object_mut) else {
        return 0;
    };
    let removed = ids
        .iter()
        .filter(|id| symbols.remove(id.as_str()).is_some())
        .count();
    if removed == 0 {
        return 0;
    }
    unlist(doc, ids);
    if let Some(list) = doc
        .get_mut("defaultFavorites")
        .and_then(Value::as_array_mut)
    {
        list.retain(|v| v.as_str().is_none_or(|id| !ids.contains(id)));
    }
    if let Some(aliases) = doc.get_mut("aliases").and_then(Value::as_object_mut) {
        aliases.retain(|_, to| to.as_str().is_none_or(|id| !ids.contains(id)));
    }
    removed
}

/// Permanently removes `ids`; each category loses them and is dropped when
/// that leaves it empty. Returns how many symbols the file held.
pub fn delete(path: &Path, ids: &BTreeSet<String>) -> Edit {
    rewrite(path, |doc| remove(doc, ids))
}

/// Removes a category and its symbols, including an already empty category.
/// Unrelated empty categories and unknown document fields are preserved.
pub fn delete_category(path: &Path, key: &str) -> Edit {
    rewrite(path, |doc| {
        let listed = doc["categories"]
            .as_array()
            .is_some_and(|all| all.iter().any(|c| c["key"] == key));
        let ids: BTreeSet<String> = doc["symbols"]
            .as_object()
            .into_iter()
            .flat_map(|symbols| symbols.iter())
            .filter(|(_, symbol)| symbol["category"] == key)
            .map(|(id, _)| id.clone())
            .collect();
        let removed = remove(doc, &ids);
        if let Some(all) = categories(doc) {
            all.retain(|category| category["key"] != key);
        }
        removed + usize::from(listed)
    })
}

/// Sets the label (and name) of each symbol in `labels`.
pub fn rename(path: &Path, labels: &BTreeMap<String, String>) -> Edit {
    rewrite(path, |doc| {
        let Some(symbols) = doc.get_mut("symbols").and_then(Value::as_object_mut) else {
            return 0;
        };
        let mut changed = 0;
        for (id, label) in labels {
            if let Some(symbol) = symbols.get_mut(id) {
                if symbol["label"] != *label || symbol["name"] != *label {
                    symbol["label"] = Value::from(label.as_str());
                    symbol["name"] = Value::from(label.as_str());
                    changed += 1;
                }
            }
        }
        changed
    })
}

/// Files `ids` under the category `key`, creating it as `name` when the file
/// has none, and dropping a category this empties. The ids themselves stay, so
/// placed instances and favorites keep working.
pub fn move_to_category(path: &Path, ids: &BTreeSet<String>, key: &str, name: &str) -> Edit {
    rewrite(path, |doc| {
        let Some(symbols) = doc.get_mut("symbols").and_then(Value::as_object_mut) else {
            return 0;
        };
        let moved: BTreeSet<String> = ids
            .iter()
            .filter(|id| {
                symbols
                    .get(id.as_str())
                    .is_some_and(|s| s["category"].as_str() != Some(key))
            })
            .cloned()
            .collect();
        if moved.is_empty() {
            return 0;
        }
        let existing = categories(doc)
            .and_then(|all| all.iter().find(|c| c["key"].as_str() == Some(key)))
            .and_then(|c| c["name"].as_str().map(str::to_owned));
        let name = existing.unwrap_or_else(|| name.to_owned());
        if let Some(symbols) = doc.get_mut("symbols").and_then(Value::as_object_mut) {
            for id in &moved {
                if let Some(symbol) = symbols.get_mut(id) {
                    symbol["category"] = Value::from(key);
                    symbol["categoryName"] = Value::from(name.as_str());
                }
            }
        }
        unlist(doc, &moved);
        if let Some(all) = categories(doc) {
            if !all.iter().any(|c| c["key"].as_str() == Some(key)) {
                all.push(serde_json::json!({"key": key, "name": name, "symbolIds": []}));
            }
            if let Some(list) = all
                .iter_mut()
                .find(|c| c["key"].as_str() == Some(key))
                .and_then(|c| c.get_mut("symbolIds"))
                .and_then(Value::as_array_mut)
            {
                list.extend(moved.iter().map(|id| Value::from(id.as_str())));
            }
        }
        moved.len()
    })
}

/// Renames the category `key` and the category name every one of its symbols
/// carries.
pub fn rename_category(path: &Path, key: &str, name: &str) -> Edit {
    rewrite(path, |doc| {
        let Some(category) =
            categories(doc).and_then(|all| all.iter_mut().find(|c| c["key"].as_str() == Some(key)))
        else {
            return 0;
        };
        if category["name"].as_str() == Some(name) {
            return 0;
        }
        category["name"] = Value::from(name);
        if let Some(symbols) = doc.get_mut("symbols").and_then(Value::as_object_mut) {
            for symbol in symbols.values_mut() {
                if symbol["category"].as_str() == Some(key) {
                    symbol["categoryName"] = Value::from(name);
                }
            }
        }
        1
    })
}

/// Folds each `(retired, kept)` pair: `retired` is deleted, the favorites list
/// points at `kept` instead, and an alias records the swap so sketches that
/// placed `retired` still resolve (aliases that led to `retired` follow).
/// Returns how many symbols were retired.
pub fn merge(path: &Path, pairs: &[(String, String)]) -> Edit {
    rewrite(path, |doc| {
        let retired: BTreeSet<String> = pairs.iter().map(|(from, _)| from.clone()).collect();
        let exists = |doc: &Value, id: &str| doc["symbols"].get(id).is_some();
        let pairs: Vec<&(String, String)> = pairs
            .iter()
            .filter(|(from, into)| {
                from != into
                    && exists(doc, from)
                    && exists(doc, into)
                    && !retired.contains(into.as_str())
            })
            .collect();
        if pairs.is_empty() {
            return 0;
        }
        let swap: BTreeMap<&str, &str> = pairs
            .iter()
            .map(|(from, into)| (from.as_str(), into.as_str()))
            .collect();
        if let Some(favorites) = doc
            .get_mut("defaultFavorites")
            .and_then(Value::as_array_mut)
        {
            let mut seen = BTreeSet::new();
            let mapped: Vec<Value> = favorites
                .iter()
                .map(|v| match v.as_str() {
                    Some(id) => Value::from(*swap.get(id).unwrap_or(&id)),
                    None => v.clone(),
                })
                .filter(|v| v.as_str().is_none_or(|id| seen.insert(id.to_owned())))
                .collect();
            *favorites = mapped;
        }
        let mut aliases = doc
            .get("aliases")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        for to in aliases.values_mut() {
            if let Some(into) = to.as_str().and_then(|id| swap.get(id)) {
                *to = Value::from(*into);
            }
        }
        for (from, into) in &pairs {
            aliases.insert(from.clone(), Value::from(into.as_str()));
        }
        let ids: BTreeSet<String> = pairs.iter().map(|(from, _)| from.clone()).collect();
        let removed = remove(doc, &ids);
        doc["aliases"] = Value::Object(aliases);
        removed
    })
}

/// Editable metadata/wall-behaviour properties for [`update_properties`].
#[derive(Debug, Clone, Default)]
pub struct SymbolProps {
    pub name: Option<String>,
    pub description: Option<String>,
    pub wall_snap: Option<WallSnap>,
    pub wall_gap_width: Option<f64>,
    pub wall_gap_alignment: Option<f64>,
    pub wall_cap_length: Option<f64>,
}

/// Result returned by [`import_with_report`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub added: usize,
    pub conflicts: Vec<String>,
}

fn formatted_label(category_name: &str, name: &str) -> String {
    let cat = category_name.trim();
    if cat.is_empty() {
        name.to_owned()
    } else {
        format!("{cat} - {name}")
    }
}

fn set_f64_if_changed(obj: &mut serde_json::Map<String, Value>, key: &str, val: f64) -> bool {
    if !val.is_finite() {
        return false;
    }
    let unchanged = obj
        .get(key)
        .and_then(Value::as_f64)
        .is_some_and(|prev| (prev - val).abs() <= 1e-9);
    if unchanged {
        return false;
    }
    obj.insert(key.to_owned(), Value::from(val));
    true
}

fn ensure_category_has_symbol(doc: &mut Value, cat_key: &str, cat_name: &str, sym_id: &str) {
    if cat_key.is_empty() {
        return;
    }
    let all = match categories(doc) {
        Some(all) => all,
        None => {
            doc["categories"] = Value::Array(Vec::new());
            categories(doc).expect("categories array just initialized")
        }
    };
    let name = if cat_name.trim().is_empty() {
        cat_key
    } else {
        cat_name.trim()
    };
    if let Some(cat) = all.iter_mut().find(|c| c["key"].as_str() == Some(cat_key)) {
        if cat["name"].as_str().unwrap_or("").trim().is_empty() {
            cat["name"] = Value::from(name);
        }
        let ids = match cat.get_mut("symbolIds").and_then(Value::as_array_mut) {
            Some(ids) => ids,
            None => {
                cat["symbolIds"] = Value::Array(Vec::new());
                cat.get_mut("symbolIds")
                    .and_then(Value::as_array_mut)
                    .expect("symbolIds array just initialized")
            }
        };
        if !ids.iter().any(|v| v.as_str() == Some(sym_id)) {
            ids.push(Value::from(sym_id));
        }
    } else {
        all.push(serde_json::json!({
            "key": cat_key,
            "name": name,
            "symbolIds": [sym_id],
        }));
    }
}

/// Updates properties and wall-snap behaviour on symbol `id` inside `path`.
pub fn update_properties(path: &Path, id: &str, props: &SymbolProps) -> Edit {
    rewrite(path, |doc| {
        let Some(sym) = doc
            .get_mut("symbols")
            .and_then(Value::as_object_mut)
            .and_then(|s| s.get_mut(id))
            .and_then(Value::as_object_mut)
        else {
            return 0;
        };
        let mut dirty = false;
        if let Some(name) = &props.name {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                let cat_name = sym
                    .get("categoryName")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let next_label = formatted_label(&cat_name, trimmed);
                if sym.get("name").and_then(Value::as_str) != Some(trimmed) {
                    sym.insert("name".into(), Value::from(trimmed));
                    dirty = true;
                }
                if sym.get("label").and_then(Value::as_str) != Some(next_label.as_str()) {
                    sym.insert("label".into(), Value::from(next_label));
                    dirty = true;
                }
            }
        }
        if let Some(desc) = &props.description {
            if sym.get("description").and_then(Value::as_str) != Some(desc.as_str()) {
                sym.insert("description".into(), Value::from(desc.as_str()));
                dirty = true;
            }
        }
        if let Some(snap) = props.wall_snap {
            let snap_val = serde_json::to_value(snap).unwrap_or(Value::String("NONE".into()));
            if sym.get("wallSnap") != Some(&snap_val) {
                sym.insert("wallSnap".into(), snap_val);
                dirty = true;
            }
        }
        if let Some(gap) = props.wall_gap_width {
            dirty |= set_f64_if_changed(sym, "wallGapWidth", gap.max(0.0));
        }
        if let Some(align) = props.wall_gap_alignment {
            dirty |= set_f64_if_changed(sym, "wallGapAlignment", align);
        }
        if let Some(cap) = props.wall_cap_length {
            dirty |= set_f64_if_changed(sym, "wallCapLength", cap.max(0.0));
        }
        usize::from(dirty)
    })
}

/// Replaces geometry and bounds on symbol `id` inside `path`.
pub fn replace_geometry(path: &Path, id: &str, geometry: &[Entity], bounds: Bounds) -> Edit {
    rewrite(path, |doc| {
        let Some(sym) = doc
            .get_mut("symbols")
            .and_then(Value::as_object_mut)
            .and_then(|s| s.get_mut(id))
            .and_then(Value::as_object_mut)
        else {
            return 0;
        };
        let next_geom = serde_json::to_value(geometry).unwrap_or(Value::Array(Vec::new()));
        let next_bounds = serde_json::to_value(bounds).unwrap_or(Value::Null);
        let changed =
            sym.get("geometry") != Some(&next_geom) || sym.get("bounds") != Some(&next_bounds);
        if !changed {
            return 0;
        }
        sym.insert("geometry".into(), next_geom);
        sym.insert("bounds".into(), next_bounds);
        1
    })
}

/// Adds a new symbol with a unique `symbol.id` to `path` and files it in its category.
pub fn add(path: &Path, symbol: &Symbol) -> Edit {
    if symbol.id.trim().is_empty() {
        return Ok(0);
    }
    rewrite(path, |doc| {
        if doc.get("symbols").and_then(Value::as_object).is_none() {
            doc["symbols"] = Value::Object(serde_json::Map::new());
        }
        let Some(symbols) = doc.get_mut("symbols").and_then(Value::as_object_mut) else {
            return 0;
        };
        if symbols.contains_key(&symbol.id) {
            return 0;
        }
        let mut to_insert = symbol.clone();
        if to_insert.units.trim().is_empty() {
            to_insert.units = "INCHES".to_owned();
        }
        if to_insert.label.trim().is_empty() && !to_insert.name.trim().is_empty() {
            to_insert.label = formatted_label(&to_insert.category_name, to_insert.name.trim());
        }
        let Ok(val) = serde_json::to_value(&to_insert) else {
            return 0;
        };
        symbols.insert(to_insert.id.clone(), val);
        ensure_category_has_symbol(
            doc,
            &to_insert.category,
            &to_insert.category_name,
            &to_insert.id,
        );
        1
    })
}

/// Duplicates symbol `id` to unique `new_id` with display name `new_name`.
pub fn duplicate(path: &Path, id: &str, new_id: &str, new_name: &str) -> Edit {
    let new_id = new_id.trim();
    let new_name = new_name.trim();
    if new_id.is_empty() || new_id == id || new_name.is_empty() {
        return Ok(0);
    }
    rewrite(path, |doc| {
        let Some(symbols) = doc.get_mut("symbols").and_then(Value::as_object_mut) else {
            return 0;
        };
        if symbols.contains_key(new_id) {
            return 0;
        }
        let Some(mut copy) = symbols.get(id).cloned() else {
            return 0;
        };
        let cat_key = copy["category"].as_str().unwrap_or("").to_owned();
        let cat_name = copy["categoryName"].as_str().unwrap_or("").to_owned();
        copy["id"] = Value::from(new_id);
        copy["name"] = Value::from(new_name);
        copy["label"] = Value::from(formatted_label(&cat_name, new_name));
        symbols.insert(new_id.to_owned(), copy);
        if !cat_key.is_empty() {
            if let Some(cat) = categories(doc).and_then(|all| {
                all.iter_mut()
                    .find(|c| c["key"].as_str() == Some(cat_key.as_str()))
            }) {
                if let Some(list) = cat.get_mut("symbolIds").and_then(Value::as_array_mut) {
                    if let Some(pos) = list.iter().position(|v| v.as_str() == Some(id)) {
                        list.insert(pos + 1, Value::from(new_id));
                    } else {
                        list.push(Value::from(new_id));
                    }
                    return 1;
                }
            }
            ensure_category_has_symbol(doc, &cat_key, &cat_name, new_id);
        }
        1
    })
}

/// Adds an empty category `(key, name)` to `path` if `key` is not already present.
pub fn add_category(path: &Path, key: &str, name: &str) -> Edit {
    let key = key.trim();
    let name = name.trim();
    if key.is_empty() || name.is_empty() {
        return Ok(0);
    }
    rewrite(path, |doc| {
        if categories(doc).is_none() {
            doc["categories"] = Value::Array(Vec::new());
        }
        let Some(all) = categories(doc) else {
            return 0;
        };
        if all.iter().any(|c| c["key"].as_str() == Some(key)) {
            return 0;
        }
        all.push(serde_json::json!({
            "key": key,
            "name": name,
            "symbolIds": [],
        }));
        1
    })
}

/// Reorders `doc["categories"]` so keys in `keys` lead in the given order; any
/// remaining categories keep their relative order afterward.
pub fn reorder_categories(path: &Path, keys: &[String]) -> Edit {
    rewrite(path, |doc| {
        let Some(all) = categories(doc) else {
            return 0;
        };
        let old_keys: Vec<String> = all
            .iter()
            .filter_map(|c| c["key"].as_str().map(str::to_owned))
            .collect();
        let mut by_key: BTreeMap<String, Value> = BTreeMap::new();
        let mut tail = Vec::new();
        for cat in all.drain(..) {
            if let Some(k) = cat["key"].as_str() {
                by_key.insert(k.to_owned(), cat);
            } else {
                tail.push(cat);
            }
        }
        let mut reordered = Vec::with_capacity(old_keys.len() + tail.len());
        for k in keys {
            if let Some(cat) = by_key.remove(k) {
                reordered.push(cat);
            }
        }
        for k in &old_keys {
            if let Some(cat) = by_key.remove(k) {
                reordered.push(cat);
            }
        }
        reordered.extend(tail);
        let new_keys: Vec<String> = reordered
            .iter()
            .filter_map(|c| c["key"].as_str().map(str::to_owned))
            .collect();
        *all = reordered;
        usize::from(new_keys != old_keys)
    })
}

/// Reorders `symbolIds` inside `category` according to `ids`; remaining ids
/// keep their relative order afterward.
pub fn reorder_symbols(path: &Path, category: &str, ids: &[String]) -> Edit {
    rewrite(path, |doc| {
        let Some(cat) = categories(doc)
            .and_then(|all| all.iter_mut().find(|c| c["key"].as_str() == Some(category)))
        else {
            return 0;
        };
        let Some(list) = cat.get_mut("symbolIds").and_then(Value::as_array_mut) else {
            return 0;
        };
        let old_ids: Vec<String> = list
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
        let old_set: BTreeSet<&str> = old_ids.iter().map(String::as_str).collect();
        let mut seen = BTreeSet::new();
        let mut new_ids: Vec<String> = Vec::with_capacity(old_ids.len());
        for id in ids {
            if old_set.contains(id.as_str()) && seen.insert(id.clone()) {
                new_ids.push(id.clone());
            }
        }
        for id in &old_ids {
            if seen.insert(id.clone()) {
                new_ids.push(id.clone());
            }
        }
        if new_ids == old_ids {
            return 0;
        }
        *list = new_ids.into_iter().map(Value::from).collect();
        1
    })
}

/// Writes a library subset (or whole library when `ids` is `None`) to `dest`.
/// Does not modify `path` and returns the number of symbols exported.
pub fn export(path: &Path, dest: &Path, ids: Option<&BTreeSet<String>>) -> Edit {
    let original = std::fs::read_to_string(path)?;
    let mut doc: Value = serde_json::from_str(&original)?;
    if let Some(keep) = ids {
        if let Some(symbols) = doc.get_mut("symbols").and_then(Value::as_object_mut) {
            symbols.retain(|id, _| keep.contains(id));
        }
        if let Some(all) = categories(&mut doc) {
            all.retain_mut(|cat| {
                let Some(list) = cat.get_mut("symbolIds").and_then(Value::as_array_mut) else {
                    return false;
                };
                list.retain(|v| v.as_str().is_some_and(|id| keep.contains(id)));
                !list.is_empty()
            });
        }
        if let Some(favs) = doc
            .get_mut("defaultFavorites")
            .and_then(Value::as_array_mut)
        {
            favs.retain(|v| v.as_str().is_some_and(|id| keep.contains(id)));
        }
        if let Some(aliases) = doc.get_mut("aliases").and_then(Value::as_object_mut) {
            aliases.retain(|_, to| to.as_str().is_some_and(|id| keep.contains(id)));
        }
    }
    let exported = doc
        .get("symbols")
        .and_then(Value::as_object)
        .map(|s| s.len())
        .unwrap_or(0);
    doc["symbolCount"] = Value::from(exported);
    if let Some(parent) = dest.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(dest, serde_json::to_vec_pretty(&doc)?)?;
    Ok(exported)
}

/// Imports symbols from `source` into `path` by id, skipping colliding ids and
/// reporting them alongside the count added.
pub fn import_with_report(path: &Path, source: &Path) -> Result<ImportReport, SymbolLibraryError> {
    let body = std::fs::read_to_string(source)?;
    let src: Value = serde_json::from_str(&body)?;
    let mut category_names: BTreeMap<String, String> = BTreeMap::new();
    if let Some(cats) = src.get("categories").and_then(Value::as_array) {
        for c in cats {
            if let (Some(k), Some(n)) = (c["key"].as_str(), c["name"].as_str()) {
                category_names.insert(k.to_owned(), n.to_owned());
            }
        }
    }
    let incoming: Vec<(String, Value)> =
        if let Some(symbols) = src.get("symbols").and_then(Value::as_object) {
            symbols
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else if src.get("geometry").is_some() {
            let fallback_id = src["id"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    let stem = source
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("imported");
                    format!("imported/{stem}")
                });
            vec![(fallback_id, src)]
        } else {
            Vec::new()
        };

    let mut report = ImportReport::default();
    rewrite(path, |doc| {
        if doc.get("symbols").and_then(Value::as_object).is_none() {
            doc["symbols"] = Value::Object(serde_json::Map::new());
        }
        let mut added = 0;
        for (sym_id, mut sym_val) in incoming {
            let exists = doc
                .get("symbols")
                .and_then(Value::as_object)
                .is_some_and(|s| s.contains_key(&sym_id));
            if exists {
                report.conflicts.push(sym_id);
                continue;
            }
            if sym_val["id"].as_str().unwrap_or("").trim().is_empty() {
                sym_val["id"] = Value::from(sym_id.as_str());
            }
            let cat_key = sym_val["category"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("imported")
                .to_owned();
            let cat_name = sym_val["categoryName"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .map(str::to_owned)
                .or_else(|| category_names.get(&cat_key).cloned())
                .unwrap_or_else(|| cat_key.clone());
            sym_val["category"] = Value::from(cat_key.as_str());
            sym_val["categoryName"] = Value::from(cat_name.as_str());
            if let Some(symbols) = doc.get_mut("symbols").and_then(Value::as_object_mut) {
                symbols.insert(sym_id.clone(), sym_val);
            }
            ensure_category_has_symbol(doc, &cat_key, &cat_name, &sym_id);
            added += 1;
        }
        report.added = added;
        added
    })?;
    Ok(report)
}

/// Imports symbols from `source` into `path` by id; returns how many symbols were added.
pub fn import(path: &Path, source: &Path) -> Edit {
    import_with_report(path, source).map(|report| report.added)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::SymbolLibrary;

    const LIBRARY: &str = r#"{"version":1,"unknownTop":5,"symbolCount":4,
        "categories":[{"key":"a","name":"A","symbolIds":["a/one","a/two"]},
                      {"key":"b","name":"B","symbolIds":["b/three"]},
                      {"key":"favorites","name":"Favorites","symbolIds":["favorites/one"]},
                      {"key":"empty","name":"Empty","symbolIds":[]}],
        "defaultFavorites":["favorites/one","b/three"],
        "symbols":{
          "a/one":{"id":"a/one","category":"a","categoryName":"A","label":"One","name":"One","futureField":1,"geometry":[{"entityType":"LINE","startPoint":{"x":0,"y":0},"endPoint":{"x":1,"y":1}}]},
          "a/two":{"id":"a/two","category":"a","categoryName":"A","label":"Two","name":"Two"},
          "b/three":{"id":"b/three","category":"b","categoryName":"B","label":"Three","name":"Three"},
          "favorites/one":{"id":"favorites/one","category":"favorites","categoryName":"Favorites","label":"One","name":"One","geometry":[{"entityType":"LINE","startPoint":{"x":0,"y":0},"endPoint":{"x":1,"y":1}}]}}}"#;

    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new(tag: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("sketch-symedit-{}-{tag}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("symbols.json"), LIBRARY).unwrap();
            Fixture(dir)
        }
        fn path(&self) -> std::path::PathBuf {
            self.0.join("symbols.json")
        }
        fn load(&self) -> SymbolLibrary {
            SymbolLibrary::load(&self.path()).unwrap()
        }
        fn raw(&self) -> Value {
            serde_json::from_str(&std::fs::read_to_string(self.path()).unwrap()).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn ids(v: &[&str]) -> BTreeSet<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn delete_refuses_a_read_only_library_and_leaves_it_unchanged() {
        let f = Fixture::new("readonly");
        let path = f.path();
        let before = std::fs::read(&path).unwrap();
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&path, perms.clone()).unwrap();
        let err = delete(&path, &ids(&["a/one"])).expect_err("read-only must fail");
        assert!(
            matches!(err, SymbolLibraryError::Io(ref e) if e.kind() == std::io::ErrorKind::PermissionDenied),
            "{err:?}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(!f.0.join("symbols.json.bak").exists());
        assert!(!f.0.join("symbols.json.tmp").exists());
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        std::fs::set_permissions(&path, perms).unwrap();
        assert_eq!(delete(&path, &ids(&["a/one"])).unwrap(), 1);
    }

    #[test]
    fn delete_removes_symbols_categories_favorites_and_keeps_a_backup() {
        let f = Fixture::new("delete");
        assert_eq!(delete(&f.path(), &ids(&["nope/x"])).unwrap(), 0);
        assert!(
            !f.0.join("symbols.json.bak").exists(),
            "no match writes nothing"
        );
        assert_eq!(
            delete(&f.path(), &ids(&["a/one", "b/three", "nope/x"])).unwrap(),
            2
        );
        let library = f.load();
        assert_eq!(library.symbol_count, 2);
        assert_eq!(library.default_favorites, ["favorites/one"]);
        let keys: Vec<_> = library.categories.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(
            keys,
            ["a", "favorites", "empty"],
            "only categories this emptied go"
        );
        assert_eq!(
            f.raw()["unknownTop"],
            5,
            "fields the port does not model survive"
        );
        assert_eq!(f.raw()["symbols"]["a/two"]["id"], "a/two");
        assert_eq!(
            std::fs::read_to_string(f.0.join("symbols.json.bak")).unwrap(),
            LIBRARY
        );
        assert!(!f.0.join("symbols.json.tmp").exists());
    }

    #[test]
    fn category_deletion_removes_empty_metadata_and_preserves_other_categories() {
        let f = Fixture::new("delete-category");
        assert_eq!(delete_category(&f.path(), "missing").unwrap(), 0);
        assert!(!f.0.join("symbols.json.bak").exists());
        assert_eq!(delete_category(&f.path(), "empty").unwrap(), 1);
        assert_eq!(f.load().symbols.len(), 4);
        assert!(!f.load().categories.iter().any(|c| c.key == "empty"));
        assert_eq!(
            std::fs::read_to_string(f.0.join("symbols.json.bak")).unwrap(),
            LIBRARY
        );
        assert_eq!(delete_category(&f.path(), "a").unwrap(), 3);
        assert!(!f.load().categories.iter().any(|c| c.key == "a"));
        assert!(f.load().symbols.contains_key("b/three"));
        assert_eq!(f.raw()["unknownTop"], 5);
    }

    #[test]
    fn rename_and_rename_category_touch_only_what_changed() {
        let f = Fixture::new("rename");
        let same = BTreeMap::from([("a/one".to_string(), "One".to_string())]);
        assert_eq!(rename(&f.path(), &same).unwrap(), 0);
        let labels = BTreeMap::from([
            ("a/one".to_string(), "First".to_string()),
            ("nope/x".to_string(), "Ghost".to_string()),
        ]);
        assert_eq!(rename(&f.path(), &labels).unwrap(), 1);
        let library = f.load();
        assert_eq!(library.symbols["a/one"].label, "First");
        assert_eq!(library.symbols["a/one"].name, "First");
        assert_eq!(rename_category(&f.path(), "a", "Alpha").unwrap(), 1);
        assert_eq!(rename_category(&f.path(), "a", "Alpha").unwrap(), 0);
        let library = f.load();
        assert_eq!(library.categories[0].name, "Alpha");
        assert_eq!(library.symbols["a/two"].category_name, "Alpha");
        assert_eq!(library.symbols["b/three"].category_name, "B");
    }

    #[test]
    fn move_files_symbols_under_an_existing_or_new_category_and_keeps_their_ids() {
        let f = Fixture::new("move");
        assert_eq!(
            move_to_category(&f.path(), &ids(&["a/one"]), "a", "A").unwrap(),
            0
        );
        assert_eq!(
            move_to_category(&f.path(), &ids(&["b/three"]), "a", "ignored").unwrap(),
            1
        );
        let library = f.load();
        let keys: Vec<_> = library.categories.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(
            keys,
            ["a", "favorites", "empty"],
            "b was emptied and dropped"
        );
        assert_eq!(
            library.categories[0].symbol_ids,
            ["a/one", "a/two", "b/three"]
        );
        assert_eq!(library.symbols["b/three"].category, "a");
        assert_eq!(
            library.symbols["b/three"].category_name, "A",
            "existing name wins"
        );
        assert_eq!(
            move_to_category(&f.path(), &ids(&["a/two"]), "new", "Brand New").unwrap(),
            1
        );
        let library = f.load();
        let new = library.categories.last().unwrap();
        assert_eq!((new.key.as_str(), new.name.as_str()), ("new", "Brand New"));
        assert_eq!(new.symbol_ids, ["a/two"]);
        assert_eq!(library.symbols["a/two"].category_name, "Brand New");
    }

    #[test]
    fn duplicates_are_found_and_merge_keeps_one_with_an_alias_and_favorites() {
        let f = Fixture::new("merge");
        let groups = f.load().duplicate_groups();
        assert_eq!(
            groups,
            [["a/one", "favorites/one"]],
            "the favorites copy goes last"
        );
        let pair = ("favorites/one".to_string(), "a/one".to_string());
        assert_eq!(
            merge(&f.path(), &[("a/one".to_string(), "a/one".to_string())]).unwrap(),
            0
        );
        assert_eq!(merge(&f.path(), &[pair]).unwrap(), 1);
        let library = f.load();
        assert!(library.symbol("favorites/one").is_none());
        assert_eq!(library.default_favorites, ["a/one", "b/three"]);
        assert_eq!(library.aliases["favorites/one"], "a/one");
        assert!(!library.categories.iter().any(|c| c.key == "favorites"));
        // A later merge of the kept symbol rewrites the alias that led to it, and
        // deleting the final target drops the alias.
        let again = ("a/one".to_string(), "a/two".to_string());
        assert_eq!(merge(&f.path(), &[again]).unwrap(), 1);
        assert_eq!(f.load().aliases["favorites/one"], "a/two");
        assert_eq!(f.load().aliases["a/one"], "a/two");
        assert_eq!(delete(&f.path(), &ids(&["a/two"])).unwrap(), 1);
        assert!(f.load().aliases.is_empty());
    }

    #[test]
    fn update_properties_and_replace_geometry_preserve_unknown_fields() {
        let f = Fixture::new("props-geom");
        let unchanged = SymbolProps {
            name: Some("One".into()),
            ..Default::default()
        };
        assert_eq!(
            update_properties(&f.path(), "a/one", &unchanged).unwrap(),
            1,
            "label becomes A - One"
        );
        assert_eq!(
            update_properties(&f.path(), "a/one", &unchanged).unwrap(),
            0,
            "no-op writes nothing"
        );

        let props = SymbolProps {
            name: Some("Updated One".into()),
            description: Some("Door fixture".into()),
            wall_snap: Some(WallSnap::Side),
            wall_gap_width: Some(36.0),
            wall_gap_alignment: Some(1.5),
            wall_cap_length: Some(4.0),
        };
        assert_eq!(update_properties(&f.path(), "a/one", &props).unwrap(), 1);
        let raw = f.raw();
        assert_eq!(raw["symbols"]["a/one"]["futureField"], 1);
        let lib = f.load();
        let s = lib.symbol("a/one").unwrap();
        assert_eq!(s.name, "Updated One");
        assert_eq!(s.label, "A - Updated One");
        assert_eq!(s.description, "Door fixture");
        assert_eq!(s.wall_snap, WallSnap::Side);
        assert_eq!(s.wall_gap_width, 36.0);
        assert_eq!(s.wall_gap_alignment, 1.5);
        assert_eq!(s.wall_cap_length, 4.0);

        let next_geom = vec![Entity {
            start_point: Some(crate::symbols::Point { x: -12.0, y: 0.0 }),
            end_point: Some(crate::symbols::Point { x: 12.0, y: 6.0 }),
            ..Default::default()
        }];
        let next_bounds = Bounds {
            min_x: -12.0,
            min_y: 0.0,
            max_x: 12.0,
            max_y: 6.0,
        };
        assert_eq!(
            replace_geometry(&f.path(), "a/one", &next_geom, next_bounds).unwrap(),
            1
        );
        assert_eq!(
            replace_geometry(&f.path(), "a/one", &next_geom, next_bounds).unwrap(),
            0
        );
        let lib = f.load();
        let s = lib.symbol("a/one").unwrap();
        assert_eq!(s.geometry.len(), 1);
        assert_eq!(s.bounds, next_bounds);
        assert!(f.0.join("symbols.json.bak").is_file());
    }

    #[test]
    fn add_duplicate_categories_and_reorder_keep_counts_and_ids_consistent() {
        let f = Fixture::new("add-dup-order");
        let new_sym = Symbol {
            id: "a/custom".into(),
            category: "a".into(),
            category_name: "A".into(),
            name: "Custom Box".into(),
            label: "".into(),
            bounds: Bounds {
                min_x: -6.0,
                min_y: -6.0,
                max_x: 6.0,
                max_y: 6.0,
            },
            geometry: vec![Entity {
                start_point: Some(crate::symbols::Point { x: -6.0, y: 0.0 }),
                end_point: Some(crate::symbols::Point { x: 6.0, y: 0.0 }),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(add(&f.path(), &new_sym).unwrap(), 1);
        assert_eq!(add(&f.path(), &new_sym).unwrap(), 0, "unique id enforced");
        let lib = f.load();
        assert_eq!(lib.symbol_count, 5);
        assert_eq!(lib.symbol("a/custom").unwrap().label, "A - Custom Box");
        assert!(lib.categories[0]
            .symbol_ids
            .contains(&"a/custom".to_owned()));

        assert_eq!(
            duplicate(&f.path(), "a/one", "a/one-copy", "One Copy").unwrap(),
            1
        );
        assert_eq!(
            duplicate(&f.path(), "a/one", "a/one-copy", "One Copy").unwrap(),
            0
        );
        let raw = f.raw();
        assert_eq!(raw["symbols"]["a/one-copy"]["futureField"], 1);
        let lib = f.load();
        assert_eq!(lib.symbol_count, 6);
        assert_eq!(lib.symbol("a/one-copy").unwrap().label, "A - One Copy");

        assert_eq!(add_category(&f.path(), "fresh", "Fresh Cat").unwrap(), 1);
        assert_eq!(add_category(&f.path(), "fresh", "Fresh Cat").unwrap(), 0);
        assert_eq!(
            reorder_categories(&f.path(), &["fresh".into(), "b".into(), "a".into()]).unwrap(),
            1
        );
        assert_eq!(
            reorder_categories(&f.path(), &["fresh".into(), "b".into(), "a".into()]).unwrap(),
            0
        );
        let lib = f.load();
        let keys: Vec<_> = lib.categories.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(keys, ["fresh", "b", "a", "favorites", "empty"]);

        assert_eq!(
            reorder_symbols(
                &f.path(),
                "a",
                &["a/custom".into(), "a/two".into(), "a/one".into()]
            )
            .unwrap(),
            1
        );
        let lib = f.load();
        let a_cat = lib.categories.iter().find(|c| c.key == "a").unwrap();
        assert_eq!(
            a_cat.symbol_ids,
            ["a/custom", "a/two", "a/one", "a/one-copy"]
        );
    }

    #[test]
    fn export_and_import_round_trip_and_report_conflicts() {
        let src = Fixture::new("export-src");
        let dest = src.0.join("subset.json");
        assert_eq!(
            export(&src.path(), &dest, Some(&ids(&["a/one", "b/three"]))).unwrap(),
            2
        );

        let clean = Fixture::new("clean-dst");
        let clean_empty = delete(
            &clean.path(),
            &ids(&["a/one", "a/two", "b/three", "favorites/one"]),
        )
        .unwrap();
        assert_eq!(clean_empty, 4);
        assert_eq!(import(&clean.path(), &dest).unwrap(), 2);
        let clean_lib = clean.load();
        assert_eq!(clean_lib.symbol_count, 2);
        assert!(clean_lib.symbol("a/one").is_some());
        assert!(clean_lib.symbol("b/three").is_some());

        // Re-importing the subset reports both ids as conflicts and changes nothing.
        let report = import_with_report(&clean.path(), &dest).unwrap();
        assert_eq!(report.added, 0);
        assert_eq!(report.conflicts.len(), 2);
        assert!(report.conflicts.contains(&"a/one".to_owned()));
        assert!(report.conflicts.contains(&"b/three".to_owned()));
    }
}
