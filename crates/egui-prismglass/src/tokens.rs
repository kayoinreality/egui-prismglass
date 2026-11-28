//! Optical design tokens for `egui-prismglass`.
//!
//! Tokens decouple visual material properties (translucency tint, gradients, directional 3D
//! bevels, dual shadows, corner radii, blur radius, vibrancy, optical edge refraction,
//! prismatic chromatic dispersion, and frosted micro-grain) from specific rendering backends.
//!
//! Every token works out-of-the-box with the pure-`egui` fallback renderer and unlocks full
//! optical fidelity automatically when `feature = "blur-wgpu"` or `feature = "blur-glow"` is active.

use egui::{Color32, Shadow};

/// Elevation tier for glass surfaces.
///
/// Higher tiers increase blur depth, directional bevel intensity, and 3D shadow elevation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GlassElevation {
    /// Minimal translucency, flat against the canvas with no drop shadow.
    Flat,
    /// Subtle frosted container, ideal for nested sections and list rows.
    Subtle,
    /// Standard glass card elevation with grounded contact shadow and 3D bevel.
    #[default]
    Elevated,
    /// High-prominence floating surface (modals, command palettes, popovers).
    Floating,
}

/// Built-in procedural studio backdrop scenes behind glass surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BackdropScene {
    /// Sleek 3D sculpted volumetric titanium/frost studio light waves (default).
    #[default]
    SculptedWaves,
    /// Apple-inspired luminous studio ribbons with rich chromatic depth behind glass.
    LiquidPrism,
    /// High-contrast architectural rings and precision light beams (ideal for inspecting real blur & refraction).
    ArchitecturalGrid,
    /// Clean, distraction-free architectural studio gradient with no diagonal bars, waves, or grid.
    /// Ideal for pro workstations, streaming studios, DAWs, and video editors.
    StudioNeutral,
    /// Soft multi-color aurora wallpaper (violet, blue, teal and pink light pools). The default
    /// scene of the Prism presets: gives glass sidebars and popovers colorful light to frost
    /// without competing with content.
    Aurora,
    /// Disables the procedural backdrop completely. `paint_prism_backdrop_ui` becomes a no-op and
    /// `blur-wgpu` automatically switches glass surfaces to real GPU alpha-blended translucent
    /// overlays over live `egui` content unless an external GPU/RGBA8 backdrop is installed.
    None,
}

impl BackdropScene {
    /// Human-readable label for UI pickers.
    pub fn label(self) -> &'static str {
        match self {
            Self::SculptedWaves => "3D Waves",
            Self::LiquidPrism => "Liquid Prism",
            Self::ArchitecturalGrid => "Studio Grid",
            Self::StudioNeutral => "Studio Neutral",
            Self::Aurora => "Aurora",
            Self::None => "None",
        }
    }

    /// Numeric identifier passed to GPU WGSL backdrop shaders.
    pub fn shader_id(self) -> u32 {
        match self {
            Self::SculptedWaves => 0,
            Self::LiquidPrism => 1,
            Self::ArchitecturalGrid => 2,
            Self::StudioNeutral => 3,
            Self::None => 4,
            Self::Aurora => 5,
        }
    }
}

/// Curated vibrant studio accent palettes supported across every Glass and Minimal theme.
///
/// Apply via [`crate::PrismTheme::with_accent_palette`] or [`crate::PrismTheme::set_accent_palette`]
/// so buttons, sliders, toggles, badges, and glass prism highlights harmonize around your brand color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AccentPalette {
    /// Electric Cobalt (`#2563FF` / `#3B82F6`) — crisp flagship blue.
    #[default]
    Cobalt,
    /// Royal Violet (`#7C3AED` / `#8B5CF6`) — vivid studio indigo-violet.
    Violet,
    /// Emerald Pulse (`#059669` / `#10B981`) — fresh luminous emerald green.
    Emerald,
    /// Crimson Ruby (`#E11D48` / `#F43F5E`) — bold ruby-coral red.
    Ruby,
    /// Solar Amber (`#EA580C` / `#F97316`) — energetic warm tangerine-amber.
    Amber,
    /// Electric Cyan (`#0284C7` / `#06B6D4`) — bright glacial cyan.
    Cyan,
    /// Magenta Rose (`#C026D3` / `#EC4899`) — vibrant magenta-fuchsia.
    Magenta,
    /// Monochrome Ink (`#18181B` light / `#F4F4F5` dark) — pure high-contrast monochrome.
    Graphite,
}

