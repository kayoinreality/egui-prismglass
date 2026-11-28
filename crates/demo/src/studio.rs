//! **Prism Studio**: the flagship demo of the Prism presets — a realistic render-queue app with
//! a glass sidebar, flat content, an inspector and a floating glass action bar.
//!
//! The sidebar's Appearance controls switch between dark/light, the two materials
//! ([`SurfaceStyle::Layered`] and [`SurfaceStyle::Frosted`]) and every accent palette. The
//! classic v0.2 showcase is still one click away.

use egui::{
    Align, Align2, Color32, ComboBox, CornerRadius, Layout, Margin, Rect, RichText, ScrollArea,
    Sense, Stroke, Ui, UiBuilder, pos2, vec2,
};
use egui_prismglass::{
    AccentPalette, FontWeight, GlassButton, GlassCheckbox, GlassElevation, GlassMenu, GlassModal,
    GlassProgress, GlassSegmented, GlassSidebar, GlassSlider, GlassTextField, GlassToasts,
    GlassToggle, Icon, MenuItem, PrismContextExt, PrismPreset, PrismTheme, SidebarItem,
    SurfaceStyle, ThemeMode, Toast, install_fonts, paint_glass_surface_ui, paint_prism_backdrop_ui,
    paint_radial_glow_orb, painter::paint_glass_gradient_fill, prism_font, prism_numeric_font,
};

use crate::DemoApp;
use crate::components::ComponentsPage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Rendering,
    Queued,
    Failed,
    Done,
}

impl JobStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Rendering => "Rendering",
            Self::Queued => "Queued",
            Self::Failed => "Failed",
            Self::Done => "Done",
        }
    }

    fn color(self, theme: &PrismTheme) -> Color32 {
        match self {
            Self::Rendering => theme.palette.accent,
            Self::Queued => theme.neutrals.muted,
            Self::Failed => theme.palette.danger,
            Self::Done => theme.palette.success,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    pub name: &'static str,
    pub engine: &'static str,
    pub resolution: &'static str,
    pub frames: u32,
    pub progress: f32,
    pub status: JobStatus,
    pub eta: &'static str,
    pub checked: bool,
}

fn sample_jobs() -> Vec<Job> {
    use JobStatus::{Done, Failed, Queued, Rendering};

    let job = |name, engine, resolution, frames, progress, status, eta, checked| Job {
        name,
        engine,
        resolution,
        frames,
        progress,
        status,
        eta,
        checked,
    };
    vec![
        job(
            "Coastline_v3 · shot_040",
            "Cycles",
            "4K",
            240,
            0.68,
            Rendering,
            "12 min",
            true,
        ),
        job(
            "Night_market · hero",
            "Cycles",
            "4K",
            240,
            0.31,
            Rendering,
            "38 min",
            true,
        ),
        job(
            "Product_shots · bottle_spin",
            "Eevee",
            "1440p",
            120,
            0.0,
            Queued,
            "—",
            false,
        ),
        job(
            "Coastline_v3 · shot_041",
            "Cycles",
            "4K",
            180,
            0.0,
            Queued,
            "—",
            false,
        ),
        job(
            "Night_market · alley_dolly",
            "Cycles",
            "4K",
            300,
            0.54,
            Failed,
            "Out of VRAM",
            false,
        ),
        job(
            "Logo_reveal · final",
            "Eevee",
            "1080p",
            96,
            1.0,
            Done,
            "Done 14:02",
            false,
        ),
        job(
            "Product_shots · cap_closeup",
            "Eevee",
            "1440p",
            60,
            1.0,
            Done,
            "Done 13:31",
            false,
        ),
        job(
            "Coastline_v3 · shot_039",
            "Cycles",
            "4K",
            200,
            1.0,
            Done,
            "Done 12:58",
            false,
        ),
        job(
            "Logo_reveal · alt_b",
            "Eevee",
            "1080p",
            96,
            1.0,
            Done,
            "Done 12:20",
            false,
        ),
        job(
            "Night_market · establishing",
            "Cycles",
            "4K",
            240,
            1.0,
            Done,
            "Done 11:47",
            false,
        ),
    ]
}

/// Which page of Prism Studio is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StudioPage {
    /// The render queue (table, inspector, floating action bar).
    #[default]
    RenderQueue,
    /// The v0.4 components gallery.
    Components,
}

/// State of the Prism Studio demo.
#[allow(clippy::struct_excessive_bools)]
pub struct StudioApp {
    pub page: StudioPage,
    pub components: ComponentsPage,
    pub toasts: GlassToasts,
    pub confirm_cancel: bool,
    pub sort: usize,
    pub theme: PrismTheme,
    pub mode: ThemeMode,
    pub material: SurfaceStyle,
    pub accent: AccentPalette,
    pub jobs: Vec<Job>,
    pub filter: usize,
    pub focused: usize,
    pub search: String,
    pub resolution: usize,
    pub samples: f32,
    pub denoise: bool,
    pub motion_blur: bool,
    pub format: usize,
    pub classic: Option<DemoApp>,
}

impl Default for StudioApp {
    fn default() -> Self {
        Self::with_preset(PrismPreset::PrismDark)
    }
}

const FORMATS: [&str; 3] = ["OpenEXR · 16-bit half", "PNG · 8-bit", "ProRes 4444"];

