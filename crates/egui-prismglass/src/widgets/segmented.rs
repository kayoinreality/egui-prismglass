//! Segmented control widget ([`GlassSegmented`]).

use egui::{Color32, Response, Sense, Ui, Widget, pos2, vec2};

use super::prism_paint::{control_colors, drop_shadow, fill};
use crate::theme::{PrismTheme, SurfaceStyle, ThemeMode};
use crate::typography::{FontWeight, prism_font, prism_numeric_font};

/// A row of mutually exclusive options (tabs, filters, view modes) bound to an index.
///
/// Works with every theme; it follows the Prism look (recessed track, raised selected segment).
///
/// ```no_run
/// # use egui_prismglass::GlassSegmented;
/// # fn demo(ui: &mut egui::Ui, filter: &mut usize) {
/// ui.add(GlassSegmented::new(filter, &["All", "Active", "Done"]).counts(&[10, 4, 5]));
/// # }
/// ```
pub struct GlassSegmented<'a> {
    selected: &'a mut usize,
    labels: Vec<String>,
    counts: Vec<u32>,
    fill_width: bool,
    height: f32,
}

impl<'a> GlassSegmented<'a> {
    /// Creates a segmented control over `labels`, with `selected` as the active index.
    pub fn new(selected: &'a mut usize, labels: &[&str]) -> Self {
        Self {
            selected,
            labels: labels.iter().map(|l| (*l).to_owned()).collect(),
            counts: Vec::new(),
            fill_width: false,
            height: 30.0,
        }
    }

    /// Shows a muted count after each label (e.g. number of items behind a filter).
    pub fn counts(mut self, counts: &[u32]) -> Self {
        self.counts = counts.to_vec();
        self
    }

    /// Stretches the control over the available width with equal-width segments.
    pub fn fill_width(mut self, fill_width: bool) -> Self {
        self.fill_width = fill_width;
        self
    }

    /// Overrides the total height in points (default `30`).
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(20.0);
        self
    }
}

impl Widget for GlassSegmented<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let ctx = ui.ctx().clone();
        let label_font = prism_font(&ctx, 12.0, FontWeight::Medium);
        let count_font = prism_numeric_font(&ctx, 12.0, FontWeight::Regular);

        let galleys: Vec<_> = self
            .labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let main = ui.painter().layout_no_wrap(
                    label.clone(),
                    label_font.clone(),
                    theme.palette.text_primary,
                );
                let count = self.counts.get(i).map(|c| {
                    ui.painter().layout_no_wrap(
                        c.to_string(),
                        count_font.clone(),
                        theme.palette.text_muted,
                    )
                });
                (main, count)
            })
            .collect();
        let natural: Vec<f32> = galleys
            .iter()
            .map(|(m, c)| m.size().x + c.as_ref().map_or(0.0, |c| c.size().x + 4.0) + 20.0)
            .collect();

        let pad = 2.0;
        let gap = 2.0;
        let n = galleys.len().max(1) as f32;
        let total_w = if self.fill_width {
            ui.available_width()
        } else {
            natural.iter().sum::<f32>() + gap * (n - 1.0) + pad * 2.0
        };
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(total_w, self.height), Sense::click());

        let inner = rect.shrink(pad);
        let widths: Vec<f32> = if self.fill_width {
            vec![(inner.width() - gap * (n - 1.0)) / n; galleys.len()]
        } else {
            natural
        };
        let mut seg_rects = Vec::with_capacity(widths.len());
        let mut x = inner.min.x;
        for w in &widths {
            seg_rects.push(egui::Rect::from_min_size(
                pos2(x, inner.min.y),
                vec2(*w, inner.height()),
            ));
            x += w + gap;
        }

        let hover_idx = response
            .hover_pos()
            .and_then(|p| seg_rects.iter().position(|r| r.contains(p)));
        if hover_idx.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if response.clicked()
            && let Some(i) = hover_idx
            && i != *self.selected
        {
            *self.selected = i;
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let dark = theme.mode == ThemeMode::Dark;
            let (track, track_border) = match (theme.surface_style, dark) {
                (SurfaceStyle::Frosted, _) => control_colors(&theme),
                (_, true) => (theme.neutrals.raised, theme.neutrals.border),
                (_, false) => (theme.neutrals.sunken, Color32::TRANSPARENT),
            };
            let outer_r = (theme.radius.md + 1.0).min(rect.height() * 0.5);
            fill(painter, rect, outer_r, track, track_border);

            let seg_r = (outer_r - pad).max(0.0);
            let on_fill = if theme.surface_style == SurfaceStyle::Frosted {
                if dark {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 36)
                } else {
                    Color32::WHITE
                }
            } else {
                theme.neutrals.control_on
            };
            for (i, ((main, count), seg)) in galleys.into_iter().zip(&seg_rects).enumerate() {
                let selected = i == *self.selected;
                if selected {
                    drop_shadow(
                        painter,
                        *seg,
                        seg_r,
                        egui::Shadow {
                            offset: [0, 1],
                            blur: 2,
                            spread: 0,
                            color: Color32::from_rgba_unmultiplied(
                                0,
                                0,
                                0,
                                if dark { 64 } else { 20 },
                            ),
                        },
                    );
                    fill(painter, *seg, seg_r, on_fill, theme.neutrals.border);
                } else if hover_idx == Some(i) {
                    fill(
                        painter,
                        *seg,
                        seg_r,
                        theme.neutrals.hover,
                        Color32::TRANSPARENT,
                    );
                }
                let text_color = if selected {
                    theme.palette.text_primary
                } else {
                    theme.palette.text_secondary
                };
                let content_w = main.size().x + count.as_ref().map_or(0.0, |c| c.size().x + 4.0);
                let mut x = seg.center().x - content_w * 0.5;
                let main_w = main.size().x;
                painter.galley(
                    pos2(x, seg.center().y - main.size().y * 0.5),
                    main,
                    text_color,
                );
                x += main_w + 4.0;
                if let Some(count) = count {
                    painter.galley(
                        pos2(x, seg.center().y - count.size().y * 0.5),
                        count,
                        theme.palette.text_muted,
                    );
                }
            }
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segmented_control_renders_in_every_preset() {
        for preset in crate::PrismPreset::PRISM
            .into_iter()
            .chain(crate::PrismPreset::CLASSIC)
        {
            let ctx = egui::Context::default();
            ctx.set_visuals(egui::Visuals::dark());
            preset.into_theme().apply(&ctx);
            let mut selected = 1;
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui.add(
                    GlassSegmented::new(&mut selected, &["All", "Active", "Done"])
                        .counts(&[3, 2, 1]),
                );
                ui.add(GlassSegmented::new(&mut selected, &["1080p", "4K"]).fill_width(true));
            });
            output.textures_delta.clear();
            assert_eq!(selected, 1);
        }
    }
}
