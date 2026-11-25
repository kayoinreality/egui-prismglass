//! Modern Apple/Antigravity-style glassmorphic button widget ([`GlassButton`]).

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

        let font_id = FontId::proportional(13.0);
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

            let (fill_col, border_col, inner_edge, text_col) =
                resolve_button_colors(&theme, self.variant, hovered, pressed, self.selected);

            let alpha_mult = if self.enabled { 1.0 } else { 0.42 };

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
        GlassButtonVariant::Primary => match theme.mode {
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
                    Color32::from_rgb(47, 48, 52) // Antigravity --palette-grey-900
                } else {
                    Color32::from_rgb(18, 19, 23) // Antigravity --palette-grey-1200
                };
                (
                    fill,
                    Color32::from_rgba_unmultiplied(18, 19, 23, 255),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 22),
                    Color32::from_rgb(255, 255, 255),
                )
            }
        },
        GlassButtonVariant::Accent => {
            let fill = if pressed {
                Color32::from_rgb(30, 96, 214)
            } else if hovered {
                Color32::from_rgb(66, 133, 244)
            } else {
                theme.palette.accent // #3279F9
            };
            (
                fill,
                Color32::from_rgba_unmultiplied(255, 255, 255, 36),
                Color32::from_rgba_unmultiplied(255, 255, 255, 32),
                Color32::WHITE,
            )
        }
        GlassButtonVariant::Secondary => match (theme.surface_style, theme.mode) {
            (SurfaceStyle::Minimal, ThemeMode::Light) => {
                let fill = if selected {
                    Color32::from_rgb(239, 244, 255)
                } else if pressed {
                    Color32::from_rgb(236, 240, 246)
                } else if hovered {
                    Color32::from_rgb(245, 247, 251)
                } else {
                    Color32::WHITE
                };
                let border = if selected {
                    theme.palette.accent
                } else if hovered {
                    Color32::from_rgb(203, 210, 220)
                } else {
                    Color32::from_rgb(222, 227, 235)
                };
                (
                    fill,
                    border,
                    Color32::TRANSPARENT,
                    theme.palette.text_primary,
                )
            }
            (SurfaceStyle::Minimal, ThemeMode::Dark) => {
                let fill = if selected {
                    Color32::from_rgb(35, 42, 58)
                } else if pressed {
                    Color32::from_rgb(32, 34, 41)
                } else if hovered {
                    Color32::from_rgb(39, 41, 49)
                } else {
                    Color32::from_rgb(30, 32, 39)
                };
                let border = if selected {
                    theme.palette.accent
                } else if hovered {
                    Color32::from_rgb(66, 69, 80)
                } else {
                    Color32::from_rgb(48, 50, 59)
                };
                (
                    fill,
                    border,
                    Color32::TRANSPARENT,
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
