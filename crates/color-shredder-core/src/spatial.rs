use crate::canvas::{Canvas, Coordinate};
use crate::color::Color;
use rstar::{PointDistance, RTree, RTreeObject, AABB};

/// Represents a candidate available coordinate in 3D RGB color space indexed by R*-tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RStarCandidate {
    pub point: [f32; 3],
    pub coord: Coordinate,
    pub id: u64,
}

impl RTreeObject for RStarCandidate {
    type Envelope = AABB<[f32; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.point)
    }
}

impl PointDistance for RStarCandidate {
    fn distance_2(&self, target: &[f32; 3]) -> f32 {
        let dr = self.point[0] - target[0];
        let dg = self.point[1] - target[1];
        let db = self.point[2] - target[2];
        dr * dr + dg * dg + db * db
    }
}

use std::collections::HashMap;

/// Dynamic R*-Tree manager maintaining spatial indexes of available frontier cells.
pub struct RStarIndex {
    tree: RTree<RStarCandidate>,
    lookup: HashMap<Coordinate, RStarCandidate>,
    next_id: u64,
}

impl RStarIndex {
    pub fn new(_width: usize, _height: usize) -> Self {
        Self {
            tree: RTree::new(),
            lookup: HashMap::new(),
            next_id: 0,
        }
    }

    /// Query the tree for the candidate with minimum color distance to `target_color`.
    #[inline]
    pub fn find_nearest(&self, target_color: Color) -> Option<Coordinate> {
        let target_pt = [target_color.r, target_color.g, target_color.b];
        self.tree.nearest_neighbor(&target_pt).map(|c| c.coord)
    }

    /// Inserts or updates a candidate cell with its latest neighborhood average color.
    pub fn upsert(&mut self, coord: Coordinate, canvas: &Canvas) {
        // Remove previous entry if it was already indexed
        if let Some(existing) = self.lookup.remove(&coord) {
            self.tree.remove(&existing);
        }

        // Compute updated neighborhood average
        if let Some((_count, avg_color)) = canvas.neighborhood_average(coord) {
            let candidate = RStarCandidate {
                point: [avg_color.r, avg_color.g, avg_color.b],
                coord,
                id: self.next_id,
            };
            self.next_id += 1;

            self.tree.insert(candidate);
            self.lookup.insert(coord, candidate);
        }
    }

    /// Remove a coordinate from the index (e.g. once it has been painted).
    pub fn remove(&mut self, coord: Coordinate) {
        if let Some(existing) = self.lookup.remove(&coord) {
            self.tree.remove(&existing);
        }
    }

    /// Number of available locations currently tracked in the R*-tree.
    #[inline]
    pub fn len(&self) -> usize {
        self.tree.size()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tree.size() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rstar_nearest_neighbor() {
        let mut canvas = Canvas::new(5, 5);
        let mut index = RStarIndex::new(5, 5);

        // Paint center red
        canvas.paint(Coordinate::new(2, 2), Color::RED);

        // Track neighbor (2, 3)
        let neighbor = Coordinate::new(2, 3);
        index.upsert(neighbor, &canvas);

        assert_eq!(index.len(), 1);
        let found = index.find_nearest(Color::RED);
        assert_eq!(found, Some(neighbor));
    }
}
