use crate::canvas::{Canvas, Coordinate};
use crate::color::Color;
use crate::generator::{generate_colors, ColorGeneratorConfig};
use crate::spatial::RStarIndex;
use crate::strategies::{evaluate_best_candidate, Strategy};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// High-level engine configuration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EngineConfig {
    pub width: usize,
    pub height: usize,
    pub start_point: Coordinate,
    pub strategy: Strategy,
    pub use_rstar: bool,
    pub color_config: ColorGeneratorConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            start_point: Coordinate::new(0, 0),
            strategy: Strategy::Neighborhood,
            use_rstar: true,
            color_config: ColorGeneratorConfig::default(),
        }
    }
}

/// Statistics reported after execution steps.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EngineStats {
    pub pixels_placed: usize,
    pub total_pixels: usize,
    pub pixels_available: usize,
    pub percent_complete: f32,
    pub is_finished: bool,
}

/// The stateful Color Shredder generation engine.
pub struct Engine {
    pub config: EngineConfig,
    pub canvas: Canvas,
    colors: Vec<Color>,
    color_index: usize,
    frontier: Vec<Coordinate>,
    frontier_set: Vec<bool>, // Fast O(1) membership check
    rstar_index: Option<RStarIndex>,
    is_finished: bool,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        let canvas = Canvas::new(config.width, config.height);
        let colors = generate_colors(&config.color_config);

        let use_rstar = config.use_rstar && config.strategy == Strategy::Neighborhood;
        let rstar_index = if use_rstar {
            Some(RStarIndex::new(config.width, config.height))
        } else {
            None
        };

        let size = config.width * config.height;
        let initial_frontier_cap = (size / 16).clamp(64, 32_768);
        let mut engine = Self {
            config,
            canvas,
            colors,
            color_index: 0,
            frontier: Vec::with_capacity(initial_frontier_cap),
            frontier_set: vec![false; size],
            rstar_index,
            is_finished: false,
        };

        // Seed with starting pixel
        engine.seed();
        engine
    }

    fn seed(&mut self) {
        if self.colors.is_empty() {
            self.is_finished = true;
            return;
        }

        let first_color = self.colors[self.color_index];
        self.color_index += 1;

        let seed_coord = self.config.start_point;
        self.canvas.paint(seed_coord, first_color);

        // Add seed's uncolored neighbors to frontier
        for neighbor in self.canvas.uncolored_neighbors(seed_coord) {
            self.add_to_frontier(neighbor);
        }
    }

    #[inline]
    fn idx(&self, coord: Coordinate) -> usize {
        coord.y * self.config.width + coord.x
    }

    fn add_to_frontier(&mut self, coord: Coordinate) {
        let idx = self.idx(coord);
        if !self.frontier_set[idx] && !self.canvas.is_colored(coord) {
            self.frontier_set[idx] = true;
            self.frontier.push(coord);
        }

        if let Some(rstar) = &mut self.rstar_index {
            rstar.upsert(coord, &self.canvas);
        }
    }

    fn remove_from_frontier(&mut self, coord: Coordinate) {
        let idx = self.idx(coord);
        if self.frontier_set[idx] {
            self.frontier_set[idx] = false;
            if let Some(pos) = self.frontier.iter().position(|&c| c == coord) {
                self.frontier.swap_remove(pos);
            }
        }

        if let Some(rstar) = &mut self.rstar_index {
            rstar.remove(coord);
        }
    }

    /// Advance the painting simulation by up to `batch_size` pixels.
    pub fn step(&mut self, batch_size: usize) -> EngineStats {
        if self.is_finished {
            return self.stats();
        }

        let total_pixels = self.config.width * self.config.height;

        for _ in 0..batch_size {
            if self.color_index >= self.colors.len() || self.frontier.is_empty() {
                self.is_finished = true;
                break;
            }

            let target_color = self.colors[self.color_index];

            // Determine best candidate coordinate
            let best_coord = if let Some(rstar) = &self.rstar_index {
                rstar.find_nearest(target_color)
            } else {
                evaluate_best_candidate(
                    self.config.strategy,
                    &self.frontier,
                    target_color,
                    &self.canvas,
                )
            };

            let Some(coord) = best_coord else {
                self.is_finished = true;
                break;
            };

            // Paint the chosen coordinate
            self.color_index += 1;
            self.canvas.paint(coord, target_color);
            self.remove_from_frontier(coord);

            // Update neighbors: add newly uncolored ones and refresh existing neighborhood colors
            for neighbor in self.canvas.neighbors(coord) {
                if !self.canvas.is_colored(neighbor) {
                    let n_idx = self.idx(neighbor);
                    if !self.frontier_set[n_idx] {
                        self.frontier_set[n_idx] = true;
                        self.frontier.push(neighbor);
                    }
                    if let Some(rstar) = &mut self.rstar_index {
                        rstar.upsert(neighbor, &self.canvas);
                    }
                }
            }

            if self.color_index >= total_pixels {
                self.is_finished = true;
                break;
            }
        }

        self.stats()
    }

    /// Run the simulation until all pixels are placed or no frontier remains.
    pub fn run_to_completion(&mut self) -> EngineStats {
        while !self.is_finished {
            self.step(500);
        }
        self.stats()
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }

    #[inline]
    pub fn rgba_buffer(&self) -> &[u8] {
        self.canvas.rgba_buffer()
    }

    pub fn stats(&self) -> EngineStats {
        let total_pixels = self.config.width * self.config.height;
        let placed = self.color_index.min(total_pixels);
        let pct = if total_pixels > 0 {
            (placed as f32 / total_pixels as f32) * 100.0
        } else {
            100.0
        };

        EngineStats {
            pixels_placed: placed,
            total_pixels,
            pixels_available: self.frontier.len(),
            percent_complete: pct,
            is_finished: self.is_finished,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_small_run() {
        let config = EngineConfig {
            width: 16,
            height: 16,
            start_point: Coordinate::new(8, 8),
            strategy: Strategy::Neighborhood,
            use_rstar: true,
            color_config: ColorGeneratorConfig {
                bit_depth: 3, // 512 colors, more than 16*16=256
                seed: Some(99),
                ..Default::default()
            },
        };

        let mut engine = Engine::new(config);
        assert!(!engine.is_finished);
        assert!(engine.stats().pixels_placed >= 1);

        let final_stats = engine.run_to_completion();
        assert!(final_stats.is_finished);
        assert_eq!(final_stats.pixels_placed, 256);
        assert_eq!(final_stats.percent_complete, 100.0);
    }
}
