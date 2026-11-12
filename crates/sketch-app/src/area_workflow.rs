//! Optional region classification, ported from p8-optional-region-classification.js.
use std::collections::{BTreeMap, HashSet};

use iced::widget::{
    button, checkbox, column, container, mouse_area, opaque, pick_list, row, scrollable, text,
    text_input, Space,
};
use iced::{Element, Length, Point, Task};
use sketch_core::geometry::{AreaCode, Pt};
use sketch_core::model::AutoPost;

use crate::{
    define_area::{self, Region},
    dock,
    theme::*,
    view_state::Mode,
    App, Message,
};

#[derive(Debug, Clone)]
pub enum Msg {
    Activate,
    NewArea,
    FindUndefined,
    Arm(String),
    Group(String),
    Advanced(bool),
    Redefine,
    Clone,
    Reopen,
    Detect,
    TypeMenu(String),
    CloseMenu,
    AssignSelected(String),
    SelectUsing(String),
    EditType(String),
    NewType,
    DeleteType(String),
    ConfirmDelete(String),
    Open(i64),
    Cancel,
    Search(String),
    ChooseGroup(String),
    Code(String),
    Name(String),
    Labels,
    Post(Post, bool),
    Suffix(String),
    Repeat(bool),
    Submit,
    DefaultPost(Post, bool),
    DefaultSuffix(String),
    DefaultPlacement(String),
    Focus(bool),
    Leave,
}

#[derive(Debug, Clone, Copy)]
pub enum Post {
    Code,
    Name,
    Dims,
    Calcs,
    Base,
    Factor,
}

