//! Modal dialog widget ([`GlassModal`]) built on [`egui::Modal`].

use egui::{Color32, Context, Id, Margin, Modal, Rect, Response, Sense, Ui, Vec2, vec2};

use super::overlay::{floating_tokens, glass_layer};
use crate::icons::Icon;
use crate::theme::{PrismTheme, ThemeMode};
use crate::typography::{FontWeight, prism_font};

/// A centered glass dialog over a dimming scrim, with a title, optional description and icon,
/// a body, and a footer of actions ([`GlassModal::footer`]).
///
/// Built on [`egui::Modal`], so while it is open:
/// - widgets behind it cannot be clicked or focused (focus stays inside the dialog);
/// - `Esc` or a click on the scrim closes it (both can be turned off);
/// - calling [`Ui::close`] from inside (e.g. a "Cancel" button) closes it.
///
/// Closing sets `*open = false`.
///
/// ```no_run
/// # use egui_prismglass::{GlassButton, GlassModal, Icon};
/// # fn demo(ctx: &egui::Context, open: &mut bool) {
/// GlassModal::new("delete-scene")
///     .title("Delete scene?")
///     .description("“Intro” and its 4 sources will be removed. This can't be undone.")
///     .icon(Icon::Trash)
///     .show(ctx, open, |ui| {
///         GlassModal::footer(ui, |ui| {
///             if ui.add(GlassButton::danger("Delete")).clicked() {
///                 // delete…
///                 ui.close();
///             }
///             if ui.add(GlassButton::secondary("Cancel")).clicked() {
///                 ui.close();
///             }
///         });
///     });
/// # }
/// ```
#[must_use = "Call `.show(...)` to display the modal"]
pub struct GlassModal {
    id: Id,
    title: Option<String>,
    description: Option<String>,
    icon: Option<(Icon, Option<Color32>)>,
    width: f32,
    close_on_scrim: bool,
    close_on_escape: bool,
    close_button: bool,
    scrim: Option<Color32>,
}

/// What happened in a [`GlassModal`] this frame.
#[derive(Debug)]
pub struct GlassModalResponse<R> {
    /// The value returned by the body closure.
    pub inner: R,
    /// The response of the dialog surface.
    pub response: Response,
    /// `true` on the frame the dialog closed (scrim click, `Esc`, close button or
    /// [`Ui::close`]).
    pub closed: bool,
}

