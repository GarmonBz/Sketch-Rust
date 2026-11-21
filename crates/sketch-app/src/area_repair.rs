//! Page-local metadata for areas opened by deleting boundary walls.
use serde::{Deserialize, Serialize};
use sketch_core::{
    geometry::Pt,
    model::{Polygon, SketchLine},
};
use std::collections::{HashMap, HashSet};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReopenedArea {
    pub original: Polygon,
    /// Older neighboring polygons must never become restoration candidates.
    pub first_replacement_id: i64,
}
fn winding(points: &[Pt]) -> f64 {
    let area = points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let q = points[(i + 1) % points.len()];
            p.x * q.y - q.x * p.y
        })
        .sum::<f64>();
    if area.abs() < 1e-9 {
        0.
    } else {
        area.signum()
    }
}
fn same_ring(a: &[Pt], b: &[Pt]) -> bool {
    if a.len() != b.len() || a.is_empty() {
        return false;
    }
    let near = |p: Pt, q: Pt| (p.x - q.x).hypot(p.y - q.y) <= 0.02;
    (0..b.len()).any(|offset| {
        (0..a.len()).all(|i| near(a[i], b[(offset + i) % b.len()]))
            || (0..a.len()).all(|i| near(a[i], b[(offset + b.len() - i) % b.len()]))
    })
}
fn matches(original: &Polygon, candidate: &Polygon, live: &HashSet<i64>) -> bool {
    if candidate.points.len() < 3
        || original.edges.len() != original.points.len()
        || candidate.edges.len() != candidate.points.len()
    {
        return false;
    }
    if original.points.len() < 2 {
        return sketch_core::area_calc::uses_native_net(original)
            && original.points.iter().all(|point| {
                candidate
                    .points
                    .iter()
                    .any(|p| (p.x - point.x).hypot(p.y - point.y) <= 0.02)
            });
    }
    let remaining: Vec<_> = original
        .edges
        .iter()
        .enumerate()
        .filter_map(|(i, e)| e.line_id.filter(|id| live.contains(id)).map(|id| (i, id)))
        .collect();
    if remaining.is_empty() {
        return same_ring(&original.points, &candidate.points);
    }
    let old_winding = winding(&original.points);
    let new_winding = winding(&candidate.points);
    if original.points.len() == 2 {
        return !remaining.is_empty()
            && remaining
                .iter()
                .all(|(_, id)| candidate.edges.iter().any(|edge| edge.line_id == Some(*id)));
    }
    if old_winding == 0. || new_winding == 0. {
        return same_ring(&original.points, &candidate.points);
    }
    remaining.iter().all(|(old, id)| {
        let Some(new) = candidate
            .edges
            .iter()
            .position(|edge| edge.line_id == Some(*id))
        else {
            return false;
        };
        let (Some(a), Some(b), Some(c), Some(d)) = (
            original.points.get(*old),
            original.points.get((*old + 1) % original.points.len()),
            candidate.points.get(new),
            candidate.points.get((new + 1) % candidate.points.len()),
        ) else {
            return false;
        };
        // The candidate must occupy the original side of each surviving wall.
        // This rejects a new neighboring room sharing the same boundary.
        let dot = (b.x - a.x) * (d.x - c.x) + (b.y - a.y) * (d.y - c.y);
        dot * old_winding * new_winding > 0.
    })
}
/// Brings an area back when every wall deleted from it has been drawn again
/// between the same two points. This works from the area's own record, so it
/// holds even when neighbouring walls touch the sides mid-wall (where a face
/// walk would see split edges and decline). Returns the restored area ids.
pub fn restore_by_walls(
    lines: &mut [SketchLine],
    polygons: &mut Vec<Polygon>,
    records: &mut Vec<ReopenedArea>,
) -> Vec<i64> {
    let live: HashSet<i64> = lines.iter().map(|line| line.id).collect();
    let near = |p: Pt, q: Pt| (p.x - q.x).hypot(p.y - q.y) <= 0.02;
    let mut restored = vec![];
    let mut consumed = HashSet::new();
    for (index, record) in records.iter().enumerate() {
        let original = &record.original;
        let n = original.points.len();
        if n < 3 || original.edges.len() != n || polygons.iter().any(|poly| poly.id == original.id)
        {
            continue;
        }
        let mut edges = original.edges.clone();
        let mut used: HashSet<i64> = HashSet::new();
        let mut missing = 0;
        let mut complete = true;
        for (i, edge) in edges.iter_mut().enumerate() {
            if edge.line_id.is_some_and(|id| live.contains(&id)) {
                continue;
            }
            missing += 1;
            let (a, b) = (original.points[i], original.points[(i + 1) % n]);
            let found = lines.iter().find(|line| {
                line.id >= record.first_replacement_id
                    && !used.contains(&line.id)
                    && ((near(line.start, a) && near(line.end, b))
                        || (near(line.start, b) && near(line.end, a)))
            });
            let Some(line) = found else {
                complete = false;
                break;
            };
            used.insert(line.id);
            let follows = near(line.start, a);
            *edge = sketch_core::geometry::Edge {
                line_id: Some(line.id),
                bow_angle: if follows {
                    line.bow_angle
                } else {
                    -line.bow_angle
                },
            };
        }
        if !complete || missing == 0 {
            continue;
        }
        let mut repaired = original.clone();
        repaired.edges = edges;
        if let Some(layer) = &repaired.layer_id {
            for line in lines.iter_mut() {
                if repaired
                    .edges
                    .iter()
                    .any(|edge| edge.line_id == Some(line.id))
                {
                    line.layer_id = Some(layer.clone());
                }
            }
        }
        restored.push(repaired.id);
        // Closing the chain may already have made a new area from the same
        // walls: the restored one takes its place instead of doubling it.
        let ids: HashSet<i64> = repaired.edges.iter().filter_map(|e| e.line_id).collect();
        let twin = polygons.iter().position(|poly| {
            poly.id != repaired.id
                && poly.edges.len() == repaired.edges.len()
                && poly
                    .edges
                    .iter()
                    .filter_map(|e| e.line_id)
                    .collect::<HashSet<i64>>()
                    == ids
        });
        match twin {
            Some(at) => polygons[at] = repaired,
            None => polygons.push(repaired),
        }
        consumed.insert(index);
    }
    let mut index = 0;
    records.retain(|_| {
        let keep = !consumed.contains(&index);
        index += 1;
        keep
    });
    restored
}