impl StudioApp {
    /// Creates the demo with one of the four Prism presets (classic presets fall back to Prism Dark).
    pub fn with_preset(preset: PrismPreset) -> Self {
        let theme = if PrismPreset::PRISM.contains(&preset) {
            preset.into_theme()
        } else {
            PrismTheme::prism_dark()
        };
        Self {
            page: StudioPage::RenderQueue,
            components: ComponentsPage::default(),
            toasts: GlassToasts::new(),
            confirm_cancel: false,
            sort: 0,
            mode: theme.mode,
            material: theme.surface_style,
            accent: theme.accent_palette,
            theme,
            jobs: sample_jobs(),
            filter: 0,
            focused: 1,
            search: String::new(),
            resolution: 2,
            samples: 512.0,
            denoise: true,
            motion_blur: false,
            format: 0,
            classic: None,
        }
    }

    /// Switches accent palette.
    pub fn set_accent(&mut self, accent: AccentPalette) {
        self.accent = accent;
        self.rebuild_theme();
    }

    fn rebuild_theme(&mut self) {
        let real_blur = self.theme.real_blur_enabled;
        let scene = self.theme.backdrop_scene;
        self.theme = PrismTheme::prism(self.mode, self.material).with_accent_palette(self.accent);
        self.theme.real_blur_enabled = real_blur;
        self.theme.backdrop_scene = scene;
    }

    /// Renders the whole app into `ui` (used by `eframe` and headless captures).
    pub fn show(&mut self, ui: &mut Ui) {
        install_fonts(ui.ctx());

        if let Some(classic) = &mut self.classic {
            classic.show(ui);
            let back = egui::Area::new(egui::Id::unique("prism_back_to_studio"))
                .anchor(Align2::RIGHT_BOTTOM, vec2(-24.0, -24.0))
                .show(ui.ctx(), |ui| {
                    ui.add(GlassButton::accent("Back to Prism Studio"))
                        .clicked()
                })
                .inner;
            if back {
                self.classic = None;
            }
            return;
        }

        ui.ctx().set_prism_theme(self.theme.clone());
        let full = ui.max_rect();
        paint_prism_backdrop_ui(ui, full, &self.theme, 0.0, 1.0);

        let frosted = self.theme.surface_style == SurfaceStyle::Frosted;
        let (margin, gap) = if frosted { (12.0, 12.0) } else { (0.0, 0.0) };
        let area = full.shrink(margin);
        let sidebar_rect = Rect::from_min_size(area.min, vec2(240.0, area.height()));
        let inspector_rect = Rect::from_min_max(pos2(area.max.x - 320.0, area.min.y), area.max);
        let main_rect = Rect::from_min_max(
            pos2(sidebar_rect.max.x + gap, area.min.y),
            pos2(inspector_rect.min.x - gap, area.max.y),
        );

        // Docked regions are square in the Layered material and float as rounded glass panes in
        // the all-glass material.
        let mut panel = self.theme.panel_surface();
        if !frosted {
            panel = panel.with_corner_radius(0.0);
        }
        self.sidebar(ui, sidebar_rect, frosted);
        match self.page {
            StudioPage::RenderQueue => {
                paint_glass_surface_ui(ui, main_rect, &panel, false);
                paint_glass_surface_ui(ui, inspector_rect, &panel, false);
                self.main_column(ui, main_rect);
                self.inspector(ui, inspector_rect);
                self.cancel_dialog(ui);
            }
            StudioPage::Components => {
                let page_rect = main_rect.union(inspector_rect);
                paint_glass_surface_ui(ui, page_rect, &panel, false);
                let theme = self.theme.clone();
                self.components
                    .show(ui, page_rect, &theme, &mut self.toasts);
            }
        }
        self.toasts.show(ui.ctx());
    }

    /// Confirmation dialog for the floating bar's "Cancel" action.
    fn cancel_dialog(&mut self, ui: &mut Ui) {
        let selected = self.jobs.iter().filter(|j| j.checked).count();
        let mut confirmed = false;
        let _ = GlassModal::new("cancel-jobs")
            .title(format!(
                "Cancel {selected} render{}?",
                if selected == 1 { "" } else { "s" }
            ))
            .description("Frames rendered so far are kept; the jobs move to Failed.")
            .icon_colored(Icon::Stop, self.theme.palette.danger)
            .show(ui.ctx(), &mut self.confirm_cancel, |ui| {
                GlassModal::footer(ui, |ui| {
                    if ui.add(GlassButton::danger("Cancel renders")).clicked() {
                        confirmed = true;
                        ui.close();
                    }
                    if ui.add(GlassButton::secondary("Keep rendering")).clicked() {
                        ui.close();
                    }
                });
            });
        if confirmed {
            self.set_checked_status(JobStatus::Failed);
            self.toasts.add(
                Toast::warning(format!("{selected} render(s) cancelled"))
                    .body("You can re-queue them from History."),
            );
        }
    }

    // ----------------------------------------------------------------------------------------
    // Sidebar

