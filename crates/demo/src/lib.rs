//! Interactive showcase application for `egui-prismglass`.
//!
//! Designed as both a live playground (`cargo run -p demo`) and a visual reference
//! for fallback-first glassmorphism across dark (`Obsidian`, `Prism Aurora`) and
//! light (`Frosted Light`) themes.

#![allow(missing_docs)]

use egui::{
    Align, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, TextEdit, Ui, pos2, vec2,
};
use egui_prismglass::{
    AccentPalette, BackdropScene, BadgeVariant, BlurCapability, Easing, GlassBadge, GlassButton,
    GlassCard, GlassElevation, GlassPanel, GlassSidebar, GlassSlider, GlassToggle, PrismAnimExt,
    PrismContextExt, PrismPreset, PrismTheme, SidebarIcon, SidebarItem, SpringConfig, SurfaceStyle,
    ThemeMode, paint_prism_backdrop_ui,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoTab {
    Overview,
    Components,
    Motion,
    Tokens,
}

/// Complete state of the `egui-prismglass` showcase dashboard.
#[allow(clippy::struct_excessive_bools)]
pub struct DemoApp {
    pub preset: PrismPreset,
    pub theme: PrismTheme,
    pub active_tab: DemoTab,
    pub orb_intensity: f32,
    pub animate_orbs: bool,
    pub prompt_input: String,
    pub last_action: String,
    pub demo_slider_a: f32,
    pub demo_slider_b: f32,
    pub demo_toggle_a: bool,
    pub demo_toggle_b: bool,
    pub motion_toggled: bool,
    pub click_count: u32,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self::with_preset(PrismPreset::Frost)
    }
}

impl DemoApp {
    /// Creates a new demo instance initialized to a specific [`PrismPreset`].
    pub fn with_preset(preset: PrismPreset) -> Self {
        Self {
            preset,
            theme: preset.into_theme(),
            active_tab: DemoTab::Overview,
            orb_intensity: 1.0,
            animate_orbs: false,
            prompt_input: "Synthesize adaptive liquid-glass tokens for high-DPI desktop apps"
                .to_owned(),
            last_action: "PrismGlass v0.2.0 — Real GPU Liquid Glass, Neumorphic Flat & 8 Palettes"
                .to_owned(),
            demo_slider_a: 22.0,
            demo_slider_b: 0.35,
            demo_toggle_a: true,
            demo_toggle_b: false,
            motion_toggled: true,
            click_count: 0,
        }
    }

    /// Switches to a new preset while preserving current tab, accent palette, and backdrop/blur toggles.
    pub fn set_preset(&mut self, preset: PrismPreset) {
        let scene = self.theme.backdrop_scene;
        let real_blur = self.theme.real_blur_enabled;
        let accent = self.theme.accent_palette;
        self.preset = preset;
        self.theme = preset.into_theme();
        if preset != PrismPreset::Glacial {
            self.theme.backdrop_scene = scene;
        }
        self.theme.real_blur_enabled = real_blur;
        self.theme.set_accent_palette(accent);
    }

    /// Renders the full showcase UI into an `egui::Ui` (used by both `eframe` and headless snapshots).
    pub fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        // Keep live inspector tweaks synchronized with context memory.
        ctx.set_prism_theme(self.theme.clone());

        let time_secs = if self.animate_orbs {
            ctx.request_repaint();
            ctx.input(|i| i.time) as f32
        } else {
            0.0
        };

        let viewport = ui.max_rect();
        paint_prism_backdrop_ui(ui, viewport, &self.theme, time_secs, self.orb_intensity);

        Frame::NONE.inner_margin(Margin::same(18)).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(18.0, 14.0);

                // Left floating glass sidebar.
                self.render_sidebar(ui);

                // Right main workspace column.
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing = vec2(14.0, 14.0);

