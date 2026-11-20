//! The Area Properties dialog state and controls.

use crate::area_props::AreaProperties;

/// The Measure To options, in the original's order.
pub(crate) const MEASURE: [&str; 3] = ["Centerline", "Interior Face", "Exterior Face"];
fn measure_value(label: &str) -> &'static str {
    match label {
        "Interior Face" => "interior",
        "Exterior Face" => "exterior",
        _ => "center",
    }
}
pub(crate) fn measure_label(value: &str) -> &'static str {
    match value {
        "interior" => "Interior Face",
        "exterior" => "Exterior Face",
        _ => "Centerline",
    }
}

/// The dialog's own state: the values being edited plus the raw text fields.
#[derive(Debug, Clone, Default)]
pub struct AreaDialog {
    pub open: bool,
    /// The payload Apply sends.
    pub props: AreaProperties,
    /// The Factor field's text (`''` is factor 0 in the original).
    pub factor_text: String,
    /// The Suffix field's text.
    pub suffix_text: String,
    /// The area-code choices, `"CODE - Name"`.
    pub codes: Vec<String>,
    /// The Positive/Negative radio. Display only: the original's
    /// `updateSelectedAreaProperties` never reads it (the sign lives on the
    /// area code), so Apply ignores it — reproduced, not corrected.
    pub positive: bool,
}

/// What a control in the dialog asks for.
#[derive(Debug, Clone)]
pub enum AreaDialogMsg {
    Code(String),
    Name(String),
    Factor(String),
    Positive(bool),
    AutoSubtract(String),
    Measure(String),
    Post(PostSwitch, bool),
    Suffix(String),
    Apply,
    Cancel,
}

/// Which auto-post switch a checkbox toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostSwitch {
    Code,
    Name,
    Dims,
    Calcs,
    Base,
    Factor,
}

/// The name part of the `"CODE - Name"` entry for `code`.
fn code_name<'a>(codes: &'a [String], code: &str) -> Option<&'a str> {
    let prefix = format!("{code} - ");
    codes.iter().find_map(|s| s.strip_prefix(prefix.as_str()))
}

