//! Floating menus and popovers built on [`egui::Popup`]: [`GlassPopover`], [`GlassMenu`] and
//! [`MenuItem`].
//!
//! Both paint the theme's floating glass (real GPU blur when available) instead of egui's
//! default popup frame, and keep egui's behavior: click-to-toggle, close on click outside or
//! `Esc`, keyboard navigation with the arrow keys, and hover-to-open submenus.

use egui::containers::menu::{MenuConfig, SubMenu};
use egui::{
    CornerRadius, Frame, Id, InnerResponse, Margin, Popup, PopupCloseBehavior, Rect, RectAlign,
    Response, Role, Sense, Ui, Vec2, Widget, WidgetInfo, pos2, vec2,
};

use super::overlay::{floating_tokens, glass_layer, glass_popup_style};
use crate::icons::Icon;
use crate::theme::PrismTheme;
use crate::typography::{FontWeight, prism_font, prism_numeric_font};

const MENU_MARGIN: i8 = 5;

// ---------------------------------------------------------------------------------------------
// GlassPopover
// ---------------------------------------------------------------------------------------------

/// A glass popover anchored to a widget: free-form content (forms, pickers, previews) that
/// toggles when the anchor is clicked and closes on a click outside or `Esc`.
///
/// ```no_run
/// # use egui_prismglass::{GlassButton, GlassPopover, GlassToggle, Icon};
/// # fn demo(ui: &mut egui::Ui, hdr: &mut bool) {
/// let button = ui.add(GlassButton::secondary("Display").icon(Icon::Sliders));
/// GlassPopover::new(&button).width(240.0).show(|ui| {
///     ui.label("Output");
///     ui.add(GlassToggle::new(hdr).label("HDR"));
/// });
/// # }
/// ```
#[must_use = "Call `.show(...)` to display the popover"]
pub struct GlassPopover<'a> {
    popup: Popup<'a>,
    width: Option<f32>,
    padding: Margin,
}

impl<'a> GlassPopover<'a> {
    /// A popover that toggles when `anchor` is clicked (open state kept in egui memory).
    pub fn new(anchor: &Response) -> Self {
        Self::from_popup(
            Popup::from_toggle_button_response(anchor)
                .close_behavior(PopupCloseBehavior::CloseOnClickOutside),
        )
    }

    /// A popover anchored to `anchor` whose open state is the given bool (you open it).
    pub fn open_bool(anchor: &Response, open: &'a mut bool) -> Self {
        Self::from_popup(
            Popup::from_response(anchor)
                .open_bool(open)
                .close_behavior(PopupCloseBehavior::CloseOnClickOutside),
        )
    }

    /// Wraps a pre-configured [`egui::Popup`] (any anchor, open mode or close behavior).
    pub fn from_popup(popup: Popup<'a>) -> Self {
        Self {
            popup: popup.frame(Frame::NONE).gap(6.0),
            width: None,
            padding: Margin::same(12),
        }
    }

    /// Where to place the popover relative to the anchor (default: below, left-aligned,
    /// flipping when there is no room).
    pub fn align(mut self, align: RectAlign) -> Self {
        self.popup = self.popup.align(align);
        self
    }

    /// Distance from the anchor in points (default `6`).
    pub fn gap(mut self, gap: f32) -> Self {
        self.popup = self.popup.gap(gap);
        self
    }

    /// Content width in points (default: fits the content).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Inner padding (default `12` on every side).
    pub fn padding(mut self, padding: Margin) -> Self {
        self.padding = padding;
        self
    }

    /// Overrides the popup id (default: derived from the anchor).
    pub fn id(mut self, id: Id) -> Self {
        self.popup = self.popup.id(id);
        self
    }

    /// Is the popover open this frame?
    pub fn is_open(&self) -> bool {
        self.popup.is_open()
    }

