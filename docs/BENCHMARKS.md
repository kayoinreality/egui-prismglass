# Benchmarks, Visual Regression & Memory Architecture

`egui-prismglass` ships with **reproducible, zero-external-harness benchmarks** and a **12-test automated visual rendering & shader regression matrix** so every rendering change can be measured on your own hardware and verified in CI.

---

## 1. One-Command Reproduction

```bash
# 1. Automated visual rendering, clipping, DPI, edge-artifact & shader regression matrix (12 tests)
cargo test -p egui-prismglass --features blur-wgpu --test visual_regression

# Refresh golden PNG baselines after intentional visual upgrades:
PRISMGLASS_UPDATE_SNAPSHOTS=1 cargo test -p egui-prismglass --features blur-wgpu --test visual_regression

# 2. CPU surface, token, backdrop-request & spring physics micro-benchmarks
cargo bench -p egui-prismglass --bench surface_bench

# 3. End-to-end GPU Dual-Kawase blur, multi-surface scaling (1..64) & VRAM/cache audit
cargo run --release -p egui-prismglass --features blur-wgpu --example gpu_benchmark

# Machine-readable JSON output + custom frame count (recommended on discrete/integrated GPUs):
cargo run --release -p egui-prismglass --features blur-wgpu --example gpu_benchmark -- --frames 30 --json
```

---

## 2. Automated Visual Rendering Test Matrix (`visual_regression.rs`)

Implemented in [`crates/egui-prismglass/tests/visual_regression.rs`](../crates/egui-prismglass/tests/visual_regression.rs) and executed on every CI push/PR on Linux via headless `egui_kittest` + `wgpu` (`mesa-vulkan-drivers`):

| Category | Test Case | What It Verifies |
| :--- | :--- | :--- |
| **Golden Snapshots & Determinism** | `frame_rendering_is_bit_exact_deterministic` | Two independent harnesses produce 100% bit-identical RGBA8 frames. |
| **Golden Snapshots & Determinism** | `golden_snapshots_match_baseline` | Compares 6 rendered fixtures (`glacial_liquid_glass`, `onyx_dark_glass`, `frost_light_glass`, `tinted_pills_and_corners`, `neumorphic_minimal_light`, `neumorphic_minimal_dark`) against committed PNGs in `tests/snapshots/` (`MAE <= 1.5/255`, `<= 0.5%` pixels with channel delta `> 4`). |
| **Clipping & Rounding** | `scroll_and_clip_rect_have_zero_pixel_bleed` | Overflows a `GlassCard` past `ui.set_clip_rect(...)` + `ScrollArea`; verifies **0 leaked pixels** outside the scissor boundary. |
| **Clipping & Rounding** | `corner_rounding_zero_standard_and_pill_clamped` | Isolates the WGSL analytic rounded-rect SDF at `0 pt`, `24 pt`, and `999 pt` (clamped pill), proving outer corner pixels outside the arc remain pixel-identical to the unblurred backdrop. |
| **DPI Scaling** | `dpi_scaling_matrix_1x_to_2_5x` | Renders across `1.0x`, `1.5x`, `2.0x`, and `2.5x` `pixels_per_point`; checks physical pyramid dimensions, exact VRAM formula, and checkerboard variance smoothing at all 4 scales. |
| **Resolution & Resize** | `resolution_matrix_and_dynamic_resize` | Steps through `320×240 -> 640×360 -> 800×600 -> 1280×720` on a single `wgpu::Device`, asserting exact 6-texture VRAM accounting and strictly `1` allocation per resize (`0` on stationary frames). |
| **Edge Artifacts** | `screen_boundary_cards_have_no_edge_artifacts` | Overhangs 4 cards (`-8 pt`) across all 4 viewport edges at max `refraction = 1.0`, `dispersion = 1.0`; verifies opaque alpha `255` everywhere and zero UV wrap-around / black border / NaN pixels. |
| **Shader Optics** | `blur_radius_monotonicity` | Verifies strictly decreasing luminance variance across `0.5 pt` (sharp) -> `10 pt` (medium Kawase) -> `32 pt` (deep 4-level Kawase). |
| **Shader Optics** | `snell_refraction_and_chromatic_dispersion_invariants` | Proves curved-bezel Snell refraction shifts vertical stripes along the bezel band (`refraction 1.0 vs 0.0`) and RGB dispersion widens `|R - B|` color fringe (`dispersion 1.0 vs 0.0`). |
| **Shader Optics** | `beer_lambert_tinted_glass_transmission` | Proves Ruby and Cobalt glass cards transmit their respective wavelengths over a neutral white backdrop. |
| **Fallback Parity** | `fallback_toggle_switches_cleanly_between_gpu_and_cpu` | Toggles `real_blur_enabled` `true -> false -> true`, verifying `BlurCapability` transitions and bit-exact restoration when GPU blur is re-enabled. |
| **Memory & Cache Reuse** | `intermediate_texture_reuse_and_static_pyramid_caching` | Proves 10 static frames execute only `1` pyramid pass (`9` cache hits, `0` reallocs), 5 animated frames execute `5` passes with `0` reallocs, and growing from `20` to `40` cards grows the uniform buffer once (`1 -> 2`). |

