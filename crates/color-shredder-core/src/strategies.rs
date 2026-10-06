use crate::canvas::{Canvas, Coordinate};
use crate::color::Color;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Strategy used to pick the "best fit" canvas location for a target color.
///
/// Each strategy defines a unique mathematical heuristic for evaluating how well a
/// target color fits into an available frontier coordinate based on surrounding placed pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Strategy {
    /// Strategy 1: Minimum distance to any adjacent colored neighbor.
    ///
    /// Finds the frontier candidate that has at least one adjacent colored neighbor
    /// whose color is closest to the target. This produces gritty, high-contrast,
    /// pixelated textures with sharp local transitions.
    Min = 1,

    /// Strategy 2: Average distance across all adjacent colored neighbors.
    ///
    /// Computes the mean Euclidean distance to all existing colored neighbors around
    /// each candidate. This creates organic, soft color blending inside regions while
    /// preserving striking, high-contrast borders between different color bands.
    Average = 2,

    /// Strategy 3: Distance to the average color of the neighborhood (Default).
    ///
    /// First averages the RGB values of all colored neighbors into a single centroid color,
    /// then computes Euclidean distance from the target color to that centroid.
    /// This produces smooth tie-dye flows and organic dendritic growth.
    /// It can also be accelerated via 3D spatial indexing (R*-tree).
    #[default]
    Neighborhood = 3,
}

impl Strategy {
    /// Parses numeric strategy ID (1, 2, or 3) into a `Strategy` enum variant.
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            1 => Some(Self::Min),
            2 => Some(Self::Average),
            3 => Some(Self::Neighborhood),
            _ => None,
        }
    }
}

/// Computes the heuristic fitness distance between a `target_color` and a `candidate` coordinate
/// according to the selected `strategy`. Lower values represent better fit.
#[inline]
fn calculate_distance(
    strategy: Strategy,
    candidate: Coordinate,
    target_color: Color,
    canvas: &Canvas,
) -> f32 {
    match strategy {
        Strategy::Min => {
            let mut min_d = f32::MAX;
            let mut count = 0;
            for n in canvas.neighbors(candidate) {
                if let Some(n_color) = canvas.get_color(n) {
                    count += 1;
                    let d = target_color.dist_sq(n_color);
                    if d < min_d {
                        min_d = d;
                    }
                }
            }
            if count > 0 { min_d } else { f32::MAX }
        }
        Strategy::Average => {
            let mut sum_d = 0.0f32;
            let mut count = 0;
            for n in canvas.neighbors(candidate) {
                if let Some(n_color) = canvas.get_color(n) {
                    count += 1;
                    sum_d += target_color.dist_sq(n_color);
                }
            }
            if count > 0 {
                sum_d / count as f32
            } else {
                f32::MAX
            }
        }
        Strategy::Neighborhood => {
            if let Some((_count, avg_color)) = canvas.neighborhood_average(candidate) {
                target_color.dist_sq(avg_color)
            } else {
                f32::MAX
            }
        }
    }
}

/// Evaluates the entire frontier for a given strategy and returns the coordinate with the lowest distance.
///
/// ## Parallel Execution
/// When compiled with the `rayon` feature, this function automatically splits the candidate
/// evaluation across all available CPU threads using Rayon work-stealing when the frontier size
/// is $\ge 64$ elements. Below 64 elements, it executes sequentially to avoid thread scheduling overhead.
pub fn evaluate_best_candidate(
    strategy: Strategy,
    frontier: &[Coordinate],
    target_color: Color,
    canvas: &Canvas,
) -> Option<Coordinate> {
    if frontier.is_empty() {
        return None;
    }

    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        // Use parallel iteration when the frontier size amortizes thread scheduling overhead (>= 64)
        if frontier.len() >= 64 {
            return frontier
                .par_iter()
                .map(|&candidate| {
                    let dist = calculate_distance(strategy, candidate, target_color, canvas);
                    (candidate, dist)
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(coord, _)| coord);
        }
    }

    // Sequential fallback for small frontiers or single-threaded builds
    let mut best_coord = frontier[0];
    let mut min_distance = f32::MAX;

    for &candidate in frontier {
        let distance = calculate_distance(strategy, candidate, target_color, canvas);
        if distance < min_distance {
            min_distance = distance;
            best_coord = candidate;
        }
    }

    Some(best_coord)
}
