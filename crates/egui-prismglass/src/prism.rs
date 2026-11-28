//! The Prism presets: Inter typography, a 12-step neutral ramp, one accent, hairline borders
//! and no bevels — in a [`SurfaceStyle::Layered`] (glass only where it floats) and a
//! [`SurfaceStyle::Frosted`] (everything glass) material.

use egui::{Color32, Context, CornerRadius, Shadow, Stroke, TextStyle, Visuals, vec2};

use crate::theme::{PrismTheme, SurfaceStyle, ThemeMode};
use crate::tokens::{
    AccentPalette, BackdropScene, GlassSurfaceTokens, PrismNeutrals, PrismPalette, RadiusTokens,
    SpacingTokens, blend_colors,
};
use crate::typography::{FontWeight, prism_font};

/// Opaque content surface: flat fill plus a hairline border, no optics.
fn solid(fill: Color32, border: Color32, radius: f32, shadow: Shadow) -> GlassSurfaceTokens {
    GlassSurfaceTokens {
        tint: fill,
        gradient_top: fill,
        gradient_bottom: fill,
        border,
        specular_top: Color32::TRANSPARENT,
        specular_accent: Color32::TRANSPARENT,
        inner_glow: Color32::TRANSPARENT,
        shadow,
        contact_shadow: Shadow::NONE,
        corner_radius: radius,
        blur_radius: 0.0,
        border_width: 1.0,
        prism_intensity: 0.0,
        vibrancy: 1.0,
        refraction_strength: 0.0,
        chromatic_dispersion: 0.0,
        grain_intensity: 0.0,
        blur_tint_alpha: 1.0,
    }
}

/// Frosted glass material: translucent film, hairline border and a 1px top highlight.
///
/// `tint` is the film used by the CPU fallback (no real blur, so it is denser); with real GPU
/// blur the film alpha is multiplied by `blur_tint_alpha` so the backdrop shows through.
struct GlassSpec {
    tint: Color32,
    border: Color32,
    highlight: Color32,
    radius: f32,
    blur: f32,
    blur_tint_alpha: f32,
    shadow: Shadow,
}

fn glass(spec: &GlassSpec) -> GlassSurfaceTokens {
    let [r, g, b, a] = spec.tint.to_srgba_unmultiplied();
    GlassSurfaceTokens {
        tint: spec.tint,
        // A barely-there top-to-bottom falloff keeps the fallback renderer on its glass path
        // (gradient body + rim) instead of the flat solid path.
        gradient_top: Color32::from_rgba_unmultiplied(r, g, b, a.saturating_sub(6)),
        gradient_bottom: Color32::from_rgba_unmultiplied(r, g, b, a.saturating_add(6)),
        border: spec.border,
        specular_top: spec.border,
        specular_accent: Color32::TRANSPARENT,
        inner_glow: spec.highlight,
        shadow: spec.shadow,
        contact_shadow: Shadow::NONE,
        corner_radius: spec.radius,
        blur_radius: spec.blur,
        border_width: 1.0,
        prism_intensity: 0.0,
        vibrancy: 1.3,
        refraction_strength: 0.0,
        chromatic_dispersion: 0.0,
        grain_intensity: 0.0,
        blur_tint_alpha: spec.blur_tint_alpha,
    }
}

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied_const(r, g, b, a)
}

fn shadow(offset_y: i8, blur: u8, color: Color32) -> Shadow {
    Shadow {
        offset: [0, offset_y],
        blur,
        spread: 0,
        color,
    }
}

impl PrismTheme {
    /// **Prism Dark**: solid, flat content with frosted glass only on floating layers.
    pub fn prism_dark() -> Self {
        Self::prism(ThemeMode::Dark, SurfaceStyle::Layered)
    }

    /// **Prism Light**: solid, flat content with frosted glass only on floating layers.
    pub fn prism_light() -> Self {
        Self::prism(ThemeMode::Light, SurfaceStyle::Layered)
    }

    /// **Prism Glass Dark**: the Prism language with every container made of frosted glass.
    pub fn prism_glass_dark() -> Self {
        Self::prism(ThemeMode::Dark, SurfaceStyle::Frosted)
    }

