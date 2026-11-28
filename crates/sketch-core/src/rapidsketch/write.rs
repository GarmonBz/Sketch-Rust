//! Writing a [`Document`] as the object graph RapidSketch expects.
//!
//! `BinaryFormatter` fills objects field by field, so every class here carries
//! exactly the members a RapidSketch 2.8 file declares, in the same order, and
//! the objects are linked the way the program links them: each object lists
//! its parts in `Controls`, parts point back through `Parent`, and joined wall
//! ends point at each other through `ConnectedPoint`.

use super::*;
use crate::nrbf::{Field, Heap, Kind, Schema};

const LIBRARIES: [&str; 3] = [
    "RapidSketch.Components, Version=2.8.0.250, Culture=neutral, PublicKeyToken=c25a65270a3c4742",
    "System.Drawing, Version=2.0.0.0, Culture=neutral, PublicKeyToken=b03f5f7f11d50a3a",
    "System.Windows.Forms, Version=2.0.0.0, Culture=neutral, PublicKeyToken=b77a5c561934e089",
];
const RAPID: usize = 0;
const DRAWING: usize = 1;
const FORMS: usize = 2;

const STRING_LIST: &str = "RapidSketch.AutomatedList`1[[System.String, mscorlib, \
                           Version=2.0.0.0, Culture=neutral, PublicKeyToken=b77a5c561934e089]]";

const fn rapid(name: &'static str) -> Kind {
    Kind::Class(name, RAPID)
}

const fn drawing(name: &'static str) -> Kind {
    Kind::Class(name, DRAWING)
}

macro_rules! schema {
    ($id:ident, $name:expr, $library:expr, [$($member:expr),* $(,)?]) => {
        static $id: Schema = Schema {
            name: $name,
            library: $library,
            members: &[$($member),*],
        };
    };
}

/// A class that starts with the members every `SketchObject` has.
macro_rules! sketch_object {
    ($id:ident, $name:literal, $parent:literal, [$($member:expr),* $(,)?]) => {
        schema!($id, $name, Some(RAPID), [
            ("APP", Kind::Str),
            ("APPVERSION", Kind::Str),
            ("Controls", rapid("RapidSketch.ObjectList")),
            ("Parent", rapid($parent)),
            ("Selected", Kind::Bool),
            $($member),*
        ]);
    };
}

/// The members `Sketch` and its subclass `Symbol` share.
macro_rules! sketch_class {
    ($id:ident, $name:literal, [$($member:expr),* $(,)?]) => {
        sketch_object!($id, $name, "RapidSketch.SketchObject", [
            ("SketchID", Kind::I32),
            ("NextAreaID", Kind::I32),
            ("Areas", rapid("RapidSketch.AreaList")),
            ("MajorGridSpacing", Kind::F32),
            ("MinorGridSpacing", Kind::F32),
            ("UOM", rapid("RapidSketch.UOMs.StandardAndInches")),
            ("SketchInformation", rapid("RapidSketch.SketchInformation")),
            ("PhotoMetricsImage", drawing("System.Drawing.Image")),
            ("PhotoMetricsImageScale", drawing("System.Drawing.SizeF")),
            ("PhotoMetricsImageLocation", drawing("System.Drawing.PointF")),
            ("PhotoMetricsImageTransparency", Kind::I32),
            $($member),*
        ]);
    };
}

sketch_class!(SKETCH, "RapidSketch.Sketch", []);
sketch_class!(
    SYMBOL,
    "RapidSketch.Symbols.Symbol",
    [("Category", Kind::Str), ("Name", Kind::Str)]
);

