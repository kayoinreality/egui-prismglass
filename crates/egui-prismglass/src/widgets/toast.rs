//! Toast notifications ([`GlassToasts`], [`Toast`]).

use egui::{
    Align2, Area, Color32, Context, CornerRadius, Id, Margin, Order, Rect, Sense, Ui, Vec2, pos2,
    vec2,
};

use super::overlay::{floating_tokens, glass_layer};
use crate::icons::Icon;
use crate::theme::PrismTheme;
use crate::typography::{FontWeight, prism_font};

/// Semantic style of a [`Toast`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ToastVariant {
    /// Neutral information (accent color, info icon).
    #[default]
    Info,
    /// Something finished well (success color, check icon).
    Success,
    /// Needs attention (warning color, triangle icon).
    Warning,
    /// Something failed (danger color, alert icon).
    Danger,
}

impl ToastVariant {
    /// The icon shown at the start of the toast.
    pub fn icon(self) -> Icon {
        match self {
            Self::Info => Icon::Info,
            Self::Success => Icon::CheckCircle,
            Self::Warning => Icon::Warning,
            Self::Danger => Icon::AlertCircle,
        }
    }

    /// The variant color in `theme`.
    pub fn color(self, theme: &PrismTheme) -> Color32 {
        match self {
            Self::Info => theme.palette.accent,
            Self::Success => theme.palette.success,
            Self::Warning => theme.palette.warning,
            Self::Danger => theme.palette.danger,
        }
    }
}

/// Where toasts stack on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ToastAnchor {
    /// Top-right corner, stacking down.
    TopRight,
    /// Top-left corner, stacking down.
    TopLeft,
    /// Top edge, centered, stacking down.
    TopCenter,
    /// Bottom-right corner, stacking up (default).
    #[default]
    BottomRight,
    /// Bottom-left corner, stacking up.
    BottomLeft,
    /// Bottom edge, centered, stacking up.
    BottomCenter,
}

impl ToastAnchor {
    fn align(self) -> Align2 {
        match self {
            Self::TopRight => Align2::RIGHT_TOP,
            Self::TopLeft => Align2::LEFT_TOP,
            Self::TopCenter => Align2::CENTER_TOP,
            Self::BottomRight => Align2::RIGHT_BOTTOM,
            Self::BottomLeft => Align2::LEFT_BOTTOM,
            Self::BottomCenter => Align2::CENTER_BOTTOM,
        }
    }

    fn is_top(self) -> bool {
        matches!(self, Self::TopRight | Self::TopLeft | Self::TopCenter)
    }
}

/// Identifies a toast in a [`GlassToasts`] queue (returned by [`GlassToasts::add`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToastId(u64);

/// One notification: a variant, a title, optional body text and a lifetime.
#[derive(Debug, Clone)]
pub struct Toast {
    variant: ToastVariant,
    title: String,
    body: Option<String>,
    duration: Option<f32>,
    closable: bool,
    // Runtime state, managed by `GlassToasts`.
    id: u64,
    elapsed: f32,
    appear: f32,
    leaving: Option<f32>,
    height: f32,
}

impl Toast {
    /// A toast that disappears after 4 s.
    pub fn new(variant: ToastVariant, title: impl Into<String>) -> Self {
        Self {
            variant,
            title: title.into(),
            body: None,
            duration: Some(4.0),
            closable: true,
            id: 0,
            elapsed: 0.0,
            appear: 0.0,
            leaving: None,
            height: 56.0,
        }
    }

    /// An [`ToastVariant::Info`] toast.
    pub fn info(title: impl Into<String>) -> Self {
        Self::new(ToastVariant::Info, title)
    }

    /// A [`ToastVariant::Success`] toast.
    pub fn success(title: impl Into<String>) -> Self {
        Self::new(ToastVariant::Success, title)
    }

    /// A [`ToastVariant::Warning`] toast.
    pub fn warning(title: impl Into<String>) -> Self {
        Self::new(ToastVariant::Warning, title)
    }

    /// A [`ToastVariant::Danger`] toast.
    pub fn danger(title: impl Into<String>) -> Self {
        Self::new(ToastVariant::Danger, title)
    }

