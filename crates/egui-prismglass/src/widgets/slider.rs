//! Glassmorphic slider widget ([`GlassSlider`]).

use std::ops::RangeInclusive;

use egui::{
    Align2, Color32, CornerRadius, FontId, Response, Sense, Stroke, Ui, Widget, pos2, vec2,
};

use crate::theme::{PrismTheme, ThemeMode};
use crate::tokens::scale_alpha;

/// A frosted-glass horizontal slider with a luminous filled track, specular thumb,
/// optional label, and formatted value readout.
pub struct GlassSlider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    label: Option<String>,
    suffix: Option<String>,
    decimals: usize,
    accent: Option<Color32>,
}

impl<'a> GlassSlider<'a> {
    /// Creates a new [`GlassSlider`] bound to `value` over `range`.
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            label: None,
            suffix: None,
            decimals: 2,
            accent: None,
        }
    }

    /// Displays a descriptive label above the slider track.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Appends a unit suffix to the numeric readout (e.g. `"px"` or `"%"`).
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Sets the number of fractional digits displayed in the readout.
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    /// Overrides the filled-track accent color.
    pub fn accent(mut self, accent: Color32) -> Self {
        self.accent = Some(accent);
        self
    }

    /// Renders the slider inside `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for GlassSlider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let accent = self.accent.unwrap_or(theme.palette.accent);
        let min = *self.range.start();
        let max = *self.range.end();
        let span = (max - min).max(f32::EPSILON);

        let has_header = self.label.is_some();
        let header_h = if has_header { 20.0 } else { 0.0 };
        let track_area_h = 24.0;
        let width = ui.available_width().max(120.0);

        let (outer_rect, mut response) = ui.allocate_exact_size(
            vec2(width, header_h + track_area_h),
            Sense::click_and_drag(),
        );

        let track_rect = egui::Rect::from_min_max(
            pos2(outer_rect.min.x, outer_rect.min.y + header_h),
            outer_rect.max,
        );
        let thumb_r = 7.5;
        let rail_min_x = track_rect.min.x + thumb_r;
        let rail_max_x = (track_rect.max.x - thumb_r).max(rail_min_x + 1.0);

        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }

        if (response.dragged() || response.clicked())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let norm = ((pointer.x - rail_min_x) / (rail_max_x - rail_min_x)).clamp(0.0, 1.0);
            let next = min + norm * span;
            if (*self.value - next).abs() > f32::EPSILON {
                *self.value = next;
                response.mark_changed();
            }
        }

        let norm = ((*self.value - min) / span).clamp(0.0, 1.0);

        if ui.is_rect_visible(outer_rect) {
            // Optional header with label on the left and tabular readout on the right.
            if let Some(label) = &self.label {
                let header_y = outer_rect.min.y + 8.0;
                ui.painter().text(
                    pos2(outer_rect.min.x, header_y),
                    Align2::LEFT_CENTER,
                    label,
                    FontId::proportional(12.5),
                    theme.palette.text_secondary,
                );

                let formatted = match &self.suffix {
                    Some(suf) => format!("{:.prec$}{suf}", *self.value, prec = self.decimals),
                    None => format!("{:.prec$}", *self.value, prec = self.decimals),
                };
                ui.painter().text(
                    pos2(outer_rect.max.x, header_y),
                    Align2::RIGHT_CENTER,
                    formatted,
                    FontId::monospace(11.5),
                    theme.palette.text_primary,
                );
            }

            let center_y = track_rect.center().y;
            let rail_h = 4.0;
            let rail_rect = egui::Rect::from_min_max(
                pos2(rail_min_x, center_y - rail_h * 0.5),
                pos2(rail_max_x, center_y + rail_h * 0.5),
            );

            // Clean translucent frosted track.
            let track_fill = match theme.mode {
                ThemeMode::Dark => Color32::from_rgba_unmultiplied(255, 255, 255, 24),
                ThemeMode::Light => Color32::from_rgba_unmultiplied(18, 19, 23, 22),
            };
            ui.painter()
                .rect_filled(rail_rect, CornerRadius::same(2), track_fill);

            // Crisp active progress bar.
            let thumb_x = rail_min_x + (rail_max_x - rail_min_x) * norm;
            let fill_rect = egui::Rect::from_min_max(rail_rect.min, pos2(thumb_x, rail_rect.max.y));
            if fill_rect.width() > 1.0 {
                ui.painter()
                    .rect_filled(fill_rect, CornerRadius::same(2), accent);
            }

            // Minimal Apple-style circular white thumb with subtle hairline border.
            let thumb_center = pos2(thumb_x, center_y);
            let active = response.hovered() || response.dragged();
            if active {
                ui.painter()
                    .circle_filled(thumb_center, thumb_r + 4.0, scale_alpha(accent, 0.18));
            }
            ui.painter()
                .circle_filled(thumb_center, thumb_r, Color32::WHITE);
            let thumb_border = match theme.mode {
                ThemeMode::Dark => Color32::from_rgba_unmultiplied(18, 19, 23, 95),
                ThemeMode::Light => Color32::from_rgba_unmultiplied(18, 19, 23, 42),
            };
            ui.painter()
                .circle_stroke(thumb_center, thumb_r, Stroke::new(1.0, thumb_border));
        }

        response
    }
}
