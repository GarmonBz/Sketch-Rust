//! Insert Room Shape dialog + placement (Screen 9).

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PendingShape {
    pub kind: String, // "rectangle" or "lshape"
    pub width: f64,
    pub depth: f64,
    pub cut_width: f64,
    pub cut_depth: f64,
    pub rotation: f64,
    pub define_as_area: bool,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct RoomShapes {
    pub open: bool,
    pub kind: String,
    pub width: String,
    pub depth: String,
    pub cut_width: String,
    pub cut_depth: String,
    pub rotation: String,
    pub define_as_area: bool,
    pub pending: Option<PendingShape>,
}

impl RoomShapes {
    pub fn open(&mut self) {
        self.open = true;
        if self.kind.is_empty() {
            self.kind = "rectangle".into();
        }
        if self.width.is_empty() {
            self.width = "12".into();
        }
        if self.depth.is_empty() {
            self.depth = "10".into();
        }
        if self.cut_width.is_empty() {
            self.cut_width = "4".into();
        }
        if self.cut_depth.is_empty() {
            self.cut_depth = "4".into();
        }
        if self.rotation.is_empty() {
            self.rotation = "0".into();
        }
    }
    pub fn close(&mut self) {
        self.open = false;
    }
}

pub fn parse_len(s: &str) -> f64 {
    s.trim().parse::<f64>().unwrap_or(0.0)
}

/// Generate shape points (relative to origin) for a room shape.
pub fn shape_points(p: &PendingShape) -> Vec<sketch_core::geometry::Pt> {
    let w = p.width;
    let d = p.depth;
    let mut pts = if p.kind == "lshape" {
        let cw = p.cut_width.min(w);
        let cd = p.cut_depth.min(d);
        vec![
            sketch_core::geometry::Pt { x: 0.0, y: 0.0 },
            sketch_core::geometry::Pt { x: w, y: 0.0 },
            sketch_core::geometry::Pt { x: w, y: d - cd },
            sketch_core::geometry::Pt {
                x: w - cw,
                y: d - cd,
            },
            sketch_core::geometry::Pt { x: w - cw, y: d },
            sketch_core::geometry::Pt { x: 0.0, y: d },
        ]
    } else {
        vec![
            sketch_core::geometry::Pt { x: 0.0, y: 0.0 },
            sketch_core::geometry::Pt { x: w, y: 0.0 },
            sketch_core::geometry::Pt { x: w, y: d },
            sketch_core::geometry::Pt { x: 0.0, y: d },
        ]
    };
    // rotate
    if p.rotation != 0.0 {
        let rad = p.rotation * std::f64::consts::PI / 180.0;
        let (c, s) = (rad.cos(), rad.sin());
        for pt in &mut pts {
            let (x, y) = (pt.x, pt.y);
            pt.x = x * c - y * s;
            pt.y = x * s + y * c;
        }
    }
    pts
}