---

## 3. CPU Micro-Benchmarks (`surface_bench`)

Measured via `cargo bench -p egui-prismglass --bench surface_bench` (release profile, 60 samples after warmup):

| Benchmark | Mean | p50 | p95 | Throughput |
| :--- | ---: | ---: | ---: | ---: |
| `PrismTheme::surface(Elevated)` | 124.0 ns | 123.6 ns | 133.4 ns | 8,062.7k ops/s |
| `BackdropRequest::from_tokens_with_mode` | 62.6 ns | 62.1 ns | 76.6 ns | 15,978.5k ops/s |
| `glass_surface_shapes (Glass Elevated)` | 13.33 µs | 13.22 µs | 14.73 µs | 75.0k ops/s |
| `glass_surface_shapes (Glass Floating)` | 13.16 µs | 13.08 µs | 13.83 µs | 76.0k ops/s |
| `glass_surface_shapes (Neumorphic Light)` | 12.38 µs | 12.32 µs | 13.20 µs | 80.8k ops/s |
| `glass_surface_shapes (Neumorphic Dark)` | 12.91 µs | 12.64 µs | 15.18 µs | 77.5k ops/s |
| `SpringState::step (1,000 springs/frame)` | 9.44 µs | 9.41 µs | 10.58 µs | 105.9k ops/s |

**Takeaway**: CPU-side styling and layout preparation adds **< 0.015 ms per fallback/neumorphic surface** and **< 0.0002 ms per GPU glass surface** (only 62.6 ns to build a `BackdropRequest`), leaving >99% of a 120 Hz (`8.33 ms`) frame budget for application logic.

---

## 4. GPU & Multi-Surface Scaling Baseline (`gpu_benchmark`)

> **Important Note on Hardware Context**: The reference numbers below were captured in our headless Linux CI environment running **software Vulkan (`llvmpipe`, LLVM 20.1.2, 256-bit AVX2 CPU rasterizer)** with synchronous full-frame RGBA8 readback (`egui_kittest`).
> Software rasterization is **50×–200× slower** in raw milliseconds than real hardware GPUs, but it preserves **exact architectural scaling ratios** (`O(W·H)` blur scaling, `O(1)` pyramid reuse across N surfaces, exact VRAM byte counts, and cache hit ratios). Run the exact command in §1 on your machine to measure your own GPU.

### Environment
- **Adapter**: `llvmpipe (LLVM 20.1.2, 256 bits)` (`DeviceType::Cpu`, `Backend::Vulkan`)
- **Driver**: `Mesa 25.2.8`
- **Build**: `--release`, 5 measured frames after 2 warmup frames

### 4.1 Dual-Kawase Blur Cost & VRAM Across Resolutions (8 Simultaneous `GlassCard`s)

| Resolution | Animated Backdrop (mean / p50 / p95) | Static Cached Backdrop (mean / p50 / p95) | Pure Blur Pyramid Delta | Pyramid VRAM (6 tex) | Allocs |
| :--- | ---: | ---: | ---: | ---: | ---: |
| 720p (`1280×720`) | 91.06 / 88.67 / 95.80 ms | 43.31 / 42.48 / 47.82 ms | **47.74 ms** | 11.12 MiB (6 tex) | 1 |
| 900p (`1440×900`) | 119.96 / 118.41 / 127.37 ms | 52.68 / 52.66 / 53.03 ms | **67.27 ms** | 15.64 MiB (6 tex) | 1 |
| 1080p (`1920×1080`) | 182.78 / 181.33 / 187.29 ms | 80.72 / 80.84 / 84.10 ms | **102.06 ms** | 25.03 MiB (6 tex) | 1 |
| 1440p (`2560×1440`) | 323.86 / 321.12 / 335.85 ms | 132.13 / 130.25 / 137.10 ms | **191.73 ms** | 44.49 MiB (6 tex) | 1 |

