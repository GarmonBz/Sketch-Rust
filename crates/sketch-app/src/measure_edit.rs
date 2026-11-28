//! Tape measurements as the Select arrow sees them: finding the one under the
//! pointer, reshaping it and moving it, and how a page stores them. A
//! measurement is a run of points; a closed one repeats its first point at the
//! end.

use serde::Deserialize;
use sketch_core::geometry::{distance, GeometryEngine, Pt};

/// The measurements a page stores: the finished ones, and the one still being
/// taped once it has a length.
pub fn to_save(finished: &[Vec<Pt>], taping: &[Pt]) -> Vec<Vec<Pt>> {
    let mut runs = finished.to_vec();
    if taping.len() >= 2 {
        runs.push(taping.to_vec());
    }
    runs
}

/// Reads stored measurements. One that does not read is dropped by itself, so
/// it cannot take the rest of its page with it.
pub fn deserialize_runs<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Vec<Pt>>, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    Ok(value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|run| Vec::<Pt>::deserialize(run).ok())
        .filter(|run| run.len() >= 2)
        .collect())
}

/// Which part of a measurement the pointer has hold of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grab {
    /// An end or a corner: dragging it changes the measurement.
    Point(usize),
    /// A leg between points: dragging it moves the whole measurement.
    Line,
}

/// The measurement under `at`, the one drawn last first: an end or corner
/// within `point_reach`, otherwise any leg within `line_reach`.
pub fn hit(runs: &[Vec<Pt>], at: Pt, point_reach: f64, line_reach: f64) -> Option<(usize, Grab)> {
    let corner = runs.iter().enumerate().rev().find_map(|(run, points)| {
        points
            .iter()
            .enumerate()
            .map(|(i, p)| (i, distance(at, *p)))
            .filter(|(_, d)| *d <= point_reach)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| (run, Grab::Point(i)))
    });
    corner.or_else(|| {
        let geo = GeometryEngine::new();
        runs.iter()
            .rposition(|points| {
                points
                    .windows(2)
                    .any(|leg| geo.distance_to_segment(at, leg[0], leg[1]) <= line_reach)
            })
            .map(|run| (run, Grab::Line))
    })
}

/// `original` with the grabbed part carried by (`dx`, `dy`). The shared point
/// of a closed measurement moves at both ends, so it stays closed.
pub fn reshape(original: &[Pt], grab: Grab, dx: f64, dy: f64) -> Vec<Pt> {
    let last = original.len().saturating_sub(1);
    let closed = original.len() > 2 && original.first() == original.last();
    original
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let carried = match grab {
                Grab::Line => true,
                Grab::Point(held) => {
                    i == held
                        || (closed && (held, i) == (0, last))
                        || (closed && (held, i) == (last, 0))
                }
            };
            if carried {
                Pt::new(p.x + dx, p.y + dy)
            } else {
                *p
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pts(xy: &[(f64, f64)]) -> Vec<Pt> {
        xy.iter().map(|&(x, y)| Pt::new(x, y)).collect()
    }

    #[test]
    fn ends_win_over_legs_and_the_newest_measurement_wins() {
        let runs = [
            pts(&[(0.0, 0.0), (10.0, 0.0)]),
            pts(&[(0.0, 0.0), (0.0, 10.0)]),
        ];
        let at = |x, y| hit(&runs, Pt::new(x, y), 0.5, 0.3);
        assert_eq!(at(10.2, 0.0), Some((0, Grab::Point(1))));
        assert_eq!(at(0.1, 0.1), Some((1, Grab::Point(0))), "the one on top");
        assert_eq!(at(5.0, 0.2), Some((0, Grab::Line)));
        assert_eq!(at(0.2, 5.0), Some((1, Grab::Line)));
        assert_eq!(at(5.0, 5.0), None);
        assert_eq!(hit(&[], Pt::new(0.0, 0.0), 1.0, 1.0), None);
    }

    #[test]
    fn an_end_stretches_and_a_leg_moves_everything() {
        let run = pts(&[(0.0, 0.0), (10.0, 0.0), (10.0, 5.0)]);
        assert_eq!(
            reshape(&run, Grab::Point(2), 1.0, 2.0),
            pts(&[(0.0, 0.0), (10.0, 0.0), (11.0, 7.0)])
        );
        assert_eq!(
            reshape(&run, Grab::Line, 1.0, 2.0),
            pts(&[(1.0, 2.0), (11.0, 2.0), (11.0, 7.0)])
        );
    }

    #[test]
    fn a_closed_measurement_stays_closed() {
        let run = pts(&[(0.0, 0.0), (10.0, 0.0), (10.0, 5.0), (0.0, 0.0)]);
        let want = pts(&[(-1.0, 1.0), (10.0, 0.0), (10.0, 5.0), (-1.0, 1.0)]);
        assert_eq!(reshape(&run, Grab::Point(0), -1.0, 1.0), want);
        assert_eq!(reshape(&run, Grab::Point(3), -1.0, 1.0), want);
    }
}
