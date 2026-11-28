//! Renders real high-DPI GPU-blurred screenshots of the `egui-prismglass` showcase into `assets/`.
//!
//! Run with:
//! ```bash
//! cargo run -p demo --example render_assets
//! ```

#![allow(missing_docs)]

use std::fs;
use std::path::Path;

use demo::{DemoApp, DemoTab, StudioApp};
use egui::{Align2, Area, Color32, FontId, Id, Margin, RichText, pos2, vec2};
use egui_kittest::Harness;
use egui_prismglass::{
    AccentPalette, BackdropScene, BadgeVariant, GlassBadge, GlassButton, GlassCard, GlassElevation,
    GlassSlider, GlassToggle, PrismContextExt, PrismPreset, PrismTheme, paint_prism_backdrop_ui,
};

struct CaptureSpec<'a> {
    preset: PrismPreset,
    tab: DemoTab,
    scene: BackdropScene,
    accent: AccentPalette,
    real_blur: bool,
    filename: &'a str,
}

fn capture(spec: &CaptureSpec<'_>, assets_dir: &Path) {
    let mut app = DemoApp::with_preset(spec.preset);
    app.active_tab = spec.tab;
    app.theme.backdrop_scene = spec.scene;
    app.theme.real_blur_enabled = spec.real_blur;
    app.theme.set_accent_palette(spec.accent);

    let mut installed = false;
    let mut harness = Harness::builder()
        .with_size(vec2(1280.0, 810.0))
        .with_pixels_per_point(1.5)
        .build_ui(move |ui| {
            #[cfg(feature = "blur-wgpu")]
            if !installed {
                egui_prismglass::blur_wgpu::install_with_format(
                    ui.ctx(),
                    egui_prismglass::blur_wgpu::wgpu::TextureFormat::Rgba8Unorm,
                );
                installed = true;
            }
            let _ = installed;

            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    app.show(ui);
                });
        });

    harness.step();
    harness.step();

    let path = assets_dir.join(spec.filename);
    let image = harness
        .render()
        .expect("headless wgpu renderer should produce RGBA image");
    image
        .save(&path)
        .expect("failed to write rendered showcase screenshot");
    println!(
        "Saved {} ({}x{})",
        path.display(),
        image.width(),
        image.height()
    );
}

