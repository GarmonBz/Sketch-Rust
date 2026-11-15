//! The area-definition editor (`showAreaDefinitionEditor`).

/// The dialog's state.
#[derive(Debug, Clone)]
pub struct DefEditor {
    pub open: bool,
    /// The code being edited. Empty means "no definition" (the original's
    /// "Select an area definition to edit." notice).
    pub code: String,
    pub name: String,
    pub color: String,
    pub fill_color: String,
    pub fill_style: String,
    pub fill_opacity: f32,
}

impl Default for DefEditor {
    fn default() -> Self {
        DefEditor {
            open: false,
            code: String::new(),
            name: String::new(),
            color: "#718096".into(),
            fill_color: "#718096".into(),
            fill_style: "solid".into(),
            fill_opacity: 10.0,
        }
    }
}

/// A control in the dialog.
#[derive(Debug, Clone)]
pub enum DefMsg {
    Name(String),
    FillStyle(String),
    FillOpacity(f32),
    Save,
    Cancel,
}

impl DefEditor {
    /// Open on one catalogue entry.
    #[cfg(test)]
    pub fn open_on(entry: &sketch_core::geometry::AreaCode) -> DefEditor {
        DefEditor {
            open: true,
            code: entry.code.clone(),
            name: entry.name.clone(),
            color: if entry.color.is_empty() {
                "#718096".to_string()
            } else {
                entry.color.clone()
            },
            fill_color: if entry.fill_color.is_empty() {
                entry.color.clone()
            } else {
                entry.fill_color.clone()
            },
            fill_style: if entry.fill_style.is_empty() {
                "solid".to_string()
            } else {
                entry.fill_style.clone()
            },
            fill_opacity: entry.fill_opacity as f32,
        }
    }

    pub fn update(&mut self, msg: DefMsg) {
        match msg {
            DefMsg::Name(v) => self.name = v,
            DefMsg::FillStyle(v) => self.fill_style = v,
            DefMsg::FillOpacity(v) => self.fill_opacity = v,
            DefMsg::Save | DefMsg::Cancel => {}
        }
    }

    /// Apply the form to the catalogue (`setAreaCodes`).
    pub fn apply(&self, codes: &mut [sketch_core::geometry::AreaCode]) {
        let name = self.name.trim().to_string();
        for entry in codes.iter_mut().filter(|c| c.code == self.code) {
            if !name.is_empty() {
                entry.name = name.clone();
            }
            entry.color = self.color.clone();
            entry.fill_color = self.fill_color.clone();
            entry.fill_style = self.fill_style.clone();
            entry.fill_opacity = f64::from(self.fill_opacity);
        }
    }

