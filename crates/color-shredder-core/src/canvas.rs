use crate::color::Color;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// 2D Canvas coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

impl Coordinate {
    #[inline]
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

/// 2D Painting canvas managing placed colors and the RGBA display buffer.
#[derive(Clone, Debug)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pixels: Vec<Option<Color>>,
    rgba_buffer: Vec<u8>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut rgba = vec![0u8; size * 4];
        // Initialize fully opaque black pixels [0, 0, 0, 255]
        for chunk in rgba.chunks_exact_mut(4) {
            chunk[3] = 255;
        }

        Self {
            width,
            height,
            pixels: vec![None; size],
            rgba_buffer: rgba,
        }
    }

    #[inline]
    pub fn index_of(&self, coord: Coordinate) -> usize {
        coord.y * self.width + coord.x
    }

    #[inline]
    pub fn in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height
    }

    #[inline]
    pub fn is_colored(&self, coord: Coordinate) -> bool {
        self.pixels[self.index_of(coord)].is_some()
    }

    #[inline]
    pub fn get_color(&self, coord: Coordinate) -> Option<Color> {
        self.pixels[self.index_of(coord)]
    }

    /// Paint a pixel at `coord` with `color` and update the RGBA buffer.
    pub fn paint(&mut self, coord: Coordinate, color: Color) {
        let idx = self.index_of(coord);
        self.pixels[idx] = Some(color);

        let [r, g, b, a] = color.to_rgba8();
        let rgba_idx = idx * 4;
        self.rgba_buffer[rgba_idx] = r;
        self.rgba_buffer[rgba_idx + 1] = g;
        self.rgba_buffer[rgba_idx + 2] = b;
        self.rgba_buffer[rgba_idx + 3] = a;
    }

    /// Read-only access to the linear RGBA byte slice for canvas / image rendering.
    #[inline]
    pub fn rgba_buffer(&self) -> &[u8] {
        &self.rgba_buffer
    }

    /// Returns up to 8 neighbors within the canvas bounds.
    pub fn neighbors(&self, coord: Coordinate) -> impl Iterator<Item = Coordinate> + '_ {
        let x = coord.x as isize;
        let y = coord.y as isize;
        let w = self.width;
        let h = self.height;

        [
            (-1, -1), (0, -1), (1, -1),
            (-1,  0),          (1,  0),
            (-1,  1), (0,  1), (1,  1),
        ]
        .into_iter()
        .filter_map(move |(dx, dy)| {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && (nx as usize) < w && ny >= 0 && (ny as usize) < h {
                Some(Coordinate::new(nx as usize, ny as usize))
            } else {
                None
            }
        })
    }

    /// Returns all colored neighbors and their colors.
    pub fn colored_neighbors(&self, coord: Coordinate) -> Vec<(Coordinate, Color)> {
        self.neighbors(coord)
            .filter_map(|n| self.get_color(n).map(|c| (n, c)))
            .collect()
    }

    /// Returns all uncolored neighbors.
    pub fn uncolored_neighbors(&self, coord: Coordinate) -> Vec<Coordinate> {
        self.neighbors(coord)
            .filter(|&n| !self.is_colored(n))
            .collect()
    }

    /// Calculates the mean neighborhood color and count of colored neighbors around `coord`.
    pub fn neighborhood_average(&self, coord: Coordinate) -> Option<(usize, Color)> {
        let mut count = 0usize;
        let mut sum_r = 0.0f32;
        let mut sum_g = 0.0f32;
        let mut sum_b = 0.0f32;

        for n in self.neighbors(coord) {
            if let Some(c) = self.get_color(n) {
                count += 1;
                sum_r += c.r;
                sum_g += c.g;
                sum_b += c.b;
            }
        }

        if count > 0 {
            let inv = 1.0 / count as f32;
            Some((count, Color::new(sum_r * inv, sum_g * inv, sum_b * inv)))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_bounds_and_neighbors() {
        let canvas = Canvas::new(10, 10);
        // Corner has 3 neighbors
        assert_eq!(canvas.neighbors(Coordinate::new(0, 0)).count(), 3);
        // Edge has 5 neighbors
        assert_eq!(canvas.neighbors(Coordinate::new(5, 0)).count(), 5);
        // Center has 8 neighbors
        assert_eq!(canvas.neighbors(Coordinate::new(5, 5)).count(), 8);
    }

    #[test]
    fn test_canvas_painting_and_averages() {
        let mut canvas = Canvas::new(5, 5);
        canvas.paint(Coordinate::new(1, 1), Color::RED);
        canvas.paint(Coordinate::new(1, 3), Color::BLUE);

        // Location (1, 2) has both Red and Blue as colored neighbors
        let (count, avg) = canvas.neighborhood_average(Coordinate::new(1, 2)).unwrap();
        assert_eq!(count, 2);
        assert!((avg.r - 0.5).abs() < 1e-6);
        assert!((avg.g - 0.0).abs() < 1e-6);
        assert!((avg.b - 0.5).abs() < 1e-6);
    }
}
