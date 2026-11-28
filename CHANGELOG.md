# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-11-28

### Added
- **`GlassTextField`**: single-line and multiline (`GlassTextField::multiline`) input on `egui::TextEdit` with placeholder, leading `Icon`, label, help text, focus ring, error state (`.error(Some("…"))` with a message), disabled state, password masking, `width`, `rows`, `char_limit` and `id_salt`. Returns the inner `TextEdit` response.
- **`GlassModal`** on `egui::Modal`: dimming scrim, focus kept inside the dialog (widgets behind it are not interactive), `Esc` and scrim click close it (both configurable), optional × button, title / description / icon header, `GlassModal::footer` for right-aligned actions, closing from inside with `ui.close()`. Returns `GlassModalResponse { inner, response, closed }`.
- **`GlassToasts` / `Toast`**: a queue of notifications with `Info` / `Success` / `Warning` / `Danger` variants (`ToastVariant`), auto-dismiss with a progress bar (`duration`, `sticky`), pause on hover, close button, enter/exit animation, stacking in any corner (`ToastAnchor`), `max_visible` (the rest wait in the queue with their timers stopped), `dismiss(ToastId)`, and `reduce_motion`.
- **`GlassMenu`, `MenuItem`, `GlassPopover`** on `egui::Popup`: dropdown (`GlassMenu::new`) and context (`GlassMenu::context`) menus with icons, shortcuts, check marks, danger items, disabled items, `GlassMenu::separator`, `GlassMenu::section` titles and `GlassMenu::submenu` (on `egui::containers::menu::SubMenu`); popovers with free-form content (`GlassPopover::new` / `open_bool` / `from_popup`). Both paint the theme's floating glass and keep egui's keyboard navigation and close behavior.
- **`Icon`** (module `icons`): 65 vector icons painted with `egui::Painter` on a 16 pt grid with a 1.5 pt stroke — the 46 icons of PrismCast Studio plus `Search`, `Info`, `Warning`, `AlertCircle`, `CheckCircle`, `List`, `Clock`, `Chart`, `More`, `Copy`, `Edit`, `Filter`, `Play`, `User`, `Bell`, `Home`, `Upload`, `Menu` and `Prism`. `Icon::paint`, `paint_icon`, `Icon::ALL`, `Icon::name`, the `GlassIcon` widget and `From<SidebarIcon> for Icon`.
- **Kit components** (module `widgets::kit`): `GlassListRow` (clickable row with hover/selection, leading selection marker, icon / title / subtitle / trailing text or custom contents), `GlassChip` (tinted uppercase chip, optional icon, filter-toggle mode), `GlassKbd` (key caps, `GlassKbd::combo`), `GlassEmptyState` (icon, title, description, actions) and `SectionHeader` (uppercase tracked title, count, actions), plus `eyebrow_text`.
- `GlassButton::icon` accepts an `Icon` (via the new `ButtonIcon`); `GlassButton::icon_only(Icon)` for square ghost icon buttons.
- `SidebarItem::icon(Icon)` and `SidebarItem::with_icon(Icon, label)`.
- `GlassSegmented::optional_counts(&[Option<u32>])`: per-segment counts where `None` shows nothing instead of `0`.
- **Live web demo**: the demo builds for `wasm32-unknown-unknown` with trunk (`crates/demo/index.html`) and is deployed to GitHub Pages by `.github/workflows/pages.yml`; CI checks the wasm32 build of the library and the demo.
- **Prism Studio**: a new **Components** page showing every v0.4 widget; the render queue now uses `GlassTextField` for search, menus for Filter and Sort, a confirmation `GlassModal` for Cancel, toasts, vector icons in the sidebar, and hides zero filter counts.
- Tests: unit tests for every new widget (typing into a text field, modal Esc / blocked background, menu open / Esc close, popover click-outside, toast queue timing and dismissal, list row sizes) and three new golden snapshots in `visual_regression` (`v04_controls_prism_dark`, `v04_controls_prism_light`, `v04_overlays_prism_glass_dark`) plus a scrim invariant.

### Changed
- **API**: `GlassButton::icon` now takes `impl Into<ButtonIcon>` instead of `impl Into<String>`. `&str` and `String` arguments compile unchanged; other `Into<String>` types (`Cow<str>`, `Box<str>`, …) need `.to_string()`.
- The demo's native entry point and blur-provider setup moved into a shared `create_app` used by both the native and web launchers.

### Unchanged
- All existing golden snapshots (classic presets and v0.3 Prism materials) render identically.

## [0.3.0] - 2026-11-28

### Added
- **Prism design language**: four new presets in two materials:
  - `SurfaceStyle::Layered` (`PrismTheme::prism_dark` / `prism_light`): solid, flat content with frosted glass reserved for floating layers (sidebars, popovers, menus, floating toolbars).
  - `SurfaceStyle::Frosted` (`PrismTheme::prism_glass_dark` / `prism_glass_light`): the same language with every container made of frosted glass.
  - `PrismTheme::prism(mode, style)`, `PrismTheme::with_material(style)`, `PrismTheme::on_accent()`, `PrismPreset::PRISM` / `PrismPreset::CLASSIC`, `SurfaceStyle::{is_prism, uses_backdrop, label}`.
