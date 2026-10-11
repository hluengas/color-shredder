use crate::color::{hls_to_rgb, hsv_to_rgb, Color, ColorSpace};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Palette generation mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PaletteMode {
    /// Pre-computes the entire color space (2^(3*bit_depth) colors) in RAM.
    Precomputed,
    /// Generates colors dynamically on the fly in batches with specified bit depth precision.
    OnTheFly,
}

/// Configuration for color palette generation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ColorGeneratorConfig {
    /// Bit depth per channel (e.g. 6 for 262k, 8 for 16.7M, up to 12 for 68.7B colors).
    pub bit_depth: u32,
    /// Color space used during color synthesis.
    pub color_space: ColorSpace,
    /// If Some(channel_idx), leaves that channel (0, 1, or 2) grouped/unshuffled.
    pub fixed_channel: Option<usize>,
    /// Whether to perform the final full shuffle across all color groups.
    pub shuffle: bool,
    /// Optional PRNG seed for deterministic runs.
    pub seed: Option<u64>,
    /// Palette generation mode: Precomputed vs On-The-Fly.
    pub mode: PaletteMode,
}

impl Default for ColorGeneratorConfig {
    fn default() -> Self {
        Self {
            bit_depth: 6,
            color_space: ColorSpace::Rgb,
            fixed_channel: None,
            shuffle: true,
            seed: None,
            mode: PaletteMode::Precomputed,
        }
    }
}

/// Generates a randomized or grouped palette of unique colors across the color space.
pub fn generate_colors(config: &ColorGeneratorConfig) -> Vec<Color> {
    let mut rng = fastrand::Rng::new();
    if let Some(s) = config.seed {
        rng.seed(s);
    }

    let values_per_channel = 1usize << config.bit_depth;
    let num_sub_colors = values_per_channel * values_per_channel;
    let total_colors = values_per_channel * num_sub_colors;

    let mut result = Vec::with_capacity(total_colors);

    // Initial slice order
    let mut hues: Vec<usize> = (0..values_per_channel).collect();
    rng.shuffle(&mut hues);

    let channel_shift = config.fixed_channel.unwrap_or(0);
    let mut sub_list = Vec::with_capacity(num_sub_colors);

    for chan1_val in hues {
        sub_list.clear();

        for chan2_val in 0..values_per_channel {
            for chan3_val in 0..values_per_channel {
                let mut raw = [0.0f32; 3];
                raw[(0 + channel_shift) % 3] = chan1_val as f32 / values_per_channel as f32;
                raw[(1 + channel_shift) % 3] = chan2_val as f32 / values_per_channel as f32;
                raw[(2 + channel_shift) % 3] = chan3_val as f32 / values_per_channel as f32;

                let color = match config.color_space {
                    ColorSpace::Rgb => Color::new(raw[0], raw[1], raw[2]),
                    ColorSpace::Hsv => hsv_to_rgb(raw[0], raw[1], raw[2]),
                    ColorSpace::Hls => hls_to_rgb(raw[0], raw[1], raw[2]),
                };

                sub_list.push(color);
            }
        }

        // Shuffle within the slice
        rng.shuffle(&mut sub_list);
        result.extend(sub_list.iter().copied());
    }

    // Final shuffle across slices if no channel was explicitly held fixed
    if config.shuffle && config.fixed_channel.is_none() {
        rng.shuffle(&mut result);
    }

    result
}

/// Provides colors either from a precomputed palette or streamed on the fly in batches.
pub struct StreamingColorSource {
    config: ColorGeneratorConfig,
    precomputed: Option<Vec<Color>>,
    color_index: usize,
    batch_buffer: Vec<Color>,
    batch_index: usize,
    rng: fastrand::Rng,
    current_fixed_val: usize,
    steps_in_fixed_slice: usize,
    slice_length: usize,
}

impl StreamingColorSource {
    pub const BATCH_SIZE: usize = 1024;

