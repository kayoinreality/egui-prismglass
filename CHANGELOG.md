# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- `blur-glow` and `blur-wgpu` optional feature integration with `backdrop-blur-egui` once aligned with `egui 0.37`
- Additional glass widgets: `GlassTextField`, `GlassTabs`, `GlassModal`, `GlassProgressBar`

## [0.1.0] - 2026-11-25

### Added
- **Design Tokens (`tokens`)**: `GlassSurfaceTokens`, `GlassElevation` (`Flat`, `Subtle`, `Elevated`, `Floating`), `PrismPalette`, `SpacingTokens`, and `RadiusTokens`.
- **Theme System (`theme`)**: `PrismTheme` with `SurfaceStyle::Glass` and `SurfaceStyle::Minimal` across 5 built-in presets (`Frost Light`, `Onyx Dark`, `Glacial Glass`, `Minimal Light`, `Minimal Dark`), global `egui::Visuals` harmonization, and `PrismContextExt`.
- **Fallback-First 3D Glass & Minimal Painter (`painter`)**: Pure-`egui` 3D directional rim-lit glass bevels over sculpted volumetric studio light ribbons, plus opaque minimal solid surfaces (`paint_prism_backdrop`).
- **Animation Kit (`anim`)**: Frame-rate-independent damped spring physics (`SpringConfig`, `SpringState`), built-in `Easing` curves, `PrismAnimExt`, 3D hover micro-lift on interactive cards, pulse, and shimmer helpers.
- **Backdrop Blur Seam (`blur`)**: Backend-agnostic `BackdropRequest`, `BlurCapability`, and `BackdropBlurProvider` trait prepared for real GPU blur upgrades without API breakage.
- **Glass Widgets (`widgets`)**: `GlassCard`, `GlassPanel`, `GlassButton` (`Primary`, `Secondary`, `Accent`, `Ghost`), `GlassSidebar` + `SidebarItem`, `GlassSlider`, `GlassBadge`, and `GlassToggle`.
- **Interactive Showcase (`crates/demo`)**: Launchable in one command via `cargo run -p demo`, featuring a live Material Inspector, Component Gallery, and Design Token Explorer.
- **Optional `serde` feature**: Serialization/deserialization support for tokens and themes.
- **Pinned `egui 0.37.0` support** with compatibility matrix documentation.