impl AreaDialog {
    /// Open with the first selected area's values (`showSelectedAreaProperties`).
    pub fn open_with(props: AreaProperties, codes: Vec<String>) -> AreaDialog {
        let factor_text = props
            .factor
            .map(|f| {
                if (f - f.round()).abs() < 1e-9 {
                    format!("{}", f.round() as i64)
                } else {
                    format!("{f}")
                }
            })
            .unwrap_or_default();
        let suffix_text = props
            .auto_post
            .as_ref()
            .map(|p| p.suffix.clone())
            .unwrap_or_else(|| "sf".into());
        AreaDialog {
            open: true,
            props,
            factor_text,
            suffix_text,
            codes,
            positive: true,
        }
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Apply a control's change to the pending payload.
    pub fn update(&mut self, msg: AreaDialogMsg) {
        match msg {
            AreaDialogMsg::Code(code) => {
                // A name that is only a default (blank, "Undefined Area", or
                // the old type's own name) follows the type; a custom one stays.
                let old_default = code_name(&self.codes, &self.props.code);
                let follows_type = self.props.name.trim().is_empty()
                    || self.props.name == "Undefined Area"
                    || old_default == Some(self.props.name.as_str());
                let new_name = code_name(&self.codes, &code).map(str::to_owned);
                self.props.code = code;
                if let (true, Some(name)) = (follows_type, new_name) {
                    self.props.name = name;
                }
            }
            AreaDialogMsg::Name(name) => self.props.name = name,
            AreaDialogMsg::Factor(text) => {
                self.factor_text = text.clone();
                // The original coerces the field with `Number(...)`: an empty
                // field is 0, junk is NaN (which clears the override).
                self.props.factor = if text.trim().is_empty() {
                    Some(0.0)
                } else {
                    text.trim().parse::<f64>().ok()
                };
            }
            AreaDialogMsg::Positive(positive) => self.positive = positive,
            AreaDialogMsg::AutoSubtract(label) => self.props.auto_subtract = label == "Automatic",
            AreaDialogMsg::Measure(label) => {
                self.props.measure_to = Some(measure_value(&label).to_string())
            }
            AreaDialogMsg::Post(which, on) => {
                let post = self.props.auto_post.get_or_insert_with(Default::default);
                match which {
                    PostSwitch::Code => post.code = on,
                    PostSwitch::Name => post.name = on,
                    PostSwitch::Dims => post.dims = on,
                    PostSwitch::Calcs => post.calcs = on,
                    PostSwitch::Base => post.base = on,
                    PostSwitch::Factor => post.factor = on,
                }
            }
            AreaDialogMsg::Suffix(text) => {
                self.suffix_text = text.clone();
                let post = self.props.auto_post.get_or_insert_with(Default::default);
                post.suffix = text;
            }
            AreaDialogMsg::Apply | AreaDialogMsg::Cancel => {}
        }
    }

    /// The payload Apply sends.
    pub fn payload(&self) -> AreaProperties {
        let mut props = self.props.clone();
        if let Some(post) = props.auto_post.as_mut() {
            post.suffix = self.suffix_text.clone();
        }
        props
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dialog() -> AreaDialog {
        AreaDialog::open_with(
            AreaProperties {
                code: "GLA1".into(),
                name: "First Floor".into(),
                factor: Some(1.0),
                auto_subtract: true,
                measure_to: Some("center".into()),
                auto_post: Some(Default::default()),
            },
            vec!["GLA1 - First Floor".into(), "GLA2 - Second Floor".into()],
        )
    }

    #[test]
    fn the_dialog_opens_with_the_areas_values() {
        let d = dialog();
        assert!(d.open);
        assert_eq!(d.props.code, "GLA1");
        assert_eq!(d.factor_text, "1", "a whole factor prints without decimals");
        assert_eq!(d.suffix_text, "sf");
        assert!(d.positive);
    }

    #[test]
    fn an_empty_factor_field_is_zero_as_in_the_original() {
        let mut d = dialog();
        d.update(AreaDialogMsg::Factor(String::new()));
        assert_eq!(d.props.factor, Some(0.0), "Number('') === 0");
        d.update(AreaDialogMsg::Factor("abc".into()));
        assert_eq!(d.props.factor, None, "junk clears the override");
        d.update(AreaDialogMsg::Factor("1.25".into()));
        assert_eq!(d.props.factor, Some(1.25));
        assert_eq!(d.payload().factor, Some(1.25));
    }

    #[test]
    fn changing_the_type_renames_a_default_name_but_keeps_a_custom_one() {
        let mut d = dialog();
        d.update(AreaDialogMsg::Code("GLA2".into()));
        assert_eq!(d.props.name, "Second Floor", "old type's own name follows");

        let mut d = dialog();
        d.props.name = "Undefined Area".into();
        d.update(AreaDialogMsg::Code("GLA2".into()));
        assert_eq!(d.props.name, "Second Floor");

        let mut d = dialog();
        d.props.name = "Master suite".into();
        d.update(AreaDialogMsg::Code("GLA2".into()));
        assert_eq!(d.props.name, "Master suite", "a custom name stays");
    }

    #[test]
    fn the_controls_reach_the_payload() {
        let mut d = dialog();
        d.update(AreaDialogMsg::Code("GLA2".into()));
        d.update(AreaDialogMsg::Name("Second Floor".into()));
        d.update(AreaDialogMsg::AutoSubtract("None".into()));
        d.update(AreaDialogMsg::Measure("Interior Face".into()));
        d.update(AreaDialogMsg::Post(PostSwitch::Code, true));
        d.update(AreaDialogMsg::Post(PostSwitch::Dims, false));
        d.update(AreaDialogMsg::Suffix("SF".into()));
        let p = d.payload();
        assert_eq!(p.code, "GLA2");
        assert_eq!(p.name, "Second Floor");
        assert!(!p.auto_subtract, "'None' is autoSubtract false");
        assert_eq!(p.measure_to.as_deref(), Some("interior"));
        let post = p.auto_post.expect("autoPost");
        assert!(post.code);
        assert!(!post.dims);
        assert_eq!(post.suffix, "SF");
        // The radio is display-only in the original.
        d.update(AreaDialogMsg::Positive(false));
        assert!(!d.positive);
        assert_eq!(d.payload().code, "GLA2");
    }
}
