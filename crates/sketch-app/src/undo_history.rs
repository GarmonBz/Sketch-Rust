//! The undo history popup state, anchored under the calling Undo button.

/// Where the popup is open and under which anchor.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HistoryView {
    /// The anchor's top-left; the popup opens under it.
    pub open: Option<(f32, f32)>,
}

impl HistoryView {
    pub fn open_at(&mut self, x: f32, y: f32) {
        self.open = Some((x, y));
    }

    pub fn close(&mut self) {
        self.open = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_and_close_update_anchor() {
        let mut v = HistoryView::default();
        assert_eq!(v.open, None);
        v.open_at(104.0, 29.0);
        assert_eq!(v.open, Some((104.0, 29.0)));
        v.close();
        assert_eq!(v.open, None);
    }
}
