//! Theme system and built-in presets for `egui-prismglass`.
//!
//! Built around a clean, architectural neutral palette inspired by Google Antigravity
//! (`#090A0D` / `#121317` Onyx, `#F8F9FC` Frost, `#3279F9` Electric Blue, brushed titanium),
//! supporting two first-class visual styles through the same unified widget API:
//! - [`SurfaceStyle::Glass`]: 3D Apple Liquid Glass (real GPU Dual-Kawase blur when enabled,
//!   or refined pure-`egui` fallback with zero GPU dependencies).
//! - [`SurfaceStyle::Minimal`]: Solid, ultra-clean flat surfaces (Linear / Vercel / Reduced Transparency).

use egui::{Color32, Context, CornerRadius, Id, Shadow, Stroke, Visuals};

use crate::tokens::{
    AccentPalette, BackdropScene, GlassElevation, GlassSurfaceTokens, PrismPalette, RadiusTokens,
    SpacingTokens,
};

const THEME_STORAGE_ID: &str = "egui_prismglass_active_theme";

/// Color polarity of a [`PrismTheme`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ThemeMode {
    /// Deep carbon-black / Onyx canvas.
    #[default]
    Dark,
    /// Airy Frost canvas (`#F8F9FC`).
    Light,
}

/// High-level surface rendering style for a [`PrismTheme`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SurfaceStyle {
    /// Translucent Apple-style frosted/liquid glass surfaces over an atmospheric backdrop.
    #[default]
    Glass,
    /// Solid, opaque minimalist surfaces over a clean flat canvas (also ideal for Reduced Transparency).
    Minimal,
}

/// Built-in visual presets shipped with `egui-prismglass`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrismPreset {
    /// Antigravity **Frost** light glass (`#F8F9FC`) with crystalline translucent white panes and `#3279F9` accents.
    Frost,
    /// **Onyx** dark 3D smoked-crystal glass (`#090A0D`) with sculpted titanium lighting and `#3279F9` accents.
    #[default]
    Onyx,
    /// High-clarity **Glacial** 3D crystal glass with stronger refraction and platinum-ice depth.
    Glacial,
    /// Ultra-clean solid **Minimal Light** (`#FFFFFF` cards on `#F6F8FB`, zero translucency, hairline borders).
    MinimalLight,
    /// Ultra-clean solid **Minimal Dark** (`#18191E` cards on `#121317`, zero translucency, hairline borders).
    MinimalDark,
}

impl PrismPreset {
    /// Human-readable display label for UI selectors.
    pub fn label(self) -> &'static str {
        match self {
            Self::Frost => "Frost Light",
            Self::Onyx => "Onyx Dark",
            Self::Glacial => "Glacial Glass",
            Self::MinimalLight => "Minimal Light",
            Self::MinimalDark => "Minimal Dark",
        }
    }

    /// Returns whether this preset uses translucent glass or solid minimal surfaces.
    pub fn surface_style(self) -> SurfaceStyle {
        match self {
            Self::Frost | Self::Onyx | Self::Glacial => SurfaceStyle::Glass,
            Self::MinimalLight | Self::MinimalDark => SurfaceStyle::Minimal,
        }
    }

    /// Instantiates the corresponding [`PrismTheme`].
    pub fn into_theme(self) -> PrismTheme {
        match self {
            Self::Frost => PrismTheme::frost(),
            Self::Onyx => PrismTheme::onyx(),
            Self::Glacial => PrismTheme::glacial(),
            Self::MinimalLight => PrismTheme::minimal_light(),
            Self::MinimalDark => PrismTheme::minimal_dark(),
        }
    }
}

/// Complete design system theme for `egui-prismglass`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrismTheme {
    /// Display name of this theme.
    pub name: String,
    /// Light or dark polarity.
    pub mode: ThemeMode,
    /// Translucent glass or solid minimalist surface style.
    pub surface_style: SurfaceStyle,
    /// Procedural studio backdrop scene used when painting the canvas in [`SurfaceStyle::Glass`] mode.
    pub backdrop_scene: BackdropScene,
    /// Active vibrant accent color palette applied across widgets and highlights.
    pub accent_palette: AccentPalette,
    /// Semantic colors and subtle backdrop luminance tones.
    pub palette: PrismPalette,
    /// Spacing scale.
    pub spacing: SpacingTokens,
    /// Corner radius scale.
    pub radius: RadiusTokens,
    /// Flat/subtle nested surface tokens.
    pub surface_subtle: GlassSurfaceTokens,
    /// Standard elevated card tokens.
    pub surface_card: GlassSurfaceTokens,
    /// High-prominence floating surface tokens.
    pub surface_floating: GlassSurfaceTokens,
    /// Sidebar / navigation rail surface tokens.
    pub surface_sidebar: GlassSurfaceTokens,
    /// Docked or floating panel surface tokens.
    pub surface_panel: GlassSurfaceTokens,
    /// Whether real GPU backdrop blur is active when a GPU provider is installed (default: `true`).
    pub real_blur_enabled: bool,
    /// Global opacity multiplier (`0.3..=1.6`) applied in [`SurfaceStyle::Glass`] mode.
    pub glass_opacity_scale: f32,
    /// Global edge highlight multiplier (`0.0..=2.0`).
    pub specular_scale: f32,
    /// Global corner radius multiplier (`0.0..=1.6`).
    pub radius_scale: f32,
    /// Global rim refraction tint multiplier (`0.0..=1.5`).
    pub prism_scale: f32,
    /// Global backdrop blur radius multiplier (`0.0..=2.2`).
    pub blur_radius_scale: f32,
    /// Global curved-edge optical refraction multiplier (`0.0..=2.0`).
    pub refraction_scale: f32,
    /// Global chromatic dispersion multiplier (`0.0..=2.0`).
    pub dispersion_scale: f32,
    /// Global macOS-style vibrancy multiplier (`0.5..=1.6`).
    pub vibrancy_scale: f32,
    /// Global frosted micro-grain multiplier (`0.0..=2.5`).
    pub grain_scale: f32,
}

