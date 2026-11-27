//! Modern Apple/Antigravity-style glassmorphic button widget ([`GlassButton`]).

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use egui::{
    Align2, Color32, FontId, Response, Sense, Stroke, Ui, Vec2, Widget, WidgetText, pos2, vec2,
};

use crate::painter::paint_glass_gradient_fill;
use crate::theme::{PrismTheme, SurfaceStyle, ThemeMode};
use crate::tokens::scale_alpha;

/// Visual prominence variant for [`GlassButton`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlassButtonVariant {
    /// High-contrast solid monochrome CTA (pure white on dark, `#121317` onyx on light).
    Primary,
    /// Apple-style translucent frosted glass button with a crisp 1px hairline border.
    #[default]
    Secondary,
    /// Signature Antigravity electric blue (`#3279F9`) CTA button.
    Accent,
    /// Minimal transparent button that reveals a subtle frosted glass surface on hover.
    Ghost,
}

/// A clean, modern frosted-glass button with crisp hairline borders and refined variants.
///
/// Implements [`egui::Widget`], so it can be added directly with `ui.add(GlassButton::primary("Launch"))`.
#[derive(Clone)]
pub struct GlassButton {
    text: WidgetText,
    icon: Option<String>,
    variant: GlassButtonVariant,
    pill: bool,
    min_size: Vec2,
    enabled: bool,
    selected: bool,
    custom_tint: Option<Color32>,
    font_size: f32,
}

