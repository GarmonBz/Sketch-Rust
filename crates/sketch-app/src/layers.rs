//! Layers (screen 10) — the flat `state.layers` list plus the tree math the
//! Layers tab is built on (`flattenLayerOrder`, `layerAncestry`,
//! `descendantLayerIds`, `uniqueLayerName`, `moveLayer`,
//! `updateLayersPanel`'s row filter — sketch-app.js:1115 / ui-manager.js:2933).
//! Dedicated area layers carry `areaPolyId`; groups fold their children away.

use serde::{Deserialize, Serialize};

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub id: String,
    pub name: String,
    #[serde(default = "yes")]
    pub visible: bool,
    #[serde(default)]
    pub locked: bool,
    /// `isGroup` - a folder row, not a paint target.
    #[serde(default)]
    pub is_group: bool,
    #[serde(default)]
    pub collapsed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// `areaPolyId` - the dedicated layer's closed area (sketch-app.js:1318).
    /// Absent until `assignAreaToDedicatedLayer` stamps it, and omitted from
    /// the save like the original's `undefined`. Ids may arrive as numbers
    /// (older workfiles; `String(layer.areaPolyId)` tolerates both).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "string_or_number"
    )]
    pub area_poly_id: Option<String>,
}

/// `String(value)` for an id field that arrives as a string or a number.
fn string_or_number<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Any {
        Text(String),
        Number(i64),
    }
    Ok(match Any::deserialize(d)? {
        Any::Text(s) => Some(s),
        Any::Number(n) => Some(n.to_string()),
    })
}

impl Layer {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            visible: true,
            locked: false,
            is_group: false,
            collapsed: false,
            parent_id: None,
            area_poly_id: None,
        }
    }
}

/// One row of the Layers tree (`updateLayersPanel`).
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The row's layer id.
    pub id: String,
    /// The row label (the layer's name).
    pub name: String,
    /// `--depth`: the row's ancestry length minus one.
    pub depth: usize,
    pub is_group: bool,
    pub visible: bool,
    pub locked: bool,
    pub collapsed: bool,
    pub active: bool,
}

/// A drop position for `moveLayer` (S10-031): before / after / inside a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragPos {
    Before,
    After,
    Inside,
}

/// `getAreaLayersForCurrentPage`'s filter inputs (sketch-app.js:1211): the
/// live page's polygon ids and the layers those polygons point at.
#[derive(Debug, Clone, Default)]
pub struct AreaLayerFilter {
    pub poly_ids: std::collections::HashSet<String>,
    pub layer_ids: std::collections::HashSet<String>,
}

impl AreaLayerFilter {
    /// A layer shows when its `areaPolyId` names a live polygon on the page
    /// and that polygon still points back at the layer.
    pub fn keeps(&self, layer: &Layer) -> bool {
        layer
            .area_poly_id
            .as_deref()
            .is_some_and(|aid| self.poly_ids.contains(aid))
            && self.layer_ids.contains(&layer.id)
    }
}

#[derive(Debug, Clone)]
pub struct Layers {
    pub items: Vec<Layer>,
    /// `state.activeLayerId`. The original defaults to the first non-group
    /// layer; the app boots with `layer-1` from `ensure_default`.
    pub active: String,
}

impl Default for Layers {
    fn default() -> Self {
        Layers {
            items: vec![Layer::new("layer-1", "Layer 1")],
            active: "layer-1".to_string(),
        }
    }
}

/// An incoming save's layer entry: ids may arrive as strings (the original)
/// or numbers (older workfiles).
fn raw_id(raw: &serde_json::Value, fallback: String) -> String {
    raw.get("id")
        .and_then(|v| {
            v.as_str()
                .map(str::to_string)
                .or_else(|| v.as_i64().map(|n| n.to_string()))
        })
        .unwrap_or(fallback)
}