impl AccentPalette {
    /// All 8 built-in vibrant accent palettes in display order.
    pub const ALL: [Self; 8] = [
        Self::Cobalt,
        Self::Violet,
        Self::Emerald,
        Self::Ruby,
        Self::Amber,
        Self::Cyan,
        Self::Magenta,
        Self::Graphite,
    ];

    /// Human-readable display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Cobalt => "Cobalt",
            Self::Violet => "Violet",
            Self::Emerald => "Emerald",
            Self::Ruby => "Ruby",
            Self::Amber => "Amber",
            Self::Cyan => "Cyan",
            Self::Magenta => "Magenta",
            Self::Graphite => "Ink",
        }
    }

    /// Representative vibrant swatch color for color pickers.
    pub fn swatch(self) -> Color32 {
        match self {
            Self::Cobalt => Color32::from_rgb(47, 118, 255),
            Self::Violet => Color32::from_rgb(132, 76, 255),
            Self::Emerald => Color32::from_rgb(16, 185, 129),
            Self::Ruby => Color32::from_rgb(244, 53, 94),
            Self::Amber => Color32::from_rgb(249, 115, 22),
            Self::Cyan => Color32::from_rgb(6, 182, 212),
            Self::Magenta => Color32::from_rgb(232, 62, 162),
            Self::Graphite => Color32::from_rgb(76, 82, 98),
        }
    }

    /// Primary vibrant accent color calibrated for light (`is_dark = false`) or dark (`is_dark = true`) themes.
    pub fn primary(self, is_dark: bool) -> Color32 {
        match (self, is_dark) {
            (Self::Cobalt, false) => Color32::from_rgb(37, 99, 255),
            (Self::Cobalt, true) => Color32::from_rgb(66, 138, 255),
            (Self::Violet, false) => Color32::from_rgb(124, 58, 237),
            (Self::Violet, true) => Color32::from_rgb(148, 98, 255),
            (Self::Emerald, false) => Color32::from_rgb(5, 150, 105),
            (Self::Emerald, true) => Color32::from_rgb(16, 198, 138),
            (Self::Ruby, false) => Color32::from_rgb(225, 29, 72),
            (Self::Ruby, true) => Color32::from_rgb(248, 68, 104),
            (Self::Amber, false) => Color32::from_rgb(234, 88, 12),
            (Self::Amber, true) => Color32::from_rgb(252, 128, 36),
            (Self::Cyan, false) => Color32::from_rgb(2, 132, 199),
            (Self::Cyan, true) => Color32::from_rgb(14, 196, 226),
            (Self::Magenta, false) => Color32::from_rgb(192, 38, 211),
            (Self::Magenta, true) => Color32::from_rgb(236, 78, 178),
            (Self::Graphite, false) => Color32::from_rgb(24, 26, 32),
            (Self::Graphite, true) => Color32::from_rgb(238, 241, 248),
        }
    }

    /// Accent calibrated for the Prism presets ([`crate::PrismTheme::prism_dark`] and friends).
    ///
    /// Every color keeps at least 4.5:1 contrast against white label text (Graphite in dark
    /// mode flips to a light fill with dark text instead), so solid accent buttons stay legible.
    pub fn prism(self, is_dark: bool) -> Color32 {
        match (self, is_dark) {
            (Self::Cobalt, _) => Color32::from_rgb(79, 91, 255),
            (Self::Violet, _) => Color32::from_rgb(124, 58, 237),
            (Self::Emerald, _) => Color32::from_rgb(13, 127, 98),
            (Self::Ruby, _) => Color32::from_rgb(214, 48, 74),
            (Self::Amber, _) => Color32::from_rgb(194, 65, 12),
            (Self::Cyan, _) => Color32::from_rgb(8, 120, 168),
            (Self::Magenta, _) => Color32::from_rgb(190, 42, 168),
            (Self::Graphite, false) => Color32::from_rgb(24, 24, 27),
            (Self::Graphite, true) => Color32::from_rgb(237, 237, 240),
        }
    }

    /// Secondary highlight / companion accent color for gradients, badges, and glass prism sheen.
    pub fn companion(self, is_dark: bool) -> Color32 {
        match (self, is_dark) {
            (Self::Cobalt, false) => Color32::from_rgb(96, 152, 255),
            (Self::Cobalt, true) => Color32::from_rgb(138, 184, 255),
            (Self::Violet, false) => Color32::from_rgb(167, 122, 255),
            (Self::Violet, true) => Color32::from_rgb(196, 162, 255),
            (Self::Emerald, false) => Color32::from_rgb(52, 211, 153),
            (Self::Emerald, true) => Color32::from_rgb(110, 231, 183),
            (Self::Ruby, false) => Color32::from_rgb(251, 113, 133),
            (Self::Ruby, true) => Color32::from_rgb(253, 156, 172),
            (Self::Amber, false) => Color32::from_rgb(251, 146, 60),
            (Self::Amber, true) => Color32::from_rgb(253, 186, 116),
            (Self::Cyan, false) => Color32::from_rgb(56, 189, 248),
            (Self::Cyan, true) => Color32::from_rgb(125, 224, 252),
            (Self::Magenta, false) => Color32::from_rgb(232, 121, 249),
            (Self::Magenta, true) => Color32::from_rgb(249, 168, 212),
            (Self::Graphite, false) => Color32::from_rgb(82, 88, 102),
            (Self::Graphite, true) => Color32::from_rgb(176, 182, 198),
        }
    }
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
    /// Buttons and compact tiles (`10.0` pt).
    pub md: f32,
    /// Standard glass cards and panels (`14.0` pt).
    pub lg: f32,
    /// Prominent hero containers and modals (`18.0` pt).
    pub xl: f32,
    /// Full pill rounding (`999.0` pt).
    pub pill: f32,
}

