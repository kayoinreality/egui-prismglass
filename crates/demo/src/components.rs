//! The **Components** page of Prism Studio: every v0.4 widget in one scrollable gallery.

use egui::{Align, Layout, Margin, Rect, RichText, ScrollArea, Ui, UiBuilder, vec2};
use egui_prismglass::{
    FontWeight, GlassButton, GlassCard, GlassChip, GlassElevation, GlassEmptyState, GlassIcon,
    GlassKbd, GlassListRow, GlassMenu, GlassModal, GlassPopover, GlassSegmented, GlassSlider,
    GlassTextField, GlassToasts, GlassToggle, Icon, MenuItem, PrismTheme, SectionHeader, Toast,
    ToastAnchor, prism_font,
};

/// State of the components gallery.
#[allow(clippy::struct_excessive_bools)]
pub struct ComponentsPage {
    pub query: String,
    pub name: String,
    pub stream_key: String,
    pub server: String,
    pub notes: String,
    pub tab: usize,
    pub selected_row: usize,
    pub filters: [bool; 3],
    pub modal_open: bool,
    pub snap: bool,
    pub hdr: bool,
    pub bitrate: f32,
    pub anchor: usize,
    pub icon_filter: String,
    pub last_action: String,
}

impl Default for ComponentsPage {
    fn default() -> Self {
        Self {
            query: String::new(),
            name: "Main camera".to_owned(),
            stream_key: "abc".to_owned(),
            server: "rtmp://live.example.com/app".to_owned(),
            notes: String::new(),
            tab: 0,
            selected_row: 0,
            filters: [true, false, true],
            modal_open: false,
            snap: true,
            hdr: false,
            bitrate: 6000.0,
            anchor: 0,
            icon_filter: String::new(),
            last_action: "—".to_owned(),
        }
    }
}

const ANCHORS: [ToastAnchor; 3] = [
    ToastAnchor::BottomRight,
    ToastAnchor::TopRight,
    ToastAnchor::BottomCenter,
];

fn title(ui: &Ui, text: &str, theme: &PrismTheme) -> RichText {
    RichText::new(text)
        .font(prism_font(ui.ctx(), 14.0, FontWeight::Semibold))
        .color(theme.palette.text_primary)
}

fn caption(ui: &Ui, text: &str, theme: &PrismTheme) -> RichText {
    RichText::new(text)
        .font(prism_font(ui.ctx(), 12.0, FontWeight::Regular))
        .color(theme.palette.text_muted)
}

/// A titled card in the gallery grid.
fn card(ui: &mut Ui, theme: &PrismTheme, heading: &str, sub: &str, add: impl FnOnce(&mut Ui)) {
    GlassCard::new()
        .elevation(GlassElevation::Subtle)
        .padding(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.label(title(ui, heading, theme));
            ui.label(caption(ui, sub, theme));
            ui.add_space(4.0);
            add(ui);
        });
}

