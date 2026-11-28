//! Vector icons painted with [`egui::Painter`] ([`Icon`], [`GlassIcon`]).
//!
//! Every icon is drawn from line segments, arcs and small fills on a 16 pt grid with a
//! 1.5 pt stroke, so it needs no icon font, stays aligned with Inter, scales with any DPI and
//! takes the color you give it (usually a theme text color).
//!
//! The set started as the icon module of the `PrismCast` Studio app and is shared by
//! [`crate::GlassButton::icon`], [`crate::SidebarItem::icon`], [`crate::GlassTextField`],
//! [`crate::MenuItem`], [`crate::GlassToasts`] and the other v0.4 widgets.
//!
//! ```no_run
//! use egui_prismglass::{GlassIcon, Icon};
//! # fn demo(ui: &mut egui::Ui, painter: &egui::Painter, rect: egui::Rect) {
//! ui.add(GlassIcon::new(Icon::Search).size(16.0));
//! Icon::Settings.paint(painter, rect, egui::Color32::WHITE);
//! # }
//! ```

use std::f32::consts::{PI, TAU};

use egui::{
    Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget, pos2, vec2,
};

use crate::theme::PrismTheme;
use crate::widgets::SidebarIcon;

/// A vector icon from the built-in set (see [`Icon::ALL`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Icon {
    /// A screen with a play triangle (studio / preview).
    Studio,
    /// A 2×2 grid of rounded tiles (widgets, apps, grid view).
    Widgets,
    /// A dot with radiating arcs (live, broadcast, wireless).
    Broadcast,
    /// Three connected nodes (network, graph, share).
    Network,
    /// A gear (settings, preferences).
    Settings,
    /// An open eye (visible, preview).
    Eye,
    /// A struck-through eye (hidden).
    EyeOff,
    /// A closed padlock (locked).
    Lock,
    /// An open padlock (unlocked).
    Unlock,
    /// A trash can (delete).
    Trash,
    /// An upward chevron.
    ChevronUp,
    /// A downward chevron (disclosure, dropdown).
    ChevronDown,
    /// A leftward chevron (back).
    ChevronLeft,
    /// A rightward chevron (forward, submenu).
    ChevronRight,
    /// A plus sign (add, new).
    Plus,
    /// A diagonal cross (close, dismiss).
    Close,
    /// A check mark (done, selected).
    Check,
    /// A speaker with sound waves.
    Volume,
    /// A muted speaker.
    VolumeMute,
    /// Three horizontal sliders (adjustments, filters).
    Sliders,
    /// A desktop monitor (display, screen capture).
    Monitor,
    /// An application window with title bar dots.
    Window,
    /// A framed landscape (image, media).
    Image,
    /// A capital T (text, typography).
    Text,
    /// A video camera.
    Camera,
    /// A globe with meridians (web, language).
    Globe,
    /// A half-filled circle (color, theme, tokens).
    Swatch,
    /// A filled circle (record).
    Record,
    /// A filled rounded square (stop).
    Stop,
    /// A circular arrow around a dot (replay buffer).
    Replay,
    /// Two vertical bars (pause).
    Pause,
    /// A portrait phone outline (vertical / 9:16 canvas).
    Vertical,
    /// Two side-by-side panes (split view).
    Columns,
    /// Two opposing arrows (swap, transition).
    Swap,
    /// A counter-clockwise arrow (undo).
    Undo,
    /// A clockwise arrow (refresh, reload).
    Refresh,
    /// A paper plane (send).
    Send,
    /// A sun (light mode).
    Sun,
    /// A crescent moon (dark mode).
    Moon,
    /// A speech bubble (chat, comments).
    Chat,
    /// Stacked layers.
    Layers,
    /// An arrow into a tray (download, export).
    Download,
    /// A key (credentials, stream key).
    Key,
    /// A microphone (audio input).
    Mic,
    /// A lightning bolt (performance, quick action).
    Bolt,
    /// A folder (files, projects).
    Folder,
    /// A magnifying glass (search).
    Search,
    /// An "i" in a circle (information).
    Info,
    /// An exclamation mark in a triangle (warning).
    Warning,
    /// An exclamation mark in a circle (error, danger).
    AlertCircle,
    /// A check mark in a circle (success).
    CheckCircle,
    /// A bulleted list.
    List,
    /// A clock face (history, schedule).
    Clock,
    /// A bar chart (analytics, stats).
    Chart,
    /// Three horizontal dots (more actions, overflow menu).
    More,
    /// Two overlapping sheets (copy, duplicate).
    Copy,
    /// A pencil (edit, rename).
    Edit,
    /// A funnel (filter).
    Filter,
    /// A filled play triangle (play, start).
    Play,
    /// A person silhouette (account, profile).
    User,
    /// A bell (notifications).
    Bell,
    /// A house (home, overview).
    Home,
    /// An arrow out of a tray (upload, import).
    Upload,
    /// Three horizontal lines (menu, navigation).
    Menu,
    /// A triangular prism (the egui-prismglass mark).
    Prism,
}

