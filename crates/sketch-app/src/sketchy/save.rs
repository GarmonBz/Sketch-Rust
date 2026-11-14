//! Update the Sketchy document using the IDs assigned when it was opened.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::{
    bulge_sign, units, Converted, Frame, Point, Result, CURRENT_VERSION, FORMAT, LEGACY_FORMAT,
};

const NAMESPACE: &str = "app.sketchrust";
const WORKFILE: &str = "app.sketchrust.workfile";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PageKey {
    Site,
    Level(String),
}

impl PageKey {
    fn level(&self) -> Option<&str> {
        match self {
            Self::Site => None,
            Self::Level(id) => Some(id),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Binding {
    id: String,
    entity: Entity,
}

impl Binding {
    pub(super) fn new(id: &str, entity: Entity) -> Self {
        Self {
            id: id.into(),
            entity,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) enum Entity {
    Space { area: i64, holes: Vec<i64> },
    OpenSpace(Vec<i64>),
    Void(i64),
    VoidWithHoles { area: i64, holes: Vec<i64> },
    Wall(Vec<i64>),
    Opening(i64),
    Feature(i64),
    Text(i64),
    Drawing(Vec<i64>),
    Label(i64),
    Symbol(i64),
    North,
}

impl Entity {
    fn collection(&self) -> &'static str {
        match self {
            Self::Space { .. } | Self::OpenSpace(_) => "spaces",
            Self::Void(_) | Self::VoidWithHoles { .. } => "voids",
            Self::Wall(_) => "walls",
            Self::Opening(_) => "openings",
            Self::Feature(_) => "features",
            _ => "annotations",
        }
    }

    fn path_lines(&self) -> &[i64] {
        match self {
            Self::Wall(ids) | Self::OpenSpace(ids) | Self::Drawing(ids) => ids,
            _ => &[],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Snapshot {
    pub model: Value,
    pub bindings: Vec<(PageKey, Vec<Binding>)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Backing {
    source: Value,
    snapshot: Snapshot,
    has_workfile: bool,
}

impl Backing {
    pub fn version(&self) -> Option<&str> {
        self.source["format_version"].as_str()
    }

    pub fn revision_id(&self) -> Option<&str> {
        self.source["revision"]["id"].as_str()
    }

    pub fn revision_hash(&self) -> Option<&str> {
        self.source["revision"]["content_hash"].as_str()
    }
    pub fn from_model(model: &Value) -> Option<Self> {
        let backing = Self::deserialize(model.get("sketchyBacking")?).ok()?;
        (super::recognizes(&backing.source) && backing.source["structures"].is_array())
            .then_some(backing)
    }
}

#[derive(Serialize, Deserialize)]
struct Workfile {
    projection: String,
    snapshot: Snapshot,
}

fn projection(document: &Value) -> String {
    let mut content = document
        .as_object()
        .expect("Sketchy document object")
        .clone();
    content.remove("revision");
    content.remove("history");
    if content
        .get("extras")
        .and_then(Value::as_object)
        .is_some_and(Map::is_empty)
    {
        content.remove("extras");
    }
    sketch_core::skx::sha256_hex(Value::Object(content).to_string().as_bytes())
}

pub(super) fn take_snapshot(source: &mut Value) -> Option<Snapshot> {
    let extras = source.get_mut("extras")?.as_object_mut()?;
    let raw = extras.remove(WORKFILE)?;
    if extras.is_empty() {
        source.as_object_mut()?.remove("extras");
    }
    let workfile: Workfile = serde_json::from_value(raw).ok()?;
    let pages = workfile.snapshot.model.get("pages")?.as_array()?;
    if pages.is_empty()
        || pages.iter().any(|page| {
            page_key(page).is_none()
                || ["lines", "polygons", "labels", "symbols"]
                    .iter()
                    .any(|key| !page[*key].is_array())
        })
    {
        return None;
    }
    // An interchange editor may modify the core document while retaining extras.
    // In that case the core geometry is authoritative on the next open.
    (projection(source) == workfile.projection).then_some(workfile.snapshot)
}

pub(super) fn converted(
    source: Value,
    snapshot: Snapshot,
    has_workfile: bool,
    omitted: bool,
) -> Converted {
    let mut model = snapshot.model.clone();
    model["sketchyBacking"] = json!(Backing {
        source,
        snapshot,
        has_workfile
    });
    Converted { model, omitted }
}

pub struct Prepared {
    pub bytes: Vec<u8>,
    pub backing: Backing,
    pub page_keys: Vec<PageKey>,
}

fn new_id() -> String {
    let mut bytes = [0; 16];
    if getrandom::fill(&mut bytes).is_ok() {
        return uuid::Builder::from_random_bytes(bytes)
            .into_uuid()
            .to_string();
    }
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "sketch-{:x}-{time:x}-{:x}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

fn array(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or_default()
}

fn list<'a>(value: &'a mut Value, key: &str) -> &'a mut Vec<Value> {
    if !value[key].is_array() {
        value[key] = json!([]);
    }
    value[key].as_array_mut().expect("initialized array")
}

fn page_key(page: &Value) -> Option<PageKey> {
    PageKey::deserialize(page.get("sketchyPage")?).ok()
}

fn assign_keys(model: &mut Value) -> Result<Vec<PageKey>> {
    let pages = model["pages"].as_array_mut().ok_or("sketch has no pages")?;
    let mut seen = HashSet::new();
    let mut keys = Vec::with_capacity(pages.len());
    for page in pages {
        let key = page_key(page)
            .filter(|key| !seen.contains(key))
            .unwrap_or_else(|| PageKey::Level(new_id()));
        seen.insert(key.clone());
        page["sketchyPage"] = json!(key);
        keys.push(key);
    }
    if keys.is_empty() {
        return Err("sketch has no pages".into());
    }
    Ok(keys)
}

pub fn prepare_save(
    mut model: Value,
    backing: Option<&Backing>,
    title: &str,
    fork: bool,
    format: Option<&str>,
) -> Result<Prepared> {
    if let Some(map) = model.as_object_mut() {
        for key in ["savedAt", "sketchyBacking", "extendedData"] {
            map.remove(key);
        }
    }
    if let Some(settings) = model["settings"].as_object_mut() {
        settings.remove("apexDocument");
    }
    let page_keys = assign_keys(&mut model)?;
    let mut source = backing.map(|state| state.source.clone()).unwrap_or_else(|| json!({
        "format": FORMAT, "format_version": CURRENT_VERSION, "id": new_id(), "title": title,
        "length_unit": if model.pointer("/settings/options/unitSystem").and_then(Value::as_str) == Some("metric") { "m" } else { "ft" },
        "structures": []
    }));
    if let Some(format) = format.filter(|&format| source["format"] != format) {
        if format == FORMAT && source["format_version"] != CURRENT_VERSION {
            return Err(
                "Save draft 0.1 workfiles with the original .sketchy.json or .sketchy extension."
                    .into(),
            );
        }
        const SKETCHY_SCHEMA: &str =
            "https://property-hackers.github.io/sketchy/schema/0.2/sketchy.schema.json";
        const SKETCHIE_SCHEMA: &str =
            "https://property-hackers.github.io/sketchie/schema/0.2/sketchie.schema.json";
        if format == FORMAT && source["$schema"] == SKETCHY_SCHEMA {
            source["$schema"] = json!(SKETCHIE_SCHEMA);
        } else if format == LEGACY_FORMAT && source["$schema"] == SKETCHIE_SCHEMA {
            source["$schema"] = json!(SKETCHY_SCHEMA);
        }
        source["format"] = json!(format);
    }
    let (feet, _) = units(source["length_unit"].as_str().unwrap_or(""))?;
    let root_frame = Frame {
        feet,
        position: [0.0; 2],
        rotation: 0.0,
        bulge_sign: bulge_sign(source["format_version"].as_str().unwrap_or_default())?,
    };
    let mut removed = HashSet::new();
    let mut bindings = Vec::new();
    let mut presentation = source
        .get("presentation")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let old_pages = backing.map_or(&[][..], |b| array(&b.snapshot.model["pages"]));
    let previous_bindings = backing.map_or(&[][..], |b| b.snapshot.bindings.as_slice());
    let mut pending: HashMap<_, _> = page_keys.iter().zip(array(&model["pages"])).collect();

    // Keep each existing structure and frame. An inserted page becomes a
    // level in the first structure using the inverse of that frame.
    for structure in list(&mut source, "structures") {
        let frame = frame_for(root_frame, structure);
        let levels = list(structure, "levels");
        let mut kept = Vec::with_capacity(levels.len());
        for mut level in std::mem::take(levels) {
            let Some(id) = level["id"].as_str() else {
                continue;
            };
            let key = PageKey::Level(id.into());
            if let Some(page) = pending.remove(&key) {
                bindings.push((
                    key.clone(),
                    update_page(
                        &key,
                        page,
                        old_pages,
                        previous_bindings,
                        frame,
                        &mut level,
                        &mut presentation,
                        &mut removed,
                    )?,
                ));
                kept.push(level);
            } else {
                removed.insert(id.to_string());
                for name in ["spaces", "walls", "voids", "openings", "features"] {
                    removed.extend(
                        array(&level[name])
                            .iter()
                            .filter_map(|v| v["id"].as_str().map(str::to_owned)),
                    );
                }
            }
        }
        *levels = kept;
    }
    if let Some(page) = pending.remove(&PageKey::Site) {
        if !source["site"].is_object() {
            source["site"] = json!({});
        }
        bindings.push((
            PageKey::Site,
            update_page(
                &PageKey::Site,
                page,
                old_pages,
                previous_bindings,
                root_frame,
                &mut source["site"],
                &mut presentation,
                &mut removed,
            )?,
        ));
    } else if old_pages
        .iter()
        .any(|page| page_key(page) == Some(PageKey::Site))
    {
        if let Some(site) = source["site"].as_object_mut() {
            if let Some(spaces) = site.remove("spaces") {
                removed.extend(
                    array(&spaces)
                        .iter()
                        .filter_map(|space| space["id"].as_str().map(str::to_owned)),
                );
            }
        }
        if let Some(annotations) = presentation["annotations"].as_array_mut() {
            annotations.retain(|item| !item["level"].is_null() || item["kind"] == "north_arrow");
        }
    }
    if !pending.is_empty() {
        let structures = list(&mut source, "structures");
        if structures.is_empty() {
            structures.push(json!({"id": new_id(), "levels": []}));
        }
        let frame = frame_for(root_frame, &structures[0]);
        let levels = list(&mut structures[0], "levels");
        let mut sequence = levels
            .iter()
            .filter(|l| l["grade"] == "above")
            .filter_map(|l| l["sequence"].as_i64())
            .max()
            .unwrap_or(0);
        for key in &page_keys {
            let (Some(id), Some(page)) = (key.level(), pending.remove(key)) else {
                continue;
            };
            sequence = sequence.saturating_add(1);
            let name = page["name"].as_str().filter(|name| !name.is_empty());
            let mut level = json!({"id": id, "grade": "above", "sequence": sequence});
            if let Some(name) = name {
                level["name"] = json!(name);
            }
            bindings.push((
                key.clone(),
                update_page(
                    key,
                    page,
                    old_pages,
                    previous_bindings,
                    frame,
                    &mut level,
                    &mut presentation,
                    &mut removed,
                )?,
            ));
            levels.push(level);
        }
    }
    // Preserve the order in which the user sees the levels within a structure.
    let order: HashMap<_, _> = page_keys
        .iter()
        .filter_map(PageKey::level)
        .enumerate()
        .map(|(i, key)| (key, i))
        .collect();
    for structure in list(&mut source, "structures") {
        list(structure, "levels")
            .sort_by_key(|level| level["id"].as_str().and_then(|id| order.get(id)).copied());
    }
    if !presentation.as_object().is_some_and(Map::is_empty) || source.get("presentation").is_some()
    {
        source["presentation"] = presentation;
    }
    prune_references(&mut source, &mut removed);

    let address = model
        .pointer("/subjectInfo/propertyAddress")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let previous_address = backing
        .and_then(|b| b.snapshot.model.pointer("/subjectInfo/propertyAddress"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if address != previous_address {
        source["property"]["address"] = json!(address);
    }

    let namespaces = list(&mut source, "extensions_used");
    if !namespaces.iter().any(|value| value == NAMESPACE) {
        namespaces.push(json!(NAMESPACE));
    }
    let snapshot = Snapshot { model, bindings };
    let changed = backing.is_none_or(|old| {
        !old.has_workfile
            || old.snapshot.model != snapshot.model
            || projection(&old.source) != projection(&source)
    });
    if changed || fork {
        revise(&mut source, fork);
        source
            .as_object_mut()
            .expect("Sketchy document object")
            .remove("results");
    }

    let workfile = Workfile {
        projection: projection(&source),
        snapshot: snapshot.clone(),
    };
    let mut output = source.clone();
    output["extras"][WORKFILE] = json!(workfile);
    let hash = super::store::content_hash(&output)?;
    output["revision"]["content_hash"] = json!(hash);
    source["revision"]["content_hash"] = json!(hash);
    let mut bytes = serde_json::to_vec_pretty(&output).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(Prepared {
        bytes,
        backing: Backing {
            source,
            snapshot,
            has_workfile: true,
        },
        page_keys,
    })
}

fn revise(source: &mut Value, fork: bool) {
    let previous = source.get("revision").cloned().unwrap_or(Value::Null);
    let parent = previous["id"].as_str();
    let mut sequence = previous["sequence"]
        .as_u64()
        .unwrap_or(u64::from(parent.is_some()))
        .saturating_add(1);
    if fork {
        let mut origin = json!({"document": source["id"]});
        if let Some(parent) = parent {
            origin["revision"] = json!(parent);
        }
        source["derived_from"] = origin;
        source["id"] = json!(new_id());
        source
            .as_object_mut()
            .expect("Sketchy document object")
            .remove("history");
        sequence = 1;
    } else if parent.is_some() {
        list(source, "history").push(previous.clone());
    }
    source["revision"] = json!({"id": new_id(), "parents": if fork { None } else { parent }.into_iter().collect::<Vec<_>>(),
        "sequence": sequence, "created_at": crate::iso_now(), "status": "draft",
        "generator": {"name": "Sketch Rust", "version": env!("CARGO_PKG_VERSION")}});
}

fn frame_for(default: Frame, structure: &Value) -> Frame {
    Frame {
        position: structure
            .get("position")
            .and_then(|p| Point::deserialize(p).ok())
            .unwrap_or_default(),
        rotation: structure["rotation"].as_f64().unwrap_or_default(),
        ..default
    }
}

impl Frame {
    fn local(self, x: f64, y: f64) -> Point {
        let x = x / self.feet - self.position[0];
        let y = y / self.feet - self.position[1];
        let (sin, cos) = self.rotation.rem_euclid(360.0).to_radians().sin_cos();
        [x * cos + y * sin, -x * sin + y * cos]
    }

    fn local_point(self, point: &Value) -> Point {
        self.local(number(&point["x"]), number(&point["y"]))
    }
}

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or_default()
}

fn id_map(list: &Value) -> HashMap<i64, &Value> {
    array(list)
        .iter()
        .filter_map(|item| Some((item["id"].as_i64()?, item)))
        .collect()
}

struct PageEdit<'a> {
    page: &'a Value,
    old: &'a Value,
    frame: Frame,
    lines: HashMap<i64, &'a Value>,
    old_lines: HashMap<i64, &'a Value>,
    areas: HashMap<i64, &'a Value>,
    old_areas: HashMap<i64, &'a Value>,
    labels: HashMap<i64, &'a Value>,
    old_labels: HashMap<i64, &'a Value>,
    symbols: HashMap<i64, &'a Value>,
    old_symbols: HashMap<i64, &'a Value>,
    space_ids: HashMap<i64, String>,
}

#[allow(clippy::too_many_arguments)]
fn update_page(
    key: &PageKey,
    page: &Value,
    old_pages: &[Value],
    previous: &[(PageKey, Vec<Binding>)],
    frame: Frame,
    level: &mut Value,
    presentation: &mut Value,
    removed: &mut HashSet<String>,
) -> Result<Vec<Binding>> {
    let old = old_pages
        .iter()
        .find(|old| page_key(old).as_ref() == Some(key))
        .unwrap_or(&Value::Null);
    let mut edit = PageEdit {
        page,
        old,
        frame,
        lines: id_map(&page["lines"]),
        old_lines: id_map(&old["lines"]),
        areas: id_map(&page["polygons"]),
        old_areas: id_map(&old["polygons"]),
        labels: id_map(&page["labels"]),
        old_labels: id_map(&old["labels"]),
        symbols: id_map(&page["symbols"]),
        old_symbols: id_map(&old["symbols"]),
        space_ids: HashMap::new(),
    };
    let existing = previous
        .iter()
        .find(|(page, _)| page == key)
        .map_or(&[][..], |(_, b)| b.as_slice());
    for binding in existing {
        if let Entity::Space { area, .. } = binding.entity {
            if edit.areas.contains_key(&area) {
                edit.space_ids.insert(area, binding.id.clone());
            }
        }
    }
    for area in array(&page["polygons"]) {
        if let Some(id) = area["id"]
            .as_i64()
            .filter(|id| !edit.old_areas.contains_key(id))
        {
            edit.space_ids.insert(id, new_id());
        }
    }
    let mut kept = Vec::new();
    for binding in existing {
        let name = binding.entity.collection();
        let container = if name == "annotations" {
            &mut *presentation
        } else {
            &mut *level
        };
        let Some(items) = container.get_mut(name).and_then(Value::as_array_mut) else {
            continue;
        };
        let Some(index) = items.iter().position(|item| item["id"] == binding.id) else {
            continue;
        };
        let mut updated = binding.clone();
        if edit.update(&mut items[index], &mut updated.entity)? {
            kept.push(updated);
        } else {
            removed.insert(binding.id.clone());
            items.remove(index);
        }
    }
    let mut released: HashSet<_> = existing
        .iter()
        .flat_map(|binding| binding.entity.path_lines().iter().copied())
        .collect();
    for id in kept.iter().flat_map(|binding| binding.entity.path_lines()) {
        released.remove(id);
    }
    for area in array(&page["polygons"]) {
        let Some(id) = area["id"]
            .as_i64()
            .filter(|id| !edit.old_areas.contains_key(id))
        else {
            continue;
        };
        let mut item = json!({"id": edit.space_ids[&id]});
        edit.update_area(&mut item, area, &Value::Null, false)?;
        kept.push(Binding::new(
            &edit.space_ids[&id],
            Entity::Space {
                area: id,
                holes: vec![],
            },
        ));
        list(level, "spaces").push(item);
    }
    for line in array(&page["lines"]) {
        let Some(id) = line["id"]
            .as_i64()
            .filter(|id| !edit.old_lines.contains_key(id) || released.contains(id))
        else {
            continue;
        };
        let item_id = new_id();
        if key.level().is_some() {
            let mut item = json!({"id": item_id});
            edit.write_wall(&mut item, line, line, number(&line["bowAngle"]), true);
            list(level, "walls").push(item);
            kept.push(Binding::new(&item_id, Entity::Wall(vec![id])));
        } else {
            let shape = edit.path(&[id], false)?;
            list(presentation, "annotations")
                .push(json!({"id": item_id, "kind":"shape", "shape": shape}));
            kept.push(Binding::new(&item_id, Entity::Drawing(vec![id])));
        }
    }
    for label in array(&page["labels"]) {
        let Some(id) = label["id"]
            .as_i64()
            .filter(|id| !edit.old_labels.contains_key(id))
        else {
            continue;
        };
        let item_id = new_id();
        let mut item = annotation(&item_id, "text", key);
        edit.write_text(&mut item, label);
        list(presentation, "annotations").push(item);
        kept.push(Binding::new(&item_id, Entity::Text(id)));
    }
    for symbol in array(&page["symbols"]) {
        let Some(id) = symbol["id"]
            .as_i64()
            .filter(|id| !edit.old_symbols.contains_key(id))
        else {
            continue;
        };
        let item_id = new_id();
        let mut item = annotation(&item_id, "symbol", key);
        edit.write_symbol(&mut item, symbol);
        list(presentation, "annotations").push(item);
        kept.push(Binding::new(&item_id, Entity::Symbol(id)));
    }
    if number(&page["northAngle"]).rem_euclid(360.0) != number(&old["northAngle"]).rem_euclid(360.0)
        && !kept.iter().any(|b| matches!(b.entity, Entity::North))
    {
        let id = new_id();
        let mut item = annotation(&id, "north_arrow", key);
        item["position"] = json!([0.0, 0.0]);
        item["rotation"] = json!(edit.rotation(&page["northAngle"]));
        list(presentation, "annotations").push(item);
        kept.push(Binding::new(&id, Entity::North));
    }
    Ok(kept)
}

fn annotation(id: &str, kind: &str, key: &PageKey) -> Value {
    let mut value = json!({"id": id, "kind": kind});
    if let Some(level) = key.level() {
        value["level"] = json!(level);
    }
    value
}

impl PageEdit<'_> {
    fn update(&self, item: &mut Value, entity: &mut Entity) -> Result<bool> {
        match entity {
            Entity::Space { .. } | Entity::VoidWithHoles { .. } => {
                let (area, holes, is_void) = match entity {
                    Entity::Space { area, holes } => (area, holes, false),
                    Entity::VoidWithHoles { area, holes } => (area, holes, true),
                    _ => unreachable!(),
                };
                let Some(poly) = self.areas.get(area) else {
                    return Ok(false);
                };
                let old = self.old_areas.get(area).copied().unwrap_or(&Value::Null);
                self.update_area(item, poly, old, is_void)?;
                let original = array(&item["shape"]["holes"]);
                let mut rings = Vec::new();
                let mut changed = false;
                for (index, id) in holes.iter().enumerate() {
                    if let Some(ring) = self.areas.get(id) {
                        let mut raw = original.get(index).cloned().unwrap_or_else(|| json!({}));
                        if boundary_changed(
                            ring,
                            self.old_areas.get(id).copied().unwrap_or(&Value::Null),
                        ) {
                            let old_ring = self.old_areas.get(id).copied().unwrap_or(&Value::Null);
                            let mut geometry = json!({});
                            self.write_ring(&mut geometry, ring)?;
                            remap_ring_metadata(&raw, &mut geometry, ring, old_ring);
                            replace_shape(&mut raw, geometry);
                            changed = true;
                        }
                        rings.push(raw);
                    } else {
                        changed = true;
                    }
                }
                if changed {
                    item["shape"]["holes"] = json!(rings);
                    holes.retain(|id| self.areas.contains_key(id));
                }
            }
            Entity::Void(id) => {
                let Some(area) = self.areas.get(id) else {
                    return Ok(false);
                };
                self.update_area(
                    item,
                    area,
                    self.old_areas.get(id).copied().unwrap_or(&Value::Null),
                    true,
                )?;
            }
            Entity::Wall(ids) => {
                let Some(first) = ids.first().and_then(|id| self.lines.get(id)) else {
                    return Ok(false);
                };
                if ids.iter().any(|id| self.line_changed(*id)) {
                    let old = self.old_lines.get(&ids[0]).copied().unwrap_or(&Value::Null);
                    if ids.len() > 1 {
                        // A large Sketchy arc can occupy multiple native walls.
                        // Keep its ID on the first edited segment and write the
                        // remaining segments independently.
                        ids.truncate(1);
                        self.write_wall(item, first, first, number(&first["bowAngle"]), true);
                    } else {
                        self.write_wall(item, first, old, number(&first["bowAngle"]), false);
                    }
                }
            }
            Entity::OpenSpace(ids) | Entity::Drawing(ids) => {
                if ids.iter().any(|id| self.line_changed(*id)) {
                    let closed = matches!(
                        item["shape"]["type"].as_str(),
                        Some("polygon" | "circle" | "ellipse")
                    );
                    let Ok(shape) = self.path(ids, closed) else {
                        return Ok(false);
                    };
                    replace_shape(&mut item["shape"], shape);
                }
            }
            Entity::Text(id) | Entity::Feature(id) => {
                let Some(text) = self.labels.get(id) else {
                    return Ok(false);
                };
                let old = self.old_labels.get(id).copied().unwrap_or(&Value::Null);
                if item["kind"] == "text" {
                    if changed(text, old, &["x", "y", "text", "rotation"]) {
                        self.write_text(item, text);
                    }
                } else {
                    if changed(text, old, &["x", "y"]) {
                        item["position"] = json!(self.frame.local_point(text));
                    }
                    if changed(text, old, &["text"]) {
                        item["name"] = text["text"].clone();
                    }
                }
            }
            Entity::Opening(id) => {
                if !self.lines.contains_key(id) {
                    return Ok(false);
                }
            }
            Entity::Label(id) => {
                let Some(area) = self.areas.get(id) else {
                    return Ok(false);
                };
                let old = self.old_areas.get(id).copied().unwrap_or(&Value::Null);
                if changed(area, old, &["labelPosition"]) {
                    if area["labelPosition"].is_object() {
                        item["position"] = json!(self.frame.local_point(&area["labelPosition"]));
                    } else if let Some(obj) = item.as_object_mut() {
                        obj.remove("position");
                    }
                }
                if changed(area, old, &["labelRotation"]) {
                    item["rotation"] = json!(self.rotation(&area["labelRotation"]));
                }
            }
            Entity::Symbol(id) => {
                let Some(symbol) = self.symbols.get(id) else {
                    return Ok(false);
                };
                if self.old_symbols.get(id) != Some(symbol) {
                    self.write_symbol(item, symbol);
                }
            }
            Entity::North => {
                if number(&self.page["northAngle"]).rem_euclid(360.0)
                    != number(&self.old["northAngle"]).rem_euclid(360.0)
                {
                    item["rotation"] = json!(self.rotation(&self.page["northAngle"]));
                }
            }
        }
        Ok(true)
    }

    fn line_changed(&self, id: i64) -> bool {
        match self.lines.get(&id) {
            Some(line) => changed(
                line,
                self.old_lines.get(&id).copied().unwrap_or(&Value::Null),
                &["start", "end", "bowAngle", "thickness", "type"],
            ),
            None => true,
        }
    }

    fn update_area(
        &self,
        item: &mut Value,
        area: &Value,
        old: &Value,
        is_void: bool,
    ) -> Result<()> {
        if changed(area, old, &["name"]) && area["name"].as_str().is_some_and(|s| !s.is_empty()) {
            item["name"] = area["name"].clone();
        }
        if boundary_changed(area, old) {
            let shape = &mut item["shape"];
            let mut ring = json!({"type": "polygon"});
            self.write_ring(&mut ring, area)?;
            remap_ring_metadata(shape, &mut ring, area, old);
            replace_shape(shape, ring);
        }
        if is_void {
            return Ok(());
        }
        if changed(area, old, &["factorOverride"]) {
            set_optional(
                item,
                "area_factor",
                area.get("factorOverride").filter(|v| !v.is_null()).cloned(),
            );
        }
        if changed(area, old, &["measureTo"]) {
            let measured = match area["measureTo"].as_str() {
                Some("interior") => "interior_face",
                Some("exterior") => "exterior_face",
                _ => "centerline",
            };
            item["measured_to"] = json!(measured);
        }
        if changed(area, old, &["subtractTargetId", "subtractMode"]) {
            let parent = area["subtractTargetId"]
                .as_i64()
                .and_then(|id| self.space_ids.get(&id));
            if area["subtractMode"] == "always" && parent.is_some() {
                item["within"] = json!(parent);
                item["subtract"] = json!(true);
            } else if item["subtract"] == true {
                item["subtract"] = json!(false);
            }
        }
        let empty = Map::new();
        let before = old["attributes"].as_object().unwrap_or(&empty);
        let after = area["attributes"].as_object().unwrap_or(&empty);
        for (key, value) in after {
            if !key.starts_with("sketchy.") && before.get(key) != Some(value) {
                item["attributes"][key] = value.clone();
            }
        }
        if let Some(attributes) = item.get_mut("attributes").and_then(Value::as_object_mut) {
            for key in before
                .keys()
                .filter(|key| !key.starts_with("sketchy.") && !after.contains_key(*key))
            {
                attributes.remove(key);
            }
        }
        Ok(())
    }

    fn write_ring(&self, ring: &mut Value, area: &Value) -> Result<()> {
        let points = array(&area["points"]);
        if points.len() < 3 {
            return Err("a closed Sketchy area needs at least three points".into());
        }
        ring["points"] = json!(points
            .iter()
            .map(|point| self.frame.local_point(point))
            .collect::<Vec<_>>());
        let edges = array(&area["edges"]);
        let bulges = (0..points.len()).map(|i| {
            edges
                .get(i)
                .map_or(0.0, |e| self.bulge(number(&e["bowAngle"])))
        });
        write_bulges(ring, bulges.collect());
        Ok(())
    }

    fn write_wall(&self, item: &mut Value, line: &Value, old: &Value, bow: f64, all: bool) {
        if all || changed(line, old, &["start", "end", "bowAngle"]) {
            item["start"] = json!(self.frame.local_point(&line["start"]));
            item["end"] = json!(self.frame.local_point(&line["end"]));
            set_optional(item, "bulge", (bow != 0.0).then(|| json!(self.bulge(bow))));
        }
        if all || changed(line, old, &["thickness"]) {
            set_optional(
                item,
                "thickness",
                line["thickness"]
                    .as_f64()
                    .map(|width| json!(width / 12.0 / self.frame.feet)),
            );
        }
        if all || changed(line, old, &["type"]) {
            item["kind"] = line["type"].clone();
        }
    }

    fn path(&self, ids: &[i64], closed: bool) -> Result<Value> {
        let lines = ids
            .iter()
            .map(|id| self.lines.get(id).copied().ok_or("path was interrupted"))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let first = lines.first().ok_or("empty path")?;
        let reverse = lines.get(1).is_some_and(|next| {
            !near_point(&first["end"], &next["start"]) && !near_point(&first["end"], &next["end"])
        });
        let first_point = &first[if reverse { "end" } else { "start" }];
        let mut from = first_point;
        let mut points = vec![self.frame.local_point(from)];
        let mut bulges = Vec::new();
        for line in lines {
            let (to, sign) = if near_point(from, &line["start"]) {
                (&line["end"], 1.0)
            } else if near_point(from, &line["end"]) {
                (&line["start"], -1.0)
            } else {
                return Err("path was interrupted".into());
            };
            points.push(self.frame.local_point(to));
            bulges.push(sign * self.bulge(number(&line["bowAngle"])));
            from = to;
        }
        if closed {
            if points.len() < 4 || !near_point(from, first_point) {
                return Err("outline is open".into());
            }
            points.pop();
        }
        let mut shape =
            json!({"type": if closed { "polygon" } else { "polyline" }, "points": points});
        write_bulges(&mut shape, bulges);
        Ok(shape)
    }

    fn rotation(&self, angle: &Value) -> f64 {
        (-number(angle) - self.frame.rotation).rem_euclid(360.0)
    }

    fn bulge(&self, bow: f64) -> f64 {
        self.frame.bulge_sign * (bow.to_radians() / 4.0).tan()
    }

    fn write_text(&self, item: &mut Value, text: &Value) {
        item["position"] = json!(self.frame.local_point(text));
        item["text"] = text["text"].clone();
        item["rotation"] = json!(self.rotation(&text["rotation"]));
    }

    fn write_symbol(&self, item: &mut Value, symbol: &Value) {
        item["position"] = json!(self.frame.local_point(symbol));
        item["rotation"] = json!(self.rotation(&symbol["rotation"]));
        item["ref"] = json!({"scheme": NAMESPACE, "name": symbol["symbolType"]});
        item["scale"] = json!([symbol["scale"], symbol["scale"]]);
        item["mirror"] = json!(if symbol["flipH"] == true { "x" } else { "none" });
    }
}

fn changed(current: &Value, old: &Value, keys: &[&str]) -> bool {
    keys.iter().any(|key| current[*key] != old[*key])
}

fn boundary_changed(current: &Value, old: &Value) -> bool {
    current["points"] != old["points"]
        || array(&current["edges"])
            .iter()
            .map(|e| number(&e["bowAngle"]))
            .ne(array(&old["edges"]).iter().map(|e| number(&e["bowAngle"])))
}

fn near_point(a: &Value, b: &Value) -> bool {
    (number(&a["x"]) - number(&b["x"])).hypot(number(&a["y"]) - number(&b["y"])) < 1e-7
}

fn write_bulges(path: &mut Value, bulges: Vec<f64>) {
    set_optional(
        path,
        "bulges",
        bulges.iter().any(|b| *b != 0.0).then(|| json!(bulges)),
    );
}

fn remap_ring_metadata(source: &Value, output: &mut Value, current: &Value, old: &Value) {
    let before = array(&old["edges"]);
    let after = array(&current["edges"]);
    for (key, default) in [
        ("edges", json!("physical")),
        ("edge_measured_to", Value::Null),
    ] {
        let Some(values) = source[key].as_array() else {
            continue;
        };
        if values.len() != before.len() {
            continue;
        }
        let fields: Vec<_> = after
            .iter()
            .enumerate()
            .map(|(i, edge)| {
                let same_endpoints = |a: usize, b: usize| {
                    let current_points = array(&current["points"]);
                    let previous_points = array(&old["points"]);
                    if current_points.len() != after.len() || previous_points.len() != before.len()
                    {
                        return false;
                    }
                    let a1 = &current_points[a];
                    let a2 = &current_points[(a + 1) % current_points.len()];
                    let b1 = &previous_points[b];
                    let b2 = &previous_points[(b + 1) % previous_points.len()];
                    (near_point(a1, b1) && near_point(a2, b2))
                        || (near_point(a1, b2) && near_point(a2, b1))
                };
                let previous = edge["lineId"]
                    .as_i64()
                    .and_then(|id| {
                        before
                            .iter()
                            .position(|prior| prior["lineId"].as_i64() == Some(id))
                    })
                    .or_else(|| (0..before.len()).find(|&j| same_endpoints(i, j)))
                    .or_else(|| (after.len() == before.len()).then_some(i));
                previous
                    .and_then(|i| values.get(i))
                    .cloned()
                    .unwrap_or_else(|| default.clone())
            })
            .collect();
        output[key] = json!(fields);
    }
}

fn replace_shape(shape: &mut Value, geometry: Value) {
    let mut result = shape.as_object().cloned().unwrap_or_default();
    for key in [
        "type",
        "points",
        "bulges",
        "center",
        "radius",
        "radius_x",
        "radius_y",
        "rotation",
        "start_angle",
        "end_angle",
        "edges",
        "edge_measured_to",
    ] {
        result.remove(key);
    }
    result.extend(geometry.as_object().expect("shape object").clone());
    *shape = Value::Object(result);
}

fn set_optional(item: &mut Value, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        item[key] = value;
    } else if let Some(item) = item.as_object_mut() {
        item.remove(key);
    }
}

fn prune_references(source: &mut Value, removed: &mut HashSet<String>) {
    if removed.is_empty() {
        return;
    }
    let references =
        |v: &Value, gone: &HashSet<String>| v.as_str().is_some_and(|id| gone.contains(id));
    for structure in list(source, "structures") {
        for level in list(structure, "levels") {
            if let Some(openings) = level.get_mut("openings").and_then(Value::as_array_mut) {
                openings.retain(|opening| {
                    if references(&opening["space"], removed)
                        || references(&opening["wall"], removed)
                    {
                        if let Some(id) = opening["id"].as_str() {
                            removed.insert(id.to_owned());
                        }
                        false
                    } else {
                        true
                    }
                });
            }
            if let Some(openings) = level.get_mut("openings").and_then(Value::as_array_mut) {
                for opening in openings {
                    if references(&opening["swing_into"], removed) {
                        set_optional(opening, "swing_into", None);
                    }
                }
            }
            prune_subdivisions(level, removed);
            if let Some(walls) = level.get_mut("walls").and_then(Value::as_array_mut) {
                for wall in walls {
                    if let Some(bounds) = wall.get_mut("bounds").and_then(Value::as_array_mut) {
                        bounds.retain(|bound| !references(&bound["space"], removed));
                    }
                }
            }
            if let Some(features) = level.get_mut("features").and_then(Value::as_array_mut) {
                for feature in features {
                    for key in ["connects_to", "counted_on", "well"] {
                        if references(&feature[key], removed) {
                            set_optional(feature, key, None);
                        }
                    }
                }
            }
        }
    }
    if let Some(site) = source.get_mut("site") {
        prune_subdivisions(site, removed);
    }
    let Some(presentation) = source.get_mut("presentation") else {
        return;
    };
    if let Some(items) = presentation
        .get_mut("annotations")
        .and_then(Value::as_array_mut)
    {
        items.retain(|item| {
            let keep = !references(&item["level"], removed)
                && !references(&item["target"], removed)
                && !references(&item["target"]["space"], removed)
                && !references(&item["target"]["wall"], removed)
                && !references(&item["host"]["wall"], removed);
            if !keep {
                if let Some(id) = item["id"].as_str() {
                    removed.insert(id.to_owned());
                }
            }
            keep
        });
    }
    if let Some(underlays) = presentation
        .get_mut("underlays")
        .and_then(Value::as_array_mut)
    {
        underlays.retain(|item| !references(&item["level"], removed));
    }
    for key in ["element_styles", "element_layers"] {
        if let Some(map) = presentation.get_mut(key).and_then(Value::as_object_mut) {
            map.retain(|id, _| !removed.contains(id));
        }
    }
    if let Some(pages) = presentation.get_mut("pages").and_then(Value::as_array_mut) {
        for page in pages {
            if let Some(views) = page.get_mut("views").and_then(Value::as_array_mut) {
                for view in views.iter_mut() {
                    if let Some(levels) = view.get_mut("levels").and_then(Value::as_array_mut) {
                        levels.retain(|id| !references(id, removed));
                    }
                }
                views.retain(|view| !view["levels"].as_array().is_some_and(Vec::is_empty));
            }
        }
    }
}

fn prune_subdivisions(page: &mut Value, removed: &HashSet<String>) {
    if let Some(spaces) = page.get_mut("spaces").and_then(Value::as_array_mut) {
        for space in spaces {
            if space["within"]
                .as_str()
                .is_some_and(|parent| removed.contains(parent))
            {
                set_optional(space, "within", None);
                set_optional(space, "subtract", None);
            }
        }
    }
}
