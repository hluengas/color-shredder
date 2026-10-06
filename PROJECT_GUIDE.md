# Color Shredder — Comprehensive Source Code & Architecture Guide

Welcome to the **Color Shredder** code review guide. This document provides an exhaustive, file-by-file walkthrough of the entire codebase, explaining the data structures, algorithmic strategies, multiprocessing synchronization models, and infrastructure architecture.

---

## 1. Executive Overview

**Color Shredder** produces organic, tie-dye pixel art from randomized color palettes. Rather than rasterizing pixels along arbitrary geometric curves, Color Shredder simulates organic crystallization:

1. A deterministic or randomized palette of unique colors is synthesized across a specified color space (RGB, HSV, or HLS) at a chosen bit depth.
2. A single seed pixel is placed at starting coordinates `(startX, startY)`.
3. All adjacent uncolored cells (up to 8 neighbors) are registered into an active **frontier** (boundary region).
4. For every subsequent color in the sequence:
   - The algorithm searches the entire active frontier to find the candidate pixel location that minimizes color distance according to one of three mathematical strategies.
   - The color is placed, the candidate is removed from the frontier, and new uncolored neighbors are added.
   - Neighborhood statistics (color averages) are updated.

---

## 2. Repository Layout & File Map

The workspace is organized as a Cargo workspace with shared core logic, a native CLI, a Dioxus Web UI, a high-performance multi-threaded browser bundle, and production Quadlet deployment manifests:

```text
color-shredder/
├── Cargo.toml                                # Workspace root manifest
├── crates/
│   ├── color-shredder-core/                  # Core algorithmic engine & data structures
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                        # Crate root & public exports
│   │       ├── color.rs                      # Color struct, Euclidean metrics & color space math
│   │       ├── canvas.rs                     # 2D Grid, RGBA buffer & neighbor topology
│   │       ├── generator.rs                  # Palette synthesis, channel slicing & shuffling
│   │       ├── spatial.rs                    # 3D R*-tree spatial indexing for Strategy 3
│   │       ├── strategies.rs                 # Strategies 1, 2, 3 & Rayon parallel search
│   │       └── engine.rs                     # Stateful simulation engine & telemetry
│   ├── color-shredder-cli/                   # High-performance native CLI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                       # CLI entrypoint, argument parsing & PNG exporter
│   └── color-shredder-ui/                    # Dioxus WebAssembly UI
│       ├── Cargo.toml
│       └── src/
│           └── main.rs                       # Dioxus reactive components & canvas renderer
├── www/
│   └── index.html                            # In-browser multi-threaded engine with SharedArrayBuffer + Atomics
└── /home/podcommander/repos/umbriel-services/color-shredder-7p4qm/
    ├── systemd/
    │   ├── color-shredder-7p4qm.pod          # Pod definition (pasta networking)
    │   ├── color-shredder-web-7p4qm.container # Nginx web server container
    │   └── color-shredder-tailscale-7p4qm.container # Tailscale ingress sidecar
    └── /mnt/fuzedrive-pool/.../nginx/conf.d/default.conf # COOP / COEP isolation headers
```

---

## 3. Core Algorithmic Strategies

The algorithm supports three distinct placement strategies that produce dramatically different aesthetic results:

| Strategy | Name | Mathematical Heuristic | Aesthetic Output |
| :--- | :--- | :--- | :--- |
| **1** | **Min Neighbor Distance** | $\min_{n \in N} \| C_{\text{target}} - C_n \|^2$ | Gritty, pixelated, high-contrast local borders. |
| **2** | **Average Neighbor Distance** | $\frac{1}{\|N\|} \sum_{n \in N} \| C_{\text{target}} - C_n \|^2$ | Soft interior gradient flow with sharp regional boundaries. |
| **3** | **Neighborhood Color Average** | $\| C_{\text{target}} - \bar{C}_{\text{neighborhood}} \|^2$ | Smooth organic tie-dye crystallization and dendritic growth. Can be accelerated with 3D R*-trees. |

