//! # Color Shredder Dioxus Web UI
//!
//! Interactive WebAssembly frontend built with the Dioxus framework.
//! Provides numeric input controls (no sliders), real-time HTML5 `<canvas>` rendering,
//! live progress metrics, and strategy selection.

use color_shredder_core::{
    ColorGeneratorConfig, ColorSpace, Coordinate, Engine, EngineConfig, EngineStats, Strategy,
};
use dioxus::prelude::*;

fn main() {
    // Launch the Dioxus web application
    dioxus::launch(App);
}

/// Root application component managing simulation state and rendering controls.
#[component]
fn App() -> Element {
    // Reactive signals for simulation parameters
    let mut width = use_signal(|| 64usize);
    let mut height = use_signal(|| 64usize);
    let mut start_x = use_signal(|| 32usize);
    let mut start_y = use_signal(|| 32usize);
    let mut bit_depth = use_signal(|| 6u32);
    let mut strategy = use_signal(|| 3u32);
    let mut color_space_idx = use_signal(|| 0u32); // 0: RGB, 1: HSV, 2: HLS
    let mut fixed_channel_idx = use_signal(|| 0u32); // 0: None, 1: Chan 1, 2: Chan 2, 3: Chan 3
    let mut shuffle = use_signal(|| true);
    let mut seed_str = use_signal(|| String::new());
    let mut batch_size = use_signal(|| 100usize);
    let mut is_running = use_signal(|| false);

    // Live execution telemetry
    let mut stats = use_signal(|| EngineStats {
        pixels_placed: 0,
        total_pixels: 4096,
        pixels_available: 0,
        percent_complete: 0.0,
        is_finished: false,
    });

    // Reactive coroutine driving the simulation steps asynchronously without blocking the browser event loop
    let _painter = use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let mut current_engine: Option<Engine> = None;

        loop {
            let running = is_running();

            if running {
                // Initialize engine instance if starting fresh
                if current_engine.is_none() {
                    let c_space = match color_space_idx() {
                        1 => ColorSpace::Hsv,
                        2 => ColorSpace::Hls,
                        _ => ColorSpace::Rgb,
                    };

                    let strat = Strategy::from_u32(strategy()).unwrap_or(Strategy::Neighborhood);

                    let fixed_chan = match fixed_channel_idx() {
                        1 => Some(0),
                        2 => Some(1),
                        3 => Some(2),
                        _ => None,
                    };

                    let seed_val = seed_str().trim().parse::<u64>().ok();

                    let w = width();
                    let h = height();
                    let sx = start_x().min(w.saturating_sub(1));
                    let sy = start_y().min(h.saturating_sub(1));

                    let cfg = EngineConfig {
                        width: w,
                        height: h,
                        start_point: Coordinate::new(sx, sy),
                        strategy: strat,
                        use_rstar: true,
                        color_config: ColorGeneratorConfig {
                            bit_depth: bit_depth(),
                            color_space: c_space,
                            fixed_channel: fixed_chan,
                            shuffle: shuffle(),
                            seed: seed_val,
                        },
                    };
                    current_engine = Some(Engine::new(cfg));
                }

                // Advance batch of pixels and update canvas
                if let Some(engine) = &mut current_engine {
                    if !engine.is_finished() {
                        let step_stats = engine.step(batch_size());
                        stats.set(step_stats);

                        #[cfg(target_arch = "wasm32")]
                        {
                            render_canvas(engine.rgba_buffer(), width(), height());
                        }

                        if step_stats.is_finished {
                            is_running.set(false);
                        }
                    } else {
                        is_running.set(false);
                    }
                }

                // Yield briefly to let the browser repaint and process user input
                gloo_timers::future::TimeoutFuture::new(1).await;
            } else {
                // Idle polling interval when paused
                gloo_timers::future::TimeoutFuture::new(50).await;
            }
        }
    });

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; font-family: system-ui, -apple-system, sans-serif; background: #090d16; color: #f8fafc; min-height: 100vh; padding: 20px;",

            header {
                style: "text-align: center; margin-bottom: 20px;",
                h1 { style: "font-size: 2.1rem; font-weight: 800; margin: 0; background: linear-gradient(135deg, #38bdf8, #818cf8, #c084fc); -webkit-background-clip: text; -webkit-text-fill-color: transparent;", "Color Shredder" }
                p { style: "color: #94a3b8; margin: 4px 0 0 0; font-size: 0.9rem;", "Tie-Dye Pixel Art Generation in Rust & WebAssembly" }
            }

            div {
                style: "display: flex; gap: 24px; max-width: 1150px; width: 100%; justify-content: center; flex-wrap: wrap;",

                // Canvas Viewport Card
                div {
                    style: "background: #131b2e; border-radius: 14px; padding: 18px; box-shadow: 0 10px 30px -5px rgba(0,0,0,0.6); border: 1px solid #23314d; display: flex; flex-direction: column; align-items: center;",
                    div {
                        style: "width: 512px; height: 512px; background: #020617; border-radius: 10px; overflow: hidden; display: flex; align-items: center; justify-content: center; border: 2px solid #23314d; position: relative;",
                        canvas {
                            id: "shredder-canvas",
                            width: "{width}",
                            height: "{height}",
                            style: "width: 100%; height: 100%; image-rendering: pixelated;",
                        }
                    }

                    div {
                        style: "width: 100%; margin-top: 14px; display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; text-align: center;",
                        div {
                            style: "background: #090d16; padding: 8px 10px; border-radius: 8px; border: 1px solid #23314d;",
                            div { style: "font-size: 0.7rem; color: #64748b; text-transform: uppercase; font-weight: 600;", "Placed" }
                            div { style: "font-size: 1.05rem; font-weight: 700; color: #38bdf8; font-variant-numeric: tabular-nums;", "{stats().pixels_placed} / {stats().total_pixels}" }
                        }
                        div {
                            style: "background: #090d16; padding: 8px 10px; border-radius: 8px; border: 1px solid #23314d;",
                            div { style: "font-size: 0.7rem; color: #64748b; text-transform: uppercase; font-weight: 600;", "Frontier" }
                            div { style: "font-size: 1.05rem; font-weight: 700; color: #c084fc; font-variant-numeric: tabular-nums;", "{stats().pixels_available}" }
                        }
                        div {
                            style: "background: #090d16; padding: 8px 10px; border-radius: 8px; border: 1px solid #23314d;",
                            div { style: "font-size: 0.7rem; color: #64748b; text-transform: uppercase; font-weight: 600;", "Progress" }
                            div { style: "font-size: 1.05rem; font-weight: 700; color: #4ade80; font-variant-numeric: tabular-nums;", "{stats().percent_complete:.1}%" }
                        }
                    }
                }

                // Controls Panel (high-precision inputs, no sliders)
                div {
                    style: "background: #131b2e; border-radius: 14px; padding: 20px; box-shadow: 0 10px 30px -5px rgba(0,0,0,0.6); border: 1px solid #23314d; width: 380px; display: flex; flex-direction: column; gap: 14px;",

                    h2 { style: "font-size: 1.15rem; font-weight: 700; margin: 0; color: #f1f5f9; border-bottom: 1px solid #23314d; padding-bottom: 8px;", "Precise Simulation Settings" }

                    // Geometry inputs
                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px;",
                        div {
                            label { style: "display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 4px;", "Width [px]" }
                            input {
                                r#type: "number",
                                min: "2",
                                max: "2048",
                                step: "1",
                                value: "{width}",
                                style: "width: 100%; padding: 8px; background: #0b1120; border: 1px solid #334155; border-radius: 6px; color: #f8fafc;",
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<usize>() {
                                        width.set(val);
                                    }
                                }
                            }
                        }
                        div {
                            label { style: "display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 4px;", "Height [px]" }
                            input {
                                r#type: "number",
                                min: "2",
                                max: "2048",
                                step: "1",
                                value: "{height}",
                                style: "width: 100%; padding: 8px; background: #0b1120; border: 1px solid #334155; border-radius: 6px; color: #f8fafc;",
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<usize>() {
                                        height.set(val);
                                    }
                                }
                            }
                        }
                    }

                    // Seed Coordinates
                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px;",
                        div {
                            label { style: "display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 4px;", "Start X [px]" }
                            input {
                                r#type: "number",
                                min: "0",
                                max: "2047",
                                step: "1",
                                value: "{start_x}",
                                style: "width: 100%; padding: 8px; background: #0b1120; border: 1px solid #334155; border-radius: 6px; color: #f8fafc;",
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<usize>() {
                                        start_x.set(val);
                                    }
                                }
                            }
                        }
                        div {
                            label { style: "display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 4px;", "Start Y [px]" }
                            input {
                                r#type: "number",
                                min: "0",
                                max: "2047",
                                step: "1",
                                value: "{start_y}",
                                style: "width: 100%; padding: 8px; background: #0b1120; border: 1px solid #334155; border-radius: 6px; color: #f8fafc;",
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<usize>() {
                                        start_y.set(val);
                                    }
                                }
                            }
                        }
                    }

                    // Strategy Selector
                    div {
                        label { style: "display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 4px;", "Placement Strategy" }
                        select {
                            style: "width: 100%; padding: 8px; background: #0b1120; border: 1px solid #334155; border-radius: 6px; color: #f8fafc;",
                            value: "{strategy}",
                            onchange: move |evt| {
                                if let Ok(val) = evt.value().parse::<u32>() {
                                    strategy.set(val);
                                }
                            },
                            option { value: "3", "3: Neighborhood Average Color" }
                            option { value: "2", "2: Average Neighbor Distance" }
                            option { value: "1", "1: Min Neighbor Distance" }
                        }
                    }

                    // Color Space & Bit Depth
                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px;",
                        div {
                            label { style: "display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 4px;", "Color Space" }
                            select {
                                style: "width: 100%; padding: 8px; background: #0b1120; border: 1px solid #334155; border-radius: 6px; color: #f8fafc;",
                                value: "{color_space_idx}",
                                onchange: move |evt| {
                                    if let Ok(val) = evt.value().parse::<u32>() {
                                        color_space_idx.set(val);
                                    }
                                },
                                option { value: "0", "RGB" }
                                option { value: "1", "HSV" }
                                option { value: "2", "HLS" }
                            }
                        }
                        div {
                            label { style: "display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 4px;", "Bit Depth [1-8]" }
                            input {
                                r#type: "number",
                                min: "1",
                                max: "8",
                                step: "1",
                                value: "{bit_depth}",
                                style: "width: 100%; padding: 8px; background: #0b1120; border: 1px solid #334155; border-radius: 6px; color: #f8fafc;",
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<u32>() {
                                        bit_depth.set(val);
                                    }
                                }
                            }
                        }
                    }

                    // Fixed Channel & PRNG Seed
                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px;",
                        div {
                            label { style: "display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 4px;", "Fixed Channel (-x)" }
                            select {
                                style: "width: 100%; padding: 8px; background: #0b1120; border: 1px solid #334155; border-radius: 6px; color: #f8fafc;",
                                value: "{fixed_channel_idx}",
                                onchange: move |evt| {
                                    if let Ok(val) = evt.value().parse::<u32>() {
                                        fixed_channel_idx.set(val);
                                    }
                                },
                                option { value: "0", "None" }
                                option { value: "1", "Channel 1" }
                                option { value: "2", "Channel 2" }
                                option { value: "3", "Channel 3" }
                            }
                        }
                        div {
                            label { style: "display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 4px;", "PRNG Seed" }
                            input {
                                r#type: "number",
                                placeholder: "Random",
                                value: "{seed_str}",
                                style: "width: 100%; padding: 8px; background: #0b1120; border: 1px solid #334155; border-radius: 6px; color: #f8fafc;",
                                oninput: move |evt| {
                                    seed_str.set(evt.value());
                                }
                            }
                        }
                    }

                    // Execution Tuning
                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px; align-items: center;",
                        div {
                            label { style: "display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 4px;", "Batch Speed [px/frame]" }
                            input {
                                r#type: "number",
                                min: "1",
                                max: "50000",
                                step: "10",
                                value: "{batch_size}",
                                style: "width: 100%; padding: 8px; background: #0b1120; border: 1px solid #334155; border-radius: 6px; color: #f8fafc;",
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<usize>() {
                                        batch_size.set(val);
                                    }
                                }
                            }
                        }
                        div {
                            label {
                                style: "display: flex; align-items: center; gap: 8px; font-size: 0.8rem; color: #f8fafc; cursor: pointer; margin-top: 18px;",
                                input {
                                    r#type: "checkbox",
                                    checked: "{shuffle}",
                                    onchange: move |evt| {
                                        shuffle.set(evt.value() == "true");
                                    }
                                }
                                "Full Shuffle"
                            }
                        }
                    }

                    // Action Buttons
                    div {
                        style: "display: flex; gap: 10px; margin-top: 6px;",
                        button {
                            style: "flex: 1; padding: 10px; border-radius: 6px; border: none; font-weight: 700; cursor: pointer; background: #38bdf8; color: #020617;",
                            onclick: move |_| {
                                is_running.set(!is_running());
                            },
                            if is_running() { "Pause" } else { "Start" }
                        }
                        button {
                            style: "flex: 1; padding: 10px; border-radius: 6px; border: 1px solid #475569; font-weight: 700; cursor: pointer; background: #0b1120; color: #f8fafc;",
                            onclick: move |_| {
                                is_running.set(false);
                            },
                            "Reset"
                        }
                    }
                }
            }
        }
    }
}

/// Blits the linear RGBA byte slice to the HTML5 `<canvas>` using clamped array memory.
#[cfg(target_arch = "wasm32")]
fn render_canvas(rgba: &[u8], width: usize, height: usize) {
    use wasm_bindgen::Clamped;
    use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d, ImageData};

    let Some(window) = window() else { return };
    let Some(document) = window.document() else { return };
    let Some(element) = document.get_element_by_id("shredder-canvas") else { return };
    let Ok(canvas) = element.dyn_into::<HtmlCanvasElement>() else { return };
    let Ok(Some(obj)) = canvas.get_context("2d") else { return };
    let Ok(ctx) = obj.dyn_into::<CanvasRenderingContext2d>() else { return };

    let clamped = Clamped(rgba);
    if let Ok(image_data) = ImageData::new_with_u8_clamped_array_and_sh(clamped, width as u32, height as u32) {
        let _ = ctx.put_image_data(&image_data, 0.0, 0.0);
    }
}
