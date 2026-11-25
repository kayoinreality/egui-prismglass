//! # egui-prismglass
//!
//! A fallback-first glassmorphic design system and widget kit for [`egui`].
//!
//! ## Why Fallback-First?
//!
//! Real GPU backdrop blur in the `egui` ecosystem currently depends on specific host
//! rendering loops (`own-loop` `egui-wgpu` or `eframe+glow`), leaving mainstream
//! `eframe+wgpu` and WebAssembly setups out. `egui-prismglass` renders crisp frosted glass
//! using **pure `egui` meshes and painters**—translucent multi-stop gradients, soft inner
//! reflections, luminous overhead specular rims, prismatic refraction highlights, and
//! elevation shadows—so it works identically on **every** backend from day one.
//!
//! Real GPU backdrop blur (`blur-wgpu` / `blur-glow`) is designed as an opt-in upgrade
//! through the [`blur::BackdropBlurProvider`] seam without breaking widget APIs.
//!
//! ## Quick Start
//!
//! ```no_run
//! use egui_prismglass::{
//!     GlassButton, GlassCard, GlassElevation, PrismContextExt, PrismTheme, paint_prism_backdrop,
//! };
//!
//! fn draw_ui(ui: &mut egui::Ui) {
//!     // Apply once during setup (or whenever switching themes):
//!     let theme = PrismTheme::onyx();
//!     ui.ctx().set_prism_theme(theme.clone());
//!
//!     // Paint an atmospheric backdrop so translucent glass pops:
//!     let rect = ui.max_rect();
//!     paint_prism_backdrop(ui.painter(), rect, &theme, 0.0, 1.0);
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

pub mod anim;
pub mod blur;
pub mod painter;
pub mod theme;
pub mod tokens;
pub mod widgets;

pub use anim::{
    Easing, PrismAnimExt, SpringConfig, SpringState, animate_hover, animate_pulse, animate_shimmer,
    animate_spring, animate_tween,
};
pub use blur::{BackdropBlurProvider, BackdropRequest, BlurCapability, FallbackBlurProvider};
pub use painter::{
    glass_surface_shapes, paint_glass_surface, paint_prism_backdrop, paint_radial_glow_orb,
};
pub use theme::{PrismPreset, PrismTheme, SurfaceStyle, ThemeMode};
pub use tokens::{GlassElevation, GlassSurfaceTokens, PrismPalette, RadiusTokens, SpacingTokens};
pub use widgets::{
    BadgeVariant, GlassBadge, GlassButton, GlassButtonVariant, GlassCard, GlassPanel, GlassSidebar,
    GlassSlider, GlassToggle, SidebarIcon, SidebarItem,
};

/// Ergonomic extension methods on [`egui::Context`] for installing and reading [`PrismTheme`].
pub trait PrismContextExt {
    /// Applies `theme` to this `egui` context and stores it for all `PrismGlass` widgets.
    fn set_prism_theme(&self, theme: PrismTheme);

    /// Returns the currently active [`PrismTheme`] (or [`PrismTheme::onyx()`] if unset).
    fn prism_theme(&self) -> PrismTheme;
}

impl PrismContextExt for egui::Context {
    fn set_prism_theme(&self, theme: PrismTheme) {
        theme.apply(self);
    }

    fn prism_theme(&self) -> PrismTheme {
        PrismTheme::from_ctx(self)
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
            .with_opacity_scale(0.5);

        assert_eq!(modified.corner_radius, 0.0);
        assert_eq!(modified.blur_radius, 0.0);
        assert_eq!(modified.prism_intensity, 1.0);
        assert!(modified.tint.a() < base.tint.a());
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
    fn context_stores_and_retrieves_active_theme() {
        let ctx = egui::Context::default();
        ctx.set_prism_theme(PrismTheme::frost());
        let active = ctx.prism_theme();
        assert_eq!(active.mode, ThemeMode::Light);
        assert_eq!(active.name, "Frost Light");
    }

    #[test]
    fn fallback_blur_provider_reports_fallback_capability() {
        let provider = FallbackBlurProvider;
        assert_eq!(provider.capability(), BlurCapability::FallbackOnly);
    }
}
