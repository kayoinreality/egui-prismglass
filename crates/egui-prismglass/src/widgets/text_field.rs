//! Text input widget ([`GlassTextField`]).

use egui::{
    Color32, CornerRadius, Frame, Id, Margin, Rect, Response, Shape, Stroke, StrokeKind, TextEdit,
    Ui, Vec2, Widget, pos2, vec2,
};

use super::prism_paint::with_alpha;
use crate::icons::Icon;
use crate::theme::{PrismTheme, SurfaceStyle, ThemeMode};
use crate::typography::{FontWeight, prism_font};

/// A themed text input built on [`egui::TextEdit`]: single line or multiline, with a
/// placeholder, an optional leading [`Icon`], a label and help text, focus / error / disabled
/// states and password masking.
///
/// The returned [`Response`] is the inner `TextEdit`'s, so `changed()`, `has_focus()` and
/// `lost_focus()` work as usual.
///
/// ```no_run
/// # use egui_prismglass::{GlassTextField, Icon};
/// # fn demo(ui: &mut egui::Ui, query: &mut String, key: &mut String, notes: &mut String) {
/// ui.add(GlassTextField::new(query).placeholder("Search sources").icon(Icon::Search));
///
/// let invalid = key.len() < 8;
/// ui.add(
///     GlassTextField::new(key)
///         .label("Stream key")
///         .password(true)
///         .error(invalid.then_some("At least 8 characters")),
/// );
///
/// ui.add(GlassTextField::multiline(notes).placeholder("Notes").rows(4));
/// # }
/// ```
pub struct GlassTextField<'a> {
    text: &'a mut String,
    multiline: bool,
    placeholder: Option<String>,
    icon: Option<Icon>,
    label: Option<String>,
    help: Option<String>,
    error: Option<String>,
    password: bool,
    enabled: bool,
    width: Option<f32>,
    rows: usize,
    id_salt: Option<Id>,
    char_limit: Option<usize>,
}

impl<'a> GlassTextField<'a> {
    /// A single-line field (32 pt tall).
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            multiline: false,
            placeholder: None,
            icon: None,
            label: None,
            help: None,
            error: None,
            password: false,
            enabled: true,
            width: None,
            rows: 3,
            id_salt: None,
            char_limit: None,
        }
    }

    /// A multiline field (3 rows by default, grows with its contents).
    pub fn multiline(text: &'a mut String) -> Self {
        Self {
            multiline: true,
            ..Self::new(text)
        }
    }

    /// Muted text shown while the field is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// A leading icon (search, key, link, …), painted in the muted text color.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A label above the field (also used as its accessible name).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Muted help text below the field (replaced by the error message when there is one).
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Puts the field in the error state; `Some(message)` shows the message below it.
    ///
    /// Accepts `Option<&str>` / `Option<String>`, e.g. `.error(invalid.then_some("Required"))`.
    pub fn error(mut self, message: Option<impl Into<String>>) -> Self {
        self.error = message.map(Into::into);
        self
    }

    /// Masks the contents (and blocks copying), for passwords and stream keys.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Enables or disables the field (disabled fields are dimmed and not editable).
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the total width in points (default: the available width).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(40.0));
        self
    }

    /// Minimum number of visible rows for multiline fields (default `3`).
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    /// Gives the field a stable id salt (needed when several fields would get the same
    /// automatic id, e.g. when created in a loop with changing order).
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Some(Id::NULL.with(("prism_text_field", salt)));
        self
    }

    /// Limits the number of characters that can be typed.
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }
}

/// Visual state of a field, in priority order (exposed for tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldState {
    Disabled,
    Error,
    Focused,
    Hovered,
    Idle,
}

