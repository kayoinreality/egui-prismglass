//! Pill-shaped translucent status badge widget ([`GlassBadge`]).

use egui::{
    Align2, Color32, CornerRadius, FontId, Response, Sense, Stroke, StrokeKind, Ui, Widget, pos2,
    vec2,
};

use crate::theme::{PrismTheme, ThemeMode};
use crate::tokens::scale_alpha;

/// Semantic color style for [`GlassBadge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BadgeVariant {
    /// Subtle frosted glass neutral chip.
    #[default]
    Neutral,
    /// Primary prism periwinkle / indigo accent chip.
    Accent,
    /// Warm solar-orange (`#EC630D`) accent chip.
    Warm,
    /// Semantic success chip.
    Success,
    /// Semantic warning chip.
    Warning,
}

/// A compact translucent pill badge with optional glowing status indicator dot.
#[derive(Debug, Clone)]
pub struct GlassBadge<'a> {
    text: &'a str,
    variant: BadgeVariant,
    dot: bool,
}

impl<'a> GlassBadge<'a> {
    /// Creates a new [`GlassBadge`] with [`BadgeVariant::Neutral`].
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            variant: BadgeVariant::Neutral,
            dot: false,
        }
    }

    /// Sets the semantic variant.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shows a small luminous status dot before the badge text.
    pub fn dot(mut self, dot: bool) -> Self {
        self.dot = dot;
        self
    }

    /// Renders the badge inside `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for GlassBadge<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let font = FontId::proportional(11.5);
        let galley = ui.painter().layout_no_wrap(
            self.text.to_owned(),
            font.clone(),
            theme.palette.text_primary,
        );

        let dot_space = if self.dot { 12.0 } else { 0.0 };
        let desired = vec2(galley.size().x + 18.0 + dot_space, 23.0);
        let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());

        if ui.is_rect_visible(rect) {
            let (base_color, text_color) = match self.variant {
                BadgeVariant::Neutral => {
                    (theme.palette.text_secondary, theme.palette.text_secondary)
                }
                BadgeVariant::Accent => (theme.palette.accent, theme.palette.accent),
                BadgeVariant::Warm => (theme.palette.accent_warm, theme.palette.accent_warm),
                BadgeVariant::Success => (theme.palette.success, theme.palette.success),
                BadgeVariant::Warning => (theme.palette.warning, theme.palette.warning),
            };

            let (fill_alpha, border_alpha) = match (self.variant, theme.mode) {
                (BadgeVariant::Neutral, ThemeMode::Dark) => (0.14, 0.24),
                (BadgeVariant::Neutral, ThemeMode::Light) => (0.10, 0.22),
                (_, ThemeMode::Dark) => (0.18, 0.44),
                (_, ThemeMode::Light) => (0.14, 0.36),
            };

            let cr = CornerRadius::same((rect.height() * 0.5).round() as u8);
            ui.painter()
                .rect_filled(rect, cr, scale_alpha(base_color, fill_alpha));
            ui.painter().rect_stroke(
                rect,
                cr,
                Stroke::new(1.0, scale_alpha(base_color, border_alpha)),
                StrokeKind::Inside,
            );

            let text_x = if self.dot {
                let dot_center = pos2(rect.min.x + 11.5, rect.center().y);
                ui.painter()
                    .circle_filled(dot_center, 5.0, scale_alpha(base_color, 0.26));
                ui.painter().circle_filled(dot_center, 2.8, base_color);
                rect.min.x + 21.0
            } else {
                rect.center().x
            };

            let align = if self.dot {
                Align2::LEFT_CENTER
            } else {
                Align2::CENTER_CENTER
            };

            let resolved_text = match (self.variant, theme.mode) {
                (BadgeVariant::Neutral, _) => theme.palette.text_primary,
                (_, ThemeMode::Dark) => Color32::from_rgb(
                    base_color.r().saturating_add(36),
                    base_color.g().saturating_add(36),
                    base_color.b().saturating_add(36),
                ),
                (_, ThemeMode::Light) => text_color,
            };

            ui.painter().text(
                pos2(text_x, rect.center().y),
                align,
                self.text,
                font,
                resolved_text,
            );
        }

        response
    }
}