impl Icon {
    /// Every icon in the set, in declaration order.
    pub const ALL: [Self; 65] = [
        Self::Studio,
        Self::Widgets,
        Self::Broadcast,
        Self::Network,
        Self::Settings,
        Self::Eye,
        Self::EyeOff,
        Self::Lock,
        Self::Unlock,
        Self::Trash,
        Self::ChevronUp,
        Self::ChevronDown,
        Self::ChevronLeft,
        Self::ChevronRight,
        Self::Plus,
        Self::Close,
        Self::Check,
        Self::Volume,
        Self::VolumeMute,
        Self::Sliders,
        Self::Monitor,
        Self::Window,
        Self::Image,
        Self::Text,
        Self::Camera,
        Self::Globe,
        Self::Swatch,
        Self::Record,
        Self::Stop,
        Self::Replay,
        Self::Pause,
        Self::Vertical,
        Self::Columns,
        Self::Swap,
        Self::Undo,
        Self::Refresh,
        Self::Send,
        Self::Sun,
        Self::Moon,
        Self::Chat,
        Self::Layers,
        Self::Download,
        Self::Key,
        Self::Mic,
        Self::Bolt,
        Self::Folder,
        Self::Search,
        Self::Info,
        Self::Warning,
        Self::AlertCircle,
        Self::CheckCircle,
        Self::List,
        Self::Clock,
        Self::Chart,
        Self::More,
        Self::Copy,
        Self::Edit,
        Self::Filter,
        Self::Play,
        Self::User,
        Self::Bell,
        Self::Home,
        Self::Upload,
        Self::Menu,
        Self::Prism,
    ];