    /// Shows the popover if it is open.
    pub fn show<R>(self, content: impl FnOnce(&mut Ui) -> R) -> Option<InnerResponse<R>> {
        let Self {
            popup,
            width,
            padding,
        } = self;
        popup.show(|ui| {
            let theme = PrismTheme::from_ctx(ui.ctx());
            let tokens = floating_tokens(&theme, Some(theme.radius.lg.min(14.0)));
            glass_layer(ui, padding, &tokens, |ui| {
                if let Some(w) = width {
                    ui.set_width(w);
                }
                content(ui)
            })
        })
    }
}

// ---------------------------------------------------------------------------------------------
// GlassMenu
// ---------------------------------------------------------------------------------------------

/// A glass dropdown or context menu of [`MenuItem`]s, with separators, section titles and
/// submenus.
///
/// Clicking an item closes the menu (like every egui menu); clicking a submenu row keeps it
/// open. Arrow keys move between items.
///
/// ```no_run
/// # use egui_prismglass::{GlassButton, GlassMenu, Icon, MenuItem};
/// # fn demo(ui: &mut egui::Ui) {
/// let button = ui.add(GlassButton::icon_only(Icon::More));
/// GlassMenu::new(&button).show(|ui| {
///     if ui.add(MenuItem::new("Rename").icon(Icon::Edit).shortcut("F2")).clicked() {
///         // …
///     }
///     ui.add(MenuItem::new("Duplicate").icon(Icon::Copy).shortcut("Ctrl+D"));
///     GlassMenu::submenu(ui, MenuItem::new("Export").icon(Icon::Download), |ui| {
///         ui.add(MenuItem::new("PNG"));
///         ui.add(MenuItem::new("MP4"));
///     });
///     GlassMenu::separator(ui);
///     ui.add(MenuItem::new("Delete").icon(Icon::Trash).danger(true));
/// });
/// # }
/// ```
#[must_use = "Call `.show(...)` to display the menu"]
pub struct GlassMenu<'a> {
    popup: Popup<'a>,
    min_width: f32,
}

impl<'a> GlassMenu<'a> {
    /// A dropdown menu that toggles when `anchor` is clicked.
    pub fn new(anchor: &Response) -> Self {
        Self::from_popup(Popup::menu(anchor))
    }

    /// A context menu that opens at the pointer when `target` is secondary-clicked.
    pub fn context(target: &Response) -> Self {
        Self::from_popup(Popup::context_menu(target))
    }

    /// Wraps a pre-configured [`egui::Popup`] (it should be of kind `PopupKind::Menu` for
    /// submenus and keyboard navigation to work, as [`Popup::menu`] is).
    pub fn from_popup(popup: Popup<'a>) -> Self {
        Self {
            popup: popup.frame(Frame::NONE).style(glass_popup_style).gap(4.0),
            min_width: 180.0,
        }
    }

    /// Where to place the menu relative to the anchor.
    pub fn align(mut self, align: RectAlign) -> Self {
        self.popup = self.popup.align(align);
        self
    }

    /// Minimum width in points (default `180`).
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// Overrides the popup id (default: derived from the anchor).
    pub fn id(mut self, id: Id) -> Self {
        self.popup = self.popup.id(id);
        self
    }

    /// Is the menu open this frame?
    pub fn is_open(&self) -> bool {
        self.popup.is_open()
    }

    /// Shows the menu if it is open.
    pub fn show<R>(self, content: impl FnOnce(&mut Ui) -> R) -> Option<InnerResponse<R>> {
        let min_width = self.min_width;
        self.popup.show(|ui| menu_body(ui, min_width, content))
    }

