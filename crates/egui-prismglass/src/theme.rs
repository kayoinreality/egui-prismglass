//! Theme system and built-in presets for `egui-prismglass`.
//!
//! Built around a clean, architectural neutral palette inspired by Google Antigravity
//! (`#121317` Onyx, `#F8F9FC` Frost, `#3279F9` Electric Blue, cool neutral greys),
//! supporting two first-class visual styles through the same unified widget API:
//! - [`SurfaceStyle::Glass`]: Apple-style flat-translucent frosted glass over an atmospheric backdrop.
//! - [`SurfaceStyle::Minimal`]: Solid, ultra-clean flat surfaces (Linear / Vercel / Reduced Transparency).

use egui::{Color32, Context, CornerRadius, Id, Shadow, Stroke, Visuals};

use crate::tokens::{
    GlassElevation, GlassSurfaceTokens, PrismPalette, RadiusTokens, SpacingTokens,
};

const THEME_STORAGE_ID: &str = "egui_prismglass_active_theme";

/// Color polarity of a [`PrismTheme`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ThemeMode {
    /// Deep Onyx canvas (`#121317`).
    #[default]
    Dark,
    /// Airy Frost canvas (`#F8F9FC`).
    Light,
}

/// High-level surface rendering style for a [`PrismTheme`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SurfaceStyle {
    /// Translucent Apple-style frosted glass surfaces over an atmospheric backdrop.
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
    /// Antigravity **Onyx** dark glass (`#121317`) with neutral smoked glass and `#3279F9` accents.
    #[default]
    Onyx,
    /// Apple visionOS-inspired **Glacial** high-translucency glass over cool atmospheric depth.
    Glacial,
    /// Ultra-clean solid **Minimal Light** (`#FFFFFF` cards on `#F8F9FC`, zero translucency, hairline borders).
    MinimalLight,
    /// Ultra-clean solid **Minimal Dark** (`#18191D` cards on `#121317`, zero translucency, hairline borders).
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
    /// Global opacity multiplier (`0.3..=1.6`) applied in [`SurfaceStyle::Glass`] mode.
    pub glass_opacity_scale: f32,
    /// Global edge highlight multiplier (`0.0..=2.0`).
    pub specular_scale: f32,
    /// Global corner radius multiplier (`0.0..=1.6`).
    pub radius_scale: f32,
    /// Global refraction tint multiplier (`0.0..=1.5`).
    pub prism_scale: f32,
}

impl Default for PrismTheme {
    fn default() -> Self {
        Self::onyx()
    }
}

