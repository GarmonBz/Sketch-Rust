//! Workfile support for the Property Hackers Sketchy 0.x draft.

use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;
use serde_json::{json, Value};
use sketch_core::geometry::{area_codes, AreaCode, Edge, GeometryEngine, Pt, MAX_BOW_ANGLE};
use sketch_core::model::{AutoPost, Polygon, SketchLine, WallType};

use crate::{layers::Layer, pages::Page, placed_text::PlacedText};

mod save;
pub use save::{prepare_save, Backing, PageKey};
use save::{Binding, Entity};

type Result<T> = std::result::Result<T, String>;
type Point = [f64; 2];

pub fn recognizes(value: &Value) -> bool {
    value.get("format").and_then(Value::as_str) == Some("sketchy")
}

#[derive(Deserialize)]
struct Document {
    id: String,
    length_unit: String,
    structures: Vec<Structure>,
    title: Option<String>,
    measured_to: Option<String>,
    #[serde(default)]
    property: Property,
    #[serde(default)]
    site: Site,
    #[serde(default)]
    presentation: Presentation,
    #[serde(default)]
    extensions_required: Vec<String>,
}

#[derive(Default, Deserialize)]
struct Property {
    #[serde(default)]
    address: String,
}

#[derive(Default, Deserialize)]
struct Site {
    #[serde(default)]
    spaces: Vec<Space>,
}

#[derive(Deserialize)]
struct Structure {
    id: String,
    name: Option<String>,
    position: Option<Point>,
    rotation: Option<f64>,
    levels: Vec<Level>,
}

#[derive(Deserialize)]
struct Level {
    id: String,
    name: Option<String>,
    grade: String,
    sequence: i64,
    #[serde(default)]
    spaces: Vec<Space>,
    #[serde(default)]
    walls: Vec<Wall>,
    #[serde(default)]
    voids: Vec<Void>,
    #[serde(default)]
    openings: Vec<Opening>,
    #[serde(default)]
    features: Vec<Feature>,
}

#[derive(Deserialize)]
struct Space {
    id: String,
    name: Option<String>,
    #[serde(rename = "use")]
    use_kind: Option<String>,
    finished: Option<bool>,
    measured_to: Option<String>,
    within: Option<String>,
    #[serde(default)]
    subtract: bool,
    area_factor: Option<f64>,
    shape: Shape,
    #[serde(default)]
    attributes: BTreeMap<String, String>,
    #[serde(default)]
    classifications: Vec<Classification>,
}

#[derive(Deserialize)]
struct Classification {
    scheme: String,
    code: String,
}

#[derive(Deserialize)]
struct Void {
    id: String,
    name: Option<String>,
    shape: Shape,
}

#[derive(Deserialize)]
struct Wall {
    id: String,
    start: Point,
    end: Point,
    #[serde(default)]
    bulge: f64,
    thickness: Option<f64>,
    kind: Option<String>,
    alignment: Option<String>,
}

#[derive(Deserialize)]
struct Opening {
    id: String,
    wall: Option<String>,
    space: Option<String>,
    edge: Option<usize>,
    offset: f64,
    width: f64,
    projection: Option<Shape>,
}

#[derive(Deserialize)]
struct Feature {
    id: String,
    kind: String,
    name: Option<String>,
    position: Point,
    rotation: Option<f64>,
    width: Option<f64>,
    length: Option<f64>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Shape {
    Polygon {
        #[serde(flatten)]
        ring: Path,
        #[serde(default)]
        holes: Vec<Path>,
    },
    Polyline {
        #[serde(flatten)]
        path: Path,
    },
    Circle {
        center: Point,
        radius: f64,
    },
    Ellipse {
        center: Point,
        radius_x: f64,
        radius_y: f64,
        rotation: Option<f64>,
    },
    Arc {
        center: Point,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize)]
struct Path {
    points: Vec<Point>,
    bulges: Option<Vec<f64>>,
}

#[derive(Default, Deserialize)]
struct Presentation {
    #[serde(default)]
    layers: Vec<SourceLayer>,
    #[serde(default)]
    element_layers: HashMap<String, String>,
    #[serde(default)]
    annotations: Vec<Annotation>,
    #[serde(default)]
    underlays: Vec<Value>,
    #[serde(default)]
    pages: Vec<Value>,
    #[serde(default)]
    styles: HashMap<String, Value>,
}

#[derive(Deserialize)]
struct SourceLayer {
    id: String,
    name: String,
    visible: Option<bool>,
    locked: Option<bool>,
    parent: Option<String>,
}

#[derive(Deserialize)]
struct Annotation {
    id: String,
    level: Option<String>,
    #[serde(flatten)]
    detail: AnnotationKind,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum AnnotationKind {
    Text {
        position: Point,
        text: String,
        rotation: Option<f64>,
    },
    Label {
        target: String,
        fields: Vec<String>,
        position: Option<Point>,
        rotation: Option<f64>,
    },
    Shape {
        shape: Shape,
    },
    NorthArrow {
        rotation: Option<f64>,
    },
    #[serde(other)]
    Unsupported,
}

#[derive(Clone, Copy)]
struct Frame {
    feet: f64,
    position: Point,
    rotation: f64,
}

impl Frame {
    fn point(self, [x, y]: Point) -> Result<Pt> {
        let (sin, cos) = self.rotation.rem_euclid(360.0).to_radians().sin_cos();
        // Both coordinates are in the native world's +y up frame.
        checked_point(
            (self.position[0] + x * cos - y * sin) * self.feet,
            (self.position[1] + x * sin + y * cos) * self.feet,
        )
    }
}

fn units(unit: &str) -> Result<(f64, bool)> {
    match unit {
        "ft" => Ok((1.0, false)),
        "in" => Ok((1.0 / 12.0, false)),
        "m" => Ok((1.0 / 0.3048, true)),
        "cm" => Ok((0.01 / 0.3048, true)),
        "mm" => Ok((0.001 / 0.3048, true)),
        unit => Err(format!("unsupported length unit {unit}")),
    }
}

fn checked_point(x: f64, y: f64) -> Result<Pt> {
    if (x as f32).is_finite() && (y as f32).is_finite() {
        Ok(Pt::new(x, y))
    } else {
        Err("coordinate is out of range".into())
    }
}

#[derive(Clone, Copy)]
struct Segment {
    start: Pt,
    end: Pt,
    bulge: f64,
}

impl Segment {
    fn bow(self) -> f64 {
        (4.0 * self.bulge.atan()).to_degrees()
    }