impl Default for RadiusTokens {
    fn default() -> Self {
        Self {
            sm: 8.0,
            md: 10.0,
            lg: 14.0,
            xl: 18.0,
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
    /// Primary ambient volumetric tone.
    pub orb_primary: Color32,
    /// Secondary ambient volumetric tone.
    pub orb_secondary: Color32,
    /// Luminous highlight spine tone.
    pub orb_accent: Color32,
    /// High-emphasis primary text color.
    pub text_primary: Color32,
    /// Medium-emphasis secondary text color.
    pub text_secondary: Color32,
    /// Low-emphasis muted caption / metadata color.
    pub text_muted: Color32,
    /// Primary brand accent (`#3279F9`).
    pub accent: Color32,
    /// Secondary accent highlight (`#8AB4F8`).
    pub accent_warm: Color32,
    /// Semantic success color.
    pub success: Color32,
    /// Semantic warning color.
    pub warning: Color32,
    /// Semantic danger / error color.
    pub danger: Color32,
}

/// Neutral interface colors shared by controls (segmented controls, inputs, tracks, rows).
///
/// The Prism presets define these from a 12-step gray ramp; the classic presets derive them
/// from their palette so newer widgets ([`crate::GlassSegmented`], [`crate::GlassProgress`])
/// work with every theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrismNeutrals {
    /// Main content background (solid panels in the Layered style).
    pub surface: Color32,
    /// Slightly raised fill: inputs, secondary buttons, stat strips, segmented tracks.
    pub raised: Color32,
    /// Recessed fill: slider rails, progress tracks, switches in the off state.
    pub sunken: Color32,
    /// The selected segment of a segmented control.
    pub control_on: Color32,
    /// Default hairline border.
    pub border: Color32,
    /// Stronger hairline for interactive controls and dividers inside floating layers.
    pub border_strong: Color32,
    /// Hover overlay for rows and ghost buttons.
    pub hover: Color32,
    /// Selected row / navigation item overlay.
    pub selected: Color32,
    /// De-emphasized status color (e.g. a queued item's dot).
    pub muted: Color32,
}

impl PrismNeutrals {
    /// Neutral ramp of the Prism dark presets.
    pub const DARK: Self = Self {
        surface: Color32::from_rgb(17, 17, 20),
        raised: Color32::from_rgb(23, 23, 27),
        sunken: Color32::from_rgb(34, 34, 40),
        control_on: Color32::from_rgb(42, 42, 49),
        border: Color32::from_rgba_premultiplied(18, 18, 18, 18),
        border_strong: Color32::from_rgba_premultiplied(31, 31, 31, 31),
        hover: Color32::from_rgba_premultiplied(10, 10, 10, 10),
        selected: Color32::from_rgba_premultiplied(20, 20, 20, 20),
        muted: Color32::from_rgb(92, 92, 102),
    };