    pub fn new(config: ColorGeneratorConfig) -> Self {
        let mut rng = fastrand::Rng::new();
        if let Some(s) = config.seed {
            rng.seed(s);
        }

        match config.mode {
            PaletteMode::Precomputed => {
                let colors = generate_colors(&config);
                Self {
                    config,
                    precomputed: Some(colors),
                    color_index: 0,
                    batch_buffer: Vec::new(),
                    batch_index: 0,
                    rng,
                    current_fixed_val: 0,
                    steps_in_fixed_slice: 0,
                    slice_length: 0,
                }
            }
            PaletteMode::OnTheFly => {
                let values_per_channel = 1usize << config.bit_depth;
                // For a fixed channel, group into slices of size ~ (values_per_channel * values_per_channel)
                // or clamped to a reasonable batch size like 256..4096
                let slice_length = (values_per_channel.saturating_mul(values_per_channel)).clamp(256, 4096);
                let initial_fixed_val = rng.usize(0..values_per_channel);

                let mut source = Self {
                    config,
                    precomputed: None,
                    color_index: 0,
                    batch_buffer: Vec::with_capacity(Self::BATCH_SIZE),
                    batch_index: 0,
                    rng,
                    current_fixed_val: initial_fixed_val,
                    steps_in_fixed_slice: 0,
                    slice_length,
                };
                source.refill_batch();
                source
            }
        }
    }

    fn refill_batch(&mut self) {
        self.batch_buffer.clear();
        self.batch_index = 0;

        let values_per_channel = 1usize << self.config.bit_depth;
        let channel_shift = self.config.fixed_channel.unwrap_or(0);

        for _ in 0..Self::BATCH_SIZE {
            let (c1, c2, c3) = if self.config.fixed_channel.is_some() {
                // If a channel is held fixed, keep c1 constant for slice_length steps then pick a new slice
                self.steps_in_fixed_slice += 1;
                if self.steps_in_fixed_slice >= self.slice_length {
                    self.steps_in_fixed_slice = 0;
                    self.current_fixed_val = self.rng.usize(0..values_per_channel);
                }
                let c2_val = self.rng.usize(0..values_per_channel);
                let c3_val = self.rng.usize(0..values_per_channel);
                (self.current_fixed_val, c2_val, c3_val)
            } else {
                let c1_val = self.rng.usize(0..values_per_channel);
                let c2_val = self.rng.usize(0..values_per_channel);
                let c3_val = self.rng.usize(0..values_per_channel);
                (c1_val, c2_val, c3_val)
            };

            let mut raw = [0.0f32; 3];
            raw[(0 + channel_shift) % 3] = c1 as f32 / values_per_channel as f32;
            raw[(1 + channel_shift) % 3] = c2 as f32 / values_per_channel as f32;
            raw[(2 + channel_shift) % 3] = c3 as f32 / values_per_channel as f32;

            let color = match self.config.color_space {
                ColorSpace::Rgb => Color::new(raw[0], raw[1], raw[2]),
                ColorSpace::Hsv => hsv_to_rgb(raw[0], raw[1], raw[2]),
                ColorSpace::Hls => hls_to_rgb(raw[0], raw[1], raw[2]),
            };

            self.batch_buffer.push(color);
        }

        // Shuffle within the batch for natural distribution
        if self.config.shuffle {
            self.rng.shuffle(&mut self.batch_buffer);
        }
    }

    /// Pull the next color from the palette/stream.
    pub fn next_color(&mut self) -> Color {
        if let Some(colors) = &self.precomputed {
            if colors.is_empty() {
                return Color::BLACK;
            }
            let color = colors[self.color_index];
            self.color_index = (self.color_index + 1) % colors.len();
            color
        } else {
            if self.batch_index >= self.batch_buffer.len() {
                self.refill_batch();
            }
            let color = self.batch_buffer[self.batch_index];
            self.batch_index += 1;
            color
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_count() {
        let config = ColorGeneratorConfig {
            bit_depth: 3, // 8 per channel -> 512 total
            seed: Some(42),
            ..Default::default()
        };
        let colors = generate_colors(&config);
        assert_eq!(colors.len(), 512);
    }

    #[test]
    fn test_reproducible_seed() {
        let config1 = ColorGeneratorConfig {
            bit_depth: 3,
            seed: Some(12345),
            ..Default::default()
        };
        let config2 = ColorGeneratorConfig {
            bit_depth: 3,
            seed: Some(12345),
            ..Default::default()
        };
        let c1 = generate_colors(&config1);
        let c2 = generate_colors(&config2);
        assert_eq!(c1, c2);
    }
}