/// Returns generated polygon IDs whose metadata was restored. Restore only
/// unambiguous matches; uncertain records remain available for later repairs.
pub fn restore(
    lines: &mut [SketchLine],
    polygons: &mut [Polygon],
    records: &mut Vec<ReopenedArea>,
) -> Vec<i64> {
    let live: HashSet<_> = lines.iter().map(|line| line.id).collect();
    records.retain(|record| !polygons.iter().any(|poly| poly.id == record.original.id));
    let mut proposals = vec![];
    for (record_index, record) in records.iter().enumerate() {
        let mut choices: Vec<_> = polygons
            .iter()
            .enumerate()
            .filter(|(_, poly)| {
                poly.id >= record.first_replacement_id && matches(&record.original, poly, &live)
            })
            .map(|(i, _)| i)
            .collect();
        if choices.len() > 1 {
            choices.retain(|i| same_ring(&record.original.points, &polygons[*i].points));
        }
        if choices.len() == 1 {
            proposals.push((record_index, choices[0]));
        }
    }
    let mut claims = HashMap::new();
    for (_, candidate) in &proposals {
        *claims.entry(*candidate).or_insert(0) += 1;
    }
    let mut consumed = HashSet::new();
    let mut restored = vec![];
    for (record_index, candidate) in proposals {
        if claims[&candidate] != 1 {
            continue;
        }
        let old = &records[record_index].original;
        let closed = &polygons[candidate];
        let generated_id = closed.id;
        let mut repaired = old.clone();
        repaired.points = closed.points.clone();
        repaired.edges = closed.edges.clone();
        repaired.gross = closed.gross;
        repaired.area = closed.area;
        if let Some(layer) = &repaired.layer_id {
            for line in lines.iter_mut() {
                if repaired
                    .edges
                    .iter()
                    .any(|edge| edge.line_id == Some(line.id))
                {
                    line.layer_id = Some(layer.clone());
                }
            }
        }
        polygons[candidate] = repaired;
        restored.push(generated_id);
        consumed.insert(record_index);
    }
    let mut index = 0;
    records.retain(|_| {
        let keep = !consumed.contains(&index);
        index += 1;
        keep
    });
    restored
}
/// Ignore malformed optional repair records without losing the live page.
pub fn deserialize_records<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Vec<ReopenedArea>, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    Ok(value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| serde_json::from_value::<ReopenedArea>(v.clone()).ok())
        .filter(|r| {
            r.first_replacement_id > r.original.id
                && (r.original.points.len() >= 2
                    || sketch_core::area_calc::uses_native_net(&r.original))
                && r.original.points.len() == r.original.edges.len()
        })
        .collect())
}

