//! Pure-WGSL real GPU backdrop blur & Apple Liquid Glass engine for `wgpu` / `egui-wgpu`.
//!
//! Enabled via `feature = "blur-wgpu"`. Works across `eframe` (default `wgpu` backend),
//! custom `egui-wgpu` render loops, and headless `egui_kittest` snapshot testing with
//! `#![forbid(unsafe_code)]`.
//!
//! # Architecture
//!
//! 1. **Backdrop Capture / Generation**: Either renders the active [`crate::BackdropScene`]
//!    procedurally on the GPU into a linear `Rgba16Float` source texture, or ingests a
//!    user-supplied RGBA8 wallpaper via [`WgpuBlurProvider::set_external_backdrop_rgba8`].
//! 2. **4-Level Linear Dual-Kawase Blur Pyramid**: Runs once per frame in
//!    [`egui_wgpu::CallbackTrait::prepare`] (4 downsample passes + medium & deep tent-filter
//!    upsample passes), giving smooth continuous blur from `0` to `48` pt with zero 8-bit banding.
//! 3. **Liquid Glass Composite**: Each glass surface (`GlassCard`, `GlassPanel`, `GlassSidebar`)
//!    renders a single GPU quad directly inside its reserved `ShapeIdx` slot with analytic
//!    rounded-rect SDF anti-aliasing, curved-bevel lens refraction, subtle RGB chromatic
//!    dispersion, macOS vibrancy boost, translucent glass film gradient, Jimenez IGN micro-grain,
//!    and directional 3D studio bevel speculars.
//!
//! # Quick Setup (`eframe`)
//!
//! ```ignore
//! eframe::run_native("My App", opts, Box::new(|cc| {
//!     if let Some(render_state) = cc.wgpu_render_state.as_ref() {
//!         egui_prismglass::blur_wgpu::install(&cc.egui_ctx, render_state);
//!     }
//!     Ok(Box::new(MyApp::new(cc)))
//! }))
//! ```

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

use bytemuck::{Pod, Zeroable};
use egui::{Color32, Context, Id, Rect, Shape, Ui};
pub use egui_wgpu::wgpu;
use egui_wgpu::{Callback, CallbackResources, CallbackTrait, ScreenDescriptor};

use crate::blur::{BackdropBlurProvider, BackdropRequest, BlurCapability, set_blur_provider};
use crate::theme::{PrismTheme, ThemeMode};

const BACKDROP_WGSL: &str = include_str!("shaders/backdrop_studio.wgsl");
const KAWASE_DOWN_WGSL: &str = include_str!("shaders/kawase_downsample.wgsl");
const KAWASE_UP_WGSL: &str = include_str!("shaders/kawase_upsample.wgsl");
const LIQUID_GLASS_WGSL: &str = include_str!("shaders/liquid_glass_composite.wgsl");

const PYRAMID_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Installs a [`WgpuBlurProvider`] configured from an `eframe` / `egui-wgpu` [`egui_wgpu::RenderState`].
///
/// Returns the shared provider handle so callers can optionally upload external backdrop images
/// via [`WgpuBlurProvider::set_external_backdrop_rgba8`].
pub fn install(ctx: &Context, render_state: &egui_wgpu::RenderState) -> Arc<WgpuBlurProvider> {
    let provider = Arc::new(WgpuBlurProvider::from_render_state(render_state));
    set_blur_provider(ctx, provider.clone());
    provider
}

/// Installs a [`WgpuBlurProvider`] for an explicit target [`wgpu::TextureFormat`].
///
/// Useful for headless test harnesses (such as `egui_kittest`, which renders to
/// [`wgpu::TextureFormat::Rgba8Unorm`]) or custom `wgpu` hosts that do not expose
/// [`egui_wgpu::RenderState`] during UI construction.
pub fn install_with_format(
    ctx: &Context,
    target_format: wgpu::TextureFormat,
) -> Arc<WgpuBlurProvider> {
    let provider = Arc::new(WgpuBlurProvider::new(target_format));
    set_blur_provider(ctx, provider.clone());
    provider
}

#[derive(Clone)]
struct BackdropParams {
    canvas_top: [f32; 4],
    canvas_bottom: [f32; 4],
    orb_primary: [f32; 4],
    orb_secondary: [f32; 4],
    orb_accent: [f32; 4],
    time_secs: f32,
    intensity: f32,
    scene_id: u32,
    is_dark: bool,
}