    /// **Prism Glass Light**: the Prism language with every container made of frosted glass.
    pub fn prism_glass_light() -> Self {
        Self::prism(ThemeMode::Light, SurfaceStyle::Frosted)
    }

    /// Builds a Prism theme for any polarity and material.
    ///
    /// `style` must be [`SurfaceStyle::Layered`] or [`SurfaceStyle::Frosted`]; any other value
    /// is treated as [`SurfaceStyle::Layered`].
    pub fn prism(mode: ThemeMode, style: SurfaceStyle) -> Self {
        let style = if style.is_prism() {
            style
        } else {
            SurfaceStyle::Layered
        };
        let dark = mode == ThemeMode::Dark;
        let accent = AccentPalette::Cobalt;

        let palette = if dark {
            PrismPalette {
                canvas: Color32::from_rgb(7, 7, 12),
                canvas_secondary: Color32::from_rgb(219, 39, 119),
                orb_primary: Color32::from_rgb(124, 58, 237),
                orb_secondary: Color32::from_rgb(37, 99, 235),
                orb_accent: Color32::from_rgb(13, 148, 136),
                text_primary: Color32::from_rgb(237, 237, 240),
                text_secondary: Color32::from_rgb(161, 161, 170),
                text_muted: Color32::from_rgb(138, 138, 148),
                accent: accent.prism(true),
                accent_warm: accent.companion(true),
                success: Color32::from_rgb(61, 214, 140),
                warning: Color32::from_rgb(245, 185, 70),
                danger: Color32::from_rgb(255, 99, 105),
            }
        } else {
            PrismPalette {
                canvas: Color32::from_rgb(232, 235, 244),
                canvas_secondary: Color32::from_rgb(244, 114, 182),
                orb_primary: Color32::from_rgb(167, 139, 250),
                orb_secondary: Color32::from_rgb(96, 165, 250),
                orb_accent: Color32::from_rgb(52, 211, 153),
                text_primary: Color32::from_rgb(24, 24, 27),
                text_secondary: Color32::from_rgb(82, 82, 91),
                text_muted: Color32::from_rgb(107, 107, 118),
                accent: accent.prism(false),
                accent_warm: accent.companion(false),
                success: Color32::from_rgb(24, 160, 88),
                warning: Color32::from_rgb(194, 122, 0),
                danger: Color32::from_rgb(220, 38, 38),
            }
        };
        let neutrals = if dark {
            PrismNeutrals::DARK
        } else {
            PrismNeutrals::LIGHT
        };
        let radius = RadiusTokens {
            sm: 6.0,
            md: 8.0,
            lg: 12.0,
            xl: 14.0,
            pill: 999.0,
        };
        let spacing = SpacingTokens {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
            xl: 24.0,
        };

        let float_shadow = if dark {
            shadow(16, 40, rgba(0, 0, 0, 115))
        } else {
            shadow(16, 40, rgba(40, 45, 80, 46))
        };
        let panel_shadow = if dark {
            shadow(8, 24, rgba(0, 0, 0, 70))
        } else {
            shadow(8, 24, rgba(40, 45, 80, 26))
        };

        // Floating layers are glass in both materials.
        let (sidebar, floating) = if dark {
            (
                glass(&GlassSpec {
                    tint: rgba(22, 22, 30, 200),
                    border: rgba(255, 255, 255, 23),
                    highlight: rgba(255, 255, 255, 18),
                    radius: radius.lg,
                    blur: 28.0,
                    blur_tint_alpha: 0.76,
                    shadow: Shadow::NONE,
                }),
                glass(&GlassSpec {
                    tint: rgba(30, 30, 40, 225),
                    border: rgba(255, 255, 255, 26),
                    highlight: rgba(255, 255, 255, 18),
                    radius: radius.lg,
                    blur: 24.0,
                    blur_tint_alpha: 0.70,
                    shadow: float_shadow,
                }),
            )
        } else {
            (
                glass(&GlassSpec {
                    tint: rgba(255, 255, 255, 178),
                    border: rgba(0, 0, 0, 18),
                    highlight: rgba(255, 255, 255, 200),
                    radius: radius.lg,
                    blur: 28.0,
                    blur_tint_alpha: 0.75,
                    shadow: Shadow::NONE,
                }),
                glass(&GlassSpec {
                    tint: rgba(255, 255, 255, 220),
                    border: rgba(0, 0, 0, 16),
                    highlight: rgba(255, 255, 255, 230),
                    radius: radius.lg,
                    blur: 24.0,
                    blur_tint_alpha: 0.82,
                    shadow: float_shadow,
                }),
            )
        };

        let (subtle, card, panel) = match (style, dark) {
            (SurfaceStyle::Frosted, true) => (
                solid(
                    rgba(255, 255, 255, 12),
                    rgba(255, 255, 255, 18),
                    radius.lg,
                    Shadow::NONE,
                ),
                glass(&GlassSpec {
                    tint: rgba(26, 26, 36, 185),
                    border: rgba(255, 255, 255, 23),
                    highlight: rgba(255, 255, 255, 16),
                    radius: radius.lg,
                    blur: 24.0,
                    blur_tint_alpha: 0.62,
                    shadow: Shadow::NONE,
                }),
                glass(&GlassSpec {
                    tint: rgba(18, 18, 26, 195),
                    border: rgba(255, 255, 255, 23),
                    highlight: rgba(255, 255, 255, 16),
                    radius: radius.xl,
                    blur: 32.0,
                    blur_tint_alpha: 0.60,
                    shadow: panel_shadow,
                }),
            ),
            (SurfaceStyle::Frosted, false) => (
                solid(
                    rgba(255, 255, 255, 110),
                    rgba(0, 0, 0, 14),
                    radius.lg,
                    Shadow::NONE,
                ),
                glass(&GlassSpec {
                    tint: rgba(255, 255, 255, 190),
                    border: rgba(0, 0, 0, 16),
                    highlight: rgba(255, 255, 255, 210),
                    radius: radius.lg,
                    blur: 24.0,
                    blur_tint_alpha: 0.72,
                    shadow: Shadow::NONE,
                }),
                glass(&GlassSpec {
                    tint: rgba(255, 255, 255, 170),
                    border: rgba(0, 0, 0, 16),
                    highlight: rgba(255, 255, 255, 210),
                    radius: radius.xl,
                    blur: 32.0,
                    blur_tint_alpha: 0.70,
                    shadow: panel_shadow,
                }),
            ),
            (_, _) => (
                solid(neutrals.raised, neutrals.border, radius.lg, Shadow::NONE),
                solid(neutrals.raised, neutrals.border, radius.lg, Shadow::NONE),
                solid(neutrals.surface, neutrals.border, radius.lg, Shadow::NONE),
            ),
        };

        let name = match (style, dark) {
            (SurfaceStyle::Frosted, true) => "Prism Glass Dark",
            (SurfaceStyle::Frosted, false) => "Prism Glass Light",
            (_, true) => "Prism Dark",
            (_, false) => "Prism Light",
        };

        Self {
            name: name.to_owned(),
            mode,
            surface_style: style,
            backdrop_scene: BackdropScene::Aurora,
            accent_palette: accent,
            palette,
            neutrals,
            spacing,
            radius,
            surface_subtle: subtle,
            surface_card: card,
            surface_floating: floating,
            surface_sidebar: sidebar,
            surface_panel: panel,
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

    /// Returns a copy of this Prism theme switched to another material
    /// ([`SurfaceStyle::Layered`] or [`SurfaceStyle::Frosted`]), keeping polarity, accent,
    /// backdrop scene and blur setting. Classic themes are returned unchanged.
    #[must_use]
    pub fn with_material(self, style: SurfaceStyle) -> Self {
        if !self.surface_style.is_prism() || !style.is_prism() {
            return self;
        }
        let mut next = Self::prism(self.mode, style);
        next.backdrop_scene = self.backdrop_scene;
        next.real_blur_enabled = self.real_blur_enabled;
        next.set_accent_palette(self.accent_palette);
        next
    }

    /// Color for text and icons drawn on top of a solid accent fill.
    pub fn on_accent(&self) -> Color32 {
        let [r, g, b, _] = self.palette.accent.to_srgba_unmultiplied();
        let luma = 0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b);
        if luma > 160.0 {
            Color32::from_rgb(17, 17, 20)
        } else {
            Color32::WHITE
        }
    }

    /// Configures `egui` visuals, spacing and the type scale for the Prism presets.
    pub(crate) fn apply_prism_style(&self, ctx: &Context) {
        let n = &self.neutrals;
        let p = &self.palette;
        let dark = self.mode == ThemeMode::Dark;
        let mut visuals = if dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        let radius = CornerRadius::same(self.radius.md as u8);
        let window_fill = if dark {
            Color32::from_rgb(28, 28, 34)
        } else {
            Color32::WHITE
        };

        visuals.override_text_color = Some(p.text_primary);
        visuals.panel_fill = n.surface;
        visuals.window_fill = window_fill;
        visuals.window_stroke = Stroke::new(1.0, n.border_strong);
        visuals.window_shadow = self.surface_floating.shadow;
        visuals.window_corner_radius = CornerRadius::same(self.radius.lg as u8);
        visuals.menu_corner_radius = CornerRadius::same(self.radius.md as u8);
        visuals.popup_shadow = self.surface_floating.shadow;
        visuals.faint_bg_color = n.raised;
        visuals.extreme_bg_color = n.raised;
        visuals.code_bg_color = n.raised;
        visuals.hyperlink_color = p.accent;
        visuals.warn_fg_color = p.warning;
        visuals.error_fg_color = p.danger;
        let [ar, ag, ab, _] = p.accent.to_srgba_unmultiplied();
        visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(ar, ag, ab, 72);
        visuals.selection.stroke = Stroke::new(1.0, p.accent);

        let hover_fill = blend_colors(n.raised, p.text_primary, 0.05);
        let widgets = &mut visuals.widgets;
        widgets.noninteractive.bg_fill = n.surface;
        widgets.noninteractive.weak_bg_fill = n.surface;
        widgets.noninteractive.bg_stroke = Stroke::new(1.0, n.border);
        widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text_secondary);
        widgets.noninteractive.corner_radius = radius;

        widgets.inactive.bg_fill = n.raised;
        widgets.inactive.weak_bg_fill = n.raised;
        widgets.inactive.bg_stroke = Stroke::new(1.0, n.border_strong);
        widgets.inactive.fg_stroke = Stroke::new(1.0, p.text_primary);
        widgets.inactive.corner_radius = radius;
        widgets.inactive.expansion = 0.0;

        widgets.hovered.bg_fill = hover_fill;
        widgets.hovered.weak_bg_fill = hover_fill;
        widgets.hovered.bg_stroke = Stroke::new(1.0, n.border_strong);
        widgets.hovered.fg_stroke = Stroke::new(1.0, p.text_primary);
        widgets.hovered.corner_radius = radius;
        widgets.hovered.expansion = 0.0;

        widgets.active.bg_fill = n.sunken;
        widgets.active.weak_bg_fill = n.sunken;
        widgets.active.bg_stroke = Stroke::new(1.0, p.accent);
        widgets.active.fg_stroke = Stroke::new(1.0, p.text_primary);
        widgets.active.corner_radius = radius;
        widgets.active.expansion = 0.0;

        widgets.open = widgets.hovered;

        ctx.set_visuals(visuals);

        let body = prism_font(ctx, 13.0, FontWeight::Regular);
        let button = prism_font(ctx, 13.0, FontWeight::Medium);
        let heading = prism_font(ctx, 20.0, FontWeight::Semibold);
        let small = prism_font(ctx, 12.0, FontWeight::Regular);
        let spacing = self.spacing;
        ctx.all_styles_mut(|style| {
            style.text_styles.insert(TextStyle::Body, body.clone());
            style.text_styles.insert(TextStyle::Button, button.clone());
            style
                .text_styles
                .insert(TextStyle::Heading, heading.clone());
            style.text_styles.insert(TextStyle::Small, small.clone());
            style
                .text_styles
                .insert(TextStyle::Monospace, egui::FontId::monospace(12.0));
            style.spacing.item_spacing = vec2(spacing.sm, spacing.sm);
            style.spacing.button_padding = vec2(spacing.md, 6.0);
            style.spacing.interact_size.y = 28.0;
            style.spacing.window_margin = egui::Margin::same(spacing.md as i8);
            style.spacing.menu_margin = egui::Margin::same(4);
            style.interaction.selectable_labels = true;
        });
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // exact values produced by clamps and constants
mod tests {
    use super::*;
    use crate::tokens::GlassElevation;

    #[test]
    fn prism_presets_have_expected_materials() {
        let layered = PrismTheme::prism_dark();
        assert_eq!(layered.surface_style, SurfaceStyle::Layered);
        assert_eq!(layered.backdrop_scene, BackdropScene::Aurora);
        // Content is solid in the Layered material, floating layers are glass.
        assert_eq!(layered.surface(GlassElevation::Elevated).blur_radius, 0.0);
        assert_eq!(layered.panel_surface().tint.a(), 255);
        assert!(layered.sidebar_surface().blur_radius > 0.0);
        assert!(layered.surface(GlassElevation::Floating).blur_radius > 0.0);

        // Everything that contains content is glass in the Frosted material.
        let frosted = PrismTheme::prism_glass_light();
        assert_eq!(frosted.surface_style, SurfaceStyle::Frosted);
        assert!(frosted.surface(GlassElevation::Elevated).blur_radius > 0.0);
        assert!(frosted.panel_surface().blur_radius > 0.0);
        assert!(frosted.panel_surface().tint.a() < 255);
    }

    #[test]
    fn prism_accents_keep_white_text_legible() {
        fn lin(c: u8) -> f32 {
            let c = f32::from(c) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        for palette in AccentPalette::ALL {
            for dark in [false, true] {
                let theme = PrismTheme::prism(
                    if dark {
                        ThemeMode::Dark
                    } else {
                        ThemeMode::Light
                    },
                    SurfaceStyle::Layered,
                )
                .with_accent_palette(palette);
                let [r, g, b, _] = theme.palette.accent.to_srgba_unmultiplied();
                let l = 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
                let [tr, tg, tb, _] = theme.on_accent().to_srgba_unmultiplied();
                let lt = 0.2126 * lin(tr) + 0.7152 * lin(tg) + 0.0722 * lin(tb);
                let contrast = (l.max(lt) + 0.05) / (l.min(lt) + 0.05);
                assert!(
                    contrast >= 4.5,
                    "{palette:?} dark={dark}: contrast {contrast:.2}"
                );
            }
        }
    }

    #[test]
    fn with_material_switches_between_layered_and_frosted() {
        let theme = PrismTheme::prism_dark()
            .with_accent_palette(AccentPalette::Emerald)
            .with_material(SurfaceStyle::Frosted);
        assert_eq!(theme.surface_style, SurfaceStyle::Frosted);
        assert_eq!(theme.mode, ThemeMode::Dark);
        assert_eq!(theme.accent_palette, AccentPalette::Emerald);
        assert_eq!(theme.palette.accent, AccentPalette::Emerald.prism(true));

        let classic = PrismTheme::onyx().with_material(SurfaceStyle::Frosted);
        assert_eq!(classic.surface_style, SurfaceStyle::Glass);
    }

    #[test]
    fn applying_prism_then_classic_restores_text_styles() {
        let ctx = Context::default();
        let default_body = ctx.style_of(egui::Theme::Dark).text_styles[&TextStyle::Body].clone();
        PrismTheme::prism_dark().apply(&ctx);
        assert_eq!(
            ctx.style_of(egui::Theme::Dark).text_styles[&TextStyle::Body].size,
            13.0
        );
        PrismTheme::onyx().apply(&ctx);
        assert_eq!(
            ctx.style_of(egui::Theme::Dark).text_styles[&TextStyle::Body],
            default_body
        );
    }
}
