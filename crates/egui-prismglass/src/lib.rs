//! # egui-prismglass
//!
//! A fallback-first 3D glassmorphic & minimal design system and widget kit for [`egui`] with
//! opt-in **real GPU backdrop blur** and **Apple Liquid Glass optics**.
//!
//! ## Rendering Modes
//!
//! - **Zero-dependency Fallback (`default = []`)**: Works identically on every `egui` backend
//!   with translucent multi-stop gradients, cached procedural frost micro-grain, dual-layer 3D
//!   elevation shadows, and directional machined bevels.
//! - **`feature = "blur-wgpu"`**: Pure-WGSL 4-level linear Dual-Kawase blur pyramid + Apple
//!   Liquid Glass composite shader (analytic rounded-rect SDF, curved-bevel lens refraction,
//!   subtle RGB chromatic dispersion, macOS vibrancy boost, Jimenez IGN micro-grain, and 3D
//!   studio bevel speculars). Works on `eframe` (`wgpu`), custom `egui-wgpu` loops, and
//!   headless `egui_kittest` — all with `#![forbid(unsafe_code)]`.
//! - **`feature = "blur-glow"`**: Live-framebuffer OpenGL 3.3 / GLES 3.0 / WebGL 2.0 grab-pass
//!   backdrop blur for `eframe` (`glow`) and custom `egui_glow` hosts, preserving
//!   `#![forbid(unsafe_code)]`.
//!
//! ## Quick Start
//!
//! ```no_run
//! use egui_prismglass::{
//!     GlassButton, GlassCard, GlassElevation, PrismContextExt, PrismTheme,
//!     paint_prism_backdrop_ui,
//! };
//!
//! fn draw_ui(ui: &mut egui::Ui) {
//!     // Apply once during setup (or whenever switching themes):
//!     let theme = PrismTheme::onyx();
//!     ui.ctx().set_prism_theme(theme.clone());
//!
//!     // Paint the studio backdrop (automatically GPU-accelerated when `blur-wgpu` is installed):
//!     let rect = ui.max_rect();
//!     paint_prism_backdrop_ui(ui, rect, &theme, 0.0, 1.0);
//!
//!     GlassCard::new()
//!         .elevation(GlassElevation::Elevated)
//!         .show(ui, |ui| {
//!             ui.heading("Hello PrismGlass");
//!             if ui.add(GlassButton::primary("Launch")).clicked() {
//!                 // Handle click
//!             }
//!         });
//! }
//! ```

use std::sync::Arc;

pub mod anim;
pub mod blur;
#[cfg(feature = "blur-glow")]
pub mod blur_glow;
#[cfg(feature = "blur-wgpu")]
pub mod blur_wgpu;
pub mod painter;
pub mod theme;
pub mod tokens;
pub mod widgets;

pub use anim::{
    Easing, PrismAnimExt, SpringConfig, SpringState, animate_hover, animate_pulse, animate_shimmer,
    animate_spring, animate_tween,
};
pub use blur::{
    BackdropBlurProvider, BackdropRequest, BlurCapability, FallbackBlurProvider, blur_provider,
    clear_blur_provider, effective_blur_capability, set_blur_provider,
};
pub use painter::{
    glass_bevel_and_shadow_shapes, glass_surface_shapes, glass_surface_shapes_with_ctx,
    paint_glass_surface, paint_prism_backdrop, paint_prism_backdrop_ui, paint_radial_glow_orb,
};
pub use theme::{PrismPreset, PrismTheme, SurfaceStyle, ThemeMode};
pub use tokens::{
    AccentPalette, BackdropScene, GlassElevation, GlassSurfaceTokens, PrismPalette, RadiusTokens,
    SpacingTokens,
};
pub use widgets::{
    BadgeVariant, GlassBadge, GlassButton, GlassButtonVariant, GlassCard, GlassPanel, GlassSidebar,
    GlassSlider, GlassToggle, SidebarIcon, SidebarItem,
};

/// Ergonomic extension methods on [`egui::Context`] for managing [`PrismTheme`] and GPU blur backends.
pub trait PrismContextExt {
    /// Applies `theme` to this `egui` context and stores it for all `PrismGlass` widgets.
    fn set_prism_theme(&self, theme: PrismTheme);

    /// Returns the currently active [`PrismTheme`] (or [`PrismTheme::onyx()`] if unset).
    fn prism_theme(&self) -> PrismTheme;

    /// Installs a custom or built-in [`BackdropBlurProvider`] into this `egui` context.
    fn set_prism_blur_provider(&self, provider: Arc<dyn BackdropBlurProvider>);

    /// Removes any installed [`BackdropBlurProvider`], reverting to pure-`egui` fallback glass.
    fn clear_prism_blur_provider(&self);

    /// Reports the effective [`BlurCapability`] for the current theme and context.
    fn prism_blur_capability(&self) -> BlurCapability;
}

impl PrismContextExt for egui::Context {
    fn set_prism_theme(&self, theme: PrismTheme) {
        theme.apply(self);
    }

    fn prism_theme(&self) -> PrismTheme {
        PrismTheme::from_ctx(self)
    }

    fn set_prism_blur_provider(&self, provider: Arc<dyn BackdropBlurProvider>) {
        set_blur_provider(self, provider);
    }

    fn clear_prism_blur_provider(&self) {
        clear_blur_provider(self);
    }