impl Post {
    fn set(self, p: &mut AutoPost, value: bool) {
        match self {
            Self::Code => p.code = value,
            Self::Name => p.name = value,
            Self::Dims => p.dims = value,
            Self::Calcs => p.calcs = value,
            Self::Base => p.base = value,
            Self::Factor => p.factor = value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chooser {
    pub region: Region,
    pub page: usize,
    pub previous_selection: crate::selection::Selection,
    pub search: String,
    pub group: String,
    pub code: Option<String>,
    pub name: String,
    pub post: AutoPost,
    pub labels: bool,
    pub repeat: bool,
    pub at: Point,
    pub existing: bool,
    pub area: f64,
    pub extra_code: Option<AreaCode>,
}

#[derive(Debug, Default)]
pub struct State {
    pub armed: Option<String>,
    pub collapsed: HashSet<String>,
    pub usage: BTreeMap<String, usize>,
    pub status: String,
    pub advanced: bool,
    pub find_undefined: bool,
    pub hover: Option<Region>,
    pub chooser: Option<Chooser>,
    pub menu: Option<String>,
    pub menu_at: Point,
    pub remembered: Option<String>,
    pub repeat: bool,
    pub previous_draw: Option<Mode>,
}

impl Chooser {
    pub fn codes(&self, codes: &[AreaCode]) -> Vec<AreaCode> {
        let mut result = codes.to_vec();
        if let Some(c) = &self.extra_code {
            result.push(c.clone());
        }
        result
    }
    pub fn matches(&self, c: &AreaCode) -> bool {
        let term = self.search.trim().to_lowercase();
        if term.is_empty() {
            c.group == self.group
        } else {
            format!(
                "{} {} {} {}",
                c.code,
                c.name,
                c.description,
                dock::palette_group_label(&c.group)
            )
            .to_lowercase()
            .contains(&term)
        }
    }
}

impl App {
    pub(super) fn workflow_refresh(&mut self) {
        self.dock.area_codes = self.area_codes.clone();
        self.workflow.usage.clear();
        for p in &self.drawing.polygons {
            *self.workflow.usage.entry(p.kind.clone()).or_default() += 1;
        }
        if self
            .workflow
            .armed
            .as_ref()
            .is_some_and(|c| !self.area_codes.iter().any(|a| &a.code == c))
        {
            self.workflow.armed = None;
        }
        let stale = self.workflow.chooser.as_ref().is_some_and(|c| {
            c.page != self.pages.current
                || match &c.region {
                    Region::Area(id) => {
                        !self.drawing.polygons.iter().any(|p| p.id == *id)
                            || !self.layer_gates().selectable(*id)
                    }
                    Region::Face(_) => false,
                }
        });
        if stale {
            self.workflow_close();
        }
    }

    pub(super) fn workflow_close(&mut self) {
        if let Some(c) = self.workflow.chooser.take() {
            if c.page != self.pages.current {
                return;
            }
            self.selection = c.previous_selection;
            let ids: HashSet<_> = self.drawing.polygons.iter().map(|p| p.id).collect();
            self.selection.polys.retain(|id| ids.contains(id));
            self.sync_edit_tools();
        }
    }

    fn workflow_activate(&mut self) -> Task<Message> {
        self.workflow_close();
        self.workflow.advanced = false;
        if self.view.mode.is_drawing() {
            self.workflow.previous_draw = Some(self.view.mode);
        }
        let task = self.set_mode(Mode::DefineArea);
        self.dock.update(dock::DockMsg::Show(dock::DockTab::Areas));
        self.workflow.status = "Click inside a closed outline. Escape returns to drawing.".into();
        self.sync_canvas();
        task
    }

    pub(super) fn workflow_escape(&mut self) -> Task<Message> {
        if self.workflow.menu.take().is_some() {
            return Task::none();
        }
        if self.workflow.advanced {
            self.workflow.advanced = false;
            return Task::none();
        }
        self.workflow_close();
        self.workflow.armed = None;
        self.workflow.hover = None;
        self.workflow.status = "Drawing mode. Area definition remains optional.".into();
        self.set_mode(self.workflow.previous_draw.unwrap_or(Mode::DrawExterior))
    }

    pub(super) fn workflow_hover(&mut self, world: Pt) {
        self.workflow.hover =
            if self.view.mode == Mode::DefineArea && self.workflow.chooser.is_none() {
                let fresh = (!self.pointer_scene_ready).then(|| self.layer_gates());
                let gates = fresh.as_ref().unwrap_or(&self.gates_cache);
                define_area::resolve(&self.drawing, gates, &mut self.region_cache, world).filter(
                    |r| match r {
                        Region::Area(id) => gates.selectable(*id),
                        _ => true,
                    },
                )
            } else {
                None
            };
    }

    pub(super) fn workflow_click(&mut self, world: Pt) -> Task<Message> {
        self.workflow_hover(world);
        let Some(region) = self.workflow.hover.clone() else {
            self.workflow.status="No closed area here. Use Detect Regions in advanced tools for enclosed wall networks.".into();
            return Task::none();
        };
        if let Some(code) = self.workflow.armed.clone() {
            let id = self.workflow_materialize(&region);
            self.workflow_assign(id, &code, None, None);
            self.workflow.status = "Assigned. Click another outline, or Escape to stop.".into();
            return Task::none();
        }
        let (x, y) = self.view.to_screen(crate::view_state::World {
            x: world.x,
            y: world.y,
        });
        self.workflow_open(
            region,
            Point::new(x as f32 + 41.0 + self.pages_nav_width(), y as f32 + 129.0),
        )
    }

    fn workflow_materialize(&mut self, region: &Region) -> i64 {
        match region {
            Region::Area(id) => *id,
            Region::Face(face) => {
                self.push_undo("Define Region");
                let id = define_area::materialize(&mut self.drawing, face);
                self.assign_area_to_dedicated_layer(id, false);
                id
            }
        }
    }

    pub(super) fn workflow_assign(
        &mut self,
        id: i64,
        code: &str,
        name: Option<&str>,
        post: Option<AutoPost>,
    ) -> bool {
        if !self.layer_gates().selectable(id) || !self.drawing.polygons.iter().any(|p| p.id == id) {
            self.workflow.status =
                "Show and unlock this area before changing its definition.".into();
            return false;
        }
        let default_name = self
            .area_codes
            .iter()
            .find(|c| c.code == code)
            .map(|c| c.name.as_str())
            .unwrap_or(code);
        let name = name
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(default_name)
            .trim()
            .to_string();
        self.push_undo("Define Area");
        define_area::define(&mut self.drawing, id, code, &name);
        if let Some(post) = post {
            let p = self
                .drawing
                .polygons
                .iter_mut()
                .find(|p| p.id == id)
                .unwrap();
            p.auto_post = post.clone();
            let ids: HashSet<_> = p.edges.iter().filter_map(|e| e.line_id).collect();
            for line in &mut self.drawing.lines {
                if ids.contains(&line.id) {
                    line.dimension_hidden = !post.dims;
                }
            }
        }
        let layer = self
            .drawing
            .polygons
            .iter()
            .find(|p| p.id == id)
            .and_then(|p| p.layer_id.clone());
        if let Some(layer) = layer {
            if self
                .layers
                .layer(&layer)
                .is_some_and(|l| define_area::is_generated_layer_name(&l.name))
            {
                let unique =
                    self.layers
                        .unique_layer_name(if name.is_empty() { "Area" } else { &name });
                self.layers.rename(&layer, unique);
            }
        }
        true
    }

    fn workflow_open(&mut self, region: Region, at: Point) -> Task<Message> {
        self.workflow_close();
        let poly = match &region {
            Region::Area(id) => self.drawing.polygons.iter().find(|p| p.id == *id),
            _ => None,
        };
        if let Region::Area(id) = &region {
            if !self.layer_gates().selectable(*id) || poly.is_none() {
                self.workflow.status =
                    "Show and unlock this area before editing its definition.".into();
                return Task::none();
            }
        }
        let existing = poly.is_some_and(|p| p.kind != "UND");
        let code = if existing {
            poly.map(|p| p.kind.clone())
        } else if self.workflow.repeat {
            self.workflow.remembered.clone()
        } else {
            None
        };
        let known = code
            .as_ref()
            .and_then(|code| self.area_codes.iter().find(|c| &c.code == code));
        let extra_code = poly
            .filter(|p| p.kind != "UND" && !self.area_codes.iter().any(|c| c.code == p.kind))
            .map(|p| AreaCode {
                code: p.kind.clone(),
                name: p.name.clone(),
                group: "Custom".into(),
                description: "Existing classification".into(),
                ..Default::default()
            });
        let area = poly
            .map(|p| {
                if existing {
                    self.computed.get(&p.id).map_or(p.area, |c| c.net)
                } else {
                    p.gross
                }
            })
            .unwrap_or_else(|| match &region {
                Region::Face(f) => f.area,
                _ => 0.0,
            });
        let c = Chooser {
            page: self.pages.current,
            region: region.clone(),
            previous_selection: self.selection.clone(),
            search: String::new(),
            group: known
                .map(|c| c.group.clone())
                .or_else(|| extra_code.as_ref().map(|c| c.group.clone()))
                .unwrap_or_else(|| "GLA".into()),
            code,
            name: poly
                .filter(|_| existing)
                .map(|p| p.name.clone())
                .unwrap_or_default(),
            post: poly
                .map(|p| p.auto_post.clone())
                .unwrap_or_else(|| self.drawing.auto_post.clone()),
            labels: false,
            repeat: self.workflow.repeat,
            at,
            existing,
            area,
            extra_code,
        };
        if let Region::Area(id) = region {
            self.selection.polys.clear();
            self.selection.polys.insert(id);
        }
        self.workflow.advanced = false;
        self.workflow.chooser = Some(c);
        self.sync_edit_tools();
        iced::widget::operation::focus("area-definition-search")
    }

    pub(super) fn workflow_msg(&mut self, msg: Msg) -> Task<Message> {
        match msg {
            Msg::Activate => return self.workflow_activate(),
            Msg::NewArea => {
                let n = self
                    .drawing
                    .polygons
                    .iter()
                    .filter(|p| p.kind == "UND" && self.layer_gates().shows(p.id))
                    .count();
                if n > 0 {
                    let _ = self.workflow_activate();
                    self.workflow.find_undefined = true;
                    self.workflow.status = format!(
                        "{n} outline{} waiting. Click one to define it, or drag a type onto it.",
                        if n == 1 { " is" } else { "s are" }
                    );
                } else {
                    let _ = self.workflow_escape();
                    self.workflow.status =
                        "Draw a closed outline, then drag a type onto it.".into();
                }
            }
            Msg::FindUndefined => {
                self.workflow.find_undefined = !self.workflow.find_undefined;
                self.workflow.status = if self.workflow.find_undefined {
                    "Undefined outlines highlighted. Classification is optional."
                } else {
                    "Undefined highlights hidden."
                }
                .into();
            }
            Msg::Arm(code) => {
                let tile_code = code.clone();
                let next = if self.workflow.armed.as_ref() == Some(&code) {
                    None
                } else {
                    Some(code)
                };
                self.drawing.selected_area_type = next.clone();
                self.drawing.selected_area_name = next
                    .as_ref()
                    .and_then(|c| self.area_codes.iter().find(|e| &e.code == c))
                    .map(|e| e.name.clone());
                if next.is_some() {
                    let _ = self.workflow_activate();
                }
                self.workflow.status = next
                    .as_ref()
                    .map(|c| {
                        format!(
                            "Click an outline to assign {}. Escape stops.",
                            self.area_codes
                                .iter()
                                .find(|e| &e.code == c)
                                .map_or(c.as_str(), |e| e.name.as_str())
                        )
                    })
                    .unwrap_or_default();
                self.workflow.armed = next;
                self.workflow.menu = None;
                return iced::widget::operation::focus(format!("area-tile-{tile_code}"));
            }
            Msg::Group(group) => {
                if !self.workflow.collapsed.remove(&group) {
                    self.workflow.collapsed.insert(group);
                }
            }
            Msg::Advanced(open) => {
                self.workflow_close();
                self.workflow.advanced = open;
            }
            Msg::TypeMenu(code) => {
                self.workflow.menu = Some(code);
                self.context.close();
            }
            Msg::CloseMenu => self.workflow.menu = None,
            Msg::AssignSelected(code) => {
                self.workflow.menu = None;
                let ids: Vec<_> = self.selection.polys.iter().copied().collect();
                let n = ids
                    .into_iter()
                    .filter(|id| self.workflow_assign(*id, &code, None, None))
                    .count();
                self.workflow.status = format!(
                    "{n} area{} set to {}.",
                    if n == 1 { "" } else { "s" },
                    self.area_codes
                        .iter()
                        .find(|c| c.code == code)
                        .map_or(code.as_str(), |c| c.name.as_str())
                );
            }
            Msg::SelectUsing(code) => {
                self.workflow.menu = None;
                self.selection.polys = self
                    .drawing
                    .polygons
                    .iter()
                    .filter(|p| p.kind == code)
                    .map(|p| p.id)
                    .collect();
                self.workflow.status = format!(
                    "Selected {} areas using {code}.",
                    self.selection.polys.len()
                );
                self.sync_edit_tools();
            }
            Msg::EditType(code) => {
                self.workflow.menu = None;
                self.workflow.advanced = false;
                if let Some(c) = self.area_codes.iter().find(|c| c.code == code) {
                    self.def_editor = crate::def_editor::DefEditor::open_on(c);
                }
            }
            Msg::NewType => {
                self.workflow.menu = None;
                return self.update(Message::OpenNewCode);
            }
            Msg::DeleteType(code) => {
                self.workflow.menu = None;
                if code == "UND" {
                    self.workflow.status =
                        "Undefined is how unclassified outlines are marked. It cannot be deleted."
                            .into();
                    return Task::none();
                }
                let n = self
                    .drawing
                    .polygons
                    .iter()
                    .filter(|p| p.kind == code)
                    .count();
                let name = self
                    .area_codes
                    .iter()
                    .find(|c| c.code == code)
                    .map_or(code.as_str(), |c| c.name.as_str());
                return self.open_dialog(crate::dialog::Dialog::confirm(
                    &format!("Delete {name}? {n} areas using it become unclassified."),
                    crate::dialog::Action::DeleteAreaType(code),
                ));
            }
            Msg::ConfirmDelete(code) => {
                if code == "UND" {
                    return Task::none();
                }
                self.push_undo("Delete Area Type");
                for p in &mut self.drawing.polygons {
                    if p.kind == code {
                        p.kind = "UND".into();
                        p.name = "Undefined Area".into();
                    }
                }
                self.area_codes.retain(|c| c.code != code);
                self.persist_area_codes();
                if self.workflow.armed.as_ref() == Some(&code) {
                    self.workflow.armed = None;
                    self.drawing.selected_area_type = None;
                    self.drawing.selected_area_name = None;
                }
                self.workflow.status = format!("{code} deleted from the library.");
            }
            Msg::Open(id) => {
                let at = if self.workflow.menu_at == Point::ORIGIN {
                    self.drawing
                        .polygons
                        .iter()
                        .find(|p| p.id == id)
                        .map(|p| {
                            let c = self.geo().polygon_centroid(&p.points);
                            let (x, y) = self
                                .view
                                .to_screen(crate::view_state::World { x: c.x, y: c.y });
                            Point::new(x as f32 + 41.0 + self.pages_nav_width(), y as f32 + 129.0)
                        })
                        .unwrap_or(Point::new(100.0, 160.0))
                } else {
                    self.workflow.menu_at
                };
                self.workflow.menu = None;
                self.context.close();
                return self.workflow_open(Region::Area(id), at);
            }
            Msg::Cancel => {
                self.workflow_close();
                self.workflow.status = "Canceled. Drawing unchanged.".into();
            }
            Msg::Search(s) => {
                if let Some(c) = &mut self.workflow.chooser {
                    c.search = s;
                }
            }
            Msg::ChooseGroup(g) => {
                if let Some(c) = &mut self.workflow.chooser {
                    c.group = g;
                    c.search.clear();
                    c.code = None;
                }
            }
            Msg::Code(value) => {
                if let Some(c) = &mut self.workflow.chooser {
                    c.code = Some(value);
                }
            }
            Msg::Name(s) => {
                if let Some(c) = &mut self.workflow.chooser {
                    c.name = s;
                }
            }
            Msg::Labels => {
                if let Some(c) = &mut self.workflow.chooser {
                    c.labels = !c.labels;
                }
            }
            Msg::Post(key, value) => {
                if let Some(c) = &mut self.workflow.chooser {
                    key.set(&mut c.post, value);
                }
            }
            Msg::Suffix(s) => {
                if let Some(c) = &mut self.workflow.chooser {
                    c.post.suffix = s;
                }
            }
            Msg::Repeat(value) => {
                if let Some(c) = &mut self.workflow.chooser {
                    c.repeat = value;
                }
            }
            Msg::DefaultPost(key, value) => {
                if matches!(key, Post::Dims) {
                    self.view.dimensions = value;
                }
                key.set(&mut self.drawing.auto_post, value);
            }
            Msg::DefaultSuffix(s) => self.drawing.auto_post.suffix = s,
            Msg::DefaultPlacement(s) => {
                self.view.dimension_placement = if s == "In" {
                    crate::view_state::DimensionPlacement::In
                } else {
                    crate::view_state::DimensionPlacement::Out
                }
            }
            Msg::Submit => {
                let Some(c) = self.workflow.chooser.as_ref() else {
                    return Task::none();
                };
                let Some(code) = c.code.clone() else {
                    return Task::none();
                };
                if !c
                    .codes(&self.area_codes)
                    .iter()
                    .any(|e| e.code == code && c.matches(e))
                {
                    return Task::none();
                }
                let c = c.clone();
                if c.page != self.pages.current {
                    self.workflow_close();
                    return Task::none();
                }
                if let Region::Area(id) = c.region {
                    if !self.layer_gates().selectable(id) {
                        self.workflow_close();
                        return Task::none();
                    }
                }
                let id = self.workflow_materialize(&c.region);
                if self.workflow_assign(id, &code, Some(&c.name), Some(c.post)) {
                    self.workflow.repeat = c.repeat;
                    self.workflow.remembered = c.repeat.then_some(code);
                    self.workflow_close();
                    self.workflow.status =
                        "Area defined. Click another outline, or choose Draw.".into();
                }
            }
            Msg::Redefine => {
                if let Some(code) = self.drawing.selected_area_type.clone() {
                    let ids: Vec<_> = self.selection.polys.iter().copied().collect();
                    if ids.is_empty() {
                        return self.open_dialog(crate::dialog::Dialog::alert(
                            "Select a defined area on the canvas first.",
                        ));
                    }
                    let gates = self.layer_gates();
                    let ids: HashSet<_> =
                        ids.into_iter().filter(|id| gates.selectable(*id)).collect();
                    if !ids.is_empty() {
                        self.push_undo("Redefine Area");
                        let name = self
                            .drawing
                            .selected_area_name
                            .clone()
                            .unwrap_or_else(|| code.clone());
                        for poly in &mut self.drawing.polygons {
                            if ids.contains(&poly.id) {
                                poly.kind = code.clone();
                                poly.name = name.clone();
                            }
                        }
                    }
                    self.selection.clear();
                    self.sync_edit_tools();
                } else {
                    return self.open_dialog(crate::dialog::Dialog::alert(
                        "Click a type in the Areas palette first, then Redefine.",
                    ));
                }
            }
            Msg::Clone => {
                let gates = self.layer_gates();
                self.selection.polys.retain(|id| gates.selectable(*id));
                return self.clone_selected_areas();
            }
            Msg::Detect => return self.workflow_detect(),
            Msg::Reopen => return self.workflow_reopen(),
            Msg::Focus(reverse) => {
                return if self.workflow.chooser.is_some() {
                    focus_cycle(reverse)
                } else if reverse {
                    iced::widget::operation::focus_previous()
                } else {
                    iced::widget::operation::focus_next()
                }
            }
            Msg::Leave => self.workflow.hover = None,
        }
        Task::none()
    }

    fn workflow_detect(&mut self) -> Task<Message> {
        let gates = self.layer_gates();
        let lines: Vec<_> = self
            .drawing
            .lines
            .iter()
            .filter(|l| gates.selectable(l.id))
            .map(|l| l.geometry())
            .collect();
        let faces = self.geo().find_enclosed_faces(&lines, 4.0, 0.02);
        if faces.is_empty() {
            self.workflow.status = "No enclosed regions found.".into();
            return Task::none();
        }
        let faces: Vec<_> = faces
            .into_iter()
            .filter(|f| {
                let center = self.geo().polygon_centroid(&f.points);
                !self.drawing.polygons.iter().any(|p| {
                    self.geo()
                        .point_in_polygon(center, &define_area::ring_of(&self.drawing, p))
                })
            })
            .collect();
        if faces.is_empty() {
            self.workflow.status = "All enclosed regions are already defined.".into();
            return Task::none();
        }
        self.push_undo("Detect Regions");
        for face in &faces {
            let id = define_area::materialize(&mut self.drawing, face);
            self.assign_area_to_dedicated_layer(id, false);
        }
        self.workflow.status = format!(
            "Detected {} enclosed region{}.",
            faces.len(),
            if faces.len() == 1 { "" } else { "s" }
        );
        Task::none()
    }

    fn workflow_reopen(&mut self) -> Task<Message> {
        let Some(poly) = self
            .drawing
            .polygons
            .iter()
            .find(|p| {
                self.selection.polys.contains(&p.id)
                    && p.points.len() >= 3
                    && self.layer_gates().selectable(p.id)
            })
            .cloned()
        else {
            self.workflow.status = "Select an area to reopen.".into();
            return Task::none();
        };
        self.push_undo("Reopen Area");
        let a = poly.points[0];
        let b = poly.points[1];
        let boundary = poly.edges.first().and_then(|e| e.line_id).or_else(|| {
            self.drawing
                .lines
                .iter()
                .find(|l| (l.start == a && l.end == b) || (l.start == b && l.end == a))
                .map(|l| l.id)
        });
        if let Some(id) = boundary {
            self.drawing.lines.retain(|l| l.id != id);
        }
        self.drawing.polygons.retain(|p| p.id != poly.id);
        if let Some(layer) = poly.layer_id {
            self.layers.active = layer;
        }
        self.selection.clear();
        self.workflow.advanced = false;
        let task = self.set_mode(Mode::DrawExterior);
        self.drawing.pen_down = true;
        self.drawing.points = poly.points[1..]
            .iter()
            .copied()
            .chain(std::iter::once(a))
            .collect();
        self.drawing.line_ids = poly
            .edges
            .iter()
            .skip(1)
            .filter_map(|e| e.line_id)
            .collect();
        self.drawing.cursor = a;
        self.drawing.pointer = a;
        self.workflow.status="Area reopened. Continue drawing from the open end, then close back to the starting point.".into();
        self.sync_edit_tools();
        task
    }
}

fn send(m: Msg) -> Message {
    Message::AreaWorkflow(m)
}

pub fn draw_overlay(
    frame: &mut iced::widget::canvas::Frame,
    scene: &crate::render::Scene,
    state: &State,
    mode: Mode,
) {
    use iced::widget::canvas::{LineDash, Path, Stroke, Text};
    let screen = |p: Pt| {
        let (x, y) = crate::handles::to_screen(p, scene.pan, scene.zoom);
        Point::new(x as f32, y as f32)
    };
    if state.find_undefined {
        for poly in scene
            .drawing
            .polygons
            .iter()
            .filter(|p| p.kind == "UND" && scene.gates.shows(p.id))
        {
            let ring = define_area::ring_of(scene.drawing, poly);
            let path = Path::new(|p| {
                if let Some(first) = ring.first() {
                    p.move_to(screen(*first));
                    for pt in ring.iter().skip(1) {
                        p.line_to(screen(*pt));
                    }
                    p.close();
                }
            });
            frame.fill(&path, iced::Color::from_rgba(0.0, 0.478, 0.8, 0.08));
            frame.stroke(
                &path,
                Stroke {
                    line_dash: LineDash {
                        segments: &[6.0, 4.0],
                        offset: 0,
                    },
                    ..Stroke::default().with_color(accent()).with_width(2.0)
                },
            );
        }
    }
    if mode == Mode::DefineArea && state.chooser.is_none() {
        if let Some(region) = &state.hover {
            let ring = define_area::region_ring(scene.drawing, region);
            crate::render::draw_drop_hint(frame, scene, &ring, hex(0x6366F1));
            let area = match region {
                Region::Face(f) => f.area,
                Region::Area(id) => scene
                    .drawing
                    .polygons
                    .iter()
                    .find(|p| p.id == *id)
                    .map_or(0.0, |p| p.gross.abs()),
            };
            let anchor = scene.drawing.engine().area_label_anchor(&ring);
            frame.fill_text(Text {
                content: format!("{area:.1}"),
                position: screen(anchor),
                color: hex(0x4F46E5),
                size: 15.0.into(),
                font: SEGOE_SEMIBOLD,
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }
    }
}
fn action<'a>(label: &str, m: Msg) -> Element<'a, Message> {
    let selected = label.ends_with(" ✓");
    let button = button(text(label.to_string()).size(12).font(SEGOE))
        .on_press(send(m.clone()))
        .padding([5, 8])
        .style(move |_, status| iced::widget::button::Style {
            background: Some(
                if selected {
                    accent()
                } else if matches!(
                    status,
                    iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
                ) {
                    bg_ribbon()
                } else {
                    bg_hover()
                }
                .into(),
            ),
            text_color: if selected {
                iced::Color::WHITE
            } else {
                text_main()
            },
            border: iced::Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        });
    crate::event_tap::EventTap::new(button)
        .keyboard_action(format!("area-action-{label}"), send(m))
        .into()
}

/// Native buttons and selects need an explicit keyboard surface in iced.
fn keyboard_select<'a>(
    content: impl Into<Element<'a, Message>>,
    id: &str,
    values: Vec<String>,
    current: Option<String>,
    message: impl Fn(String) -> Msg + 'a,
) -> Element<'a, Message> {
    let next = |delta: isize| {
        let index = current
            .as_ref()
            .and_then(|c| values.iter().position(|v| v == c));
        let n = values.len() as isize;
        if n == 0 {
            None
        } else {
            Some(
                values[(index.map_or(if delta > 0 { -1 } else { 0 }, |i| i as isize) + delta)
                    .rem_euclid(n) as usize]
                    .clone(),
            )
        }
    };
    let forward = next(1);
    let backward = next(-1);
    let activate = forward
        .clone()
        .map(|v| send(message(v)))
        .unwrap_or(Message::Noop);
    crate::event_tap::EventTap::new(content)
        .keyboard_action(id.to_string(), activate)
        .focused_key(move |key, _| {
            use iced::keyboard::{key::Named, Key};
            let value = match key.as_ref() {
                Key::Named(Named::ArrowDown | Named::ArrowRight) => forward.clone(),
                Key::Named(Named::ArrowUp | Named::ArrowLeft) => backward.clone(),
                _ => None,
            };
            value.map(|v| send(message(v)))
        })
        .into()
}

fn focus_cycle(reverse: bool) -> Task<Message> {
    use iced::advanced::widget::{operation, Operation};
    struct Cycle {
        index: usize,
        target: usize,
    }
    impl Operation<Message> for Cycle {
        fn focusable(
            &mut self,
            _id: Option<&iced::advanced::widget::Id>,
            _bounds: iced::Rectangle,
            state: &mut dyn operation::Focusable,
        ) {
            if self.index == self.target {
                state.focus();
            } else {
                state.unfocus();
            }
            self.index += 1;
        }
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Message>)) {
            operate(self);
        }
    }
    let op = if reverse {
        operation::then(operation::focusable::count(), |count| {
            let n = count.total.max(1);
            Cycle {
                index: 0,
                target: count.focused.map_or(n - 1, |i| (i + n - 1) % n),
            }
        })
    } else {
        operation::then(operation::focusable::count(), |count| {
            let n = count.total.max(1);
            Cycle {
                index: 0,
                target: count.focused.map_or(0, |i| (i + 1) % n),
            }
        })
    };
    iced_runtime::task::widget(operation::scope("area-chooser-scope".into(), op))
}

pub fn post_controls<'a>(post: &AutoPost, defaults: bool) -> Element<'a, Message> {
    let mut list = column![].spacing(5);
    for (key, label, value) in [
        (Post::Code, "Code", post.code),
        (Post::Name, "Name", post.name),
        (Post::Dims, "Dimensions", post.dims),
        (Post::Calcs, "Area", post.calcs),
        (Post::Base, "Base", post.base),
        (Post::Factor, "Factor", post.factor),
    ] {
        let control = checkbox(value)
            .label(label)
            .on_toggle(move |v| {
                send(if defaults {
                    Msg::DefaultPost(key, v)
                } else {
                    Msg::Post(key, v)
                })
            })
            .text_size(12);
        list = list.push(crate::event_tap::EventTap::new(control).keyboard_action(
            format!("area-post-{label}"),
            send(if defaults {
                Msg::DefaultPost(key, !value)
            } else {
                Msg::Post(key, !value)
            }),
        ));
    }
    let values: Vec<String> = [
        "sf", "ft²", "SF", "FT²", "s.f.", "S.F.", "sq. ft.", "Sq. Ft.", "m", "m²", "sq. m", "Sq. M",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let picker = pick_list(values.clone(), Some(post.suffix.clone()), move |s| {
        send(if defaults {
            Msg::DefaultSuffix(s)
        } else {
            Msg::Suffix(s)
        })
    });
    list = list
        .push(text("Area suffix").size(12))
        .push(keyboard_select(
            picker,
            "area-suffix",
            values,
            Some(post.suffix.clone()),
            move |s| {
                if defaults {
                    Msg::DefaultSuffix(s)
                } else {
                    Msg::Suffix(s)
                }
            },
        ));
    list.into()
}

pub fn advanced<'a>(
    post: &AutoPost,
    dimensions: bool,
    placement: crate::view_state::DimensionPlacement,
) -> Element<'a, Message> {
    let mut post = post.clone();
    post.dims = dimensions;
    let placement = if placement == crate::view_state::DimensionPlacement::In {
        "In"
    } else {
        "Out"
    }
    .to_string();
    let values = vec!["Out".to_string(), "In".to_string()];
    let picker = pick_list(values.clone(), Some(placement.clone()), |s| {
        send(Msg::DefaultPlacement(s))
    });
    scrollable(column![
        row![text("Edit areas").font(SEGOE_BOLD).size(14),Space::new().width(Length::Fill),action("×",Msg::Advanced(false))].align_y(iced::Alignment::Center),
        text("Area operations").size(13).font(SEGOE_BOLD),
        text("Acts on the areas selected on the canvas. Redefine uses the type armed in the palette.").size(12),
        row![action("Redefine",Msg::Redefine),action("Clone",Msg::Clone)].spacing(4),
        row![action("Reopen",Msg::Reopen),action("Detect Regions",Msg::Detect)].spacing(4),
        text("Label defaults").size(13).font(SEGOE_BOLD),text("Applied to areas you define from now on.").size(12),post_controls(&post,true),
        text("Dimension placement").size(12),keyboard_select(picker,"area-dimension-placement",values,Some(placement),Msg::DefaultPlacement)
    ].spacing(12)).into()
}