    /// Secondary text under the title.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// How long the toast stays once visible, in seconds (default `4`). Hovering pauses it.
    pub fn duration(mut self, seconds: f32) -> Self {
        self.duration = Some(seconds.max(0.1));
        self
    }

    /// Keeps the toast until it is closed (no auto-dismiss, no progress bar).
    pub fn sticky(mut self) -> Self {
        self.duration = None;
        self
    }

    /// Whether to show the × button (default `true`).
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// The variant.
    pub fn variant(&self) -> ToastVariant {
        self.variant
    }

    /// The title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Remaining lifetime as a fraction (`1` = just shown, `0` = expired), or `None` if sticky.
    pub fn remaining_fraction(&self) -> Option<f32> {
        self.duration
            .map(|d| (1.0 - self.elapsed / d).clamp(0.0, 1.0))
    }
}

const APPEAR_SECS: f32 = 0.18;
const LEAVE_SECS: f32 = 0.16;

/// A queue of toast notifications, shown stacked in a screen corner.
///
/// Keep one in your app state, push toasts from anywhere, and call [`GlassToasts::show`] once
/// per frame (after the rest of your UI). Up to [`GlassToasts::max_visible`] toasts are on
/// screen at once; the rest wait in the queue and only start their timers once visible.
/// Hovering a toast pauses its timer.
///
/// ```no_run
/// # use egui_prismglass::{GlassToasts, Toast};
/// # fn demo(ctx: &egui::Context, toasts: &mut GlassToasts, saved: bool) {
/// if saved {
///     toasts.add(Toast::success("Scene saved").body("Intro · 4 sources"));
/// }
/// toasts.add(Toast::danger("Stream disconnected").sticky());
/// toasts.show(ctx);
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GlassToasts {
    toasts: Vec<Toast>,
    next_id: u64,
    anchor: ToastAnchor,
    max_visible: usize,
    width: f32,
    margin: Vec2,
    spacing: f32,
    reduce_motion: bool,
    last_time: Option<f64>,
}

impl Default for GlassToasts {
    fn default() -> Self {
        Self::new()
    }
}

