use crate::{clipboard, placed_symbol, placed_text, selection, view_state, App};

impl App {
    /// Freeform ink uses ordinary interior segments, so saving, printing,
    /// selection and exports all use the existing document pipeline.
    pub(crate) fn stroke_to(&mut self, world: sketch_core::geometry::Pt) {
        use sketch_core::geometry::{distance, Pt};
        use sketch_core::model::{SketchLine, WallType};
        use view_state::Drag;
        let Some(drag) = self.view.drag.clone() else {
            return;
        };
        match drag {
            Drag::Freeform { last } => {
                if self.layers.layer_locked(&self.layers.active)
                    || !self.layers.layer_visible(&self.layers.active)
                {
                    return;
                }
                let start = Pt::new(last.x, last.y);
                if distance(start, world) < selection::tolerance(self.view.zoom, 1.0) {
                    return;
                }
                if !self.drag_banked {
                    self.push_undo("Freeform Draw");
                    self.drag_banked = true;
                }
                let id = self.drawing.gen_id();
                self.drawing.lines.push(SketchLine {
                    id,
                    start,
                    end: world,
                    kind: WallType::Interior,
                    color: Some(self.drawing.draw_color.clone()),
                    line_thickness: Some(2.0),
                    thickness: Some(0.0),
                    dimension_hidden: true,
                    layer_id: Some(self.layers.active.clone()),
                    bow_angle: 0.0,
                    arc_height: 0.0,
                    line_style: None,
                    dimension_position: None,
                    dimension_anchor: None,
                    extra: Default::default(),
                });
                self.view.drag = Some(Drag::Freeform {
                    last: view_state::World {
                        x: world.x,
                        y: world.y,
                    },
                });
            }
            Drag::Erase { last } => {
                let gates = self.layer_gates();
                let start = Pt::new(last.x, last.y);
                // Sample the swept path in screen space so fast drags cannot
                // skip thin walls between pointer events.
                let steps = (distance(start, world) / selection::tolerance(self.view.zoom, 4.0))
                    .ceil()
                    .max(1.0) as usize;
                let mut erased = selection::Selection::default();
                let editable_placed_layer = |layer_id: i64| {
                    let named = layer_id.to_string();
                    let layer = self.entity_layer_id((layer_id != 0).then_some(named.as_str()));
                    self.layers.layer_visible(&layer) && !self.layers.layer_locked(&layer)
                };
                for i in 0..=steps {
                    let t = i as f64 / steps as f64;
                    let p = Pt::new(
                        start.x + (world.x - start.x) * t,
                        start.y + (world.y - start.y) * t,
                    );
                    // Pick objects on their strokes; empty area interiors do
                    // not erase the surrounding floor plan.
                    if let Some(selection::Hit::Line(id)) =
                        selection::hit_test_at(p, &self.drawing.lines, &[], self.view.zoom, &|id| {
                            gates.selectable(id)
                        })
                    {
                        erased.lines.insert(id);
                    }
                    for text in &self.drawing.texts {
                        if gates.selectable(text.id)
                            && editable_placed_layer(text.layer_id)
                            && placed_text::point_in_text(text, p, self.view.zoom)
                        {
                            erased.texts.insert(text.id);
                        }
                    }
                    for sym in &self.drawing.symbols {
                        let bounds = self.symbols_lib().symbol(&sym.symbol_type).map(|d| {
                            (
                                d.bounds.min_x,
                                d.bounds.min_y,
                                d.bounds.max_x,
                                d.bounds.max_y,
                            )
                        });
                        if gates.selectable(sym.id)
                            && editable_placed_layer(sym.layer_id)
                            && placed_symbol::point_in_symbol(sym, bounds, p, self.view.zoom)
                        {
                            erased.symbols.insert(sym.id);
                        }
                    }
                }
                if !erased.is_empty() {
                    if !self.drag_banked {
                        self.push_undo("Erase");
                        self.drag_banked = true;
                    }
                    clipboard::delete(&mut self.drawing, &erased);
                    self.drawing.texts.retain(|t| !erased.texts.contains(&t.id));
                    self.drawing
                        .symbols
                        .retain(|s| !erased.symbols.contains(&s.id));
                    self.sync_edit_tools();
                }
                self.view.drag = Some(Drag::Erase {
                    last: view_state::World {
                        x: world.x,
                        y: world.y,
                    },
                });
            }
            _ => {}
        }
    }
}