impl Layers {
    /// `ensureLayers(layers, activeLayerId)`: restore the layer list; empty
    /// or absent falls back to `layer-1`, ids fall back to positional names,
    /// and the active id falls back to the first non-group layer.
    pub fn ensure(
        &mut self,
        layers: Option<&serde_json::Value>,
        active: Option<&serde_json::Value>,
    ) {
        let mut items: Vec<Layer> = vec![];
        if let Some(arr) = layers.and_then(|v| v.as_array()).filter(|a| !a.is_empty()) {
            for (index, raw) in arr.iter().enumerate() {
                items.push(Layer {
                    id: raw_id(raw, format!("layer-{}", index + 1)),
                    name: raw
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                        .unwrap_or_else(|| format!("Layer {}", index + 1)),
                    visible: raw
                        .get("visible")
                        .map(|v| v.as_bool() != Some(false))
                        .unwrap_or(true),
                    locked: raw.get("locked").and_then(|v| v.as_bool()).unwrap_or(false),
                    is_group: raw
                        .get("isGroup")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    collapsed: raw
                        .get("collapsed")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    parent_id: raw
                        .get("parentId")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    // Earlier workfiles stored the area id as a number.
                    area_poly_id: raw.get("areaPolyId").and_then(|v| {
                        v.as_str()
                            .map(str::to_string)
                            .or_else(|| v.as_i64().map(|n| n.to_string()))
                        // `areaPolyId: null`/absent stays None.
                    }),
                });
            }
        }
        if items.is_empty() {
            items.push(Layer::new("layer-1", "Layer 1"));
        }
        self.items = items;
        let fallback = self.default_layer_id();
        self.active = active
            .and_then(|v| {
                v.as_str()
                    .map(str::to_string)
                    .or_else(|| v.as_i64().map(|n| n.to_string()))
            })
            .filter(|id| self.items.iter().any(|l| l.id == id.clone()))
            .unwrap_or(fallback);
    }

    /// `getDefaultLayerId`: the first non-group layer in paint order, else
    /// the first row (sketch-app.js:1161).
    pub fn default_layer_id(&self) -> String {
        self.flatten_layer_order()
            .into_iter()
            .find(|l| !l.is_group)
            .map(|l| l.id.clone())
            .or_else(|| self.items.first().map(|l| l.id.clone()))
            .unwrap_or_else(|| "layer-1".to_string())
    }

    /// `createLayerRaw(name, isGroup)`: pushes a layer; the id draws from
    /// the caller's shared counter (the original's `genId`), never colliding
    /// with a live one.
    pub fn create_layer_raw(&mut self, id: String, name: String, is_group: bool) -> String {
        self.items.push(Layer {
            id: id.clone(),
            name,
            visible: true,
            locked: false,
            is_group,
            collapsed: false,
            parent_id: None,
            area_poly_id: None,
        });
        id
    }

    /// `removeLayer`: drops the layer and its whole descendant branch. The
    /// entity reassignment onto the default layer is the caller's.
    pub fn remove(&mut self, id: &str) {
        let branch: std::collections::HashSet<String> =
            self.descendant_layer_ids(id).into_iter().collect();
        self.items.retain(|l| !branch.contains(&l.id));
        let alive_once = branch.contains(&self.active);
        let alive = self.items.iter().any(|l| l.id == self.active);
        if alive_once && !alive {
            self.active = self.default_layer_id();
        }
    }

    pub fn set_visible(&mut self, id: &str, v: bool) {
        if let Some(l) = self.items.iter_mut().find(|l| l.id == id) {
            l.visible = v;
        }
    }

    pub fn set_locked(&mut self, id: &str, v: bool) {
        if let Some(l) = self.items.iter_mut().find(|l| l.id == id) {
            l.locked = v;
        }
    }

    pub fn set_active(&mut self, id: String) {
        self.active = id;
    }

    pub fn rename(&mut self, id: &str, name: String) {
        if let Some(l) = self.items.iter_mut().find(|l| l.id == id) {
            l.name = name;
        }
    }

    /// `setLayerCollapsed` (S10-029).
    pub fn set_collapsed(&mut self, id: &str, v: bool) {
        if let Some(l) = self.items.iter_mut().find(|l| l.id == id) {
            l.collapsed = v;
        }
    }

    /// The layer, if any.
    pub fn layer(&self, id: &str) -> Option<&Layer> {
        self.items.iter().find(|l| l.id == id)
    }

