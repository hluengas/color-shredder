use crate::color::{hls_to_rgb, hsv_to_rgb, Color, ColorSpace};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Configuration for color palette generation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ColorGeneratorConfig {
    /// Bit depth per channel (e.g. 4 for 4096 colors, 6 for 262,144 colors, 8 for 16,777,216 colors).
    pub bit_depth: u32,
    /// Color space used during color synthesis.
    pub color_space: ColorSpace,
    /// If Some(channel_idx), leaves that channel (0, 1, or 2) grouped/unshuffled.
    pub fixed_channel: Option<usize>,
    /// Whether to perform the final full shuffle across all color groups.
    pub shuffle: bool,
    /// Optional PRNG seed for deterministic runs.
    pub seed: Option<u64>,
}

impl Default for ColorGeneratorConfig {
    fn default() -> Self {
        Self {
            bit_depth: 6,
            color_space: ColorSpace::Rgb,
            fixed_channel: None,
            shuffle: true,
            seed: None,
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