    fn sidebar(&mut self, ui: &mut Ui, rect: Rect, frosted: bool) {
        let theme = self.theme.clone();
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            let mut sidebar = GlassSidebar::new()
                .width(rect.width())
                .padding(Margin::symmetric(12, 14));
            if !frosted {
                sidebar = sidebar.corner_radius(0.0);
            }
            sidebar.show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                brand_header(ui, &theme);
                ui.add_space(18.0);

                if SidebarItem::with_icon(Icon::List, "Render queue")
                    .badge("4")
                    .selected(self.page == StudioPage::RenderQueue)
                    .show(ui)
                    .clicked()
                {
                    self.page = StudioPage::RenderQueue;
                }
                if SidebarItem::with_icon(Icon::Widgets, "Components")
                    .badge("New")
                    .selected(self.page == StudioPage::Components)
                    .show(ui)
                    .clicked()
                {
                    self.page = StudioPage::Components;
                }
                SidebarItem::with_icon(Icon::Layers, "Assets").show(ui);
                SidebarItem::with_icon(Icon::Clock, "History").show(ui);
                SidebarItem::with_icon(Icon::Chart, "Usage").show(ui);
                ui.add_space(18.0);

                section_label(ui, &theme, "Projects");
                for (name, color) in [
                    ("Coastline", Color32::from_rgb(56, 189, 248)),
                    ("Night market", Color32::from_rgb(245, 158, 11)),
                    ("Product shots", Color32::from_rgb(167, 139, 250)),
                    ("Logo reveal", Color32::from_rgb(52, 211, 153)),
                ] {
                    project_row(ui, &theme, name, color);
                }
                ui.add_space(18.0);

                self.appearance(ui);

                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 10.0);
                    if SidebarItem::with_icon(Icon::Sun, "Classic showcase")
                        .show(ui)
                        .clicked()
                    {
                        self.classic = Some(DemoApp::default());
                    }
                    gpu_card(ui, &theme);
                });
            });
        });
    }

    fn appearance(&mut self, ui: &mut Ui) {
        let theme = self.theme.clone();
        section_label(ui, &theme, "Appearance");
        ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

        let mut mode_idx = usize::from(self.mode == ThemeMode::Light);
        if ui
            .add(GlassSegmented::new(&mut mode_idx, &["Dark", "Light"]).fill_width(true))
            .changed()
        {
            self.mode = if mode_idx == 0 {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            };
            self.rebuild_theme();
        }

        let mut material_idx = usize::from(self.material == SurfaceStyle::Frosted);
        if ui
            .add(GlassSegmented::new(&mut material_idx, &["Layered", "All glass"]).fill_width(true))
            .changed()
        {
            self.material = if material_idx == 0 {
                SurfaceStyle::Layered
            } else {
                SurfaceStyle::Frosted
            };
            self.rebuild_theme();
        }

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            let dark = self.mode == ThemeMode::Dark;
            for accent in AccentPalette::ALL {
                let (rect, response) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::click());
                let response = response.on_hover_text(accent.label());
                let painter = ui.painter();
                painter.circle_filled(rect.center(), 8.0, accent.prism(dark));
                if accent == self.accent {
                    painter.circle_stroke(
                        rect.center(),
                        10.5,
                        Stroke::new(1.5, theme.palette.text_primary),
                    );
                }
                if response.clicked() {
                    self.set_accent(accent);
                }
            }
        });
        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
    }

    // ----------------------------------------------------------------------------------------
    // Main column

    fn main_column(&mut self, ui: &mut Ui, rect: Rect) {
        let theme = self.theme.clone();
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ui.set_clip_rect(rect);
            let header = Rect::from_min_size(rect.min, vec2(rect.width(), 56.0));
            self.toolbar(ui, header);
            ui.painter().hline(
                rect.x_range(),
                header.max.y,
                Stroke::new(1.0, theme.neutrals.border),
            );

            let body = Rect::from_min_max(pos2(rect.min.x, header.max.y + 1.0), rect.max);
            ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
                ui.add_space(16.0);
                let content = ui.max_rect().shrink2(vec2(20.0, 0.0));

                // Filters + sort.
                let row = Rect::from_min_size(
                    pos2(content.min.x, ui.cursor().min.y),
                    vec2(content.width(), 30.0),
                );
                ui.scope_builder(UiBuilder::new().max_rect(row), |ui| {
                    ui.horizontal_centered(|ui| {
                        // Zero counts are hidden (`None`) instead of showing "0".
                        let counts = self.filter_counts().map(|c| (c > 0).then_some(c));
                        ui.add(
                            GlassSegmented::new(
                                &mut self.filter,
                                &["All", "Active", "Done", "Failed"],
                            )
                            .optional_counts(&counts),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            const SORTS: [&str; 3] = ["Priority", "Progress", "Name"];
                            let sort = ui.add(
                                GlassButton::ghost(format!("Sort: {}", SORTS[self.sort]))
                                    .icon(Icon::ChevronDown)
                                    .min_size(vec2(0.0, 28.0)),
                            );
                            let _ = GlassMenu::new(&sort).min_width(160.0).show(|ui| {
                                GlassMenu::section(ui, "Sort by");
                                for (i, label) in SORTS.into_iter().enumerate() {
                                    if ui
                                        .add(MenuItem::new(label).checked(self.sort == i))
                                        .clicked()
                                    {
                                        self.sort = i;
                                    }
                                }
                            });
                        });
                    });
                });
                ui.add_space(16.0);

                // Stats strip.
                let stats = Rect::from_min_size(
                    pos2(content.min.x, ui.cursor().min.y),
                    vec2(content.width(), 70.0),
                );
                stats_strip(ui, &theme, stats);
                ui.add_space(86.0);

                // Table.
                let table = Rect::from_min_max(pos2(rect.min.x, ui.cursor().min.y), body.max);
                ui.scope_builder(UiBuilder::new().max_rect(table), |ui| {
                    self.jobs_table(ui, &theme);
                });
            });

            self.floating_bar(ui, &theme, rect);
        });
    }

    fn filter_counts(&self) -> [u32; 4] {
        let count = |f: &dyn Fn(&Job) -> bool| self.jobs.iter().filter(|j| f(j)).count() as u32;
        [
            self.jobs.len() as u32,
            count(&|j| matches!(j.status, JobStatus::Rendering | JobStatus::Queued)),
            count(&|j| j.status == JobStatus::Done),
            count(&|j| j.status == JobStatus::Failed),
        ]
    }

    fn toolbar(&mut self, ui: &mut Ui, rect: Rect) {
        let theme = self.theme.clone();
        let ctx = ui.ctx().clone();
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(rect.shrink2(vec2(20.0, 0.0)))
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 1.0);
                    ui.add_space(9.0);
                    ui.label(styled(
                        &ctx,
                        "Render queue",
                        15.0,
                        FontWeight::Semibold,
                        theme.palette.text_primary,
                    ));
                    let rendering = self
                        .jobs
                        .iter()
                        .filter(|j| j.status == JobStatus::Rendering)
                        .count();
                    let queued = self
                        .jobs
                        .iter()
                        .filter(|j| j.status == JobStatus::Queued)
                        .count();
                    ui.label(styled(
                        &ctx,
                        &format!("{rendering} rendering · {queued} queued"),
                        12.0,
                        FontWeight::Regular,
                        theme.palette.text_muted,
                    ));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = vec2(10.0, 0.0);
                    if ui
                        .add(GlassButton::primary("New render").icon(Icon::Plus))
                        .clicked()
                    {
                        self.toasts.add(
                            Toast::success("Render queued")
                                .body("“Untitled” · 1920×1080 · 512 samples"),
                        );
                    }
                    let filter = ui.add(GlassButton::secondary("Filter").icon(Icon::Filter));
                    let _ = GlassMenu::new(&filter).show(|ui| {
                        ui.add(MenuItem::new("Only my jobs").icon(Icon::User));
                        ui.add(MenuItem::new("GPU jobs").icon(Icon::Bolt));
                        GlassMenu::separator(ui);
                        ui.add(MenuItem::new("Clear filters").icon(Icon::Close));
                    });
                    ui.add(
                        GlassTextField::new(&mut self.search)
                            .placeholder("Search jobs")
                            .icon(Icon::Search)
                            .width(200.0),
                    );
                });
            },
        );
    }

    fn jobs_table(&mut self, ui: &mut Ui, theme: &PrismTheme) {
        let ctx = ui.ctx().clone();
        let full = ui.max_rect();
        let cols = TableColumns::new(full.shrink2(vec2(20.0, 0.0)));

        // Header row.
        let (header, _) = ui.allocate_exact_size(vec2(full.width(), 36.0), Sense::hover());
        let header_font = prism_font(&ctx, 12.0, FontWeight::Medium);
        let painter = ui.painter();
        for (x, text) in [
            (cols.name, "Name"),
            (cols.status, "Status"),
            (cols.progress, "Progress"),
            (cols.eta, "ETA"),
        ] {
            painter.text(
                pos2(x, header.center().y),
                Align2::LEFT_CENTER,
                text,
                header_font.clone(),
                theme.palette.text_muted,
            );
        }
        painter.hline(
            full.x_range(),
            header.max.y - 0.5,
            Stroke::new(1.0, theme.neutrals.border),
        );
        let mut all = self.jobs.iter().all(|j| j.checked);
        if ui
            .place(
                Rect::from_center_size(pos2(cols.check + 8.0, header.center().y), vec2(18.0, 18.0)),
                GlassCheckbox::new(&mut all),
            )
            .accessible_name("Select all jobs")
            .changed()
        {
            for job in &mut self.jobs {
                job.checked = all;
            }
        }

        let filter = self.filter;
        let needle = self.search.to_lowercase();
        let [ar, ag, ab, _] = theme.palette.accent.to_srgba_unmultiplied();
        let dark = theme.mode == ThemeMode::Dark;
        let (soft, strong) = if dark { (20, 41) } else { (13, 26) };

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                for (index, job) in self.jobs.iter_mut().enumerate() {
                    let visible = match filter {
                        1 => matches!(job.status, JobStatus::Rendering | JobStatus::Queued),
                        2 => job.status == JobStatus::Done,
                        3 => job.status == JobStatus::Failed,
                        _ => true,
                    } && (needle.is_empty()
                        || job.name.to_lowercase().contains(&needle));
                    if !visible {
                        continue;
                    }
                    let (row, response) =
                        ui.allocate_exact_size(vec2(full.width(), 52.0), Sense::click());
                    if response.clicked() {
                        self.focused = index;
                    }
                    let painter = ui.painter();
                    let bg = if index == self.focused {
                        Color32::from_rgba_unmultiplied(ar, ag, ab, strong)
                    } else if job.checked {
                        Color32::from_rgba_unmultiplied(ar, ag, ab, soft)
                    } else if response.hovered() {
                        theme.neutrals.hover
                    } else {
                        Color32::TRANSPARENT
                    };
                    painter.rect_filled(row, CornerRadius::ZERO, bg);
                    painter.hline(
                        row.x_range(),
                        row.max.y - 0.5,
                        Stroke::new(1.0, theme.neutrals.border),
                    );

                    let cy = row.center().y;
                    let name_w = cols.status - cols.name - 12.0;
                    truncated_text(
                        painter,
                        pos2(cols.name, cy - 8.0),
                        job.name,
                        prism_font(&ctx, 13.0, FontWeight::Medium),
                        theme.palette.text_primary,
                        name_w,
                    );
                    truncated_text(
                        painter,
                        pos2(cols.name, cy + 9.0),
                        &format!(
                            "{} · {} · {} frames",
                            job.engine, job.resolution, job.frames
                        ),
                        prism_font(&ctx, 12.0, FontWeight::Regular),
                        theme.palette.text_muted,
                        name_w,
                    );
                    painter.circle_filled(
                        pos2(cols.status + 3.0, cy),
                        3.0,
                        job.status.color(theme),
                    );
                    painter.text(
                        pos2(cols.status + 13.0, cy),
                        Align2::LEFT_CENTER,
                        job.status.label(),
                        prism_font(&ctx, 12.0, FontWeight::Medium),
                        theme.palette.text_secondary,
                    );
                    painter.text(
                        pos2(cols.progress + 150.0, cy),
                        Align2::RIGHT_CENTER,
                        format!("{:.0}%", job.progress * 100.0),
                        prism_numeric_font(&ctx, 12.0, FontWeight::Regular),
                        theme.palette.text_secondary,
                    );
                    let eta_color = match job.status {
                        JobStatus::Failed => theme.palette.danger,
                        JobStatus::Rendering => theme.palette.text_secondary,
                        _ => theme.palette.text_muted,
                    };
                    painter.text(
                        pos2(cols.eta, cy),
                        Align2::LEFT_CENTER,
                        job.eta,
                        prism_numeric_font(&ctx, 12.0, FontWeight::Regular),
                        eta_color,
                    );
                    for dx in [-5.0, 0.0, 5.0] {
                        painter.circle_filled(
                            pos2(cols.more + 14.0 + dx, cy),
                            1.5,
                            theme.palette.text_muted,
                        );
                    }

                    let bar_color = match job.status {
                        JobStatus::Failed => theme.palette.danger,
                        JobStatus::Rendering => theme.palette.accent,
                        _ => theme.neutrals.muted,
                    };
                    ui.place(
                        Rect::from_min_size(pos2(cols.progress, cy - 2.0), vec2(108.0, 4.0)),
                        GlassProgress::new(job.progress)
                            .width(108.0)
                            .color(bar_color),
                    );
                    ui.place(
                        Rect::from_center_size(pos2(cols.check + 8.0, cy), vec2(18.0, 18.0)),
                        GlassCheckbox::new(&mut job.checked),
                    )
                    .accessible_name(format!("Select {}", job.name));
                }
                ui.add_space(80.0);
            });
    }

    fn floating_bar(&mut self, ui: &mut Ui, theme: &PrismTheme, main: Rect) {
        let selected = self.jobs.iter().filter(|j| j.checked).count();
        if selected == 0 {
            return;
        }
        let size = vec2(392.0, 44.0);
        let rect = Rect::from_center_size(
            pos2(main.center().x, main.max.y - 20.0 - size.y * 0.5),
            size,
        );
        // The GPU blur samples the backdrop, not live widgets, so a bar floating over table text
        // uses a dense frosted film (no backdrop blur) to stay legible.
        let tokens = theme
            .surface(GlassElevation::Floating)
            .with_blur_radius(0.0)
            .with_opacity_scale(1.12);
        paint_glass_surface_ui(ui, rect, &tokens, false);
        let ctx = ui.ctx().clone();
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(rect.shrink2(vec2(14.0, 6.0)))
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                ui.label(styled(
                    &ctx,
                    &format!("{selected} selected"),
                    13.0,
                    FontWeight::Medium,
                    theme.palette.text_primary,
                ));
                ui.add_space(8.0);
                let (div, _) = ui.allocate_exact_size(vec2(1.0, 16.0), Sense::hover());
                ui.painter()
                    .rect_filled(div, CornerRadius::ZERO, theme.neutrals.border_strong);
                ui.add_space(8.0);
                let small = vec2(0.0, 30.0);
                if ui
                    .add(GlassButton::ghost("Pause").min_size(small))
                    .clicked()
                {
                    self.set_checked_status(JobStatus::Queued);
                }
                ui.add(GlassButton::ghost("Prioritize").min_size(small));
                if ui
                    .add(GlassButton::ghost("Cancel").min_size(small))
                    .clicked()
                {
                    self.confirm_cancel = true;
                }
                if ui
                    .add(GlassButton::ghost("Clear").min_size(small))
                    .clicked()
                {
                    for job in &mut self.jobs {
                        job.checked = false;
                    }
                }
            },
        );
    }

    fn set_checked_status(&mut self, status: JobStatus) {
        for job in self.jobs.iter_mut().filter(|j| j.checked) {
            job.status = status;
            job.eta = match status {
                JobStatus::Failed => "Cancelled",
                _ => "—",
            };
            job.checked = false;
        }
    }

    // ----------------------------------------------------------------------------------------
    // Inspector

    fn inspector(&mut self, ui: &mut Ui, rect: Rect) {
        let theme = self.theme.clone();
        let ctx = ui.ctx().clone();
        let job = self.jobs[self.focused.min(self.jobs.len() - 1)].clone();
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ui.set_clip_rect(rect);
            // Header.
            let header = Rect::from_min_size(rect.min, vec2(rect.width(), 56.0));
            ui.scope_builder(
                UiBuilder::new()
                    .max_rect(header.shrink2(vec2(20.0, 0.0)))
                    .layout(Layout::top_down(Align::Min)),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 1.0);
                    ui.add_space(9.0);
                    ui.label(styled(
                        &ctx,
                        job.name,
                        14.0,
                        FontWeight::Semibold,
                        theme.palette.text_primary,
                    ));
                    ui.label(styled(
                        &ctx,
                        &format!("Started 13:48 · {}", job.engine),
                        12.0,
                        FontWeight::Regular,
                        theme.palette.text_muted,
                    ));
                },
            );
            ui.painter().hline(
                rect.x_range(),
                header.max.y,
                Stroke::new(1.0, theme.neutrals.border),
            );

            // Footer.
            let footer = Rect::from_min_max(pos2(rect.min.x, rect.max.y - 66.0), rect.max);
            ui.painter().hline(
                rect.x_range(),
                footer.min.y,
                Stroke::new(1.0, theme.neutrals.border),
            );
            ui.scope_builder(
                UiBuilder::new()
                    .max_rect(footer.shrink2(vec2(20.0, 16.0)))
                    .layout(Layout::left_to_right(Align::Center)),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
                    let w = (ui.available_width() - 8.0) * 0.5;
                    ui.add(GlassButton::secondary("Pause").min_size(vec2(w, 34.0)));
                    ui.add(GlassButton::primary("Apply changes").min_size(vec2(w, 34.0)));
                },
            );

            // Body.
            let body = Rect::from_min_max(
                pos2(rect.min.x + 20.0, header.max.y + 20.0),
                pos2(rect.max.x - 20.0, footer.min.y - 12.0),
            );
            ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                let (preview, _) =
                    ui.allocate_exact_size(vec2(body.width(), 150.0), Sense::hover());
                paint_preview(ui, &theme, preview, &job);
                ui.add_space(18.0);

                let done = (job.progress * job.frames as f32).round() as u32;
                two_column(
                    ui,
                    styled(
                        &ctx,
                        "Progress",
                        12.0,
                        FontWeight::Medium,
                        theme.palette.text_secondary,
                    ),
                    numeric(
                        &ctx,
                        &format!("{:.0}%", job.progress * 100.0),
                        FontWeight::Medium,
                        theme.palette.text_primary,
                    ),
                );
                ui.add_space(8.0);
                ui.add(
                    GlassProgress::new(job.progress)
                        .height(6.0)
                        .color(job.status.color(&theme)),
                );
                ui.add_space(8.0);
                two_column(
                    ui,
                    numeric(
                        &ctx,
                        &format!("{done} / {} frames", job.frames),
                        FontWeight::Regular,
                        theme.palette.text_muted,
                    ),
                    numeric(&ctx, job.eta, FontWeight::Regular, theme.palette.text_muted),
                );
                ui.add_space(18.0);
                let (div, _) = ui.allocate_exact_size(vec2(body.width(), 1.0), Sense::hover());
                ui.painter()
                    .rect_filled(div, CornerRadius::ZERO, theme.neutrals.border);
                ui.add_space(18.0);

                ui.label(styled(
                    &ctx,
                    "Output",
                    13.0,
                    FontWeight::Semibold,
                    theme.palette.text_primary,
                ));
                ui.add_space(14.0);
                ui.label(styled(
                    &ctx,
                    "Resolution",
                    12.0,
                    FontWeight::Medium,
                    theme.palette.text_secondary,
                ));
                ui.add_space(8.0);
                ui.add(
                    GlassSegmented::new(&mut self.resolution, &["1080p", "1440p", "4K"])
                        .fill_width(true),
                );
                ui.add_space(14.0);
                ui.add(
                    GlassSlider::new(&mut self.samples, 0.0..=1024.0)
                        .label("Samples")
                        .decimals(0),
                );
                ui.add_space(10.0);
                toggle_row(
                    ui,
                    &theme,
                    "Denoise",
                    "AI denoiser on final pass",
                    &mut self.denoise,
                );
                ui.add_space(6.0);
                toggle_row(
                    ui,
                    &theme,
                    "Motion blur",
                    "Adds ~20% render time",
                    &mut self.motion_blur,
                );
                ui.add_space(14.0);
                ui.label(styled(
                    &ctx,
                    "Format",
                    12.0,
                    FontWeight::Medium,
                    theme.palette.text_secondary,
                ));
                ui.add_space(8.0);
                ComboBox::from_id_salt("prism_studio_format")
                    .width(body.width())
                    .selected_text(FORMATS[self.format])
                    .show_ui(ui, |ui| {
                        for (i, f) in FORMATS.iter().enumerate() {
                            ui.selectable_value(&mut self.format, i, *f);
                        }
                    })
                    .response
                    .accessible_name("Format");
            });
        });
    }
}