impl GlassButton {
    /// Creates a [`GlassButtonVariant::Secondary`] frosted glass button.
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            variant: GlassButtonVariant::Secondary,
            pill: false,
            min_size: vec2(0.0, 34.0),
            enabled: true,
            selected: false,
            custom_tint: None,
            font_size: 13.0,
        }
    }

    /// Creates a [`GlassButtonVariant::Primary`] high-contrast Antigravity CTA button.
    pub fn primary(text: impl Into<WidgetText>) -> Self {
        Self::new(text).variant(GlassButtonVariant::Primary)
    }

    /// Creates a [`GlassButtonVariant::Secondary`] Apple frosted glass button.
    pub fn secondary(text: impl Into<WidgetText>) -> Self {
        Self::new(text).variant(GlassButtonVariant::Secondary)
    }

    /// Creates a [`GlassButtonVariant::Accent`] Antigravity blue (`#3279F9`) CTA button.
    pub fn accent(text: impl Into<WidgetText>) -> Self {
        Self::new(text).variant(GlassButtonVariant::Accent)
    }

    /// Creates a [`GlassButtonVariant::Ghost`] minimal hover-glass button.
    pub fn ghost(text: impl Into<WidgetText>) -> Self {
        Self::new(text).variant(GlassButtonVariant::Ghost)
    }

    /// Sets the visual variant.
    pub fn variant(mut self, variant: GlassButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Adds a leading glyph or icon string rendered before the button label.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Enables pill-shaped rounding (`corner_radius = height / 2`).
    pub fn pill(mut self, pill: bool) -> Self {
        self.pill = pill;
        self
    }

    /// Sets a minimum button size.
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Overrides the label font size in logical points (defaults to `13.0`).
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size.max(8.0);
        self
    }

    /// Tints the button as a vibrant colored Liquid Glass pill (e.g. ruby, cobalt, amber, violet).
    pub fn tint(mut self, tint: Color32) -> Self {
        self.custom_tint = Some(tint);
        self
    }

    /// Marks the button as visually selected/active (useful for segmented controls).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Renders the button inside `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for GlassButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let padding = vec2(16.0, 8.0);

        let label_text = match &self.icon {
            Some(ic) => format!("{ic}  {}", self.text.text()),
            None => self.text.text().to_owned(),
        };

        let font_id = FontId::proportional(self.font_size);
        let galley =
            ui.painter()
                .layout_no_wrap(label_text, font_id.clone(), theme.palette.text_primary);

        let desired = vec2(
            (galley.size().x + padding.x * 2.0).max(self.min_size.x),
            (galley.size().y + padding.y * 2.0).max(self.min_size.y),
        );

        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(desired, sense);

        if self.enabled && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        if ui.is_rect_visible(rect) {
            let hovered = self.enabled && response.hovered();
            let pressed = self.enabled && response.is_pointer_button_down_on();
            let corner_radius = if self.pill {
                rect.height() * 0.5
            } else {
                theme.radius.md * theme.radius_scale.clamp(0.4, 1.5)
            };
            let alpha_mult = if self.enabled { 1.0 } else { 0.42 };

            if let Some(custom_tint) = self.custom_tint {
                let mut tokens = theme.surface(crate::tokens::GlassElevation::Floating);
                let [r, g, b, _] = custom_tint.to_srgba_unmultiplied();
                let top_alpha = if pressed {
                    210
                } else if hovered {
                    188
                } else {
                    168
                };
                let bot_alpha = if pressed { 226 } else { 196 };
                tokens.tint = Color32::from_rgba_unmultiplied(r, g, b, top_alpha);
                tokens.gradient_top = Color32::from_rgba_unmultiplied(
                    r.saturating_add(18),
                    g.saturating_add(18),
                    b.saturating_add(18),
                    top_alpha,
                );
                tokens.gradient_bottom = Color32::from_rgba_unmultiplied(
                    (f32::from(r) * 0.65) as u8,
                    (f32::from(g) * 0.65) as u8,
                    (f32::from(b) * 0.65) as u8,
                    bot_alpha,
                );
                tokens.corner_radius = corner_radius;
                tokens.blur_radius = 4.0;
                tokens.refraction_strength = 1.0;
                tokens.chromatic_dispersion = 0.52;
                tokens.blur_tint_alpha = 0.60;
                tokens.specular_top = Color32::from_rgba_unmultiplied(255, 255, 255, 245);
                tokens.specular_accent = Color32::from_rgba_unmultiplied(
                    r.saturating_add(110),
                    g.saturating_add(110),
                    b.saturating_add(110),
                    195,
                );
                tokens.border = Color32::from_rgba_unmultiplied(255, 255, 255, 92);
                tokens.grain_intensity = 0.004;

                let btn_id = ui.make_persistent_id((
                    "prism_tinted_btn",
                    rect.min.x as i32,
                    rect.min.y as i32,
                ));
                let shapes =
                    crate::blur::resolve_glass_surface_shapes(ui, btn_id, rect, &tokens, &theme);
                ui.painter().add(egui::Shape::Vec(shapes));

                let text_pos = pos2(
                    rect.center().x,
                    rect.center().y + if pressed { 0.5 } else { 0.0 },
                );
                ui.painter().text(
                    text_pos,
                    Align2::CENTER_CENTER,
                    galley.text(),
                    font_id,
                    scale_alpha(Color32::WHITE, alpha_mult),
                );
            } else {
                let (fill_col, border_col, inner_edge, text_col) =
                    resolve_button_colors(&theme, self.variant, hovered, pressed, self.selected);

                // In Minimal (Neumorphic) mode, extrude Secondary/Primary/Accent buttons with tactile soft shadows!
                if theme.surface_style == SurfaceStyle::Minimal && fill_col.a() > 0 && !pressed {
                    let cr =
                        egui::CornerRadius::same(corner_radius.round().clamp(0.0, 255.0) as u8);
                    match self.variant {
                        GlassButtonVariant::Accent | GlassButtonVariant::Primary => {
                            let glow_color = if self.variant == GlassButtonVariant::Accent
                                || theme.accent_palette != crate::tokens::AccentPalette::Graphite
                            {
                                scale_alpha(theme.palette.accent, 0.34 * alpha_mult)
                            } else {
                                Color32::from_rgba_unmultiplied(0, 0, 0, 48)
                            };
                            let drop = egui::Shadow {
                                offset: [0, 4],
                                blur: 12,
                                spread: 0,
                                color: glow_color,
                            };
                            ui.painter().add(egui::Shape::from(drop.as_shape(rect, cr)));
                        }
                        GlassButtonVariant::Secondary if !self.selected => {
                            let (drop_col, light_col) = match theme.mode {
                                ThemeMode::Light => (
                                    Color32::from_rgba_unmultiplied(150, 166, 194, 96),
                                    Color32::from_rgba_unmultiplied(255, 255, 255, 220),
                                ),
                                ThemeMode::Dark => (
                                    Color32::from_rgba_unmultiplied(5, 7, 13, 170),
                                    Color32::from_rgba_unmultiplied(160, 180, 224, 18),
                                ),
                            };
                            let light_s = egui::Shadow {
                                offset: [-3, -3],
                                blur: 8,
                                spread: 0,
                                color: light_col,
                            };
                            let drop_s = egui::Shadow {
                                offset: [3, 4],
                                blur: 10,
                                spread: 0,
                                color: drop_col,
                            };
                            ui.painter()
                                .add(egui::Shape::from(drop_s.as_shape(rect, cr)));
                            ui.painter()
                                .add(egui::Shape::from(light_s.as_shape(rect, cr)));
                        }
                        _ => {}
                    }
                }

                if fill_col.a() > 0 || border_col.a() > 0 {
                    paint_glass_gradient_fill(
                        ui.painter(),
                        rect,
                        corner_radius,
                        scale_alpha(fill_col, alpha_mult),
                        scale_alpha(fill_col, alpha_mult),
                        Stroke::new(1.0, scale_alpha(border_col, alpha_mult)),
                        scale_alpha(inner_edge, alpha_mult),
                    );
                }

                let text_pos = pos2(
                    rect.center().x,
                    rect.center().y + if pressed { 0.5 } else { 0.0 },
                );
                ui.painter().text(
                    text_pos,
                    Align2::CENTER_CENTER,
                    galley.text(),
                    font_id,
                    scale_alpha(text_col, alpha_mult),
                );
            }
        }

        response
    }
}

