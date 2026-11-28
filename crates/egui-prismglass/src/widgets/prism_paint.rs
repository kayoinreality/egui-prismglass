//! Small painting helpers shared by the Prism-style widget renderers.

use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Shadow, Stroke, StrokeKind, pos2};

use crate::theme::{PrismTheme, SurfaceStyle, ThemeMode};
use crate::tokens::blend_colors;

pub(crate) fn corner(radius: f32) -> CornerRadius {
    CornerRadius::same(radius.round().clamp(0.0, 255.0) as u8)
}

/// Flat rounded fill with an optional inside hairline.
pub(crate) fn fill(painter: &Painter, rect: Rect, radius: f32, fill: Color32, border: Color32) {
    let cr = corner(radius.min(rect.height() * 0.5));
    if fill.a() > 0 {
        painter.rect_filled(rect, cr, fill);
    }
    if border.a() > 0 {
        painter.rect_stroke(rect, cr, Stroke::new(1.0, border), StrokeKind::Inside);
    }
}

/// A 1px highlight along the top inner edge (the "inset 0 1px 0" sheen of solid buttons).
pub(crate) fn top_highlight(painter: &Painter, rect: Rect, radius: f32, color: Color32) {
    if color.a() == 0 || rect.width() < 2.0 * radius + 2.0 {
        return;
    }
    let r = radius.min(rect.height() * 0.5);
    let y = rect.min.y + 1.0;
    painter.line_segment(
        [pos2(rect.min.x + r, y), pos2(rect.max.x - r, y)],
        Stroke::new(1.0, color),
    );
}

pub(crate) fn drop_shadow(painter: &Painter, rect: Rect, radius: f32, shadow: Shadow) {
    if shadow.color.a() > 0 {
        painter.add(shadow.as_shape(rect, corner(radius.min(rect.height() * 0.5))));
    }
}

/// White switch / slider knob with a soft contact shadow.
pub(crate) fn knob(painter: &Painter, center: Pos2, radius: f32) {
    painter.circle_filled(
        pos2(center.x, center.y + 1.0),
        radius + 0.6,
        Color32::from_rgba_unmultiplied(0, 0, 0, 46),
    );
    painter.circle_filled(center, radius, Color32::WHITE);
    painter.circle_stroke(
        center,
        radius,
        Stroke::new(0.5, Color32::from_rgba_unmultiplied(0, 0, 0, 40)),
    );
}

/// Fill and border for a raised control (secondary button, segmented track) in the active
/// Prism material: solid neutrals in Layered, translucent frost in Frosted.
pub(crate) fn control_colors(theme: &PrismTheme) -> (Color32, Color32) {
    let dark = theme.mode == ThemeMode::Dark;
    match (theme.surface_style, dark) {
        (SurfaceStyle::Frosted, true) => (
            Color32::from_rgba_unmultiplied(255, 255, 255, 16),
            Color32::from_rgba_unmultiplied(255, 255, 255, 30),
        ),
        (SurfaceStyle::Frosted, false) => (
            Color32::from_rgba_unmultiplied(255, 255, 255, 150),
            Color32::from_rgba_unmultiplied(0, 0, 0, 24),
        ),
        _ => (theme.neutrals.raised, theme.neutrals.border_strong),
    }
}

/// Recessed track color (slider rails, progress tracks, switch off state).
pub(crate) fn track_color(theme: &PrismTheme) -> Color32 {
    match (theme.surface_style, theme.mode) {
        (SurfaceStyle::Frosted, ThemeMode::Dark) => {
            Color32::from_rgba_unmultiplied(255, 255, 255, 26)
        }
        (SurfaceStyle::Frosted, ThemeMode::Light) => Color32::from_rgba_unmultiplied(0, 0, 0, 22),
        _ => theme.neutrals.sunken,
    }
}

/// Lightens (`amount > 0`) or darkens (`amount < 0`) a solid fill for hover / press states.
pub(crate) fn shade(color: Color32, amount: f32) -> Color32 {
    if amount >= 0.0 {
        blend_colors(color, Color32::WHITE, amount)
    } else {
        blend_colors(color, Color32::BLACK, -amount)
    }
}