    /// A [`MenuItem`] that opens a nested glass menu on hover (or click / `→`).
    ///
    /// Returns the row's response and the submenu's, if it is open.
    pub fn submenu<R>(
        ui: &mut Ui,
        item: MenuItem,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> (Response, Option<InnerResponse<R>>) {
        let response = ui.add(item.with_arrow());
        let inner = SubMenu::new()
            .config(MenuConfig::new().style(glass_popup_style))
            .show(ui, &response, |ui| menu_body(ui, 160.0, content));
        (response, inner)
    }

    /// A hairline separator between groups of items.
    pub fn separator(ui: &mut Ui) {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 9.0), Sense::hover());
        ui.painter().hline(
            rect.x_range().shrink(4.0),
            rect.center().y,
            egui::Stroke::new(1.0, theme.neutrals.border_strong),
        );
    }

    /// A small uppercase group title.
    pub fn section(ui: &mut Ui, title: &str) {
        let theme = PrismTheme::from_ctx(ui.ctx());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(super::kit::eyebrow_text(
                ui.ctx(),
                title,
                theme.palette.text_muted,
            ));
        });
        ui.add_space(2.0);
    }
}

fn menu_body<R>(ui: &mut Ui, min_width: f32, content: impl FnOnce(&mut Ui) -> R) -> R {
    let theme = PrismTheme::from_ctx(ui.ctx());
    let tokens = floating_tokens(&theme, Some(theme.radius.md + 3.0));
    glass_layer(ui, Margin::same(MENU_MARGIN), &tokens, |ui| {
        ui.set_min_width(min_width);
        ui.spacing_mut().item_spacing.y = 1.0;
        content(ui)
    })
}

// ---------------------------------------------------------------------------------------------
// MenuItem
// ---------------------------------------------------------------------------------------------

/// A row in a [`GlassMenu`]: optional leading icon or check mark, label, right-aligned
/// shortcut, and a danger style for destructive actions.
#[derive(Debug, Clone)]
pub struct MenuItem {
    label: String,
    icon: Option<Icon>,
    shortcut: Option<String>,
    checked: Option<bool>,
    danger: bool,
    enabled: bool,
    arrow: bool,
}

impl MenuItem {
    /// Creates a menu item.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            shortcut: None,
            checked: None,
            danger: false,
            enabled: true,
            arrow: false,
        }
    }

    /// Adds a leading icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Shows a shortcut hint on the right (e.g. `"Ctrl+D"`, `"⌘K"`).
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Makes the item a checkable option: `true` shows a check mark in the icon slot.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Styles the item as destructive (danger color).
    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// Enables or disables the item.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn with_arrow(mut self) -> Self {
        self.arrow = true;
        self
    }
}

impl Widget for MenuItem {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = PrismTheme::from_ctx(ui.ctx());
        let ctx = ui.ctx().clone();
        let has_slot = self.icon.is_some() || self.checked.is_some();
        let label_galley = ui.painter().layout_no_wrap(
            self.label.clone(),
            prism_font(&ctx, 13.0, FontWeight::Regular),
            theme.palette.text_primary,
        );
        let shortcut_galley = self.shortcut.as_ref().map(|s| {
            ui.painter().layout_no_wrap(
                s.clone(),
                prism_numeric_font(&ctx, 12.0, FontWeight::Regular),
                theme.palette.text_muted,
            )
        });
        let pad = 8.0;
        let slot = if has_slot { 24.0 } else { 0.0 };
        let right = shortcut_galley.as_ref().map_or(0.0, |g| g.size().x + 24.0)
            + if self.arrow { 18.0 } else { 0.0 };
        let desired = vec2(pad * 2.0 + slot + label_galley.size().x + right, 28.0);
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = ui.allocate_response(desired, sense);
        let label = self.label.clone();
        let (enabled, checked) = (self.enabled, self.checked);
        response.widget_info(|| match checked {
            Some(on) => WidgetInfo::selected(Role::CheckBox, enabled, on, label.clone()),
            None => WidgetInfo::labeled(Role::Button, enabled, label.clone()),
        });
        let rect = response.rect;
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let highlighted = self.enabled && (response.hovered() || response.has_focus());
        let painter = ui.painter();
        if highlighted {
            let bg = if self.danger {
                super::prism_paint::with_alpha(theme.palette.danger, 30)
            } else {
                theme.neutrals.selected
            };
            painter.rect_filled(
                rect,
                CornerRadius::same((theme.radius.md - 1.0).max(2.0) as u8),
                bg,
            );
        }
        let mut color = if self.danger {
            theme.palette.danger
        } else {
            theme.palette.text_primary
        };
        if !self.enabled {
            color = theme.palette.text_muted;
        }
        let icon_color = if self.danger || !self.enabled || highlighted {
            color
        } else {
            theme.palette.text_secondary
        };
        let mut x = rect.left() + pad;
        if has_slot {
            let icon_rect = Rect::from_min_size(pos2(x, rect.center().y - 7.5), Vec2::splat(15.0));
            match (self.checked, self.icon) {
                (Some(true), _) => Icon::Check.paint(painter, icon_rect, theme.palette.accent),
                (None, Some(icon)) => icon.paint(painter, icon_rect, icon_color),
                _ => {}
            }
            x += slot;
        }
        painter.galley_with_override_text_color(
            pos2(x, rect.center().y - label_galley.size().y * 0.5),
            label_galley,
            color,
        );
        let mut rx = rect.right() - pad;
        if self.arrow {
            Icon::ChevronRight.paint(
                painter,
                Rect::from_min_size(pos2(rx - 12.0, rect.center().y - 6.0), Vec2::splat(12.0)),
                theme.palette.text_muted,
            );
            rx -= 18.0;
        }
        if let Some(g) = shortcut_galley {
            painter.galley(
                pos2(rx - g.size().x, rect.center().y - g.size().y * 0.5),
                g,
                theme.palette.text_muted,
            );
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn click_at(pos: egui::Pos2) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    }