    /// Neutral ramp of the Prism light presets.
    pub const LIGHT: Self = Self {
        surface: Color32::from_rgb(255, 255, 255),
        raised: Color32::from_rgb(247, 247, 248),
        sunken: Color32::from_rgb(236, 236, 239),
        control_on: Color32::from_rgb(255, 255, 255),
        border: Color32::from_rgba_premultiplied(0, 0, 0, 18),
        border_strong: Color32::from_rgba_premultiplied(0, 0, 0, 31),
        hover: Color32::from_rgba_premultiplied(0, 0, 0, 8),
        selected: Color32::from_rgba_premultiplied(0, 0, 0, 15),
        muted: Color32::from_rgb(196, 196, 204),
    };

    /// Derives neutrals for a classic preset from its canvas and text colors.
    pub fn derived(is_dark: bool, palette: &PrismPalette) -> Self {
        let base = if is_dark { Self::DARK } else { Self::LIGHT };
        Self {
            surface: palette.canvas,
            muted: palette.text_muted,
            ..base
        }
    }
}

/// Complete optical material definition for a single glass or minimal surface.
///
/// Consumed by [`crate::painter::glass_surface_shapes`] in fallback mode and by GPU
/// backdrop-blur backends (`blur-wgpu` / `blur-glow`) when active.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GlassSurfaceTokens {
    /// Core translucent surface tint (unmultiplied sRGBA).
    pub tint: Color32,
    /// Upper gradient stop for subtle frosted depth.
    pub gradient_top: Color32,
    /// Lower gradient stop for subtle depth.
    pub gradient_bottom: Color32,
    /// Perimeter hairline glass border color.
    pub border: Color32,
    /// Luminous top-left key-light specular highlight along the machined bevel.
    pub specular_top: Color32,
    /// Subtle prismatic refraction highlight tint blended along upper edges.
    pub specular_accent: Color32,
    /// Subtle inner thickness bevel rim (`1px` inset).
    pub inner_glow: Color32,
    /// Wide ambient elevation shadow.
    pub shadow: Shadow,
    /// Tight, dense 3D contact occlusion shadow anchoring the surface.
    pub contact_shadow: Shadow,
    /// Uniform corner radius in logical points.
    pub corner_radius: f32,
    /// Target backdrop blur radius in logical points (`0.0..=48.0`).
    pub blur_radius: f32,
    /// Perimeter border stroke width in logical points.
    pub border_width: f32,
    /// Strength (`0.0..=1.0`) of directional rim tinting.
    pub prism_intensity: f32,
    /// macOS-style vibrancy saturation multiplier (`1.0` = neutral, `1.15..=1.35` = vibrant glass).
    pub vibrancy: f32,
    /// Optical lens refraction intensity along rounded glass bevel edges (`0.0..=1.0`).
    pub refraction_strength: f32,
    /// Subtle RGB chromatic dispersion along the refracted glass bevel (`0.0..=1.0`).
    pub chromatic_dispersion: f32,
    /// Tactile frosted micro-grain amplitude (`0.0..=0.08`) to eliminate 8-bit banding.
    pub grain_intensity: f32,
    /// Multiplier (`0.15..=1.0`) applied to tint alpha when real GPU backdrop blur is active.
    pub blur_tint_alpha: f32,
}