impl GlassModal {
    /// Creates a modal; `id_salt` must be unique among open modals.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            id: Id::NULL.with(("prism_glass_modal", id_salt)),
            title: None,
            description: None,
            icon: None,
            width: 400.0,
            close_on_scrim: true,
            close_on_escape: true,
            close_button: true,
            scrim: None,
        }
    }

    /// The dialog title (Inter Semibold 16 pt).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Secondary text under the title.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// An icon in a soft circle next to the title, in the muted color.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some((icon, None));
        self
    }

    /// An icon in a soft circle next to the title, in `color` (e.g. the danger color).
    pub fn icon_colored(mut self, icon: Icon, color: Color32) -> Self {
        self.icon = Some((icon, Some(color)));
        self
    }

    /// Dialog width in points (default `400`, clamped to the screen).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(200.0);
        self
    }

    /// Whether a click on the scrim closes the dialog (default `true`).
    pub fn close_on_scrim(mut self, close: bool) -> Self {
        self.close_on_scrim = close;
        self
    }

    /// Whether `Esc` closes the dialog (default `true`).
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.close_on_escape = close;
        self
    }

    /// Whether to show the × button in the header (default `true`; needs a title).
    pub fn close_button(mut self, show: bool) -> Self {
        self.close_button = show;
        self
    }

    /// Overrides the scrim color (default: black at 45 % in dark themes, 28 % in light).
    pub fn scrim(mut self, color: Color32) -> Self {
        self.scrim = Some(color);
        self
    }

    /// Shows the dialog while `*open` is `true`; returns `None` when it is closed.
    pub fn show<R>(
        self,
        ctx: &Context,
        open: &mut bool,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<GlassModalResponse<R>> {
        if !*open {
            return None;
        }
        let theme = PrismTheme::from_ctx(ctx);
        let scrim = self.scrim.unwrap_or_else(|| {
            Color32::from_black_alpha(if theme.mode == ThemeMode::Dark {
                115
            } else {
                72
            })
        });
        let screen = ctx.content_rect();
        let width = self.width.min((screen.width() - 32.0).max(200.0));
        let mut close_clicked = false;

        let modal_response = Modal::new(self.id)
            .backdrop_color(scrim)
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let tokens = floating_tokens(&theme, Some(theme.radius.lg.clamp(10.0, 16.0)));
                glass_layer(ui, Margin::same(20), &tokens, |ui| {
                    ui.set_width(width - 40.0);
                    ui.spacing_mut().item_spacing.y = 6.0;
                    close_clicked = self.header(ui, &theme);
                    body(ui)
                })
            });

        let pressed_escape = self.close_on_escape
            && modal_response.is_top_modal
            && !modal_response.any_popup_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        let closed = close_clicked
            || pressed_escape
            || modal_response.response.should_close()
            || (self.close_on_scrim && modal_response.backdrop_response.clicked());
        if closed {
            *open = false;
        }
        Some(GlassModalResponse {
            inner: modal_response.inner,
            response: modal_response.response,
            closed,
        })
    }

    /// Paints the header; returns `true` if the close button was clicked.
    fn header(&self, ui: &mut Ui, theme: &PrismTheme) -> bool {
        let mut clicked = false;
        if self.title.is_none() && self.description.is_none() {
            return false;
        }
        ui.horizontal_top(|ui| {
            if let Some((icon, color)) = self.icon {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
                let color = color.unwrap_or(theme.palette.text_secondary);
                ui.painter().circle_filled(
                    rect.center(),
                    18.0,
                    super::prism_paint::with_alpha(color, 30),
                );
                icon.paint(
                    ui.painter(),
                    Rect::from_center_size(rect.center(), Vec2::splat(18.0)),
                    color,
                );
                ui.add_space(6.0);
            }
            let close_w = if self.close_button { 28.0 } else { 0.0 };
            ui.vertical(|ui| {
                ui.set_width((ui.available_width() - close_w).max(40.0));
                ui.spacing_mut().item_spacing.y = 4.0;
                if let Some(title) = &self.title {
                    ui.label(
                        egui::RichText::new(title)
                            .font(prism_font(ui.ctx(), 16.0, FontWeight::Semibold))
                            .color(theme.palette.text_primary),
                    );
                }
                if let Some(desc) = &self.description {
                    ui.label(
                        egui::RichText::new(desc)
                            .font(prism_font(ui.ctx(), 13.0, FontWeight::Regular))
                            .color(theme.palette.text_secondary),
                    );
                }
            });
            if self.close_button {
                let (rect, response) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::click());
                let response = response.on_hover_text("Close");
                response
                    .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Close"));
                if response.hovered() || response.has_focus() {
                    ui.painter().rect_filled(rect, 6.0, theme.neutrals.selected);
                }
                Icon::Close.paint(
                    ui.painter(),
                    Rect::from_center_size(rect.center(), Vec2::splat(12.0)),
                    theme.palette.text_secondary,
                );
                clicked = response.clicked();
            }
        });
        ui.add_space(6.0);
        clicked
    }

    /// A footer row: a hairline, then `actions` laid out right to left (add the primary
    /// action first so it ends up on the right).
    pub fn footer<R>(ui: &mut Ui, actions: impl FnOnce(&mut Ui) -> R) -> R {
        let theme = PrismTheme::from_ctx(ui.ctx());
        ui.add_space(8.0);
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
        ui.painter().hline(
            (rect.left() - 20.0)..=(rect.right() + 20.0),
            rect.center().y,
            egui::Stroke::new(1.0, theme.neutrals.border),
        );
        ui.add_space(8.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            actions(ui)
        })
        .inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(k: egui::Key) -> egui::Event {
        egui::Event::Key {
            key: k,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }

    /// Runs one frame with a button behind an open modal; returns whether the button's layer
    /// could be interacted with.
    fn modal_frame(ctx: &egui::Context, events: Vec<egui::Event>, open: &mut bool) -> bool {
        let input = egui::RawInput {
            events,
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        };
        let mut background_interactive = true;
        let mut out = ctx.run_ui(input, |ui| {
            let behind = ui.button("behind");
            let shown = GlassModal::new("test")
                .title("Delete?")
                .description("Really?")
                .icon(Icon::Trash)
                .show(ui.ctx(), open, |ui| {
                    GlassModal::footer(ui, |ui| ui.button("OK"));
                });
            if shown.is_some() {
                background_interactive = ui.ctx().memory(|m| m.allows_interaction(behind.layer_id));
            }
        });
        out.textures_delta.clear();
        background_interactive
    }

    #[test]
    fn escape_closes_and_background_is_blocked() {
        let ctx = egui::Context::default();
        PrismTheme::prism_dark().apply(&ctx);
        let mut open = true;
        modal_frame(&ctx, vec![], &mut open);
        let interactive = modal_frame(&ctx, vec![], &mut open);
        assert!(open);
        assert!(
            !interactive,
            "widgets behind the modal must not be interactive"
        );
        modal_frame(&ctx, vec![key(egui::Key::Escape)], &mut open);
        assert!(!open, "Esc closes the modal");
    }

    #[test]
    fn escape_can_be_disabled_and_closed_modal_returns_none() {
        let ctx = egui::Context::default();
        let mut open = true;
        for _ in 0..2 {
            let input = egui::RawInput {
                events: vec![key(egui::Key::Escape)],
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                let _ = GlassModal::new("sticky").close_on_escape(false).show(
                    ui.ctx(),
                    &mut open,
                    |ui| ui.label("x"),
                );
            });
            out.textures_delta.clear();
        }
        assert!(open);
        let mut hidden = false;
        let mut is_open = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            hidden = GlassModal::new("x")
                .show(ui.ctx(), &mut is_open, |_| ())
                .is_none();
        });
        out.textures_delta.clear();
        assert!(hidden);
    }
}