**Key scaling verification**:
- Pixel count ratio `1440p / 720p` = **4.000×**.
- Measured Pure Blur Pyramid Delta `1440p / 720p` = `191.73 / 47.74` = **4.016×** (strict `O(W·H)` pixel-linear scaling).
- Measured VRAM ratio `1440p / 720p` = `44.49 / 11.12` = **4.001×**.
- **Static Backdrop Cache (`BackdropCacheKey`)** eliminates **100% of the 9-pass pyramid GPU work** (`52%–59%` total frame reduction in software Vulkan; ~0.00 ms blur cost on real GPUs when typing, hovering, or clicking on a static desktop background).

### 4.2 Simultaneous Surfaces Scaling at `1280×720` (`1` .. `64` Cards)

| Surfaces | GPU Liquid Glass (Animated) | GPU Liquid Glass (Cached) | CPU Fallback (Pure `egui`) | Neumorphic Minimal |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 71.82 ms | 25.95 ms | 78.42 ms | 34.43 ms |
| 4 | 80.92 ms | 33.11 ms | 88.78 ms | 38.29 ms |
| 16 | 113.53 ms | 65.21 ms | 137.70 ms | 63.61 ms |
| 32 | 143.38 ms | 94.51 ms | 187.75 ms | 89.87 ms |
| 64 | 183.63 ms | 133.44 ms | 238.94 ms | 124.24 ms |
| **Marginal ms / surface** (`64 vs 1`) | **1.775 ms/surf** | **1.706 ms/surf** | **2.548 ms/surf** | **1.426 ms/surf** |

**Key insight**: Because the Dual-Kawase pyramid runs **once per frame (`O(1)` with respect to widget count)** and each `GlassCard` renders a **single 2-triangle GPU quad** via dynamic uniform buffer offsets, **GPU Liquid Glass scales better than pure-`egui` Fallback** at high card density (16–64 cards), where Fallback must tessellate multi-stop gradient meshes, frosted noise textures, and layered vector shadows per card.

### 4.3 Intermediate Texture Reuse & Graphics Resource Caching Audit (`1280×720`)

| Metric | Observed | Expected |
| :--- | ---: | ---: |
| Stationary frames (`24` cards) | 15 frames | — |
| Pyramid texture allocations | **1** | `1` (zero per-frame reallocation) |
| Pyramid GPU passes executed | **1** | `1` (first frame only) |
| Static pyramid cache hits | **14** | `14` (`0.00 ms` blur cost on frames 2..15) |
| Card uniform buffer allocations (`24` -> `64` cards) | **1 -> 2** | `1 -> 2` (power-of-two capacity reuse) |
| Dynamic resize allocations (`4` transitions) | **4** (avg 275.16 ms/resize on `llvmpipe`) | `4` (strictly 1 per resolution change) |

---

## 5. Memory Architecture & Integrated GPU (iGPU) Engineering Analysis

### 5.1 In-Place 6-Texture `Rgba16Float` Pyramid

Rather than allocating separate textures for both the downsample and upsample passes (9 textures = `1.906 × W × H × 8` bytes), `WgpuBlurEngine` executes an **in-place hazard-free 6-texture schedule**:

1. **Pass 0**: Render procedural studio backdrop or copy external RGBA8 wallpaper -> `source` (`W × H`)
2. **Passes 1..4 (Downsample)**: `source -> down1_or_deep (1/2) -> down2 (1/4) -> down3 (1/8) -> down4 (1/16)`
3. **Pass 5 (Medium blur)**: `down2 (1/4) -> blur_med (1/2)` *(sampled before `down2` is reused!)*
4. **Passes 6..8 (In-place Upsample)**: `down4 -> down3 (1/8) -> down2 (1/4) -> down1_or_deep (1/2)`

This reduces pyramid VRAM by **~17%** while producing mathematically identical Dual-Kawase blur output:

$$\text{VRAM}_{\text{pyramid}} = 8\text{ B/px} \times \left(1 + \frac{1}{4} + \frac{1}{16} + \frac{1}{64} + \frac{1}{256} + \frac{1}{4}\right) \cdot W \cdot H \approx 1.582 \times 8 \cdot W \cdot H$$