/// Renders a dedicated Apple Liquid Glass Optics Studio showcase featuring crystal-clear
/// Snell-refracting cards, dual clear/smoked segmented pills, and vibrant tinted Liquid Glass
/// pills positioned right across the razor-sharp Indigo-Cobalt diagonal silk crease.
fn capture_liquid_glass_studio(assets_dir: &Path) {
    let mut installed = false;
    let mut harness = Harness::builder()
        .with_size(vec2(1280.0, 810.0))
        .with_pixels_per_point(1.5)
        .build_ui(move |ui| {
            let mut theme = PrismTheme::glacial();
            theme.backdrop_scene = BackdropScene::LiquidPrism;
            theme.real_blur_enabled = true;
            ui.ctx().set_prism_theme(theme.clone());

            #[cfg(feature = "blur-wgpu")]
            if !installed {
                egui_prismglass::blur_wgpu::install_with_format(
                    ui.ctx(),
                    egui_prismglass::blur_wgpu::wgpu::TextureFormat::Rgba8Unorm,
                );
                installed = true;
            }
            let _ = installed;

            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    let full_rect = egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 810.0));
                    paint_prism_backdrop_ui(ui, full_rect, &theme, 0.0, 1.0);

                    // 1. Left: Crystal Liquid Glass Calendar Card crossing the diagonal silk fold.
                    Area::new(Id::NULL.with("liquid_calendar_card"))
                        .fixed_pos(pos2(58.0, 62.0))
                        .show(ui.ctx(), |ui| {
                            let mut cal_tokens = theme.surface(GlassElevation::Floating);
                            cal_tokens.corner_radius = 28.0;
                            cal_tokens.blur_radius = 8.0;
                            cal_tokens.refraction_strength = 1.0;
                            cal_tokens.chromatic_dispersion = 0.55;
                            cal_tokens.blur_tint_alpha = 0.16;
                            cal_tokens.border = Color32::from_rgba_unmultiplied(255, 255, 255, 80);
                            cal_tokens.specular_top =
                                Color32::from_rgba_unmultiplied(255, 255, 255, 248);

                            GlassCard::new()
                                .tokens(cal_tokens)
                                .padding(Margin::symmetric(30, 26))
                                .show(ui, |ui| {
                                    ui.set_width(476.0);
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("November 2026  >")
                                                .size(22.0)
                                                .strong()
                                                .color(Color32::WHITE),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    RichText::new("<    >")
                                                        .size(20.0)
                                                        .color(Color32::WHITE),
                                                );
                                            },
                                        );
                                    });
                                    ui.add_space(20.0);

                                    let cell_w = 68.0;
                                    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                                        for d in days {
                                            let (r, _) = ui.allocate_exact_size(
                                                vec2(cell_w, 28.0),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().text(
                                                r.center(),
                                                Align2::CENTER_CENTER,
                                                d,
                                                FontId::proportional(14.0),
                                                Color32::from_rgba_unmultiplied(235, 242, 255, 215),
                                            );
                                        }
                                    });
                                    ui.add_space(10.0);

                                    let weeks = [
                                        [28, 29, 30, 31, 1, 2, 3],
                                        [4, 5, 6, 7, 8, 9, 10],
                                        [11, 12, 13, 14, 15, 16, 17],
                                        [18, 19, 20, 21, 22, 23, 24],
                                        [25, 26, 27, 28, 29, 30, 31],
                                        [1, 2, 3, 4, 5, 6, 7],
                                    ];
                                    for week in weeks {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                                            for num in week {
                                                let (r, _) = ui.allocate_exact_size(
                                                    vec2(cell_w, 56.0),
                                                    egui::Sense::hover(),
                                                );
                                                if num == 22 {
                                                    ui.painter().circle_filled(
                                                        r.center(),
                                                        20.0,
                                                        Color32::from_rgba_unmultiplied(
                                                            255, 255, 255, 58,
                                                        ),
                                                    );
                                                }
                                                ui.painter().text(
                                                    r.center(),
                                                    Align2::CENTER_CENTER,
                                                    format!("{num}"),
                                                    FontId::proportional(19.0),
                                                    Color32::WHITE,
                                                );
                                            }
                                        });
                                    }
                                });
                        });

                    // 2. Top-Right: Clear & Dark Liquid Glass Segmented Pills crossing the upper crease.
                    Area::new(Id::NULL.with("liquid_pill_clear"))
                        .fixed_pos(pos2(666.0, 62.0))
                        .show(ui.ctx(), |ui| {
                            let mut pill_tokens = theme.surface(GlassElevation::Floating);
                            pill_tokens.corner_radius = 999.0;
                            pill_tokens.blur_radius = 6.0;
                            pill_tokens.refraction_strength = 1.0;
                            pill_tokens.chromatic_dispersion = 0.60;
                            pill_tokens.blur_tint_alpha = 0.16;
                            pill_tokens.tint = Color32::from_rgba_unmultiplied(150, 140, 255, 90);
                            pill_tokens.gradient_top =
                                Color32::from_rgba_unmultiplied(170, 160, 255, 96);
                            pill_tokens.gradient_bottom =
                                Color32::from_rgba_unmultiplied(90, 110, 220, 88);
                            pill_tokens.specular_top =
                                Color32::from_rgba_unmultiplied(255, 255, 255, 250);

                            GlassCard::new()
                                .tokens(pill_tokens)
                                .padding(Margin::symmetric(16, 14))
                                .show(ui, |ui| {
                                    ui.set_width(470.0);
                                    ui.horizontal(|ui| {
                                        let (pill_rect, _) = ui.allocate_exact_size(
                                            vec2(220.0, 74.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().rect_filled(
                                            pill_rect,
                                            37.0,
                                            Color32::from_rgba_unmultiplied(255, 255, 255, 58),
                                        );
                                        ui.painter().text(
                                            pill_rect.center(),
                                            Align2::CENTER_CENTER,
                                            "Command",
                                            FontId::proportional(19.0),
                                            Color32::WHITE,
                                        );

                                        let (right_rect, _) = ui.allocate_exact_size(
                                            vec2(220.0, 74.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().text(
                                            right_rect.center(),
                                            Align2::CENTER_CENTER,
                                            "Inbox",
                                            FontId::proportional(19.0),
                                            Color32::WHITE,
                                        );
                                    });
                                });
                        });

                    Area::new(Id::NULL.with("liquid_pill_dark"))
                        .fixed_pos(pos2(666.0, 192.0))
                        .show(ui.ctx(), |ui| {
                            let mut dark_pill = theme.surface(GlassElevation::Floating);
                            dark_pill.corner_radius = 999.0;
                            dark_pill.blur_radius = 6.0;
                            dark_pill.refraction_strength = 1.0;
                            dark_pill.chromatic_dispersion = 0.58;
                            dark_pill.blur_tint_alpha = 0.42;
                            dark_pill.tint = Color32::from_rgba_unmultiplied(8, 14, 38, 165);
                            dark_pill.gradient_top =
                                Color32::from_rgba_unmultiplied(12, 20, 52, 160);
                            dark_pill.gradient_bottom =
                                Color32::from_rgba_unmultiplied(5, 10, 28, 178);
                            dark_pill.specular_top =
                                Color32::from_rgba_unmultiplied(255, 255, 255, 250);

                            GlassCard::new()
                                .tokens(dark_pill)
                                .padding(Margin::symmetric(16, 14))
                                .show(ui, |ui| {
                                    ui.set_width(470.0);
                                    ui.horizontal(|ui| {
                                        let (pill_rect, _) = ui.allocate_exact_size(
                                            vec2(220.0, 74.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().rect_filled(
                                            pill_rect,
                                            37.0,
                                            Color32::from_rgba_unmultiplied(6, 10, 24, 155),
                                        );
                                        ui.painter().text(
                                            pill_rect.center(),
                                            Align2::CENTER_CENTER,
                                            "Command",
                                            FontId::proportional(19.0),
                                            Color32::WHITE,
                                        );

                                        let (right_rect, _) = ui.allocate_exact_size(
                                            vec2(220.0, 74.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().text(
                                            right_rect.center(),
                                            Align2::CENTER_CENTER,
                                            "Inbox",
                                            FontId::proportional(19.0),
                                            Color32::WHITE,
                                        );
                                    });
                                });
                        });

                    // 3. Stepped Vibrant Tinted Liquid Glass Pills straddling the diagonal silk crease!
                    Area::new(Id::NULL.with("liquid_tint_ruby"))
                        .fixed_pos(pos2(710.0, 350.0))
                        .show(ui.ctx(), |ui| {
                            ui.add(
                                GlassButton::new("Add")
                                    .pill(true)
                                    .font_size(22.0)
                                    .min_size(vec2(236.0, 82.0))
                                    .tint(Color32::from_rgb(186, 18, 14)),
                            );
                        });

                    Area::new(Id::NULL.with("liquid_tint_cobalt"))
                        .fixed_pos(pos2(620.0, 468.0))
                        .show(ui.ctx(), |ui| {
                            ui.add(
                                GlassButton::new("Reply")
                                    .pill(true)
                                    .font_size(22.0)
                                    .min_size(vec2(246.0, 82.0))
                                    .tint(Color32::from_rgb(12, 88, 255)),
                            );
                        });

                    Area::new(Id::NULL.with("liquid_tint_amber"))
                        .fixed_pos(pos2(534.0, 588.0))
                        .show(ui.ctx(), |ui| {
                            ui.add(
                                GlassButton::new("Check In")
                                    .pill(true)
                                    .font_size(22.0)
                                    .min_size(vec2(256.0, 82.0))
                                    .tint(Color32::from_rgb(168, 112, 18)),
                            );
                        });

                    Area::new(Id::NULL.with("liquid_tint_violet"))
                        .fixed_pos(pos2(986.0, 350.0))
                        .show(ui.ctx(), |ui| {
                            ui.add(
                                GlassButton::new("AI")
                                    .pill(true)
                                    .font_size(22.0)
                                    .min_size(vec2(92.0, 92.0))
                                    .tint(Color32::from_rgb(148, 24, 242)),
                            );
                        });

                    // 4. Bottom-Left Clear Liquid Glass Hero Pill straddling the lower fold
                    Area::new(Id::NULL.with("liquid_clear_hero"))
                        .fixed_pos(pos2(102.0, 622.0))
                        .show(ui.ctx(), |ui| {
                            let mut clear_pill = theme.surface(GlassElevation::Floating);
                            clear_pill.corner_radius = 999.0;
                            clear_pill.blur_radius = 5.0;
                            clear_pill.refraction_strength = 1.0;
                            clear_pill.chromatic_dispersion = 0.62;
                            clear_pill.blur_tint_alpha = 0.14;
                            clear_pill.tint = Color32::from_rgba_unmultiplied(150, 175, 255, 68);
                            clear_pill.gradient_top =
                                Color32::from_rgba_unmultiplied(180, 200, 255, 72);
                            clear_pill.gradient_bottom =
                                Color32::from_rgba_unmultiplied(80, 110, 210, 62);
                            clear_pill.specular_top =
                                Color32::from_rgba_unmultiplied(255, 255, 255, 252);

                            GlassCard::new()
                                .tokens(clear_pill)
                                .padding(Margin::symmetric(34, 18))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("Liquid Glass  v0.2")
                                            .size(23.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                });
                        });
                });
        });

    harness.step();
    harness.step();

    let path = assets_dir.join("showcase-liquid-studio.png");
    let image = harness
        .render()
        .expect("headless wgpu renderer should produce Liquid Studio RGBA image");
    image
        .save(&path)
        .expect("failed to write Liquid Studio showcase screenshot");
    println!(
        "Saved {} ({}x{})",
        path.display(),
        image.width(),
        image.height()
    );
}

/// Renders a split Light & Dark Modern Tactile Neumorphism + Vibrant Solid Color Palettes showcase.
fn capture_neumorphic_palettes(assets_dir: &Path) {
    let mut harness = Harness::builder()
        .with_size(vec2(1280.0, 810.0))
        .with_pixels_per_point(1.5)
        .build_ui(move |ui| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    let full = egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 810.0));
                    let left_rect = egui::Rect::from_min_size(full.min, vec2(640.0, 810.0));
                    let right_rect =
                        egui::Rect::from_min_size(pos2(640.0, 0.0), vec2(640.0, 810.0));

                    let light_base = PrismTheme::minimal_light().with_accent_palette(AccentPalette::Emerald);
                    let dark_base = PrismTheme::minimal_dark().with_accent_palette(AccentPalette::Ruby);

                    ui.painter().rect_filled(left_rect, 0.0, light_base.palette.canvas);
                    ui.painter().rect_filled(right_rect, 0.0, dark_base.palette.canvas);

                    let draw_half = |ui: &mut egui::Ui,
                                     rect: egui::Rect,
                                     title: &str,
                                     subtitle: &str,
                                     base_theme: &PrismTheme,
                                     featured: [AccentPalette; 4]| {
                        ui.ctx().set_prism_theme(base_theme.clone());
                        let mut child_ui = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(rect.shrink(30.0))
                                .layout(egui::Layout::top_down(egui::Align::Min)),
                        );
                        child_ui.spacing_mut().item_spacing = vec2(16.0, 16.0);

                        child_ui.label(
                            RichText::new(title)
                                .size(24.0)
                                .strong()
                                .color(base_theme.palette.text_primary),
                        );
                        child_ui.label(
                            RichText::new(subtitle)
                                .size(13.0)
                                .color(base_theme.palette.text_secondary),
                        );
                        child_ui.add_space(4.0);

                        // Full 8-color vibrant palette strip card
                        GlassCard::new()
                            .elevation(GlassElevation::Elevated)
                            .padding(Margin::symmetric(18, 16))
                            .fill_width(true)
                            .show(&mut child_ui, |ui| {
                                ui.label(
                                    RichText::new("8 CURATED VIBRANT PALETTES")
                                        .size(10.5)
                                        .strong()
                                        .color(base_theme.palette.text_muted),
                                );
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing = vec2(14.0, 0.0);
                                    for p in AccentPalette::ALL {
                                        let is_active = p == base_theme.accent_palette;
                                        ui.vertical(|ui| {
                                            ui.spacing_mut().item_spacing = vec2(0.0, 5.0);
                                            let (r, _) = ui.allocate_exact_size(
                                                vec2(48.0, 36.0),
                                                egui::Sense::hover(),
                                            );
                                            let swatch_col = p.primary(base_theme.mode == egui_prismglass::ThemeMode::Dark);
                                            ui.painter().rect_filled(r, 10.0, swatch_col);
                                            if is_active {
                                                ui.painter().rect_stroke(
                                                    r.expand(2.5),
                                                    12.0,
                                                    egui::Stroke::new(2.0, base_theme.palette.text_primary),
                                                    egui::StrokeKind::Outside,
                                                );
                                            }
                                            ui.label(
                                                RichText::new(p.label())
                                                    .size(10.5)
                                                    .color(base_theme.palette.text_secondary),
                                            );
                                        });
                                    }
                                });
                            });

                        // 2x2 grid of tactile neumorphic cards, each styled with a distinct vibrant accent!
                        for row in featured.chunks(2) {
                            child_ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = vec2(16.0, 16.0);
                                let card_w = (ui.available_width() - 16.0) * 0.5;
                                for &accent in row {
                                    let themed = base_theme.clone().with_accent_palette(accent);
                                    ui.ctx().set_prism_theme(themed.clone());
                                    ui.vertical(|ui| {
                                        ui.set_width(card_w);
                                        GlassCard::new()
                                            .elevation(GlassElevation::Floating)
                                            .padding(Margin::symmetric(18, 16))
                                            .fill_width(true)
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        RichText::new(accent.label())
                                                            .size(16.0)
                                                            .strong()
                                                            .color(themed.palette.text_primary),
                                                    );
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(egui::Align::Center),
                                                        |ui| {
                                                            ui.add(
                                                                GlassBadge::new("Vibrant")
                                                                    .variant(BadgeVariant::Accent)
                                                                    .dot(true),
                                                            );
                                                        },
                                                    );
                                                });
                                                ui.add_space(10.0);
                                                let mut val = 72.0_f32;
                                                ui.add(
                                                    GlassSlider::new(&mut val, 0.0..=100.0)
                                                        .label("Intensity")
                                                        .suffix("%"),
                                                );
                                                ui.add_space(10.0);
                                                let mut on = true;
                                                ui.add(GlassToggle::new(&mut on).label("Active"));
                                                ui.add_space(12.0);
                                                ui.horizontal(|ui| {
                                                    ui.add(
                                                        GlassButton::primary("Launch")
                                                            .pill(true)
                                                            .min_size(vec2(104.0, 34.0)),
                                                    );
                                                    ui.add(
                                                        GlassButton::secondary("Tactile")
                                                            .pill(true)
                                                            .min_size(vec2(92.0, 34.0)),
                                                    );
                                                });
                                            });
                                    });
                                }
                            });
                        }

                        // Recessed tactile summary bar at bottom
                        ui.ctx().set_prism_theme(base_theme.clone());
                        GlassCard::new()
                            .elevation(GlassElevation::Subtle)
                            .padding(Margin::symmetric(18, 14))
                            .fill_width(true)
                            .show(&mut child_ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Recessed Tactile Well")
                                            .size(13.5)
                                            .strong()
                                            .color(base_theme.palette.text_primary),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new("Zero translucency • Dual directional shadows")
                                                    .size(11.5)
                                                    .color(base_theme.palette.text_secondary),
                                            );
                                        },
                                    );
                                });
                            });
                    };

                    draw_half(
                        ui,
                        left_rect,
                        "Neumorphic Light",
                        "Soft extruded porcelain surfaces with vibrant solid accents.",
                        &light_base,
                        [
                            AccentPalette::Cobalt,
                            AccentPalette::Emerald,
                            AccentPalette::Ruby,
                            AccentPalette::Amber,
                        ],
                    );
                    draw_half(
                        ui,
                        right_rect,
                        "Neumorphic Dark",
                        "Deep sculpted carbon-slate surfaces with luminous solid accents.",
                        &dark_base,
                        [
                            AccentPalette::Violet,
                            AccentPalette::Ruby,
                            AccentPalette::Cyan,
                            AccentPalette::Magenta,
                        ],
                    );
                });
        });

    harness.step();
    harness.step();

    let path = assets_dir.join("showcase-neumorphic-palettes.png");
    let image = harness
        .render()
        .expect("headless wgpu renderer should produce Neumorphic Palettes image");
    image
        .save(&path)
        .expect("failed to write Neumorphic Palettes screenshot");
    println!(
        "Saved {} ({}x{})",
        path.display(),
        image.width(),
        image.height()
    );
}

