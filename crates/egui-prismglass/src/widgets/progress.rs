//! Thin progress bar widget ([`GlassProgress`]).

use egui::{Color32, CornerRadius, Response, Sense, Ui, Widget, pos2, vec2};

use super::prism_paint::track_color;
use crate::theme::PrismTheme;

/// A thin, rounded progress bar (default `4` pt tall) on a recessed track.
///
/// ```no_run
/// # use egui_prismglass::GlassProgress;
/// # fn demo(ui: &mut egui::Ui) {
/// ui.add(GlassProgress::new(0.31).height(6.0));
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GlassProgress {
    fraction: f32,
    height: f32,
    width: Option<f32>,
    color: Option<Color32>,
}

impl GlassProgress {
    /// Creates a progress bar filled to `fraction` (`0.0..=1.0`).
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            height: 4.0,
            width: None,
            color: None,
        }
    }

    /// Bar thickness in points (default `4`).
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.0);
        self
    }

    /// Fixed width in points (defaults to the available width).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(4.0));
        self
    }

    /// Fill color (defaults to the theme accent). Use status colors for status, e.g. danger
    /// for a failed job or a neutral color for completed ones.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

impl Widget for GlassProgress {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let (rect, response) = ui.allocate_exact_size(vec2(width, self.height), Sense::hover());
        let fraction = self.fraction;
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::Role::ProgressIndicator,
                true,
                format!("{:.0}%", fraction * 100.0),
            )
        });
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let cr = CornerRadius::same((self.height * 0.5).round().clamp(0.0, 255.0) as u8);
            painter.rect_filled(rect, cr, track_color(&theme));
            let fill_w = rect.width() * self.fraction;
            if fill_w > 0.5 {
                let filled = egui::Rect::from_min_max(
                    rect.min,
                    pos2(rect.min.x + fill_w.max(self.height), rect.max.y),
                );
                painter.rect_filled(filled, cr, self.color.unwrap_or(theme.palette.accent));
            }
        }
        response
    }
}
