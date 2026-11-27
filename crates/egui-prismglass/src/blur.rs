//! Backend-agnostic GPU backdrop-blur seam and context registry.
//!
//! `egui-prismglass` is **fallback-first**: every surface renders cleanly with zero GPU
//! dependencies on any `egui` backend. Installing a GPU provider via
//! [`crate::PrismContextExt::set_prism_blur_provider`] (backed by `feature = "blur-wgpu"`
//! or `feature = "blur-glow"`) upgrades every [`crate::GlassCard`], [`crate::GlassPanel`],
//! and [`crate::GlassSidebar`] to real GPU backdrop blur automatically without changing
//! widget call sites.

use std::sync::Arc;

use egui::{Color32, Context, Id, Rect, Shadow, Shape, Ui};

use crate::theme::{PrismTheme, SurfaceStyle, ThemeMode};
use crate::tokens::GlassSurfaceTokens;

const BLUR_PROVIDER_STORAGE_ID: &str = "egui_prismglass_blur_provider";

/// Complete optical & geometric descriptor for a frosted surface rendered this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropRequest {
    /// Target rectangle in logical `egui` points.
    pub rect: Rect,
    /// Uniform corner radius in logical points.
    pub corner_radius: f32,
    /// Target backdrop blur radius in logical points (`0.0` disables blur).
    pub blur_radius: f32,
    /// Core translucent glass film tint (already adjusted for real-blur opacity when applicable).
    pub tint: Color32,
    /// Top gradient stop of glass body tint.
    pub gradient_top: Color32,
    /// Bottom gradient stop of glass body tint.
    pub gradient_bottom: Color32,
    /// Perimeter hairline glass border color.
    pub border: Color32,
    /// Top-left directional key-light bevel highlight.
    pub specular_top: Color32,
    /// Prismatic accent bevel tint.
    pub specular_accent: Color32,
    /// Inner `1px` glass thickness catch.
    pub inner_glow: Color32,
    /// Border width in logical points.
    pub border_width: f32,
    /// Directional prism tint intensity (`0.0..=1.0`).
    pub prism_intensity: f32,
    /// macOS-style saturation vibrancy multiplier (`1.0` = neutral).
    pub vibrancy: f32,
    /// Apple Liquid Glass curved-bevel lens refraction intensity (`0.0..=1.0`).
    pub refraction_strength: f32,
    /// Subtle RGB chromatic dispersion along curved glass bevels (`0.0..=1.0`).
    pub chromatic_dispersion: f32,
    /// Frosted micro-grain amplitude (`0.0..=0.10`).
    pub grain_intensity: f32,
    /// Wide ambient drop shadow.
    pub shadow: Shadow,
    /// Tight grounded contact occlusion shadow.
    pub contact_shadow: Shadow,
    /// Visibility/presence dial (`0.0..=1.0`) for animated fade-in/out surfaces.
    pub presence: f32,
    /// Whether the active theme polarity is dark.
    pub is_dark: bool,
}

impl BackdropRequest {
    /// Builds a [`BackdropRequest`] from a target [`Rect`], [`GlassSurfaceTokens`], and polarity.
    pub fn from_tokens(rect: Rect, tokens: &GlassSurfaceTokens) -> Self {
        Self::from_tokens_with_mode(rect, tokens, ThemeMode::Dark)
    }

    /// Builds a [`BackdropRequest`] from a target [`Rect`], [`GlassSurfaceTokens`], and [`ThemeMode`].
    pub fn from_tokens_with_mode(rect: Rect, tokens: &GlassSurfaceTokens, mode: ThemeMode) -> Self {
        let resolved = tokens.resolved_for_real_blur();
        Self {
            rect,
            corner_radius: resolved.corner_radius,
            blur_radius: resolved.blur_radius,
            tint: resolved.tint,
            gradient_top: resolved.gradient_top,
            gradient_bottom: resolved.gradient_bottom,
            border: resolved.border,
            specular_top: resolved.specular_top,
            specular_accent: resolved.specular_accent,
            inner_glow: resolved.inner_glow,
            border_width: resolved.border_width,
            prism_intensity: resolved.prism_intensity,
            vibrancy: resolved.vibrancy,
            refraction_strength: resolved.refraction_strength,
            chromatic_dispersion: resolved.chromatic_dispersion,
            grain_intensity: resolved.grain_intensity,
            shadow: resolved.shadow,
            contact_shadow: resolved.contact_shadow,
            presence: 1.0,
            is_dark: mode == ThemeMode::Dark,
        }
    }
}

