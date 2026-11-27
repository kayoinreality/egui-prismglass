//! Pure-`egui` 3D-beveled Apple glassmorphism, solid minimal renderer, tactile micro-grain,
//! and sculpted volumetric backdrop painter.
//!
//! Renders crisp, modern surfaces with zero custom GPU shaders (used as the universal
//! fallback when GPU backdrop blur is disabled or unavailable):
//! - **Glass mode (`SurfaceStyle::Glass`)**:
//!   1. Dual-layer 3D shadow (wide ambient float shadow + tight contact occlusion shadow).
//!   2. Clean smoked/frosted translucent glass body.
//!   3. Subtle tiled procedural frost micro-grain texture (`128x128` cached once per context).
//!   4. Delicate `1px` inset inner glass thickness edge.
//!   5. Machined `1px` directional 3D rim-lit bevel (bright top-left key light, subtle bounce rim).
//! - **Minimal mode (`SurfaceStyle::Minimal`)**:
//!   Fast opaque solid rounded rectangles with razor-sharp `1px` neutral hairline borders.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{
    Color32, ColorImage, Context, CornerRadius, Id, Painter, Pos2, Rect, Shape, Stroke, StrokeKind,
    TextureHandle, TextureOptions, TextureWrapMode, Ui, pos2, vec2,
};

use crate::theme::{PrismTheme, SurfaceStyle, ThemeMode};
use crate::tokens::{BackdropScene, GlassSurfaceTokens, blend_colors, scale_alpha};

const CORNER_SEGMENTS: usize = 8;
const GRAIN_TEXTURE_ID: &str = "egui_prismglass_frost_grain_tex";

/// Paints a complete fallback glass (or solid minimal) surface using [`GlassSurfaceTokens`].
pub fn paint_glass_surface(painter: &Painter, rect: Rect, tokens: &GlassSurfaceTokens) {
    let shapes = glass_surface_shapes(rect, tokens);
    if !shapes.is_empty() {
        painter.extend(shapes);
    }
}

/// Builds the complete list of [`egui::Shape`]s for a 3D-beveled glass or solid minimal surface.
pub fn glass_surface_shapes(rect: Rect, tokens: &GlassSurfaceTokens) -> Vec<Shape> {
    build_surface_shapes(rect, tokens, None)
}

/// Builds fallback glass surface shapes with cached procedural frost micro-grain texture.
pub fn glass_surface_shapes_with_ctx(
    ctx: &Context,
    rect: Rect,
    tokens: &GlassSurfaceTokens,
) -> Vec<Shape> {
    let grain_tex = if tokens.grain_intensity > 0.002
        && !(tokens.gradient_top == tokens.gradient_bottom && tokens.gradient_top == tokens.tint)
    {
        Some(get_or_create_grain_texture(ctx))
    } else {
        None
    };
    build_surface_shapes(rect, tokens, grain_tex.as_ref())
}

/// Builds only the 3D elevation shadows and machined directional bevel overlay (without body fill).
///
/// Used by grab-pass GPU blur backends (`blur-glow`) that composite the blurred frosted body
/// directly into the framebuffer and layer `egui-prismglass`'s 3D bevel cleanly on top.
pub fn glass_bevel_and_shadow_shapes(
    rect: Rect,
    tokens: &GlassSurfaceTokens,
    include_shadows: bool,
) -> Vec<Shape> {
    if !rect.is_positive() {
        return Vec::new();
    }
    let mut shapes = Vec::with_capacity(8);
    let max_r = (rect.width().min(rect.height()) * 0.5).max(0.0);
    let r = tokens.corner_radius.min(max_r);
    let corner_radius = CornerRadius::same(r.round().clamp(0.0, 255.0) as u8);

    if include_shadows {
        push_shadow_shapes(&mut shapes, rect, corner_radius, tokens);
    }
    push_bevel_shapes(&mut shapes, rect, r, tokens);
    shapes
}

