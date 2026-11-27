//! Live-framebuffer grab-pass GPU backdrop blur backend for `glow` / `egui_glow`.
//!
//! Enabled via `feature = "blur-glow"`. Wraps [`backdrop_blur_glow::GlowBlur`] with
//! `#![forbid(unsafe_code)]`, grabbing whatever was previously rendered into the active
//! OpenGL 3.3 / GLES 3.0 / WebGL 2.0 draw framebuffer behind each glass surface, blurring it
//! on the GPU, compositing the frosted surface back, and layering `egui-prismglass`'s 3D
//! machined bevel cleanly on top.
//!
//! # Lifecycle Contract
//!
//! OpenGL resources owned by [`GlowBlurProvider`] must be released explicitly while the GL
//! context is still current by calling [`GlowBlurProvider::destroy`] from
//! `eframe::App::on_exit`.
//!
//! # Quick Setup (`eframe` + `glow`)
//!
//! ```ignore
//! eframe::run_native("My Glow App", opts, Box::new(|cc| {
//!     let blur_provider = cc.gl.as_ref().and_then(|gl| {
//!         egui_prismglass::blur_glow::install(&cc.egui_ctx, gl).ok()
//!     });
//!     Ok(Box::new(MyApp { blur_provider }))
//! }))
//! ```

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::sync::{Arc, Mutex};

use backdrop_blur_core::{
    BlurError, BlurRadius, BlurRequest, CornerRadius as BlurCornerRadius, GlRegion, Presence,
    Scale, Tint,
};
use backdrop_blur_glow::{FramebufferSize, GlowBlur, current_draw_framebuffer};
use egui::{Context, CornerRadius, Id, PaintCallback, PaintCallbackInfo, Shape, Ui};

use crate::blur::{BackdropBlurProvider, BackdropRequest, BlurCapability, set_blur_provider};
use crate::painter::glass_bevel_and_shadow_shapes;
use crate::tokens::GlassSurfaceTokens;

/// Real GPU backdrop-blur provider powered by live-framebuffer OpenGL grab-pass (`glow`).
pub struct GlowBlurProvider {
    blur: Arc<Mutex<GlowBlur>>,
}

impl GlowBlurProvider {
    /// Initializes OpenGL blur programs and scratch resources against `gl`.
    ///
    /// # Errors
    ///
    /// Returns [`BlurError`] if the active GL context does not meet OpenGL 3.3 / GLES 3.0 /
    /// WebGL 2.0 minimums or shader compilation fails.
    pub fn new(gl: &glow::Context) -> Result<Self, BlurError> {
        let blur = GlowBlur::new(gl)?;
        Ok(Self {
            blur: Arc::new(Mutex::new(blur)),
        })
    }

    /// Frees all GPU textures, framebuffers, and shader programs owned by this provider.
    ///
    /// Call from `eframe::App::on_exit` while the GL context is still current. Idempotent.
    pub fn destroy(&self, gl: &glow::Context) {
        let mut guard = match self.blur.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                self.blur.clear_poison();
                poisoned.into_inner()
            }
        };
        guard.destroy(gl);
    }
}

/// Initializes a [`GlowBlurProvider`] and installs it into `ctx`.
///
/// Retain the returned `Arc<GlowBlurProvider>` so you can call [`GlowBlurProvider::destroy`]
/// inside `eframe::App::on_exit`.
///
/// # Errors
///
/// Returns [`BlurError`] if the GL context is unsupported or shader compilation fails.
pub fn install(ctx: &Context, gl: &glow::Context) -> Result<Arc<GlowBlurProvider>, BlurError> {
    let provider = Arc::new(GlowBlurProvider::new(gl)?);
    set_blur_provider(ctx, provider.clone());
    Ok(provider)
}

impl BackdropBlurProvider for GlowBlurProvider {
    fn capability(&self) -> BlurCapability {
        BlurCapability::RealBackdrop {
            backend: "glow-grab-pass",
        }
    }

