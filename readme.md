# Color Shredder

A high-performance Rust rewrite and WebAssembly/Dioxus application for generating tie-dye pixel art paintings from randomized color sets.

Inspired by the original Code Golf challenge *"Images with all Colors"* and based on the research paper [*"Tie-Dye Pixel Art Generation and Acceleration"*](https://luengas.dev/assets/documents/Tie_Dye_Pixel_Art_Generation.pdf) (Henry Luengas & Maria Pantoja, Cal Poly SLO).

---

## Architecture & Crates

The codebase is structured as a modular Cargo workspace:

* [`crates/color-shredder-core`](crates/color-shredder-core): Pure-Rust algorithmic engine.
  * **Color Model**: Normalized floating-point channels (`Color { r, g, b: f32 }` in $[0.0, 1.0]$) resolving distance squaring overflow issues and enabling precise color-space conversions (RGB, HSV, HLS).
  * **2D Canvas**: Contiguous linear RGBA buffer enabling zero-copy canvas painting and image output.
  * **Spatial Acceleration**: 3D RGB nearest-neighbor search powered by an in-memory R*-tree ([`rstar`](https://crates.io/crates/rstar)), providing $O(\log N)$ placement lookups for Strategy 3.
  * **Stateful Engine**: Step-by-step batched generator state machine that yields seamlessly to browser rendering loops.
* [`crates/color-shredder-cli`](crates/color-shredder-cli): Native command-line tool with full argument parity with the legacy implementation, live terminal statistics, and instant PNG export.
* [`crates/color-shredder-ui`](crates/color-shredder-ui): Cross-platform interactive application built with [Dioxus 0.6](https://dioxuslabs.com), targeting **Web (WebAssembly)** and **Native Desktop**.

---

## Placement Strategies

1. **Strategy 1: Minimum Color Distance (`-q 1`)**
   $$\min_{p \in \text{Frontier}} \left( \min_{n \in \text{Neighbors}(p)} \|C_{\text{target}} - C_n\|^2 \right)$$
   * Produces gritty, high-frequency, highly pixelated textures.
   * Slow frontier growth.
2. **Strategy 2: Average Color Distance (`-q 2`)**
   $$\min_{p \in \text{Frontier}} \left( \frac{1}{|\text{Neighbors}(p)|} \sum_{n \in \text{Neighbors}(p)} \|C_{\text{target}} - C_n\|^2 \right)$$
   * Produces soft flowing color tracks with distinct, high-contrast borders.
   * Rapid quadratic frontier growth.
3. **Strategy 3: Neighborhood Average Color (`-q 3`, Default)**
   $$\bar{C}_p = \frac{1}{|\text{Neighbors}(p)|} \sum_{n \in \text{Neighbors}(p)} C_n, \quad \min_{p \in \text{Frontier}} \|C_{\text{target}} - \bar{C}_p\|^2$$
   * Produces smooth gradient flows with circular boundary fronts.
   * **Accelerated via R*-tree 3D nearest-neighbor indexing**.

---

## Quick Start

### 1. Running the Native CLI

```bash
# Basic 64x64 render with default Neighborhood strategy
cargo run --release -p color-shredder-cli -- -d 64 64 -s 32 32 -c 6 -q 3 -f output

# HSV color space with a 128x128 canvas
cargo run --release -p color-shredder-cli -- -d 128 128 -s 64 64 -c 6 --hsv -f tie-dye-hsv

# Group colors along channel 1 (un-shuffled slice)
cargo run --release -p color-shredder-cli -- -d 64 64 -c 6 -x 1 -f grouped-channel
```

CLI options:
* `-d <width> <height>`: Canvas dimensions (default: `64 64`)
* `-s <x> <y>`: Starting coordinate (default: `0 0`)
* `-c <depth>`: Color bit depth per channel (e.g. `6` for $262\text{k}$ colors, `8` for $16.7\text{M}$)
* `-q <1|2|3>`: Placement strategy (`1`: Min, `2`: Average, `3`: Neighborhood)
* `-x <1|2|3>`: Leave a color channel grouped / un-shuffled
* `--hsv`: Synthesize colors in HSV space
* `--hls`: Synthesize colors in HLS space
* `--no-rtree`: Disable R*-tree acceleration (linear frontier scan)
* `-f <name>`: Output PNG filename prefix
* `--seed <seed>`: PRNG seed for deterministic runs

### 2. Running the Dioxus WebApp

Using the Dioxus CLI (`dx`):

```bash
# Serve locally with live reloading in your browser
dx serve --package color-shredder-ui
```

### 3. Running Unit Tests

```bash
cargo test --workspace
```

---

## Legacy Codebase Reference

* The legacy Python and OpenCL implementation is preserved in `/home/podcommander/repos/color-shredder/color-shredder-legacy`.
* Academic report: [*"Tie-Dye Pixel Art Generation and Acceleration"*](https://luengas.dev/assets/documents/Tie_Dye_Pixel_Art_Generation.pdf).