fn build_surface_shapes(
    rect: Rect,
    tokens: &GlassSurfaceTokens,
    grain_tex: Option<&TextureHandle>,
) -> Vec<Shape> {
    if !rect.is_positive() {
        return Vec::new();
    }

    let mut shapes = Vec::with_capacity(16);
    let max_r = (rect.width().min(rect.height()) * 0.5).max(0.0);
    let r = tokens.corner_radius.min(max_r);
    let corner_radius = CornerRadius::same(r.round().clamp(0.0, 255.0) as u8);

    // 1. Dual-layer 3D shadow (ambient float + tight contact occlusion).
    push_shadow_shapes(&mut shapes, rect, corner_radius, tokens);

    let is_solid_minimal =
        tokens.gradient_top == tokens.gradient_bottom && tokens.gradient_top == tokens.tint;

    // 2. Surface body fill.
    if is_solid_minimal {
        shapes.push(Shape::rect_filled(rect, corner_radius, tokens.tint));
        if tokens.border_width > 0.0 && tokens.border.a() > 0 {
            shapes.push(Shape::rect_stroke(
                rect,
                corner_radius,
                Stroke::new(tokens.border_width, tokens.border),
                StrokeKind::Inside,
            ));
        }
        return shapes;
    }

    let fill_mesh = rounded_rect_gradient_mesh(
        rect,
        r,
        tokens.gradient_top,
        tokens.tint,
        tokens.gradient_bottom,
    );
    shapes.push(Shape::mesh(fill_mesh));

    // 3. Subtle procedural micro-grain overlay for tactile frosted-glass feel.
    if let Some(tex) = grain_tex {
        let grain_alpha = (tokens.grain_intensity * 255.0 * 0.85)
            .round()
            .clamp(0.0, 28.0) as u8;
        if grain_alpha > 0 {
            let grain_mesh = rounded_rect_tiled_grain_mesh(
                rect,
                r,
                tex.id(),
                Color32::from_rgba_unmultiplied(255, 255, 255, grain_alpha),
            );
            shapes.push(Shape::mesh(grain_mesh));
        }
    }

    // 4. Inner thickness edge + directional 3D rim-lit perimeter bevel.
    push_bevel_shapes(&mut shapes, rect, r, tokens);

    shapes
}

fn push_shadow_shapes(
    shapes: &mut Vec<Shape>,
    rect: Rect,
    corner_radius: CornerRadius,
    tokens: &GlassSurfaceTokens,
) {
    if tokens.shadow != egui::Shadow::NONE {
        shapes.push(Shape::from(tokens.shadow.as_shape(rect, corner_radius)));
    }
    if tokens.contact_shadow != egui::Shadow::NONE {
        shapes.push(Shape::from(
            tokens.contact_shadow.as_shape(rect, corner_radius),
        ));
    }
}

fn push_bevel_shapes(shapes: &mut Vec<Shape>, rect: Rect, r: f32, tokens: &GlassSurfaceTokens) {
    if tokens.inner_glow.a() > 0 && rect.width() > 6.0 && rect.height() > 6.0 {
        let inset_rect = rect.shrink(1.0);
        let inner_radius = (r - 1.0).max(0.0);
        let inner_rim = directional_bevel_ring_mesh(
            inset_rect,
            inner_radius,
            1.0,
            tokens.inner_glow,
            scale_alpha(tokens.inner_glow, 0.28),
            scale_alpha(tokens.inner_glow, 0.08),
        );
        shapes.push(Shape::mesh(inner_rim));
    }

    if tokens.border_width > 0.0 && (tokens.border.a() > 0 || tokens.specular_top.a() > 0) {
        let key_col = if tokens.specular_top.a() > 0 {
            blend_colors(
                tokens.specular_top,
                tokens.specular_accent,
                tokens.prism_intensity.clamp(0.0, 1.0) * 0.35,
            )
        } else {
            tokens.border
        };
        let side_col = tokens.border;
        let shadow_rim = scale_alpha(tokens.border, 0.42);
        let rim_mesh = directional_bevel_ring_mesh(
            rect,
            r,
            tokens.border_width.max(1.0),
            key_col,
            side_col,
            shadow_rim,
        );
        shapes.push(Shape::mesh(rim_mesh));
    }
}