/// Renders the Prism Studio demo (the flagship of the Prism presets) at 1280×810 @1.5x.
fn capture_studio(preset: PrismPreset, accent: AccentPalette, filename: &str, assets_dir: &Path) {
    capture_studio_with(preset, accent, true, filename, assets_dir);
}

fn capture_studio_with(
    preset: PrismPreset,
    accent: AccentPalette,
    real_blur: bool,
    filename: &str,
    assets_dir: &Path,
) {
    let mut app = StudioApp::with_preset(preset);
    app.set_accent(accent);
    app.theme.real_blur_enabled = real_blur;

    let mut installed = false;
    let mut harness = Harness::builder()
        .with_size(vec2(1280.0, 810.0))
        .with_pixels_per_point(1.5)
        .build_ui(move |ui| {
            #[cfg(feature = "blur-wgpu")]
            if !installed {
                egui_prismglass::blur_wgpu::install_with_format(
                    ui.ctx(),
                    egui_prismglass::blur_wgpu::wgpu::TextureFormat::Rgba8Unorm,
                );
                installed = true;
            }
            let _ = installed;
            // Use the whole viewport (the harness root `Ui` has a small margin).
            let full = ui.ctx().content_rect();
            ui.scope_builder(egui::UiBuilder::new().max_rect(full), |ui| {
                ui.set_clip_rect(full);
                app.show(ui);
            });
        });
    harness.step();
    harness.step();
    harness.step();

    let path = assets_dir.join(filename);
    let image = harness
        .render()
        .expect("headless wgpu renderer should produce RGBA image");
    image
        .save(&path)
        .expect("failed to write rendered showcase screenshot");
    println!("Saved {}", path.display());
}

