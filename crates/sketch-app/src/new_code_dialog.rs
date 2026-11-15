//! The New Area Code dialog (`showAddAreaCode`).
//!
//! The create logic is the original's form submit, including its exact notices.

pub const FILL_STYLES: [&str; 5] = ["solid", "none", "diagonal", "crosshatch", "dots"];
pub fn style_label(value: &str) -> &'static str {
    match value {
        "none" => "None",
        "diagonal" => "Diagonal",
        "crosshatch" => "Crosshatch",
        "dots" => "Dots",
        _ => "Solid",
    }
}
pub fn style_value(label: &str) -> &'static str {
    match label {
        "None" => "none",
        "Diagonal" => "diagonal",
        "Crosshatch" => "crosshatch",
        "Dots" => "dots",
        _ => "solid",
    }
}

/// The dialog's state.
#[derive(Debug, Clone)]
pub struct NewCodeDialog {
    pub open: bool,
    pub code: String,
    pub name: String,
    pub description: String,
    pub group: String,
    pub multiplier_text: String,
    pub include_gla: bool,
    pub color: String,
    pub fill_color: String,
    pub fill_style: String,
    pub fill_opacity: f32,
}

impl Default for NewCodeDialog {
    fn default() -> Self {
        NewCodeDialog {
            open: false,
            code: String::new(),
            name: String::new(),
            description: String::new(),
            group: "GLA".into(),
            multiplier_text: "1".into(),
            include_gla: false,
            color: "#718096".into(),
            fill_color: "#718096".into(),
            fill_style: "solid".into(),
            fill_opacity: 10.0,
        }
    }
}

/// A control in the dialog.
#[derive(Debug, Clone)]
pub enum NewCodeMsg {
    Code(String),
    Name(String),
    Description(String),
    Group(String),
    Multiplier(String),
    IncludeGla(bool),
    FillStyle(String),
    FillOpacity(f32),
    Create,
    Cancel,
}

impl NewCodeDialog {
    /// Open with the first group preselected (`preferredGroup || groups[0]`).
    #[cfg(test)]
    pub fn open_with(group: &str) -> NewCodeDialog {
        NewCodeDialog {
            open: true,
            group: if group.is_empty() { "OTH" } else { group }.to_string(),
            ..NewCodeDialog::default()
        }
    }

    pub fn update(&mut self, msg: NewCodeMsg) {
        match msg {
            NewCodeMsg::Code(v) => self.code = v,
            NewCodeMsg::Name(v) => self.name = v,
            NewCodeMsg::Description(v) => self.description = v,
            NewCodeMsg::Group(v) => self.group = v,
            NewCodeMsg::Multiplier(v) => self.multiplier_text = v,
            NewCodeMsg::IncludeGla(v) => self.include_gla = v,
            NewCodeMsg::FillStyle(v) => self.fill_style = v,
            NewCodeMsg::FillOpacity(v) => self.fill_opacity = v,
            NewCodeMsg::Create | NewCodeMsg::Cancel => {}
        }
    }

    /// The entry this form would create, or the original's notice.
    pub fn create(
        &self,
        existing: &[sketch_core::geometry::AreaCode],
    ) -> Result<sketch_core::geometry::AreaCode, String> {
        let code = self.code.trim().to_uppercase();
        let name = self.name.trim().to_string();
        if code.is_empty() {
            return Err("Code is required.".into());
        }
        if name.is_empty() {
            return Err("Name is required.".into());
        }
        if existing.iter().any(|c| c.code == code) {
            return Err(format!("Area code \"{code}\" already exists."));
        }
        let description = {
            let d = self.description.trim();
            if d.is_empty() {
                name.clone()
            } else {
                d.to_string()
            }
        };
        let group = {
            let g = self.group.trim().to_uppercase();
            if g.is_empty() {
                "OTH".to_string()
            } else {
                g
            }
        };
        // `Math.max(0, Number(x) || 1)`.
        let multiplier = self
            .multiplier_text
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| *v != 0.0 && v.is_finite())
            .unwrap_or(1.0)
            .max(0.0);
        let entry = sketch_core::geometry::AreaCode {
            code,
            name,
            description,
            group,
            multiplier,
            positive: true,
            include_in_gla: self.include_gla,
            color: self.color.clone(),
            fill: format!("rgba(113,128,150,{:.2})", self.fill_opacity / 100.0),
            fill_color: self.fill_color.clone(),
            fill_style: self.fill_style.clone(),
            fill_opacity: self.fill_opacity as f64,
        };
        Ok(sketch_core::geometry::area_codes::normalize_area_code(
            entry,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sketch_core::geometry::area_codes::default_area_codes;

    #[test]
    fn a_new_code_is_normalized_and_appended() {
        let existing = default_area_codes();
        let mut d = NewCodeDialog::open_with("GLA");
        d.update(NewCodeMsg::Code("  gla5 ".into()));
        d.update(NewCodeMsg::Name(" Fifth Floor ".into()));
        d.update(NewCodeMsg::IncludeGla(true));
        let entry = d.create(&existing).expect("created");
        assert_eq!(entry.code, "GLA5", "upper-cased and trimmed");
        assert_eq!(entry.name, "Fifth Floor");
        assert_eq!(
            entry.description, "Fifth Floor",
            "description defaults to the name"
        );
        assert_eq!(entry.group, "GLA");
        assert_eq!(entry.multiplier, 1.0);
        assert!(entry.include_in_gla);
    }

    #[test]
    fn the_originals_notices_come_back_verbatim() {
        let existing = default_area_codes();
        let mut d = NewCodeDialog::open_with("GLA");
        d.update(NewCodeMsg::Name("Fifth Floor".into()));
        assert_eq!(d.create(&existing).unwrap_err(), "Code is required.");
        d.update(NewCodeMsg::Code("GLA5".into()));
        d.update(NewCodeMsg::Name("  ".into()));
        assert_eq!(d.create(&existing).unwrap_err(), "Name is required.");
        d.update(NewCodeMsg::Name("Fifth".into()));
        d.update(NewCodeMsg::Code("gla1".into()));
        assert_eq!(
            d.create(&existing).unwrap_err(),
            "Area code \"GLA1\" already exists."
        );
    }

    #[test]
    fn the_multiplier_follows_the_js_coercion() {
        let existing = default_area_codes();
        let mut d = NewCodeDialog::open_with("OTH");
        d.update(NewCodeMsg::Code("X1".into()));
        d.update(NewCodeMsg::Name("Test".into()));
        // `Number(x) || 1`, so junk and 0 become 1, and negatives clamp to 0.
        for (text, want) in [
            ("2.5", 2.5),
            ("", 1.0),
            ("abc", 1.0),
            ("0", 1.0),
            ("-3", 0.0),
        ] {
            d.update(NewCodeMsg::Multiplier(text.into()));
            let entry = d.create(&existing).expect("created");
            assert_eq!(entry.multiplier, want, "multiplier {text:?}");
        }
    }

    #[test]
    fn an_empty_group_becomes_oth() {
        let existing = default_area_codes();
        let mut d = NewCodeDialog::open_with("");
        d.update(NewCodeMsg::Code("X2".into()));
        d.update(NewCodeMsg::Name("Test".into()));
        d.update(NewCodeMsg::Group("  ".into()));
        let entry = d.create(&existing).expect("created");
        assert_eq!(entry.group, "OTH");
    }
}
