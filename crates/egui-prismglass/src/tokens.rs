//! Design tokens for `egui-prismglass`.
//!
//! Tokens decouple visual design decisions (surface tints, gradients, specular rims,
//! elevation shadows, corner radii, spacing, and target blur radius) from specific
//! rendering backends. Every token works out-of-the-box with the fallback pure-`egui`
//! renderer and carries metadata (such as [`GlassSurfaceTokens::blur_radius`]) ready
//! for real GPU backdrop-blur backends.

use egui::{Color32, Shadow};

/// Elevation tier for glass surfaces.
///
/// Higher tiers increase surface contrast, top specular sheen, and ambient shadow depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GlassElevation {
    /// Minimal translucency, flat against the canvas with no drop shadow.
    Flat,
    /// Subtle frosted container, ideal for nested sections and chips.
    Subtle,
    /// Standard glass card elevation with soft shadow and specular rim.
    #[default]
    Elevated,
    /// High-prominence floating surface (modals, command palettes, popovers).
    Floating,
}

/// Consistent spacing scale in logical points.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpacingTokens {
    /// Extra-small spacing (`4.0` pt).
    pub xs: f32,
    /// Small spacing (`8.0` pt).
    pub sm: f32,
    /// Medium spacing (`14.0` pt).
    pub md: f32,
    /// Large spacing (`20.0` pt).
    pub lg: f32,
    /// Extra-large spacing (`28.0` pt).
    pub xl: f32,
}

impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 14.0,
            lg: 20.0,
            xl: 28.0,
        }
    }
}

/// Consistent corner radius scale in logical points.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RadiusTokens {
    /// Small controls, inputs, tags (`8.0` pt).
    pub sm: f32,
    /// Buttons and compact tiles (`12.0` pt).
    pub md: f32,
    /// Standard glass cards and panels (`18.0` pt).
    pub lg: f32,
    /// Prominent hero containers and modals (`24.0` pt).
    pub xl: f32,
    /// Full pill rounding (`999.0` pt).
    pub pill: f32,
}

impl Default for RadiusTokens {
    fn default() -> Self {
        Self {
            sm: 8.0,
            md: 12.0,
            lg: 18.0,
            xl: 24.0,
            pill: 999.0,
        }
    }
}

/// Semantic color palette defining the ambient backdrop, typography, and accents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrismPalette {
    /// Base canvas color (top-left of ambient backdrop).
    pub canvas: Color32,
    /// Secondary canvas color (bottom-right of ambient backdrop).
    pub canvas_secondary: Color32,
    /// Primary ambient glow orb color (typically prism periwinkle / indigo).
    pub orb_primary: Color32,
    /// Secondary ambient glow orb color (typically twilight violet / slate blue).
    pub orb_secondary: Color32,
    /// Warm accent glow orb color (typically solar amber / coral).
    pub orb_accent: Color32,
    /// High-emphasis primary text color.
    pub text_primary: Color32,
    /// Medium-emphasis secondary text color.
    pub text_secondary: Color32,
    /// Low-emphasis muted caption / metadata color.
    pub text_muted: Color32,
    /// Primary brand accent (prism periwinkle / electric indigo).
    pub accent: Color32,
    /// Secondary warm accent (solar orange `#EC630D`).
    pub accent_warm: Color32,
    /// Semantic success color.
    pub success: Color32,
    /// Semantic warning color.
    pub warning: Color32,
    /// Semantic danger / error color.
    pub danger: Color32,
}

/// Complete material definition for a single glass surface.
///
/// Used by [`crate::painter::paint_glass_surface`] for fallback glass rendering and
/// by [`crate::blur::BackdropRequest`] when a real backdrop-blur provider is active.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GlassSurfaceTokens {
    /// Core translucent surface tint (unmultiplied sRGBA).
    pub tint: Color32,
    /// Top-left gradient stop for frosted sheen.
    pub gradient_top: Color32,
    /// Bottom-right gradient stop for depth.
    pub gradient_bottom: Color32,
    /// Perimeter hairline glass border color.
    pub border: Color32,
    /// Luminous top-edge specular highlight simulating overhead light refraction.
    pub specular_top: Color32,
    /// Prismatic chromatic highlight tint blended along upper edges.
    pub specular_accent: Color32,
    /// Soft inner reflection band near the top of the glass pane.
    pub inner_glow: Color32,
    /// Ambient drop shadow for floating depth.
    pub shadow: Shadow,
    /// Uniform corner radius in logical points.
    pub corner_radius: f32,
    /// Target backdrop blur radius in logical points (for real-blur upgrades).
    pub blur_radius: f32,
    /// Perimeter border stroke width in logical points.
    pub border_width: f32,
    /// Strength (`0.0..=1.0`) of the prismatic refraction shimmer along the top rim.
    pub prism_intensity: f32,
}

