//! # Color Shredder CLI
//!
//! Native command-line interface for generating tie-dye pixel art with Color Shredder.
//! Supports native multi-threading via Rayon, R*-tree spatial acceleration, customizable
//! color bit depths, color spaces (RGB, HSV, HLS), and direct PNG image export.

use clap::Parser;
use color_shredder_core::{
    ColorGeneratorConfig, ColorSpace, Coordinate, Engine, EngineConfig, Strategy,
};
use std::path::Path;
use std::time::Instant;

/// Command-line arguments for the Color Shredder native CLI.
#[derive(Parser, Debug)]
#[command(
    name = "color-shredder",
    about = "The Color Shredder chooses colors from a randomized set, placing them where they fit best."
)]
struct Args {
    /// Canvas dimensions [width, height] in pixels.
    #[arg(short = 'd', num_args = 2, default_values_t = [64, 64])]
    dimensions: Vec<usize>,

    /// Starting coordinates [x, y] for the seed pixel.
    #[arg(short = 's', num_args = 2, default_values_t = [0, 0])]
    start: Vec<usize>,

    /// Color space bit depth per channel (e.g. 4 for 4,096 colors, 6 for 262k, 8 for 16.7M).
    #[arg(short = 'c', default_value_t = 6)]
    bit_depth: u32,

    /// Best-fit strategy: 1 (min neighbor distance), 2 (average neighbor distance), 3 (neighborhood average color).
    #[arg(short = 'q', default_value_t = 3)]
    strategy: u32,

    /// Leave a color channel (1, 2, or 3) grouped and un-shuffled.
    #[arg(short = 'x')]
    fixed_channel: Option<usize>,

    /// Generate colors using HSV (Hue, Saturation, Value) color space.
    #[arg(long = "hsv")]
    hsv: bool,

    /// Generate colors using HLS (Hue, Lightness, Saturation) color space.
    #[arg(long = "hls")]
    hls: bool,

    /// Disable R*-tree spatial index acceleration for Strategy 3 (falls back to parallel brute-force search).
    #[arg(long = "no-rtree")]
    no_rtree: bool,

    /// Output filename prefix (without .png extension).
    #[arg(short = 'f', default_value = "painting")]
    output: String,

    /// Optional 64-bit PRNG seed for deterministic runs.
    #[arg(long = "seed")]
    seed: Option<u64>,
}

fn main() {
    let args = Args::parse();

    let width = args.dimensions[0];
    let height = args.dimensions[1];
    let start_x = args.start[0];
    let start_y = args.start[1];

    // Map numeric strategy to enum
    let strategy = Strategy::from_u32(args.strategy).unwrap_or(Strategy::Neighborhood);

    // Resolve color space
    let color_space = if args.hls {
        ColorSpace::Hls
    } else if args.hsv {
        ColorSpace::Hsv
    } else {
        ColorSpace::Rgb
    };

    // Convert 1-based channel option to 0-based index
    let fixed_channel = args.fixed_channel.and_then(|c| {
        if (1..=3).contains(&c) {
            Some(c - 1)
        } else {
            None
        }
    });

    let color_config = ColorGeneratorConfig {
        bit_depth: args.bit_depth,
        color_space,
        fixed_channel,
        shuffle: fixed_channel.is_none(),
        seed: args.seed,
    };

    let engine_config = EngineConfig {
        width,
        height,
        start_point: Coordinate::new(start_x, start_y),
        strategy,
        use_rstar: !args.no_rtree,
        color_config,
    };

    println!(
        "Initializing Color Shredder ({}x{}, strategy: {:?}, bit-depth: {}, space: {:?})...",
        width, height, strategy, args.bit_depth, color_space
    );

    let mut engine = Engine::new(engine_config);
    println!("Palette generated. Painting canvas...");

    let start_time = Instant::now();
    let mut last_print = Instant::now();

    // Advance the simulation in batches until all canvas pixels are filled
    while !engine.is_finished() {
        let stats = engine.step(500);

        // Update progress readout roughly 4 times per second
        if last_print.elapsed().as_millis() >= 250 || stats.is_finished {
            let elapsed = start_time.elapsed().as_secs_f32();
            let rate = if elapsed > 0.0 {
                stats.pixels_placed as f32 / elapsed
            } else {
                0.0
            };

            print!(
                "\rPixels: {}/{} ({:.1}%) | Frontier: {} | Rate: {:.1} px/s",
                stats.pixels_placed, stats.total_pixels, stats.percent_complete, stats.pixels_available, rate
            );
            use std::io::Write;
            std::io::stdout().flush().unwrap();
            last_print = Instant::now();
        }
    }

    println!();
    let total_elapsed = start_time.elapsed();
    println!(
        "Painting completed in {:.2?} ({:.1} px/s)!",
        total_elapsed,
        (width * height) as f32 / total_elapsed.as_secs_f32()
    );

    // Save final rendered buffer to PNG
    let png_path = format!("{}.png", args.output);
    println!("Saving image to {}...", png_path);
    let buffer = engine.rgba_buffer();
    image::save_buffer(
        Path::new(&png_path),
        buffer,
        width as u32,
        height as u32,
        image::ExtendedColorType::Rgba8,
    )
    .expect("Failed to save output PNG");

    println!("Done!");
}