sketch_object!(
    AREA,
    "RapidSketch.Area",
    "RapidSketch.Sketch",
    [
        ("AreaID", Kind::I32),
        ("FixedFontSizeAreaSize", Kind::Object),
        ("FixedFontSizeForLines", Kind::Object),
        ("FixedFontSizeForAreaName", Kind::Object),
        ("Lines", rapid("RapidSketch.LineList")),
        ("VerticalList", rapid("RapidSketch.VerticalList")),
        ("InternalVerticalList", rapid("RapidSketch.VerticalList")),
        ("DimensionLabel", rapid("RapidSketch.Label")),
        ("DimensionLabelFormat", Kind::Str),
        ("Label", rapid("RapidSketch.Note")),
        ("ShowPerimeter", Kind::Bool),
        ("UOM", rapid("RapidSketch.UOMs.Standard")),
        ("PerimeterUOM", rapid("RapidSketch.UOMs.StandardAndInches")),
        ("Multiplier", Kind::F32),
        ("Hatch", Kind::Bool),
        ("FillColor", drawing("System.Drawing.Color")),
        ("FillColorB", drawing("System.Drawing.Color")),
        ("GradientStyle", Kind::I32),
        ("Category", Kind::Str),
        ("Name", Kind::Str),
        ("Continuous", Kind::Bool),
        ("LivingArea", Kind::Bool),
        ("LineTemplate", rapid("RapidSketch.Line")),
        ("DefaultUOM", Kind::Bool),
        ("HatchStyle", drawing("System.Drawing.Drawing2D.HatchStyle")),
        ("TemplateHasBeenInitialized", Kind::Bool),
    ]
);

sketch_object!(
    FREEFORM_LINE,
    "RapidSketch.FreeformLine",
    "RapidSketch.Sketch",
    [
        ("Start", rapid("RapidSketch.SnappingHandle")),
        ("End", rapid("RapidSketch.SnappingHandle")),
    ]
);

sketch_object!(
    SNAPPING_HANDLE,
    "RapidSketch.SnappingHandle",
    "RapidSketch.FreeformLine",
    [("Location", drawing("System.Drawing.PointF"))]
);

/// `Note` and `Label` declare the same members.
macro_rules! text_class {
    ($id:ident, $name:literal, $parent:literal) => {
        sketch_object!(
            $id,
            $name,
            $parent,
            [
                ("X", Kind::F32),
                ("Y", Kind::F32),
                ("OffsetLocation", drawing("System.Drawing.PointF")),
                ("OffsetAngleInDegrees", Kind::F32),
                (
                    "Alignment",
                    Kind::Class("System.Windows.Forms.HorizontalAlignment", FORMS)
                ),
                ("Visible", Kind::Bool),
                ("Color", drawing("System.Drawing.Color")),
                ("Font", drawing("System.Drawing.Font")),
                ("Rotation", Kind::F32),
                ("Lines", rapid(STRING_LIST)),
            ]
        );
    };
}

text_class!(NOTE, "RapidSketch.Note", "RapidSketch.Sketch");
text_class!(LABEL, "RapidSketch.Label", "RapidSketch.Area");

sketch_object!(
    LINE,
    "RapidSketch.Line",
    "RapidSketch.SketchObject",
    [
        ("ArcHandle", rapid("RapidSketch.ArcHandle")),
        ("Start", rapid("RapidSketch.Point")),
        ("End", rapid("RapidSketch.Point")),
        ("FontSizeOffset", Kind::F32),
        ("DimensionLabelRotationOffset", Kind::F32),
        ("DashStyle", drawing("System.Drawing.Drawing2D.DashStyle")),
        ("Width", Kind::F32),
        ("Color", drawing("System.Drawing.Color")),
        ("ArcHeight", Kind::F64),
        ("Label", rapid("RapidSketch.Label")),
        ("VerticalList", rapid("RapidSketch.VerticalList")),
        ("StartCap", drawing("System.Drawing.Drawing2D.LineCap")),
        ("EndCap", drawing("System.Drawing.Drawing2D.LineCap")),
        ("UOM", rapid("RapidSketch.UOMs.Standard")),
    ]
);

sketch_object!(
    ARC_HANDLE,
    "RapidSketch.ArcHandle",
    "RapidSketch.Line",
    [
        ("Width", Kind::F32),
        ("Location", drawing("System.Drawing.PointF")),
    ]
);

sketch_object!(
    POINT,
    "RapidSketch.Point",
    "RapidSketch.Line",
    [
        ("Location", drawing("System.Drawing.PointF")),
        ("ConnectedPoint", rapid("RapidSketch.Point")),
    ]
);

