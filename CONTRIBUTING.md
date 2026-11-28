# Contributing to `egui-prismglass`

Thank you for your interest in contributing to **egui-prismglass**! We welcome bug reports, widget proposals, theme presets, documentation improvements, and pull requests of all sizes.

## Design Principles

1. **Fallback-First Glass**: Every widget and token must look great using pure `egui` primitives (`Painter` / `Mesh`) with zero required GPU shaders so users on `eframe+wgpu`, `eframe+glow`, own-loop `egui-wgpu`, and WASM work on day one.
2. **Token-Driven Styling**: Avoid hardcoded colors or radii inside widgets. Read from `PrismTheme::from_ctx(ui.ctx())` and allow local builder overrides.
3. **Zero Frame-1 Flicker**: Dynamic glass containers reserve a `ShapeIdx` placeholder before laying out children and fill it once the final `Rect` is known.
4. **Strict `egui` Version Clarity**: Whenever upgrading `egui`, update the compatibility matrix in `README.md` and `CHANGELOG.md`.

## Local Development

```bash
# Run the interactive showcase app
cargo run -p demo

# Run the web demo (needs `rustup target add wasm32-unknown-unknown` and `cargo install trunk`)
cd crates/demo && trunk serve

# Run all unit & doc tests
cargo test -p egui-prismglass --all-features

# Check formatting & lints (same checks as CI)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Good First Issues

Looking for an entry point? Check issues labeled `good first issue`, such as:
- Adding new glass widgets (`GlassTabs`, `GlassSelect`, tooltips, a command palette)
- Creating new `PrismPreset` themes (e.g., Catppuccin-inspired or High-Contrast Glass)
- Adding snapshot visual regression tests with `egui_kittest`
- Improving the WASM web demo (e.g. GPU backdrop blur on WebGPU)
- Adding icons to the shared `Icon` set (16 pt grid, 1.5 pt stroke)

## Pull Request Checklist

- [ ] Code is formatted with `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes cleanly
- [ ] New public items include rustdoc comments (`#![warn(missing_docs)]`)
- [ ] Unit tests or visual examples cover new functionality
- [ ] `CHANGELOG.md` updated under `[Unreleased]` for user-visible changes
