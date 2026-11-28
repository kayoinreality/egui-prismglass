//! Small kit components that most apps end up writing themselves: a selectable list row
//! ([`GlassListRow`]), a compact chip ([`GlassChip`]), a keyboard key cap ([`GlassKbd`]), an
//! empty state ([`GlassEmptyState`]) and a tracked uppercase section header
//! ([`SectionHeader`]).
//!
//! They follow the active [`PrismTheme`] (Inter, neutrals, accent) and work with the classic
//! presets too.

use egui::text::{LayoutJob, TextFormat};
use egui::{
    Color32, CornerRadius, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2,
    Widget, WidgetInfo, pos2, vec2,
};

use super::prism_paint::with_alpha;
use crate::icons::Icon;
use crate::theme::{PrismTheme, ThemeMode};
use crate::typography::{FontWeight, prism_font, prism_numeric_font};

/// Selection overlay used by list rows: a light accent tint that works on solid and glass.
pub(crate) fn selection_tint(theme: &PrismTheme) -> Color32 {
    let alpha = if theme.mode == ThemeMode::Dark {
        34
    } else {
        26
    };
    with_alpha(theme.palette.accent, alpha)
}

fn row_radius(theme: &PrismTheme) -> f32 {
    (theme.radius.md * theme.radius_scale).clamp(4.0, 12.0)
}

// ---------------------------------------------------------------------------------------------
// GlassListRow
// ---------------------------------------------------------------------------------------------

/// A full-width clickable list row (scenes, sources, files, channels) with hover and selection
/// states and an optional 3 pt selection marker on the leading edge.
///
/// Use the builder for the common layout (icon, title, subtitle, trailing text) with
/// `ui.add(...)`, or [`GlassListRow::show`] to lay out your own contents; widgets added there
/// sit on top of the row and keep their own interaction (icon buttons, toggles).
///
/// ```no_run
/// # use egui_prismglass::{GlassListRow, Icon};
/// # fn demo(ui: &mut egui::Ui, selected: &mut usize) {
/// for (i, name) in ["Main camera", "Screen", "Overlay"].iter().enumerate() {
///     let row = GlassListRow::new()
///         .icon(Icon::Camera)
///         .title(*name)
///         .trailing("1080p")
///         .selected(*selected == i);
///     if ui.add(row).clicked() {
///         *selected = i;
///     }
/// }
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GlassListRow {
    selected: bool,
    height: Option<f32>,
    marker: Option<Color32>,
    hide_marker: bool,
    icon: Option<Icon>,
    title: Option<String>,
    subtitle: Option<String>,
    trailing: Option<String>,
}

impl Default for GlassListRow {
    fn default() -> Self {
        Self::new()
    }
}

impl GlassListRow {
    /// Creates an empty row (36 pt tall, 48 pt with a subtitle).
    pub fn new() -> Self {
        Self {
            selected: false,
            height: None,
            marker: None,
            hide_marker: false,
            icon: None,
            title: None,
            subtitle: None,
            trailing: None,
        }
    }