    /// A stable kebab-case name (`"eye-off"`, `"chevron-right"`), handy for search and docs.
    pub fn name(self) -> &'static str {
        match self {
            Self::Studio => "studio",
            Self::Widgets => "widgets",
            Self::Broadcast => "broadcast",
            Self::Network => "network",
            Self::Settings => "settings",
            Self::Eye => "eye",
            Self::EyeOff => "eye-off",
            Self::Lock => "lock",
            Self::Unlock => "unlock",
            Self::Trash => "trash",
            Self::ChevronUp => "chevron-up",
            Self::ChevronDown => "chevron-down",
            Self::ChevronLeft => "chevron-left",
            Self::ChevronRight => "chevron-right",
            Self::Plus => "plus",
            Self::Close => "close",
            Self::Check => "check",
            Self::Volume => "volume",
            Self::VolumeMute => "volume-mute",
            Self::Sliders => "sliders",
            Self::Monitor => "monitor",
            Self::Window => "window",
            Self::Image => "image",
            Self::Text => "text",
            Self::Camera => "camera",
            Self::Globe => "globe",
            Self::Swatch => "swatch",
            Self::Record => "record",
            Self::Stop => "stop",
            Self::Replay => "replay",
            Self::Pause => "pause",
            Self::Vertical => "vertical",
            Self::Columns => "columns",
            Self::Swap => "swap",
            Self::Undo => "undo",
            Self::Refresh => "refresh",
            Self::Send => "send",
            Self::Sun => "sun",
            Self::Moon => "moon",
            Self::Chat => "chat",
            Self::Layers => "layers",
            Self::Download => "download",
            Self::Key => "key",
            Self::Mic => "mic",
            Self::Bolt => "bolt",
            Self::Folder => "folder",
            Self::Search => "search",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::AlertCircle => "alert-circle",
            Self::CheckCircle => "check-circle",
            Self::List => "list",
            Self::Clock => "clock",
            Self::Chart => "chart",
            Self::More => "more",
            Self::Copy => "copy",
            Self::Edit => "edit",
            Self::Filter => "filter",
            Self::Play => "play",
            Self::User => "user",
            Self::Bell => "bell",
            Self::Home => "home",
            Self::Upload => "upload",
            Self::Menu => "menu",
            Self::Prism => "prism",
        }
    }

    /// Paints the icon centered in `rect` (it uses the shorter side) with `color`.
    ///
    /// Rects smaller than 1 pt are ignored.
    pub fn paint(self, painter: &Painter, rect: Rect, color: Color32) {
        paint(painter, rect, self, color);
    }
}

impl From<SidebarIcon> for Icon {
    fn from(icon: SidebarIcon) -> Self {
        match icon {
            SidebarIcon::Prism => Self::Prism,
            SidebarIcon::Grid => Self::Widgets,
            SidebarIcon::Tokens => Self::Swatch,
            SidebarIcon::Sliders => Self::Sliders,
            SidebarIcon::List => Self::List,
            SidebarIcon::Layers => Self::Layers,
            SidebarIcon::Clock => Self::Clock,
            SidebarIcon::Chart => Self::Chart,
            SidebarIcon::Folder => Self::Folder,
        }
    }
}

/// Paints `icon` centered in `rect` with `color` (same as [`Icon::paint`]).
pub fn paint_icon(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    paint(painter, rect, icon, color);
}

/// A standalone icon widget that allocates a square and paints an [`Icon`] in it.
///
/// The color defaults to the theme's secondary text color.
#[derive(Debug, Clone, Copy)]
pub struct GlassIcon {
    icon: Icon,
    size: f32,
    color: Option<Color32>,
}

impl GlassIcon {
    /// Creates a 16 pt icon.
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: 16.0,
            color: None,
        }
    }

    /// Sets the side of the square in points (default `16`).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }

    /// Overrides the color.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

impl Widget for GlassIcon {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        if ui.is_rect_visible(rect) {
            let color = self
                .color
                .unwrap_or_else(|| PrismTheme::from_ctx(ui.ctx()).palette.text_secondary);
            self.icon.paint(ui.painter(), rect, color);
        }
        response
    }
}

