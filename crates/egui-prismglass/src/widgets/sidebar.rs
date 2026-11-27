//! Floating glass sidebar navigation widget ([`GlassSidebar`] and [`SidebarItem`]).

use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, InnerResponse, Margin, Response, Sense, Shape,
    Stroke, Ui, pos2, vec2,
};

use crate::painter::paint_glass_gradient_fill;
use crate::theme::{PrismTheme, SurfaceStyle, ThemeMode};
use crate::tokens::scale_alpha;

/// A floating frosted-glass navigation sidebar container.
///
/// Designed to hold a brand header, [`SidebarItem`] navigation rows, and footer controls
/// while floating gracefully above an atmospheric backdrop.
#[derive(Debug, Clone)]
pub struct GlassSidebar {
    width: f32,
    padding: Option<Margin>,
    corner_radius: Option<f32>,
}

impl Default for GlassSidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl GlassSidebar {
    /// Creates a new [`GlassSidebar`] with a default width of `220.0` logical points.
    pub fn new() -> Self {
        Self {
            width: 220.0,
            padding: None,
            corner_radius: None,
        }
    }

    /// Sets the fixed width of the sidebar in logical points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(64.0);
        self
    }

    /// Overrides the inner padding margin.
    pub fn padding(mut self, margin: Margin) -> Self {
        self.padding = Some(margin);
        self
    }

    /// Overrides the sidebar corner radius in logical points.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Renders the glass sidebar and invokes `add_contents` inside its vertical body.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let mut tokens = theme.sidebar_surface();
        if let Some(r) = self.corner_radius {
            tokens = tokens.with_corner_radius(r);
        }

        let padding = self
            .padding
            .unwrap_or_else(|| Margin::same(theme.spacing.md as i8));
        let inner_width = (self.width - f32::from(padding.left + padding.right)).max(32.0);
        let target_height = ui.available_height();

        let bg_idx = ui.painter().add(Shape::Noop);

        let prepared = Frame::NONE.inner_margin(padding).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(inner_width);
                ui.set_max_width(inner_width);
                ui.set_min_height(
                    (target_height - f32::from(padding.top + padding.bottom)).max(0.0),
                );
                add_contents(ui)
            })
            .inner
        });

        let rect = prepared.response.rect;
        let surface_id = ui.make_persistent_id((
            "prism_sidebar_surface",
            rect.min.x as i32,
            rect.min.y as i32,
        ));
        let shapes =
            crate::blur::resolve_glass_surface_shapes(ui, surface_id, rect, &tokens, &theme);
        ui.painter().set(bg_idx, Shape::Vec(shapes));

        prepared
    }
}

/// Crisp built-in geometric vector icons for [`SidebarItem`].
///
/// Painted directly via [`egui::Painter`] so they look razor-sharp on every platform
/// and never trigger missing-glyph fallbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SidebarIcon {
    /// Four-point prism diamond (ideal for Overview / Home / AI).
    #[default]
    Prism,
    /// 2x2 rounded bento grid (ideal for Components / Gallery).
    Grid,
    /// Concentric token circles (ideal for Design Tokens / Palette).
    Tokens,
    /// Sliders / settings bars (ideal for Inspector / Preferences).
    Sliders,
}

/// A selectable glass navigation row designed for use inside [`GlassSidebar`].
#[derive(Debug, Clone)]
pub struct SidebarItem<'a> {
    icon: SidebarIcon,
    custom_glyph: Option<&'a str>,
    label: &'a str,
    badge: Option<&'a str>,
    selected: bool,
}

impl<'a> SidebarItem<'a> {
    /// Creates a new [`SidebarItem`] with a built-in vector [`SidebarIcon`] and label.
    pub fn new(icon: SidebarIcon, label: &'a str) -> Self {
        Self {
            icon,
            custom_glyph: None,
            label,
            badge: None,
            selected: false,
        }
    }

    /// Overrides the vector icon with a custom text glyph (must be present in the active font).
    pub fn custom_glyph(mut self, glyph: &'a str) -> Self {
        self.custom_glyph = Some(glyph);
        self
    }

