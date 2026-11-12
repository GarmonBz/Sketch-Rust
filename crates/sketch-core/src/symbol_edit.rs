//! In-place edits of the external library file (`symbols.json`).
//!
//! Every edit works on the raw JSON, so fields this port does not model
//! survive, and goes through [`rewrite`]: the file as it was is kept beside it
//! as `<name>.bak`, the new one is written whole and swapped in, so a failed
//! write leaves the library as it was. An edit that changes nothing writes
//! nothing.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::symbols::{SymbolLibraryError, SYMBOL_LIBRARY_FILENAME};

type Edit = Result<usize, SymbolLibraryError>;

/// Runs `edit` on the library document at `path`; when it reports a non-zero
/// count the document is saved (see the module docs) and the count returned.
fn rewrite(path: &Path, edit: impl FnOnce(&mut Value) -> usize) -> Edit {
    let original = std::fs::read_to_string(path)?;
    let mut doc: Value = serde_json::from_str(&original)?;
    let changed = edit(&mut doc);
    if changed == 0 {
        return Ok(0);
    }
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
}