#[allow(clippy::many_single_char_names, clippy::similar_names)]
fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let s = rect.width().min(rect.height());
    if s <= 1.0 {
        return;
    }
    let c = rect.center();
    // Normalized coordinates: (-1..1) inside a 16 pt box with a 1.5 pt margin.
    let u = s / 2.0 * 0.82;
    let p = |x: f32, y: f32| pos2(c.x + x * u, c.y + y * u);
    let w = (s / 16.0 * 1.5).clamp(1.1, 2.4);
    let st = Stroke::new(w, color);

    let line = |pts: Vec<Pos2>| {
        painter.add(Shape::line(pts, st));
    };
    let closed = |pts: Vec<Pos2>| {
        painter.add(Shape::closed_line(pts, st));
    };
    let seg = |a: Pos2, b: Pos2| {
        painter.line_segment([a, b], st);
    };
    let rrect = |r: Rect, radius: f32| {
        painter.rect_stroke(r, radius, st, egui::StrokeKind::Middle);
    };
    let box_r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(p(x0, y0), p(x1, y1));

    match icon {
        Icon::Studio => {
            rrect(box_r(-0.95, -0.75, 0.95, 0.55), u * 0.18);
            seg(p(-0.4, 0.9), p(0.4, 0.9));
            painter.add(Shape::convex_polygon(
                vec![p(-0.22, -0.42), p(0.36, -0.1), p(-0.22, 0.22)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Widgets => {
            for (x, y) in [(-0.9, -0.9), (0.1, -0.9), (-0.9, 0.1), (0.1, 0.1)] {
                rrect(box_r(x, y, x + 0.8, y + 0.8), u * 0.16);
            }
        }
        Icon::Broadcast => {
            painter.circle_filled(p(0.0, 0.0), u * 0.17, color);
            line(arc(p(0.0, 0.0), u * 0.5, -PI * 0.28, PI * 0.28, 10));
            line(arc(p(0.0, 0.0), u * 0.5, PI * 0.72, PI * 1.28, 10));
            line(arc(p(0.0, 0.0), u * 0.9, -PI * 0.25, PI * 0.25, 12));
            line(arc(p(0.0, 0.0), u * 0.9, PI * 0.75, PI * 1.25, 12));
        }
        Icon::Network => {
            let nodes = [p(0.0, -0.7), p(-0.72, 0.6), p(0.72, 0.6)];
            seg(nodes[0], nodes[1]);
            seg(nodes[0], nodes[2]);
            seg(nodes[1], nodes[2]);
            for n in nodes {
                painter.circle_filled(n, u * 0.26, color);
            }
        }
        Icon::Settings => {
            painter.circle_stroke(p(0.0, 0.0), u * 0.3, st);
            for i in 0..8 {
                let a = i as f32 / 8.0 * TAU;
                let d = vec2(a.cos(), a.sin());
                seg(c + d * u * 0.58, c + d * u * 0.92);
            }
            painter.circle_stroke(p(0.0, 0.0), u * 0.62, st);
        }
        Icon::Eye | Icon::EyeOff => {
            let mut pts = arc(p(0.0, 0.9), u * 1.25, -PI * 0.78, -PI * 0.22, 14);
            pts.extend(arc(p(0.0, -0.9), u * 1.25, PI * 0.22, PI * 0.78, 14));
            closed(pts);
            painter.circle_stroke(p(0.0, 0.0), u * 0.28, st);
            if icon == Icon::EyeOff {
                seg(p(-0.85, -0.85), p(0.85, 0.85));
            }
        }
        Icon::Lock | Icon::Unlock => {
            rrect(box_r(-0.7, -0.1, 0.7, 0.9), u * 0.14);
            let arc_pts = if icon == Icon::Lock {
                let mut a = vec![p(-0.42, -0.1)];
                a.extend(arc(p(0.0, -0.45), u * 0.42, PI, TAU, 10));
                a.push(p(0.42, -0.1));
                a
            } else {
                let mut a = vec![p(-0.42, -0.1)];
                a.extend(arc(p(0.0, -0.55), u * 0.42, PI, TAU * 0.93, 10));
                a
            };
            line(arc_pts);
            painter.circle_filled(p(0.0, 0.38), u * 0.1, color);
        }
        Icon::Trash => {
            seg(p(-0.85, -0.6), p(0.85, -0.6));
            line(vec![
                p(-0.3, -0.6),
                p(-0.25, -0.9),
                p(0.25, -0.9),
                p(0.3, -0.6),
            ]);
            line(vec![
                p(-0.62, -0.6),
                p(-0.5, 0.9),
                p(0.5, 0.9),
                p(0.62, -0.6),
            ]);
            seg(p(-0.15, -0.25), p(-0.12, 0.55));
            seg(p(0.15, -0.25), p(0.12, 0.55));
        }
        Icon::ChevronUp => line(vec![p(-0.6, 0.3), p(0.0, -0.3), p(0.6, 0.3)]),
        Icon::ChevronDown => line(vec![p(-0.6, -0.3), p(0.0, 0.3), p(0.6, -0.3)]),
        Icon::ChevronLeft => line(vec![p(0.3, -0.6), p(-0.3, 0.0), p(0.3, 0.6)]),
        Icon::ChevronRight => line(vec![p(-0.3, -0.6), p(0.3, 0.0), p(-0.3, 0.6)]),
        Icon::Plus => {
            seg(p(-0.7, 0.0), p(0.7, 0.0));
            seg(p(0.0, -0.7), p(0.0, 0.7));
        }
        Icon::Close => {
            seg(p(-0.58, -0.58), p(0.58, 0.58));
            seg(p(0.58, -0.58), p(-0.58, 0.58));
        }
        Icon::Check => line(vec![p(-0.7, 0.05), p(-0.2, 0.55), p(0.72, -0.5)]),
        Icon::Volume | Icon::VolumeMute => {
            closed(vec![
                p(-0.9, -0.3),
                p(-0.5, -0.3),
                p(0.0, -0.75),
                p(0.0, 0.75),
                p(-0.5, 0.3),
                p(-0.9, 0.3),
            ]);
            if icon == Icon::Volume {
                line(arc(p(0.0, 0.0), u * 0.42, -PI * 0.3, PI * 0.3, 8));
                line(arc(p(0.0, 0.0), u * 0.8, -PI * 0.3, PI * 0.3, 10));
            } else {
                seg(p(0.3, -0.35), p(0.9, 0.35));
                seg(p(0.9, -0.35), p(0.3, 0.35));
            }
        }
        Icon::Sliders => {
            for (y, knob) in [(-0.6, 0.35), (0.0, -0.4), (0.6, 0.1)] {
                seg(p(-0.9, y), p(0.9, y));
                painter.circle_filled(p(knob, y), u * 0.2, color);
            }
        }
        Icon::Monitor => {
            rrect(box_r(-0.95, -0.75, 0.95, 0.5), u * 0.14);
            seg(p(0.0, 0.5), p(0.0, 0.85));
            seg(p(-0.45, 0.88), p(0.45, 0.88));
        }
        Icon::Window => {
            rrect(box_r(-0.95, -0.8, 0.95, 0.8), u * 0.14);
            seg(p(-0.95, -0.38), p(0.95, -0.38));
            painter.circle_filled(p(-0.66, -0.6), u * 0.07, color);
            painter.circle_filled(p(-0.44, -0.6), u * 0.07, color);
        }
        Icon::Image => {
            rrect(box_r(-0.95, -0.8, 0.95, 0.8), u * 0.14);
            painter.circle_stroke(p(0.4, -0.32), u * 0.16, st);
            line(vec![
                p(-0.95, 0.6),
                p(-0.3, -0.05),
                p(0.2, 0.42),
                p(0.45, 0.2),
                p(0.95, 0.62),
            ]);
        }
        Icon::Text => {
            seg(p(-0.75, -0.75), p(0.75, -0.75));
            seg(p(0.0, -0.75), p(0.0, 0.8));
            seg(p(-0.3, 0.8), p(0.3, 0.8));
        }
        Icon::Camera => {
            rrect(box_r(-0.95, -0.5, 0.35, 0.55), u * 0.14);
            closed(vec![
                p(0.35, -0.12),
                p(0.95, -0.45),
                p(0.95, 0.5),
                p(0.35, 0.18),
            ]);
        }
        Icon::Globe => {
            painter.circle_stroke(c, u * 0.9, st);
            seg(p(-0.9, 0.0), p(0.9, 0.0));
            let mut mer = arc(c, u * 0.9, -PI * 0.5, PI * 0.5, 12);
            for q in &mut mer {
                q.x = c.x + (q.x - c.x) * 0.45;
            }
            line(mer.clone());
            for q in &mut mer {
                q.x = c.x - (q.x - c.x);
            }
            line(mer);
        }
        Icon::Swatch => {
            painter.circle_stroke(c, u * 0.88, st);
            painter.add(Shape::convex_polygon(
                arc(c, u * 0.88, -PI * 0.5, PI * 0.5, 14),
                color,
                Stroke::NONE,
            ));
        }
        Icon::Record => {
            painter.circle_filled(c, u * 0.55, color);
        }
        Icon::Stop => {
            painter.rect_filled(box_r(-0.5, -0.5, 0.5, 0.5), u * 0.12, color);
        }
        Icon::Replay | Icon::Refresh | Icon::Undo => {
            let (a0, a1) = if icon == Icon::Undo {
                (PI * 1.05, PI * 2.6)
            } else {
                (-PI * 0.35, PI * 1.35)
            };
            let pts = arc(c, u * 0.72, a0, a1, 22);
            let tip = *pts.last().unwrap_or(&c);
            let prev = pts[pts.len().saturating_sub(3)];
            line(pts);
            let dir = (tip - prev).normalized();
            let n = vec2(-dir.y, dir.x);
            painter.add(Shape::convex_polygon(
                vec![tip + dir * u * 0.3, tip + n * u * 0.26, tip - n * u * 0.26],
                color,
                Stroke::NONE,
            ));
            if icon == Icon::Replay {
                painter.circle_filled(c, u * 0.14, color);
            }
        }
        Icon::Pause => {
            painter.rect_filled(box_r(-0.55, -0.7, -0.15, 0.7), u * 0.08, color);
            painter.rect_filled(box_r(0.15, -0.7, 0.55, 0.7), u * 0.08, color);
        }
        Icon::Vertical => {
            rrect(box_r(-0.5, -0.92, 0.5, 0.92), u * 0.2);
            seg(p(-0.15, 0.62), p(0.15, 0.62));
        }
        Icon::Columns => {
            rrect(box_r(-0.95, -0.7, -0.08, 0.7), u * 0.12);
            rrect(box_r(0.08, -0.7, 0.95, 0.7), u * 0.12);
        }
        Icon::Swap => {
            line(vec![p(-0.8, -0.35), p(0.8, -0.35)]);
            line(vec![p(0.45, -0.7), p(0.8, -0.35), p(0.45, 0.0)]);
            line(vec![p(0.8, 0.4), p(-0.8, 0.4)]);
            line(vec![p(-0.45, 0.05), p(-0.8, 0.4), p(-0.45, 0.75)]);
        }
        Icon::Send => {
            closed(vec![
                p(-0.9, -0.8),
                p(0.9, 0.0),
                p(-0.9, 0.8),
                p(-0.55, 0.0),
            ]);
            seg(p(-0.55, 0.0), p(0.2, 0.0));
        }
        Icon::Sun => {
            painter.circle_stroke(c, u * 0.38, st);
            for i in 0..8 {
                let a = i as f32 / 8.0 * TAU;
                let d = vec2(a.cos(), a.sin());
                seg(c + d * u * 0.62, c + d * u * 0.92);
            }
        }
        Icon::Moon => {
            let mut pts = arc(c, u * 0.85, PI * 0.30, PI * 1.70, 20);
            let inner_c = p(0.42, -0.22);
            let mut inner = arc(inner_c, u * 0.68, PI * 1.62, PI * 0.42, 16);
            inner.reverse();
            inner.reverse();
            pts.extend(inner);
            closed(pts);
        }
        Icon::Chat => {
            let r = box_r(-0.92, -0.78, 0.92, 0.5);
            rrect(r, u * 0.3);
            line(vec![p(-0.45, 0.5), p(-0.6, 0.92), p(-0.05, 0.5)]);
        }
        Icon::Layers => {
            let diamond = |dy: f32| {
                vec![
                    p(0.0, -0.55 + dy),
                    p(0.9, -0.1 + dy),
                    p(0.0, 0.35 + dy),
                    p(-0.9, -0.1 + dy),
                ]
            };
            closed(diamond(-0.15));
            line(vec![p(-0.9, 0.3), p(0.0, 0.75), p(0.9, 0.3)]);
        }
        Icon::Download => {
            seg(p(0.0, -0.9), p(0.0, 0.35));
            line(vec![p(-0.45, -0.05), p(0.0, 0.4), p(0.45, -0.05)]);
            line(vec![p(-0.9, 0.45), p(-0.9, 0.9), p(0.9, 0.9), p(0.9, 0.45)]);
        }
        Icon::Key => {
            painter.circle_stroke(p(-0.45, 0.0), u * 0.38, st);
            seg(p(-0.07, 0.0), p(0.92, 0.0));
            seg(p(0.55, 0.0), p(0.55, 0.38));
            seg(p(0.82, 0.0), p(0.82, 0.3));
        }
        Icon::Mic => {
            rrect(box_r(-0.32, -0.92, 0.32, 0.3), u * 0.32);
            line(arc(p(0.0, -0.05), u * 0.62, 0.0, PI, 12));
            seg(p(0.0, 0.57), p(0.0, 0.92));
        }
        Icon::Bolt => {
            closed(vec![
                p(0.15, -0.95),
                p(-0.6, 0.1),
                p(-0.05, 0.1),
                p(-0.2, 0.95),
                p(0.6, -0.15),
                p(0.05, -0.15),
            ]);
        }
        Icon::Folder => {
            closed(vec![
                p(-0.95, -0.7),
                p(-0.3, -0.7),
                p(-0.1, -0.45),
                p(0.95, -0.45),
                p(0.95, 0.8),
                p(-0.95, 0.8),
            ]);
        }
        Icon::Search => {
            painter.circle_stroke(p(-0.15, -0.15), u * 0.58, st);
            seg(p(0.27, 0.27), p(0.85, 0.85));
        }
        Icon::Info => {
            painter.circle_stroke(c, u * 0.9, st);
            seg(p(0.0, -0.05), p(0.0, 0.5));
            painter.circle_filled(p(0.0, -0.4), u * 0.1, color);
        }
        Icon::Warning => {
            closed(vec![p(0.0, -0.85), p(0.92, 0.75), p(-0.92, 0.75)]);
            seg(p(0.0, -0.3), p(0.0, 0.2));
            painter.circle_filled(p(0.0, 0.47), u * 0.1, color);
        }
        Icon::AlertCircle => {
            painter.circle_stroke(c, u * 0.9, st);
            seg(p(0.0, -0.5), p(0.0, 0.08));
            painter.circle_filled(p(0.0, 0.42), u * 0.1, color);
        }
        Icon::CheckCircle => {
            painter.circle_stroke(c, u * 0.9, st);
            line(vec![p(-0.42, 0.02), p(-0.1, 0.34), p(0.45, -0.3)]);
        }
        Icon::List => {
            for y in [-0.6, 0.0, 0.6] {
                painter.circle_filled(p(-0.78, y), u * 0.11, color);
                seg(p(-0.4, y), p(0.9, y));
            }
        }
        Icon::Clock => {
            painter.circle_stroke(c, u * 0.9, st);
            line(vec![p(0.0, -0.5), p(0.0, 0.0), p(0.38, 0.25)]);
        }
        Icon::Chart => {
            rrect(box_r(-0.85, 0.05, -0.35, 0.85), u * 0.08);
            rrect(box_r(-0.25, -0.4, 0.25, 0.85), u * 0.08);
            rrect(box_r(0.35, -0.85, 0.85, 0.85), u * 0.08);
        }
        Icon::More => {
            for x in [-0.62, 0.0, 0.62] {
                painter.circle_filled(p(x, 0.0), u * 0.13, color);
            }
        }
        Icon::Copy => {
            rrect(box_r(-0.4, -0.4, 0.9, 0.9), u * 0.14);
            line(vec![
                p(0.4, -0.4),
                p(0.4, -0.9),
                p(-0.9, -0.9),
                p(-0.9, 0.4),
                p(-0.4, 0.4),
            ]);
        }
        Icon::Edit => {
            closed(vec![
                p(0.45, -0.85),
                p(0.85, -0.45),
                p(-0.4, 0.8),
                p(-0.85, 0.85),
                p(-0.8, 0.4),
            ]);
            seg(p(0.2, -0.6), p(0.6, -0.2));
        }
        Icon::Filter => {
            closed(vec![
                p(-0.9, -0.75),
                p(0.9, -0.75),
                p(0.2, 0.1),
                p(0.2, 0.8),
                p(-0.2, 0.6),
                p(-0.2, 0.1),
            ]);
        }
        Icon::Play => {
            painter.add(Shape::convex_polygon(
                vec![p(-0.45, -0.72), p(0.72, 0.0), p(-0.45, 0.72)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::User => {
            painter.circle_stroke(p(0.0, -0.4), u * 0.38, st);
            line(arc(p(0.0, 0.95), u * 0.85, PI, TAU, 14));
        }
        Icon::Bell => {
            let mut pts = vec![p(-0.75, 0.5), p(-0.55, 0.2), p(-0.55, -0.2)];
            pts.extend(arc(p(0.0, -0.2), u * 0.55, PI, TAU, 12));
            pts.extend([p(0.55, 0.2), p(0.75, 0.5)]);
            closed(pts);
            seg(p(-0.2, 0.8), p(0.2, 0.8));
        }
        Icon::Home => {
            line(vec![p(-0.9, -0.05), p(0.0, -0.85), p(0.9, -0.05)]);
            line(vec![
                p(-0.6, -0.3),
                p(-0.6, 0.85),
                p(0.6, 0.85),
                p(0.6, -0.3),
            ]);
        }
        Icon::Upload => {
            seg(p(0.0, 0.35), p(0.0, -0.9));
            line(vec![p(-0.45, -0.45), p(0.0, -0.9), p(0.45, -0.45)]);
            line(vec![p(-0.9, 0.45), p(-0.9, 0.9), p(0.9, 0.9), p(0.9, 0.45)]);
        }
        Icon::Menu => {
            for y in [-0.55, 0.0, 0.55] {
                seg(p(-0.85, y), p(0.85, y));
            }
        }
        Icon::Prism => {
            closed(vec![p(0.0, -0.85), p(0.88, 0.72), p(-0.88, 0.72)]);
            seg(p(0.0, -0.85), p(0.18, 0.72));
        }
    }
}

/// Points of an arc (angles in radians, clockwise on screen).
fn arc(center: Pos2, radius: f32, a0: f32, a1: f32, steps: usize) -> Vec<Pos2> {
    let steps = steps.max(2);
    (0..=steps)
        .map(|i| {
            let t = a0 + (a1 - a0) * (i as f32 / steps as f32);
            center + Vec2::new(t.cos(), t.sin()) * radius
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_paints_without_panicking() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            let painter = ui.painter();
            for (i, icon) in Icon::ALL.iter().enumerate() {
                let r = Rect::from_min_size(pos2(i as f32 * 20.0, 0.0), Vec2::splat(16.0));
                icon.paint(painter, r, Color32::WHITE);
            }
            // A degenerate rect is a no-op.
            paint_icon(painter, Rect::NOTHING, Icon::Plus, Color32::WHITE);
            ui.add(GlassIcon::new(Icon::Search).size(20.0));
        });
        assert!(out.shapes.len() > 1);
        out.textures_delta.clear();
    }

    #[test]
    fn icon_names_are_unique_kebab_case() {
        let mut names: Vec<_> = Icon::ALL.iter().map(|i| i.name()).collect();
        assert!(
            names
                .iter()
                .all(|n| n.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
        );
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), Icon::ALL.len());
        assert_eq!(Icon::EyeOff.name(), "eye-off");
    }

    #[test]
    fn sidebar_icons_map_into_the_shared_set() {
        assert_eq!(Icon::from(SidebarIcon::Grid), Icon::Widgets);
        assert_eq!(Icon::from(SidebarIcon::Clock), Icon::Clock);
    }
}