/// Reports whether real GPU backdrop blur is active or fallback glass rendering is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlurCapability {
    /// Pure `egui` translucent glass fallback (works on all backends).
    FallbackOnly,
    /// Real GPU backdrop blur backed by a named adapter (e.g. `"wgpu-dual-kawase"`, `"glow-grab-pass"`).
    RealBackdrop {
        /// Human-readable backend identifier.
        backend: &'static str,
    },
}

/// Extension seam for plugging a real GPU backdrop-blur backend into `egui-prismglass`.
pub trait BackdropBlurProvider: Send + Sync {
    /// Returns the active blur capability of this provider.
    fn capability(&self) -> BlurCapability;

    /// Legacy hook for enqueuing a pre-foreground backdrop pass (used by grab-pass hosts).
    fn enqueue_backdrop(&self, _ui: &Ui, _request: BackdropRequest) {}

    /// Builds the background [`egui::Shape`]s for a glass surface (placed directly into a
    /// container's reserved `ShapeIdx` slot behind foreground children).
    ///
    /// Returning `None` instructs the widget to use the pure-`egui` fallback renderer.
    fn render_glass_surface(
        &self,
        _ui: &Ui,
        _id: Id,
        _request: &BackdropRequest,
    ) -> Option<Vec<Shape>> {
        None
    }

    /// Optional GPU backdrop renderer called by [`crate::paint_prism_backdrop_ui`].
    ///
    /// Returning `true` indicates the provider painted the backdrop on the GPU (and populated
    /// its blur source texture); returning `false` falls back to CPU mesh rendering.
    fn render_backdrop(
        &self,
        _ui: &Ui,
        _rect: Rect,
        _theme: &PrismTheme,
        _time_secs: f32,
        _intensity: f32,
    ) -> bool {
        false
    }
}

/// Default zero-dependency fallback provider.
#[derive(Debug, Default, Clone, Copy)]
pub struct FallbackBlurProvider;

impl BackdropBlurProvider for FallbackBlurProvider {
    fn capability(&self) -> BlurCapability {
        BlurCapability::FallbackOnly
    }
}

#[derive(Clone, Default)]
struct StoredBlurProvider(Option<Arc<dyn BackdropBlurProvider>>);

/// Installs a [`BackdropBlurProvider`] into an [`egui::Context`].
pub fn set_blur_provider(ctx: &Context, provider: Arc<dyn BackdropBlurProvider>) {
    ctx.data_mut(|data| {
        data.insert_temp(
            Id::NULL.with(BLUR_PROVIDER_STORAGE_ID),
            StoredBlurProvider(Some(provider)),
        );
    });
}

/// Removes any active [`BackdropBlurProvider`] from an [`egui::Context`], reverting to fallback glass.
pub fn clear_blur_provider(ctx: &Context) {
    ctx.data_mut(|data| {
        data.remove_temp::<StoredBlurProvider>(Id::NULL.with(BLUR_PROVIDER_STORAGE_ID));
    });
}

/// Returns the installed [`BackdropBlurProvider`], if any.
pub fn blur_provider(ctx: &Context) -> Option<Arc<dyn BackdropBlurProvider>> {
    ctx.data(|data| data.get_temp::<StoredBlurProvider>(Id::NULL.with(BLUR_PROVIDER_STORAGE_ID)))
        .and_then(|stored| stored.0)
}

/// Returns the effective [`BlurCapability`] for the current frame (respecting `theme.real_blur_enabled`).
pub fn effective_blur_capability(ctx: &Context, theme: &PrismTheme) -> BlurCapability {
    if !theme.real_blur_enabled || theme.surface_style == SurfaceStyle::Minimal {
        return BlurCapability::FallbackOnly;
    }
    blur_provider(ctx).map_or(BlurCapability::FallbackOnly, |p| p.capability())
}

/// Helper used by glass containers (`GlassCard`, `GlassPanel`, `GlassSidebar`) to build
/// either real-GPU-blurred shapes or fallback pure-`egui` shapes seamlessly.
pub(crate) fn resolve_glass_surface_shapes(
    ui: &Ui,
    id: Id,
    rect: Rect,
    tokens: &GlassSurfaceTokens,
    theme: &PrismTheme,
) -> Vec<Shape> {
    if theme.real_blur_enabled
        && theme.surface_style == SurfaceStyle::Glass
        && tokens.blur_radius > 0.1
        && let Some(provider) = blur_provider(ui.ctx())
    {
        let req = BackdropRequest::from_tokens_with_mode(rect, tokens, theme.mode);
        if let Some(shapes) = provider.render_glass_surface(ui, id, &req) {
            return shapes;
        }
    }

    crate::painter::glass_surface_shapes_with_ctx(ui.ctx(), rect, tokens)
}