#[cfg(test)]
mod restore_by_walls_tests {
    use super::*;
    use serde_json::json;

    fn line(id: i64, sx: f64, sy: f64, ex: f64, ey: f64) -> SketchLine {
        serde_json::from_value(json!({
            "id": id, "start": {"x": sx, "y": sy}, "end": {"x": ex, "y": ey}, "type": "exterior"
        }))
        .expect("a wall")
    }

    fn room() -> (Vec<SketchLine>, Polygon) {
        let lines = vec![
            line(1, 0.0, 0.0, 10.0, 0.0),
            line(2, 10.0, 0.0, 10.0, 10.0),
            line(3, 10.0, 10.0, 0.0, 10.0),
            line(4, 0.0, 10.0, 0.0, 0.0),
        ];
        let poly: Polygon = serde_json::from_value(json!({
            "id": 50, "type": "GLA1", "name": "Living",
            "points": [{"x":0.0,"y":0.0},{"x":10.0,"y":0.0},{"x":10.0,"y":10.0},{"x":0.0,"y":10.0}],
            "edges": [{"lineId":1},{"lineId":2},{"lineId":3},{"lineId":4}],
            "gross": 100.0, "area": 100.0, "layerId": "layer-9"
        }))
        .expect("an area");
        (lines, poly)
    }

    #[test]
    fn redrawing_the_deleted_wall_brings_the_area_back_as_it_was() {
        let (mut lines, poly) = room();
        // Wall 3 is deleted: the area is recorded as opened.
        lines.retain(|l| l.id != 3);
        let mut records = vec![ReopenedArea {
            original: poly,
            first_replacement_id: 90,
        }];
        let mut polygons: Vec<Polygon> = Vec::new();
        // Nothing yet: no wall between the open ends.
        assert!(restore_by_walls(&mut lines, &mut polygons, &mut records).is_empty());
        // A neighbour's wall touching the room mid-wall changes nothing here.
        lines.push(line(91, 5.0, 10.0, 5.0, 20.0));
        assert!(restore_by_walls(&mut lines, &mut polygons, &mut records).is_empty());
        // The wall is drawn again, in the other direction.
        lines.push(line(92, 0.0, 10.0, 10.0, 10.0));
        let restored = restore_by_walls(&mut lines, &mut polygons, &mut records);
        assert_eq!(restored, vec![50]);
        assert!(records.is_empty(), "the record is used up");
        assert_eq!(polygons.len(), 1);
        let back = &polygons[0];
        assert_eq!((back.kind.as_str(), back.name.as_str()), ("GLA1", "Living"));
        assert_eq!(
            back.edges[2].line_id,
            Some(92),
            "the new wall fills the gap"
        );
        assert_eq!(back.edges[0].line_id, Some(1));
        // Its walls go back to the area's layer.
        let layer = lines.iter().find(|l| l.id == 92).unwrap().layer_id.clone();
        assert_eq!(layer.as_deref(), Some("layer-9"));
    }

    #[test]
    fn a_wall_that_misses_the_gap_or_predates_the_deletion_does_not_close_it() {
        let (mut lines, poly) = room();
        lines.retain(|l| l.id != 3);
        let mut records = vec![ReopenedArea {
            original: poly,
            first_replacement_id: 90,
        }];
        let mut polygons: Vec<Polygon> = Vec::new();
        lines.push(line(95, 0.0, 10.0, 9.0, 10.0)); // too short
        lines.push(line(40, 10.0, 10.0, 0.0, 10.0)); // older than the deletion
        assert!(restore_by_walls(&mut lines, &mut polygons, &mut records).is_empty());
        assert!(polygons.is_empty() && records.len() == 1);
    }
}