impl GlassSurfaceTokens {
    /// Returns a copy with an overridden corner radius.
    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius.max(0.0);
        self
    }

    /// Returns a copy with an overridden target blur radius (`0.0..=48.0` logical points).
    pub fn with_blur_radius(mut self, blur_radius: f32) -> Self {
        self.blur_radius = blur_radius.clamp(0.0, 48.0);
        self
    }

    /// Returns a copy with an overridden prism rim intensity (`0.0..=1.0`).
    pub fn with_prism_intensity(mut self, intensity: f32) -> Self {
        self.prism_intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Returns a copy with an overridden vibrancy saturation multiplier (`0.5..=2.0`).
    pub fn with_vibrancy(mut self, vibrancy: f32) -> Self {
        self.vibrancy = vibrancy.clamp(0.5, 2.0);
        self
    }

    /// Returns a copy with an overridden Apple Liquid Glass edge refraction strength (`0.0..=1.0`).
    pub fn with_refraction(mut self, strength: f32) -> Self {
        self.refraction_strength = strength.clamp(0.0, 1.0);
        self
    }

    /// Returns a copy with an overridden bevel chromatic dispersion intensity (`0.0..=1.0`).
    pub fn with_dispersion(mut self, dispersion: f32) -> Self {
        self.chromatic_dispersion = dispersion.clamp(0.0, 1.0);
        self
    }

    /// Returns a copy with an overridden frosted micro-grain intensity (`0.0..=0.10`).
    pub fn with_grain(mut self, grain: f32) -> Self {
        self.grain_intensity = grain.clamp(0.0, 0.10);
        self
    }

    /// Returns a copy with an overridden contact shadow.
    pub fn with_contact_shadow(mut self, shadow: Shadow) -> Self {
        self.contact_shadow = shadow;
        self
    }

    /// Returns a copy with a custom core tint color, also nudging top/bottom gradient stops.
    pub fn with_tint(mut self, tint: Color32) -> Self {
        self.tint = tint;
        self.gradient_top = blend_colors(self.gradient_top, tint, 0.55);
        self.gradient_bottom = blend_colors(self.gradient_bottom, tint, 0.55);
        self
    }

    /// Scales the alpha of the translucent surface fill (`0.15..=2.0`).
    pub fn with_opacity_scale(mut self, scale: f32) -> Self {
        let s = scale.clamp(0.15, 2.0);
        self.tint = scale_alpha(self.tint, s);
        self.gradient_top = scale_alpha(self.gradient_top, s);
        self.gradient_bottom = scale_alpha(self.gradient_bottom, s);
        self
    }

    /// Scales the specular rim and inner thickness edge intensity (`0.0..=2.0`).
    pub fn with_specular_scale(mut self, scale: f32) -> Self {
        let s = scale.clamp(0.0, 2.0);
        self.specular_top = scale_alpha(self.specular_top, s);
        self.specular_accent = scale_alpha(self.specular_accent, s);
        self.inner_glow = scale_alpha(self.inner_glow, s);
        self
    }

    /// Adapts surface tint translucency for real GPU backdrop blur so the blurred backdrop
    /// shines through cleanly rather than being masked by opaque fallback fill alpha.
    pub fn resolved_for_real_blur(mut self) -> Self {
        let mult = self.blur_tint_alpha.clamp(0.15, 1.0);
        self.tint = scale_alpha(self.tint, mult);
        self.gradient_top = scale_alpha(self.gradient_top, mult);
        self.gradient_bottom = scale_alpha(self.gradient_bottom, mult);
        self
    }

    /// Returns an elevated visual variant suitable for hovered interactive surfaces.
    pub fn hovered(mut self) -> Self {
        self.gradient_top = scale_alpha(self.gradient_top, 1.14);
        self.gradient_bottom = scale_alpha(self.gradient_bottom, 1.10);
        self.border = scale_alpha(self.border, 1.38);
        self.specular_top = scale_alpha(self.specular_top, 1.30);
        self.inner_glow = scale_alpha(self.inner_glow, 1.25);
        self.prism_intensity = (self.prism_intensity + 0.15).min(1.0);
        self
    }

    /// Returns a pressed/active visual variant for interactive controls.
    pub fn pressed(mut self) -> Self {
        self.gradient_top = scale_alpha(self.gradient_top, 0.92);
        self.gradient_bottom = scale_alpha(self.gradient_bottom, 1.05);
        self.border = scale_alpha(self.border, 1.5);
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