| Physical Resolution | Pyramid VRAM (6 × `Rgba16Float`) | Animated Blur Bandwidth / Frame (Read + Write) | 60 Hz Animated Bandwidth | Static Cached Bandwidth |
| :--- | ---: | ---: | ---: | ---: |
| `1280×720` (720p) | **11.12 MiB** | ~27.8 MiB/frame | ~1.67 GB/s | **0.00 GB/s** (blur) |
| `1920×1080` (1080p) | **25.03 MiB** | ~62.6 MiB/frame | ~3.75 GB/s | **0.00 GB/s** (blur) |
| `2560×1440` (1440p) | **44.49 MiB** | ~111.2 MiB/frame | ~6.67 GB/s | **0.00 GB/s** (blur) |
| `3840×2160` (4K HiDPI) | **100.10 MiB** | ~250.3 MiB/frame | ~15.02 GB/s | **0.00 GB/s** (blur) |

*(Bandwidth assumes 5-tap downsample + 9-tap tent upsample with L2 texture cache hit rates typical of tiled/desktop GPUs, counting ~1× read + 1× write per pass pixel at 8 B/px.)*

### 5.2 Behavior on Integrated GPUs (iGPUs & Unified Memory)

Integrated GPUs share system DRAM bandwidth with the CPU and operate under tight thermal/battery envelopes. Comparing our exact memory/bandwidth profile against common iGPU classes:

- **Intel UHD / Iris Xe (LPDDR4X / DDR4, ~50–68 GB/s shared)**:
  - At **1080p @ 60 Hz**, animated blur consumes ~3.75 GB/s (**~6% of peak DRAM bandwidth**) and 25 MiB of shared memory — comfortably leaving >90% headroom for 60 FPS desktop apps.
  - At **4K HiDPI (2.0x DPI on 1920×1080 logical)** with an *animated* backdrop, bandwidth reaches ~15 GB/s (~25% of LPDDR4X). For battery-powered 4K laptops, keep the backdrop static (`time_secs = 0.0`) so **`BackdropCacheKey` skips all 9 blur passes (`0.00 GB/s` blur bandwidth)** on stationary frames!
- **AMD Radeon 780M / 680M (LPDDR5X, ~96–120 GB/s shared)**:
  - Even 1440p @ 120 Hz animated blur (~13.3 GB/s) stays around ~12% of available memory bandwidth, with sub-millisecond per-frame blur execution.
- **Apple Silicon (M1 / M2 / M3 / M4 UMA, 100–400+ GB/s TBDR)**:
  - Tile-Based Deferred Rendering (TBDR) keeps `Rgba16Float` intermediate reads/writes hot in on-chip tile memory/SLC; static desktop frames hit `BackdropCacheKey` for near-zero GPU power draw.
- **Low-Power / Headless / Software-Only Environments**:
  - Toggle `theme.real_blur_enabled = false` (or switch to `PrismTheme::minimal_light()` / `PrismTheme::minimal_dark()`) at runtime to bypass GPU pyramid allocations completely (`0 MiB` blur VRAM).

### 5.3 Graphics Resource Caching Summary

| Resource | Allocation Strategy | Lifetime & Reuse |
| :--- | :--- | :--- |
| WGSL Pipelines (`6` pipelines) & Bind Group Layouts (`4`) | Created once in `WgpuBlurEngine::new` inside `egui-wgpu` `CallbackResources` | Reused for entire `wgpu::Device` lifetime (`0` per-frame allocations) |
| Dual-Kawase Pyramid (`6` `Rgba16Float` textures + `10` bind groups + `8` Kawase uniform buffers) | Created on first frame; rebuilt **only** when physical `[width, height]` changes | `0` per-frame allocations; skipped entirely on `BackdropCacheKey` hit |
| Dynamic Glass Card Uniform Buffer | Initial capacity `32` cards (256-byte aligned stride); grows by power-of-two (`64`, `128`, ...) only if exceeded | `0` reallocations once peak card count is reached; single `queue.write_buffer` upload per frame |
| External Wallpaper Texture (`Rgba8UnormSrgb`) | Uploaded **only** when `set_external_backdrop_rgba8` bumps `generation` | Cached indefinitely across frames; automatically invalidates `BackdropCacheKey` on update |
| Live Telemetry (`WgpuBlurProvider::metrics()`) | Lock-protected snapshot updated per frame | Exposes `pyramid_size`, `pyramid_texture_count`, `estimated_vram_bytes`, `pyramid_allocations`, `pyramid_passes_executed`, `pyramid_cache_hits`, `card_buffer_allocations`, and `last_frame_card_count` |