---

## 4. File-by-File Source Code Review

### Part 1: Rust Core (`crates/color-shredder-core`)

#### [1. `crates/color-shredder-core/src/color.rs`](file:///home/podcommander/repos/color-shredder/color-shredder/crates/color-shredder-core/src/color.rs)

- **Purpose**: Defines normalized floating-point RGB colors and color space conversions.
- **Key Types & Functions**:
  - `struct Color { r: f32, g: f32, b: f32 }`: Channels are stored as normalized `f32` in the range `[0.0, 1.0]`.
  - `dist_sq(self, other: Self) -> f32`: Squared Euclidean distance in RGB color space $(\Delta r^2 + \Delta g^2 + \Delta b^2)$. Avoids square root operations during candidate comparisons.
  - `to_rgb8()` and `to_rgba8()`: Converts normalized floats to 8-bit integers (`[u8; 4]`) for display and export.
  - `hsv_to_rgb(h, s, v)`: Converts Hue-Saturation-Value to RGB. Replicates Python's `colorsys.hsv_to_rgb`.
  - `hls_to_rgb(h, l, s)`: Converts Hue-Lightness-Saturation to RGB. Replicates Python's `colorsys.hls_to_rgb`.

#### [2. `crates/color-shredder-core/src/canvas.rs`](file:///home/podcommander/repos/color-shredder/color-shredder/crates/color-shredder-core/src/canvas.rs)

- **Purpose**: Manages the 2D pixel grid, coordinate arithmetic, neighbor topology, and linear RGBA framebuffer.
- **Key Types & Functions**:
  - `struct Coordinate { x: usize, y: usize }`: Lightweight 2D integer position.
  - `struct Canvas`: Encapsulates `width`, `height`, `pixels: Vec<Option<Color>>` (for simulation state), and `rgba_buffer: Vec<u8>` (pre-allocated 4-byte-per-pixel buffer ready for canvas rendering).
  - `neighbors(coord)`: Returns up to 8 surrounding valid coordinates inside canvas boundaries.
  - `colored_neighbors(coord)` / `uncolored_neighbors(coord)`: Filters neighbors based on placement status.
  - `neighborhood_average(coord) -> Option<(usize, Color)>`: Computes the centroid color of all placed neighbors surrounding a frontier coordinate in $O(1)$ time.

#### [3. `crates/color-shredder-core/src/generator.rs`](file:///home/podcommander/repos/color-shredder/color-shredder/crates/color-shredder-core/src/generator.rs)

- **Purpose**: Synthesizes deterministic or randomized unique color palettes.
- **Key Types & Functions**:
  - `struct ColorGeneratorConfig`: Holds `bit_depth` (e.g. 4 for 4,096 colors, 6 for 262,144 colors), `color_space` (RGB/HSV/HLS), `fixed_channel`, `shuffle`, and optional PRNG `seed`.
  - `generate_colors(config) -> Vec<Color>`:
    1. Generates discrete values per channel ($2^{\text{bit\_depth}}$).
    2. Shuffles the primary channel slices.
    3. Synthesizes colors within each slice, optionally holding one channel fixed to produce color banding.
    4. Applies optional full global Fisher-Yates shuffle across all color groups.

#### [4. `crates/color-shredder-core/src/spatial.rs`](file:///home/podcommander/repos/color-shredder/color-shredder/crates/color-shredder-core/src/spatial.rs)

- **Purpose**: Accelerates Strategy 3 (Neighborhood Color Average) using an R*-tree spatial index.
- **Key Types & Functions**:
  - `struct RStarCandidate`: Maps an average neighborhood color point `[r, g, b]` to a canvas `Coordinate`. Implements `rstar::RTreeObject` and `rstar::PointDistance`.
  - `struct RStarIndex`: Wraps `rstar::RTree<RStarCandidate>`.
  - `find_nearest(target_color) -> Option<Coordinate>`: Performs a $k$-NN nearest neighbor query in 3D color space in $O(\log F)$ time instead of an $O(F)$ linear scan, accelerating large canvas generation dramatically.
  - `upsert(coord, canvas)` / `remove(coord)`: Updates candidate positions in the spatial index as neighbors change.