sketch_object!(
    CONTAINER,
    "RapidSketch.ContainerControl",
    "RapidSketch.Sketch",
    [
        ("Location", drawing("System.Drawing.PointF")),
        ("Graphic", rapid("RapidSketch.Symbols.Symbol")),
        ("VerticalFlip", Kind::Bool),
        ("HorizontalFlip", Kind::Bool),
        ("Rotation", Kind::F32),
        ("TopLeft", rapid("RapidSketch.GrabHandle")),
        ("Top", rapid("RapidSketch.GrabHandle")),
        ("TopRight", rapid("RapidSketch.GrabHandle")),
        ("Left", rapid("RapidSketch.GrabHandle")),
        ("Right", rapid("RapidSketch.GrabHandle")),
        ("BottomLeft", rapid("RapidSketch.GrabHandle")),
        ("Bottom", rapid("RapidSketch.GrabHandle")),
        ("BottomRight", rapid("RapidSketch.GrabHandle")),
        ("Size", drawing("System.Drawing.SizeF")),
        ("ConstrainProportions", Kind::Bool),
    ]
);

schema!(
    OBJECT_LIST,
    "RapidSketch.ObjectList",
    Some(RAPID),
    [("List", Kind::System("System.Collections.ArrayList"))]
);
schema!(
    AREA_LIST,
    "RapidSketch.AreaList",
    Some(RAPID),
    [("List", Kind::System("System.Collections.ArrayList"))]
);
schema!(
    LINE_LIST,
    "RapidSketch.LineList",
    Some(RAPID),
    [("List", Kind::System("System.Collections.ArrayList"))]
);
schema!(
    TEXT_LINES,
    STRING_LIST,
    Some(RAPID),
    [("List", Kind::System("System.Collections.ArrayList"))]
);
schema!(
    ARRAY_LIST,
    "System.Collections.ArrayList",
    None,
    [
        ("_items", Kind::ObjectArray),
        ("_size", Kind::I32),
        ("_version", Kind::I32),
    ]
);
schema!(
    VERTICAL_LIST,
    "RapidSketch.VerticalList",
    Some(RAPID),
    [
        ("List`1+_items", rapid("RapidSketch.VerticalListItem[]")),
        ("List`1+_size", Kind::I32),
        ("List`1+_version", Kind::I32),
    ]
);
schema!(UOM_FEET, "RapidSketch.UOMs.Standard", Some(RAPID), []);
schema!(
    UOM_INCHES,
    "RapidSketch.UOMs.StandardAndInches",
    Some(RAPID),
    []
);
schema!(
    INFORMATION,
    "RapidSketch.SketchInformation",
    Some(RAPID),
    [
        ("CaseNumber", Kind::Str),
        ("FileNumber", Kind::Str),
        ("Address", Kind::Str),
        ("City", Kind::Str),
        ("State", Kind::Str),
        ("Zip", Kind::Str),
        ("Borrower", Kind::Str),
        ("County", Kind::Str),
        ("LenderOrClient", Kind::Str),
        ("LenderOrClientAddress", Kind::Str),
        ("AppraiserName", Kind::Str),
        ("AppraiserAddress", Kind::Str),
    ]
);
schema!(
    POINT_F,
    "System.Drawing.PointF",
    Some(DRAWING),
    [("x", Kind::F32), ("y", Kind::F32)]
);
schema!(
    SIZE_F,
    "System.Drawing.SizeF",
    Some(DRAWING),
    [("width", Kind::F32), ("height", Kind::F32)]
);
schema!(
    COLOR,
    "System.Drawing.Color",
    Some(DRAWING),
    [
        ("name", Kind::Str),
        ("value", Kind::I64),
        ("knownColor", Kind::I16),
        ("state", Kind::I16),
    ]
);
schema!(
    FONT,
    "System.Drawing.Font",
    Some(DRAWING),
    [
        ("Name", Kind::Str),
        ("Size", Kind::F32),
        ("Style", drawing("System.Drawing.FontStyle")),
        ("Unit", drawing("System.Drawing.GraphicsUnit")),
    ]
);

macro_rules! enum_class {
    ($id:ident, $name:literal, $library:expr) => {
        schema!($id, $name, Some($library), [("value__", Kind::I32)]);
    };
}

enum_class!(FONT_STYLE, "System.Drawing.FontStyle", DRAWING);
enum_class!(GRAPHICS_UNIT, "System.Drawing.GraphicsUnit", DRAWING);
enum_class!(DASH_STYLE, "System.Drawing.Drawing2D.DashStyle", DRAWING);
enum_class!(LINE_CAP, "System.Drawing.Drawing2D.LineCap", DRAWING);
enum_class!(HATCH_STYLE, "System.Drawing.Drawing2D.HatchStyle", DRAWING);
enum_class!(ALIGNMENT, "System.Windows.Forms.HorizontalAlignment", FORMS);