    fn prism_blur_capability(&self) -> BlurCapability {
        let theme = self.prism_theme();
        effective_blur_capability(self, &theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_produce_distinct_themes() {
        let onyx = PrismTheme::onyx();
        let frost = PrismTheme::frost();
        let glacial = PrismTheme::glacial();
        let minimal_light = PrismTheme::minimal_light();
        let minimal_dark = PrismTheme::minimal_dark();

        assert_eq!(onyx.mode, ThemeMode::Dark);
        assert_eq!(frost.mode, ThemeMode::Light);
        assert_eq!(glacial.mode, ThemeMode::Dark);
        assert_eq!(onyx.surface_style, SurfaceStyle::Glass);
        assert_eq!(minimal_light.surface_style, SurfaceStyle::Minimal);
        assert_eq!(minimal_dark.surface_style, SurfaceStyle::Minimal);
        assert_eq!(
            minimal_light.surface(GlassElevation::Elevated).blur_radius,
            0.0
        );
        assert_eq!(minimal_dark.surface(GlassElevation::Elevated).tint.a(), 255);
        assert_ne!(onyx.palette.canvas, frost.palette.canvas);
        assert_ne!(onyx.palette.accent, glacial.palette.accent);
    }

    #[test]
    fn surface_token_builders_clamp_values() {
        let base = PrismTheme::onyx().surface(GlassElevation::Elevated);
        let modified = base
            .with_corner_radius(-10.0)
            .with_blur_radius(-4.0)
            .with_prism_intensity(5.0)
            .with_vibrancy(4.5)
            .with_refraction(3.0)
            .with_dispersion(-1.0)
            .with_grain(1.0)
            .with_opacity_scale(0.5);

        assert_eq!(modified.corner_radius, 0.0);
        assert_eq!(modified.blur_radius, 0.0);
        assert_eq!(modified.prism_intensity, 1.0);
        assert_eq!(modified.vibrancy, 2.0);
        assert_eq!(modified.refraction_strength, 1.0);
        assert_eq!(modified.chromatic_dispersion, 0.0);
        assert_eq!(modified.grain_intensity, 0.10);
        assert!(modified.tint.a() < base.tint.a());
    }

    #[test]
    fn resolved_for_real_blur_reduces_film_opacity() {
        let onyx_card = PrismTheme::onyx().surface(GlassElevation::Elevated);
        let real_blur = onyx_card.resolved_for_real_blur();
        assert!(real_blur.tint.a() < onyx_card.tint.a());
        assert!(real_blur.gradient_top.a() < onyx_card.gradient_top.a());
    }

    #[test]
    fn backdrop_scenes_have_distinct_shader_ids() {
        assert_eq!(BackdropScene::SculptedWaves.shader_id(), 0);
        assert_eq!(BackdropScene::LiquidPrism.shader_id(), 1);
        assert_eq!(BackdropScene::ArchitecturalGrid.shader_id(), 2);
        assert_eq!(BackdropScene::StudioNeutral.shader_id(), 3);
        assert_eq!(BackdropScene::None.shader_id(), 4);
    }

    #[test]
    fn glass_surface_shapes_handles_empty_and_valid_rects() {
        let tokens = PrismTheme::onyx().surface(GlassElevation::Floating);
        assert_eq!(glass_surface_shapes(egui::Rect::NOTHING, &tokens).len(), 0);

        let valid = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(240.0, 140.0));
        let shapes = glass_surface_shapes(valid, &tokens);
        assert!(shapes.len() >= 4);
    }

    #[test]
    fn context_stores_theme_and_blur_provider() {
        struct DummyRealBlur;
        impl BackdropBlurProvider for DummyRealBlur {
            fn capability(&self) -> BlurCapability {
                BlurCapability::RealBackdrop { backend: "test" }
            }
        }

        let ctx = egui::Context::default();
        ctx.set_prism_theme(PrismTheme::frost());
        let active = ctx.prism_theme();
        assert_eq!(active.mode, ThemeMode::Light);
        assert_eq!(active.name, "Frost Light");
        assert_eq!(ctx.prism_blur_capability(), BlurCapability::FallbackOnly);

        ctx.set_prism_blur_provider(Arc::new(DummyRealBlur));
        assert_eq!(
            ctx.prism_blur_capability(),
            BlurCapability::RealBackdrop { backend: "test" }
        );

        let mut disabled = ctx.prism_theme();
        disabled.real_blur_enabled = false;
        ctx.set_prism_theme(disabled);
        assert_eq!(ctx.prism_blur_capability(), BlurCapability::FallbackOnly);

        ctx.clear_prism_blur_provider();
        ctx.set_prism_theme(PrismTheme::onyx());
        assert_eq!(ctx.prism_blur_capability(), BlurCapability::FallbackOnly);
    }

    #[test]
    fn fallback_blur_provider_reports_fallback_capability() {
        let provider = FallbackBlurProvider;
        assert_eq!(provider.capability(), BlurCapability::FallbackOnly);
    }

    #[test]
    fn accent_palettes_update_glass_and_minimal_themes() {
        for palette in AccentPalette::ALL {
            let light = PrismTheme::minimal_light().with_accent_palette(palette);
            let dark = PrismTheme::minimal_dark().with_accent_palette(palette);
            let glass = PrismTheme::glacial().with_accent_palette(palette);
            assert_eq!(light.accent_palette, palette);
            assert_eq!(dark.accent_palette, palette);
            assert_eq!(glass.accent_palette, palette);
            assert_eq!(light.palette.accent, palette.primary(false));
            assert_eq!(dark.palette.accent, palette.primary(true));
        }
    }
}