    /// Marks this item as the currently active route.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Attaches an optional trailing pill badge (e.g., `"3"` or `"AI"`).
    pub fn badge(mut self, badge: &'a str) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Renders the navigation item across the available width of the sidebar.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let width = ui.available_width();
        let height = 38.0;
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered();
            let radius = theme.radius.md * theme.radius_scale.clamp(0.4, 1.4);

            if self.selected {
                let (fill, border, inner) = match (theme.surface_style, theme.mode) {
                    (SurfaceStyle::Minimal, ThemeMode::Light) => (
                        Color32::from_rgb(240, 244, 250),
                        Color32::from_rgb(225, 231, 240),
                        Color32::TRANSPARENT,
                    ),
                    (SurfaceStyle::Minimal, ThemeMode::Dark) => (
                        Color32::from_rgb(33, 35, 43),
                        Color32::from_rgb(48, 51, 61),
                        Color32::TRANSPARENT,
                    ),
                    (SurfaceStyle::Glass, ThemeMode::Dark) => (
                        Color32::from_rgba_unmultiplied(255, 255, 255, 22),
                        Color32::from_rgba_unmultiplied(255, 255, 255, 36),
                        Color32::from_rgba_unmultiplied(255, 255, 255, 14),
                    ),
                    (SurfaceStyle::Glass, ThemeMode::Light) => (
                        Color32::from_rgba_unmultiplied(255, 255, 255, 215),
                        Color32::from_rgba_unmultiplied(18, 19, 23, 20),
                        Color32::from_rgba_unmultiplied(255, 255, 255, 200),
                    ),
                };
                paint_glass_gradient_fill(
                    ui.painter(),
                    rect,
                    radius,
                    fill,
                    fill,
                    Stroke::new(1.0, border),
                    inner,
                );
            } else if hovered {
                let hover_fill = match theme.mode {
                    ThemeMode::Dark => Color32::from_rgba_unmultiplied(255, 255, 255, 11),
                    ThemeMode::Light => Color32::from_rgba_unmultiplied(18, 19, 23, 10),
                };
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::same(radius.round() as u8),
                    hover_fill,
                );
            }

            let text_col = if self.selected || hovered {
                theme.palette.text_primary
            } else {
                theme.palette.text_secondary
            };

            let icon_col = if self.selected {
                theme.palette.accent
            } else {
                text_col
            };

            let icon_center = pos2(rect.min.x + 24.0, rect.center().y);
            if let Some(glyph) = self.custom_glyph {
                ui.painter().text(
                    icon_center,
                    Align2::CENTER_CENTER,
                    glyph,
                    FontId::proportional(14.0),
                    icon_col,
                );
            } else {
                paint_sidebar_icon(ui.painter(), icon_center, self.icon, icon_col);
            }

            ui.painter().text(
                pos2(rect.min.x + 42.0, rect.center().y),
                Align2::LEFT_CENTER,
                self.label,
                FontId::proportional(13.5),
                text_col,
            );

            if let Some(badge) = self.badge {
                let badge_font = FontId::proportional(10.5);
                let galley =
                    ui.painter()
                        .layout_no_wrap(badge.to_owned(), badge_font.clone(), text_col);
                let bw = (galley.size().x + 12.0).max(22.0);
                let bh = 18.0;
                let badge_rect = egui::Rect::from_center_size(
                    pos2(rect.max.x - bw * 0.5 - 10.0, rect.center().y),
                    vec2(bw, bh),
                );
                let badge_bg = if self.selected {
                    scale_alpha(theme.palette.accent, 0.28)
                } else {
                    scale_alpha(theme.palette.text_muted, 0.2)
                };
                ui.painter()
                    .rect_filled(badge_rect, CornerRadius::same(9), badge_bg);
                ui.painter().text(
                    badge_rect.center(),
                    Align2::CENTER_CENTER,
                    badge,
                    badge_font,
                    if self.selected {
                        theme.palette.text_primary
                    } else {
                        theme.palette.text_secondary
                    },
                );
            }
        }

        response
    }
}

fn paint_sidebar_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    icon: SidebarIcon,
    color: Color32,
) {
    let stroke = Stroke::new(1.4, color);
    match icon {
        SidebarIcon::Prism => {
            let pts = vec![
                pos2(center.x, center.y - 6.0),
                pos2(center.x + 5.5, center.y),
                pos2(center.x, center.y + 6.0),
                pos2(center.x - 5.5, center.y),
            ];
            painter.add(egui::Shape::closed_line(pts, stroke));
            painter.circle_filled(center, 1.6, color);
        }
        SidebarIcon::Grid => {
            let gap = 2.0;
            let cell = 4.6;
            for dx in [-1.0_f32, 1.0] {
                for dy in [-1.0_f32, 1.0] {
                    let c = pos2(
                        center.x + dx * (cell * 0.5 + gap * 0.5),
                        center.y + dy * (cell * 0.5 + gap * 0.5),
                    );
                    let r = egui::Rect::from_center_size(c, vec2(cell, cell));
                    painter.rect_stroke(r, CornerRadius::same(1), stroke, egui::StrokeKind::Inside);
                }
            }
        }
        SidebarIcon::Tokens => {
            painter.circle_stroke(center, 5.5, stroke);
            painter.circle_filled(center, 2.2, color);
        }
        SidebarIcon::Sliders => {
            for (dy, knob_dx) in [(-4.0_f32, -2.0_f32), (0.0, 2.5), (4.0, -1.0)] {
                let y = center.y + dy;
                painter.line_segment([pos2(center.x - 5.5, y), pos2(center.x + 5.5, y)], stroke);
                painter.circle_filled(pos2(center.x + knob_dx, y), 1.9, color);
            }
        }
    }
}