    /// Marks the row as selected (accent tint plus the leading marker).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Overrides the row height in points.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height.max(16.0));
        self
    }

    /// Sets the selection marker color, or `None` to hide it (default: the accent).
    pub fn marker(mut self, color: Option<Color32>) -> Self {
        self.hide_marker = color.is_none();
        self.marker = color;
        self
    }

    /// Adds a leading icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the primary text.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets a second, muted line under the title.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets right-aligned muted text (tabular figures), e.g. a count, size or duration.
    pub fn trailing(mut self, trailing: impl Into<String>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    fn resolved_height(&self) -> f32 {
        self.height
            .unwrap_or(if self.subtitle.is_some() { 48.0 } else { 36.0 })
    }

    /// Allocates and paints the row background, returning its rect and response.
    fn paint_background(&self, ui: &mut Ui, theme: &PrismTheme) -> (Rect, Response) {
        let height = self.resolved_height();
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
        let label = self.title.clone().unwrap_or_default();
        let selected = self.selected;
        response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, label.clone()));
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if ui.is_rect_visible(rect) {
            let radius = row_radius(theme);
            let cr = CornerRadius::same(radius.round() as u8);
            if self.selected {
                ui.painter().rect_filled(rect, cr, selection_tint(theme));
                if !self.hide_marker {
                    let color = self.marker.unwrap_or(theme.palette.accent);
                    let inset = (rect.height() * 0.22).clamp(5.0, 10.0);
                    let bar = Rect::from_min_size(
                        pos2(rect.left() + 1.0, rect.top() + inset),
                        vec2(3.0, rect.height() - inset * 2.0),
                    );
                    ui.painter().rect_filled(bar, 2.0, color);
                }
            } else if response.hovered() {
                ui.painter().rect_filled(rect, cr, theme.neutrals.hover);
            }
        }
        (rect, response)
    }

    /// Paints the row and lays out `add_contents` inside it (left to right, vertically
    /// centered, with 10 pt side padding). Returns the row's response.
    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let (rect, response) = self.paint_background(ui, &theme);
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(rect.shrink2(vec2(10.0, 0.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        add_contents(&mut child);
        response
    }
}

impl Widget for GlassListRow {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let (rect, response) = self.paint_background(ui, &theme);
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let painter = ui.painter();
        let ctx = ui.ctx();
        let hovered = response.hovered();
        let strong = self.selected || hovered;
        let mut x = rect.left() + 12.0;
        if let Some(icon) = self.icon {
            let icon_rect =
                Rect::from_center_size(pos2(x + 8.0, rect.center().y), Vec2::splat(15.0));
            let color = if self.selected {
                theme.palette.accent
            } else if strong {
                theme.palette.text_primary
            } else {
                theme.palette.text_secondary
            };
            icon.paint(painter, icon_rect, color);
            x += 26.0;
        }

        let trailing = self.trailing.as_ref().map(|t| {
            painter.layout_no_wrap(
                t.clone(),
                prism_numeric_font(ctx, 12.0, FontWeight::Regular),
                theme.palette.text_muted,
            )
        });
        let right = rect.right() - 12.0 - trailing.as_ref().map_or(0.0, |g| g.size().x + 10.0);
        let max_w = (right - x).max(0.0);

        let title = self.title.as_ref().map(|t| {
            painter.layout(
                t.clone(),
                prism_font(ctx, 13.0, FontWeight::Medium),
                theme.palette.text_primary,
                max_w,
            )
        });
        let subtitle = self.subtitle.as_ref().map(|t| {
            painter.layout(
                t.clone(),
                prism_font(ctx, 11.5, FontWeight::Regular),
                theme.palette.text_muted,
                max_w,
            )
        });
        let title_h = title.as_ref().map_or(0.0, |g| g.size().y);
        let sub_h = subtitle.as_ref().map_or(0.0, |g| g.size().y + 1.0);
        let mut y = rect.center().y - f32::midpoint(title_h, sub_h);
        let clip = painter.with_clip_rect(Rect::from_x_y_ranges(x..=right, rect.y_range()));
        if let Some(g) = title {
            let color = if strong {
                theme.palette.text_primary
            } else {
                blend(theme.palette.text_primary, theme.palette.text_secondary)
            };
            clip.galley_with_override_text_color(pos2(x, y), g, color);
            y += title_h + 1.0;
        }
        if let Some(g) = subtitle {
            clip.galley(pos2(x, y), g, theme.palette.text_muted);
        }
        if let Some(g) = trailing {
            let pos = pos2(
                rect.right() - 12.0 - g.size().x,
                rect.center().y - g.size().y * 0.5,
            );
            painter.galley(pos, g, theme.palette.text_muted);
        }
        response
    }
}

fn blend(a: Color32, b: Color32) -> Color32 {
    crate::tokens::blend_colors(a, b, 0.35)
}

// ---------------------------------------------------------------------------------------------
// GlassChip
// ---------------------------------------------------------------------------------------------

/// A compact tinted chip for categories and metadata (source type, platform, resolution).
///
/// Uppercase by default, in Inter Semibold with a light tint of its color.
///
/// ```no_run
/// # use egui_prismglass::GlassChip;
/// # fn demo(ui: &mut egui::Ui) {
/// ui.add(GlassChip::new("RTMP"));
/// ui.add(GlassChip::new("Live").color(egui::Color32::from_rgb(220, 38, 38)));
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GlassChip {
    text: String,
    color: Option<Color32>,
    icon: Option<Icon>,
    uppercase: bool,
    selected: Option<bool>,
}