                    self.render_top_bar(ui);

                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = vec2(14.0, 14.0);
                            match self.active_tab {
                                DemoTab::Overview => self.render_overview(ui),
                                DemoTab::Components => self.render_components(ui),
                                DemoTab::Motion => self.render_motion(ui),
                                DemoTab::Tokens => self.render_tokens(ui),
                            }
                        });
                });
            });
        });
    }

    fn render_sidebar(&mut self, ui: &mut Ui) {
        GlassSidebar::new().width(228.0).show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(7.0, 7.0);

            // Brand header with vector prism emblem.
            ui.horizontal(|ui| {
                paint_prism_logo(ui, &self.theme);
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 1.0);
                    ui.label(
                        RichText::new("PrismGlass")
                            .size(16.0)
                            .strong()
                            .color(self.theme.palette.text_primary),
                    );
                    ui.label(
                        RichText::new("Design Kit  v0.2")
                            .size(11.0)
                            .color(self.theme.palette.text_muted),
                    );
                });
            });

            ui.add_space(8.0);
            ui.label(
                RichText::new("WORKSPACE")
                    .size(10.5)
                    .strong()
                    .color(self.theme.palette.text_muted),
            );

            if SidebarItem::new(SidebarIcon::Prism, "Overview")
                .badge("GPU")
                .selected(self.active_tab == DemoTab::Overview)
                .show(ui)
                .clicked()
            {
                self.active_tab = DemoTab::Overview;
            }

            if SidebarItem::new(SidebarIcon::Grid, "Components")
                .badge("7")
                .selected(self.active_tab == DemoTab::Components)
                .show(ui)
                .clicked()
            {
                self.active_tab = DemoTab::Components;
            }

            if SidebarItem::new(SidebarIcon::Sliders, "Motion Kit")
                .selected(self.active_tab == DemoTab::Motion)
                .show(ui)
                .clicked()
            {
                self.active_tab = DemoTab::Motion;
            }

            if SidebarItem::new(SidebarIcon::Tokens, "Design Tokens")
                .selected(self.active_tab == DemoTab::Tokens)
                .show(ui)
                .clicked()
            {
                self.active_tab = DemoTab::Tokens;
            }

            ui.add_space(6.0);
            ui.label(
                RichText::new("GLASS THEMES")
                    .size(10.0)
                    .strong()
                    .color(self.theme.palette.text_muted),
            );

            for preset in [PrismPreset::Frost, PrismPreset::Onyx, PrismPreset::Glacial] {
                let active = self.preset == preset;
                if ui
                    .add(
                        GlassButton::secondary(preset.label())
                            .selected(active)
                            .min_size(vec2(ui.available_width(), 28.0)),
                    )
                    .clicked()
                {
                    self.set_preset(preset);
                    self.last_action = format!("Switched theme preset to {}", preset.label());
                }
            }

            ui.add_space(6.0);
            ui.label(
                RichText::new("NEUMORPHIC FLAT")
                    .size(10.0)
                    .strong()
                    .color(self.theme.palette.text_muted),
            );

            for preset in [PrismPreset::MinimalLight, PrismPreset::MinimalDark] {
                let active = self.preset == preset;
                if ui
                    .add(
                        GlassButton::secondary(preset.label())
                            .selected(active)
                            .min_size(vec2(ui.available_width(), 28.0)),
                    )
                    .clicked()
                {
                    self.set_preset(preset);
                    self.last_action = format!("Switched theme preset to {}", preset.label());
                }
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("COLOR PALETTE")
                        .size(10.0)
                        .strong()
                        .color(self.theme.palette.text_muted),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(self.theme.accent_palette.label())
                            .size(10.5)
                            .strong()
                            .color(self.theme.palette.accent),
                    );
                });
            });
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                for palette_choice in AccentPalette::ALL {
                    let selected = self.theme.accent_palette == palette_choice;
                    let (swatch_rect, resp) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    let center = swatch_rect.center();
                    let fill = palette_choice.swatch();
                    if selected {
                        ui.painter().circle_stroke(
                            center,
                            9.5,
                            Stroke::new(1.8, self.theme.palette.text_primary),
                        );
                    } else if resp.hovered() {
                        ui.painter().circle_stroke(
                            center,
                            9.0,
                            Stroke::new(1.2, self.theme.palette.text_muted),
                        );
                    }
                    ui.painter()
                        .circle_filled(center, if selected { 6.6 } else { 7.5 }, fill);
                    if resp.on_hover_text(palette_choice.label()).clicked() {
                        self.theme.set_accent_palette(palette_choice);
                        self.last_action =
                            format!("Applied {} color palette", palette_choice.label());
                    }
                }
            });

            ui.add_space(8.0);

            // Compact live rendering status card pinned near bottom of sidebar.
            GlassCard::new()
                .elevation(GlassElevation::Subtle)
                .padding(Margin::symmetric(11, 9))
                .fill_width(true)
                .show(ui, |ui| {
                    let cap = ui.ctx().prism_blur_capability();
                    let (mode_badge, badge_variant, mode_desc) = match (self.theme.surface_style, cap)
                    {
                        (SurfaceStyle::Minimal, _) => (
                            "Neumorphic Flat",
                            BadgeVariant::Accent,
                            "Sculpted tactile solid surfaces with dual soft shadows & vibrant accents.",
                        ),
                        (SurfaceStyle::Glass, BlurCapability::RealBackdrop { backend }) => (
                            if backend.contains("glow") {
                                "GPU Grab-Pass"
                            } else {
                                "GPU Dual-Kawase"
                            },
                            BadgeVariant::Success,
                            "Real WGSL Dual-Kawase pyramid + Liquid Glass bevel refraction & vibrancy.",
                        ),
                        (SurfaceStyle::Glass, BlurCapability::FallbackOnly) => (
                            "Fallback Glass",
                            BadgeVariant::Warm,
                            "Pure-egui translucent glass with cached frost grain & dual 3D shadows.",
                        ),
                    };
                    ui.horizontal(|ui| {
                        ui.add(
                            GlassBadge::new(mode_badge)
                                .variant(badge_variant)
                                .dot(true),
                        );
                    });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(mode_desc)
                            .size(10.5)
                            .color(self.theme.palette.text_secondary),
                    );
                });
        });
    }

    fn render_top_bar(&mut self, ui: &mut Ui) {
        GlassPanel::new().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                    let title = match self.active_tab {
                        DemoTab::Overview => "PrismGlass Studio",
                        DemoTab::Components => "Component Gallery",
                        DemoTab::Motion => "Motion & Spring Physics",
                        DemoTab::Tokens => "Design Token System",
                    };
                    ui.label(
                        RichText::new(title)
                            .size(19.5)
                            .strong()
                            .color(self.theme.palette.text_primary),
                    );
                    ui.label(
                        RichText::new(&self.last_action)
                            .size(12.0)
                            .color(self.theme.palette.text_secondary),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = vec2(10.0, 0.0);

                    if ui
                        .add(GlassButton::primary("Launch Studio").pill(true))
                        .clicked()
                    {
                        self.click_count += 1;
                        self.last_action =
                            format!("Triggered primary action (#{})", self.click_count);
                    }

                    let (toggle_label, next_preset) =
                        match (self.theme.surface_style, self.theme.mode) {
                            (SurfaceStyle::Glass, ThemeMode::Light) => {
                                ("Onyx Dark", PrismPreset::Onyx)
                            }
                            (SurfaceStyle::Glass, ThemeMode::Dark) => {
                                ("Frost Light", PrismPreset::Frost)
                            }
                            (SurfaceStyle::Minimal, ThemeMode::Light) => {
                                ("Minimal Dark", PrismPreset::MinimalDark)
                            }
                            (SurfaceStyle::Minimal, ThemeMode::Dark) => {
                                ("Minimal Light", PrismPreset::MinimalLight)
                            }
                        };
                    if ui
                        .add(GlassButton::secondary(toggle_label).pill(true))
                        .clicked()
                    {
                        self.set_preset(next_preset);
                        self.last_action = format!("Switched to {}", next_preset.label());
                    }

                    ui.add(
                        GlassBadge::new("egui 0.37")
                            .variant(BadgeVariant::Accent)
                            .dot(true),
                    );
                });
            });
        });
    }

    fn render_overview(&mut self, ui: &mut Ui) {
        let cap = ui.ctx().prism_blur_capability();
        let (engine_val, engine_tag, engine_variant) = match (self.theme.surface_style, cap) {
            (SurfaceStyle::Minimal, _) => ("Minimal", "Opaque", BadgeVariant::Neutral),
            (SurfaceStyle::Glass, BlurCapability::RealBackdrop { backend }) => (
                if backend.contains("glow") {
                    "Glow GL"
                } else {
                    "WGSL GPU"
                },
                "Real Blur",
                BadgeVariant::Success,
            ),
            (SurfaceStyle::Glass, BlurCapability::FallbackOnly) => {
                ("Fallback", "Pure egui", BadgeVariant::Warm)
            }
        };

        let blur_pts = self.theme.surface(GlassElevation::Elevated).blur_radius;
        let refract_pct = (self
            .theme
            .surface(GlassElevation::Elevated)
            .refraction_strength
            * 100.0)
            .round();

        // Row 1: 4 Equal-width KPI GlassCards.
        let kpis = [
            ("Blur Engine", engine_val, engine_tag, engine_variant),
            (
                "Kawase Radius",
                &format!("{blur_pts:.0} pt"),
                "4-Level Pyramid",
                BadgeVariant::Accent,
            ),
            (
                "Bevel Refraction",
                &format!("{refract_pct:.0}%"),
                "Liquid Glass",
                BadgeVariant::Neutral,
            ),
            (
                "Studio Scene",
                self.theme.backdrop_scene.label(),
                "Linear HDR16",
                BadgeVariant::Warm,
            ),
        ];

        ui.columns(4, |cols| {
            for (idx, (label, value, tag, variant)) in kpis.into_iter().enumerate() {
                cols[idx].vertical(|ui| {
                    let resp = GlassCard::new()
                        .elevation(GlassElevation::Elevated)
                        .padding(Margin::symmetric(16, 13))
                        .interactive(true)
                        .fill_width(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(label)
                                        .size(12.0)
                                        .color(self.theme.palette.text_secondary),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add(GlassBadge::new(tag).variant(variant));
                                });
                            });
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(value)
                                    .size(23.0)
                                    .strong()
                                    .color(self.theme.palette.text_primary),
                            );
                        });
                    if resp.response.clicked() {
                        self.last_action = format!("Inspected KPI card: {label} ({value})");
                    }
                });
            }
        });

        // Row 2: AI Assistant Workspace (Left) + Live Material Inspector (Right).
        let total_w = ui.available_width();
        let gap = 14.0;
        let left_w = ((total_w - gap) * 0.59).max(320.0);
        let right_w = (total_w - gap - left_w).max(260.0);

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, gap);

            // Left column: AI Assistant Command Card + Interactive Action Strip.
            ui.vertical(|ui| {
                ui.set_width(left_w);
                ui.spacing_mut().item_spacing = vec2(14.0, 14.0);

                GlassCard::new()
                    .elevation(GlassElevation::Floating)
                    .padding(Margin::symmetric(18, 15))
                    .fill_width(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Liquid Glass Optics")
                                    .size(16.0)
                                    .strong()
                                    .color(self.theme.palette.text_primary),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add(
                                    GlassBadge::new("v0.2 Rendering")
                                        .variant(BadgeVariant::Success)
                                        .dot(true),
                                );
                            });
                        });

                        ui.add_space(3.0);
                        ui.label(
                            RichText::new(
                                "Real 4-level linear Dual-Kawase backdrop blur, curved-bevel lens refraction, RGB dispersion, and macOS vibrancy.",
                            )
                            .size(12.0)
                            .color(self.theme.palette.text_secondary),
                        );

                        ui.add_space(9.0);

                        // Nested subtle glass architecture items.
                        let items = [
                            (
                                "WGSL Dual-Kawase Pyramid",
                                "4 downsample + continuous tent upsample passes in linear Rgba16Float — zero 8-bit banding.",
                                "blur-wgpu",
                                BadgeVariant::Accent,
                            ),
                            (
                                "Apple Liquid Glass SDF",
                                "Analytic rounded-rect SDF bends backdrop UVs along machined bevel edges with subtle prism dispersion.",
                                "Optics",
                                BadgeVariant::Success,
                            ),
                            (
                                "OpenGL Grab-Pass & Fallback",
                                "Opt-in live framebuffer blur on eframe+glow, plus upgraded frosted micro-grain & dual shadows everywhere.",
                                "blur-glow",
                                BadgeVariant::Warm,
                            ),
                        ];

                        for (title, desc, badge, variant) in items {
                            GlassCard::new()
                                .elevation(GlassElevation::Subtle)
                                .padding(Margin::symmetric(13, 9))
                                .fill_width(true)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(title)
                                                .size(13.0)
                                                .strong()
                                                .color(self.theme.palette.text_primary),
                                        );
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                ui.add(GlassBadge::new(badge).variant(variant));
                                            },
                                        );
                                    });
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(desc)
                                            .size(11.5)
                                            .color(self.theme.palette.text_secondary),
                                    );
                                });
                            ui.add_space(4.0);
                        }

                        ui.add_space(4.0);

                        // Frosted prompt input bar.
                        GlassCard::new()
                            .elevation(GlassElevation::Flat)
                            .corner_radius(13.0)
                            .padding(Margin::symmetric(12, 6))
                            .fill_width(true)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let input_w = (ui.available_width() - 122.0).max(120.0);
                                    ui.add_sized(
                                        vec2(input_w, 28.0),
                                        TextEdit::singleline(&mut self.prompt_input)
                                            .frame(Frame::NONE)
                                            .hint_text("Ask PrismGlass AI..."),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui
                                            .add(GlassButton::primary("Run Prompt").pill(true))
                                            .clicked()
                                        {
                                            self.last_action =
                                                format!("Executed prompt: \"{}\"", self.prompt_input);
                                        }
                                    });
                                });
                            });
                    });

                // Compact action bar card showcasing all button variants.
                GlassCard::new()
                    .elevation(GlassElevation::Elevated)
                    .padding(Margin::symmetric(16, 11))
                    .fill_width(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(10.0, 6.0);
                            if ui
                                .add(GlassButton::primary("Primary CTA").pill(true))
                                .clicked()
                            {
                                "Clicked Primary glass button".clone_into(&mut self.last_action);
                            }
                            if ui
                                .add(GlassButton::accent("Accent CTA").pill(true))
                                .clicked()
                            {
                                "Clicked Accent CTA button".clone_into(&mut self.last_action);
                            }
                            if ui
                                .add(GlassButton::secondary("Frosted Glass").pill(true))
                                .clicked()
                            {
                                "Clicked Secondary frosted button"
                                    .clone_into(&mut self.last_action);
                            }
                            if ui.add(GlassButton::ghost("Ghost Action")).clicked() {
                                "Clicked Ghost glass button".clone_into(&mut self.last_action);
                            }
                        });
                    });
            });

            // Right column: Live Optical Material Inspector.
            ui.vertical(|ui| {
                ui.set_width(right_w);

                GlassCard::new()
                    .elevation(GlassElevation::Elevated)
                    .padding(Margin::symmetric(16, 14))
                    .fill_width(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Optical Inspector")
                                    .size(15.5)
                                    .strong()
                                    .color(self.theme.palette.text_primary),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add(
                                    GlassBadge::new("Live GPU")
                                        .variant(BadgeVariant::Accent)
                                        .dot(true),
                                );
                            });
                        });
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(
                                "Dial real backdrop blur, edge refraction, vibrancy, and scene.",
                            )
                            .size(11.5)
                            .color(self.theme.palette.text_secondary),
                        );

                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("BACKDROP SCENE")
                                .size(10.0)
                                .strong()
                                .color(self.theme.palette.text_muted),
                        );
                        ui.add_space(3.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                            for scene in [
                                BackdropScene::SculptedWaves,
                                BackdropScene::LiquidPrism,
                                BackdropScene::ArchitecturalGrid,
                                BackdropScene::StudioNeutral,
                                BackdropScene::None,
                            ] {
                                let selected = self.theme.backdrop_scene == scene;
                                if ui
                                    .add(
                                        GlassButton::secondary(scene.label())
                                            .selected(selected)
                                            .min_size(vec2(82.0, 27.0)),
                                    )
                                    .clicked()
                                {
                                    self.theme.backdrop_scene = scene;
                                    self.last_action =
                                        format!("Switched studio scene to {}", scene.label());
                                }
                            }
                        });

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.add(
                                GlassToggle::new(&mut self.theme.real_blur_enabled)
                                    .label("Real GPU Blur"),
                            );
                            ui.add_space(12.0);
                            ui.add(GlassToggle::new(&mut self.animate_orbs).label("Animate"));
                        });

                        ui.add_space(8.0);
                        ui.add(
                            GlassSlider::new(&mut self.theme.blur_radius_scale, 0.0..=1.8)
                                .label("Blur Radius")
                                .suffix("x"),
                        );
                        ui.add_space(2.0);
                        ui.add(
                            GlassSlider::new(&mut self.theme.refraction_scale, 0.0..=2.0)
                                .label("Edge Refraction")
                                .suffix("x")
                                .accent(self.theme.palette.accent_warm),
                        );
                        ui.add_space(2.0);
                        ui.add(
                            GlassSlider::new(&mut self.theme.dispersion_scale, 0.0..=2.2)
                                .label("Prism Dispersion")
                                .suffix("x"),
                        );
                        ui.add_space(2.0);
                        ui.add(
                            GlassSlider::new(&mut self.theme.vibrancy_scale, 0.7..=1.5)
                                .label("Vibrancy Boost")
                                .suffix("x"),
                        );
                        ui.add_space(4.0);
                        ui.add(
                            GlassSlider::new(&mut self.theme.grain_scale, 0.0..=2.0)
                                .label("Frost Grain")
                                .suffix("x"),
                        );

                        ui.add_space(10.0);
                        if ui
                            .add(
                                GlassButton::secondary("Reset Optical Defaults")
                                    .min_size(vec2(ui.available_width(), 32.0)),
                            )
                            .clicked()
                        {
                            self.set_preset(self.preset);
                            self.orb_intensity = 1.0;
                            "Reset optical tokens to preset defaults"
                                .clone_into(&mut self.last_action);
                        }
                    });
            });
        });
    }

    fn render_components(&mut self, ui: &mut Ui) {
        // Elevation cards row.
        GlassCard::new()
            .elevation(GlassElevation::Subtle)
            .padding(Margin::same(18))
            .fill_width(true)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Glass Elevation Tiers")
                        .size(16.0)
                        .strong()
                        .color(self.theme.palette.text_primary),
                );
                ui.add_space(10.0);
                let tiers = [
                    (
                        "Flat",
                        GlassElevation::Flat,
                        "Minimal translucent tint, zero shadow.",
                    ),
                    (
                        "Subtle",
                        GlassElevation::Subtle,
                        "Soft frost for nested cards & list rows.",
                    ),
                    (
                        "Elevated",
                        GlassElevation::Elevated,
                        "Balanced gradient, rim & soft shadow.",
                    ),
                    (
                        "Floating",
                        GlassElevation::Floating,
                        "High contrast for modals & popovers.",
                    ),
                ];
                ui.columns(4, |cols| {
                    for (idx, (name, elev, desc)) in tiers.into_iter().enumerate() {
                        cols[idx].vertical(|ui| {
                            GlassCard::new()
                                .elevation(elev)
                                .padding(Margin::same(14))
                                .fill_width(true)
                                .min_height(88.0)
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(name)
                                            .size(14.0)
                                            .strong()
                                            .color(self.theme.palette.text_primary),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(desc)
                                            .size(11.5)
                                            .color(self.theme.palette.text_secondary),
                                    );
                                });
                        });
                    }
                });
            });

        // Buttons & Badges + Sliders & Toggles side-by-side.
        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                GlassCard::new()
                    .elevation(GlassElevation::Elevated)
                    .padding(Margin::same(18))
                    .fill_width(true)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("Buttons & Badges")
                                .size(15.5)
                                .strong()
                                .color(self.theme.palette.text_primary),
                        );
                        ui.add_space(10.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                            let _ = ui.add(GlassButton::primary("Primary"));
                            let _ = ui.add(GlassButton::accent("Accent"));
                            let _ = ui.add(GlassButton::secondary("Secondary"));
                            let _ = ui.add(GlassButton::ghost("Ghost"));
                            let _ = ui.add(GlassButton::primary("Pill CTA").pill(true));
                            let _ = ui.add(GlassButton::secondary("Disabled").enabled(false));
                        });
                        ui.add_space(14.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                            ui.add(GlassBadge::new("Neutral"));
                            ui.add(
                                GlassBadge::new("Prism Accent")
                                    .variant(BadgeVariant::Accent)
                                    .dot(true),
                            );
                            ui.add(
                                GlassBadge::new("Sky Blue")
                                    .variant(BadgeVariant::Warm)
                                    .dot(true),
                            );
                            ui.add(
                                GlassBadge::new("Operational")
                                    .variant(BadgeVariant::Success)
                                    .dot(true),
                            );
                            ui.add(GlassBadge::new("Preview").variant(BadgeVariant::Warning));
                        });
                    });
            });

            cols[1].vertical(|ui| {
                GlassCard::new()
                    .elevation(GlassElevation::Elevated)
                    .padding(Margin::same(18))
                    .fill_width(true)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("Controls: Sliders & Toggles")
                                .size(15.5)
                                .strong()
                                .color(self.theme.palette.text_primary),
                        );
                        ui.add_space(10.0);
                        ui.add(
                            GlassSlider::new(&mut self.demo_slider_a, 0.0..=32.0)
                                .label("Target Blur Radius")
                                .suffix(" pt")
                                .decimals(1),
                        );
                        ui.add_space(8.0);
                        ui.add(
                            GlassSlider::new(&mut self.demo_slider_b, 0.0..=1.0)
                                .label("Specular Sheen")
                                .accent(self.theme.palette.accent_warm),
                        );
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(18.0, 8.0);
                            ui.add(GlassToggle::new(&mut self.demo_toggle_a).label("Specular Rim"));
                            ui.add(
                                GlassToggle::new(&mut self.demo_toggle_b).label("High Contrast"),
                            );
                        });
                    });
            });
        });
    }

    fn render_motion(&mut self, ui: &mut Ui) {
        let target = if self.motion_toggled { 1.0 } else { 0.0 };

        GlassCard::new()
            .elevation(GlassElevation::Elevated)
            .padding(Margin::same(18))
            .fill_width(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Spring Physics Playground")
                                .size(16.0)
                                .strong()
                                .color(self.theme.palette.text_primary),
                        );
                        ui.label(
                            RichText::new(
                                "Frame-rate-independent damped harmonic oscillators with zero CPU idle cost.",
                            )
                            .size(12.0)
                            .color(self.theme.palette.text_secondary),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(GlassButton::primary("Trigger Springs").pill(true))
                            .clicked()
                        {
                            self.motion_toggled = !self.motion_toggled;
                        }
                    });
                });

                ui.add_space(14.0);

                let springs = [
                    ("Snappy", "Buttons & toggles", SpringConfig::snappy()),
                    ("Smooth", "Cards & sidebars", SpringConfig::smooth()),
                    ("Bouncy", "Badges & popovers", SpringConfig::bouncy()),
                    ("Gentle", "Modals & views", SpringConfig::gentle()),
                ];

                for (name, desc, cfg) in springs {
                    let pos_t = ui
                        .ctx()
                        .prism_spring(egui::Id::NULL.with(("demo_spring", name)), target, cfg);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(136.0);
                            ui.label(
                                RichText::new(name)
                                    .size(13.0)
                                    .strong()
                                    .color(self.theme.palette.text_primary),
                            );
                            ui.label(
                                RichText::new(desc)
                                    .size(11.0)
                                    .color(self.theme.palette.text_muted),
                            );
                        });

                        let track_w = ui.available_width().max(160.0);
                        let (track_rect, _) =
                            ui.allocate_exact_size(vec2(track_w, 34.0), Sense::hover());

                        let track_bg = match self.theme.mode {
                            ThemeMode::Dark => Color32::from_rgba_unmultiplied(255, 255, 255, 12),
                            ThemeMode::Light => Color32::from_rgba_unmultiplied(18, 19, 23, 12),
                        };
                        ui.painter()
                            .rect_filled(track_rect, CornerRadius::same(10), track_bg);

                        let pill_w = 68.0;
                        let travel = (track_rect.width() - pill_w - 8.0).max(0.0);
                        let x = track_rect.min.x + 4.0 + travel * pos_t.clamp(-0.08, 1.08);
                        let pill_rect = egui::Rect::from_min_size(
                            pos2(x, track_rect.min.y + 4.0),
                            vec2(pill_w, track_rect.height() - 8.0),
                        );
                        ui.painter().rect_filled(
                            pill_rect,
                            CornerRadius::same(7),
                            self.theme.palette.accent,
                        );
                    });
                    ui.add_space(6.0);
                }
            });

        // Easing curves visual gallery.
        GlassCard::new()
            .elevation(GlassElevation::Elevated)
            .padding(Margin::same(18))
            .fill_width(true)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Built-in Easing Curves")
                        .size(15.5)
                        .strong()
                        .color(self.theme.palette.text_primary),
                );
                ui.add_space(10.0);

                let curves = [
                    ("EaseOutCubic", Easing::EaseOutCubic),
                    ("EaseInOutCubic", Easing::EaseInOutCubic),
                    ("EaseOutExpo", Easing::EaseOutExpo),
                    ("EaseOutBack", Easing::EaseOutBack),
                ];

                ui.columns(4, |cols| {
                    for (idx, (label, easing)) in curves.into_iter().enumerate() {
                        cols[idx].vertical(|ui| {
                            GlassCard::new()
                                .elevation(GlassElevation::Subtle)
                                .padding(Margin::same(12))
                                .fill_width(true)
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(label)
                                            .size(12.5)
                                            .strong()
                                            .color(self.theme.palette.text_primary),
                                    );
                                    ui.add_space(6.0);
                                    let (plot_rect, _) = ui.allocate_exact_size(
                                        vec2(ui.available_width(), 64.0),
                                        Sense::hover(),
                                    );
                                    let inner = plot_rect.shrink(6.0);
                                    let steps = 28_usize;
                                    let mut pts = Vec::with_capacity(steps + 1);
                                    for i in 0..=steps {
                                        let t = i as f32 / steps as f32;
                                        let y_val = easing.apply(t);
                                        let px = inner.min.x + inner.width() * t;
                                        let py = inner.max.y - inner.height() * y_val * 0.85;
                                        pts.push(pos2(px, py));
                                    }
                                    ui.painter().add(egui::Shape::line(
                                        pts,
                                        Stroke::new(2.0, self.theme.palette.accent),
                                    ));
                                });
                        });
                    }
                });
            });
    }

    fn render_tokens(&self, ui: &mut Ui) {
        let p = self.theme.palette;
        let swatches = [
            ("Canvas", p.canvas),
            ("Canvas Secondary", p.canvas_secondary),
            ("Orb Primary", p.orb_primary),
            ("Orb Secondary", p.orb_secondary),
            ("Orb Accent", p.orb_accent),
            ("Accent Prism", p.accent),
            ("Accent Warm", p.accent_warm),
            ("Text Primary", p.text_primary),
            ("Text Secondary", p.text_secondary),
            ("Success", p.success),
            ("Warning", p.warning),
            ("Danger", p.danger),
        ];

        GlassCard::new()
            .elevation(GlassElevation::Elevated)
            .padding(Margin::same(18))
            .fill_width(true)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("PrismPalette — {}", self.theme.name))
                        .size(16.0)
                        .strong()
                        .color(self.theme.palette.text_primary),
                );
                ui.add_space(12.0);

                ui.columns(4, |cols| {
                    for (idx, (name, col)) in swatches.into_iter().enumerate() {
                        cols[idx % 4].vertical(|ui| {
                            GlassCard::new()
                                .elevation(GlassElevation::Subtle)
                                .padding(Margin::same(10))
                                .fill_width(true)
                                .show(ui, |ui| {
                                    let (swatch_rect, _) = ui.allocate_exact_size(
                                        vec2(ui.available_width(), 34.0),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        swatch_rect,
                                        CornerRadius::same(8),
                                        col,
                                    );
                                    ui.painter().rect_stroke(
                                        swatch_rect,
                                        CornerRadius::same(8),
                                        Stroke::new(
                                            1.0,
                                            Color32::from_rgba_unmultiplied(255, 255, 255, 40),
                                        ),
                                        StrokeKind::Inside,
                                    );
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(name)
                                            .size(12.0)
                                            .strong()
                                            .color(self.theme.palette.text_primary),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "#{:02X}{:02X}{:02X}",
                                            col.r(),
                                            col.g(),
                                            col.b()
                                        ))
                                        .font(FontId::monospace(11.0))
                                        .color(self.theme.palette.text_muted),
                                    );
                                });
                            ui.add_space(8.0);
                        });
                    }
                });
            });
    }
}