impl ComponentsPage {
    /// Renders the gallery into `rect`; toasts are pushed into the app-wide queue.
    pub fn show(&mut self, ui: &mut Ui, rect: Rect, theme: &PrismTheme, toasts: &mut GlassToasts) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ui.set_clip_rect(rect);
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.set_width(rect.width() - 48.0);
                            self.contents(ui, theme, toasts);
                        });
                    });
                    ui.add_space(24.0);
                });
        });

        self.modal(ui, theme, toasts);
    }

    fn contents(&mut self, ui: &mut Ui, theme: &PrismTheme, toasts: &mut GlassToasts) {
        ui.label(
            RichText::new("Components")
                .font(prism_font(ui.ctx(), 20.0, FontWeight::Semibold))
                .color(theme.palette.text_primary),
        );
        ui.label(caption(
            ui,
            "New in egui-prismglass 0.4: text fields, modals, toasts, menus & popovers, the shared \
             icon set and kit components. Switch Dark/Light, material and accent in the sidebar.",
            theme,
        ));
        ui.add_space(16.0);

        let col_w = ((ui.available_width() - 16.0) * 0.5).max(260.0);
        let two_columns = ui.available_width() > 620.0;
        let column = |ui: &mut Ui, add: &mut dyn FnMut(&mut Ui)| {
            if two_columns {
                ui.allocate_ui_with_layout(vec2(col_w, 0.0), Layout::top_down(Align::Min), |ui| {
                    ui.set_width(col_w);
                    add(ui);
                });
            } else {
                add(ui);
            }
        };

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = vec2(16.0, 16.0);
            column(ui, &mut |ui| {
                self.inputs(ui, theme);
                ui.add_space(16.0);
                self.overlays(ui, theme, toasts);
            });
            column(ui, &mut |ui| {
                self.kit(ui, theme);
                ui.add_space(16.0);
                Self::buttons(ui, theme, toasts);
            });
        });
        ui.add_space(16.0);
        self.icons(ui, theme);
    }

    fn inputs(&mut self, ui: &mut Ui, theme: &PrismTheme) {
        card(
            ui,
            theme,
            "GlassTextField",
            "Placeholder, leading icon, focus / error / disabled, password, multiline.",
            |ui| {
                ui.add(
                    GlassTextField::new(&mut self.query)
                        .placeholder("Search sources")
                        .icon(Icon::Search),
                );
                ui.add(
                    GlassTextField::new(&mut self.name)
                        .label("Source name")
                        .icon(Icon::Camera)
                        .help("Shown in the scene list."),
                );
                let invalid = self.stream_key.len() < 8;
                ui.add(
                    GlassTextField::new(&mut self.stream_key)
                        .label("Stream key")
                        .icon(Icon::Key)
                        .password(true)
                        .error(invalid.then_some("At least 8 characters")),
                );
                ui.add(
                    GlassTextField::new(&mut self.server)
                        .label("Server (managed)")
                        .icon(Icon::Globe)
                        .enabled(false),
                );
                ui.add(
                    GlassTextField::multiline(&mut self.notes)
                        .label("Notes")
                        .placeholder("Anything the crew should know…")
                        .rows(3),
                );
            },
        );
    }

    fn overlays(&mut self, ui: &mut Ui, theme: &PrismTheme, toasts: &mut GlassToasts) {
        card(
            ui,
            theme,
            "Menus, popovers, modal & toasts",
            "Floating glass layers on egui::Popup / egui::Modal. Right-click the row below.",
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    let file = ui.add(GlassButton::secondary("File").icon(Icon::Folder));
                    let _ = GlassMenu::new(&file).show(|ui| {
                        if ui
                            .add(
                                MenuItem::new("New scene")
                                    .icon(Icon::Plus)
                                    .shortcut("Ctrl+N"),
                            )
                            .clicked()
                        {
                            "New scene".clone_into(&mut self.last_action);
                        }
                        if ui
                            .add(
                                MenuItem::new("Duplicate")
                                    .icon(Icon::Copy)
                                    .shortcut("Ctrl+D"),
                            )
                            .clicked()
                        {
                            "Duplicate".clone_into(&mut self.last_action);
                        }
                        if ui
                            .add(MenuItem::new("Snap to grid").checked(self.snap))
                            .clicked()
                        {
                            self.snap = !self.snap;
                        }
                        GlassMenu::submenu(
                            ui,
                            MenuItem::new("Export").icon(Icon::Download),
                            |ui| {
                                for format in ["PNG", "MP4", "ProRes"] {
                                    if ui.add(MenuItem::new(format)).clicked() {
                                        self.last_action = format!("Export {format}");
                                    }
                                }
                            },
                        );
                        GlassMenu::separator(ui);
                        ui.add(MenuItem::new("Locked").icon(Icon::Lock).enabled(false));
                        if ui
                            .add(MenuItem::new("Delete").icon(Icon::Trash).danger(true))
                            .clicked()
                        {
                            self.modal_open = true;
                        }
                    });

                    let display = ui.add(GlassButton::secondary("Display").icon(Icon::Sliders));
                    let _ = GlassPopover::new(&display).width(240.0).show(|ui| {
                        ui.spacing_mut().item_spacing.y = 10.0;
                        ui.add(SectionHeader::new("Output"));
                        ui.add(GlassToggle::new(&mut self.hdr).label("HDR"));
                        ui.add(
                            GlassSlider::new(&mut self.bitrate, 1000.0..=20000.0)
                                .label("Bitrate (kbps)")
                                .decimals(0),
                        );
                    });

                    if ui
                        .add(GlassButton::danger("Delete scene").icon(Icon::Trash))
                        .clicked()
                    {
                        self.modal_open = true;
                    }
                });
                let row = GlassListRow::new()
                    .icon(Icon::Image)
                    .title("Lower third.png")
                    .subtitle("Right-click for a context menu")
                    .trailing("1.2 MB");
                let response = ui.add(row);
                let _ = GlassMenu::context(&response).show(|ui| {
                    ui.add(MenuItem::new("Rename").icon(Icon::Edit).shortcut("F2"));
                    ui.add(MenuItem::new("Reveal in folder").icon(Icon::Folder));
                    GlassMenu::separator(ui);
                    ui.add(MenuItem::new("Remove").icon(Icon::Trash).danger(true));
                });
                ui.label(caption(
                    ui,
                    &format!("Last menu action: {}", self.last_action),
                    theme,
                ));

                ui.add_space(4.0);
                ui.add(SectionHeader::new("Toasts"));
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add(GlassButton::secondary("Success").icon(Icon::CheckCircle))
                        .clicked()
                    {
                        toasts.add(Toast::success("Scene saved").body("Intro · 4 sources"));
                    }
                    if ui
                        .add(GlassButton::secondary("Info").icon(Icon::Info))
                        .clicked()
                    {
                        toasts.add(Toast::info("Update available").body("PrismCast 3.3 is ready."));
                    }
                    if ui
                        .add(GlassButton::secondary("Warning").icon(Icon::Warning))
                        .clicked()
                    {
                        toasts.add(
                            Toast::warning("Dropped frames").body("2.4 % in the last minute."),
                        );
                    }
                    if ui
                        .add(GlassButton::secondary("Danger").icon(Icon::AlertCircle))
                        .clicked()
                    {
                        toasts.add(
                            Toast::danger("Stream disconnected")
                                .body("Reconnecting in 5 s…")
                                .sticky(),
                        );
                    }
                });
                let before = self.anchor;
                ui.add(
                    GlassSegmented::new(&mut self.anchor, &["Bottom right", "Top right", "Bottom"])
                        .fill_width(true),
                );
                if before != self.anchor {
                    *toasts = std::mem::take(toasts).anchor(ANCHORS[self.anchor]);
                }
            },
        );
    }

    fn modal(&mut self, ui: &mut Ui, theme: &PrismTheme, toasts: &mut GlassToasts) {
        let mut deleted = false;
        let _ = GlassModal::new("components-delete")
            .title("Delete scene?")
            .description("“Intro” and its 4 sources will be removed. This can't be undone.")
            .icon_colored(Icon::Trash, theme.palette.danger)
            .show(ui.ctx(), &mut self.modal_open, |ui| {
                let mut confirm = String::new();
                ui.add(
                    GlassTextField::new(&mut confirm)
                        .placeholder("Tab moves focus only inside the dialog")
                        .icon(Icon::Lock),
                );
                GlassModal::footer(ui, |ui| {
                    if ui.add(GlassButton::danger("Delete")).clicked() {
                        deleted = true;
                        ui.close();
                    }
                    if ui.add(GlassButton::secondary("Cancel")).clicked() {
                        ui.close();
                    }
                });
            });
        if deleted {
            toasts.add(Toast::success("Scene deleted").duration(3.0));
        }
    }

    fn kit(&mut self, ui: &mut Ui, theme: &PrismTheme) {
        card(
            ui,
            theme,
            "Kit components",
            "List rows, chips, key caps, section headers and an empty state.",
            |ui| {
                SectionHeader::new("Scenes").count(3).show(ui, |ui| {
                    ui.add(GlassButton::icon_only(Icon::Plus))
                        .on_hover_text("Add scene");
                });
                for (i, (icon, name, meta)) in [
                    (Icon::Camera, "Main camera", "1080p"),
                    (Icon::Monitor, "Display 2", "1440p"),
                    (Icon::Image, "Overlay", "PNG"),
                ]
                .into_iter()
                .enumerate()
                {
                    let row = GlassListRow::new()
                        .icon(icon)
                        .title(name)
                        .trailing(meta)
                        .selected(self.selected_row == i);
                    if ui.add(row).clicked() {
                        self.selected_row = i;
                    }
                }
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.add(GlassChip::new("rtmp"));
                    ui.add(GlassChip::new("Live").color(theme.palette.danger));
                    ui.add(GlassChip::new("4K · 60").color(theme.palette.success));
                    for (i, label) in ["Video", "Audio", "Images"].into_iter().enumerate() {
                        if ui
                            .add(
                                GlassChip::new(label)
                                    .uppercase(false)
                                    .selected(self.filters[i]),
                            )
                            .clicked()
                        {
                            self.filters[i] = !self.filters[i];
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(caption(ui, "Command palette", theme));
                    ui.add(GlassKbd::combo(["Ctrl", "K"]));
                    ui.label(caption(ui, "Close", theme));
                    ui.add(GlassKbd::new("Esc"));
                });
                ui.add(
                    GlassSegmented::new(&mut self.tab, &["All", "Errors", "Warnings"])
                        .optional_counts(&[Some(24), None, Some(3)]),
                )
                .on_hover_text("optional_counts: no \"0\" next to Errors");
                GlassEmptyState::new(Icon::Layers, "No sources yet")
                    .description("Add a camera, a window or an image.")
                    .show(ui, |ui| {
                        ui.add(GlassButton::primary("Add source").icon(Icon::Plus));
                    });
            },
        );
    }

    fn buttons(ui: &mut Ui, theme: &PrismTheme, toasts: &mut GlassToasts) {
        card(
            ui,
            theme,
            "Buttons with vector icons",
            "GlassButton::icon takes an Icon (or a glyph string, as in 0.3).",
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add(GlassButton::primary("Go live").icon(Icon::Broadcast))
                        .clicked()
                    {
                        toasts
                            .add(Toast::info("You are live").body("Streaming to 2 destinations."));
                    }
                    ui.add(GlassButton::secondary("Record").icon(Icon::Record));
                    ui.add(GlassButton::ghost("Settings").icon(Icon::Settings));
                    ui.add(GlassButton::secondary("Glyph").icon("★"));
                });
                ui.horizontal(|ui| {
                    for (icon, tip) in [
                        (Icon::Undo, "Undo"),
                        (Icon::Refresh, "Refresh"),
                        (Icon::Eye, "Show"),
                        (Icon::Lock, "Lock"),
                        (Icon::More, "More"),
                    ] {
                        ui.add(GlassButton::icon_only(icon)).on_hover_text(tip);
                    }
                });
            },
        );
    }

    fn icons(&mut self, ui: &mut Ui, theme: &PrismTheme) {
        card(
            ui,
            theme,
            "Icon set",
            &format!(
                "{} vector icons painted on a 16 pt grid with a 1.5 pt stroke. Hover for names.",
                Icon::ALL.len()
            ),
            |ui| {
                ui.add(
                    GlassTextField::new(&mut self.icon_filter)
                        .placeholder("Filter icons")
                        .icon(Icon::Search)
                        .width(240.0),
                );
                let filter = self.icon_filter.to_lowercase();
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                    for icon in Icon::ALL.iter().filter(|i| i.name().contains(&filter)) {
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(36.0, 36.0), egui::Sense::hover());
                        if response.hovered() {
                            ui.painter().rect_filled(rect, 8.0, theme.neutrals.hover);
                        }
                        icon.paint(
                            ui.painter(),
                            Rect::from_center_size(rect.center(), vec2(18.0, 18.0)),
                            theme.palette.text_secondary,
                        );
                        response.on_hover_text(icon.name());
                    }
                });
                ui.horizontal(|ui| {
                    ui.add(
                        GlassIcon::new(Icon::Prism)
                            .size(20.0)
                            .color(theme.palette.accent),
                    );
                    ui.label(caption(ui, "GlassIcon::new(Icon::Prism).size(20.0)", theme));
                });
            },
        );
    }
}