pub(crate) fn field_colors(theme: &PrismTheme, state: FieldState) -> (Color32, Color32) {
    let dark = theme.mode == ThemeMode::Dark;
    let base = match (theme.surface_style, dark) {
        (SurfaceStyle::Frosted | SurfaceStyle::Glass, true) => {
            Color32::from_rgba_unmultiplied(255, 255, 255, 14)
        }
        (SurfaceStyle::Frosted | SurfaceStyle::Glass, false) => {
            Color32::from_rgba_unmultiplied(255, 255, 255, 150)
        }
        (_, true) => theme.neutrals.sunken,
        (_, false) => theme.neutrals.surface,
    };
    let border = theme.neutrals.border_strong;
    match state {
        FieldState::Disabled => (with_alpha_scaled(base, 0.6), with_alpha_scaled(border, 0.6)),
        FieldState::Error => (base, theme.palette.danger),
        FieldState::Focused => (base, theme.palette.accent),
        FieldState::Hovered => (
            base,
            crate::tokens::blend_colors(border, theme.palette.text_muted, 0.35),
        ),
        FieldState::Idle => (base, border),
    }
}

fn with_alpha_scaled(c: Color32, f: f32) -> Color32 {
    crate::tokens::scale_alpha(c, f)
}

impl Widget for GlassTextField<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let ctx = ui.ctx().clone();
        let width = self.width.unwrap_or_else(|| ui.available_width()).max(40.0);
        let font = prism_font(&ctx, 13.0, FontWeight::Regular);

        // Only stack vertically when there is a label or a message, so a bare field keeps the
        // parent layout's alignment (e.g. vertically centered in a toolbar row).
        let stacked = self.label.is_some() || self.help.is_some() || self.error.is_some();
        let body = |ui: &mut Ui| {
            if stacked {
                ui.set_width(width);
                ui.spacing_mut().item_spacing.y = 5.0;
            }
            let label_id = self.label.as_ref().map(|label| {
                ui.label(
                    egui::RichText::new(label)
                        .font(prism_font(&ctx, 12.0, FontWeight::Medium))
                        .color(if self.enabled {
                            theme.palette.text_secondary
                        } else {
                            theme.palette.text_muted
                        }),
                )
                .id
            });

            let pad_x = 10.0;
            let icon_space = if self.icon.is_some() { 22.0 } else { 0.0 };
            let pad_y = if self.multiline { 8.0 } else { 7.0 };
            let margin = Margin {
                left: (pad_x + icon_space) as i8,
                right: pad_x as i8,
                top: pad_y as i8,
                bottom: pad_y as i8,
            };

            let bg_idx = ui.painter().add(Shape::Noop);
            let placeholder = self.placeholder.clone().unwrap_or_default();
            let hint = egui::RichText::new(placeholder)
                .font(font.clone())
                .color(theme.palette.text_muted);
            let mut edit = if self.multiline {
                TextEdit::multiline(self.text).desired_rows(self.rows)
            } else {
                TextEdit::singleline(self.text).align(egui::Align2::LEFT_CENTER)
            };
            edit = edit
                // With an explicit frame `TextEdit` ignores `.margin()`, so pad via the frame.
                .frame(Frame::NONE.inner_margin(margin))
                .font(font)
                .text_color(theme.palette.text_primary)
                .hint_text(hint)
                .password(self.password)
                .desired_width(width - f32::from(margin.left) - f32::from(margin.right))
                .min_size(vec2(width, if self.multiline { 0.0 } else { 32.0 }));
            if let Some(id) = self.id_salt {
                edit = edit.id(id);
            }
            if let Some(limit) = self.char_limit {
                edit = edit.char_limit(limit);
            }

            let mut response = ui.add_enabled(self.enabled, edit);
            if let Some(label_id) = label_id {
                response = response.labelled_by(label_id);
            }

            let rect = response.rect;
            let state = if !self.enabled {
                FieldState::Disabled
            } else if self.error.is_some() {
                FieldState::Error
            } else if response.has_focus() {
                FieldState::Focused
            } else if response.hovered() {
                FieldState::Hovered
            } else {
                FieldState::Idle
            };
            let (fill, border) = field_colors(&theme, state);
            let radius = (theme.radius.md * theme.radius_scale.clamp(0.0, 2.0)).min(12.0);
            let cr = CornerRadius::same(radius.round() as u8);
            let mut shapes = vec![
                Shape::rect_filled(rect, cr, fill),
                Shape::rect_stroke(rect, cr, Stroke::new(1.0, border), StrokeKind::Inside),
            ];
            if matches!(state, FieldState::Focused | FieldState::Error) {
                // A soft 3 pt focus ring outside the border.
                shapes.insert(
                    0,
                    Shape::rect_stroke(
                        rect.expand(1.5),
                        CornerRadius::same((radius + 1.5).round() as u8),
                        Stroke::new(3.0, with_alpha(border, 52)),
                        StrokeKind::Middle,
                    ),
                );
            }
            ui.painter().set(bg_idx, Shape::Vec(shapes));

            if let Some(icon) = self.icon {
                let cy = if self.multiline {
                    rect.top() + pad_y + 8.0
                } else {
                    rect.center().y
                };
                let color = match state {
                    FieldState::Focused => theme.palette.text_secondary,
                    FieldState::Error => theme.palette.danger,
                    _ => theme.palette.text_muted,
                };
                icon.paint(
                    ui.painter(),
                    Rect::from_center_size(pos2(rect.left() + pad_x + 7.0, cy), Vec2::splat(14.0)),
                    color,
                );
            }

            let message = self
                .error
                .as_ref()
                .map(|e| (e, theme.palette.danger))
                .or_else(|| self.help.as_ref().map(|h| (h, theme.palette.text_muted)));
            if let Some((text, color)) = message {
                ui.label(
                    egui::RichText::new(text)
                        .font(prism_font(&ctx, 11.5, FontWeight::Regular))
                        .color(color),
                );
            }
            response
        };
        if stacked {
            ui.vertical(body).inner
        } else {
            body(ui)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PrismPreset;

    #[test]
    fn text_fields_render_in_every_preset_and_state() {
        for preset in PrismPreset::PRISM.into_iter().chain(PrismPreset::CLASSIC) {
            let ctx = egui::Context::default();
            preset.into_theme().apply(&ctx);
            let mut a = String::new();
            let mut b = String::from("secret");
            let mut c = String::from("line 1\nline 2");
            let mut d = String::from("read only");
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui.add(
                    GlassTextField::new(&mut a)
                        .placeholder("Search")
                        .icon(Icon::Search),
                );
                ui.add(
                    GlassTextField::new(&mut b)
                        .label("Key")
                        .password(true)
                        .error(Some("Too short")),
                );
                ui.add(
                    GlassTextField::multiline(&mut c)
                        .rows(4)
                        .help("Markdown supported"),
                );
                ui.add(GlassTextField::new(&mut d).enabled(false).width(200.0));
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn single_line_field_is_32pt_and_accepts_typing() {
        let ctx = egui::Context::default();
        PrismTheme::prism_dark().apply(&ctx);
        let mut text = String::new();
        let mut height = 0.0;
        let id = "typing";
        // Frame 1: lay out and focus.
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            let r = ui.add(GlassTextField::new(&mut text).id_salt(id));
            height = r.rect.height();
            r.request_focus();
        });
        out.textures_delta.clear();
        // Frame 2: type.
        let input = egui::RawInput {
            events: vec![egui::Event::Text("hi".into())],
            ..Default::default()
        };
        let mut changed = false;
        let mut out = ctx.run_ui(input, |ui| {
            changed = ui.add(GlassTextField::new(&mut text).id_salt(id)).changed();
        });
        out.textures_delta.clear();
        assert!((height - 32.0).abs() < 0.5, "height={height}");
        assert!(changed);
        assert_eq!(text, "hi");
    }

    #[test]
    fn error_state_wins_over_focus_and_uses_danger_border() {
        let theme = PrismTheme::prism_light();
        let (_, border) = field_colors(&theme, FieldState::Error);
        assert_eq!(border, theme.palette.danger);
        let (_, border) = field_colors(&theme, FieldState::Focused);
        assert_eq!(border, theme.palette.accent);
    }
}