/// `HorizontalAlignment.Center`.
const CENTER: i32 = 2;
/// `GraphicsUnit.Point`.
const POINTS: i32 = 3;
/// `KnownColor.Transparent` and `KnownColor.Black`.
const TRANSPARENT: i16 = 27;
const BLACK: i16 = 35;
/// The colour RapidSketch draws walls in.
const WALL_COLOR: u32 = 0xFF00_4080;

fn inline(schema: &'static Schema, fields: Vec<Field>) -> Field {
    Field::Inline(schema, fields)
}

fn enum_value(schema: &'static Schema, value: i32) -> Field {
    inline(schema, vec![Field::I32(value)])
}

fn point_f(p: Pt) -> Field {
    inline(
        &POINT_F,
        vec![Field::F32(p.x as f32), Field::F32(p.y as f32)],
    )
}

fn size_f(width: f64, height: f64) -> Field {
    inline(
        &SIZE_F,
        vec![Field::F32(width as f32), Field::F32(height as f32)],
    )
}

fn argb(value: u32) -> Field {
    inline(
        &COLOR,
        vec![
            Field::Null,
            Field::I64(i64::from(value)),
            Field::I16(0),
            Field::I16(2),
        ],
    )
}

fn named_color(known: i16) -> Field {
    inline(
        &COLOR,
        vec![Field::Null, Field::I64(0), Field::I16(known), Field::I16(1)],
    )
}

/// What a text object labels, which decides how its default position is kept.
#[derive(Clone, Copy, PartialEq)]
enum Role {
    /// A free note on the sketch: no default position.
    Note,
    /// An area's name or size line.
    Area,
    /// A wall's dimension: the default angle follows the wall.
    Wall,
}

/// The wall properties that differ between a sketch wall and a symbol stroke.
#[derive(Clone, Copy)]
struct Pen {
    width: f32,
    color: u32,
    inches: bool,
}

const WALL_PEN: Pen = Pen {
    width: 2.0,
    color: WALL_COLOR,
    inches: true,
};

struct Builder {
    heap: Heap,
    /// The one empty array every `VerticalList` shares.
    no_items: usize,
}

impl Builder {
    fn array_list(&mut self, items: Vec<Field>) -> usize {
        let size = items.len();
        let capacity = if size == 0 {
            0
        } else {
            size.next_power_of_two().max(4)
        };
        let array = self.heap.array(items, capacity);
        self.heap.add(
            &ARRAY_LIST,
            vec![
                Field::Ref(array),
                Field::I32(size as i32),
                Field::I32(size as i32),
            ],
        )
    }

    fn list_of(&mut self, schema: &'static Schema, items: Vec<Field>) -> Field {
        let inner = self.array_list(items);
        Field::Ref(self.heap.add(schema, vec![Field::Ref(inner)]))
    }

    fn controls(&mut self, ids: &[usize]) -> Field {
        let items = ids.iter().map(|&id| Field::Ref(id)).collect();
        self.list_of(&OBJECT_LIST, items)
    }

    fn uom(&mut self, inches: bool) -> Field {
        let schema = if inches { &UOM_INCHES } else { &UOM_FEET };
        Field::Ref(self.heap.add(schema, vec![]))
    }

    fn vertical_list(&mut self) -> Field {
        Field::Ref(self.heap.add(
            &VERTICAL_LIST,
            vec![Field::Ref(self.no_items), Field::I32(0), Field::I32(0)],
        ))
    }

    /// The five members every `SketchObject` starts with.
    fn base(&mut self, children: &[usize], parent: Option<usize>) -> Vec<Field> {
        vec![
            Field::Str("RapidSketch".into()),
            Field::Str(APP_VERSION.into()),
            self.controls(children),
            parent.map_or(Field::Null, Field::Ref),
            Field::Bool(false),
        ]
    }