    /// `flattenLayerOrder`: parents before children, DFS over `parentId`
    /// from the roots, with the saved list's order as the tie-break inside
    /// each group (a visited set guards against a corrupt cycle).
    pub fn flatten_layer_order(&self) -> Vec<&Layer> {
        let ids: std::collections::HashSet<&str> =
            self.items.iter().map(|l| l.id.as_str()).collect();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut out: Vec<&Layer> = vec![];
        fn visit<'a>(
            parent: Option<&str>,
            layers: &'a [Layer],
            seen: &mut std::collections::HashSet<String>,
            out: &mut Vec<&'a Layer>,
        ) {
            for layer in layers.iter().filter(|l| l.parent_id.as_deref() == parent) {
                if !seen.insert(layer.id.clone()) {
                    continue;
                }
                out.push(layer);
                visit(Some(&layer.id), layers, seen, out);
            }
        }
        // Only roots start the walk; the closure above re-enters children.
        for layer in self
            .items
            .iter()
            .filter(|l| l.parent_id.as_deref().is_none_or(|p| !ids.contains(p)))
        {
            if !seen.insert(layer.id.clone()) {
                continue;
            }
            out.push(layer);
            visit(Some(&layer.id), &self.items, &mut seen, &mut out);
        }
        // Orphans of a cycle survive as leftovers in saved order.
        for layer in &self.items {
            if seen.insert(layer.id.clone()) {
                out.push(layer);
            }
        }
        out
    }

    /// `layerAncestry`: root first, self last.
    pub fn layer_ancestry(&self, id: &str) -> Vec<&Layer> {
        let mut chain: Vec<&Layer> = vec![];
        let mut current = self.layer(id);
        while let Some(l) = current {
            chain.insert(0, l);
            current = l.parent_id.as_deref().and_then(|p| self.layer(p));
        }
        chain
    }

    /// `descendantLayerIds`: self first, then children's branches.
    pub fn descendant_layer_ids(&self, id: &str) -> Vec<String> {
        let mut ids = vec![id.to_string()];
        for l in &self.items {
            if l.parent_id.as_deref() == Some(id) {
                ids.extend(self.descendant_layer_ids(&l.id));
            }
        }
        ids
    }

    /// `uniqueLayerName`: `"name"`, then `"name 2"`, `"name 3"`, ...
    pub fn unique_layer_name(&self, base: &str) -> String {
        if !self.items.iter().any(|l| l.name == base) {
            return base.to_string();
        }
        let mut index = 2;
        while self
            .items
            .iter()
            .any(|l| l.name == format!("{} {}", base, index))
        {
            index += 1;
        }
        format!("{} {}", base, index)
    }

    /// `layerAncestry` visibility: every layer in the chain is visible
    /// (sketch-app.js:1230).
    pub fn layer_visible(&self, id: &str) -> bool {
        self.layer_ancestry(id).iter().all(|l| l.visible)
    }

    /// `isLayerLocked`: any layer in the chain is locked (sketch-app.js:1235).
    pub fn layer_locked(&self, id: &str) -> bool {
        self.layer_ancestry(id).iter().any(|l| l.locked)
    }

    /// `updateLayersPanel`'s row build over `layers.filter(keep)` in paint
    /// order: rows under a collapsed group ancestor fold away, and each row
    /// keeps its unfiltered ancestry depth (`--depth`).
    pub fn visible_rows(&self, keep: impl Fn(&Layer) -> bool) -> Vec<Row> {
        let mut rows = vec![];
        for layer in self.flatten_layer_order() {
            if !keep(layer) {
                continue;
            }
            let chain = self.layer_ancestry(&layer.id);
            let hidden_by_group = chain[..chain.len().saturating_sub(1)]
                .iter()
                .any(|anc| anc.is_group && anc.collapsed);
            if hidden_by_group {
                continue;
            }
            rows.push(Row {
                id: layer.id.clone(),
                name: layer.name.clone(),
                depth: chain.len().saturating_sub(1),
                is_group: layer.is_group,
                visible: layer.visible,
                locked: layer.locked,
                collapsed: layer.collapsed,
                active: layer.id == self.active,
            });
        }
        rows
    }

    /// `moveLayer(id, targetId, position)` (S10-031): before / after / into
    /// a group. Refused (false) when either id is unknown, identical, or
    /// the target sits inside the moving branch. The `Move Layer` undo push
    /// is the caller's.
    pub fn move_layer(&mut self, id: &str, target: &str, position: DragPos) -> bool {
        if id == target || self.layer(id).is_none() || self.layer(target).is_none() {
            return false;
        }
        let moving: std::collections::HashSet<String> =
            self.descendant_layer_ids(id).into_iter().collect();
        if moving.contains(target) {
            return false; // cannot move a layer inside itself
        }
        let target_children: std::collections::HashSet<String> =
            self.descendant_layer_ids(target).into_iter().collect();
        // Strip the branch out and re-insert it.
        let branch: Vec<Layer> = {
            let mut others = std::mem::take(&mut self.items);
            let taken: Vec<Layer> = others
                .iter()
                .filter(|l| moving.contains(&l.id))
                .cloned()
                .collect();
            others.retain(|l| !moving.contains(&l.id));
            self.items = others;
            taken
        };
        let moving_layer = branch.iter().find(|l| l.id == id).cloned().expect("moving");
        let insert_at = self
            .items
            .iter()
            .position(|l| l.id == target)
            .expect("target survived the strip");
        match position {
            DragPos::Inside => {
                if let Some(t) = self.items.iter_mut().find(|l| l.id == target) {
                    t.collapsed = false;
                }
                let mut layer = moving_layer;
                layer.parent_id = Some(target.to_string());
                self.items.insert(insert_at + 1, layer);
            }
            DragPos::Before => self.items.insert(insert_at, moving_layer),
            DragPos::After => {
                let mut at = insert_at + 1;
                while at < self.items.len() && target_children.contains(&self.items[at].id) {
                    at += 1;
                }
                self.items.insert(at, moving_layer);
            }
        }
        true
    }
}