// --------------------------------------------------------------------------------------------
// Helpers

struct TableColumns {
    check: f32,
    name: f32,
    status: f32,
    progress: f32,
    eta: f32,
    more: f32,
}

impl TableColumns {
    fn new(rect: Rect) -> Self {
        let gap = 12.0;
        let more = rect.max.x - 28.0;
        let eta = more - gap - 96.0;
        let progress = eta - gap - 150.0;
        let status = progress - gap - 104.0;
        Self {
            check: rect.min.x,
            name: rect.min.x + 28.0 + gap,
            status,
            progress,
            eta,
            more,
        }
    }
}

fn styled(
    ctx: &egui::Context,
    text: &str,
    size: f32,
    weight: FontWeight,
    color: Color32,
) -> RichText {
    RichText::new(text)
        .font(prism_font(ctx, size, weight))
        .color(color)
}

fn numeric(ctx: &egui::Context, text: &str, weight: FontWeight, color: Color32) -> RichText {
    RichText::new(text)
        .font(prism_numeric_font(ctx, 12.0, weight))
        .color(color)
}

fn two_column(ui: &mut Ui, left: RichText, right: RichText) {
    ui.horizontal(|ui| {
        ui.label(left);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(right);
        });
    });
}

fn section_label(ui: &mut Ui, theme: &PrismTheme, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(styled(
            ui.ctx(),
            text,
            12.0,
            FontWeight::Medium,
            theme.palette.text_muted,
        ));
    });
    ui.add_space(4.0);
}