- **Typography (`typography`, feature `inter`, on by default)**: bundled Inter variable font (SIL OFL 1.1) registered in Regular / Medium / Semibold plus tabular figures; `install_fonts`, `prism_font`, `prism_numeric_font`, `FontWeight`, `PrismContextExt::install_prism_fonts`. Widgets fall back to egui's fonts when Inter is not installed.
- **Neutral tokens (`PrismNeutrals`)** on every theme (`PrismTheme::neutrals`): surface, raised, sunken, control-on, hairlines, hover/selected overlays and a muted status color.
- **Prism accents (`AccentPalette::prism`)**: each palette calibrated to keep ≥ 4.5:1 contrast with its label color.
- **Aurora backdrop (`BackdropScene::Aurora`)**: soft multi-color light pools, in WGSL and in the CPU fallback; the default scene of the Prism presets.
- **New widgets**: `GlassSegmented` (with counts), `GlassProgress`, `GlassCheckbox`, `GlassButton::danger` / `GlassButtonVariant::Danger`, `BadgeVariant::Danger`, and `SidebarIcon::{List, Layers, Clock, Chart, Folder}`.
- **`paint_glass_surface_ui`**: paint a themed surface (solid or GPU glass) into any rect for custom layouts.
- **Prism Studio demo**: `cargo run -p demo` now opens a render-queue app with Dark/Light, Layered/All glass and accent switching; the v0.2 showcase stays available as "Classic showcase".
- Visual regression: Prism goldens for both materials plus a material invariant (Layered cards opaque, All-glass cards transmit the backdrop).

### Changed
- `PrismTheme::default()` (and the fallback of `PrismTheme::from_ctx`) is now **Prism Dark**; `PrismPreset::default()` is `PrismDark`.
- `GlassButton::min_size` is now optional internally; the classic default height (34 pt) is unchanged.
- Prism-style widgets expose accessibility roles (switch, checkbox, slider, progress indicator).

### Unchanged
- The classic presets (`Frost`, `Onyx`, `Glacial`, `MinimalLight`, `MinimalDark`) render exactly as in v0.2 (golden snapshots enforced).

## [0.2.0] - 2026-11-27

### Added
- **Real GPU Backdrop Blur (`feature = "blur-wgpu"`)**: Pure-WGSL 4-level linear `Rgba16Float` Dual-Kawase blur pyramid + Apple Liquid Glass composite shader (`blur_wgpu::WgpuBlurProvider`, `blur_wgpu::install`, `blur_wgpu::install_with_format`), working on `eframe` (`wgpu`), own-loop `egui-wgpu`, and headless `egui_kittest` while preserving `#![forbid(unsafe_code)]`.
  - Analytic rounded-rect SDF with sub-pixel anti-aliasing (`fwidth`).
  - Curved-bevel optical lens refraction and subtle RGB chromatic dispersion along glass lips.
  - Linear-space macOS vibrancy saturation boost and Jimenez Interleaved Gradient Noise (IGN) micro-grain.
  - External backdrop texture support (`WgpuBlurProvider::set_external_backdrop_rgba8` / `clear_external_backdrop`).
- **Live-Framebuffer OpenGL Grab-Pass Blur (`feature = "blur-glow"`)**: Safe `GlowBlurProvider` wrapping `backdrop-blur-glow` (`glow 0.17` / `egui_glow 0.37.0`) for `eframe+glow`, preserving `#![forbid(unsafe_code)]`, plus `crates/demo/examples/glow_blur.rs`.
- **Optical Material Tokens & Studio Scenes**:
  - Extended `GlassSurfaceTokens` with `contact_shadow`, `vibrancy`, `refraction_strength`, `chromatic_dispersion`, `grain_intensity`, `blur_tint_alpha`, and `resolved_for_real_blur()`.
  - Added `BackdropScene` (`SculptedWaves`, `LiquidPrism`, `ArchitecturalGrid`) and `paint_prism_backdrop_ui`.
  - Added live optical dials on `PrismTheme` (`real_blur_enabled`, `backdrop_scene`, `blur_radius_scale`, `refraction_scale`, `dispersion_scale`, `vibrancy_scale`, `grain_scale`).
- **Upgraded Fallback Renderer (`default = []`)**: Dual-layer 3D shadows (`shadow` + `contact_shadow`), cached `128×128` procedural frosted micro-grain texture (`glass_surface_shapes_with_ctx`), and `glass_bevel_and_shadow_shapes`.
- **Context Blur Registry (`PrismContextExt`)**: `set_prism_blur_provider`, `clear_prism_blur_provider`, and `prism_blur_capability`.
- **Modern Tactile Neumorphism (`MinimalLight` & `MinimalDark`)**: Sculpted solid porcelain-light and carbon-slate-dark themes with dual directional soft shadows (upper-left light + lower-right depth), recessed nested wells, and extruded tactile buttons.
- **8 Curated Vibrant Color Palettes (`AccentPalette`)**: `Cobalt`, `Violet`, `Emerald`, `Ruby`, `Amber`, `Cyan`, `Magenta`, and `Graphite` via `PrismTheme::with_accent_palette` / `set_accent_palette`, harmonizing every Glass and Neumorphic theme — plus `GlassButton::tint(...)` for colored Liquid Glass pills and `GlassCard::tokens(...)`.
- **Upgraded Showcase Demo (`crates/demo`)**: Enabled `blur-wgpu` by default, added live Optical Inspector sliders, Studio Backdrop Scene switcher, Real GPU Blur vs Fallback toggle, 8-color Palette Picker across all themes, and real GPU `wgpu_blur` integration tests.

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