fn resolve_button_colors(
    theme: &PrismTheme,
    variant: GlassButtonVariant,
    hovered: bool,
    pressed: bool,
    selected: bool,
) -> (Color32, Color32, Color32, Color32) {
    match variant {
        GlassButtonVariant::Primary => {
            if theme.surface_style == SurfaceStyle::Minimal
                && theme.accent_palette != crate::tokens::AccentPalette::Graphite
            {
                let base = theme.palette.accent;
                let fill = if pressed {
                    crate::tokens::blend_colors(base, Color32::BLACK, 0.16)
                } else if hovered {
                    crate::tokens::blend_colors(base, Color32::WHITE, 0.12)
                } else {
                    base
                };
                return (
                    fill,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 42),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 58),
                    Color32::WHITE,
                );
            }
            match theme.mode {
                ThemeMode::Dark => {
                    let fill = if pressed {
                        Color32::from_rgb(218, 222, 232)
                    } else if hovered {
                        Color32::from_rgb(232, 236, 244)
                    } else {
                        Color32::from_rgb(248, 249, 252)
                    };
                    (
                        fill,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 40),
                        Color32::TRANSPARENT,
                        Color32::from_rgb(18, 19, 23),
                    )
                }
                ThemeMode::Light => {
                    let fill = if pressed {
                        Color32::from_rgb(56, 58, 64)
                    } else if hovered {
                        Color32::from_rgb(47, 48, 52)
                    } else {
                        Color32::from_rgb(18, 19, 23)
                    };
                    (
                        fill,
                        Color32::from_rgba_unmultiplied(18, 19, 23, 255),
                        Color32::from_rgba_unmultiplied(255, 255, 255, 22),
                        Color32::from_rgb(255, 255, 255),
                    )
                }
            }
        }
        GlassButtonVariant::Accent => {
            let base = theme.palette.accent;
            let fill = if pressed {
                crate::tokens::blend_colors(base, Color32::BLACK, 0.16)
            } else if hovered {
                crate::tokens::blend_colors(base, Color32::WHITE, 0.12)
            } else {
                base
            };
            let text_col = if theme.accent_palette == crate::tokens::AccentPalette::Graphite
                && theme.mode == ThemeMode::Dark
            {
                Color32::from_rgb(18, 20, 26)
            } else {
                Color32::WHITE
            };
            (
                fill,
                Color32::from_rgba_unmultiplied(255, 255, 255, 44),
                Color32::from_rgba_unmultiplied(255, 255, 255, 48),
                text_col,
            )
        }
        GlassButtonVariant::Secondary => match (theme.surface_style, theme.mode) {
            (SurfaceStyle::Minimal, ThemeMode::Light) => {
                let fill = if selected {
                    crate::tokens::blend_colors(
                        Color32::from_rgb(244, 247, 252),
                        theme.palette.accent,
                        0.12,
                    )
                } else if pressed {
                    Color32::from_rgb(225, 231, 241)
                } else if hovered {
                    Color32::from_rgb(248, 250, 254)
                } else {
                    Color32::from_rgb(241, 245, 251)
                };
                let border = if selected {
                    theme.palette.accent
                } else if hovered {
                    Color32::from_rgb(196, 208, 226)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 215)
                };
                let text = if selected {
                    theme.palette.accent
                } else {
                    theme.palette.text_primary
                };
                (
                    fill,
                    border,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 180),
                    text,
                )
            }
            (SurfaceStyle::Minimal, ThemeMode::Dark) => {
                let fill = if selected {
                    crate::tokens::blend_colors(
                        Color32::from_rgb(30, 35, 47),
                        theme.palette.accent,
                        0.18,
                    )
                } else if pressed {
                    Color32::from_rgb(22, 26, 36)
                } else if hovered {
                    Color32::from_rgb(38, 44, 59)
                } else {
                    Color32::from_rgb(32, 37, 50)
                };
                let border = if selected {
                    theme.palette.accent
                } else if hovered {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 38)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 22)
                };
                (
                    fill,
                    border,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 16),
                    theme.palette.text_primary,
                )
            }
            (SurfaceStyle::Glass, ThemeMode::Dark) => {
                let fill = if selected {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 34)
                } else if pressed {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 16)
                } else if hovered {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 28)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 18)
                };
                let border = if selected {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 76)
                } else if hovered {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 52)
                } else {
                    Color32::from_rgba_unmultiplied(230, 234, 240, 32)
                };
                let inner = Color32::from_rgba_unmultiplied(255, 255, 255, 14);
                (fill, border, inner, theme.palette.text_primary)
            }
            (SurfaceStyle::Glass, ThemeMode::Light) => {
                let fill = if selected {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 235)
                } else if pressed {
                    Color32::from_rgba_unmultiplied(238, 241, 247, 210)
                } else if hovered {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 224)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 175)
                };
                let border = if selected {
                    Color32::from_rgba_unmultiplied(50, 121, 249, 130)
                } else if hovered {
                    Color32::from_rgba_unmultiplied(18, 19, 23, 34)
                } else {
                    Color32::from_rgba_unmultiplied(18, 19, 23, 22)
                };
                let inner = Color32::from_rgba_unmultiplied(255, 255, 255, 190);
                (fill, border, inner, theme.palette.text_primary)
            }
        },
        GlassButtonVariant::Ghost => {
            if selected || hovered || pressed {
                let fill = match theme.mode {
                    ThemeMode::Dark => Color32::from_rgba_unmultiplied(255, 255, 255, 16),
                    ThemeMode::Light => Color32::from_rgba_unmultiplied(18, 19, 23, 12),
                };
                (
                    fill,
                    Color32::TRANSPARENT,
                    Color32::TRANSPARENT,
                    theme.palette.text_primary,
                )
            } else {
                (
                    Color32::TRANSPARENT,
                    Color32::TRANSPARENT,
                    Color32::TRANSPARENT,
                    theme.palette.text_secondary,
                )
            }
        }
    }
}