/// Paints a crisp geometric prism emblem in Antigravity monochrome + electric blue.
fn paint_prism_logo(ui: &mut Ui, theme: &PrismTheme) {
    let (rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
    let painter = ui.painter();
    let cr = CornerRadius::same(9);

    let bg = match theme.mode {
        ThemeMode::Dark => Color32::from_rgba_unmultiplied(255, 255, 255, 16),
        ThemeMode::Light => Color32::from_rgba_unmultiplied(18, 19, 23, 12),
    };
    let border = match theme.mode {
        ThemeMode::Dark => Color32::from_rgba_unmultiplied(255, 255, 255, 34),
        ThemeMode::Light => Color32::from_rgba_unmultiplied(18, 19, 23, 24),
    };

    painter.rect_filled(rect, cr, bg);
    painter.rect_stroke(rect, cr, Stroke::new(1.0, border), StrokeKind::Inside);

    let c = rect.center();
    let top = pos2(c.x, rect.min.y + 7.0);
    let left = pos2(rect.min.x + 8.0, rect.max.y - 8.5);
    let right = pos2(rect.max.x - 8.0, rect.max.y - 8.5);
    let inner = pos2(c.x, c.y + 2.5);

    painter.line_segment([top, left], Stroke::new(1.5, theme.palette.text_primary));
    painter.line_segment([top, right], Stroke::new(1.5, theme.palette.accent));
    painter.line_segment([left, right], Stroke::new(1.4, theme.palette.text_primary));
    painter.line_segment([top, inner], Stroke::new(1.1, theme.palette.accent));
    painter.line_segment(
        [inner, left],
        Stroke::new(1.0, theme.palette.text_secondary),
    );
    painter.line_segment([inner, right], Stroke::new(1.0, theme.palette.accent));
}