    fn text(
        &mut self,
        id: usize,
        schema: &'static Schema,
        parent: usize,
        block: &TextBlock,
        role: Role,
    ) {
        let font = self.heap.add(
            &FONT,
            vec![
                Field::Str(block.font.name.clone()),
                Field::F32(block.font.size as f32),
                enum_value(&FONT_STYLE, block.font.style as i32),
                enum_value(&GRAPHICS_UNIT, POINTS),
            ],
        );
        let lines = block
            .text
            .split('\n')
            .map(|l| Field::Str(l.to_string()))
            .collect();
        let lines = self.list_of(&TEXT_LINES, lines);
        let offset = if role == Role::Note {
            Pt::new(0.0, 0.0)
        } else {
            block.at
        };
        let offset_angle = if role == Role::Wall {
            (360.0 - block.rotation).rem_euclid(360.0)
        } else {
            0.0
        };
        let mut fields = self.base(&[], Some(parent));
        fields.extend([
            Field::F32(block.at.x as f32),
            Field::F32(block.at.y as f32),
            point_f(offset),
            Field::F32(offset_angle as f32),
            enum_value(&ALIGNMENT, CENTER),
            Field::Bool(block.visible),
            if block.color == 0xFF00_0000 {
                named_color(BLACK)
            } else {
                argb(block.color)
            },
            Field::Ref(font),
            Field::F32(block.rotation as f32),
            lines,
        ]);
        self.heap.set(id, schema, fields);
    }

    fn point(&mut self, id: usize, line: usize, at: Pt, connected: Option<usize>) {
        let mut fields = self.base(&[], Some(line));
        fields.extend([point_f(at), connected.map_or(Field::Null, Field::Ref)]);
        self.heap.set(id, &POINT, fields);
    }

    /// Fill a reserved `Line`. `ends` are its two reserved points, each with
    /// the neighbouring point it joins.
    fn line(
        &mut self,
        id: usize,
        parent: Option<usize>,
        wall: &Wall,
        ends: [(usize, Option<usize>); 2],
        pen: Pen,
    ) {
        let s = wall.segment;
        let [(start, before), (end, after)] = ends;
        self.point(start, id, s.start, before);
        self.point(end, id, s.end, after);
        let handle = self.heap.reserve();
        let mut fields = self.base(&[], Some(id));
        fields.extend([Field::F32(1.0), point_f(s.apex())]);
        self.heap.set(handle, &ARC_HANDLE, fields);
        let label = self.heap.reserve();
        self.text(label, &LABEL, id, &wall.dimension, Role::Wall);
        let dash = wall.dash.clamp(0, 4) as i32;
        let uom = self.uom(pen.inches);
        let vertical = self.vertical_list();
        let mut fields = self.base(&[start, end, label, handle], parent);
        fields.extend([
            Field::Ref(handle),
            Field::Ref(start),
            Field::Ref(end),
            Field::F32(0.0),
            Field::F32(0.0),
            enum_value(&DASH_STYLE, dash),
            Field::F32(pen.width),
            argb(pen.color),
            Field::F64(s.arc),
            Field::Ref(label),
            vertical,
            enum_value(&LINE_CAP, 0),
            enum_value(&LINE_CAP, 0),
            uom,
        ]);
        self.heap.set(id, &LINE, fields);
    }

    /// Fill a reserved `FreeformLine`, an interior wall.
    fn freeform(&mut self, id: usize, sketch: usize, wall: &Segment) {
        let ends = [wall.start, wall.end].map(|at| {
            let handle = self.heap.reserve();
            let mut fields = self.base(&[], Some(id));
            fields.push(point_f(at));
            self.heap.set(handle, &SNAPPING_HANDLE, fields);
            handle
        });
        let mut fields = self.base(&ends, Some(sketch));
        fields.extend(ends.map(Field::Ref));
        self.heap.set(id, &FREEFORM_LINE, fields);
    }

    /// The walls of one area, joined where their ends meet.
    fn walls(&mut self, parent: usize, walls: &[Wall], closed: bool, pen: Pen) -> Vec<usize> {
        let ids: Vec<_> = walls
            .iter()
            .map(|_| {
                (
                    self.heap.reserve(),
                    self.heap.reserve(),
                    self.heap.reserve(),
                )
            })
            .collect();
        let count = walls.len();
        for (i, wall) in walls.iter().enumerate() {
            let (line, start, end) = ids[i];
            let joined =
                |from: usize, to: usize| near(walls[from].segment.end, walls[to].segment.start);
            let previous = (i + count - 1) % count;
            let next = (i + 1) % count;
            let before = ((i > 0 || closed) && joined(previous, i)).then(|| ids[previous].2);
            let after = ((i + 1 < count || closed) && joined(i, next)).then(|| ids[next].1);
            self.line(
                line,
                Some(parent),
                wall,
                [(start, before), (end, after)],
                pen,
            );
        }
        ids.into_iter().map(|(line, _, _)| line).collect()
    }