    fn split(self) -> Result<Vec<Self>> {
        if self.bow().abs() <= MAX_BOW_ANGLE {
            return Ok(vec![self]);
        }
        // The midpoint of a bulged edge is offset by its sagitta to the left.
        let mid = checked_point(
            self.start.x / 2.0 + self.end.x / 2.0 - (self.end.y - self.start.y) * self.bulge / 2.0,
            self.start.y / 2.0 + self.end.y / 2.0 + (self.end.x - self.start.x) * self.bulge / 2.0,
        )?;
        let bulge = (self.bulge.atan() / 2.0).tan();
        Ok(vec![
            Self {
                end: mid,
                bulge,
                ..self
            },
            Self {
                start: mid,
                bulge,
                ..self
            },
        ])
    }

    fn chord(self) -> f64 {
        (self.end.x - self.start.x).hypot(self.end.y - self.start.y)
    }

    // Point at a fraction of arc length and the forward tangent. Sketchy's
    // positive bulge bows left, so it sweeps clockwise about its center.
    fn sample(self, t: f64) -> (Pt, f64) {
        let (dx, dy) = (self.end.x - self.start.x, self.end.y - self.start.y);
        let heading = dy.atan2(dx);
        if self.bulge.abs() < 1e-8 {
            return (
                Pt::new(self.start.x + dx * t, self.start.y + dy * t),
                heading,
            );
        }
        let sweep = -4.0 * self.bulge.atan();
        let half = sweep / 2.0;
        let angle = heading - half + sweep * t / 2.0;
        let length = self.chord() * (half * t).sin() / half.sin();
        (
            Pt::new(
                self.start.x + length * angle.cos(),
                self.start.y + length * angle.sin(),
            ),
            heading - half + sweep * t,
        )
    }
}

fn segments(points: &[Pt], closed: bool, bulges: Option<&[f64]>) -> Result<Vec<Segment>> {
    if points.len() < if closed { 3 } else { 2 } {
        return Err("too few points for the shape".into());
    }
    let count = points.len() - usize::from(!closed);
    if bulges.is_some_and(|b| b.len() != count) {
        return Err("bulge count does not match edge count".into());
    }
    (0..count)
        .map(|i| {
            let segment = Segment {
                start: points[i],
                end: points[(i + 1) % points.len()],
                bulge: bulges.map_or(0.0, |b| b[i]),
            };
            if segment.chord() == 0.0 {
                Err("edge has coincident endpoints".into())
            } else {
                Ok(segment)
            }
        })
        .collect()
}

impl Path {
    fn transform(&self, frame: Frame, closed: bool) -> Result<Vec<Segment>> {
        let points = self
            .points
            .iter()
            .map(|p| frame.point(*p))
            .collect::<Result<Vec<_>>>()?;
        segments(&points, closed, self.bulges.as_deref())
    }
}

fn radial(center: Point, radii: Point, rotation: f64, angle: f64) -> Point {
    let (sin, cos) = rotation.rem_euclid(360.0).to_radians().sin_cos();
    let (x, y) = (radii[0] * angle.cos(), radii[1] * angle.sin());
    [center[0] + x * cos - y * sin, center[1] + x * sin + y * cos]
}

impl Shape {
    fn outline(&self, frame: Frame) -> Result<Option<(bool, Vec<Segment>)>> {
        let outline = match self {
            Self::Polygon { ring, .. } => (true, ring.transform(frame, true)?),
            Self::Polyline { path } => (false, path.transform(frame, false)?),
            Self::Circle { center, radius } => {
                if *radius <= 0.0 {
                    return Err("circle radius must be positive".into());
                }
                let points = [
                    [center[0] + radius, center[1]],
                    [center[0], center[1] + radius],
                    [center[0] - radius, center[1]],
                    [center[0], center[1] - radius],
                ];
                let points = points
                    .map(|p| frame.point(p))
                    .into_iter()
                    .collect::<Result<Vec<_>>>()?;
                (
                    true,
                    segments(
                        &points,
                        true,
                        Some(&[-(std::f64::consts::PI / 8.0).tan(); 4]),
                    )?,
                )
            }
            Self::Ellipse {
                center,
                radius_x,
                radius_y,
                rotation,
            } => {
                if *radius_x <= 0.0 || *radius_y <= 0.0 {
                    return Err("ellipse radii must be positive".into());
                }
                let points = (0..128)
                    .map(|i| {
                        frame.point(radial(
                            *center,
                            [*radius_x, *radius_y],
                            rotation.unwrap_or(0.0),
                            f64::from(i) * std::f64::consts::TAU / 128.0,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                (true, segments(&points, true, None)?)
            }
            Self::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            } => {
                if *radius <= 0.0 {
                    return Err("arc radius must be positive".into());
                }
                let start = start_angle.rem_euclid(360.0);
                let sweep = (end_angle.rem_euclid(360.0) - start).rem_euclid(360.0);
                let sweep = if sweep == 0.0 { 360.0 } else { sweep };
                let count = (sweep / 180.0).ceil() as usize;
                let step = (sweep / count as f64).to_radians();
                let points = (0..=count)
                    .map(|i| {
                        frame.point(radial(
                            *center,
                            [*radius; 2],
                            0.0,
                            start.to_radians() + i as f64 * step,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                (
                    false,
                    segments(&points, false, Some(&vec![-(step / 4.0).tan(); count]))?,
                )
            }
            Self::Unknown => return Ok(None),
        };
        Ok(Some(outline))
    }
}

pub const AREA_CODE: &str = "SKETCHY";

pub fn area_code() -> AreaCode {
    AreaCode {
        code: AREA_CODE.into(),
        name: "Sketchy Area".into(),
        description: "Imported area; no measurement standard applied".into(),
        fill_style: "none".into(),
        ..AreaCode::default()
    }
}

pub struct Converted {
    pub model: Value,
    pub omitted: bool,
}

pub fn convert(mut source: Value) -> Result<Converted> {
    let version = source["format_version"]
        .as_str()
        .ok_or("missing format_version")?;
    if !version.split_once('.').is_some_and(|(major, minor)| {
        major == "0" && !minor.is_empty() && minor.bytes().all(|c| c.is_ascii_digit())
    }) {
        return Err(format!(
            "unsupported Sketchy version {version}; expected 0.x"
        ));
    }
    let snapshot = save::take_snapshot(&mut source);
    let document: Document = Document::deserialize(&source).map_err(|e| e.to_string())?;
    if !document.extensions_required.is_empty() {
        return Err(format!(
            "unsupported required Sketchy extensions: {}",
            document.extensions_required.join(", ")
        ));
    }
    if document.id.is_empty() {
        return Err("missing document ID".into());
    }
    let (feet, metric) = units(&document.length_unit)?;
    if let Some(snapshot) = snapshot {
        return Ok(save::converted(source, snapshot, true, false));
    }
    let presentation = &document.presentation;
    // The app assigns unlayered geometry to the first layer. Keep that
    // geometry visible independently of the source's named layers.
    let default_layer = (1..)
        .map(|i| format!("layer-{i}"))
        .find(|id| presentation.layers.iter().all(|layer| layer.id != *id))
        .expect("unused layer ID");
    let mut layers = vec![Layer::new(default_layer, "Sketchy Geometry")];
    for raw in &presentation.layers {
        let mut layer = Layer::new(&raw.id, &raw.name);
        layer.parent_id = raw.parent.clone();
        layer.visible = raw.visible.unwrap_or(true);
        layer.locked = raw.locked.unwrap_or(false);
        layers.push(layer);
    }
    let north = presentation
        .annotations
        .iter()
        .find_map(|a| match a.detail {
            AnnotationKind::NorthArrow { rotation } if a.level.is_none() => {
                Some(-rotation.unwrap_or(0.0))
            }
            _ => None,
        })
        .unwrap_or(0.0);

    let mut pages = Vec::new();
    let mut bindings = Vec::new();
    let mut next_id = 1;
    let mut omitted = !presentation.underlays.is_empty()
        || !presentation.pages.is_empty()
        || !presentation.styles.is_empty()
        || presentation
            .annotations
            .iter()
            .any(|a| matches!(a.detail, AnnotationKind::Unsupported));
    let mut build = |frame: Frame, level: Option<&Level>, spaces: &[Space], name| -> Result<()> {
        let mut builder = PageBuilder {
            page: Page {
                name,
                north_angle: north,
                sketchy_page: Some(
                    level.map_or(PageKey::Site, |level| PageKey::Level(level.id.clone())),
                ),
                ..Page::default()
            },
            frame,
            next_id: &mut next_id,
            omitted: &mut omitted,
            presentation,
            wall_segments: HashMap::new(),
            space_segments: HashMap::new(),
            edge_ids: HashMap::new(),
            area_indices: HashMap::new(),
            label_layers: HashMap::new(),
            metric,
            bindings: Vec::new(),
        };
        if let Some(level) = level {
            for wall in &level.walls {
                let first = builder.page.lines.len();
                builder
                    .wall(wall)
                    .map_err(|e| format!("wall {}: {e}", wall.id))?;
                builder.bindings.push(Binding::new(
                    &wall.id,
                    Entity::Wall(builder.line_ids(first)),
                ));
            }
        }
        for space in spaces {
            builder
                .space(space, document.measured_to.as_deref())
                .map_err(|e| format!("space {}: {e}", space.id))?;
        }
        for space in spaces.iter().filter(|s| s.within.is_some()) {
            if let Some(&index) = builder.area_indices.get(space.id.as_str()) {
                let target = space
                    .within
                    .as_deref()
                    .and_then(|p| builder.area_indices.get(p));
                if space.subtract {
                    if let Some(&target) = target {
                        builder.page.polygons[index].subtract_mode = "always".into();
                        builder.page.polygons[index].subtract_target_id =
                            Some(builder.page.polygons[target].id);
                    } else {
                        *builder.omitted = true;
                    }
                }
            }
        }
        if let Some(level) = level {
            for cutout in &level.voids {
                let first = builder.page.polygons.len();
                builder
                    .void(cutout)
                    .map_err(|e| format!("void {}: {e}", cutout.id))?;
                if let Some(area) = builder.page.polygons.get(first) {
                    builder
                        .bindings
                        .push(Binding::new(&cutout.id, Entity::Void(area.id)));
                }
            }
            for opening in &level.openings {
                let first = builder.page.lines.len();
                builder
                    .opening(opening)
                    .map_err(|e| format!("opening {}: {e}", opening.id))?;
                if let Some(line) = builder.page.lines.get(first) {
                    builder
                        .bindings
                        .push(Binding::new(&opening.id, Entity::Opening(line.id)));
                }
            }
            for feature in &level.features {
                let first = builder.page.labels.len();
                builder
                    .feature(feature)
                    .map_err(|e| format!("feature {}: {e}", feature.id))?;
                if let Some(label) = builder.page.labels.get(first) {
                    builder
                        .bindings
                        .push(Binding::new(&feature.id, Entity::Feature(label.id)));
                }
            }
        }
        for annotation in presentation
            .annotations
            .iter()
            .filter(|a| a.level.as_deref() == level.map(|l| l.id.as_str()))
        {
            let (line, label) = (builder.page.lines.len(), builder.page.labels.len());
            builder
                .annotate(annotation)
                .map_err(|e| format!("annotation {}: {e}", annotation.id))?;
            let entity = match &annotation.detail {
                AnnotationKind::Text { .. } => builder
                    .page
                    .labels
                    .get(label)
                    .map(|label| Entity::Text(label.id)),
                AnnotationKind::Shape { .. } if builder.page.lines.len() > line => {
                    Some(Entity::Drawing(builder.line_ids(line)))
                }
                AnnotationKind::NorthArrow { .. } => Some(Entity::North),
                AnnotationKind::Label { target, .. } => builder
                    .area_indices
                    .get(target.as_str())
                    .map(|&i| Entity::Label(builder.page.polygons[i].id)),
                _ => None,
            };
            if let Some(entity) = entity {
                builder.bindings.push(Binding::new(&annotation.id, entity));
            }
        }
        for poly in builder.page.polygons.iter().filter(|p| p.kind == AREA_CODE) {
            let deducted = builder
                .page
                .polygons
                .iter()
                .filter(|p| p.subtract_target_id == Some(poly.id))
                .map(|p| p.gross)
                .sum::<f64>();
            if !(poly.gross - deducted)
                .mul_add(poly.factor, 0.0)
                .is_finite()
            {
                return Err(format!("area of {} is out of range", poly.name));
            }
        }
        let mut page = serde_json::to_value(&builder.page).map_err(|e| e.to_string())?;
        for label in page["labels"]
            .as_array_mut()
            .expect("page labels are an array")
        {
            if let Some(layer) = label["id"]
                .as_i64()
                .and_then(|id| builder.label_layers.get(&id))
            {
                label["layerId"] = json!(layer);
            }
        }
        bindings.push((
            builder.page.sketchy_page.expect("Sketchy page key"),
            builder.bindings,
        ));
        pages.push(page);
        Ok(())
    };
    for structure in &document.structures {
        let frame = Frame {
            feet,
            position: structure.position.unwrap_or_default(),
            rotation: structure.rotation.unwrap_or_default(),
        };
        for level in &structure.levels {
            let level_name = level
                .name
                .clone()
                .unwrap_or_else(|| match level.grade.as_str() {
                    "below" => format!("Basement {}", level.sequence),
                    "partially_below" => format!("Partially below grade {}", level.sequence),
                    _ => format!("Level {}", level.sequence),
                });
            let structure_name = structure.name.as_deref().unwrap_or(&structure.id);
            let name = if document.structures.len() > 1 {
                format!("{structure_name}: {level_name}")
            } else {
                level_name
            };
            build(frame, Some(level), &level.spaces, name)?;
        }
    }
    if !document.site.spaces.is_empty()
        || presentation.annotations.iter().any(|a| {
            a.level.is_none()
                && matches!(
                    a.detail,
                    AnnotationKind::Text { .. } | AnnotationKind::Shape { .. }
                )
        })
    {
        build(
            Frame {
                feet,
                position: [0.0; 2],
                rotation: 0.0,
            },
            None,
            &document.site.spaces,
            "Site".into(),
        )?;
    }
    if pages.is_empty() {
        return Err("no levels or site drawing found".into());
    }

    let mut codes = area_codes::default_area_codes();
    codes.push(area_code());
    Ok(save::converted(
        source,
        save::Snapshot {
            model: json!({
                "fileName": document.title,
                "nextId": next_id,
                "areaCodes": codes,
                "layers": layers,
                "pages": pages,
                "subjectInfo": { "propertyAddress": document.property.address },
                "settings": { "options": { "unitSystem": if metric { "metric" } else { "imperial" } } }
            }),
            bindings,
        },
        false,
        omitted,
    ))
}

struct PageBuilder<'a> {
    page: Page,
    frame: Frame,
    next_id: &'a mut i64,
    omitted: &'a mut bool,
    presentation: &'a Presentation,
    wall_segments: HashMap<&'a str, Segment>,
    space_segments: HashMap<&'a str, Vec<Segment>>,
    edge_ids: HashMap<[u64; 5], i64>,
    area_indices: HashMap<&'a str, usize>,
    label_layers: HashMap<i64, String>,
    metric: bool,
    bindings: Vec<Binding>,
}

fn edge_key(segment: Segment) -> [u64; 5] {
    let (a, b, bulge) = if (segment.start.x, segment.start.y) <= (segment.end.x, segment.end.y) {
        (segment.start, segment.end, segment.bulge)
    } else {
        (segment.end, segment.start, -segment.bulge)
    };
    [a.x, a.y, b.x, b.y, bulge].map(|n| if n == 0.0 { 0 } else { n.to_bits() })
}

impl<'a> PageBuilder<'a> {
    fn line_ids(&self, first: usize) -> Vec<i64> {
        self.page.lines[first..]
            .iter()
            .map(|line| line.id)
            .collect()
    }
    fn id(&mut self) -> i64 {
        let id = *self.next_id;
        *self.next_id += 1;
        id
    }

    fn layer(&self, id: &str) -> Option<String> {
        self.presentation.element_layers.get(id).cloned()
    }

    fn line(&mut self, segment: Segment, style: &LineStyle, reuse: bool) -> i64 {
        let key = edge_key(segment);
        if reuse {
            if let Some(&id) = self.edge_ids.get(&key) {
                return id;
            }
        }
        let id = self.id();
        if style.color.is_none() {
            self.edge_ids.entry(key).or_insert(id);
        }
        self.page.lines.push(SketchLine {
            id,
            start: segment.start,
            end: segment.end,
            kind: style.kind,
            thickness: style.thickness,
            layer_id: style.layer.clone(),
            bow_angle: segment.bow(),
            arc_height: segment.chord() * segment.bulge / 2.0,
            dimension_hidden: style.hide_dimension,
            color: style.color.map(str::to_owned),
            line_thickness: None,
            line_style: None,
            dimension_position: None,
            dimension_anchor: None,
        });
        id
    }

    fn trace(
        &mut self,
        outline: &[Segment],
        style: &LineStyle,
        reuse: bool,
    ) -> Result<(Vec<Pt>, Vec<Edge>)> {
        let mut points = Vec::new();
        let mut edges = Vec::new();
        for segment in outline {
            for part in segment.split()? {
                let line_id = self.line(part, style, reuse);
                points.push(part.start);
                edges.push(Edge {
                    line_id: Some(line_id),
                    bow_angle: part.bow(),
                });
            }
        }
        Ok((points, edges))
    }

    fn wall(&mut self, wall: &'a Wall) -> Result<()> {
        let segment = Segment {
            start: self.frame.point(wall.start)?,
            end: self.frame.point(wall.end)?,
            bulge: wall.bulge,
        };
        if segment.chord() == 0.0 {
            return Err("wall has coincident endpoints".into());
        }
        if self.wall_segments.insert(&wall.id, segment).is_some() {
            return Err("duplicate wall ID".into());
        }
        // Keep the measured path. A native wall only has a centerline, so
        // preserve width but don't turn on solid walls or shift face-aligned geometry.
        *self.omitted |= matches!(wall.alignment.as_deref(), Some("left" | "right"));
        let thickness = wall.thickness.map(|t| t * self.frame.feet * 12.0);
        if thickness.is_some_and(|t| !t.is_finite() || t < 0.0) {
            return Err("wall thickness is out of range".into());
        }
        let style = LineStyle {
            kind: if matches!(
                wall.kind.as_deref(),
                Some("interior" | "party" | "half_wall")
            ) {
                WallType::Interior
            } else {
                WallType::Exterior
            },
            thickness,
            layer: self.layer(&wall.id),
            ..LineStyle::default()
        };
        self.trace(&[segment], &style, false)?;
        Ok(())
    }

    fn space(&mut self, space: &'a Space, measured_to: Option<&str>) -> Result<()> {
        let Some((closed, outline)) = space.shape.outline(self.frame)? else {
            *self.omitted = true;
            return Ok(());
        };
        let name = space
            .name
            .as_deref()
            .or(space.use_kind.as_deref())
            .unwrap_or(&space.id);
        let detail = space.within.is_some();
        let style = LineStyle {
            layer: self.layer(&space.id),
            hide_dimension: detail,
            ..LineStyle::default()
        };
        if !closed {
            let (_, edges) = self.trace(&outline, &style, true)?;
            self.bindings.push(Binding::new(
                &space.id,
                Entity::OpenSpace(edges.into_iter().filter_map(|edge| edge.line_id).collect()),
            ));
            if self.space_segments.insert(&space.id, outline).is_some() {
                return Err("duplicate space ID".into());
            }
            return Ok(());
        }
        let index = self.page.polygons.len();
        let mut area = self.area(&outline, &style, name, None)?;
        area.kind = if detail { "UND" } else { AREA_CODE }.into();
        area.factor = space.area_factor.unwrap_or(1.0);
        area.factor_override = Some(area.factor);
        area.area = area.gross * area.factor;
        if !area.area.is_finite() {
            return Err("area factor is out of range".into());
        }
        area.measure_to = match space.measured_to.as_deref().or(measured_to) {
            Some("interior_face") => "interior",
            Some("exterior_face") => "exterior",
            _ => "center",
        }
        .into();
        area.attributes
            .extend(space.attributes.iter().map(|(key, value)| {
                // Never let foreign metadata activate native area calculations.
                (
                    if key.starts_with("apex.") {
                        format!("sketchy.attribute.{key}")
                    } else {
                        key.clone()
                    },
                    value.clone(),
                )
            }));
        area.attributes
            .insert("sketchy.id".into(), space.id.clone());
        if let Some(kind) = &space.use_kind {
            area.attributes.insert("sketchy.use".into(), kind.clone());
        }
        if let Some(finished) = space.finished {
            area.attributes
                .insert("sketchy.finished".into(), finished.to_string());
        }
        for c in &space.classifications {
            let entry = area
                .attributes
                .entry(format!("sketchy.classification.{}", c.scheme))
                .or_default();
            if !entry.is_empty() {
                entry.push_str(", ");
            }
            entry.push_str(&c.code);
        }
        let id = area.id;
        self.page.polygons.push(area);
        self.area_indices.insert(&space.id, index);
        if self.space_segments.insert(&space.id, outline).is_some() {
            return Err("duplicate space ID".into());
        }
        if let Shape::Polygon { holes, .. } = &space.shape {
            for hole in holes {
                let ring = hole.transform(self.frame, true)?;
                let style = LineStyle {
                    hide_dimension: true,
                    ..style.clone()
                };
                let hole = self.area(&ring, &style, "Cutout", Some(id))?;
                self.page.polygons.push(hole);
            }
        }
        let holes = self.page.polygons[index + 1..]
            .iter()
            .map(|area| area.id)
            .collect();
        self.bindings
            .push(Binding::new(&space.id, Entity::Space { area: id, holes }));
        Ok(())
    }

    fn area(
        &mut self,
        outline: &[Segment],
        style: &LineStyle,
        name: &str,
        target: Option<i64>,
    ) -> Result<Polygon> {
        let (points, edges) = self.trace(outline, style, true)?;
        let gross = GeometryEngine::new().calculate_area(&points, &edges);
        if !gross.is_finite() || gross <= 0.0 {
            return Err("invalid closed area".into());
        }
        let id = self.id();
        serde_json::from_value(json!({
            "id": id, "name": name, "points": points, "edges": edges,
            "gross": gross, "area": gross, "layerId": style.layer,
            "subtractMode": if target.is_some() { "always" } else { "never" },
            "subtractTargetId": target,
            "autoPost": { "name": true, "dims": false, "calcs": true, "suffix": if self.metric { "m²" } else { "sf" } }
        })).map_err(|e| e.to_string())
    }

    fn void(&mut self, void: &Void) -> Result<()> {
        let Some((true, outline)) = void.shape.outline(self.frame)? else {
            *self.omitted = true;
            return Ok(());
        };
        let style = LineStyle {
            layer: self.layer(&void.id),
            hide_dimension: true,
            ..LineStyle::default()
        };
        let mut area = self.area(
            &outline,
            &style,
            void.name.as_deref().unwrap_or("Void"),
            None,
        )?;
        let geo = GeometryEngine::new();
        if let Some(parent) = self
            .page
            .polygons
            .iter()
            .filter(|p| {
                p.kind == AREA_CODE
                    && p.gross > area.gross
                    && geo.polygon_contains(
                        &p.points,
                        &p.edges,
                        &area.points,
                        &area.edges,
                        1e-4,
                        0.01,
                    )
            })
            .min_by(|a, b| a.gross.total_cmp(&b.gross))
        {
            area.subtract_mode = "always".into();
            area.subtract_target_id = Some(parent.id);
        }
        self.page.polygons.push(area);
        Ok(())
    }

    fn opening(&mut self, opening: &Opening) -> Result<()> {
        let host = if let Some(wall) = &opening.wall {
            self.wall_segments.get(wall.as_str()).copied()
        } else {
            opening
                .space
                .as_deref()
                .zip(opening.edge)
                .and_then(|(space, index)| self.space_segments.get(space)?.get(index).copied())
        };
        let style = LineStyle {
            layer: self.layer(&opening.id),
            hide_dimension: true,
            color: Some("#2563eb"),
            ..LineStyle::default()
        };
        if let Some(host) = host {
            let length: f64 = host
                .split()?
                .iter()
                .map(|s| GeometryEngine::new().arc_length(s.start, s.end, s.bow()))
                .sum();
            let start = opening.offset * self.frame.feet;
            let width = opening.width * self.frame.feet;
            let end = start + width;
            if start >= 0.0 && width > 0.0 && end <= length + 0.0001 {
                let (t0, t1) = ((start / length).min(1.0), (end / length).min(1.0));
                let (a, at) = host.sample(t0);
                let (b, bt) = host.sample(t1);
                if t1 > t0 {
                    let span = Segment {
                        start: a,
                        end: b,
                        bulge: (host.bulge.atan() * (t1 - t0)).tan(),
                    };
                    self.trace(&[span], &style, false)?;
                    for (p, angle) in [(a, at), (b, bt)] {
                        let r = (width / 8.0).clamp(0.08, 0.3);
                        let (x, y) = (-angle.sin() * r, angle.cos() * r);
                        self.line(
                            Segment {
                                start: Pt::new(p.x - x, p.y - y),
                                end: Pt::new(p.x + x, p.y + y),
                                bulge: 0.0,
                            },
                            &style,
                            false,
                        );
                    }
                }
            } else {
                *self.omitted = true;
            }
        } else {
            *self.omitted = true;
        }
        if let Some(projection) = &opening.projection {
            self.shape_lines(projection, &style)?;
        }
        Ok(())
    }

    fn feature(&mut self, feature: &Feature) -> Result<()> {
        let position = self.frame.point(feature.position)?;
        let style = LineStyle {
            layer: self.layer(&feature.id),
            hide_dimension: true,
            color: Some("#64748b"),
            ..LineStyle::default()
        };
        if let Some((w, l)) = feature
            .width
            .zip(feature.length)
            .filter(|(w, l)| *w > 0.0 && *l > 0.0)
        {
            let (x, y) = (w / 2.0, l / 2.0);
            let rotation = feature
                .rotation
                .unwrap_or(0.0)
                .rem_euclid(360.0)
                .to_radians();
            let (sin, cos) = rotation.sin_cos();
            let points = [[-x, -y], [x, -y], [x, y], [-x, y]]
                .map(|[x, y]| {
                    self.frame.point([
                        feature.position[0] + x * cos - y * sin,
                        feature.position[1] + x * sin + y * cos,
                    ])
                })
                .into_iter()
                .collect::<Result<Vec<_>>>()?;
            self.trace(&segments(&points, true, None)?, &style, false)?;
        }
        self.label(
            feature
                .name
                .as_deref()
                .unwrap_or(&feature.kind)
                .replace('_', " "),
            position,
            0.0,
            style.layer,
        );
        Ok(())
    }

    fn shape_lines(&mut self, shape: &Shape, style: &LineStyle) -> Result<()> {
        if let Some((_, outline)) = shape.outline(self.frame)? {
            self.trace(&outline, style, false)?;
            if let Shape::Polygon { holes, .. } = shape {
                for hole in holes {
                    self.trace(&hole.transform(self.frame, true)?, style, false)?;
                }
            }
        } else {
            *self.omitted = true;
        }
        Ok(())
    }

    fn label(&mut self, text: String, point: Pt, rotation: f64, layer: Option<String>) {
        let id = self.id();
        let mut label = PlacedText::new(id, text);
        label.x = point.x;
        label.y = point.y;
        label.rotation = rotation;
        label.color.clear();
        if let Some(layer) = layer {
            self.label_layers.insert(id, layer);
        }
        self.page.labels.push(label);
    }

    fn annotate(&mut self, annotation: &Annotation) -> Result<()> {
        let layer = self.layer(&annotation.id);
        let rotation = |r: Option<f64>| {
            -(r.unwrap_or(0.0).rem_euclid(360.0) + self.frame.rotation.rem_euclid(360.0))
        };
        match &annotation.detail {
            AnnotationKind::Text {
                position,
                text,
                rotation: angle,
            } => {
                self.label(
                    text.clone(),
                    self.frame.point(*position)?,
                    rotation(*angle),
                    layer,
                );
            }
            AnnotationKind::Shape { shape } => {
                self.shape_lines(
                    shape,
                    &LineStyle {
                        layer,
                        hide_dimension: true,
                        ..LineStyle::default()
                    },
                )?;
            }
            AnnotationKind::NorthArrow { rotation: angle } => {
                self.page.north_angle = rotation(*angle)
            }
            AnnotationKind::Label {
                target,
                fields,
                position,
                rotation: angle,
            } => {
                if let Some(&i) = self.area_indices.get(target.as_str()) {
                    let position = position.map(|p| self.frame.point(p)).transpose()?;
                    let field = |f: &str| fields.iter().any(|s| s == f);
                    let poly = &mut self.page.polygons[i];
                    if poly.kind == AREA_CODE {
                        poly.label_position = position;
                        poly.label_rotation = angle.map(|a| rotation(Some(a)));
                        poly.auto_post = AutoPost {
                            name: field("name"),
                            code: false,
                            dims: field("dimensions"),
                            calcs: field("area"),
                            factor: field("factor"),
                            ..poly.auto_post.clone()
                        };
                        // A classification belongs to its own scheme, not the app's neutral area code.
                        *self.omitted |= field("perimeter");
                    } else if field("name") || field("area") {
                        let position = position.unwrap_or_else(|| {
                            GeometryEngine::new().area_label_anchor(&poly.points)
                        });
                        let mut parts = Vec::new();
                        if field("name") {
                            parts.push(poly.name.clone());
                        }
                        if field("area") {
                            let (area, suffix) = if self.metric {
                                (poly.area * 0.3048f64.powi(2), "m²")
                            } else {
                                (poly.area, "sf")
                            };
                            parts.push(format!("{area:.2} {suffix}"));
                        }
                        self.label(parts.join("\n"), position, rotation(*angle), layer);
                    }
                } else {
                    *self.omitted = true;
                }
            }
            AnnotationKind::Unsupported => *self.omitted = true,
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
struct LineStyle {
    kind: WallType,
    thickness: Option<f64>,
    layer: Option<String>,
    hide_dimension: bool,
    color: Option<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{read_sketch_model, App, Message, SketchFileFormat};

    fn fixture() -> Value {
        json!({
            "format": "sketchy", "format_version": "0.1", "id": "test-plan",
            "length_unit": "in", "property": { "address": "100 Example Lane" },
            "structures": [{"id": "house", "position": [240, 120], "rotation": 90,
                "levels": [{"id": "ground", "grade": "above", "sequence": 1,
                    "spaces": [
                        {"id":"room", "name":"Main Floor", "area_factor":0.5,
                            "shape":{"type":"polygon", "points":[[0,0],[120,0],[120,120],[0,120]],
                                     "holes":[{"points":[[12,12],[36,12],[36,36],[12,36]]}]}},
                        {"id":"detail", "within":"room",
                            "shape":{"type":"polygon", "points":[[0,0],[48,0],[48,48],[0,48]]}},
                        {"id":"deduct", "within":"room", "subtract":true,
                            "shape":{"type":"polygon", "points":[[48,48],[60,48],[60,60],[48,60]]}},
                        {"id":"open", "shape":{"type":"polyline", "points":[[144,0],[180,0]], "bulges":[1]}}
                    ],
                    "walls":[{"id":"front", "start":[0,0], "end":[120,0], "thickness":6,"kind":"exterior"}],
                    "openings":[{"id":"front-door","kind":"door","wall":"front","offset":24,"width":36}],
                    "voids":[{"id":"stairwell","shape":{"type":"circle","center":[84,84],"radius":12}}]
                }]
            }],
            "presentation":{"annotations":[{"id":"note","kind":"text","level":"ground",
                "text":"Test note","position":[12,24],"rotation":30}]}
        })
    }

    fn near(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-7, "{actual} != {expected}");
    }

    #[test]
    fn sketchy_converts_units_curves_and_deductions_without_double_counting_subdivisions() {
        let converted = convert(fixture()).unwrap();
        assert!(!converted.omitted);
        let page: Page = serde_json::from_value(converted.model["pages"][0].clone()).unwrap();
        assert_eq!(page.name, "Level 1");
        assert_eq!(page.polygons.len(), 5, "open paths aren't areas");
        let wall = &page.lines[0];
        near(wall.start.x, 20.0);
        near(wall.start.y, 10.0);
        near(wall.end.x, 20.0);
        near(wall.end.y, 20.0);
        assert_eq!(wall.thickness, Some(6.0));
        assert_eq!(page.polygons[0].edges[0].line_id, Some(wall.id));
        let opening = page
            .lines
            .iter()
            .find(|line| line.color.as_deref() == Some("#2563eb"))
            .unwrap();
        near(opening.start.y, 12.0);
        near(opening.end.y, 15.0);
        assert!(page.lines.iter().any(|line| line.bow_angle == 180.0));
        let label = &page.labels[0];
        near(label.x, 18.0);
        near(label.y, 11.0);
        near(label.rotation, -120.0);

        let results = sketch_core::area_calc::recalculate(
            &page.polygons,
            &[area_code()],
            &GeometryEngine::new(),
        );
        near(
            results.totals.total_all_areas,
            (100.0 - 4.0 - 1.0 - std::f64::consts::PI) * 0.5,
        );
        assert_eq!(results.totals.total_gla, 0.0);
    }

    #[test]
    fn sketchy_dispatch_requires_the_signature_and_validates_versions_and_required_extensions() {
        let mut source = fixture();
        source["format_version"] = json!("1.0");
        assert!(read_sketch_model(source.to_string().as_bytes())
            .unwrap_err()
            .contains("unsupported Sketchy version"));
        source["format_version"] = json!("0.3");
        source["extensions_required"] = json!(["org.example.required"]);
        assert!(read_sketch_model(source.to_string().as_bytes())
            .unwrap_err()
            .contains("org.example.required"));
        source
            .as_object_mut()
            .unwrap()
            .remove("extensions_required");
        assert_eq!(
            read_sketch_model(source.to_string().as_bytes()).unwrap().1,
            SketchFileFormat::Sketchy
        );
        let malformed = source
            .pointer_mut("/structures/0/levels/0/spaces/0/shape/points")
            .unwrap();
        *malformed = json!([[0, 0], [1, 0]]);
        assert!(read_sketch_model(source.to_string().as_bytes())
            .unwrap_err()
            .contains("too few points"));
        let native = br#"{"pages":[{}],"format":"other"}"#;
        assert_eq!(read_sketch_model(native).unwrap().1, SketchFileFormat::Json);
        let skx = sketch_core::skx::write(&json!({"pages":[{}]}), &Default::default()).unwrap();
        assert_eq!(read_sketch_model(&skx).unwrap().1, SketchFileFormat::Skx);
    }

    #[test]
    fn sketchy_saves_edits_in_place_and_save_as_creates_a_new_workfile() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/sketchy-tests")
            .join(std::process::id().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let dir = dir.canonicalize().unwrap();
        // Even a file without the recommended suffix is detected by its header.
        let source = dir.join("renamed.json");
        let copy = dir.join("copy.sketchy.json");
        let mut source_model = fixture();
        source_model["title"] = json!("Plan: 100 Example Lane");
        source_model["revision"] = json!({"id":"source-revision", "parents":[], "status":"final"});
        source_model["extras"]["org.example.fixture"] = json!({"message":"Keep me"});
        source_model["presentation"]["underlays"] =
            json!([{"id":"image", "level":"ground", "uri":"file:///example.png"}]);
        std::fs::write(&source, source_model.to_string()).unwrap();
        let (mut app, _) = App::boot(1.0);
        let _ = app.open_sketch_path(source.clone());
        assert_eq!(app.pages.page().name, "Level 1");
        assert_eq!(app.file_name, "Plan: 100 Example Lane");
        assert_eq!(app.subject_info.property_address, "100 Example Lane");
        assert_eq!(app.recent[0].path.as_ref(), Some(&source));
        let front = app.drawing.lines[0].id;
        assert!(crate::transform::move_line_endpoint(
            &mut app.drawing,
            &mut app.selection,
            front,
            false,
            Pt::new(20.0, 21.0)
        ));
        app.drawing.lines[0].thickness = Some(8.0);
        app.drawing.polygons[0].name = "Renamed floor".into();
        app.drawing.texts[0].text = "Edited note".into();
        let (_, requests) = crate::file_dialogs::with_save_path(None, || {
            let _ = app.update(Message::SaveFile);
        });
        assert!(requests.is_empty(), "Save writes the opened file directly");
        let saved = std::fs::read(&source).unwrap();
        let raw: Value = serde_json::from_slice(&saved).unwrap();
        assert_eq!(raw["format"], "sketchy");
        assert_eq!(raw["id"], "test-plan");
        assert_eq!(raw["revision"]["parents"], json!(["source-revision"]));
        assert_eq!(raw["history"][0], source_model["revision"]);
        assert_eq!(
            raw["presentation"]["underlays"],
            source_model["presentation"]["underlays"]
        );
        assert_eq!(
            raw["extras"]["org.example.fixture"],
            source_model["extras"]["org.example.fixture"]
        );
        assert_eq!(raw["structures"][0]["levels"][0]["spaces"][0]["id"], "room");
        assert_eq!(
            raw["structures"][0]["levels"][0]["spaces"][0]["name"],
            "Renamed floor"
        );
        near(
            raw["structures"][0]["levels"][0]["walls"][0]["thickness"]
                .as_f64()
                .unwrap(),
            8.0,
        );
        let level = &raw["structures"][0]["levels"][0];
        near(level["walls"][0]["end"][0].as_f64().unwrap(), 132.0);
        near(
            level["spaces"][0]["shape"]["points"][1][0]
                .as_f64()
                .unwrap(),
            132.0,
        );
        assert_eq!(raw["presentation"]["annotations"][0]["text"], "Edited note");
        let _ = app.update(Message::SaveFile);
        let unchanged: Value = serde_json::from_slice(&std::fs::read(&source).unwrap()).unwrap();
        assert_eq!(
            unchanged["revision"], raw["revision"],
            "a repeated Save keeps the revision"
        );
        assert!(std::fs::read(&source).unwrap() == saved);
        let (mut reopened, _) = App::boot(1.0);
        let _ = reopened.open_sketch_path(source.clone());
        let saved_wall = reopened
            .drawing
            .lines
            .iter()
            .find(|line| line.id == front)
            .unwrap();
        assert_eq!(saved_wall.thickness, Some(8.0));
        assert_eq!(reopened.drawing.texts[0].text, "Edited note");
        let (_, requests) = crate::file_dialogs::with_save_path(Some(copy.clone()), || {
            let _ = app.update(Message::SaveFileAs);
        });
        assert_eq!(requests, ["renamed.sketchy.json"]);
        assert!(std::fs::read(&source).unwrap() == saved);
        let fork: Value = serde_json::from_slice(&std::fs::read(&copy).unwrap()).unwrap();
        assert_ne!(fork["id"], raw["id"]);
        assert_eq!(fork["derived_from"]["document"], raw["id"]);
        assert_eq!(fork["derived_from"]["revision"], raw["revision"]["id"]);
        app.drawing.texts[0].text = "Changed copy".into();
        let (_, requests) = crate::file_dialogs::with_save_path(None, || {
            let _ = app.update(Message::SaveFile);
        });
        assert!(requests.is_empty());
        let written: Value = serde_json::from_slice(&std::fs::read(&copy).unwrap()).unwrap();
        assert_eq!(
            written["presentation"]["annotations"][0]["text"],
            "Changed copy"
        );
        assert!(std::fs::read(&source).unwrap() == saved);
        let _ = app.update(Message::RecentOpen(1));
        assert_eq!(app.current_file.as_ref().unwrap().path, source);
    }

    #[test]
    fn sketchy_new_workfile_has_standard_geometry_and_respects_external_edits() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/sketchy-tests")
            .join(std::process::id().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("new-plan.sketchy.json");
        let (mut app, _) = App::boot(1.0);
        app.drawing.add_wall_shape(
            crate::drawing::WallShape::Rectangle,
            WallType::Exterior,
            20.0,
            Some(10.0),
            None,
            Pt::new(0.0, 0.0),
        );
        crate::file_dialogs::with_save_path(Some(file.clone()), || {
            let _ = app.update(Message::SaveFileAs);
        });
        let raw: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        assert_eq!(raw["format"], "sketchy");
        let level = &raw["structures"][0]["levels"][0];
        assert_eq!(level["walls"].as_array().unwrap().len(), 4);
        assert_eq!(
            level["spaces"][0]["shape"]["points"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        let mut external = raw.clone();
        external["structures"][0]["levels"][0]["walls"][0]["thickness"] = json!(0.75);
        let converted = convert(external).unwrap();
        near(
            converted.model["pages"][0]["lines"][0]["thickness"]
                .as_f64()
                .unwrap(),
            9.0,
        );
        let reopened = convert(raw).unwrap();
        assert_eq!(
            reopened.model["pages"][0]["lines"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
    }

    #[test]
    #[ignore = "requires the three upstream examples in SKETCHY_FIXTURE_DIR"]
    fn sketchy_roundtrips_all_three_upstream_examples() {
        let dir = std::path::PathBuf::from(std::env::var_os("SKETCHY_FIXTURE_DIR").unwrap());
        let workfiles = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/sketchy-tests")
            .join(std::process::id().to_string());
        std::fs::create_dir_all(&workfiles).unwrap();
        for (name, counts, metric) in [
            ("single-family", &[5, 2, 2, 1, 1][..], false),
            ("condo-unit", &[7][..], true),
            ("commercial-building", &[4, 5, 5, 1][..], false),
        ] {
            let filename = format!("{name}.sketchy.json");
            let original = std::fs::read(dir.join(&filename)).unwrap();
            let file = workfiles.join(&filename);
            std::fs::write(&file, &original).unwrap();
            let (mut app, _) = App::boot(1.0);
            let _ = app.open_sketch_path(file.clone());
            assert_eq!(
                app.current_file.as_ref().map(|file| file.format),
                Some(SketchFileFormat::Sketchy),
                "{name}: {:?}",
                app.toasts.text()
            );
            assert_eq!(
                app.pages
                    .pages
                    .iter()
                    .map(|p| p.polygons.len())
                    .collect::<Vec<_>>(),
                counts,
                "{name}"
            );
            assert_eq!(app.drawing.options.units.metric, metric, "{name}");
            match name {
                "single-family" => {
                    near(
                        app.drawing.polygons[1].gross,
                        40.0 + 12.5 * std::f64::consts::PI,
                    );
                    app.go_to_page(3);
                    near(app.drawing.polygons[0].gross, 480.0);
                }
                "condo-unit" => {
                    near(app.drawing.polygons[0].gross * 0.3048f64.powi(2), 90.0);
                    near(app.drawing.lines[0].thickness.unwrap() * 0.0254, 0.25);
                }
                "commercial-building" => {
                    app.go_to_page(1);
                    let void = &app.drawing.polygons[4];
                    assert_eq!(void.subtract_target_id, Some(app.drawing.polygons[0].id));
                    near(void.gross, 16.0 * std::f64::consts::PI);
                }
                _ => unreachable!(),
            }
            app.drawing.polygons[0].name = "Edited in Sketch".into();
            let _ = app.update(Message::SaveFile);
            let before: Value = serde_json::from_slice(&original).unwrap();
            let mut after: Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
            assert_eq!(after["id"], before["id"], "{name}");
            assert_eq!(
                after["revision"]["parents"],
                json!([before["revision"]["id"]]),
                "{name}"
            );
            assert_eq!(after["presentation"], before["presentation"], "{name}");
            after["extras"]
                .as_object_mut()
                .unwrap()
                .remove("app.sketchrust.workfile");
            let portable = convert(after).unwrap();
            assert_eq!(
                portable.model["pages"].as_array().unwrap().len(),
                counts.len(),
                "{name}"
            );
            assert!(
                portable.model["pages"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|page| {
                        page["polygons"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|area| area["name"] == "Edited in Sketch")
                    }),
                "{name}"
            );
        }
    }
}