    /// The original also renames the polygons of that code and repaints the
    /// lines of their edges. Returns how many polygons were renamed.
    pub fn rename_polygons(
        &self,
        polygons: &mut [sketch_core::model::Polygon],
        lines: &mut [sketch_core::model::SketchLine],
    ) -> usize {
        let name = self.name.trim().to_string();
        let mut touched = 0usize;
        for poly in polygons.iter_mut().filter(|p| p.kind == self.code) {
            if !name.is_empty() {
                poly.name = name.clone();
            }
            for edge in poly.edges.iter() {
                if let Some(line) = lines.iter_mut().find(|l| Some(l.id) == edge.line_id) {
                    line.color = Some(self.color.clone());
                }
            }
            touched += 1;
        }
        touched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sketch_core::geometry::area_codes::default_area_codes;
    use sketch_core::geometry::AreaCode;
    use sketch_core::geometry::{Edge, Pt};
    use sketch_core::model::{Polygon, SketchLine, WallType};

    #[test]
    fn it_opens_on_the_entrys_own_values() {
        let codes = default_area_codes();
        let gla = codes.iter().find(|c| c.code == "GLA1").expect("GLA1");
        let d = DefEditor::open_on(gla);
        assert!(d.open);
        assert_eq!(d.code, "GLA1");
        assert_eq!(d.name, gla.name);
        assert_eq!(d.fill_opacity, gla.fill_opacity as f32);
    }

    #[test]
    fn saving_rewrites_only_that_code() {
        let mut codes = default_area_codes();
        let mut d = DefEditor::open_on(codes.iter().find(|c| c.code == "GLA1").expect("GLA1"));
        d.update(DefMsg::Name(" Main Level ".into()));
        d.update(DefMsg::FillStyle("crosshatch".into()));
        d.update(DefMsg::FillOpacity(40.0));
        d.apply(&mut codes);
        let gla = codes.iter().find(|c| c.code == "GLA1").expect("GLA1");
        assert_eq!(gla.name, "Main Level", "trimmed");
        assert_eq!(gla.fill_style, "crosshatch");
        assert_eq!(gla.fill_opacity, 40.0);
        // The other codes keep their names.
        assert_eq!(
            codes.iter().find(|c| c.code == "GLA2").expect("GLA2").name,
            default_area_codes()
                .iter()
                .find(|c| c.code == "GLA2")
                .expect("GLA2")
                .name
        );
    }

    #[test]
    fn saving_renames_the_polygons_and_repaints_their_edges() {
        let codes = default_area_codes();
        let mut d = DefEditor::open_on(codes.iter().find(|c| c.code == "GLA1").expect("GLA1"));
        d.update(DefMsg::Name("Main Level".into()));
        d.color = "#FF0000".to_string();
        let mut polygons = vec![
            Polygon {
                id: 1,
                points: vec![],
                edges: vec![Edge {
                    line_id: Some(10),
                    bow_angle: 0.0,
                }],
                kind: "GLA1".into(),
                name: "Old".into(),
                ..sample_polygon()
            },
            Polygon {
                id: 2,
                points: vec![],
                edges: vec![Edge {
                    line_id: Some(11),
                    bow_angle: 0.0,
                }],
                kind: "GAR".into(),
                name: "Garage".into(),
                ..sample_polygon()
            },
        ];
        let mut lines = vec![
            SketchLine {
                id: 10,
                color: Some("#111111".into()),
                ..sample_line()
            },
            SketchLine {
                id: 11,
                color: Some("#222222".into()),
                ..sample_line()
            },
        ];
        let touched = d.rename_polygons(&mut polygons, &mut lines);
        assert_eq!(touched, 1, "only the GLA1 polygon");
        assert_eq!(polygons[0].name, "Main Level");
        assert_eq!(
            lines[0].color.as_deref(),
            Some("#FF0000"),
            "its edge line was repainted"
        );
        assert_eq!(polygons[1].name, "Garage", "the GAR polygon is untouched");
        assert_eq!(
            lines[1].color.as_deref(),
            Some("#222222"),
            "and so is its line"
        );
    }

    fn sample_polygon() -> Polygon {
        Polygon {
            id: 0,
            points: vec![],
            edges: vec![],
            kind: "UND".into(),
            name: String::new(),
            gross: 0.0,
            area: 0.0,
            factor_override: None,
            factor: 1.0,
            measure_to: "center".into(),
            subtract_mode: "auto".into(),
            subtract_target_id: None,
            auto_post: Default::default(),
            attributes: Default::default(),
            layer_id: None,
            fill_color: None,
            fill_pattern: None,
            fill_opacity: None,
            line_color: None,
            line_style: None,
            line_thickness: None,
            label_rotation: None,
            area_size_rotation: None,
            label_position: None,
            area_size_position: None,
            label_scale: None,
            area_size_scale: None,
            label_font_size: None,
            label_font_family: None,
            label_bold: None,
            label_italic: None,
            label_underline: None,
            label_color: None,
        }
    }
    fn sample_line() -> SketchLine {
        SketchLine {
            id: 0,
            start: Pt::new(0.0, 0.0),
            end: Pt::new(1.0, 1.0),
            kind: WallType::Exterior,
            color: None,
            line_thickness: None,
            thickness: None,
            dimension_hidden: false,
            layer_id: None,
            bow_angle: 0.0,
            arc_height: 0.0,
            line_style: None,
            dimension_position: None,
            dimension_anchor: None,
        }
    }
    #[test]
    fn the_codes_default_fill_style_is_solid_when_empty() {
        let mut entry: AreaCode = sketch_core::geometry::AreaCode {
            code: "X".into(),
            name: "Y".into(),
            ..default_area_codes()
                .into_iter()
                .find(|c| c.code == "OTH")
                .expect("OTH")
        };
        entry.code = "X".into();
        entry.color = String::new();
        entry.fill_color = String::new();
        entry.fill_style = String::new();
        let d = DefEditor::open_on(&entry);
        assert_eq!(d.fill_style, "solid");
        assert_eq!(
            d.fill_color, "",
            "an empty fill falls back to the line colour"
        );
        assert_eq!(
            d.color, "#718096",
            "and an empty line colour to the default"
        );
    }
}