#### [5. `crates/color-shredder-core/src/strategies.rs`](file:///home/podcommander/repos/color-shredder/color-shredder/crates/color-shredder-core/src/strategies.rs)

- **Purpose**: Implements the distance heuristics for Strategies 1, 2, and 3, and coordinates parallel CPU execution via Rayon.
- **Key Types & Functions**:
  - `enum Strategy`: Variants `Min = 1`, `Average = 2`, `Neighborhood = 3`.
  - `calculate_distance(strategy, candidate, target_color, canvas) -> f32`: Calculates candidate distance according to strategy rules.
  - `evaluate_best_candidate(strategy, frontier, target_color, canvas) -> Option<Coordinate>`:
    - **Rayon Multiprocessing**: When `feature = "rayon"` is active and the frontier length $\ge 64$, it executes `frontier.par_iter()` across all available CPU cores.
    - **Sequential Threshold**: For small frontiers ($< 64$ candidates), it executes sequentially on a single thread to eliminate thread scheduling overhead.

#### [6. `crates/color-shredder-core/src/engine.rs`](file:///home/podcommander/repos/color-shredder/color-shredder/crates/color-shredder-core/src/engine.rs)

- **Purpose**: Stateful orchestrator tying canvas, color palette, frontier set, and strategies together.
- **Key Types & Functions**:
  - `struct EngineConfig` & `struct EngineStats`.
  - `struct Engine`: Holds simulation state, `frontier: Vec<Coordinate>`, and fast membership bitmap `frontier_set: Vec<bool>`.
  - `seed()`: Paints the initial seed pixel and expands the first set of frontier neighbors.
  - `step(batch_size) -> EngineStats`: Advances the painting process by placing up to `batch_size` pixels, updating frontier sets, and reporting progress statistics.
  - `run_to_completion()`: Runs continuously until the entire canvas is filled.

#### [7. `crates/color-shredder-core/src/lib.rs`](file:///home/podcommander/repos/color-shredder/color-shredder/crates/color-shredder-core/src/lib.rs)

- **Purpose**: Crate root exporting public APIs, modules, and documentation.

---

### Part 2: Native CLI (`crates/color-shredder-cli`)

#### [8. `crates/color-shredder-cli/src/main.rs`](file:///home/podcommander/repos/color-shredder/color-shredder/crates/color-shredder-cli/src/main.rs)

- **Purpose**: Native command-line tool for Color Shredder with full multi-threading and image export.
- **Key Features**:
  - CLI argument parsing via `clap` (supporting dimensions `-d`, seed `-s`, bit depth `-c`, strategy `-q`, fixed channel `-x`, color spaces `--hsv`/`--hls`, and `--no-rtree`).
  - Terminal progress readout updating placed pixels, percentage, frontier size, and placement rate (px/s).
  - High-performance direct image export to PNG via the `image` crate.

---

### Part 3: Dioxus Web UI (`crates/color-shredder-ui`)

#### [9. `crates/color-shredder-ui/src/main.rs`](file:///home/podcommander/repos/color-shredder/color-shredder/crates/color-shredder-ui/src/main.rs)

- **Purpose**: Modern WebAssembly frontend written with Dioxus.
- **Key Features**:
  - High-precision numeric inputs for all parameters (no sliders).
  - Reactive painter coroutine driving asynchronous simulation steps without blocking browser UI updates.
  - Direct HTML5 `<canvas>` rendering via `web-sys` and `wasm-bindgen::Clamped`.

---

### Part 4: Compiled Rust WebAssembly & Browser Runtime

#### [10. `crates/color-shredder-wasm`](file:///home/podcommander/repos/color-shredder/color-shredder/crates/color-shredder-wasm) & [`www/worker.js`](file:///home/podcommander/repos/color-shredder/color-shredder/www/worker.js)

