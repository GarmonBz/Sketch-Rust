//! The Area Attributes dialog (`showSelectedAreaAttributes`
//! + `updateSelectedAreaAttributes`).

/// The four attributes, in the original's order: (key, label).
pub const ATTRS: [(&str, &str); 4] = [
    ("planRefNumber", "Plan Ref Number"),
    ("floorCode", "Floor Code"),
    ("floorCodeText", "FloorCodeText"),
    ("yearBuilt", "Year Built"),
];

/// The dialog's state.
#[derive(Debug, Clone, Default)]
pub struct AttrDialog {
    pub open: bool,
    /// `editable` false is the read-only view.
    pub editable: bool,
    /// The four values, in `ATTRS` order.
    pub values: [String; 4],
}

/// A control in the attributes dialog.
#[derive(Debug, Clone)]
pub enum AttrMsg {
    /// The index into `ATTRS`.
    Value(usize, String),
    Ok,
    Cancel,
}

impl AttrDialog {
    /// Open with the first selected area's attributes.
    pub fn open_with(
        attributes: &std::collections::BTreeMap<String, String>,
        editable: bool,
    ) -> AttrDialog {
        let mut values: [String; 4] = Default::default();
        for (i, (key, _)) in ATTRS.iter().enumerate() {
            values[i] = attributes.get(*key).cloned().unwrap_or_default();
        }
        AttrDialog {
            open: true,
            editable,
            values,
        }
    }

    pub fn update(&mut self, msg: AttrMsg) {
        if let AttrMsg::Value(i, v) = msg {
            if i < 4 {
                self.values[i] = v;
            }
        }
    }

    /// `updateSelectedAreaAttributes(Object.fromEntries(data))`.
    pub fn payload(&self) -> Vec<(String, String)> {
        ATTRS
            .iter()
            .enumerate()
            .map(|(i, (key, _))| ((*key).to_string(), self.values[i].clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dialog_opens_with_the_areas_attributes() {
        let mut attrs = std::collections::BTreeMap::new();
        attrs.insert("planRefNumber".to_string(), "A-1".to_string());
        attrs.insert("yearBuilt".to_string(), "1998".to_string());
        let d = AttrDialog::open_with(&attrs, true);
        assert!(d.open && d.editable);
        assert_eq!(d.values[0], "A-1");
        assert_eq!(d.values[1], "", "a missing attribute is empty");
        assert_eq!(d.values[3], "1998");
        // The payload carries all four keys, in the original's order.
        let p = d.payload();
        assert_eq!(p.len(), 4);
        assert_eq!(p[0].0, "planRefNumber");
        assert_eq!(p[3].0, "yearBuilt");
        assert_eq!(p[3].1, "1998");
    }

    #[test]
    fn edits_land_in_the_payload() {
        let mut d = AttrDialog::open_with(&Default::default(), true);
        d.update(AttrMsg::Value(1, "2".into()));
        d.update(AttrMsg::Value(2, "Second".into()));
        let p = d.payload();
        assert_eq!(p[1].1, "2");
        assert_eq!(p[2].1, "Second");
        // OK and Cancel do not change the values.
        d.update(AttrMsg::Ok);
        assert_eq!(d.payload()[1].1, "2");
    }
}
