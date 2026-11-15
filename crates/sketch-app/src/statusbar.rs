//! Status bar messages and option models.

#[derive(Debug, Clone)]
pub enum StatusMsg {
    CenterSketch,
    CenterFit,
    ZoomFence,
    ToggleGrid,
    /// View › Apply (Dims): outside -> inside -> hidden.
    CycleDimensions,
    ToggleSnap,
    ToggleAreaSnap,
    Pitch(PitchOption),
    ToggleFilter,
    /// Filter checkbox `index` (FILTERS order) toggled.
    Filter(usize),
    TotalsChip,
    ZoomOut,
    ZoomIn,
    ZoomSlider(f64),
    ScaleEdit(String),
    ScaleSubmit,
    North,
    PageDelete,
    PagePrev,
    PageNext,
    PageAdd,
    PageFirst,
    PageLast,
    Underlay(Option<usize>),
    /// The defined-areas select: that area becomes the selection.
    AreaPick(i64),
}

/// An `#area-snap-pitch` option.
#[derive(Debug, Clone, PartialEq)]
pub struct PitchOption {
    pub label: &'static str,
    pub value: f64,
}

impl std::fmt::Display for PitchOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label)
    }
}