    fn render_glass_surface(
        &self,
        _ui: &Ui,
        _id: Id,
        request: &BackdropRequest,
    ) -> Option<Vec<Shape>> {
        if !request.rect.is_positive() {
            return Some(Vec::new());
        }

        let max_r = (request.rect.width().min(request.rect.height()) * 0.5).max(0.0);
        let r = request.corner_radius.min(max_r);
        let corner_radius = CornerRadius::same(r.round().clamp(0.0, 255.0) as u8);

        let mut shapes = Vec::with_capacity(8);

        // 1. Paint dual-layer 3D elevation shadows underneath the frosted glass region.
        if request.shadow != egui::Shadow::NONE {
            shapes.push(Shape::from(
                request.shadow.as_shape(request.rect, corner_radius),
            ));
        }
        if request.contact_shadow != egui::Shadow::NONE {
            shapes.push(Shape::from(
                request.contact_shadow.as_shape(request.rect, corner_radius),
            ));
        }

        // 2. Enqueue live-framebuffer grab-pass blur + frosted tint composite.
        let blur = Arc::clone(&self.blur);
        let blur_radius = BlurRadius::new(request.blur_radius);
        let tint = Tint::from_srgb_unmultiplied(request.tint.to_srgba_unmultiplied());
        let mask_radius = BlurCornerRadius::new(r);
        let presence = Presence::new(request.presence);

        let callback = egui_glow::CallbackFn::new(move |info, painter| {
            let Some((region, fb_size)) = callback_region(&info) else {
                return;
            };
            let gl = painter.gl();
            let target = current_draw_framebuffer(gl);

            let mut guard = match blur.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    blur.clear_poison();
                    poisoned.into_inner()
                }
            };

            let blur_req = BlurRequest {
                source_region: region.into_region(),
                target_rect: region.into_region(),
                blur_radius,
                tint,
                corner_radius: mask_radius,
                presence,
            };
            let _ = guard.frost_region(gl, target, region, fb_size, &blur_req);
        });

        shapes.push(Shape::Callback(PaintCallback {
            rect: request.rect,
            callback: Arc::new(callback),
        }));

        // 3. Layer crisp directional 3D bevel + inner thickness highlight over the blurred surface.
        let bevel_tokens = GlassSurfaceTokens {
            tint: request.tint,
            gradient_top: request.gradient_top,
            gradient_bottom: request.gradient_bottom,
            border: request.border,
            specular_top: request.specular_top,
            specular_accent: request.specular_accent,
            inner_glow: request.inner_glow,
            shadow: request.shadow,
            contact_shadow: request.contact_shadow,
            corner_radius: r,
            blur_radius: request.blur_radius,
            border_width: request.border_width,
            prism_intensity: request.prism_intensity,
            vibrancy: request.vibrancy,
            refraction_strength: request.refraction_strength,
            chromatic_dispersion: request.chromatic_dispersion,
            grain_intensity: request.grain_intensity,
            blur_tint_alpha: 1.0,
        };
        shapes.extend(glass_bevel_and_shadow_shapes(
            request.rect,
            &bevel_tokens,
            false,
        ));

        Some(shapes)
    }
}

/// Converts `egui`'s callback geometry directly into a GL bottom-left [`GlRegion`] and full
/// [`FramebufferSize`], intersecting `viewport ∩ clip_rect ∩ framebuffer`.
fn callback_region(info: &PaintCallbackInfo) -> Option<(GlRegion, FramebufferSize)> {
    let ppp = info.pixels_per_point;
    let to_gl = |v: &egui::epaint::ViewportInPixels| {
        GlRegion::from_bottom_px(
            [v.left_px.max(0) as u32, v.from_bottom_px.max(0) as u32],
            [v.width_px.max(0) as u32, v.height_px.max(0) as u32],
            Scale::new(ppp),
        )
    };
    let viewport = to_gl(&info.viewport_in_pixels());
    let clip = to_gl(&info.clip_rect_in_pixels());
    let region = viewport.intersect(&clip)?.clip_to(info.screen_size_px)?;
    Some((region, FramebufferSize(info.screen_size_px)))
}
