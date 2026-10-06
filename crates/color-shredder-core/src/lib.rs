//! # Color Shredder Core
//!
//! High-performance algorithmic engine for tie-dye pixel art generation.
//!
//! Color Shredder generates unique pixel art by taking a deterministic, randomized
//! color palette and placing each color sequentially onto a 2D canvas at the
//! available frontier location that minimizes color distance according to one of
//! three algorithmic strategies:
//!
//! 1. **Strategy 1 (Min Neighbor Distance)**: Minimizes distance to any adjacent colored pixel,
//!    creating gritty, high-contrast, pixelated textures.
//! 2. **Strategy 2 (Average Neighbor Distance)**: Minimizes the average distance across all
//!    adjacent colored pixels, producing smooth gradients with sharp boundary lines.
//! 3. **Strategy 3 (Neighborhood Color Average)**: Minimizes distance to the mean color of the
//!    adjacent colored neighborhood. Can be accelerated via 3D spatial indexing (R*-tree).
//!
//! ## Multiprocessing & Concurrency
//!
//! When the `rayon` feature is enabled (default), candidate evaluation across large frontiers
//! is automatically parallelized across all available CPU cores using work-stealing threads.

pub mod color;
pub mod canvas;
pub mod generator;
pub mod spatial;
pub mod strategies;
pub mod engine;

pub use color::{Color, ColorSpace};
pub use canvas::{Canvas, Coordinate};
pub use generator::{generate_colors, ColorGeneratorConfig};
pub use spatial::RStarIndex;
pub use strategies::Strategy;
pub use engine::{Engine, EngineConfig, EngineStats};