impl GlassSurfaceTokens {
    /// Returns a copy with an overridden corner radius.
    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius.max(0.0);
        self
    }

    /// Returns a copy with an overridden target blur radius.
    pub fn with_blur_radius(mut self, blur_radius: f32) -> Self {
        self.blur_radius = blur_radius.max(0.0);
        self
    }

    /// Returns a copy with an overridden prism refraction intensity (`0.0..=1.0`).
    pub fn with_prism_intensity(mut self, intensity: f32) -> Self {
        self.prism_intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Returns a copy with a custom core tint color, also nudging top/bottom gradient stops.
    pub fn with_tint(mut self, tint: Color32) -> Self {
        self.tint = tint;
        self.gradient_top = blend_colors(self.gradient_top, tint, 0.55);
        self.gradient_bottom = blend_colors(self.gradient_bottom, tint, 0.55);
        self
    }

    /// Scales the alpha of the translucent surface fill (`0.2..=1.8`).
    pub fn with_opacity_scale(mut self, scale: f32) -> Self {
        let s = scale.clamp(0.15, 2.0);
        self.tint = scale_alpha(self.tint, s);
        self.gradient_top = scale_alpha(self.gradient_top, s);
        self.gradient_bottom = scale_alpha(self.gradient_bottom, s);
        self
    }

    /// Scales the specular rim and inner glow intensity (`0.0..=2.0`).
    pub fn with_specular_scale(mut self, scale: f32) -> Self {
        let s = scale.clamp(0.0, 2.0);
        self.specular_top = scale_alpha(self.specular_top, s);
        self.specular_accent = scale_alpha(self.specular_accent, s);
        self.inner_glow = scale_alpha(self.inner_glow, s);
        self
    }

    /// Returns an elevated visual variant suitable for hovered interactive surfaces.
    pub fn hovered(mut self) -> Self {
        self.gradient_top = scale_alpha(self.gradient_top, 1.18);
        self.gradient_bottom = scale_alpha(self.gradient_bottom, 1.12);
        self.border = scale_alpha(self.border, 1.45);
        self.specular_top = scale_alpha(self.specular_top, 1.35);
        self.inner_glow = scale_alpha(self.inner_glow, 1.25);
        self.prism_intensity = (self.prism_intensity + 0.2).min(1.0);
        self
    }

    /// Returns a pressed/active visual variant for interactive controls.
    pub fn pressed(mut self) -> Self {
        self.gradient_top = scale_alpha(self.gradient_top, 0.92);
        self.gradient_bottom = scale_alpha(self.gradient_bottom, 1.05);
        self.border = scale_alpha(self.border, 1.6);
        self.specular_top = scale_alpha(self.specular_top, 0.85);
        self
    }
}

/// Scales the alpha channel of a [`Color32`] while preserving its RGB values.
pub(crate) fn scale_alpha(color: Color32, factor: f32) -> Color32 {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let new_a = (f32::from(a) * factor).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgba_unmultiplied(r, g, b, new_a)
}

/// Linearly interpolates two unmultiplied sRGBA [`Color32`] values.
pub(crate) fn blend_colors(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let [ar, ag, ab, aa] = a.to_srgba_unmultiplied();
    let [br, bg, bb, ba] = b.to_srgba_unmultiplied();
    let lerp = |x: u8, y: u8| -> u8 {
        (f32::from(x) + (f32::from(y) - f32::from(x)) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color32::from_rgba_unmultiplied(lerp(ar, br), lerp(ag, bg), lerp(ab, bb), lerp(aa, ba))
}