fn brand_header(ui: &mut Ui, theme: &PrismTheme) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::hover());
    let logo = Rect::from_min_size(
        pos2(rect.min.x + 6.0, rect.center().y - 14.0),
        vec2(28.0, 28.0),
    );
    let painter = ui.painter();
    paint_glass_gradient_fill(
        painter,
        logo,
        8.0,
        theme.palette.accent,
        Color32::from_rgb(168, 85, 247),
        Stroke::NONE,
        Color32::from_rgba_unmultiplied(255, 255, 255, 64),
    );
    let c = logo.center();
    let tri = vec![
        pos2(c.x, c.y - 6.5),
        pos2(c.x + 7.0, c.y + 5.5),
        pos2(c.x - 7.0, c.y + 5.5),
    ];
    painter.add(egui::Shape::closed_line(
        tri,
        Stroke::new(1.6, Color32::WHITE),
    ));
    painter.line_segment(
        [pos2(c.x, c.y - 6.5), pos2(c.x, c.y + 5.5)],
        Stroke::new(1.6, Color32::WHITE),
    );
    let ctx = ui.ctx();
    painter.text(
        pos2(logo.max.x + 10.0, rect.center().y - 8.0),
        Align2::LEFT_CENTER,
        "Prism Studio",
        prism_font(ctx, 13.0, FontWeight::Semibold),
        theme.palette.text_primary,
    );
    painter.text(
        pos2(logo.max.x + 10.0, rect.center().y + 8.0),
        Align2::LEFT_CENTER,
        "Personal",
        prism_font(ctx, 12.0, FontWeight::Regular),
        theme.palette.text_muted,
    );
}