/// Paints a clean pill or rounded-rect button surface (flat or subtly frosted, never glossy).
pub fn paint_glass_gradient_fill(
    painter: &Painter,
    rect: Rect,
    corner_radius: f32,
    top_left: Color32,
    bottom_right: Color32,
    border: Stroke,
    inner_edge: Color32,
) {
    if !rect.is_positive() {
        return;
    }
    let r = corner_radius.min(rect.width().min(rect.height()) * 0.5);
    let cr = CornerRadius::same(r.round().clamp(0.0, 255.0) as u8);

    if top_left == bottom_right {
        painter.rect_filled(rect, cr, top_left);
    } else {
        let mid = blend_colors(top_left, bottom_right, 0.5);
        let mesh = rounded_rect_gradient_mesh(rect, r, top_left, mid, bottom_right);
        painter.add(Shape::mesh(mesh));
    }

    if inner_edge.a() > 0 && rect.width() > 8.0 && rect.height() > 8.0 {
        let inset = rect.shrink(1.0);
        let inner_radius = (r - 1.0).max(0.0);
        let inner_rim = directional_bevel_ring_mesh(
            inset,
            inner_radius,
            1.0,
            inner_edge,
            scale_alpha(inner_edge, 0.25),
            Color32::TRANSPARENT,
        );
        painter.add(Shape::mesh(inner_rim));
    }

    if border.width > 0.0 && border.color.a() > 0 {
        painter.rect_stroke(rect, cr, border, StrokeKind::Inside);
    }
}

/// Paints the canvas backdrop via the active GPU [`crate::BackdropBlurProvider`] when available
/// and enabled, falling back seamlessly to [`paint_prism_backdrop`] otherwise.
pub fn paint_prism_backdrop_ui(
    ui: &Ui,
    rect: Rect,
    theme: &PrismTheme,
    time_secs: f32,
    orb_intensity: f32,
) {
    if !rect.is_positive() {
        return;
    }

    if theme.surface_style == SurfaceStyle::Minimal {
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, theme.palette.canvas);
        return;
    }

    if theme.real_blur_enabled
        && let Some(provider) = crate::blur::blur_provider(ui.ctx())
        && provider.render_backdrop(ui, rect, theme, time_secs, orb_intensity)
    {
        return;
    }

    paint_prism_backdrop(ui.painter(), rect, theme, time_secs, orb_intensity);
}