    fn area(&mut self, id: usize, parent: usize, number: i32, area: &Area, closed: bool, pen: Pen) {
        let size_label = self.heap.reserve();
        self.text(size_label, &LABEL, id, &area.size_label, Role::Area);
        let lines = self.walls(id, &area.walls, closed, pen);
        let label = self.heap.reserve();
        self.text(label, &NOTE, id, &area.label, Role::Area);
        let mut children = vec![size_label];
        children.extend(&lines);
        children.push(label);
        let line_list = lines.iter().map(|&l| Field::Ref(l)).collect();
        let line_list = self.list_of(&LINE_LIST, line_list);

        // The pen new walls of this area would be drawn with.
        let template = self.heap.reserve();
        let (start, end) = (self.heap.reserve(), self.heap.reserve());
        let origin = Pt::new(0.0, 0.0);
        let blank = Wall {
            segment: Segment {
                start: origin,
                end: origin,
                arc: 0.0,
            },
            dimension: TextBlock {
                text: String::new(),
                at: Pt::new(0.0, -6.0),
                rotation: 0.0,
                visible: true,
                font: Font {
                    name: "Microsoft Sans Serif".into(),
                    size: 8.0,
                    style: 0,
                },
                color: 0xFF00_0000,
            },
            dash: 0,
        };
        self.line(
            template,
            None,
            &blank,
            [(start, None), (end, None)],
            WALL_PEN,
        );

        let vertical = self.vertical_list();
        let internal = self.vertical_list();
        let uom = self.uom(false);
        let perimeter_uom = self.uom(true);
        let multiplier = area.multiplier as f32;
        let mut fields = self.base(&children, Some(parent));
        fields.extend([
            Field::I32(number),
            Field::Null,
            Field::Null,
            Field::Null,
            line_list,
            vertical,
            internal,
            Field::Ref(size_label),
            Field::Str("Normal".into()),
            Field::Ref(label),
            Field::Bool(false),
            uom,
            perimeter_uom,
            Field::F32(multiplier),
            Field::Bool(false),
            argb(area.fill),
            if area.fill >> 24 == 0 {
                named_color(TRANSPARENT)
            } else {
                argb(0x00FF_FFFF)
            },
            Field::I32(0),
            Field::Str("\\".into()),
            Field::Str(area.name.clone()),
            Field::Bool(closed),
            Field::Bool(area.living),
            Field::Ref(template),
            Field::Bool(true),
            enum_value(&HATCH_STYLE, 0),
            Field::Bool(true),
        ]);
        self.heap.set(id, &AREA, fields);
    }

    /// The members `Sketch` and `Symbol` share.
    fn sketch_fields(
        &mut self,
        children: &[usize],
        areas: &[usize],
        next_area: i32,
        information: &Information,
    ) -> Vec<Field> {
        let area_list = areas.iter().map(|&a| Field::Ref(a)).collect();
        let area_list = self.list_of(&AREA_LIST, area_list);
        let text = |s: &String| Field::Str(s.clone());
        let i = information;
        let information = self.heap.add(
            &INFORMATION,
            vec![
                text(&i.case_number),
                text(&i.file_number),
                text(&i.address),
                text(&i.city),
                text(&i.state),
                text(&i.zip),
                text(&i.borrower),
                text(&i.county),
                text(&i.client),
                text(&i.client_address),
                text(&i.appraiser_name),
                text(&i.appraiser_address),
            ],
        );
        let uom = self.uom(true);
        let mut fields = self.base(children, None);
        fields.extend([
            Field::I32(0),
            Field::I32(next_area),
            area_list,
            Field::F32(10.0),
            Field::F32(1.0),
            uom,
            Field::Ref(information),
            Field::Null,
            size_f(1.0, 1.0),
            point_f(Pt::new(0.0, 0.0)),
            Field::I32(255),
        ]);
        fields
    }

