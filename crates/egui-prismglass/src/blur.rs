//! Backend-agnostic backdrop-blur seam.
//!
//! `egui-prismglass` is **fallback-first**: every surface looks crisp and glassmorphic
//! on any `egui` backend (`eframe+wgpu`, `eframe+glow`, WASM, own-loop `egui-wgpu`,
//! or headless `egui_kittest`) with zero GPU dependencies.
//!
//! This module defines the vocabulary (`BackdropRequest`, `BlurCapability`, and
//! `BackdropBlurProvider`) so real GPU backdrop blur (via `backdrop-blur-egui` under
//! upcoming `blur-wgpu` and `blur-glow` features) can plug directly into every widget
//! without breaking public APIs.

use egui::{Color32, Rect, Ui};

use crate::tokens::GlassSurfaceTokens;

/// Describes a frosted backdrop region requested by a glass widget for the current frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropRequest {
    /// Target rectangle in logical `egui` points.
    pub rect: Rect,
    /// Corner radius in logical points.
    pub corner_radius: f32,
    /// Desired backdrop blur radius in logical points (`0.0` disables blur).
    pub blur_radius: f32,
    /// Film tint color composited over the blurred backdrop.
    pub tint: Color32,
    /// Visibility/presence dial (`0.0..=1.0`) for animated fade-in/out surfaces.
    pub presence: f32,
}

impl BackdropRequest {
    /// Builds a [`BackdropRequest`] from a target [`Rect`] and [`GlassSurfaceTokens`].
    pub fn from_tokens(rect: Rect, tokens: &GlassSurfaceTokens) -> Self {
        Self {
            rect,
            corner_radius: tokens.corner_radius,
            blur_radius: tokens.blur_radius,
            tint: tokens.tint,
            presence: 1.0,
        }
    }
}

/// Reports whether real GPU backdrop blur is active or fallback glass rendering is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlurCapability {
    /// Pure `egui` translucent glass fallback (works on all backends).
    FallbackOnly,
    /// Real GPU backdrop blur backed by a named adapter (e.g. `"glow"`, `"wgpu"`).
    RealBackdrop {
        /// Human-readable backend identifier.
        backend: &'static str,
    },
}

/// Extension seam for plugging a real GPU backdrop-blur backend into `egui-prismglass`.
///
/// Widgets call [`BackdropBlurProvider::enqueue_backdrop`] *before* painting their
/// foreground contents, matching the contract required by `backdrop-blur-egui`.
pub trait BackdropBlurProvider {
    /// Returns the active blur capability of this provider.
    fn capability(&self) -> BlurCapability;

    /// Enqueues a frosted backdrop pass behind a glass surface.
    fn enqueue_backdrop(&self, ui: &Ui, request: BackdropRequest);
}

/// Default zero-dependency fallback provider.
///
/// Does not enqueue GPU blur passes; surfaces are rendered via
/// [`crate::painter::paint_glass_surface`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FallbackBlurProvider;

impl BackdropBlurProvider for FallbackBlurProvider {
    fn capability(&self) -> BlurCapability {
        BlurCapability::FallbackOnly
    }

    fn enqueue_backdrop(&self, _ui: &Ui, _request: BackdropRequest) {}
}