/// Paints the canvas backdrop for a [`PrismTheme`] using pure `egui` primitives.
pub fn paint_prism_backdrop(
    painter: &Painter,
    rect: Rect,
    theme: &PrismTheme,
    time_secs: f32,
    orb_intensity: f32,
) {
    if !rect.is_positive() {
        return;
    }

    if theme.surface_style == SurfaceStyle::Minimal {
        painter.rect_filled(rect, CornerRadius::ZERO, theme.palette.canvas);
        return;
    }

    let mid = blend_colors(theme.palette.canvas, theme.palette.canvas_secondary, 0.5);
    let bg_mesh = rounded_rect_gradient_mesh(
        rect,
        0.0,
        theme.palette.canvas,
        mid,
        theme.palette.canvas_secondary,
    );
    painter.add(Shape::mesh(bg_mesh));

    let intensity = orb_intensity.clamp(0.0, 1.5);
    if intensity <= 0.01 {
        return;
    }

    let width = rect.width();
    let height = rect.height();
    let phase = time_secs * 0.35;

    match (theme.mode, theme.backdrop_scene) {
        (ThemeMode::Light, BackdropScene::SculptedWaves) => {
            paint_radial_glow_orb(
                painter,
                pos2(rect.min.x + width * 0.78, rect.min.y + height * 0.18),
                width.max(height) * 0.44,
                scale_alpha(theme.palette.orb_primary, 0.16 * intensity),
            );
            paint_radial_glow_orb(
                painter,
                pos2(rect.min.x + width * 0.24, rect.min.y + height * 0.55),
                width.max(height) * 0.40,
                scale_alpha(theme.palette.orb_secondary, 0.18 * intensity),
            );
            paint_sculpted_light_wave(
                painter,
                rect,
                WaveSpec {
                    y_start: 0.72,
                    y_end: 0.18,
                    amplitude: 0.12,
                    phase,
                    half_width: height * 0.20,
                    core_color: scale_alpha(Color32::WHITE, 0.55 * intensity),
                    mid_color: scale_alpha(theme.palette.orb_secondary, 0.18 * intensity),
                },
            );
        }
        (ThemeMode::Light, BackdropScene::LiquidPrism) => {
            paint_radial_glow_orb(
                painter,
                pos2(rect.min.x + width * 0.22, rect.min.y + height * 0.25),
                width.max(height) * 0.42,
                scale_alpha(Color32::from_rgb(56, 146, 255), 0.26 * intensity),
            );
            paint_radial_glow_orb(
                painter,
                pos2(rect.min.x + width * 0.76, rect.min.y + height * 0.64),
                width.max(height) * 0.44,
                scale_alpha(Color32::from_rgb(122, 108, 252), 0.22 * intensity),
            );
            paint_sculpted_light_wave(
                painter,
                rect,
                WaveSpec {
                    y_start: 0.80,
                    y_end: 0.16,
                    amplitude: 0.14,
                    phase,
                    half_width: height * 0.22,
                    core_color: scale_alpha(Color32::from_rgb(50, 121, 249), 0.24 * intensity),
                    mid_color: scale_alpha(Color32::from_rgb(112, 186, 255), 0.16 * intensity),
                },
            );
        }
        (ThemeMode::Dark, BackdropScene::SculptedWaves) => {
            paint_radial_glow_orb(
                painter,
                pos2(
                    rect.min.x + width * 0.68 + phase.sin() * width * 0.02,
                    rect.min.y + height * 0.26,
                ),
                width.max(height) * 0.46,
                scale_alpha(theme.palette.orb_secondary, 0.14 * intensity),
            );
            paint_radial_glow_orb(
                painter,
                pos2(
                    rect.min.x + width * 0.28 - phase.cos() * width * 0.02,
                    rect.min.y + height * 0.78,
                ),
                width.max(height) * 0.42,
                scale_alpha(theme.palette.orb_primary, 0.11 * intensity),
            );
            paint_sculpted_light_wave(
                painter,
                rect,
                WaveSpec {
                    y_start: 0.78,
                    y_end: 0.14,
                    amplitude: 0.13,
                    phase,
                    half_width: height * 0.24,
                    core_color: scale_alpha(theme.palette.orb_accent, 0.30 * intensity),
                    mid_color: scale_alpha(theme.palette.orb_primary, 0.16 * intensity),
                },
            );
            paint_sculpted_light_wave(
                painter,
                rect,
                WaveSpec {
                    y_start: 0.34,
                    y_end: 0.86,
                    amplitude: -0.10,
                    phase: phase * 0.7 + 1.2,
                    half_width: height * 0.17,
                    core_color: scale_alpha(theme.palette.orb_secondary, 0.22 * intensity),
                    mid_color: scale_alpha(theme.palette.orb_primary, 0.10 * intensity),
                },
            );
            paint_sculpted_light_wave(
                painter,
                rect,
                WaveSpec {
                    y_start: 0.78,
                    y_end: 0.14,
                    amplitude: 0.13,
                    phase,
                    half_width: height * 0.032,
                    core_color: scale_alpha(Color32::from_rgb(245, 248, 255), 0.22 * intensity),
                    mid_color: scale_alpha(theme.palette.orb_accent, 0.09 * intensity),
                },
            );
        }
        (ThemeMode::Dark, BackdropScene::LiquidPrism) => {
            paint_radial_glow_orb(
                painter,
                pos2(rect.min.x + width * 0.24, rect.min.y + height * 0.26),
                width.max(height) * 0.46,
                scale_alpha(Color32::from_rgb(56, 134, 255), 0.32 * intensity),
            );
            paint_radial_glow_orb(
                painter,
                pos2(rect.min.x + width * 0.76, rect.min.y + height * 0.70),
                width.max(height) * 0.44,
                scale_alpha(Color32::from_rgb(120, 102, 248), 0.28 * intensity),
            );
            paint_sculpted_light_wave(
                painter,
                rect,
                WaveSpec {
                    y_start: 0.82,
                    y_end: 0.16,
                    amplitude: 0.14,
                    phase,
                    half_width: height * 0.22,
                    core_color: scale_alpha(Color32::from_rgb(138, 206, 255), 0.38 * intensity),
                    mid_color: scale_alpha(Color32::from_rgb(66, 128, 255), 0.22 * intensity),
                },
            );
            paint_sculpted_light_wave(
                painter,
                rect,
                WaveSpec {
                    y_start: 0.24,
                    y_end: 0.84,
                    amplitude: -0.12,
                    phase: phase * 0.8 + 1.4,
                    half_width: height * 0.16,
                    core_color: scale_alpha(Color32::from_rgb(198, 174, 255), 0.28 * intensity),
                    mid_color: scale_alpha(Color32::from_rgb(96, 120, 248), 0.14 * intensity),
                },
            );
        }
        (_, BackdropScene::ArchitecturalGrid) => {
            paint_architectural_grid_fallback(painter, rect, theme, intensity);
        }
    }
}

