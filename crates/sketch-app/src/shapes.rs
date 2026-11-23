//! The wall-shape menu state and prompt chain (Rectangle / Circle / Polygon).

use sketch_core::model::WallType;

use crate::drawing::WallShape;

/// Which prompt the chain is asking for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prompt {
    Width,
    Height,
    Sides,
}

impl Prompt {
    /// `shape === 'circle' ? 'Diameter' : 'Width'`.
    pub fn title(self, shape: WallShape) -> &'static str {
        match self {
            Prompt::Width if shape == WallShape::Circle => "Diameter in feet",
            Prompt::Width => "Width in feet",
            Prompt::Height => "Height in feet",
            Prompt::Sides => "Number of sides",
        }
    }

    pub fn default_value(self, shape: WallShape) -> &'static str {
        match self {
            Prompt::Width | Prompt::Height => "20",
            Prompt::Sides if shape == WallShape::Circle => "36",
            Prompt::Sides => "8",
        }
    }
}

/// A shape's size as it is being entered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pending {
    pub shape: WallShape,
    pub kind: WallType,
    pub width: f64,
    pub height: Option<f64>,
    /// The typed side count (polygon only).
    pub sides: Option<f64>,
    pub prompt: Prompt,
}

#[derive(Debug, Clone, Default)]
pub struct Shapes {
    pub open: Option<(WallType, iced::Vector)>,
    pub pending: Option<Pending>,
}

#[derive(Debug, Clone)]
pub enum ShapesMsg {
    Open(WallType, f32),
    Line(WallType),
    Choose(WallShape),
    Dismiss,
    Value(String),
}

impl Shapes {
    pub fn open_at(&mut self, kind: WallType, anchor_y: f32) {
        self.open = Some((kind, iced::Vector::new(8.0, anchor_y + 34.0)));
    }

    pub fn dismiss(&mut self) {
        self.open = None;
        self.pending = None;
    }

    /// A shape row: start the prompt chain (Line is handled by the caller,
    /// which just sets the mode).
    pub fn choose(&mut self, shape: WallShape, kind: WallType) {
        self.open = None;
        self.pending = Some(Pending {
            shape,
            kind,
            width: 20.0,
            height: None,
            sides: None,
            prompt: Prompt::Width,
        });
    }

    /// The prompt on screen (`title`, `default`).
    pub fn prompt(&self) -> Option<(&'static str, &'static str)> {
        self.pending
            .as_ref()
            .map(|p| (p.prompt.title(p.shape), p.prompt.default_value(p.shape)))
    }

    /// A prompt's OK: advance the chain, and on the last step report the
    /// shape to add. `Number(await promptDialog(...))` — a falsy entry
    /// abandons the shape.
    pub fn submit(&mut self, value: &str) -> Option<Pending> {
        let pending = self.pending?;
        let number = value
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|n| *n != 0.0 && n.is_finite());
        let Some(number) = number else {
            self.pending = None;
            return None;
        };
        match pending.prompt {
            // A circle takes its diameter once: it is also the height, and
            // its side count is fixed at 36.
            Prompt::Width if pending.shape == WallShape::Circle => {
                self.pending = None;
                Some(Pending {
                    shape: pending.shape,
                    kind: pending.kind,
                    width: number,
                    height: Some(number),
                    sides: None,
                    prompt: Prompt::Sides,
                })
            }
            Prompt::Width => {
                self.pending = Some(Pending {
                    width: number,
                    prompt: Prompt::Height,
                    ..pending
                });
                None
            }
            // Only a polygon asks for its side count; a rectangle finishes
            // after the height.
            Prompt::Height if pending.shape == WallShape::Polygon => {
                self.pending = Some(Pending {
                    height: Some(number),
                    prompt: Prompt::Sides,
                    ..pending
                });
                None
            }
            Prompt::Height => {
                self.pending = None;
                Some(Pending {
                    height: Some(number),
                    ..pending
                })
            }
            Prompt::Sides => {
                self.pending = None;
                Some(Pending {
                    height: Some(pending.height.unwrap_or(pending.width)),
                    sides: Some(number),
                    ..pending
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prompt_chain_matches_the_original() {
        // showWallShapeMenu: Rectangle asks Width -> Height (both default
        // 20); Circle asks Diameter once (its height is the diameter);
        // Polygon asks Width -> Height -> Number of sides (8).
        let mut s = Shapes::default();
        s.choose(WallShape::Rectangle, WallType::Exterior);
        assert_eq!(s.prompt(), Some(("Width in feet", "20")));
        assert!(s.submit("20").is_none());
        assert_eq!(s.prompt(), Some(("Height in feet", "20")));
        let done = s
            .submit("20")
            .expect("a rectangle completes after the height");
        assert_eq!(
            (done.shape, done.kind, done.width, done.height),
            (WallShape::Rectangle, WallType::Exterior, 20.0, Some(20.0))
        );

        let mut c = Shapes::default();
        c.choose(WallShape::Circle, WallType::Interior);
        assert_eq!(c.prompt(), Some(("Diameter in feet", "20")));
        let done = c.submit("10").expect("a circle completes after one prompt");
        assert_eq!(
            (done.width, done.height),
            (10.0, Some(10.0)),
            "the diameter is the height"
        );

        // A polygon is the only shape that asks for sides.
        let mut p = Shapes::default();
        p.choose(WallShape::Polygon, WallType::Exterior);
        assert!(p.submit("12").is_none());
        assert!(
            p.submit("8").is_none(),
            "the polygon continues to the sides"
        );
        assert_eq!(p.prompt(), Some(("Number of sides", "8")));
        let done = p.submit("6").expect("polygon completes");
        assert_eq!(
            (done.width, done.height, done.sides),
            (12.0, Some(8.0), Some(6.0))
        );

        // A falsy or unparsable entry abandons the shape.
        let mut a = Shapes::default();
        a.choose(WallShape::Rectangle, WallType::Exterior);
        assert!(a.submit("0").is_none() && a.prompt().is_none());
        let mut b = Shapes::default();
        b.choose(WallShape::Rectangle, WallType::Exterior);
        assert!(b.submit("").is_none() && b.prompt().is_none());
    }
}
