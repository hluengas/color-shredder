use color_shredder_core::{
    ColorGeneratorConfig, ColorSpace, Coordinate, Engine, EngineConfig, Strategy,
};
use wasm_bindgen::prelude::*;

// Export the Rayon thread pool initializer directly to JavaScript
pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub struct ShredderEngine {
    engine: Engine,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct WasmStats {
    pub pixels_placed: usize,
    pub total_pixels: usize,
    pub pixels_available: usize,
    pub percent_complete: f32,
    pub is_finished: bool,
}

#[wasm_bindgen]
impl ShredderEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(
        width: usize,
        height: usize,
        start_x: usize,
        start_y: usize,
        strategy_num: u32,
        bit_depth: u32,
        color_space_str: &str,
        fixed_channel_opt: Option<usize>,
        shuffle: bool,
        seed_opt: Option<f64>,
    ) -> Self {
        let strategy = Strategy::from_u32(strategy_num).unwrap_or(Strategy::Neighborhood);
        let color_space = match color_space_str {
            "hsv" => ColorSpace::Hsv,
            "hls" => ColorSpace::Hls,
            _ => ColorSpace::Rgb,
        };

        let seed = seed_opt.map(|s| s as u64);

        let config = EngineConfig {
            width,
            height,
            start_point: Coordinate::new(start_x, start_y),
            strategy,
            use_rstar: true,
            color_config: ColorGeneratorConfig {
                bit_depth,
                color_space,
                fixed_channel: fixed_channel_opt,
                shuffle,
                seed,
            },
        };

        Self {
            engine: Engine::new(config),
        }
    }

    /// Advance the simulation by up to `batch_size` pixels.
    /// Internal candidate search utilizes Rayon multithreading across Web Workers.
    pub fn step(&mut self, batch_size: usize) -> WasmStats {
        let stats = self.engine.step(batch_size);
        WasmStats {
            pixels_placed: stats.pixels_placed,
            total_pixels: stats.total_pixels,
            pixels_available: stats.pixels_available,
            percent_complete: stats.percent_complete,
            is_finished: stats.is_finished,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.engine.is_finished()
    }

    /// Pointer to the linear RGBA pixel byte buffer in WebAssembly linear memory.
    pub fn get_rgba_ptr(&self) -> *const u8 {
        self.engine.rgba_buffer().as_ptr()
    }

    /// Total byte length of the linear RGBA pixel buffer (width * height * 4).
    pub fn get_rgba_len(&self) -> usize {
        self.engine.rgba_buffer().len()
    }
}