impl GlassToasts {
    /// An empty queue anchored bottom-right, showing up to 4 toasts 340 pt wide.
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 1,
            anchor: ToastAnchor::default(),
            max_visible: 4,
            width: 340.0,
            margin: vec2(16.0, 16.0),
            spacing: 8.0,
            reduce_motion: false,
            last_time: None,
        }
    }

    /// Sets the screen corner or edge (default bottom-right).
    pub fn anchor(mut self, anchor: ToastAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// How many toasts can be visible at once (default `4`, at least `1`).
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max.max(1);
        self
    }

    /// Toast width in points (default `340`).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(160.0);
        self
    }

    /// Distance from the screen edges (default `16 × 16`).
    pub fn margin(mut self, margin: Vec2) -> Self {
        self.margin = margin;
        self
    }

    /// Shows and hides toasts without fade / slide animations (for users who prefer reduced
    /// motion, and for deterministic screenshots).
    pub fn reduce_motion(mut self, reduce: bool) -> Self {
        self.reduce_motion = reduce;
        self
    }

    /// Queues a toast and returns its id.
    pub fn add(&mut self, mut toast: Toast) -> ToastId {
        toast.id = self.next_id;
        self.next_id += 1;
        let id = ToastId(toast.id);
        self.toasts.push(toast);
        id
    }

    /// Queues an info toast.
    pub fn info(&mut self, title: impl Into<String>) -> ToastId {
        self.add(Toast::info(title))
    }

    /// Queues a success toast.
    pub fn success(&mut self, title: impl Into<String>) -> ToastId {
        self.add(Toast::success(title))
    }

    /// Queues a warning toast.
    pub fn warning(&mut self, title: impl Into<String>) -> ToastId {
        self.add(Toast::warning(title))
    }

    /// Queues a danger toast.
    pub fn danger(&mut self, title: impl Into<String>) -> ToastId {
        self.add(Toast::danger(title))
    }

    /// Starts closing the toast (it animates out on the next frames).
    pub fn dismiss(&mut self, id: ToastId) {
        if let Some(t) = self.toasts.iter_mut().find(|t| t.id == id.0) {
            t.leaving.get_or_insert(0.0);
        }
    }

    /// Removes every toast immediately.
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Number of toasts, visible or queued.
    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    /// `true` when there are no toasts.
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    /// The toasts in queue order (the first [`Self::max_visible`] are on screen).
    pub fn iter(&self) -> impl Iterator<Item = &Toast> {
        self.toasts.iter()
    }

    /// Advances timers and paints the visible toasts. Call once per frame.
    pub fn show(&mut self, ctx: &Context) {
        let now = ctx.input(|i| i.time);
        let dt = self
            .last_time
            .map_or(0.0, |t| (now - t).clamp(0.0, 0.25) as f32);
        self.last_time = Some(now);
        if self.toasts.is_empty() {
            return;
        }
        let theme = PrismTheme::from_ctx(ctx);
        let pointer = ctx.pointer_hover_pos();
        let align = self.anchor.align();
        let is_top = self.anchor.is_top();
        let screen = ctx.content_rect();
        let width = self
            .width
            .min(screen.width() - self.margin.x * 2.0)
            .max(120.0);

        let mut offset = 0.0_f32;
        let visible = self.toasts.len().min(self.max_visible);
        for toast in self.toasts.iter_mut().take(visible) {
            // Timers: appear → live (paused while hovered) → leaving.
            toast.appear = if self.reduce_motion {
                1.0
            } else {
                (toast.appear + dt / APPEAR_SECS).min(1.0)
            };
            let pos_y = if is_top {
                screen.top() + self.margin.y + offset
            } else {
                screen.bottom() - self.margin.y - offset - toast.height
            };
            let x = match align.x() {
                egui::Align::Min => screen.left() + self.margin.x,
                egui::Align::Center => screen.center().x - width * 0.5,
                egui::Align::Max => screen.right() - self.margin.x - width,
            };
            let rect = Rect::from_min_size(pos2(x, pos_y), vec2(width, toast.height));
            let hovered = pointer.is_some_and(|p| rect.contains(p));
            if let Some(l) = toast.leaving.as_mut() {
                *l = if self.reduce_motion {
                    1.0
                } else {
                    *l + dt / LEAVE_SECS
                };
            } else if let Some(d) = toast.duration {
                if !hovered {
                    toast.elapsed += dt;
                }
                if toast.elapsed >= d {
                    toast.leaving = Some(0.0);
                }
            }

            let fade = toast.appear * (1.0 - toast.leaving.unwrap_or(0.0)).clamp(0.0, 1.0);
            let slide = (1.0 - fade) * 10.0 * if is_top { -1.0 } else { 1.0 };
            let area = Area::new(Id::NULL.with(("prism_toast", toast.id)))
                .order(Order::Foreground)
                .fixed_pos(pos2(x, pos_y + slide))
                .interactable(true)
                .sense(Sense::hover())
                .show(ctx, |ui| {
                    ui.set_opacity(fade);
                    ui.set_width(width);
                    paint_toast(ui, toast, &theme, width)
                });
            if area.inner {
                toast.leaving.get_or_insert(0.0);
            }
            toast.height = area.response.rect.height().max(1.0);
            offset += (toast.height + self.spacing) * fade.max(0.2);
        }
        self.toasts.retain(|t| t.leaving.is_none_or(|l| l < 1.0));
        if !self.toasts.is_empty() {
            ctx.request_repaint();
        }
    }
}