fn paint_architectural_grid_fallback(
    painter: &Painter,
    rect: Rect,
    theme: &PrismTheme,
    intensity: f32,
) {
    let step = 48.0;
    let line_col = match theme.mode {
        ThemeMode::Dark => Color32::from_rgba_unmultiplied(210, 224, 255, (28.0 * intensity) as u8),
        ThemeMode::Light => Color32::from_rgba_unmultiplied(28, 36, 56, (22.0 * intensity) as u8),
    };
    let mut x = rect.min.x + step;
    while x < rect.max.x {
        painter.line_segment(
            [pos2(x, rect.min.y), pos2(x, rect.max.y)],
            Stroke::new(1.0, line_col),
        );
        x += step;
    }
    let mut y = rect.min.y + step;
    while y < rect.max.y {
        painter.line_segment(
            [pos2(rect.min.x, y), pos2(rect.max.x, y)],
            Stroke::new(1.0, line_col),
        );
        y += step;
    }
    paint_radial_glow_orb(
        painter,
        pos2(
            rect.min.x + rect.width() * 0.72,
            rect.min.y + rect.height() * 0.28,
        ),
        rect.width().max(rect.height()) * 0.36,
        scale_alpha(theme.palette.accent, 0.30 * intensity),
    );
}

/// Paints a soft, multi-ring Gaussian-like radial glow orb with zero hard edges.
pub fn paint_radial_glow_orb(painter: &Painter, center: Pos2, radius: f32, peak_color: Color32) {
    if radius <= 1.0 || peak_color.a() == 0 {
        return;
    }

    let rings = 6_usize;
    let segments = 32_usize;
    let mut mesh = Mesh::default();

    mesh.vertices.push(Vertex {
        pos: center,
        uv: WHITE_UV,
        color: peak_color,
    });

    for ring in 1..=rings {
        let t = ring as f32 / rings as f32;
        let r = radius * t;
        let falloff = (1.0 - t).powf(2.6);
        let col = scale_alpha(peak_color, falloff);

        for seg in 0..segments {
            let angle = (seg as f32 / segments as f32) * TAU;
            let pos = pos2(center.x + angle.cos() * r, center.y + angle.sin() * r);
            mesh.vertices.push(Vertex {
                pos,
                uv: WHITE_UV,
                color: col,
            });
        }
    }

    for seg in 0..segments {
        let next = (seg + 1) % segments;
        mesh.indices.push(0);
        mesh.indices.push((1 + seg) as u32);
        mesh.indices.push((1 + next) as u32);
    }

    for ring in 1..rings {
        let inner_start = 1 + (ring - 1) * segments;
        let outer_start = 1 + ring * segments;
        for seg in 0..segments {
            let next = (seg + 1) % segments;
            let i0 = (inner_start + seg) as u32;
            let i1 = (inner_start + next) as u32;
            let o0 = (outer_start + seg) as u32;
            let o1 = (outer_start + next) as u32;

            mesh.indices.extend_from_slice(&[i0, o0, o1, i0, o1, i1]);
        }
    }

    painter.add(Shape::mesh(mesh));
}

#[derive(Clone, Copy)]
struct WaveSpec {
    y_start: f32,
    y_end: f32,
    amplitude: f32,
    phase: f32,
    half_width: f32,
    core_color: Color32,
    mid_color: Color32,
}

