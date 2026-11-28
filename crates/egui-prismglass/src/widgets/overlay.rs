//! Shared painting for floating layers (popovers, menus, modals, toasts).

use egui::{Frame, Margin, Shape, Style, Ui};

use crate::theme::PrismTheme;
use crate::tokens::{GlassElevation, GlassSurfaceTokens};

/// Surface tokens for floating layers: the theme's floating glass with an optional radius.
pub(crate) fn floating_tokens(theme: &PrismTheme, radius: Option<f32>) -> GlassSurfaceTokens {
    let tokens = theme.surface(GlassElevation::Floating);
    match radius {
        Some(r) => tokens.with_corner_radius(r),
        None => tokens,
    }
}

/// Lays out `add_contents` with `margin` and paints a floating glass surface behind it
/// (real GPU blur when available, the frosted fallback otherwise).
pub(crate) fn glass_layer<R>(
    ui: &mut Ui,
    margin: Margin,
    tokens: &GlassSurfaceTokens,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let theme = PrismTheme::from_ctx(ui.ctx());
    let bg_idx = ui.painter().add(Shape::Noop);
    let prepared = Frame::NONE.inner_margin(margin).show(ui, add_contents);
    let rect = prepared.response.rect;
    let id = ui.unique_id().with("prism_floating_layer");
    let shapes =
        crate::blur::resolve_glass_surface_shapes_with_overlay(ui, id, rect, tokens, &theme, true);
    ui.painter().set(bg_idx, Shape::Vec(shapes));
    prepared.inner
}

/// Style modifier for popups that paint their own glass: removes egui's popup frame so only
/// the glass surface shows, and keeps egui's menu tweaks.
pub(crate) fn glass_popup_style(style: &mut Style) {
    egui::containers::menu::menu_style(style);
    style.visuals.window_fill = egui::Color32::TRANSPARENT;
    style.visuals.window_stroke = egui::Stroke::NONE;
    style.visuals.popup_shadow = egui::Shadow::NONE;
    style.spacing.menu_margin = Margin::ZERO;
    style.spacing.item_spacing.y = 1.0;
}