/// Paints one toast; returns `true` if its close button was clicked.
fn paint_toast(ui: &mut Ui, toast: &Toast, theme: &PrismTheme, width: f32) -> bool {
    let color = toast.variant.color(theme);
    let tokens = floating_tokens(theme, Some(theme.radius.md + 2.0));
    let mut close = false;
    let inner_w = width - 28.0;
    glass_layer(ui, Margin::symmetric(14, 12), &tokens, |ui| {
        ui.set_width(inner_w);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
            toast.variant.icon().paint(ui.painter(), icon_rect, color);
            let close_w = if toast.closable { 26.0 } else { 0.0 };
            ui.vertical(|ui| {
                ui.set_width((ui.available_width() - close_w).max(40.0));
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(
                    egui::RichText::new(&toast.title)
                        .font(prism_font(ui.ctx(), 13.0, FontWeight::Semibold))
                        .color(theme.palette.text_primary),
                );
                if let Some(body) = &toast.body {
                    ui.label(
                        egui::RichText::new(body)
                            .font(prism_font(ui.ctx(), 12.0, FontWeight::Regular))
                            .color(theme.palette.text_secondary),
                    );
                }
            });
            if toast.closable {
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
                response
                    .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Dismiss"));
                let c = if response.hovered() {
                    theme.palette.text_primary
                } else {
                    theme.palette.text_muted
                };
                Icon::Close.paint(
                    ui.painter(),
                    Rect::from_center_size(rect.center(), Vec2::splat(10.0)),
                    c,
                );
                close = response.clicked();
            }
        });
    });
    if let Some(frac) = toast.remaining_fraction() {
        // Progress bar along the bottom inner edge.
        let r = ui.min_rect();
        let radius = theme.radius.md + 2.0;
        let track = Rect::from_min_max(
            pos2(r.left() + radius, r.bottom() - 3.0),
            pos2(r.right() - radius, r.bottom() - 1.5),
        );
        let bar = Rect::from_min_max(
            track.min,
            pos2(track.left() + track.width() * frac, track.bottom()),
        );
        ui.painter().rect_filled(
            track,
            CornerRadius::same(1),
            super::prism_paint::with_alpha(color, 36),
        );
        ui.painter().rect_filled(bar, CornerRadius::same(1), color);
    }
    close
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(ctx: &Context, toasts: &mut GlassToasts, time: f64) {
        let input = egui::RawInput {
            time: Some(time),
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| toasts.show(ui.ctx()));
        out.textures_delta.clear();
    }

    #[test]
    fn toasts_auto_dismiss_after_their_duration() {
        let ctx = egui::Context::default();
        PrismTheme::prism_dark().apply(&ctx);
        let mut toasts = GlassToasts::new();
        toasts.add(Toast::success("Saved").duration(1.0));
        toasts.add(Toast::warning("Sticky").sticky());
        let mut t = 0.0;
        while t < 1.6 {
            frame(&ctx, &mut toasts, t);
            t += 0.05;
        }
        assert_eq!(
            toasts.len(),
            1,
            "the timed toast is gone, the sticky one stays"
        );
        assert_eq!(toasts.iter().next().unwrap().title(), "Sticky");
    }

    #[test]
    fn queued_toasts_wait_until_visible() {
        let ctx = egui::Context::default();
        let mut toasts = GlassToasts::new().max_visible(2);
        for i in 0..4 {
            toasts.add(Toast::info(format!("#{i}")).duration(1.0));
        }
        frame(&ctx, &mut toasts, 0.0);
        frame(&ctx, &mut toasts, 0.5);
        let remaining: Vec<_> = toasts.iter().map(Toast::remaining_fraction).collect();
        assert!(remaining[0].unwrap() < 1.0);
        assert!(remaining[1].unwrap() < 1.0);
        assert_eq!(
            remaining[2],
            Some(1.0),
            "queued toasts keep their full time"
        );
        assert_eq!(remaining[3], Some(1.0));
    }

    #[test]
    fn dismiss_removes_after_exit_animation() {
        let ctx = egui::Context::default();
        let mut toasts = GlassToasts::new().anchor(ToastAnchor::TopCenter);
        let id = toasts.add(Toast::danger("Failed").body("Network error").sticky());
        frame(&ctx, &mut toasts, 0.0);
        toasts.dismiss(id);
        frame(&ctx, &mut toasts, 0.1);
        frame(&ctx, &mut toasts, 0.2);
        frame(&ctx, &mut toasts, 0.3);
        assert!(toasts.is_empty());
    }

    #[test]
    fn variants_map_to_icons_and_theme_colors() {
        let theme = PrismTheme::prism_light();
        assert_eq!(ToastVariant::Success.icon(), Icon::CheckCircle);
        assert_eq!(ToastVariant::Danger.color(&theme), theme.palette.danger);
        assert_eq!(ToastVariant::Info.color(&theme), theme.palette.accent);
    }
}