fn main() {
    let assets_dir = Path::new("assets");
    fs::create_dir_all(assets_dir).expect("failed to create assets directory");

    capture_studio(
        PrismPreset::PrismDark,
        AccentPalette::Cobalt,
        "prism-dark.png",
        assets_dir,
    );
    capture_studio(
        PrismPreset::PrismLight,
        AccentPalette::Cobalt,
        "prism-light.png",
        assets_dir,
    );
    capture_studio(
        PrismPreset::PrismGlassDark,
        AccentPalette::Violet,
        "prism-glass-dark.png",
        assets_dir,
    );
    capture_studio(
        PrismPreset::PrismGlassLight,
        AccentPalette::Emerald,
        "prism-glass-light.png",
        assets_dir,
    );
    capture_studio_with(
        PrismPreset::PrismGlassDark,
        AccentPalette::Violet,
        false,
        "prism-glass-dark-fallback.png",
        assets_dir,
    );
    if std::env::args().nth(1).as_deref() == Some("prism") {
        return;
    }

    capture_liquid_glass_studio(assets_dir);
    capture_neumorphic_palettes(assets_dir);

    let captures = [
        CaptureSpec {
            preset: PrismPreset::Glacial,
            tab: DemoTab::Overview,
            scene: BackdropScene::LiquidPrism,
            accent: AccentPalette::Cobalt,
            real_blur: true,
            filename: "showcase-liquid-prism.png",
        },
        CaptureSpec {
            preset: PrismPreset::Onyx,
            tab: DemoTab::Overview,
            scene: BackdropScene::SculptedWaves,
            accent: AccentPalette::Cobalt,
            real_blur: true,
            filename: "showcase-onyx.png",
        },
        CaptureSpec {
            preset: PrismPreset::Frost,
            tab: DemoTab::Overview,
            scene: BackdropScene::SculptedWaves,
            accent: AccentPalette::Cobalt,
            real_blur: true,
            filename: "showcase-frost.png",
        },
        CaptureSpec {
            preset: PrismPreset::Onyx,
            tab: DemoTab::Overview,
            scene: BackdropScene::ArchitecturalGrid,
            accent: AccentPalette::Cyan,
            real_blur: true,
            filename: "showcase-grid-blur.png",
        },
        CaptureSpec {
            preset: PrismPreset::Onyx,
            tab: DemoTab::Overview,
            scene: BackdropScene::ArchitecturalGrid,
            accent: AccentPalette::Cyan,
            real_blur: false,
            filename: "showcase-grid-fallback.png",
        },
        CaptureSpec {
            preset: PrismPreset::Glacial,
            tab: DemoTab::Components,
            scene: BackdropScene::LiquidPrism,
            accent: AccentPalette::Cobalt,
            real_blur: true,
            filename: "showcase-glacial.png",
        },
        CaptureSpec {
            preset: PrismPreset::MinimalLight,
            tab: DemoTab::Overview,
            scene: BackdropScene::SculptedWaves,
            accent: AccentPalette::Cobalt,
            real_blur: false,
            filename: "showcase-minimal-light.png",
        },
        CaptureSpec {
            preset: PrismPreset::MinimalDark,
            tab: DemoTab::Overview,
            scene: BackdropScene::SculptedWaves,
            accent: AccentPalette::Violet,
            real_blur: false,
            filename: "showcase-minimal-dark.png",
        },
        CaptureSpec {
            preset: PrismPreset::Glacial,
            tab: DemoTab::Motion,
            scene: BackdropScene::LiquidPrism,
            accent: AccentPalette::Cobalt,
            real_blur: true,
            filename: "showcase-motion.png",
        },
        CaptureSpec {
            preset: PrismPreset::Onyx,
            tab: DemoTab::Tokens,
            scene: BackdropScene::SculptedWaves,
            accent: AccentPalette::Cobalt,
            real_blur: true,
            filename: "showcase-tokens.png",
        },
    ];

    for spec in &captures {
        capture(spec, assets_dir);
    }
}
