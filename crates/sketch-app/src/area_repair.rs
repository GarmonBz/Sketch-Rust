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
        return original
            .attributes
            .get("apex.calculation")
            .is_some_and(|v| v == "net")
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
                    || r.original
                        .attributes
                        .get("apex.calculation")
                        .is_some_and(|v| v == "net"))
                && r.original.points.len() == r.original.edges.len()
        })
        .collect())
}