impl GlassChip {
    /// Creates a chip with the theme's secondary text color.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            icon: None,
            uppercase: true,
            selected: None,
        }
    }

    /// Sets the chip color (text, icon and tint).
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Adds a small leading icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Keeps the original letter case (default: uppercase).
    pub fn uppercase(mut self, uppercase: bool) -> Self {
        self.uppercase = uppercase;
        self
    }

    /// Makes the chip a clickable filter toggle: `true` uses the accent, `false` stays neutral.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }
}

impl Widget for GlassChip {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let color = match (self.color, self.selected) {
            (Some(c), _) => c,
            (None, Some(true)) => theme.palette.accent,
            _ => theme.palette.text_secondary,
        };
        let text = if self.uppercase {
            self.text.to_uppercase()
        } else {
            self.text.clone()
        };
        let mut job = LayoutJob::default();
        job.append(
            &text,
            0.0,
            TextFormat {
                font_id: prism_font(
                    ui.ctx(),
                    if self.uppercase { 9.5 } else { 11.0 },
                    FontWeight::Semibold,
                ),
                color,
                extra_letter_spacing: if self.uppercase { 0.4 } else { 0.0 },
                ..Default::default()
            },
        );
        let galley = ui.painter().layout_job(job);
        let icon_w = if self.icon.is_some() { 14.0 } else { 0.0 };
        let height = if self.uppercase { 18.0 } else { 20.0 };
        let size = vec2(galley.size().x + icon_w + 12.0, height);
        let sense = if self.selected.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(size, sense);
        let label = self.text.clone();
        let selected = self.selected;
        response.widget_info(|| match selected {
            Some(on) => WidgetInfo::selected(Role::Button, true, on, label.clone()),
            None => WidgetInfo::labeled(Role::Label, true, label.clone()),
        });
        if self.selected.is_some() && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if ui.is_rect_visible(rect) {
            let dark = theme.mode == ThemeMode::Dark;
            let mut alpha: u8 = if dark { 30 } else { 24 };
            if self.selected == Some(false) {
                alpha = if dark { 18 } else { 14 };
            }
            if response.hovered() && self.selected.is_some() {
                alpha = alpha.saturating_add(14);
            }
            let cr = CornerRadius::same(5);
            ui.painter().rect_filled(rect, cr, with_alpha(color, alpha));
            if self.selected == Some(true) {
                ui.painter().rect_stroke(
                    rect,
                    cr,
                    Stroke::new(1.0, with_alpha(color, 90)),
                    StrokeKind::Inside,
                );
            }
            let content_w = galley.size().x + icon_w;
            let mut x = rect.center().x - content_w * 0.5;
            if let Some(icon) = self.icon {
                icon.paint(
                    ui.painter(),
                    Rect::from_min_size(pos2(x, rect.center().y - 5.5), Vec2::splat(11.0)),
                    color,
                );
                x += icon_w;
            }
            ui.painter().galley(
                pos2(x, rect.center().y - galley.size().y * 0.5),
                galley,
                color,
            );
        }
        response
    }
}

// ---------------------------------------------------------------------------------------------
// GlassKbd
// ---------------------------------------------------------------------------------------------

/// A keyboard key cap (`Ctrl`, `K`, `Del`) for shortcut hints, or a sequence of them.
///
/// ```no_run
/// # use egui_prismglass::GlassKbd;
/// # fn demo(ui: &mut egui::Ui) {
/// ui.add(GlassKbd::new("Esc"));
/// ui.add(GlassKbd::combo(["Ctrl", "Shift", "P"]));
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GlassKbd {
    keys: Vec<String>,
}

impl GlassKbd {
    /// A single key.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            keys: vec![key.into()],
        }
    }

    /// Several keys pressed together, drawn as separate caps.
    pub fn combo<S: Into<String>>(keys: impl IntoIterator<Item = S>) -> Self {
        Self {
            keys: keys.into_iter().map(Into::into).collect(),
        }
    }
}