fn project_row(ui: &mut Ui, theme: &PrismTheme, name: &str, color: Color32) {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, CornerRadius::same(8), theme.neutrals.hover);
    }
    let dot = Rect::from_center_size(pos2(rect.min.x + 18.0, rect.center().y), vec2(8.0, 8.0));
    painter.rect_filled(dot, CornerRadius::same(3), color);
    painter.text(
        pos2(rect.min.x + 36.0, rect.center().y),
        Align2::LEFT_CENTER,
        name,
        prism_font(ui.ctx(), 13.0, FontWeight::Regular),
        if response.hovered() {
            theme.palette.text_primary
        } else {
            theme.palette.text_secondary
        },
    );
}

fn gpu_card(ui: &mut Ui, theme: &PrismTheme) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 74.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(10), theme.neutrals.hover);
    painter.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0, theme.neutrals.border),
        egui::StrokeKind::Inside,
    );
    let ctx = ui.ctx().clone();
    let inner = rect.shrink(12.0);
    painter.text(
        pos2(inner.min.x, inner.min.y + 7.0),
        Align2::LEFT_CENTER,
        "GPU 0 · RTX 4080",
        prism_font(&ctx, 12.0, FontWeight::Medium),
        theme.palette.text_primary,
    );
    painter.text(
        pos2(inner.max.x, inner.min.y + 7.0),
        Align2::RIGHT_CENTER,
        "87%",
        prism_numeric_font(&ctx, 12.0, FontWeight::Medium),
        theme.palette.text_secondary,
    );
    painter.text(
        pos2(inner.min.x, inner.max.y - 7.0),
        Align2::LEFT_CENTER,
        "11.2 of 16 GB VRAM",
        prism_numeric_font(&ctx, 12.0, FontWeight::Regular),
        theme.palette.text_muted,
    );
    ui.place(
        Rect::from_min_size(
            pos2(inner.min.x, inner.center().y - 2.0),
            vec2(inner.width(), 4.0),
        ),
        GlassProgress::new(0.7).width(inner.width()),
    );
}