    fn symbol(&mut self, container: usize, sketch: usize, symbol: &Symbol) {
        let graphic = self.heap.reserve();
        // The graphic has its own coordinates, starting a little off zero.
        let shift = Pt::new(symbol.width / 2.0 + 6.4, symbol.height / 2.0 + 6.4);
        let mut areas = Vec::new();
        for (number, shape) in symbol.shapes.iter().enumerate() {
            let id = self.heap.reserve();
            let moved = |p: Pt| Pt::new(p.x + shift.x, p.y + shift.y);
            let hidden = |text: &str, size: f64| TextBlock {
                text: text.to_string(),
                at: shift,
                rotation: 0.0,
                visible: false,
                font: Font {
                    name: "Microsoft Sans Serif".into(),
                    size,
                    style: 0,
                },
                color: 0xFF00_0000,
            };
            let area = Area {
                name: "New Area".into(),
                living: true,
                multiplier: 1.0,
                fill: if shape.closed {
                    0x7DFF_FFFF
                } else {
                    0x00FF_FFFF
                },
                walls: shape
                    .segments
                    .iter()
                    .map(|s| Wall {
                        segment: Segment {
                            start: moved(s.start),
                            end: moved(s.end),
                            arc: s.arc,
                        },
                        dimension: hidden("", 6.0),
                        dash: 0,
                    })
                    .collect(),
                label: hidden("New Area", 10.0),
                size_label: hidden("", 8.25),
            };
            let pen = Pen {
                width: SYMBOL_PEN as f32,
                color: 0xFF00_0000,
                inches: false,
            };
            self.area(id, graphic, number as i32 + 1, &area, shape.closed, pen);
            areas.push(id);
        }
        let mut fields = self.sketch_fields(&areas, &areas, 1, &Information::default());
        fields.extend([
            Field::Str(symbol.category.clone()),
            Field::Str(symbol.name.clone()),
        ]);
        self.heap.set(graphic, &SYMBOL, fields);

        let mut fields = self.base(&[], Some(sketch));
        fields.extend([
            point_f(symbol.center),
            Field::Ref(graphic),
            Field::Bool(symbol.flip_v),
            Field::Bool(symbol.flip_h),
            Field::F32(symbol.rotation as f32),
        ]);
        fields.extend(std::iter::repeat_n(Field::Null, 8));
        fields.extend([size_f(symbol.width, symbol.height), Field::Bool(true)]);
        self.heap.set(container, &CONTAINER, fields);
    }
}

/// A run of walls with no area of its own, written as an open, unlabelled one.
fn open_area(walls: &[Wall]) -> Area {
    let at = walls.first().map_or(Pt::new(0.0, 0.0), |w| w.segment.start);
    let hidden = |text: &str, size: f64| TextBlock {
        text: text.to_string(),
        at,
        rotation: 0.0,
        visible: false,
        font: Font {
            name: "Microsoft Sans Serif".into(),
            size,
            style: 0,
        },
        color: 0xFF00_0000,
    };
    Area {
        name: "Walls".into(),
        living: false,
        multiplier: 1.0,
        fill: 0x00FF_FFFF,
        walls: walls.to_vec(),
        label: hidden("Walls", 10.0),
        size_label: hidden("", 8.25),
    }
}

/// Encode a document as a `.rs` / `.acisketch` file.
pub fn write(doc: &Document) -> Vec<u8> {
    let mut heap = Heap::default();
    let sketch = heap.reserve();
    let no_items = heap.empty_array("RapidSketch.VerticalListItem", RAPID);
    let mut b = Builder { heap, no_items };

    let mut children = Vec::new();
    let mut areas = Vec::new();
    let open = doc.open_walls.iter().map(|walls| (open_area(walls), false));
    let all = doc.areas.iter().map(|a| (a.clone(), true)).chain(open);
    for (number, (area, closed)) in all.enumerate() {
        let id = b.heap.reserve();
        b.area(id, sketch, number as i32 + 1, &area, closed, WALL_PEN);
        areas.push(id);
        children.push(id);
    }
    for wall in &doc.interior_walls {
        let id = b.heap.reserve();
        b.freeform(id, sketch, wall);
        children.push(id);
    }
    for note in &doc.notes {
        let id = b.heap.reserve();
        b.text(id, &NOTE, sketch, note, Role::Note);
        children.push(id);
    }
    for symbol in &doc.symbols {
        let id = b.heap.reserve();
        b.symbol(id, sketch, symbol);
        children.push(id);
    }
    let fields = b.sketch_fields(&children, &areas, areas.len() as i32 + 1, &doc.information);
    b.heap.set(sketch, &SKETCH, fields);
    crate::nrbf::write(&b.heap, &LIBRARIES)
}