impl Default for PrismTheme {
    fn default() -> Self {
        Self::onyx()
    }
}

impl PrismTheme {
    /// Creates the **Onyx** dark 3D smoked-glass theme (Apple Pro / Raycast dark studio aesthetic).
    pub fn onyx() -> Self {
        let palette = PrismPalette {
            canvas: Color32::from_rgb(9, 10, 13),
            canvas_secondary: Color32::from_rgb(14, 15, 19),
            orb_primary: Color32::from_rgb(138, 146, 166),
            orb_secondary: Color32::from_rgb(186, 194, 210),
            orb_accent: Color32::from_rgb(232, 236, 245),
            text_primary: Color32::from_rgb(248, 249, 252),
            text_secondary: Color32::from_rgb(166, 172, 184),
            text_muted: Color32::from_rgb(104, 109, 120),
            accent: Color32::from_rgb(50, 121, 249),
            accent_warm: Color32::from_rgb(138, 180, 248),
            success: Color32::from_rgb(52, 199, 123),
            warning: Color32::from_rgb(234, 179, 8),
            danger: Color32::from_rgb(239, 68, 68),
        };

        let radius = RadiusTokens::default();

        let contact_sm = Shadow {
            offset: [0, 2],
            blur: 6,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(0, 0, 0, 115),
        };
        let contact_lg = Shadow {
            offset: [0, 3],
            blur: 10,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(0, 0, 0, 145),
        };

        let surface_subtle = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(15, 16, 21, 142),
            gradient_top: Color32::from_rgba_unmultiplied(24, 26, 33, 150),
            gradient_bottom: Color32::from_rgba_unmultiplied(11, 12, 16, 156),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 22),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 62),
            specular_accent: Color32::from_rgba_unmultiplied(220, 228, 245, 58),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 14),
            shadow: Shadow::NONE,
            contact_shadow: Shadow::NONE,
            corner_radius: radius.md,
            blur_radius: 14.0,
            border_width: 1.0,
            prism_intensity: 0.35,
            vibrancy: 1.24,
            refraction_strength: 0.62,
            chromatic_dispersion: 0.28,
            grain_intensity: 0.012,
            blur_tint_alpha: 0.52,
        };

        let surface_card = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(15, 18, 28, 178),
            gradient_top: Color32::from_rgba_unmultiplied(24, 28, 42, 182),
            gradient_bottom: Color32::from_rgba_unmultiplied(10, 12, 20, 198),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 40),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 165),
            specular_accent: Color32::from_rgba_unmultiplied(186, 212, 255, 110),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 34),
            shadow: Shadow {
                offset: [0, 12],
                blur: 30,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 140),
            },
            contact_shadow: contact_sm,
            corner_radius: radius.lg,
            blur_radius: 14.0,
            border_width: 1.0,
            prism_intensity: 0.42,
            vibrancy: 1.30,
            refraction_strength: 0.82,
            chromatic_dispersion: 0.42,
            grain_intensity: 0.014,
            blur_tint_alpha: 0.48,
        };

        let surface_floating = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(18, 21, 32, 190),
            gradient_top: Color32::from_rgba_unmultiplied(28, 33, 48, 194),
            gradient_bottom: Color32::from_rgba_unmultiplied(12, 14, 24, 210),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 50),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 195),
            specular_accent: Color32::from_rgba_unmultiplied(200, 222, 255, 130),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 42),
            shadow: Shadow {
                offset: [0, 18],
                blur: 42,
                spread: 1,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 175),
            },
            contact_shadow: contact_lg,
            corner_radius: radius.lg,
            blur_radius: 18.0,
            border_width: 1.0,
            prism_intensity: 0.52,
            vibrancy: 1.34,
            refraction_strength: 0.90,
            chromatic_dispersion: 0.50,
            grain_intensity: 0.015,
            blur_tint_alpha: 0.44,
        };

        let surface_sidebar = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(13, 16, 25, 182),
            gradient_top: Color32::from_rgba_unmultiplied(20, 24, 36, 186),
            gradient_bottom: Color32::from_rgba_unmultiplied(9, 11, 18, 202),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 38),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 150),
            specular_accent: Color32::from_rgba_unmultiplied(186, 212, 255, 96),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 30),
            shadow: Shadow {
                offset: [0, 12],
                blur: 32,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 140),
            },
            contact_shadow: contact_sm,
            corner_radius: radius.xl,
            blur_radius: 16.0,
            border_width: 1.0,
            prism_intensity: 0.38,
            vibrancy: 1.26,
            refraction_strength: 0.78,
            chromatic_dispersion: 0.34,
            grain_intensity: 0.014,
            blur_tint_alpha: 0.50,
        };

        let surface_panel = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(14, 17, 26, 178),
            gradient_top: Color32::from_rgba_unmultiplied(22, 26, 38, 182),
            gradient_bottom: Color32::from_rgba_unmultiplied(10, 12, 20, 196),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 40),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 155),
            specular_accent: Color32::from_rgba_unmultiplied(186, 212, 255, 100),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 32),
            shadow: Shadow {
                offset: [0, 10],
                blur: 24,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 125),
            },
            contact_shadow: contact_sm,
            corner_radius: radius.lg,
            blur_radius: 14.0,
            border_width: 1.0,
            prism_intensity: 0.38,
            vibrancy: 1.26,
            refraction_strength: 0.76,
            chromatic_dispersion: 0.34,
            grain_intensity: 0.014,
            blur_tint_alpha: 0.48,
        };

        Self {
            name: "Onyx Dark".to_owned(),
            mode: ThemeMode::Dark,
            surface_style: SurfaceStyle::Glass,
            backdrop_scene: BackdropScene::SculptedWaves,
            accent_palette: AccentPalette::Cobalt,
            palette,
            spacing: SpacingTokens::default(),
            radius,
            surface_subtle,
            surface_card,
            surface_floating,
            surface_sidebar,
            surface_panel,
            real_blur_enabled: true,
            glass_opacity_scale: 1.0,
            specular_scale: 1.0,
            radius_scale: 1.0,
            prism_scale: 1.0,
            blur_radius_scale: 1.0,
            refraction_scale: 1.0,
            dispersion_scale: 1.0,
            vibrancy_scale: 1.0,
            grain_scale: 1.0,
        }
    }

    /// Creates the airy **Frost Light** glass theme inspired by Google Antigravity + macOS glass.
    pub fn frost() -> Self {
        let palette = PrismPalette {
            canvas: Color32::from_rgb(248, 249, 252),
            canvas_secondary: Color32::from_rgb(235, 240, 248),
            orb_primary: Color32::from_rgb(50, 121, 249),
            orb_secondary: Color32::from_rgb(138, 180, 248),
            orb_accent: Color32::from_rgb(183, 191, 217),
            text_primary: Color32::from_rgb(18, 19, 23),
            text_secondary: Color32::from_rgb(69, 71, 77),
            text_muted: Color32::from_rgb(106, 106, 113),
            accent: Color32::from_rgb(50, 121, 249),
            accent_warm: Color32::from_rgb(26, 115, 232),
            success: Color32::from_rgb(24, 146, 92),
            warning: Color32::from_rgb(196, 124, 16),
            danger: Color32::from_rgb(217, 48, 37),
        };

        let radius = RadiusTokens::default();
        let contact_light = Shadow {
            offset: [0, 1],
            blur: 4,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(18, 19, 23, 14),
        };

        let surface_subtle = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(255, 255, 255, 138),
            gradient_top: Color32::from_rgba_unmultiplied(255, 255, 255, 156),
            gradient_bottom: Color32::from_rgba_unmultiplied(248, 250, 254, 132),
            border: Color32::from_rgba_unmultiplied(18, 19, 23, 18),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 225),
            specular_accent: Color32::from_rgba_unmultiplied(255, 255, 255, 210),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 150),
            shadow: Shadow::NONE,
            contact_shadow: Shadow::NONE,
            corner_radius: radius.md,
            blur_radius: 12.0,
            border_width: 1.0,
            prism_intensity: 0.2,
            vibrancy: 1.10,
            refraction_strength: 0.38,
            chromatic_dispersion: 0.18,
            grain_intensity: 0.014,
            blur_tint_alpha: 0.62,
        };

        let surface_card = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(255, 255, 255, 172),
            gradient_top: Color32::from_rgba_unmultiplied(255, 255, 255, 192),
            gradient_bottom: Color32::from_rgba_unmultiplied(248, 250, 254, 162),
            border: Color32::from_rgba_unmultiplied(18, 19, 23, 22),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 245),
            specular_accent: Color32::from_rgba_unmultiplied(255, 255, 255, 230),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 180),
            shadow: Shadow {
                offset: [0, 6],
                blur: 20,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(18, 19, 23, 16),
            },
            contact_shadow: contact_light,
            corner_radius: radius.lg,
            blur_radius: 22.0,
            border_width: 1.0,
            prism_intensity: 0.25,
            vibrancy: 1.16,
            refraction_strength: 0.55,
            chromatic_dispersion: 0.28,
            grain_intensity: 0.016,
            blur_tint_alpha: 0.58,
        };

        let surface_floating = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(255, 255, 255, 196),
            gradient_top: Color32::from_rgba_unmultiplied(255, 255, 255, 216),
            gradient_bottom: Color32::from_rgba_unmultiplied(250, 251, 255, 186),
            border: Color32::from_rgba_unmultiplied(18, 19, 23, 24),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 255),
            specular_accent: Color32::from_rgba_unmultiplied(255, 255, 255, 240),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 210),
            shadow: Shadow {
                offset: [0, 10],
                blur: 28,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(18, 19, 23, 22),
            },
            contact_shadow: contact_light,
            corner_radius: radius.lg,
            blur_radius: 30.0,
            border_width: 1.0,
            prism_intensity: 0.3,
            vibrancy: 1.20,
            refraction_strength: 0.68,
            chromatic_dispersion: 0.35,
            grain_intensity: 0.018,
            blur_tint_alpha: 0.55,
        };

        let surface_sidebar = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(255, 255, 255, 166),
            gradient_top: Color32::from_rgba_unmultiplied(255, 255, 255, 184),
            gradient_bottom: Color32::from_rgba_unmultiplied(246, 248, 253, 158),
            border: Color32::from_rgba_unmultiplied(18, 19, 23, 20),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 240),
            specular_accent: Color32::from_rgba_unmultiplied(255, 255, 255, 225),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 175),
            shadow: Shadow {
                offset: [0, 5],
                blur: 20,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(18, 19, 23, 14),
            },
            contact_shadow: contact_light,
            corner_radius: radius.xl,
            blur_radius: 24.0,
            border_width: 1.0,
            prism_intensity: 0.22,
            vibrancy: 1.14,
            refraction_strength: 0.48,
            chromatic_dispersion: 0.22,
            grain_intensity: 0.015,
            blur_tint_alpha: 0.60,
        };

        let surface_panel = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(255, 255, 255, 174),
            gradient_top: Color32::from_rgba_unmultiplied(255, 255, 255, 192),
            gradient_bottom: Color32::from_rgba_unmultiplied(248, 250, 254, 166),
            border: Color32::from_rgba_unmultiplied(18, 19, 23, 20),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 240),
            specular_accent: Color32::from_rgba_unmultiplied(255, 255, 255, 225),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 175),
            shadow: Shadow {
                offset: [0, 4],
                blur: 16,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(18, 19, 23, 14),
            },
            contact_shadow: contact_light,
            corner_radius: radius.lg,
            blur_radius: 20.0,
            border_width: 1.0,
            prism_intensity: 0.22,
            vibrancy: 1.14,
            refraction_strength: 0.45,
            chromatic_dispersion: 0.22,
            grain_intensity: 0.015,
            blur_tint_alpha: 0.60,
        };

        Self {
            name: "Frost Light".to_owned(),
            mode: ThemeMode::Light,
            surface_style: SurfaceStyle::Glass,
            backdrop_scene: BackdropScene::SculptedWaves,
            accent_palette: AccentPalette::Cobalt,
            palette,
            spacing: SpacingTokens::default(),
            radius,
            surface_subtle,
            surface_card,
            surface_floating,
            surface_sidebar,
            surface_panel,
            real_blur_enabled: true,
            glass_opacity_scale: 1.0,
            specular_scale: 1.0,
            radius_scale: 1.0,
            prism_scale: 1.0,
            blur_radius_scale: 1.0,
            refraction_scale: 1.0,
            dispersion_scale: 1.0,
            vibrancy_scale: 1.0,
            grain_scale: 1.0,
        }
    }

    /// Creates the high-clarity **Glacial Glass** 3D crystal dark theme.
    pub fn glacial() -> Self {
        let mut theme = Self::onyx();
        "Glacial Glass".clone_into(&mut theme.name);
        theme.backdrop_scene = BackdropScene::LiquidPrism;
        theme.palette.canvas = Color32::from_rgb(7, 8, 12);
        theme.palette.canvas_secondary = Color32::from_rgb(13, 16, 24);
        theme.palette.orb_primary = Color32::from_rgb(122, 152, 214);
        theme.palette.orb_secondary = Color32::from_rgb(186, 206, 245);
        theme.backdrop_scene = BackdropScene::LiquidPrism;
        theme.palette.text_primary = Color32::from_rgb(255, 255, 255);
        theme.palette.text_secondary = Color32::from_rgb(218, 226, 248);
        theme.palette.text_muted = Color32::from_rgb(168, 182, 224);
        theme.palette.orb_accent = Color32::from_rgb(240, 245, 255);
        theme.palette.accent = Color32::from_rgb(76, 148, 255);

        theme.surface_subtle.tint = Color32::from_rgba_unmultiplied(22, 34, 78, 138);
        theme.surface_subtle.gradient_top = Color32::from_rgba_unmultiplied(36, 52, 112, 144);
        theme.surface_subtle.gradient_bottom = Color32::from_rgba_unmultiplied(14, 22, 54, 152);
        theme.surface_subtle.border = Color32::from_rgba_unmultiplied(210, 228, 255, 58);
        theme.surface_subtle.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 190);
        theme.surface_subtle.specular_accent = Color32::from_rgba_unmultiplied(156, 200, 255, 135);
        theme.surface_subtle.blur_radius = 10.0;
        theme.surface_subtle.refraction_strength = 0.72;
        theme.surface_subtle.blur_tint_alpha = 0.42;

        theme.surface_card.tint = Color32::from_rgba_unmultiplied(24, 38, 86, 132);
        theme.surface_card.gradient_top = Color32::from_rgba_unmultiplied(44, 64, 130, 138);
        theme.surface_card.gradient_bottom = Color32::from_rgba_unmultiplied(16, 26, 62, 148);
        theme.surface_card.border = Color32::from_rgba_unmultiplied(220, 234, 255, 68);
        theme.surface_card.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 228);
        theme.surface_card.specular_accent = Color32::from_rgba_unmultiplied(168, 208, 255, 156);
        theme.surface_card.inner_glow = Color32::from_rgba_unmultiplied(225, 238, 255, 54);
        theme.surface_card.blur_radius = 10.0;
        theme.surface_card.refraction_strength = 0.92;
        theme.surface_card.chromatic_dispersion = 0.48;
        theme.surface_card.vibrancy = 1.34;
        theme.surface_card.grain_intensity = 0.008;
        theme.surface_card.blur_tint_alpha = 0.34;

        theme.surface_floating.tint = Color32::from_rgba_unmultiplied(28, 42, 94, 138);
        theme.surface_floating.gradient_top = Color32::from_rgba_unmultiplied(52, 72, 144, 144);
        theme.surface_floating.gradient_bottom = Color32::from_rgba_unmultiplied(18, 28, 66, 154);
        theme.surface_floating.border = Color32::from_rgba_unmultiplied(230, 240, 255, 78);
        theme.surface_floating.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 242);
        theme.surface_floating.specular_accent =
            Color32::from_rgba_unmultiplied(180, 214, 255, 170);
        theme.surface_floating.inner_glow = Color32::from_rgba_unmultiplied(235, 244, 255, 64);
        theme.surface_floating.blur_radius = 12.0;
        theme.surface_floating.refraction_strength = 1.00;
        theme.surface_floating.chromatic_dispersion = 0.56;
        theme.surface_floating.vibrancy = 1.38;
        theme.surface_floating.grain_intensity = 0.008;
        theme.surface_floating.blur_tint_alpha = 0.30;

        theme.surface_sidebar.tint = Color32::from_rgba_unmultiplied(18, 28, 68, 148);
        theme.surface_sidebar.gradient_top = Color32::from_rgba_unmultiplied(32, 46, 102, 152);
        theme.surface_sidebar.gradient_bottom = Color32::from_rgba_unmultiplied(12, 20, 48, 166);
        theme.surface_sidebar.border = Color32::from_rgba_unmultiplied(216, 232, 255, 64);
        theme.surface_sidebar.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 210);
        theme.surface_sidebar.specular_accent = Color32::from_rgba_unmultiplied(160, 202, 255, 140);
        theme.surface_sidebar.blur_radius = 12.0;
        theme.surface_sidebar.refraction_strength = 0.88;
        theme.surface_sidebar.grain_intensity = 0.008;
        theme.surface_sidebar.blur_tint_alpha = 0.38;

        theme.surface_panel.tint = Color32::from_rgba_unmultiplied(20, 32, 74, 144);
        theme.surface_panel.gradient_top = Color32::from_rgba_unmultiplied(36, 52, 112, 148);
        theme.surface_panel.gradient_bottom = Color32::from_rgba_unmultiplied(14, 22, 54, 160);
        theme.surface_panel.border = Color32::from_rgba_unmultiplied(218, 234, 255, 66);
        theme.surface_panel.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 215);
        theme.surface_panel.specular_accent = Color32::from_rgba_unmultiplied(164, 204, 255, 144);
        theme.surface_panel.blur_radius = 12.0;
        theme.surface_panel.refraction_strength = 0.86;
        theme.surface_panel.grain_intensity = 0.008;
        theme.surface_panel.blur_tint_alpha = 0.36;

        theme
    }

    /// Creates the **Minimal Light** tactile neumorphic theme (soft extruded solid surfaces,
    /// dual directional shadows, and vibrant solid accents).
    pub fn minimal_light() -> Self {
        let mut theme = Self::frost();
        "Minimal Light".clone_into(&mut theme.name);
        theme.surface_style = SurfaceStyle::Minimal;
        theme.real_blur_enabled = false;
        // Cool porcelain studio canvas — gives pure-white top-left neumorphic light crisp separation.
        theme.palette.canvas = Color32::from_rgb(233, 238, 246);
        theme.palette.canvas_secondary = Color32::from_rgb(227, 233, 243);
        theme.palette.text_primary = Color32::from_rgb(17, 21, 32);
        theme.palette.text_secondary = Color32::from_rgb(82, 92, 112);
        theme.palette.text_muted = Color32::from_rgb(126, 136, 156);

        let neu = |top: Color32,
                   mid: Color32,
                   bottom: Color32,
                   border: Color32,
                   spec_top: Color32,
                   radius: f32,
                   drop_shadow: Shadow,
                   light_shadow: Shadow| GlassSurfaceTokens {
            tint: mid,
            gradient_top: top,
            gradient_bottom: bottom,
            border,
            specular_top: spec_top,
            specular_accent: Color32::TRANSPARENT,
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 130),
            shadow: drop_shadow,
            contact_shadow: light_shadow,
            corner_radius: radius,
            blur_radius: 0.0,
            border_width: 1.0,
            prism_intensity: 0.0,
            vibrancy: 1.0,
            refraction_strength: 0.0,
            chromatic_dispersion: 0.0,
            grain_intensity: 0.0,
            blur_tint_alpha: 1.0,
        };

        let drop_card = Shadow {
            offset: [5, 6],
            blur: 15,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(146, 162, 190, 112),
        };
        let light_card = Shadow {
            offset: [-5, -5],
            blur: 13,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(255, 255, 255, 225),
        };
        let drop_float = Shadow {
            offset: [7, 9],
            blur: 20,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(136, 154, 186, 128),
        };
        let light_float = Shadow {
            offset: [-6, -6],
            blur: 16,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(255, 255, 255, 240),
        };

        // Softly recessed tactile well for nested items (subtle debossed contrast).
        theme.surface_subtle = neu(
            Color32::from_rgb(226, 232, 242),
            Color32::from_rgb(231, 237, 246),
            Color32::from_rgb(237, 241, 249),
            Color32::from_rgba_unmultiplied(178, 192, 214, 120),
            Color32::from_rgba_unmultiplied(186, 198, 218, 95),
            theme.radius.md,
            Shadow::NONE,
            Shadow::NONE,
        );
        theme.surface_subtle.inner_glow = Color32::from_rgba_unmultiplied(255, 255, 255, 95);

        theme.surface_card = neu(
            Color32::from_rgb(245, 248, 253),
            Color32::from_rgb(239, 243, 250),
            Color32::from_rgb(234, 239, 247),
            Color32::from_rgba_unmultiplied(190, 202, 222, 115),
            Color32::from_rgba_unmultiplied(255, 255, 255, 245),
            theme.radius.lg,
            drop_card,
            light_card,
        );
        theme.surface_floating = neu(
            Color32::from_rgb(249, 251, 255),
            Color32::from_rgb(242, 246, 252),
            Color32::from_rgb(236, 241, 249),
            Color32::from_rgba_unmultiplied(184, 198, 220, 125),
            Color32::from_rgba_unmultiplied(255, 255, 255, 255),
            theme.radius.lg,
            drop_float,
            light_float,
        );
        theme.surface_sidebar = neu(
            Color32::from_rgb(243, 247, 252),
            Color32::from_rgb(238, 242, 249),
            Color32::from_rgb(233, 238, 247),
            Color32::from_rgba_unmultiplied(190, 202, 222, 110),
            Color32::from_rgba_unmultiplied(255, 255, 255, 240),
            theme.radius.xl,
            drop_card,
            light_card,
        );
        theme.surface_panel = neu(
            Color32::from_rgb(245, 248, 253),
            Color32::from_rgb(240, 244, 250),
            Color32::from_rgb(235, 240, 248),
            Color32::from_rgba_unmultiplied(190, 202, 222, 115),
            Color32::from_rgba_unmultiplied(255, 255, 255, 245),
            theme.radius.lg,
            drop_card,
            light_card,
        );

        theme.set_accent_palette(AccentPalette::Cobalt);
        theme
    }

    /// Creates the **Minimal Dark** tactile neumorphic theme (deep sculpted slate surfaces,
    /// dual directional dark-mode soft shadows, and vibrant solid accents).
    pub fn minimal_dark() -> Self {
        let mut theme = Self::onyx();
        "Minimal Dark".clone_into(&mut theme.name);
        theme.surface_style = SurfaceStyle::Minimal;
        theme.real_blur_enabled = false;
        // Deep tactile slate-carbon canvas so both upper-left highlight and lower-right shadow read cleanly.
        theme.palette.canvas = Color32::from_rgb(22, 26, 35);
        theme.palette.canvas_secondary = Color32::from_rgb(18, 22, 30);
        theme.palette.text_primary = Color32::from_rgb(246, 248, 253);
        theme.palette.text_secondary = Color32::from_rgb(164, 174, 194);
        theme.palette.text_muted = Color32::from_rgb(108, 118, 138);

        let neu = |top: Color32,
                   mid: Color32,
                   bottom: Color32,
                   border: Color32,
                   spec_top: Color32,
                   radius: f32,
                   drop_shadow: Shadow,
                   light_shadow: Shadow| GlassSurfaceTokens {
            tint: mid,
            gradient_top: top,
            gradient_bottom: bottom,
            border,
            specular_top: spec_top,
            specular_accent: Color32::TRANSPARENT,
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 14),
            shadow: drop_shadow,
            contact_shadow: light_shadow,
            corner_radius: radius,
            blur_radius: 0.0,
            border_width: 1.0,
            prism_intensity: 0.0,
            vibrancy: 1.0,
            refraction_strength: 0.0,
            chromatic_dispersion: 0.0,
            grain_intensity: 0.0,
            blur_tint_alpha: 1.0,
        };

        let drop_card = Shadow {
            offset: [6, 7],
            blur: 16,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(6, 8, 14, 185),
        };
        let light_card = Shadow {
            offset: [-4, -4],
            blur: 12,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(150, 172, 215, 19),
        };
        let drop_float = Shadow {
            offset: [8, 10],
            blur: 22,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(4, 6, 11, 215),
        };
        let light_float = Shadow {
            offset: [-5, -5],
            blur: 14,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(165, 186, 230, 24),
        };

        // Softly recessed dark tactile well for nested items.
        theme.surface_subtle = neu(
            Color32::from_rgb(19, 22, 31),
            Color32::from_rgb(22, 26, 36),
            Color32::from_rgb(25, 30, 41),
            Color32::from_rgba_unmultiplied(255, 255, 255, 16),
            Color32::from_rgba_unmultiplied(8, 10, 16, 160),
            theme.radius.md,
            Shadow::NONE,
            Shadow::NONE,
        );
        theme.surface_subtle.inner_glow = Color32::from_rgba_unmultiplied(255, 255, 255, 8);

        theme.surface_card = neu(
            Color32::from_rgb(33, 39, 52),
            Color32::from_rgb(29, 34, 46),
            Color32::from_rgb(25, 30, 40),
            Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            Color32::from_rgba_unmultiplied(255, 255, 255, 42),
            theme.radius.lg,
            drop_card,
            light_card,
        );
        theme.surface_floating = neu(
            Color32::from_rgb(37, 43, 58),
            Color32::from_rgb(32, 38, 51),
            Color32::from_rgb(27, 32, 43),
            Color32::from_rgba_unmultiplied(255, 255, 255, 24),
            Color32::from_rgba_unmultiplied(255, 255, 255, 52),
            theme.radius.lg,
            drop_float,
            light_float,
        );
        theme.surface_sidebar = neu(
            Color32::from_rgb(30, 35, 47),
            Color32::from_rgb(27, 32, 43),
            Color32::from_rgb(24, 28, 38),
            Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            Color32::from_rgba_unmultiplied(255, 255, 255, 36),
            theme.radius.xl,
            drop_card,
            light_card,
        );
        theme.surface_panel = neu(
            Color32::from_rgb(33, 39, 52),
            Color32::from_rgb(29, 34, 46),
            Color32::from_rgb(26, 30, 41),
            Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            Color32::from_rgba_unmultiplied(255, 255, 255, 42),
            theme.radius.lg,
            drop_card,
            light_card,
        );

        theme.set_accent_palette(AccentPalette::Cobalt);
        theme
    }

    /// Builder that applies a curated vibrant [`AccentPalette`] across the theme and returns `self`.
    #[must_use]
    pub fn with_accent_palette(mut self, accent: AccentPalette) -> Self {
        self.set_accent_palette(accent);
        self
    }

    /// Updates the theme's active vibrant [`AccentPalette`] in place, harmonizing brand accents
    /// and glass prism edge highlights.
    pub fn set_accent_palette(&mut self, accent: AccentPalette) {
        self.accent_palette = accent;
        let is_dark = self.mode == ThemeMode::Dark;
        let primary = accent.primary(is_dark);
        let companion = accent.companion(is_dark);

        self.palette.accent = primary;
        self.palette.accent_warm = companion;

        if self.surface_style == SurfaceStyle::Glass {
            let [red, green, blue, _] = companion.to_srgba_unmultiplied();
            let alpha_card = if is_dark { 138 } else { 96 };
            let alpha_float = if is_dark { 165 } else { 112 };
            self.surface_card.specular_accent =
                Color32::from_rgba_unmultiplied(red, green, blue, alpha_card);
            self.surface_floating.specular_accent =
                Color32::from_rgba_unmultiplied(red, green, blue, alpha_float);
            self.surface_sidebar.specular_accent =
                Color32::from_rgba_unmultiplied(red, green, blue, alpha_card.saturating_sub(18));
            self.surface_panel.specular_accent =
                Color32::from_rgba_unmultiplied(red, green, blue, alpha_card.saturating_sub(12));
        }
    }

    /// Resolves effective [`GlassSurfaceTokens`] for a given [`GlassElevation`],
    /// applying the theme's live optical scale multipliers.
    pub fn surface(&self, elevation: GlassElevation) -> GlassSurfaceTokens {
        let base = match elevation {
            GlassElevation::Flat => {
                let mut t = self.surface_subtle;
                t.shadow = Shadow::NONE;
                t.contact_shadow = Shadow::NONE;
                t.prism_intensity *= 0.5;
                t.blur_radius *= 0.65;
                t
            }
            GlassElevation::Subtle => self.surface_subtle,
            GlassElevation::Elevated => self.surface_card,
            GlassElevation::Floating => self.surface_floating,
        };
        self.apply_scales(base)
    }

    /// Resolves effective [`GlassSurfaceTokens`] for a sidebar container.
    pub fn sidebar_surface(&self) -> GlassSurfaceTokens {
        self.apply_scales(self.surface_sidebar)
    }

    /// Resolves effective [`GlassSurfaceTokens`] for a panel container.
    pub fn panel_surface(&self) -> GlassSurfaceTokens {
        self.apply_scales(self.surface_panel)
    }

    fn apply_scales(&self, tokens: GlassSurfaceTokens) -> GlassSurfaceTokens {
        let scaled =
            tokens.with_corner_radius(tokens.corner_radius * self.radius_scale.clamp(0.0, 2.0));
        match self.surface_style {
            SurfaceStyle::Glass => scaled
                .with_opacity_scale(self.glass_opacity_scale)
                .with_specular_scale(self.specular_scale)
                .with_prism_intensity(tokens.prism_intensity * self.prism_scale.clamp(0.0, 1.8))
                .with_blur_radius(tokens.blur_radius * self.blur_radius_scale.clamp(0.0, 2.5))
                .with_refraction(tokens.refraction_strength * self.refraction_scale.clamp(0.0, 2.0))
                .with_dispersion(
                    tokens.chromatic_dispersion * self.dispersion_scale.clamp(0.0, 2.0),
                )
                .with_vibrancy(tokens.vibrancy * self.vibrancy_scale.clamp(0.5, 1.8))
                .with_grain(tokens.grain_intensity * self.grain_scale.clamp(0.0, 3.0)),
            SurfaceStyle::Minimal => scaled,
        }
    }

    /// Applies this theme to an [`egui::Context`].
    ///
    /// Configures `egui`'s [`Visuals`] and spacing to harmonize with `PrismGlass` surfaces
    /// and stores a copy of this theme in context memory for all widgets to read.
    pub fn apply(&self, ctx: &Context) {
        let mut visuals = match self.mode {
            ThemeMode::Dark => Visuals::dark(),
            ThemeMode::Light => Visuals::light(),
        };

        visuals.override_text_color = Some(self.palette.text_primary);
        visuals.panel_fill = self.palette.canvas;
        visuals.window_fill = self.surface_floating.tint;
        visuals.window_corner_radius = CornerRadius::same(self.radius.lg as u8);
        visuals.window_stroke = Stroke::new(1.0, self.surface_floating.border);
        visuals.window_shadow = self.surface_floating.shadow;
        visuals.faint_bg_color = self.surface_subtle.tint;
        visuals.extreme_bg_color = match self.mode {
            ThemeMode::Dark => Color32::from_rgba_unmultiplied(18, 19, 23, 210),
            ThemeMode::Light => Color32::from_rgba_unmultiplied(238, 241, 246, 230),
        };
        visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(
            self.palette.accent.r(),
            self.palette.accent.g(),
            self.palette.accent.b(),
            64,
        );
        visuals.selection.stroke = Stroke::new(1.0, self.palette.accent);
        visuals.hyperlink_color = self.palette.accent;
        visuals.warn_fg_color = self.palette.warning;
        visuals.error_fg_color = self.palette.danger;

        let widget_radius = CornerRadius::same(self.radius.md as u8);
        visuals.widgets.noninteractive.bg_fill = self.surface_subtle.tint;
        visuals.widgets.noninteractive.weak_bg_fill = self.surface_subtle.tint;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.surface_subtle.border);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.palette.text_secondary);
        visuals.widgets.noninteractive.corner_radius = widget_radius;

        visuals.widgets.inactive.bg_fill = self.surface_subtle.tint;
        visuals.widgets.inactive.weak_bg_fill = self.surface_subtle.tint;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.surface_subtle.border);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.palette.text_primary);
        visuals.widgets.inactive.corner_radius = widget_radius;

        let hovered = self.surface_subtle.hovered();
        visuals.widgets.hovered.bg_fill = hovered.tint;
        visuals.widgets.hovered.weak_bg_fill = hovered.tint;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, hovered.border);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.2, self.palette.text_primary);
        visuals.widgets.hovered.corner_radius = widget_radius;

        let active = self.surface_subtle.pressed();
        visuals.widgets.active.bg_fill = active.tint;
        visuals.widgets.active.weak_bg_fill = active.tint;
        visuals.widgets.active.bg_stroke = Stroke::new(1.2, self.palette.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.5, self.palette.text_primary);
        visuals.widgets.active.corner_radius = widget_radius;

        ctx.set_visuals(visuals);

        ctx.all_styles_mut(|style| {
            style.spacing.item_spacing = egui::vec2(self.spacing.sm, self.spacing.sm);
            style.spacing.button_padding = egui::vec2(self.spacing.md, self.spacing.sm);
            style.spacing.window_margin = egui::Margin::same(self.spacing.md as i8);
            style.interaction.selectable_labels = true;
        });

        ctx.data_mut(|data| {
            data.insert_temp(Id::NULL.with(THEME_STORAGE_ID), self.clone());
        });
    }

    /// Reads the currently active [`PrismTheme`] from [`egui::Context`], falling back to
    /// [`PrismTheme::onyx()`] if [`PrismTheme::apply`] has not been called yet.
    pub fn from_ctx(ctx: &Context) -> Self {
        ctx.data(|data| data.get_temp::<Self>(Id::NULL.with(THEME_STORAGE_ID)))
            .unwrap_or_default()
    }
}
