//! Glassmorphic card container widget ([`GlassCard`]).

use egui::{Color32, Frame, InnerResponse, Margin, Sense, Shape, Ui};

use crate::painter::glass_surface_shapes;
use crate::theme::PrismTheme;
use crate::tokens::GlassElevation;

/// A translucent frosted-glass card container with gradient fill, luminous top
/// specular border, and soft elevation shadow.
///
/// # Example
/// ```no_run
/// # use egui_prismglass::{GlassCard, GlassElevation};
/// # fn demo(ui: &mut egui::Ui) {
/// GlassCard::new()
///     .elevation(GlassElevation::Elevated)
///     .show(ui, |ui| {
///         ui.heading("Prism Card");
///         ui.label("Works on every egui backend.");
///     });
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GlassCard {
    elevation: GlassElevation,
    padding: Option<Margin>,
    corner_radius: Option<f32>,
    tint_override: Option<Color32>,
    prism_intensity: Option<f32>,
    interactive: bool,
    fill_width: bool,
    min_height: Option<f32>,
}

impl Default for GlassCard {
    fn default() -> Self {
        Self::new()
    }
}

impl GlassCard {
    /// Creates a new [`GlassCard`] with [`GlassElevation::Elevated`].
    pub fn new() -> Self {
        Self {
            elevation: GlassElevation::Elevated,
            padding: None,
            corner_radius: None,
            tint_override: None,
            prism_intensity: None,
            interactive: false,
            fill_width: false,
            min_height: None,
        }
    }

    /// Sets the card's glass elevation tier.
    pub fn elevation(mut self, elevation: GlassElevation) -> Self {
        self.elevation = elevation;
        self
    }

    /// Overrides the inner padding margin (defaults to `theme.spacing.lg`).
    pub fn padding(mut self, margin: Margin) -> Self {
        self.padding = Some(margin);
        self
    }

    /// Overrides the corner radius in logical points.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Overrides the core glass tint color.
    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint_override = Some(tint);
        self
    }

    /// Overrides the top-edge prismatic refraction intensity (`0.0..=1.0`).
    pub fn prism_intensity(mut self, intensity: f32) -> Self {
        self.prism_intensity = Some(intensity);
        self
    }

    /// Enables interactive hover/click feedback on the card surface.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Expands the card to take the full available width of its parent layout.
    pub fn fill_width(mut self, fill_width: bool) -> Self {
        self.fill_width = fill_width;
        self
    }

    /// Enforces a minimum height for the card's inner layout.
    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    /// Renders the glass card and executes `add_contents` inside its padded body.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let mut tokens = theme.surface(self.elevation);
        if let Some(r) = self.corner_radius {
            tokens = tokens.with_corner_radius(r);
        }
        if let Some(tint) = self.tint_override {
            tokens = tokens.with_tint(tint);
        }
        if let Some(prism) = self.prism_intensity {
            tokens = tokens.with_prism_intensity(prism);
        }

        let padding = self
            .padding
            .unwrap_or_else(|| Margin::same(theme.spacing.lg as i8));

        // Reserve a background shape slot BEFORE children paint so the card's
        // exact laid-out rectangle is rendered strictly behind its contents on frame 1.
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
        let response = if self.interactive {
            let id =
                ui.make_persistent_id(("prism_glass_card", rect.min.x as i32, rect.min.y as i32));
            let resp = ui.interact(rect, id, Sense::click());
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            resp
        } else {
            prepared.response
        };

        let (effective_rect, effective_tokens) = if self.interactive {
            let hover_t = crate::anim::animate_hover(ui.ctx(), response.id, response.hovered());
            let lifted_rect = rect.translate(egui::vec2(0.0, -1.5 * hover_t));
            let next_tokens = if response.is_pointer_button_down_on() {
                tokens.pressed()
            } else if hover_t > 0.01 {
                tokens
                    .with_specular_scale(1.0 + 0.35 * hover_t)
                    .with_opacity_scale(1.0 + 0.06 * hover_t)
            } else {
                tokens
            };
            (lifted_rect, next_tokens)
        } else {
            (rect, tokens)
        };

        let shapes = glass_surface_shapes(effective_rect, &effective_tokens);
        ui.painter().set(bg_idx, Shape::Vec(shapes));

        InnerResponse::new(prepared.inner, response)
    }
}