impl PrismTheme {
    /// Creates the **Onyx** dark 3D smoked-glass theme (Apple Pro / Raycast dark studio aesthetic).
    ///
    /// Deep carbon-black `#090A0D` canvas with sculpted monochrome titanium studio lighting
    /// (zero muddy gray, zero random blue wash), smoked crystal glass panes, directional 3D
    /// top-left bevel catch, and crisp grounded contact shadows.
    pub fn onyx() -> Self {
        let palette = PrismPalette {
            canvas: Color32::from_rgb(9, 10, 13), // #090A0D deep carbon black
            canvas_secondary: Color32::from_rgb(14, 15, 19), // #0E0F13
            orb_primary: Color32::from_rgb(138, 146, 166), // brushed titanium silver
            orb_secondary: Color32::from_rgb(186, 194, 210), // studio soft silver
            orb_accent: Color32::from_rgb(232, 236, 245), // crisp platinum highlight spine
            text_primary: Color32::from_rgb(248, 249, 252), // #F8F9FC
            text_secondary: Color32::from_rgb(166, 172, 184), // crisp cool silver
            text_muted: Color32::from_rgb(104, 109, 120),
            accent: Color32::from_rgb(50, 121, 249), // #3279F9 Antigravity blue
            accent_warm: Color32::from_rgb(138, 180, 248),
            success: Color32::from_rgb(52, 199, 123),
            warning: Color32::from_rgb(234, 179, 8),
            danger: Color32::from_rgb(239, 68, 68),
        };

        let radius = RadiusTokens {
            sm: 8.0,
            md: 10.0,
            lg: 14.0,
            xl: 18.0,
            pill: 999.0,
        };

        // Deep smoked dark crystal — low-luminance dark tint so glass feels sleek & 3D, never milky gray.
        let surface_subtle = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(15, 16, 21, 142),
            gradient_top: Color32::from_rgba_unmultiplied(24, 26, 33, 150),
            gradient_bottom: Color32::from_rgba_unmultiplied(11, 12, 16, 156),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 22),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 62),
            specular_accent: Color32::from_rgba_unmultiplied(220, 228, 245, 58),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 14),
            shadow: Shadow::NONE,
            corner_radius: radius.md,
            blur_radius: 18.0,
            border_width: 1.0,
            prism_intensity: 0.25,
        };

        let surface_card = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(15, 16, 21, 172),
            gradient_top: Color32::from_rgba_unmultiplied(26, 28, 36, 178),
            gradient_bottom: Color32::from_rgba_unmultiplied(10, 11, 15, 192),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 30),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 92),
            specular_accent: Color32::from_rgba_unmultiplied(226, 232, 248, 84),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            shadow: Shadow {
                offset: [0, 12],
                blur: 30,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 155),
            },
            corner_radius: radius.lg,
            blur_radius: 24.0,
            border_width: 1.0,
            prism_intensity: 0.35,
        };

        let surface_floating = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(18, 20, 26, 186),
            gradient_top: Color32::from_rgba_unmultiplied(32, 35, 45, 192),
            gradient_bottom: Color32::from_rgba_unmultiplied(12, 13, 18, 206),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 40),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 122),
            specular_accent: Color32::from_rgba_unmultiplied(235, 240, 252, 108),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 28),
            shadow: Shadow {
                offset: [0, 18],
                blur: 42,
                spread: 1,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 185),
            },
            corner_radius: radius.lg,
            blur_radius: 30.0,
            border_width: 1.0,
            prism_intensity: 0.45,
        };

        let surface_sidebar = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(13, 14, 18, 178),
            gradient_top: Color32::from_rgba_unmultiplied(22, 24, 30, 182),
            gradient_bottom: Color32::from_rgba_unmultiplied(9, 10, 13, 198),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 28),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 82),
            specular_accent: Color32::from_rgba_unmultiplied(220, 228, 244, 72),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            shadow: Shadow {
                offset: [0, 12],
                blur: 32,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 150),
            },
            corner_radius: radius.xl,
            blur_radius: 26.0,
            border_width: 1.0,
            prism_intensity: 0.30,
        };

        let surface_panel = GlassSurfaceTokens {
            tint: Color32::from_rgba_unmultiplied(14, 15, 20, 174),
            gradient_top: Color32::from_rgba_unmultiplied(24, 26, 33, 180),
            gradient_bottom: Color32::from_rgba_unmultiplied(10, 11, 15, 192),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 28),
            specular_top: Color32::from_rgba_unmultiplied(255, 255, 255, 84),
            specular_accent: Color32::from_rgba_unmultiplied(222, 228, 245, 74),
            inner_glow: Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            shadow: Shadow {
                offset: [0, 10],
                blur: 24,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 135),
            },
            corner_radius: radius.lg,
            blur_radius: 22.0,
            border_width: 1.0,
            prism_intensity: 0.30,
        };

        Self {
            name: "Onyx Dark".to_owned(),
            mode: ThemeMode::Dark,
            surface_style: SurfaceStyle::Glass,
            palette,
            spacing: SpacingTokens::default(),
            radius,
            surface_subtle,
            surface_card,
            surface_floating,
            surface_sidebar,
            surface_panel,
            glass_opacity_scale: 1.0,
            specular_scale: 1.0,
            radius_scale: 1.0,
            prism_scale: 1.0,
        }
    }

    /// Creates the airy **Frost Light** glass theme inspired by Google Antigravity + macOS glass.
    pub fn frost() -> Self {
        let palette = PrismPalette {
            canvas: Color32::from_rgb(248, 249, 252), // #F8F9FC
            canvas_secondary: Color32::from_rgb(235, 240, 248), // #EBF0F8
            orb_primary: Color32::from_rgb(50, 121, 249), // #3279F9
            orb_secondary: Color32::from_rgb(138, 180, 248), // #8AB4F8
            orb_accent: Color32::from_rgb(183, 191, 217), // #B7BFD9
            text_primary: Color32::from_rgb(18, 19, 23), // #121317
            text_secondary: Color32::from_rgb(69, 71, 77), // #45474D
            text_muted: Color32::from_rgb(106, 106, 113), // #6A6A71
            accent: Color32::from_rgb(50, 121, 249),  // #3279F9
            accent_warm: Color32::from_rgb(26, 115, 232), // #1A73E8
            success: Color32::from_rgb(24, 146, 92),
            warning: Color32::from_rgb(196, 124, 16),
            danger: Color32::from_rgb(217, 48, 37),
        };

        let radius = RadiusTokens {
            sm: 8.0,
            md: 10.0,
            lg: 14.0,
            xl: 18.0,
            pill: 999.0,
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
            corner_radius: radius.md,
            blur_radius: 16.0,
            border_width: 1.0,
            prism_intensity: 0.2,
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
                offset: [0, 4],
                blur: 16,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(18, 19, 23, 16),
            },
            corner_radius: radius.lg,
            blur_radius: 22.0,
            border_width: 1.0,
            prism_intensity: 0.25,
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
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(18, 19, 23, 22),
            },
            corner_radius: radius.lg,
            blur_radius: 28.0,
            border_width: 1.0,
            prism_intensity: 0.3,
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
                offset: [0, 4],
                blur: 18,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(18, 19, 23, 14),
            },
            corner_radius: radius.xl,
            blur_radius: 24.0,
            border_width: 1.0,
            prism_intensity: 0.22,
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
                offset: [0, 3],
                blur: 14,
                spread: 0,
                color: Color32::from_rgba_unmultiplied(18, 19, 23, 14),
            },
            corner_radius: radius.lg,
            blur_radius: 20.0,
            border_width: 1.0,
            prism_intensity: 0.22,
        };

        Self {
            name: "Frost Light".to_owned(),
            mode: ThemeMode::Light,
            surface_style: SurfaceStyle::Glass,
            palette,
            spacing: SpacingTokens::default(),
            radius,
            surface_subtle,
            surface_card,
            surface_floating,
            surface_sidebar,
            surface_panel,
            glass_opacity_scale: 1.0,
            specular_scale: 1.0,
            radius_scale: 1.0,
            prism_scale: 1.0,
        }
    }

    /// Creates the high-clarity **Glacial Glass** 3D crystal dark theme.
    ///
    /// Deep obsidian canvas with sculpted platinum-ice 3D light ribbons visibly refracting
    /// behind ultra-clean smoked crystal panes and crisp machined top-left glass bevels.
    pub fn glacial() -> Self {
        let mut theme = Self::onyx();
        "Glacial Glass".clone_into(&mut theme.name);
        theme.palette.canvas = Color32::from_rgb(7, 8, 12);
        theme.palette.canvas_secondary = Color32::from_rgb(13, 16, 24);
        theme.palette.orb_primary = Color32::from_rgb(122, 152, 214);
        theme.palette.orb_secondary = Color32::from_rgb(186, 206, 245);
        theme.palette.orb_accent = Color32::from_rgb(240, 245, 255);
        theme.palette.accent = Color32::from_rgb(96, 165, 250);

        // Crystal-clear dark smoked translucency (NOT milky white-gray) so 3D folds shine right through.
        theme.surface_subtle.tint = Color32::from_rgba_unmultiplied(16, 19, 27, 108);
        theme.surface_subtle.gradient_top = Color32::from_rgba_unmultiplied(28, 33, 46, 118);
        theme.surface_subtle.gradient_bottom = Color32::from_rgba_unmultiplied(10, 12, 18, 124);
        theme.surface_subtle.border = Color32::from_rgba_unmultiplied(255, 255, 255, 32);
        theme.surface_subtle.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 92);

        theme.surface_card.tint = Color32::from_rgba_unmultiplied(16, 20, 29, 126);
        theme.surface_card.gradient_top = Color32::from_rgba_unmultiplied(30, 36, 52, 136);
        theme.surface_card.gradient_bottom = Color32::from_rgba_unmultiplied(10, 12, 18, 148);
        theme.surface_card.border = Color32::from_rgba_unmultiplied(255, 255, 255, 42);
        theme.surface_card.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 132);
        theme.surface_card.inner_glow = Color32::from_rgba_unmultiplied(235, 244, 255, 28);

        theme.surface_floating.tint = Color32::from_rgba_unmultiplied(19, 24, 35, 144);
        theme.surface_floating.gradient_top = Color32::from_rgba_unmultiplied(36, 44, 64, 154);
        theme.surface_floating.gradient_bottom = Color32::from_rgba_unmultiplied(12, 15, 22, 166);
        theme.surface_floating.border = Color32::from_rgba_unmultiplied(255, 255, 255, 54);
        theme.surface_floating.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 158);
        theme.surface_floating.inner_glow = Color32::from_rgba_unmultiplied(240, 248, 255, 38);

        theme.surface_sidebar.tint = Color32::from_rgba_unmultiplied(13, 16, 24, 136);
        theme.surface_sidebar.gradient_top = Color32::from_rgba_unmultiplied(25, 30, 44, 144);
        theme.surface_sidebar.gradient_bottom = Color32::from_rgba_unmultiplied(9, 11, 17, 156);
        theme.surface_sidebar.border = Color32::from_rgba_unmultiplied(255, 255, 255, 38);
        theme.surface_sidebar.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 112);

        theme.surface_panel.tint = Color32::from_rgba_unmultiplied(15, 18, 26, 130);
        theme.surface_panel.gradient_top = Color32::from_rgba_unmultiplied(28, 33, 48, 138);
        theme.surface_panel.gradient_bottom = Color32::from_rgba_unmultiplied(10, 12, 18, 150);
        theme.surface_panel.border = Color32::from_rgba_unmultiplied(255, 255, 255, 40);
        theme.surface_panel.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 118);

        theme
    }

    /// Creates the solid **Minimal Light** theme (zero glassmorphism / Reduced Transparency).
    ///
    /// Designed for ultra-crisp desktop productivity apps in the style of Antigravity,
    /// Linear, and Vercel: pure `#FFFFFF` cards over `#F6F8FB` canvas, razor-sharp
    /// `#E2E7EE` hairline borders, `#121317` ink, and `#3279F9` blue accents.
    pub fn minimal_light() -> Self {
        let mut theme = Self::frost();
        "Minimal Light".clone_into(&mut theme.name);
        theme.surface_style = SurfaceStyle::Minimal;
        theme.palette.canvas = Color32::from_rgb(246, 248, 251); // #F6F8FB
        theme.palette.canvas_secondary = theme.palette.canvas;

        let border_subtle = Color32::from_rgb(232, 236, 242); // #E8ECF2
        let border_std = Color32::from_rgb(225, 230, 236); // #E1E6EC
        let border_strong = Color32::from_rgb(212, 218, 227); // #D4DAE3

        let solid =
            |fill: Color32, border: Color32, radius: f32, shadow: Shadow| GlassSurfaceTokens {
                tint: fill,
                gradient_top: fill,
                gradient_bottom: fill,
                border,
                specular_top: Color32::TRANSPARENT,
                specular_accent: Color32::TRANSPARENT,
                inner_glow: Color32::TRANSPARENT,
                shadow,
                corner_radius: radius,
                blur_radius: 0.0,
                border_width: 1.0,
                prism_intensity: 0.0,
            };

        let soft_shadow = Shadow {
            offset: [0, 1],
            blur: 3,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(18, 19, 23, 10),
        };
        let lift_shadow = Shadow {
            offset: [0, 4],
            blur: 14,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(18, 19, 23, 14),
        };

        theme.surface_subtle = solid(
            Color32::from_rgb(248, 249, 252),
            border_subtle,
            theme.radius.md,
            Shadow::NONE,
        );
        theme.surface_card = solid(Color32::WHITE, border_std, theme.radius.lg, soft_shadow);
        theme.surface_floating = solid(Color32::WHITE, border_strong, theme.radius.lg, lift_shadow);
        theme.surface_sidebar = solid(Color32::WHITE, border_std, theme.radius.lg, soft_shadow);
        theme.surface_panel = solid(Color32::WHITE, border_std, theme.radius.lg, soft_shadow);

        theme
    }

    /// Creates the solid **Minimal Dark** theme (zero glassmorphism / Reduced Transparency).
    ///
    /// Crisp `#18191D` surfaces over `#121317` Onyx canvas with `#272930` hairline borders,
    /// `#F8F9FC` typography, and `#3279F9` accents—ideal for high-focus desktop IDEs and tools.
    pub fn minimal_dark() -> Self {
        let mut theme = Self::onyx();
        "Minimal Dark".clone_into(&mut theme.name);
        theme.surface_style = SurfaceStyle::Minimal;
        theme.palette.canvas = Color32::from_rgb(18, 19, 23); // #121317
        theme.palette.canvas_secondary = theme.palette.canvas;

        let border_subtle = Color32::from_rgb(38, 40, 47); // #26282F
        let border_std = Color32::from_rgb(44, 46, 54); // #2C2E36
        let border_strong = Color32::from_rgb(56, 59, 69); // #383B45

        let solid =
            |fill: Color32, border: Color32, radius: f32, shadow: Shadow| GlassSurfaceTokens {
                tint: fill,
                gradient_top: fill,
                gradient_bottom: fill,
                border,
                specular_top: Color32::TRANSPARENT,
                specular_accent: Color32::TRANSPARENT,
                inner_glow: Color32::TRANSPARENT,
                shadow,
                corner_radius: radius,
                blur_radius: 0.0,
                border_width: 1.0,
                prism_intensity: 0.0,
            };

        let soft_shadow = Shadow {
            offset: [0, 2],
            blur: 6,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(0, 0, 0, 36),
        };
        let lift_shadow = Shadow {
            offset: [0, 6],
            blur: 18,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(0, 0, 0, 56),
        };

        theme.surface_subtle = solid(
            Color32::from_rgb(28, 30, 36),
            border_subtle,
            theme.radius.md,
            Shadow::NONE,
        );
        theme.surface_card = solid(
            Color32::from_rgb(24, 25, 30),
            border_std,
            theme.radius.lg,
            soft_shadow,
        );
        theme.surface_floating = solid(
            Color32::from_rgb(29, 31, 37),
            border_strong,
            theme.radius.lg,
            lift_shadow,
        );
        theme.surface_sidebar = solid(
            Color32::from_rgb(22, 23, 28),
            border_std,
            theme.radius.lg,
            soft_shadow,
        );
        theme.surface_panel = solid(
            Color32::from_rgb(24, 25, 30),
            border_std,
            theme.radius.lg,
            soft_shadow,
        );

        theme
    }

    /// Resolves effective [`GlassSurfaceTokens`] for a given [`GlassElevation`],
    /// applying the theme's live scale multipliers.
    pub fn surface(&self, elevation: GlassElevation) -> GlassSurfaceTokens {
        let base = match elevation {
            GlassElevation::Flat => {
                let mut t = self.surface_subtle;
                t.shadow = Shadow::NONE;
                t.prism_intensity *= 0.5;
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
                .with_prism_intensity(tokens.prism_intensity * self.prism_scale.clamp(0.0, 1.8)),
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