fn paint_sculpted_light_wave(painter: &Painter, rect: Rect, spec: WaveSpec) {
    let steps = 44_usize;
    let width = rect.width();
    let height = rect.height();

    let eval_curve = |t: f32| -> Pos2 {
        let x = rect.min.x - width * 0.08 + (width * 1.16) * t;
        let linear_y = spec.y_start + (spec.y_end - spec.y_start) * t;
        let wave = (t * PI * 1.45 + spec.phase).sin() * spec.amplitude;
        let y = rect.min.y + height * (linear_y + wave);
        pos2(x, y)
    };

    let offsets = [-1.0_f32, -0.34, 0.0, 0.34, 1.0];
    let mut mesh = Mesh::default();

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let center = eval_curve(t);
        let prev = eval_curve((t - 0.015).max(0.0));
        let next = eval_curve((t + 0.015).min(1.0));
        let tangent = (next - prev).normalized();
        let normal = vec2(-tangent.y, tangent.x);

        let end_fade = (t * PI).sin().powf(0.55);
        let local_hw = spec.half_width * (0.72 + 0.42 * (t * PI * 1.2 + 0.4).sin().abs());

        let band_colors = [
            Color32::TRANSPARENT,
            scale_alpha(spec.mid_color, end_fade),
            scale_alpha(spec.core_color, end_fade),
            scale_alpha(spec.mid_color, end_fade * 0.72),
            Color32::TRANSPARENT,
        ];

        for (&off, &col) in offsets.iter().zip(band_colors.iter()) {
            let pos = center + normal * (off * local_hw);
            mesh.vertices.push(Vertex {
                pos,
                uv: WHITE_UV,
                color: col,
            });
        }
    }

    let cols = offsets.len();
    for i in 0..steps {
        let row0 = i * cols;
        let row1 = (i + 1) * cols;
        for c in 0..(cols - 1) {
            let a = (row0 + c) as u32;
            let b = (row0 + c + 1) as u32;
            let d = (row1 + c) as u32;
            let e = (row1 + c + 1) as u32;
            mesh.indices.extend_from_slice(&[a, b, e, a, e, d]);
        }
    }

    painter.add(Shape::mesh(mesh));
}

fn directional_bevel_ring_mesh(
    rect: Rect,
    radius: f32,
    stroke_width: f32,
    key_color: Color32,
    side_color: Color32,
    shadow_rim_color: Color32,
) -> Mesh {
    let outer = rounded_rect_perimeter(rect, radius, CORNER_SEGMENTS);
    let inner_rect = rect.shrink(stroke_width);
    if !inner_rect.is_positive() {
        return Mesh::default();
    }
    let inner_radius = (radius - stroke_width).max(0.0);
    let inner = rounded_rect_perimeter(inner_rect, inner_radius, CORNER_SEGMENTS);

    let count = outer.len().min(inner.len());
    let mut mesh = Mesh::default();
    let key_light = vec2(-0.38, -0.925).normalized();

    for i in 0..count {
        let out_p = outer[i];
        let in_p = inner[i];
        let outward = (out_p - in_p).normalized();
        let ndotl = outward.dot(key_light);

        let col = if ndotl >= 0.0 {
            blend_colors(side_color, key_color, ndotl.powf(1.35))
        } else {
            blend_colors(side_color, shadow_rim_color, (-ndotl).powf(0.9))
        };

        mesh.vertices.push(Vertex {
            pos: out_p,
            uv: WHITE_UV,
            color: col,
        });
        mesh.vertices.push(Vertex {
            pos: in_p,
            uv: WHITE_UV,
            color: col,
        });
    }

    for i in 0..count {
        let next = (i + 1) % count;
        let o0 = (i * 2) as u32;
        let i0 = (i * 2 + 1) as u32;
        let o1 = (next * 2) as u32;
        let i1 = (next * 2 + 1) as u32;
        mesh.indices.extend_from_slice(&[o0, o1, i1, o0, i1, i0]);
    }

    mesh
}