/// `isEntityVisible`/`isEntityLocked` precomputed for a frame: the entity
/// ids (walls and areas share the one id space) sitting on layers whose
/// ancestry hides or locks them. The canvas render skips hidden ids; the
/// hit tests refuse both.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LayerGates {
    pub hidden: std::collections::HashSet<i64>,
    pub locked: std::collections::HashSet<i64>,
}

impl LayerGates {
    /// The id paints (its layer chain is visible).
    pub fn shows(&self, id: i64) -> bool {
        !self.hidden.contains(&id)
    }

    /// The id can be selected or erased (visible and unlocked).
    pub fn selectable(&self, id: i64) -> bool {
        !self.hidden.contains(&id) && !self.locked.contains(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [layer-1, Areas grp, Room A (child), Room B (child), Kitchen] — the
    /// saved order matches the app's push order (groups at the top then
    /// their children). Rooms A/B carry `areaPolyId`: they are the dedicated
    /// area layers the tab lists; layer-1/Kitchen are legacy defaults.
    fn tree() -> Layers {
        let mut l = Layers::default();
        let mut grp = Layer::new("grp", "Areas");
        grp.is_group = true;
        let mut room_a = Layer::new("room-a", "Room A");
        room_a.parent_id = Some("grp".to_string());
        room_a.area_poly_id = Some("1".to_string());
        let mut room_b = Layer::new("room-b", "Room B");
        room_b.parent_id = Some("grp".to_string());
        room_b.area_poly_id = Some("2".to_string());
        l.items = vec![
            Layer::new("layer-1", "Layer 1"),
            grp,
            room_a,
            room_b,
            Layer::new("kitchen", "Kitchen"),
        ];
        l.active = "kitchen".to_string();
        l
    }

    #[test]
    fn flatten_orders_parents_before_children() {
        let l = tree();
        let ids: Vec<&str> = l
            .flatten_layer_order()
            .iter()
            .map(|x| x.id.as_str())
            .collect();
        // The DFS walks roots in list order: layer-1, then the group and
        // its children, then Kitchen.
        assert_eq!(ids, vec!["layer-1", "grp", "room-a", "room-b", "kitchen"]);
        // A saved cycle must not hang: the dupes survive as leftovers.
        let mut odd = l.clone();
        let mut self_loop = Layer::new("weird", "Weird");
        self_loop.parent_id = Some("weird".to_string());
        odd.items.push(self_loop);
        assert_eq!(odd.flatten_layer_order().len(), 6);
    }

    #[test]
    fn visible_rows_keep_depth_and_fold_behind_a_collapsed_group() {
        let mut l = tree();
        // Only dedicated area layers land in the panel
        // (getAreaLayersForCurrentPage): the legacy default rows stay out
        // even while their depth math still counts them.
        let keep = |x: &Layer| x.area_poly_id.is_some();
        let rows = l.visible_rows(keep);
        assert_eq!(
            rows.iter()
                .map(|r| (r.id.as_str(), r.depth))
                .collect::<Vec<_>>(),
            vec![("room-a", 1), ("room-b", 1)]
        );
        // A group can even surface itself: an areaPolyId-bearing group row
        // shows the caret branch the original reserves for `layer.isGroup`.
        let mut l2 = tree();
        l2.items[1].area_poly_id = Some("9".to_string());
        let rows = l2.visible_rows(|x| x.area_poly_id.is_some());
        let grp_row = rows.iter().find(|r| r.id == "grp").unwrap();
        assert!(grp_row.is_group && grp_row.depth == 0);
        // Collapsing the group folds its children out (S10-029).
        l.set_collapsed("grp", true);
        assert!(l.visible_rows(keep).is_empty());
        // And the empty panel shows the original's empty text (the dock's).
        l.items.clear();
        l.items.push(Layer::new("layer-1", "Layer 1"));
        assert!(l.visible_rows(keep).is_empty());
    }

    #[test]
    fn move_layer_ports_before_after_and_inside() {
        let mut l = tree();
        // Before: Kitchen slides in front of grp.
        assert!(l.move_layer("kitchen", "grp", DragPos::Before));
        assert_eq!(
            l.items.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(),
            vec!["layer-1", "kitchen", "grp", "room-a", "room-b"]
        );
        assert!(l.layer("kitchen").unwrap().parent_id.is_none());
        // Inside: the layer becomes the group's first child, group unfolded.
        l.set_collapsed("grp", true);
        assert!(l.move_layer("kitchen", "grp", DragPos::Inside));
        assert_eq!(
            l.layer("kitchen").unwrap().parent_id.as_deref(),
            Some("grp")
        );
        assert!(!l.layer("grp").unwrap().collapsed);
        assert_eq!(l.items[2].id, "kitchen");
        // After skips past the target's whole child block.
        assert!(l.move_layer("kitchen", "grp", DragPos::After));
        assert_eq!(
            l.items.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(),
            vec!["layer-1", "grp", "room-a", "room-b", "kitchen"]
        );
        // A move into its own branch is refused.
        assert!(!l.move_layer("grp", "room-a", DragPos::Inside));
    }

    #[test]
    fn unique_layer_name_goes_name_two_name_three() {
        let mut l = tree();
        assert_eq!(l.unique_layer_name("LIV"), "LIV");
        l.items.push(Layer::new("liv-1", "LIV"));
        assert_eq!(l.unique_layer_name("LIV"), "LIV 2");
        l.items.push(Layer::new("liv-2", "LIV 2"));
        assert_eq!(l.unique_layer_name("LIV"), "LIV 3");
    }

    #[test]
    fn area_poly_ids_round_trip_through_the_save_keys() {
        let mut l = tree();
        l.items[2].area_poly_id = Some("3".to_string());
        let named = serde_json::to_value(&l.items[2]).unwrap();
        assert_eq!(named["areaPolyId"], "3");
        // Undefined (unassigned) is omitted from the save, like the original.
        let unnamed = serde_json::to_value(&l.items[4]).unwrap();
        assert!(unnamed.get("areaPolyId").is_none());
        // A cycled old file may carry the area id as a number.
        let raw = serde_json::json!({"id": "room-b", "name": "Room B", "areaPolyId": 7});
        let layer: Layer = serde_json::from_value(raw).unwrap();
        assert_eq!(layer.area_poly_id.as_deref(), Some("7"));
    }

    #[test]
    fn layer_visibility_and_lock_walk_the_ancestry() {
        let mut l = tree();
        // A group's eye hides its children (isLayerVisible walks the chain).
        assert!(l.layer_visible("room-a"));
        l.set_visible("grp", false);
        assert!(!l.layer_visible("room-a"));
        assert!(l.layer_visible("kitchen"));
        // One locked ancestor locks the branch.
        assert!(!l.layer_locked("room-a"));
        l.set_locked("grp", true);
        assert!(l.layer_locked("room-a"));
        assert!(!l.layer_locked("kitchen"));
    }

    #[test]
    fn remove_drops_the_branch_and_falls_back_to_the_default_layer() {
        let mut l = tree();
        l.remove("grp");
        // The group and its children all go; Kitchen is the default now.
        assert!(l.layer("grp").is_none() && l.layer("room-a").is_none());
        assert_eq!(l.default_layer_id(), "layer-1");
        l.remove("layer-1");
        assert_eq!(l.default_layer_id(), "kitchen");
        assert_eq!(l.active, "kitchen");
    }
}