impl Default for BackdropParams {
    fn default() -> Self {
        let theme = PrismTheme::onyx();
        Self::from_theme(&theme, 0.0, 1.0)
    }
}

impl BackdropParams {
    fn from_theme(theme: &PrismTheme, time_secs: f32, intensity: f32) -> Self {
        Self {
            canvas_top: color32_to_linear_rgba(theme.palette.canvas),
            canvas_bottom: color32_to_linear_rgba(theme.palette.canvas_secondary),
            orb_primary: color32_to_linear_rgba(theme.palette.orb_primary),
            orb_secondary: color32_to_linear_rgba(theme.palette.orb_secondary),
            orb_accent: color32_to_linear_rgba(theme.palette.orb_accent),
            time_secs,
            intensity: intensity.clamp(0.0, 1.5),
            scene_id: theme.backdrop_scene.shader_id(),
            is_dark: theme.mode == ThemeMode::Dark,
        }
    }
}

struct ExternalBackdropImage {
    width: u32,
    height: u32,
    rgba8: Vec<u8>,
    generation: u64,
}

struct SharedBackdropState {
    target_format: wgpu::TextureFormat,
    params: BackdropParams,
    external_image: Option<ExternalBackdropImage>,
    next_generation: u64,
}

/// Real GPU backdrop-blur provider powered by pure-WGSL Dual-Kawase + Apple Liquid Glass optics.
pub struct WgpuBlurProvider {
    shared: Arc<RwLock<SharedBackdropState>>,
}

impl WgpuBlurProvider {
    /// Creates a new provider targeting `target_format`.
    #[must_use]
    pub fn new(target_format: wgpu::TextureFormat) -> Self {
        Self {
            shared: Arc::new(RwLock::new(SharedBackdropState {
                target_format,
                params: BackdropParams::default(),
                external_image: None,
                next_generation: 1,
            })),
        }
    }

    /// Creates a new provider configured from an [`egui_wgpu::RenderState`].
    #[must_use]
    pub fn from_render_state(render_state: &egui_wgpu::RenderState) -> Self {
        Self::new(render_state.target_format)
    }

    /// Uploads a custom sRGB RGBA8 backdrop image (e.g., wallpaper or 3D viewport render) to be
    /// blurred behind every glass surface instead of the procedural studio backdrop.
    ///
    /// `rgba8.len()` must equal `width * height * 4`. Invalid buffers are ignored.
    pub fn set_external_backdrop_rgba8(&self, width: u32, height: u32, rgba8: Vec<u8>) {
        if width == 0 || height == 0 || rgba8.len() != (width as usize) * (height as usize) * 4 {
            return;
        }
        if let Ok(mut state) = self.shared.write() {
            let generation = state.next_generation;
            state.next_generation = state.next_generation.wrapping_add(1).max(1);
            state.external_image = Some(ExternalBackdropImage {
                width,
                height,
                rgba8,
                generation,
            });
        }
    }

    /// Clears any custom backdrop image and reverts to the procedural WGSL studio backdrop.
    pub fn clear_external_backdrop(&self) {
        if let Ok(mut state) = self.shared.write() {
            state.external_image = None;
        }
    }
}

impl BackdropBlurProvider for WgpuBlurProvider {
    fn capability(&self) -> BlurCapability {
        BlurCapability::RealBackdrop {
            backend: "wgpu-dual-kawase",
        }
    }

    fn render_backdrop(
        &self,
        ui: &Ui,
        rect: Rect,
        theme: &PrismTheme,
        time_secs: f32,
        intensity: f32,
    ) -> bool {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return false;
        }

        let params = BackdropParams::from_theme(theme, time_secs, intensity);
        if let Ok(mut state) = self.shared.write() {
            state.params = params.clone();
        }

        let frame_id = ui.ctx().cumulative_frame_nr();
        let callback = BackdropPaintCallback {
            shared: Arc::clone(&self.shared),
            params,
            frame_id,
        };
        ui.painter()
            .add(Callback::new_paint_callback(rect, callback));
        true
    }

    fn render_glass_surface(
        &self,
        ui: &Ui,
        _id: Id,
        request: &BackdropRequest,
    ) -> Option<Vec<Shape>> {
        if request.rect.width() <= 0.0 || request.rect.height() <= 0.0 {
            return Some(Vec::new());
        }

        let mut shapes = Vec::with_capacity(3);
        let max_r = (request.rect.width().min(request.rect.height()) * 0.5).max(0.0);
        let corner_radius = egui::CornerRadius::same(
            request.corner_radius.min(max_r).round().clamp(0.0, 255.0) as u8,
        );

        // Grounded 3D dual-layer shadows are painted underneath the GPU liquid-glass quad.
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

        let frame_id = ui.ctx().cumulative_frame_nr();
        let callback = GlassSurfaceCallback {
            shared: Arc::clone(&self.shared),
            request: *request,
            frame_id,
            slot_index: AtomicU32::new(u32::MAX),
        };
        shapes.push(Shape::Callback(Callback::new_paint_callback(
            request.rect,
            callback,
        )));

        Some(shapes)
    }
}