impl Widget for GlassKbd {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let font = prism_font(ui.ctx(), 10.5, FontWeight::Medium);
        let color = theme.palette.text_secondary;
        let galleys: Vec<_> = self
            .keys
            .iter()
            .map(|k| ui.painter().layout_no_wrap(k.clone(), font.clone(), color))
            .collect();
        let gap = 3.0;
        let widths: Vec<f32> = galleys
            .iter()
            .map(|g| (g.size().x + 10.0).max(18.0))
            .collect();
        let total = widths.iter().sum::<f32>() + gap * (widths.len().saturating_sub(1)) as f32;
        let (rect, response) = ui.allocate_exact_size(vec2(total, 18.0), Sense::hover());
        let label = self.keys.join("+");
        response.widget_info(|| WidgetInfo::labeled(Role::Label, true, label.clone()));
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let mut x = rect.left();
            for (g, w) in galleys.into_iter().zip(widths) {
                let cap = Rect::from_min_size(pos2(x, rect.top()), vec2(w, rect.height()));
                let cr = CornerRadius::same(4);
                painter.rect_filled(cap, cr, theme.neutrals.raised);
                painter.rect_stroke(
                    cap,
                    cr,
                    Stroke::new(1.0, theme.neutrals.border_strong),
                    StrokeKind::Inside,
                );
                // Key caps sit on a slightly darker bottom edge.
                painter.hline(
                    cap.x_range().shrink(3.0),
                    cap.bottom() - 0.5,
                    Stroke::new(1.0, theme.neutrals.border_strong),
                );
                painter.galley(cap.center() - g.size() * 0.5 - vec2(0.0, 0.5), g, color);
                x += w + gap;
            }
        }
        response
    }
}

// ---------------------------------------------------------------------------------------------
// GlassEmptyState
// ---------------------------------------------------------------------------------------------

/// A centered empty state: an icon in a soft circle, a title and a short description,
/// with optional actions underneath ([`GlassEmptyState::show`]).
///
/// ```no_run
/// # use egui_prismglass::{GlassButton, GlassEmptyState, Icon};
/// # fn demo(ui: &mut egui::Ui) {
/// GlassEmptyState::new(Icon::Layers, "No sources yet")
///     .description("Add a camera, a window or an image to get started.")
///     .show(ui, |ui| {
///         ui.add(GlassButton::primary("Add source").icon(Icon::Plus));
///     });
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GlassEmptyState {
    icon: Icon,
    title: String,
    description: Option<String>,
}

impl GlassEmptyState {
    /// Creates an empty state with an icon and a title.
    pub fn new(icon: Icon, title: impl Into<String>) -> Self {
        Self {
            icon,
            title: title.into(),
            description: None,
        }
    }

    /// Adds a muted description under the title.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Paints the empty state and lays out `add_actions` centered below it.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_actions: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let theme = PrismTheme::from_ctx(ui.ctx());
        ui.vertical_centered(|ui| {
            self.paint_body(ui, &theme);
            ui.add_space(4.0);
            let inner = ui
                .horizontal(|ui| {
                    // `vertical_centered` only centers single widgets; center the row by hand.
                    let row_w = ui
                        .ctx()
                        .data(|d| d.get_temp::<f32>(ui.unique_id().with("empty_state_actions_w")));
                    if let Some(w) = row_w {
                        ui.add_space(((ui.available_width() - w) * 0.5).max(0.0));
                    }
                    let start = ui.cursor().min.x;
                    let r = add_actions(ui);
                    let w = ui.min_rect().right() - start;
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(ui.unique_id().with("empty_state_actions_w"), w)
                    });
                    r
                })
                .inner;
            ui.add_space(8.0);
            inner
        })
    }

    fn paint_body(&self, ui: &mut Ui, theme: &PrismTheme) {
        ui.add_space(10.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
        if ui.is_rect_visible(rect) {
            let fill = if theme.mode == ThemeMode::Dark {
                Color32::from_rgba_unmultiplied(255, 255, 255, 14)
            } else {
                Color32::from_rgba_unmultiplied(0, 0, 0, 12)
            };
            ui.painter().circle_filled(rect.center(), 20.0, fill);
            self.icon.paint(
                ui.painter(),
                Rect::from_center_size(rect.center(), Vec2::splat(18.0)),
                theme.palette.text_muted,
            );
        }
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(&self.title)
                .font(prism_font(ui.ctx(), 13.5, FontWeight::Semibold))
                .color(theme.palette.text_primary),
        );
        if let Some(desc) = &self.description {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(desc)
                    .font(prism_font(ui.ctx(), 12.0, FontWeight::Regular))
                    .color(theme.palette.text_muted),
            );
        }
    }
}

impl Widget for GlassEmptyState {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        ui.vertical_centered(|ui| {
            self.paint_body(ui, &theme);
            ui.add_space(8.0);
        })
        .response
    }
}

// ---------------------------------------------------------------------------------------------
// SectionHeader
// ---------------------------------------------------------------------------------------------