    /// Runs one frame with a "Menu" anchor and a glass menu; returns (menu open, anchor center).
    fn menu_frame(ctx: &egui::Context, events: Vec<egui::Event>) -> (bool, egui::Pos2) {
        let input = egui::RawInput {
            events,
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        };
        let mut open = false;
        let mut center = egui::Pos2::ZERO;
        let mut out = ctx.run_ui(input, |ui| {
            let anchor = ui.button("Menu");
            center = anchor.rect.center();
            open = GlassMenu::new(&anchor)
                .show(|ui| {
                    ui.add(MenuItem::new("Rename").icon(Icon::Edit).shortcut("F2"));
                    ui.add(MenuItem::new("Grid").checked(true));
                    GlassMenu::section(ui, "More");
                    GlassMenu::submenu(ui, MenuItem::new("Export"), |ui| {
                        ui.add(MenuItem::new("PNG"));
                    });
                    GlassMenu::separator(ui);
                    ui.add(MenuItem::new("Delete").danger(true));
                    ui.add(MenuItem::new("Disabled").enabled(false));
                })
                .is_some();
        });
        out.textures_delta.clear();
        (open, center)
    }

    #[test]
    fn menu_opens_on_click_and_closes_on_escape() {
        let ctx = egui::Context::default();
        PrismTheme::prism_dark().apply(&ctx);
        let (open, center) = menu_frame(&ctx, vec![]);
        assert!(!open);
        let (open, _) = menu_frame(&ctx, click_at(center));
        assert!(open, "menu should open after clicking its anchor");
        let (open, _) = menu_frame(&ctx, vec![]);
        assert!(open, "menu stays open");
        let escape = egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        menu_frame(&ctx, vec![escape]);
        let (open, _) = menu_frame(&ctx, vec![]);
        assert!(!open, "Esc closes the menu");
    }

    #[test]
    fn popover_with_bool_state_shows_and_closes_outside() {
        let ctx = egui::Context::default();
        PrismTheme::prism_glass_light().apply(&ctx);
        let mut open = true;
        let mut shown = false;
        for events in [vec![], vec![], click_at(pos2(700.0, 500.0)), vec![]] {
            let input = egui::RawInput {
                events,
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 600.0))),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                let anchor = ui.button("Display");
                shown = GlassPopover::open_bool(&anchor, &mut open)
                    .width(200.0)
                    .show(|ui| ui.label("content"))
                    .is_some();
            });
            out.textures_delta.clear();
        }
        assert!(!open, "click outside closes a popover");
        assert!(!shown);
    }
}