// ---------------------------------------------------------------------------
// GPU Uniform Layouts (16-byte aligned, Pod/Zeroable, zero unsafe)
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BackdropUniforms {
    canvas_top: [f32; 4],
    canvas_bottom: [f32; 4],
    orb_primary: [f32; 4],
    orb_secondary: [f32; 4],
    orb_accent: [f32; 4],
    screen_size: [f32; 2],
    time_secs: f32,
    intensity: f32,
    scene_id: u32,
    is_dark: u32,
    target_is_srgb: u32,
    pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct KawaseUniforms {
    half_pixel: [f32; 2],
    offset: f32,
    pad: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GlassCardUniforms {
    rect_px: [f32; 4],
    screen_and_geom: [f32; 4],
    optics: [f32; 4],
    surface_fx: [f32; 4],
    gradient_top: [f32; 4],
    gradient_bottom: [f32; 4],
    border: [f32; 4],
    specular_top: [f32; 4],
    specular_accent: [f32; 4],
    inner_glow: [f32; 4],
    flags: [u32; 4],
}

impl GlassCardUniforms {
    fn from_request(
        req: &BackdropRequest,
        screen: &ScreenDescriptor,
        target_is_srgb: bool,
    ) -> Self {
        let ppp = screen.pixels_per_point.max(1.0);
        let min_x = req.rect.min.x * ppp;
        let min_y = req.rect.min.y * ppp;
        let width = req.rect.width() * ppp;
        let height = req.rect.height() * ppp;

        Self {
            rect_px: [min_x, min_y, width, height],
            screen_and_geom: [
                screen.size_in_pixels[0] as f32,
                screen.size_in_pixels[1] as f32,
                req.corner_radius * ppp,
                ppp,
            ],
            optics: [
                req.blur_radius * ppp,
                req.vibrancy,
                req.refraction_strength,
                req.chromatic_dispersion,
            ],
            surface_fx: [
                req.grain_intensity,
                (req.border_width * ppp).max(1.0),
                req.prism_intensity,
                req.presence,
            ],
            gradient_top: color32_to_linear_rgba(req.gradient_top),
            gradient_bottom: color32_to_linear_rgba(req.gradient_bottom),
            border: color32_to_linear_rgba(req.border),
            specular_top: color32_to_linear_rgba(req.specular_top),
            specular_accent: color32_to_linear_rgba(req.specular_accent),
            inner_glow: color32_to_linear_rgba(req.inner_glow),
            flags: [u32::from(req.is_dark), u32::from(target_is_srgb), 0, 0],
        }
    }
}

// ---------------------------------------------------------------------------
// Paint Callbacks
// ---------------------------------------------------------------------------

struct BackdropPaintCallback {
    shared: Arc<RwLock<SharedBackdropState>>,
    params: BackdropParams,
    frame_id: u64,
}

impl CallbackTrait for BackdropPaintCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let target_format = self
            .shared
            .read()
            .map_or(wgpu::TextureFormat::Rgba8Unorm, |s| s.target_format);
        let engine = ensure_engine(device, target_format, callback_resources);
        engine.prepare_frame_pyramid(
            device,
            queue,
            screen_descriptor,
            egui_encoder,
            self.frame_id,
            &self.params,
            &self.shared,
        );
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let Some(engine) = callback_resources.get::<WgpuBlurEngine>() else {
            return;
        };
        let Some(pyramid) = engine.pyramid.as_ref() else {
            return;
        };

        render_pass.set_viewport(
            0.0,
            0.0,
            info.screen_size_px[0] as f32,
            info.screen_size_px[1] as f32,
            0.0,
            1.0,
        );
        render_pass.set_pipeline(&engine.blit_screen_pipeline);
        render_pass.set_bind_group(0, &engine.backdrop_uniform_bg, &[]);
        render_pass.set_bind_group(1, &pyramid.blit_source_bg, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

struct GlassSurfaceCallback {
    shared: Arc<RwLock<SharedBackdropState>>,
    request: BackdropRequest,
    frame_id: u64,
    slot_index: AtomicU32,
}

impl CallbackTrait for GlassSurfaceCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let (target_format, fallback_params) = self.shared.read().map_or_else(
            |_| (wgpu::TextureFormat::Rgba8Unorm, BackdropParams::default()),
            |s| (s.target_format, s.params.clone()),
        );
        let engine = ensure_engine(device, target_format, callback_resources);
        engine.prepare_frame_pyramid(
            device,
            queue,
            screen_descriptor,
            egui_encoder,
            self.frame_id,
            &fallback_params,
            &self.shared,
        );

        let uniforms = GlassCardUniforms::from_request(
            &self.request,
            screen_descriptor,
            engine.target_format.is_srgb(),
        );
        let slot = engine.push_card(self.frame_id, uniforms);
        self.slot_index.store(slot, Ordering::Relaxed);
        Vec::new()
    }

    fn finish_prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(engine) = callback_resources.get_mut::<WgpuBlurEngine>() {
            engine.flush_cards_to_gpu(device, queue, self.frame_id);
        }
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let slot = self.slot_index.load(Ordering::Relaxed);
        if slot == u32::MAX {
            return;
        }
        let Some(engine) = callback_resources.get::<WgpuBlurEngine>() else {
            return;
        };
        let Some(pyramid) = engine.pyramid.as_ref() else {
            return;
        };
        if (slot as usize) >= engine.uploaded_card_count {
            return;
        }

        let dynamic_offset = slot * engine.card_stride;
        render_pass.set_viewport(
            0.0,
            0.0,
            info.screen_size_px[0] as f32,
            info.screen_size_px[1] as f32,
            0.0,
            1.0,
        );
        render_pass.set_pipeline(&engine.glass_pipeline);
        render_pass.set_bind_group(0, &engine.card_uniform_bg, &[dynamic_offset]);
        render_pass.set_bind_group(1, &pyramid.composite_bg, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

// ---------------------------------------------------------------------------
// Internal GPU Engine & Dual-Kawase Pyramid
// ---------------------------------------------------------------------------

struct RenderTarget {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: [u32; 2],
}

impl RenderTarget {
    fn new(device: &wgpu::Device, width: u32, height: u32, label: &str) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PYRAMID_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            _texture: texture,
            view,
            size: [width, height],
        }
    }
}

struct KawasePassStep {
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

struct BlurPyramid {
    size: [u32; 2],
    source: RenderTarget,
    down1: RenderTarget,
    down2: RenderTarget,
    down3: RenderTarget,
    down4: RenderTarget,
    up3: RenderTarget,
    up2: RenderTarget,
    blur_med: RenderTarget,
    blur_deep: RenderTarget,
    // Pre-built bind groups for zero per-frame allocation:
    blit_source_bg: wgpu::BindGroup,
    composite_bg: wgpu::BindGroup,
    step_down1: KawasePassStep,
    step_down2: KawasePassStep,
    step_down3: KawasePassStep,
    step_down4: KawasePassStep,
    step_med: KawasePassStep,
    step_up3: KawasePassStep,
    step_up2: KawasePassStep,
    step_deep: KawasePassStep,
}

struct UploadedExternal {
    generation: u64,
    _texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

struct WgpuBlurEngine {
    target_format: wgpu::TextureFormat,
    sampler: wgpu::Sampler,

    // Layouts
    single_tex_bgl: wgpu::BindGroupLayout,
    kawase_bgl: wgpu::BindGroupLayout,
    composite_tex_bgl: wgpu::BindGroupLayout,
    card_uniform_bgl: wgpu::BindGroupLayout,

    // Pipelines
    generate_backdrop_pipeline: wgpu::RenderPipeline,
    copy_external_pipeline: wgpu::RenderPipeline,
    blit_screen_pipeline: wgpu::RenderPipeline,
    kawase_down_pipeline: wgpu::RenderPipeline,
    kawase_up_pipeline: wgpu::RenderPipeline,
    glass_pipeline: wgpu::RenderPipeline,

    // Backdrop uniforms (screen blit + procedural generator)
    backdrop_uniform_buf: wgpu::Buffer,
    backdrop_uniform_bg: wgpu::BindGroup,
    copy_linear_uniform_buf: wgpu::Buffer,
    copy_linear_uniform_bg: wgpu::BindGroup,

    // Pyramid & external texture caches
    pyramid: Option<BlurPyramid>,
    external: Option<UploadedExternal>,

    // Per-frame dynamic card uniforms
    card_stride: u32,
    card_capacity: usize,
    card_uniform_buf: wgpu::Buffer,
    card_uniform_bg: wgpu::BindGroup,
    staged_cards: Vec<GlassCardUniforms>,
    staged_bytes: Vec<u8>,
    uploaded_card_count: usize,

    // Per-frame deduplication
    pyramid_frame_id: Option<u64>,
    cards_frame_id: Option<u64>,
    flushed_frame_id: Option<u64>,
}

fn ensure_engine<'a>(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    resources: &'a mut CallbackResources,
) -> &'a mut WgpuBlurEngine {
    let needs_recreate = resources
        .get::<WgpuBlurEngine>()
        .is_none_or(|engine| engine.target_format != target_format);
    if needs_recreate {
        resources.insert(WgpuBlurEngine::new(device, target_format));
    }
    resources
        .get_mut::<WgpuBlurEngine>()
        .expect("WgpuBlurEngine just inserted")
}

impl WgpuBlurEngine {
    #[allow(clippy::too_many_lines)]
    fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("prismglass_linear_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let backdrop_uniform_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("prismglass_backdrop_uniform_bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let single_tex_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("prismglass_single_tex_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let kawase_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("prismglass_kawase_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let card_uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("prismglass_card_uniform_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<GlassCardUniforms>() as u64,
                    ),
                },
                count: None,
            }],
        });

        let composite_tex_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("prismglass_composite_tex_bgl"),
            entries: &[
                tex_entry(0),
                tex_entry(1),
                tex_entry(2),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let backdrop_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("prismglass_backdrop_wgsl"),
            source: wgpu::ShaderSource::Wgsl(BACKDROP_WGSL.into()),
        });
        let kawase_down_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("prismglass_kawase_down_wgsl"),
            source: wgpu::ShaderSource::Wgsl(KAWASE_DOWN_WGSL.into()),
        });
        let kawase_up_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("prismglass_kawase_up_wgsl"),
            source: wgpu::ShaderSource::Wgsl(KAWASE_UP_WGSL.into()),
        });
        let glass_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("prismglass_liquid_glass_wgsl"),
            source: wgpu::ShaderSource::Wgsl(LIQUID_GLASS_WGSL.into()),
        });

        let gen_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("prismglass_gen_layout"),
            bind_group_layouts: &[Some(&backdrop_uniform_bgl)],
            immediate_size: 0,
        });
        let blit_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("prismglass_blit_layout"),
            bind_group_layouts: &[Some(&backdrop_uniform_bgl), Some(&single_tex_bgl)],
            immediate_size: 0,
        });
        let kawase_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("prismglass_kawase_layout"),
            bind_group_layouts: &[Some(&kawase_bgl)],
            immediate_size: 0,
        });
        let glass_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("prismglass_glass_layout"),
            bind_group_layouts: &[Some(&card_uniform_bgl), Some(&composite_tex_bgl)],
            immediate_size: 0,
        });

        let generate_backdrop_pipeline = create_simple_pipeline(
            device,
            "prismglass_generate_backdrop",
            &gen_layout,
            &backdrop_shader,
            "vs_fullscreen",
            "fs_generate",
            PYRAMID_FORMAT,
            None,
        );
        let copy_external_pipeline = create_simple_pipeline(
            device,
            "prismglass_copy_external",
            &blit_layout,
            &backdrop_shader,
            "vs_fullscreen",
            "fs_blit",
            PYRAMID_FORMAT,
            None,
        );
        let blit_screen_pipeline = create_simple_pipeline(
            device,
            "prismglass_blit_screen",
            &blit_layout,
            &backdrop_shader,
            "vs_fullscreen",
            "fs_blit",
            target_format,
            None,
        );
        let kawase_down_pipeline = create_simple_pipeline(
            device,
            "prismglass_kawase_down",
            &kawase_layout,
            &kawase_down_shader,
            "vs_main",
            "fs_main",
            PYRAMID_FORMAT,
            None,
        );
        let kawase_up_pipeline = create_simple_pipeline(
            device,
            "prismglass_kawase_up",
            &kawase_layout,
            &kawase_up_shader,
            "vs_main",
            "fs_main",
            PYRAMID_FORMAT,
            None,
        );
        let glass_pipeline = create_simple_pipeline(
            device,
            "prismglass_liquid_glass",
            &glass_layout,
            &glass_shader,
            "vs_main",
            "fs_main",
            target_format,
            Some(wgpu::BlendState::ALPHA_BLENDING),
        );

        let backdrop_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("prismglass_backdrop_uniform_buf"),
            size: std::mem::size_of::<BackdropUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let backdrop_uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("prismglass_backdrop_uniform_bg"),
            layout: &backdrop_uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: backdrop_uniform_buf.as_entire_binding(),
            }],
        });

        // Static uniform with `target_is_srgb = 1` for linear-to-linear external texture copy.
        let copy_linear_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("prismglass_copy_linear_uniform_buf"),
            size: std::mem::size_of::<BackdropUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let copy_linear_uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("prismglass_copy_linear_uniform_bg"),
            layout: &backdrop_uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: copy_linear_uniform_buf.as_entire_binding(),
            }],
        });

        let align = device.limits().min_uniform_buffer_offset_alignment.max(256);
        let raw_size = std::mem::size_of::<GlassCardUniforms>() as u32;
        let card_stride = raw_size.div_ceil(align) * align;
        let card_capacity = 32;
        let (card_uniform_buf, card_uniform_bg) =
            create_card_buffer(device, &card_uniform_bgl, card_stride, card_capacity);

        Self {
            target_format,
            sampler,
            single_tex_bgl,
            kawase_bgl,
            composite_tex_bgl,
            card_uniform_bgl,
            generate_backdrop_pipeline,
            copy_external_pipeline,
            blit_screen_pipeline,
            kawase_down_pipeline,
            kawase_up_pipeline,
            glass_pipeline,
            backdrop_uniform_buf,
            backdrop_uniform_bg,
            copy_linear_uniform_buf,
            copy_linear_uniform_bg,
            pyramid: None,
            external: None,
            card_stride,
            card_capacity,
            card_uniform_buf,
            card_uniform_bg,
            staged_cards: Vec::with_capacity(card_capacity),
            staged_bytes: Vec::with_capacity(card_capacity * (card_stride as usize)),
            uploaded_card_count: 0,
            pyramid_frame_id: None,
            cards_frame_id: None,
            flushed_frame_id: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_frame_pyramid(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        frame_id: u64,
        params: &BackdropParams,
        shared: &Arc<RwLock<SharedBackdropState>>,
    ) {
        if self.pyramid_frame_id == Some(frame_id) {
            return;
        }
        self.pyramid_frame_id = Some(frame_id);

        let width = screen.size_in_pixels[0].max(1);
        let height = screen.size_in_pixels[1].max(1);
        let needs_resize = self
            .pyramid
            .as_ref()
            .is_none_or(|p| p.size != [width, height]);
        if needs_resize {
            self.pyramid = Some(self.build_pyramid(device, queue, width, height));
        }

        // Synchronize optional external wallpaper texture if present.
        self.sync_external_texture(device, queue, shared);

        let backdrop_uniform = BackdropUniforms {
            canvas_top: params.canvas_top,
            canvas_bottom: params.canvas_bottom,
            orb_primary: params.orb_primary,
            orb_secondary: params.orb_secondary,
            orb_accent: params.orb_accent,
            screen_size: [width as f32, height as f32],
            time_secs: params.time_secs,
            intensity: params.intensity,
            scene_id: params.scene_id,
            is_dark: u32::from(params.is_dark),
            target_is_srgb: u32::from(self.target_format.is_srgb()),
            pad: 0,
        };
        queue.write_buffer(
            &self.backdrop_uniform_buf,
            0,
            bytemuck::bytes_of(&backdrop_uniform),
        );

        let mut linear_copy_uniform = backdrop_uniform;
        linear_copy_uniform.target_is_srgb = 1;
        queue.write_buffer(
            &self.copy_linear_uniform_buf,
            0,
            bytemuck::bytes_of(&linear_copy_uniform),
        );

        let Some(pyramid) = self.pyramid.as_ref() else {
            return;
        };

        // Pass 0: Populate linear `source` texture (either external wallpaper or procedural studio WGSL).
        {
            let mut pass = begin_color_pass(encoder, &pyramid.source.view, "prismglass_source");
            if let Some(ext) = self.external.as_ref() {
                pass.set_pipeline(&self.copy_external_pipeline);
                pass.set_bind_group(0, &self.copy_linear_uniform_bg, &[]);
                pass.set_bind_group(1, &ext.bind_group, &[]);
                pass.draw(0..3, 0..1);
            } else {
                pass.set_pipeline(&self.generate_backdrop_pipeline);
                pass.set_bind_group(0, &self.backdrop_uniform_bg, &[]);
                pass.draw(0..3, 0..1);
            }
        }

        // Passes 1..4: 4-level Dual-Kawase downsample chain (`1/2`, `1/4`, `1/8`, `1/16`).
        run_kawase_pass(
            encoder,
            &self.kawase_down_pipeline,
            &pyramid.step_down1,
            &pyramid.down1.view,
            "prismglass_down1",
        );
        run_kawase_pass(
            encoder,
            &self.kawase_down_pipeline,
            &pyramid.step_down2,
            &pyramid.down2.view,
            "prismglass_down2",
        );
        run_kawase_pass(
            encoder,
            &self.kawase_down_pipeline,
            &pyramid.step_down3,
            &pyramid.down3.view,
            "prismglass_down3",
        );
        run_kawase_pass(
            encoder,
            &self.kawase_down_pipeline,
            &pyramid.step_down4,
            &pyramid.down4.view,
            "prismglass_down4",
        );

        // Pass 5: Medium blur target (`down2 -> blur_med` at 1/2 res).
        run_kawase_pass(
            encoder,
            &self.kawase_up_pipeline,
            &pyramid.step_med,
            &pyramid.blur_med.view,
            "prismglass_blur_med",
        );

        // Passes 6..8: Deep frosted blur chain (`down4 -> up3 -> up2 -> blur_deep` at 1/2 res).
        run_kawase_pass(
            encoder,
            &self.kawase_up_pipeline,
            &pyramid.step_up3,
            &pyramid.up3.view,
            "prismglass_up3",
        );
        run_kawase_pass(
            encoder,
            &self.kawase_up_pipeline,
            &pyramid.step_up2,
            &pyramid.up2.view,
            "prismglass_up2",
        );
        run_kawase_pass(
            encoder,
            &self.kawase_up_pipeline,
            &pyramid.step_deep,
            &pyramid.blur_deep.view,
            "prismglass_blur_deep",
        );
    }

    fn sync_external_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shared: &Arc<RwLock<SharedBackdropState>>,
    ) {
        let Ok(state) = shared.read() else {
            return;
        };
        let Some(ext) = state.external_image.as_ref() else {
            self.external = None;
            return;
        };
        if self
            .external
            .as_ref()
            .is_some_and(|uploaded| uploaded.generation == ext.generation)
        {
            return;
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("prismglass_external_backdrop"),
            size: wgpu::Extent3d {
                width: ext.width,
                height: ext.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &ext.rgba8,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ext.width * 4),
                rows_per_image: Some(ext.height),
            },
            wgpu::Extent3d {
                width: ext.width,
                height: ext.height,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("prismglass_external_bg"),
            layout: &self.single_tex_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        self.external = Some(UploadedExternal {
            generation: ext.generation,
            _texture: texture,
            bind_group,
        });
    }

    fn build_pyramid(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> BlurPyramid {
        let w1 = (width / 2).max(1);
        let h1 = (height / 2).max(1);
        let w2 = (width / 4).max(1);
        let h2 = (height / 4).max(1);
        let w3 = (width / 8).max(1);
        let h3 = (height / 8).max(1);
        let w4 = (width / 16).max(1);
        let h4 = (height / 16).max(1);

        let source = RenderTarget::new(device, width, height, "prismglass_source");
        let down1 = RenderTarget::new(device, w1, h1, "prismglass_down1");
        let down2 = RenderTarget::new(device, w2, h2, "prismglass_down2");
        let down3 = RenderTarget::new(device, w3, h3, "prismglass_down3");
        let down4 = RenderTarget::new(device, w4, h4, "prismglass_down4");
        let up3 = RenderTarget::new(device, w3, h3, "prismglass_up3");
        let up2 = RenderTarget::new(device, w2, h2, "prismglass_up2");
        let blur_med = RenderTarget::new(device, w1, h1, "prismglass_blur_med");
        let blur_deep = RenderTarget::new(device, w1, h1, "prismglass_blur_deep");

        let blit_source_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("prismglass_blit_source_bg"),
            layout: &self.single_tex_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        let composite_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("prismglass_composite_bg"),
            layout: &self.composite_tex_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&blur_med.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&blur_deep.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        let step_down1 = self.make_kawase_step(device, queue, &source.view, down1.size, 1.2);
        let step_down2 = self.make_kawase_step(device, queue, &down1.view, down2.size, 1.35);
        let step_down3 = self.make_kawase_step(device, queue, &down2.view, down3.size, 1.45);
        let step_down4 = self.make_kawase_step(device, queue, &down3.view, down4.size, 1.55);

        let step_med = self.make_kawase_step(device, queue, &down2.view, blur_med.size, 1.25);

        let step_up3 = self.make_kawase_step(device, queue, &down4.view, up3.size, 1.5);
        let step_up2 = self.make_kawase_step(device, queue, &up3.view, up2.size, 1.4);
        let step_deep = self.make_kawase_step(device, queue, &up2.view, blur_deep.size, 1.35);

        BlurPyramid {
            size: [width, height],
            source,
            down1,
            down2,
            down3,
            down4,
            up3,
            up2,
            blur_med,
            blur_deep,
            blit_source_bg,
            composite_bg,
            step_down1,
            step_down2,
            step_down3,
            step_down4,
            step_med,
            step_up3,
            step_up2,
            step_deep,
        }
    }

    fn make_kawase_step(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        src_view: &wgpu::TextureView,
        dst_size: [u32; 2],
        offset: f32,
    ) -> KawasePassStep {
        let uniform = KawaseUniforms {
            half_pixel: [0.5 / dst_size[0] as f32, 0.5 / dst_size[1] as f32],
            offset,
            pad: 0.0,
        };
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("prismglass_kawase_uniform"),
            size: std::mem::size_of::<KawaseUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform_buf, 0, bytemuck::bytes_of(&uniform));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("prismglass_kawase_step_bg"),
            layout: &self.kawase_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        KawasePassStep {
            uniform_buf,
            bind_group,
        }
    }

    fn push_card(&mut self, frame_id: u64, uniforms: GlassCardUniforms) -> u32 {
        if self.cards_frame_id != Some(frame_id) {
            self.cards_frame_id = Some(frame_id);
            self.staged_cards.clear();
        }
        let slot = self.staged_cards.len() as u32;
        self.staged_cards.push(uniforms);
        slot
    }

    fn flush_cards_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame_id: u64) {
        if self.flushed_frame_id == Some(frame_id) {
            return;
        }
        self.flushed_frame_id = Some(frame_id);

        let count = self.staged_cards.len();
        self.uploaded_card_count = count;
        if count == 0 {
            return;
        }

        if count > self.card_capacity {
            self.card_capacity = count.next_power_of_two().max(32);
            let (buf, bg) = create_card_buffer(
                device,
                &self.card_uniform_bgl,
                self.card_stride,
                self.card_capacity,
            );
            self.card_uniform_buf = buf;
            self.card_uniform_bg = bg;
        }

        let stride = self.card_stride as usize;
        let total_bytes = count * stride;
        self.staged_bytes.resize(total_bytes, 0);
        let uniform_size = std::mem::size_of::<GlassCardUniforms>();
        for (idx, card) in self.staged_cards.iter().enumerate() {
            let start = idx * stride;
            self.staged_bytes[start..start + uniform_size]
                .copy_from_slice(bytemuck::bytes_of(card));
        }
        queue.write_buffer(&self.card_uniform_buf, 0, &self.staged_bytes[..total_bytes]);
    }
}

