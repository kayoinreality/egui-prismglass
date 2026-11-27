//! Glassmorphic panel container widget ([`GlassPanel`]).

use egui::{Color32, Frame, InnerResponse, Margin, Shape, Ui};

use crate::theme::PrismTheme;
use crate::tokens::GlassElevation;

/// A floating or docked glassmorphic panel suitable for headers, toolbars,
/// inspectors, or grouped workspace sections.
///
/// Unlike [`crate::GlassCard`], [`GlassPanel`] defaults to full-width layout and
/// uses the theme's dedicated `panel_surface` tokens with balanced padding.
#[derive(Debug, Clone)]
pub struct GlassPanel {
    elevation: Option<GlassElevation>,
    padding: Option<Margin>,
    corner_radius: Option<f32>,
    tint_override: Option<Color32>,
    fill_width: bool,
    min_height: Option<f32>,
    translucent_overlay: bool,
}

impl Default for GlassPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl GlassPanel {
    /// Creates a new [`GlassPanel`] using the active theme's panel tokens.
    pub fn new() -> Self {
        Self {
            elevation: None,
            padding: None,
            corner_radius: None,
            tint_override: None,
            fill_width: true,
            min_height: None,
            translucent_overlay: false,
        }
    }

    /// Overrides the panel's elevation tier.
    pub fn elevation(mut self, elevation: GlassElevation) -> Self {
        self.elevation = Some(elevation);
        self
    }

    /// Overrides the inner padding margin (defaults to horizontal `lg`, vertical `md`).
    pub fn padding(mut self, margin: Margin) -> Self {
        self.padding = Some(margin);
        self
    }

    /// Overrides the corner radius in logical points.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Overrides the translucent tint color.
    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint_override = Some(tint);
        self
    }

    /// Controls whether the panel expands to fill available horizontal width (default: `true`).
    pub fn fill_width(mut self, fill_width: bool) -> Self {
        self.fill_width = fill_width;
        self
    }

    /// Sets an optional minimum height for the panel body.
    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    /// Renders this panel as a real GPU alpha-blended translucent overlay over live `egui` content.
    pub fn translucent_overlay(mut self, enabled: bool) -> Self {
        self.translucent_overlay = enabled;
        self
    }

    /// Renders the glass panel and invokes `add_contents` inside it.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let mut tokens = match self.elevation {
            Some(elev) => theme.surface(elev),
            None => theme.panel_surface(),
        };
        if let Some(r) = self.corner_radius {
            tokens = tokens.with_corner_radius(r);
        }
        if let Some(tint) = self.tint_override {
            tokens = tokens.with_tint(tint);
        }

        let padding = self
            .padding
            .unwrap_or_else(|| Margin::symmetric(theme.spacing.lg as i8, theme.spacing.md as i8));

        let bg_idx = ui.painter().add(Shape::Noop);

        let prepared = Frame::NONE.inner_margin(padding).show(ui, |ui| {
            ui.vertical(|ui| {
                if self.fill_width {
                    let w = ui.available_width();
                    ui.set_width(w);
                    ui.set_max_width(w);
                }
                if let Some(min_h) = self.min_height {
                    ui.set_min_height(min_h);
                }
                add_contents(ui)
            })
            .inner
        });

        let rect = prepared.response.rect;
        let surface_id =
            ui.make_persistent_id(("prism_panel_surface", rect.min.x as i32, rect.min.y as i32));
        let shapes = crate::blur::resolve_glass_surface_shapes_with_overlay(
            ui,
            surface_id,
            rect,
            &tokens,
            &theme,
            self.translucent_overlay,
        );
        ui.painter().set(bg_idx, Shape::Vec(shapes));

        prepared
    }
}
