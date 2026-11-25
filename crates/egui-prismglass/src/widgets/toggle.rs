//! Glassmorphic toggle switch widget ([`GlassToggle`]).

use egui::{
    Align2, Color32, CornerRadius, FontId, Response, Sense, Stroke, StrokeKind, Ui, Widget, pos2,
    vec2,
};

use crate::theme::{PrismTheme, ThemeMode};
use crate::tokens::{blend_colors, scale_alpha};

/// A frosted-glass boolean toggle switch with smooth thumb animation and optional label.
pub struct GlassToggle<'a> {
    on: &'a mut bool,
    label: Option<String>,
}

impl<'a> GlassToggle<'a> {
    /// Creates a new [`GlassToggle`] bound to `on`.
    pub fn new(on: &'a mut bool) -> Self {
        Self { on, label: None }
    }

    /// Attaches a label rendered cleanly beside the switch.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Renders the toggle switch inside `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for GlassToggle<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let switch_w: f32 = 40.0;
        let switch_h: f32 = 22.0;

        let font = FontId::proportional(13.0);
        let label_galley = self.label.as_ref().map(|text| {
            ui.painter()
                .layout_no_wrap(text.clone(), font.clone(), theme.palette.text_primary)
        });

        let total_w = match &label_galley {
            Some(g) => switch_w + 10.0 + g.size().x,
            None => switch_w,
        };
        let (outer_rect, mut response) =
            ui.allocate_exact_size(vec2(total_w, switch_h.max(24.0)), Sense::click());

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        if response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }

        let anim = ui.ctx().animate_bool_responsive(response.id, *self.on);

        if ui.is_rect_visible(outer_rect) {
            let switch_rect = egui::Rect::from_center_size(
                pos2(outer_rect.min.x + switch_w * 0.5, outer_rect.center().y),
                vec2(switch_w, switch_h),
            );

            let off_bg = match theme.mode {
                ThemeMode::Dark => Color32::from_rgba_unmultiplied(255, 255, 255, 24),
                ThemeMode::Light => Color32::from_rgba_unmultiplied(77, 80, 110, 34),
            };
            let on_bg = scale_alpha(theme.palette.accent, 0.85);
            let bg = blend_colors(off_bg, on_bg, anim);

            let border_off = match theme.mode {
                ThemeMode::Dark => Color32::from_rgba_unmultiplied(210, 220, 255, 42),
                ThemeMode::Light => Color32::from_rgba_unmultiplied(108, 114, 172, 48),
            };
            let border = blend_colors(border_off, scale_alpha(Color32::WHITE, 0.5), anim);

            let cr = CornerRadius::same((switch_h * 0.5).round() as u8);
            ui.painter().rect_filled(switch_rect, cr, bg);
            ui.painter().rect_stroke(
                switch_rect,
                cr,
                Stroke::new(1.0, border),
                StrokeKind::Inside,
            );

            let knob_r = 8.0;
            let x_off = switch_rect.min.x + knob_r + 3.0;
            let x_on = switch_rect.max.x - knob_r - 3.0;
            let knob_center = pos2(x_off + (x_on - x_off) * anim, switch_rect.center().y);

            if anim > 0.05 {
                ui.painter().circle_filled(
                    knob_center,
                    knob_r + 3.5,
                    scale_alpha(theme.palette.accent, 0.22 * anim),
                );
            }
            ui.painter()
                .circle_filled(knob_center, knob_r, Color32::from_rgb(248, 250, 255));

            if let Some(label) = &self.label {
                ui.painter().text(
                    pos2(switch_rect.max.x + 10.0, outer_rect.center().y),
                    Align2::LEFT_CENTER,
                    label,
                    font,
                    theme.palette.text_primary,
                );
            }
        }

        response
    }
}