pub fn chooser_layer<'a>(
    state: &State,
    codes: &[AreaCode],
    size: iced::Size,
) -> Option<Element<'a, Message>> {
    let c = state.chooser.as_ref()?;
    let all = c.codes(codes);
    let mut groups = iced::widget::Column::new().spacing(4);
    let mut seen = HashSet::new();
    let mut group_row = row![].spacing(4);
    let mut count = 0;
    for code in &all {
        if seen.insert(code.group.clone())
            && (c.search.trim().is_empty()
                || all.iter().any(|a| a.group == code.group && c.matches(a)))
        {
            let label = match dock::palette_group_label(&code.group) {
                "" => code.group.as_str(),
                l => l,
            };
            group_row = group_row.push(action(
                &format!("{}{}", label, if c.group == code.group { " ✓" } else { "" }),
                Msg::ChooseGroup(code.group.clone()),
            ));
            count += 1;
            if count % 2 == 0 {
                groups = groups.push(group_row);
                group_row = row![].spacing(4);
            }
        }
    }
    groups = groups.push(group_row);
    let matches: Vec<_> = all
        .iter()
        .filter(|a| c.matches(a))
        .map(|a| format!("{} ({})", a.name, a.code))
        .collect();
    let selected = c
        .code
        .as_ref()
        .and_then(|code| all.iter().find(|a| &a.code == code && c.matches(a)))
        .map(|a| format!("{} ({})", a.name, a.code));
    let submit = if c.existing {
        "Save changes"
    } else {
        "Define area"
    };
    let parse_code = |s: String| {
        Msg::Code(
            s.rsplit_once(" (")
                .map_or(s.clone(), |(_, c)| c.trim_end_matches(')').to_string()),
        )
    };
    let picker = pick_list(matches.clone(), selected.clone(), move |s| {
        send(parse_code(s))
    })
    .placeholder("Choose a classification...")
    .text_size(12);
    let classification = keyboard_select(
        picker,
        "area-classification",
        matches,
        selected.clone(),
        parse_code,
    );
    let mut body = column![
        row![
            text(if c.existing {
                "Edit definition"
            } else {
                "Define this area"
            })
            .font(SEGOE_BOLD)
            .size(14),
            Space::new().width(Length::Fill),
            text(format!("{:.1} sf", c.area)).size(12)
        ],
        text("What is this space?").size(12),
        text_input("Find any type or custom code...", &c.search)
            .id("area-definition-search")
            .on_input(|s| send(Msg::Search(s)))
            .on_submit(send(Msg::Submit))
            .size(12),
        groups,
        text("Classification").size(12),
        classification,
        text("Name · optional").size(12),
        text_input("Custom area name", &c.name)
            .on_input(|s| send(Msg::Name(s)))
            .on_submit(send(Msg::Submit))
            .size(12),
        action(
            if c.labels {
                "▾ Label settings"
            } else {
                "▸ Label settings"
            },
            Msg::Labels
        )
    ]
    .spacing(8);
    if c.labels {
        body = body.push(post_controls(&c.post, false));
    }
    let mut save = button(text(submit).size(12))
        .padding([6, 10])
        .style(|_, status| iced::widget::button::Style {
            background: Some(
                if matches!(status, iced::widget::button::Status::Disabled) {
                    bg_hover()
                } else {
                    accent()
                }
                .into(),
            ),
            text_color: if matches!(status, iced::widget::button::Status::Disabled) {
                muted()
            } else {
                iced::Color::WHITE
            },
            border: iced::Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        });
    if selected.is_some() {
        save = save.on_press(send(Msg::Submit));
    }
    let repeat = crate::event_tap::EventTap::new(
        checkbox(c.repeat)
            .label("Keep assigning this type")
            .on_toggle(|v| send(Msg::Repeat(v)))
            .text_size(12),
    )
    .keyboard_action("area-repeat", send(Msg::Repeat(!c.repeat)));
    let save: Element<'a, Message> = if selected.is_some() {
        crate::event_tap::EventTap::new(save)
            .keyboard_action("area-submit", send(Msg::Submit))
            .into()
    } else {
        save.into()
    };
    body = body.push(repeat).push(
        row![
            Space::new().width(Length::Fill),
            action("Cancel", Msg::Cancel),
            save
        ]
        .spacing(8),
    );
    let height = if c.labels { 720.0_f32 } else { 510.0_f32 }.min((size.height - 20.0).max(80.0));
    let panel = container(scrollable(body).height(Length::Shrink))
        .id("area-chooser-scope")
        .padding(14)
        .width(300)
        .max_height(height)
        .style(|_| container::Style {
            background: Some(bg_panel().into()),
            text_color: Some(text_main()),
            border: iced::Border {
                color: border(),
                width: 1.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        });
    let left = (c.at.x + 16.0).min(size.width - 310.0).max(10.0);
    let top = c.at.y.min(size.height - height - 10.0).max(10.0);
    Some(
        iced::widget::stack![
            opaque(
                mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
                    .on_press(send(Msg::Cancel))
            ),
            crate::theme::at(left, top, opaque(panel))
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    )
}

pub fn menu_layer<'a>(
    state: &State,
    codes: &[AreaCode],
    selected: usize,
    size: iced::Size,
) -> Option<Element<'a, Message>> {
    let code = state.menu.as_ref()?;
    let name = codes
        .iter()
        .find(|c| &c.code == code)
        .map_or(code.as_str(), |c| c.name.as_str());
    let mut body = column![text(name.to_string()).size(12).font(SEGOE_BOLD)].spacing(3);
    if selected > 0 {
        body = body.push(action(
            &format!(
                "{} {selected} selected areas",
                if code == "UND" { "Clear" } else { "Assign to" }
            ),
            Msg::AssignSelected(code.clone()),
        ));
    }
    body = body.push(action(
        if state.armed.as_ref() == Some(code) {
            "Stop assigning"
        } else {
            "Assign by clicking outlines"
        },
        Msg::Arm(code.clone()),
    ));
    if let Some(n) = state.usage.get(code).filter(|n| **n > 0) {
        body = body.push(action(
            &format!("Select the {n} areas using it"),
            Msg::SelectUsing(code.clone()),
        ));
    }
    body = body
        .push(action("Edit type...", Msg::EditType(code.clone())))
        .push(action("New type...", Msg::NewType))
        .push(action("Delete type...", Msg::DeleteType(code.clone())));
    let panel = container(body)
        .padding(6)
        .width(265)
        .style(|_| container::Style {
            background: Some(bg_panel().into()),
            border: iced::Border {
                color: border(),
                width: 1.0,
                ..Default::default()
            },
            ..Default::default()
        });
    Some(
        iced::widget::stack![
            opaque(
                mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
                    .on_press(send(Msg::CloseMenu))
            ),
            crate::theme::at(
                state.menu_at.x.min(size.width - 273.0).max(8.0),
                state.menu_at.y.min(size.height - 260.0).max(8.0),
                opaque(panel)
            )
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    )
}

pub(super) fn drag_ghost<'a>(
    drag: Option<&crate::DockDrag>,
    codes: &[AreaCode],
    size: iced::Size,
) -> Option<Element<'a, Message>> {
    let drag = drag.filter(|d| d.active)?;
    let dock::DragItem::Area { code, .. } = &drag.item else {
        return None;
    };
    let at = drag.cursor?;
    let entry = codes.iter().find(|c| &c.code == code);
    let color = entry.map_or(hex(0x858585), |e| hex(dock::code_color(e)));
    let swatch = container(Space::new().width(8).height(16)).style(move |_| container::Style {
        background: Some(color.into()),
        ..Default::default()
    });
    let name = entry.map_or(code.as_str(), |e| e.name.as_str()).to_string();
    let body = container(row![swatch, text(name).size(12), text(code.clone()).size(10)].spacing(8))
        .padding(8)
        .style(|_| container::Style {
            background: Some(bg_panel().into()),
            border: iced::Border {
                color: border(),
                width: 1.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        });
    Some(crate::theme::at(
        (at.x + 16.0).min(size.width - 240.0).max(0.0),
        (at.y + 16.0).min(size.height - 40.0).max(0.0),
        body,
    ))
}