fn rounded_rect_gradient_mesh(
    rect: Rect,
    radius: f32,
    top_color: Color32,
    mid_color: Color32,
    bottom_color: Color32,
) -> Mesh {
    let perimeter = rounded_rect_perimeter(rect, radius, CORNER_SEGMENTS);
    let mut mesh = Mesh::default();

    let height = rect.height().max(1.0);
    let sample_color = |point: Pos2| -> Color32 {
        let mix = ((point.y - rect.min.y) / height).clamp(0.0, 1.0);
        if mix < 0.5 {
            blend_colors(top_color, mid_color, mix * 2.0)
        } else {
            blend_colors(mid_color, bottom_color, (mix - 0.5) * 2.0)
        }
    };

    let center = rect.center();
    mesh.vertices.push(Vertex {
        pos: center,
        uv: WHITE_UV,
        color: sample_color(center),
    });

    for &p in &perimeter {
        mesh.vertices.push(Vertex {
            pos: p,
            uv: WHITE_UV,
            color: sample_color(p),
        });
    }

    let n = perimeter.len();
    for i in 0..n {
        let next = (i + 1) % n;
        mesh.indices.push(0);
        mesh.indices.push((1 + i) as u32);
        mesh.indices.push((1 + next) as u32);
    }

    mesh
}

fn rounded_rect_tiled_grain_mesh(
    rect: Rect,
    radius: f32,
    texture_id: egui::TextureId,
    tint: Color32,
) -> Mesh {
    let perimeter = rounded_rect_perimeter(rect, radius, CORNER_SEGMENTS);
    let mut mesh = Mesh::with_texture(texture_id);
    let tile = 96.0;

    let uv_of = |p: Pos2| pos2(p.x / tile, p.y / tile);
    let center = rect.center();
    mesh.vertices.push(Vertex {
        pos: center,
        uv: uv_of(center),
        color: tint,
    });
    for &p in &perimeter {
        mesh.vertices.push(Vertex {
            pos: p,
            uv: uv_of(p),
            color: tint,
        });
    }
    let n = perimeter.len();
    for i in 0..n {
        let next = (i + 1) % n;
        mesh.indices.push(0);
        mesh.indices.push((1 + i) as u32);
        mesh.indices.push((1 + next) as u32);
    }
    mesh
}

fn get_or_create_grain_texture(ctx: &Context) -> TextureHandle {
    let key = Id::NULL.with(GRAIN_TEXTURE_ID);
    if let Some(handle) = ctx.data(|d| d.get_temp::<TextureHandle>(key)) {
        return handle;
    }

    // Deterministic monochrome fine-grain texture (128x128) generated once per context.
    let size = 128_usize;
    let mut pixels = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            // Fast 2D integer hash -> uniform grain in [96..255].
            let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
            h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
            let v = ((h ^ (h >> 16)) & 0xFF) as u8;
            let lum = 120 + (v / 2);
            pixels.push(Color32::from_rgba_unmultiplied(lum, lum, lum, v));
        }
    }

    let image = ColorImage::new([size, size], pixels);
    let options = TextureOptions {
        magnification: egui::TextureFilter::Linear,
        minification: egui::TextureFilter::Linear,
        wrap_mode: TextureWrapMode::Repeat,
        mipmap_mode: None,
    };
    let handle = ctx.load_texture("prismglass_frost_grain", image, options);
    ctx.data_mut(|d| d.insert_temp(key, handle.clone()));
    handle
}

fn rounded_rect_perimeter(rect: Rect, radius: f32, corner_segments: usize) -> Vec<Pos2> {
    let r = radius.min(rect.width() * 0.5).min(rect.height() * 0.5);
    if r <= 0.5 {
        return vec![
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
    }

    let mut pts = Vec::with_capacity((corner_segments + 1) * 4);
    let corners = [
        (pos2(rect.min.x + r, rect.min.y + r), PI),
        (pos2(rect.max.x - r, rect.min.y + r), 1.5 * PI),
        (pos2(rect.max.x - r, rect.max.y - r), 0.0),
        (pos2(rect.min.x + r, rect.max.y - r), 0.5 * PI),
    ];

    for (center, start_angle) in corners {
        for i in 0..=corner_segments {
            let angle = start_angle + (i as f32 / corner_segments as f32) * FRAC_PI_2;
            pts.push(pos2(center.x + angle.cos() * r, center.y + angle.sin() * r));
        }
    }

    pts
}
