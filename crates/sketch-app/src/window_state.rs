//! Where the window was last left (`window.json` in the persist store): its
//! restored ("normal") position and size, and whether it was maximized.
//!
//! Units follow iced's window events: sizes are CSS px (the app's layout
//! unit); positions are the `Moved` event's value, which is the OS position
//! divided by the OS scale and the app's own scale.

use iced::{Point, Size};
use serde::{Deserialize, Serialize};

const FILE: &str = "window.json";

/// The smallest window the layout still works in (CSS px): the ribbon's
/// Home tab and one dock panel stay usable.
pub const MIN_W: f32 = 800.0;
pub const MIN_H: f32 = 560.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    /// `None` until the window has reported a position: it opens centred.
    pub pos: Option<(f32, f32)>,
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
}

impl Placement {
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// The position to hand `window::Position::Specific`, which takes
    /// OS-logical px (no app scale).
    pub fn restore_position(&self, ui_scale: f32) -> Option<Point> {
        self.pos
            .map(|(x, y)| Point::new(x * ui_scale, y * ui_scale))
    }

    /// A stored placement made safe to use: a sane size, and a position
    /// only if its title bar would land on a monitor that still exists.
    fn sanitized(mut self, ui_scale: f32, system_scale: f32) -> Option<Self> {
        if !self.width.is_finite() || !self.height.is_finite() {
            return None;
        }
        self.width = self.width.clamp(MIN_W, 16_000.0);
        self.height = self.height.clamp(MIN_H, 16_000.0);
        if let Some((x, y)) = self.pos {
            let scale = ui_scale * system_scale;
            // A point inside the title bar, in physical px.
            let visible = x.is_finite()
                && y.is_finite()
                && crate::platform::point_on_screen(
                    ((x + 60.0) * scale) as i32,
                    ((y + 15.0) * scale) as i32,
                );
            if !visible {
                self.pos = None;
            }
        }
        Some(self)
    }
}

/// The stored placement, or `None` (no store, nothing stored, not valid).
pub fn load(ui_scale: f32, system_scale: f32) -> Option<Placement> {
    crate::persist::read_json::<Placement>(FILE)?.sanitized(ui_scale, system_scale)
}

pub fn save(placement: &Placement) {
    let _ = crate::persist::write_json(FILE, placement);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_placement_round_trips_through_the_store() {
        crate::persist::with_temp_store(|| {
            assert_eq!(load(1.0, 1.0), None, "nothing stored yet");
            let placement = Placement {
                pos: None,
                width: 1200.0,
                height: 700.0,
                maximized: true,
            };
            save(&placement);
            assert_eq!(load(1.0, 1.0), Some(placement));
        });
    }

    #[test]
    fn a_stored_size_is_clamped_and_nonsense_is_dropped() {
        crate::persist::with_temp_store(|| {
            save(&Placement {
                pos: None,
                width: 10.0,
                height: 5.0,
                maximized: false,
            });
            let loaded = load(1.0, 1.0).expect("a small size is clamped, not dropped");
            assert_eq!((loaded.width, loaded.height), (MIN_W, MIN_H));
            crate::persist::write(FILE, r#"{"pos":null,"width":null,"height":1,"maximized":false}"#)
                .unwrap();
            assert_eq!(load(1.0, 1.0), None);
        });
    }

    #[test]
    fn the_restore_position_undoes_the_app_scale() {
        let placement = Placement {
            pos: Some((100.0, 50.0)),
            width: 1000.0,
            height: 700.0,
            maximized: false,
        };
        assert_eq!(
            placement.restore_position(2.0),
            Some(Point::new(200.0, 100.0))
        );
    }
}