fn stats_strip(ui: &mut Ui, theme: &PrismTheme, rect: Rect) {
    paint_glass_surface_ui(ui, rect, &theme.surface(GlassElevation::Subtle), false);
    let ctx = ui.ctx().clone();
    let painter = ui.painter();
    let cell_w = rect.width() / 3.0;
    let stats = [
        ("Throughput", "42", "frames/min", Some("+8%")),
        ("Avg. queue time", "18", "min", None),
        ("GPU utilization", "87", "%", None),
    ];
    for (i, (label, value, unit, delta)) in stats.into_iter().enumerate() {
        let x0 = rect.min.x + cell_w * i as f32;
        if i > 0 {
            painter.vline(x0, rect.y_range(), Stroke::new(1.0, theme.neutrals.border));
        }
        painter.text(
            pos2(x0 + 16.0, rect.min.y + 22.0),
            Align2::LEFT_CENTER,
            label,
            prism_font(&ctx, 12.0, FontWeight::Regular),
            theme.palette.text_muted,
        );
        let value_rect = painter.text(
            pos2(x0 + 16.0, rect.min.y + 47.0),
            Align2::LEFT_CENTER,
            value,
            prism_numeric_font(&ctx, 22.0, FontWeight::Semibold),
            theme.palette.text_primary,
        );
        painter.text(
            pos2(value_rect.max.x + 6.0, value_rect.max.y - 4.0),
            Align2::LEFT_BOTTOM,
            unit,
            prism_font(&ctx, 12.0, FontWeight::Regular),
            theme.palette.text_muted,
        );
        if let Some(delta) = delta {
            painter.text(
                pos2(x0 + cell_w - 16.0, value_rect.max.y - 4.0),
                Align2::RIGHT_BOTTOM,
                delta,
                prism_numeric_font(&ctx, 12.0, FontWeight::Medium),
                theme.palette.success,
            );
        }
    }
}