- **Purpose**: Compiles the core Rust engine into a multi-threaded WebAssembly binary (`color_shredder_wasm_bg.wasm`) with native Rayon support (`wasm-bindgen-rayon`).
- **Architecture**:
  1. **Rust WebAssembly Module (`color-shredder-wasm`)**:
     - Exports `ShredderEngine` and `initThreadPool` directly to JavaScript.
     - Runs the exact same core algorithms (color generation, distance metrics, R*-tree indexing).
     - When `engine.step()` is invoked, candidate search runs natively in parallel across Web Workers via Rayon (`frontier.par_iter()`).
  2. **Dedicated Simulation Worker (`worker.js`)**:
     - Runs the WebAssembly runtime in a background thread, preventing any blocking of the browser UI.
     - Initializes Rayon's Web Worker thread pool with `initThreadPool(numThreads)`.
     - Extracts the linear RGBA pixel buffer directly from WebAssembly linear memory (`wasmModule.memory.buffer`) and transfers frames to the main UI thread via zero-copy transferable `ArrayBuffer`s.
  3. **Lightweight HTML/CSS Frontend (`www/index.html`)**:
     - High-precision numeric inputs, real-time telemetry display, and crisp 60 FPS `<canvas>` blitting.
     - Eliminates over 1,000 lines of manual JavaScript worker strings and memory offsets.

---

### Part 5: Deployment Manifests (`umbriel-services`)

- **[`color-shredder-7p4qm.pod`](file:///home/podcommander/repos/umbriel-services/color-shredder-7p4qm/systemd/color-shredder-7p4qm.pod)**: Quadlet pod configuring private `pasta` IPv4 networking and mounting persistent directories (`/ts-state` and `/usr/share/nginx/html`).
- **[`color-shredder-web-7p4qm.container`](file:///home/podcommander/repos/umbriel-services/color-shredder-7p4qm/systemd/color-shredder-web-7p4qm.container)**: Container running `nginx:alpine` serving the web root.
- **[`color-shredder-tailscale-7p4qm.container`](file:///home/podcommander/repos/umbriel-services/color-shredder-7p4qm/systemd/color-shredder-tailscale-7p4qm.container)**: Tailscale sidecar authenticating with tailnet tag `color-shredder` and terminating HTTPS TLS on `color-shredder.tail607cb2.ts.net:443`.
- **[`default.conf`](file:///mnt/fuzedrive-pool/containers/podcommander/appdata/color-shredder-7p4qm/nginx/conf.d/default.conf)**: Nginx configuration injecting critical cross-origin isolation headers:
  - `Cross-Origin-Opener-Policy: same-origin`
  - `Cross-Origin-Embedder-Policy: require-corp`
  *(These headers enable `SharedArrayBuffer` in modern browsers).*

---

## 5. Build, Run & Verification Commands

### Native Rust Core Tests

```bash
cargo test -p color-shredder-core
```

### Native CLI Execution

```bash
# Generate a 128x128 image with Strategy 3 and 6-bit depth (262k palette)
cargo run --release -p color-shredder-cli -- -d 128 128 -c 6 -q 3 -f output_image
```

### Service Management (on `umbriel`)

```bash
# Check service status
systemctl --user status color-shredder-web-7p4qm.service
systemctl --user status color-shredder-tailscale-7p4qm.service

# View Tailscale sidecar status
podman exec color-shredder-tailscale-7p4qm tailscale status
```

---

## 6. Accessing the Live Web Application

- **Tailscale URL**: [https://color-shredder.tail607cb2.ts.net/](https://color-shredder.tail607cb2.ts.net/)
- **Features**:
  - Precision numeric inputs (no sliders).
  - Detected hardware concurrency with selectable CPU threads (1–128).
  - Uncapped maximum speed mode saturating all CPU threads.
  - Live 60 FPS pixel rendering, real-time stats (placed, frontier, %, rate), and instant PNG export.