fn tex_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn create_card_buffer(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    stride: u32,
    capacity: usize,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("prismglass_card_uniforms"),
        size: u64::from(stride) * (capacity as u64),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("prismglass_card_uniforms_bg"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: wgpu::BufferSize::new(std::mem::size_of::<GlassCardUniforms>() as u64),
            }),
        }],
    });
    (buffer, bind_group)
}

#[allow(clippy::too_many_arguments)]
fn create_simple_pipeline(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    vs_entry: &str,
    fs_entry: &str,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(vs_entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fs_entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn begin_color_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
    label: &'a str,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

fn run_kawase_pass(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    step: &KawasePassStep,
    dst_view: &wgpu::TextureView,
    label: &str,
) {
    let _ = &step.uniform_buf;
    let mut pass = begin_color_pass(encoder, dst_view, label);
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &step.bind_group, &[]);
    pass.draw(0..3, 0..1);
}

fn srgb_channel_to_linear(c: u8) -> f32 {
    let s = f32::from(c) / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

fn color32_to_linear_rgba(color: Color32) -> [f32; 4] {
    let [red, green, blue, alpha] = color.to_srgba_unmultiplied();
    [
        srgb_channel_to_linear(red),
        srgb_channel_to_linear(green),
        srgb_channel_to_linear(blue),
        f32::from(alpha) / 255.0,
    ]
}