/// Builds the uppercase, letter-spaced "eyebrow" label used by [`SectionHeader`] (Inter
/// Semibold 10.5 pt, tracking +0.7) so you can reuse it in custom layouts.
pub fn eyebrow_text(ctx: &egui::Context, text: &str, color: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.append(
        &text.to_uppercase(),
        0.0,
        TextFormat {
            font_id: prism_font(ctx, 10.5, FontWeight::Semibold),
            color,
            extra_letter_spacing: 0.7,
            ..Default::default()
        },
    );
    job
}

/// A quiet section header: an uppercase tracked title, an optional count and right-aligned
/// actions ([`SectionHeader::show`]).
///
/// ```no_run
/// # use egui_prismglass::{GlassButton, Icon, SectionHeader};
/// # fn demo(ui: &mut egui::Ui) {
/// ui.add(SectionHeader::new("Scenes").count(4));
/// SectionHeader::new("Sources").show(ui, |ui| {
///     ui.add(GlassButton::icon_only(Icon::Plus)).on_hover_text("Add source");
/// });
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SectionHeader {
    title: String,
    count: Option<u32>,
    color: Option<Color32>,
}

impl SectionHeader {
    /// Creates a header with the given title (rendered uppercase).
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            count: None,
            color: None,
        }
    }

    /// Shows a muted count after the title.
    pub fn count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    /// Overrides the title color (default: muted text).
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Lays the header out across the available width, with `actions` right-aligned.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        actions: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let response = ui.horizontal(|ui| {
            ui.set_min_height(26.0);
            self.paint_title(ui, &theme);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                actions(ui)
            })
            .inner
        });
        ui.add_space(2.0);
        response
    }

    fn paint_title(&self, ui: &mut Ui, theme: &PrismTheme) {
        let color = self.color.unwrap_or(theme.palette.text_muted);
        ui.label(eyebrow_text(ui.ctx(), &self.title, color));
        if let Some(count) = self.count {
            ui.label(
                egui::RichText::new(count.to_string())
                    .font(prism_numeric_font(ui.ctx(), 10.5, FontWeight::Medium))
                    .color(theme.palette.text_muted),
            );
        }
    }
}

impl Widget for SectionHeader {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        ui.horizontal(|ui| {
            ui.set_min_height(26.0);
            self.paint_title(ui, &theme);
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PrismPreset;

    #[test]
    fn kit_components_render_in_every_preset() {
        for preset in PrismPreset::PRISM.into_iter().chain(PrismPreset::CLASSIC) {
            let ctx = egui::Context::default();
            preset.into_theme().apply(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui.add(
                    GlassListRow::new()
                        .icon(Icon::Camera)
                        .title("Main camera")
                        .subtitle("1920×1080 · 60 fps")
                        .trailing("12")
                        .selected(true),
                );
                ui.add(GlassListRow::new().title("Plain"));
                GlassListRow::new().show(ui, |ui| {
                    ui.label("custom");
                });
                ui.add(GlassChip::new("rtmp"));
                ui.add(GlassChip::new("Filter").selected(true).icon(Icon::Filter));
                ui.add(GlassKbd::combo(["Ctrl", "K"]));
                ui.add(GlassEmptyState::new(Icon::Folder, "Empty").description("Nothing here."));
                GlassEmptyState::new(Icon::Folder, "Empty").show(ui, |ui| ui.label("action"));
                ui.add(SectionHeader::new("Sources").count(3));
                SectionHeader::new("Scenes").show(ui, |ui| ui.label("+"));
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn list_rows_have_expected_height() {
        let ctx = egui::Context::default();
        let mut heights = Vec::new();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            heights.push(ui.add(GlassListRow::new().title("A")).rect.height());
            heights.push(
                ui.add(GlassListRow::new().title("A").subtitle("B"))
                    .rect
                    .height(),
            );
            heights.push(ui.add(GlassListRow::new().height(60.0)).rect.height());
        });
        out.textures_delta.clear();
        assert_eq!(heights, vec![36.0, 48.0, 60.0]);
    }

    #[test]
    fn eyebrow_is_uppercase_with_tracking() {
        let ctx = egui::Context::default();
        let job = eyebrow_text(&ctx, "Scenes", Color32::WHITE);
        assert_eq!(job.text, "SCENES");
        assert!((job.sections[0].format.extra_letter_spacing - 0.7).abs() < f32::EPSILON);
    }
}
