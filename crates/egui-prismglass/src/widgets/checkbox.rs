//! Checkbox widget ([`GlassCheckbox`]).

use egui::{Color32, Response, Sense, Stroke, Ui, Widget, pos2, vec2};

use super::prism_paint::{control_colors, fill};
use crate::theme::PrismTheme;
use crate::typography::{FontWeight, prism_font};

/// A 16pt checkbox: hairline box when off, solid accent with a white check when on.
///
/// ```no_run
/// # use egui_prismglass::GlassCheckbox;
/// # fn demo(ui: &mut egui::Ui, denoise: &mut bool) {
/// ui.add(GlassCheckbox::new(denoise).label("Denoise"));
/// # }
/// ```
pub struct GlassCheckbox<'a> {
    checked: &'a mut bool,
    label: Option<String>,
}

impl<'a> GlassCheckbox<'a> {
    /// Creates a checkbox bound to `checked`.
    pub fn new(checked: &'a mut bool) -> Self {
        Self {
            checked,
            label: None,
        }
    }

    /// Adds a clickable label after the box (also used as the accessible name).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Widget for GlassCheckbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let box_size = 16.0;
        let galley = self.label.as_ref().map(|text| {
            ui.painter().layout_no_wrap(
                text.clone(),
                prism_font(ui.ctx(), 13.0, FontWeight::Regular),
                theme.palette.text_primary,
            )
        });
        let width = galley
            .as_ref()
            .map_or(box_size, |g| box_size + 8.0 + g.size().x);
        let (rect, mut response) = ui.allocate_exact_size(vec2(width, 20.0), Sense::click());
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        let checked = *self.checked;
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::Role::CheckBox,
                ui.is_enabled(),
                checked,
                self.label.clone().unwrap_or_default(),
            )
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let anim = ui.ctx().animate_bool_responsive(response.id, checked);
            let bx = egui::Rect::from_center_size(
                pos2(rect.min.x + box_size * 0.5, rect.center().y),
                vec2(box_size, box_size),
            );
            let (off_fill, off_border) = control_colors(&theme);
            let border = if response.hovered() && !checked {
                theme.palette.text_muted
            } else {
                off_border
            };
            fill(
                painter,
                bx,
                4.0,
                crate::tokens::blend_colors(off_fill, theme.palette.accent, anim),
                crate::tokens::scale_alpha(border, 1.0 - anim),
            );
            if anim > 0.05 {
                let c = bx.center();
                let check = vec![
                    pos2(c.x - 3.6, c.y + 0.2),
                    pos2(c.x - 1.0, c.y + 2.8),
                    pos2(c.x + 3.8, c.y - 2.6),
                ];
                let on = theme.on_accent();
                painter.add(egui::Shape::line(
                    check,
                    Stroke::new(
                        1.8,
                        Color32::from_rgba_unmultiplied(
                            on.r(),
                            on.g(),
                            on.b(),
                            (255.0 * anim) as u8,
                        ),
                    ),
                ));
            }
            if let Some(galley) = galley {
                painter.galley(
                    pos2(bx.max.x + 8.0, rect.center().y - galley.size().y * 0.5),
                    galley,
                    theme.palette.text_primary,
                );
            }
        }
        response
    }
}