fn toggle_row(ui: &mut Ui, theme: &PrismTheme, title: &str, subtitle: &str, value: &mut bool) {
    let ctx = ui.ctx().clone();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 1.0);
            ui.label(styled(
                &ctx,
                title,
                13.0,
                FontWeight::Medium,
                theme.palette.text_primary,
            ));
            ui.label(styled(
                &ctx,
                subtitle,
                12.0,
                FontWeight::Regular,
                theme.palette.text_muted,
            ));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(GlassToggle::new(value)).accessible_name(title);
        });
    });
}

/// A stand-in render preview: a night street with warm lantern lights, plus a glass frame pill.
fn paint_preview(ui: &mut Ui, theme: &PrismTheme, rect: Rect, job: &Job) {
    let painter = ui.painter().with_clip_rect(rect.shrink(1.0));
    paint_glass_gradient_fill(
        &painter,
        rect,
        10.0,
        Color32::from_rgb(10, 14, 42),
        Color32::from_rgb(58, 35, 80),
        Stroke::NONE,
        Color32::TRANSPARENT,
    );
    let at = |x: f32, y: f32| {
        pos2(
            rect.min.x + rect.width() * x,
            rect.min.y + rect.height() * y,
        )
    };
    paint_radial_glow_orb(
        &painter,
        at(0.5, 1.05),
        rect.width() * 0.55,
        Color32::from_rgba_unmultiplied(194, 98, 45, 210),
    );
    for (x, y, r, col) in [
        (0.18, 0.70, 22.0, Color32::from_rgb(255, 176, 90)),
        (0.34, 0.62, 16.0, Color32::from_rgb(255, 120, 80)),
        (0.58, 0.74, 22.0, Color32::from_rgb(255, 200, 120)),
        (0.76, 0.64, 16.0, Color32::from_rgb(255, 110, 140)),
    ] {
        paint_radial_glow_orb(&painter, at(x, y), r, col);
        painter.circle_filled(at(x, y), 2.5, Color32::from_rgb(255, 240, 210));
    }
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0, theme.neutrals.border),
        egui::StrokeKind::Inside,
    );

    let done = (job.progress * job.frames as f32).round() as u32;
    let label = format!("Frame {done} / {}", job.frames);
    let font = prism_numeric_font(ui.ctx(), 11.0, FontWeight::Medium);
    let galley = ui.painter().layout_no_wrap(label, font, Color32::WHITE);
    let pill = Rect::from_min_size(
        pos2(rect.min.x + 10.0, rect.max.y - 10.0 - 22.0),
        vec2(galley.size().x + 16.0, 22.0),
    );
    ui.painter().rect_filled(
        pill,
        CornerRadius::same(11),
        Color32::from_rgba_unmultiplied(16, 16, 24, 150),
    );
    ui.painter().rect_stroke(
        pill,
        CornerRadius::same(11),
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 42)),
        egui::StrokeKind::Inside,
    );
    ui.painter().galley(
        pos2(pill.min.x + 8.0, pill.center().y - galley.size().y * 0.5),
        galley,
        Color32::WHITE,
    );
    if job.status == JobStatus::Rendering {
        // Over imagery, status chips use the same dark glass pill as the frame counter.
        let font = prism_font(ui.ctx(), 11.0, FontWeight::Medium);
        let live = ui
            .painter()
            .layout_no_wrap("Live".to_owned(), font, Color32::WHITE);
        let chip = Rect::from_min_size(
            pos2(rect.max.x - 10.0 - live.size().x - 26.0, rect.min.y + 10.0),
            vec2(live.size().x + 26.0, 22.0),
        );
        ui.painter().rect_filled(
            chip,
            CornerRadius::same(11),
            Color32::from_rgba_unmultiplied(16, 16, 24, 150),
        );
        ui.painter().rect_stroke(
            chip,
            CornerRadius::same(11),
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 42)),
            egui::StrokeKind::Inside,
        );
        ui.painter().circle_filled(
            pos2(chip.min.x + 11.0, chip.center().y),
            3.0,
            Color32::from_rgb(61, 214, 140),
        );
        ui.painter().galley(
            pos2(chip.min.x + 19.0, chip.center().y - live.size().y * 0.5),
            live,
            Color32::WHITE,
        );
    }
}

/// Paints single-line text, cutting it with an ellipsis when it is wider than `max_width`.
fn truncated_text(
    painter: &egui::Painter,
    left_center: egui::Pos2,
    text: &str,
    font: egui::FontId,
    color: Color32,
    max_width: f32,
) {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(max_width.max(8.0));
    let galley = painter.layout_job(job);
    let pos = pos2(left_center.x, left_center.y - galley.size().y * 0.5);
    painter.galley(pos, galley, color);
}
