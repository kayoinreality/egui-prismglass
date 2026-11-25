//! Smooth, frame-rate-independent animation kit for `egui-prismglass`.
//!
//! Provides Apple-inspired spring physics, easing curves, animated 3D hover lift,
//! and pulse/shimmer helpers stored cleanly in [`egui::Context`] temporary memory.
//! Animations automatically request repaints only while active and settle to zero
//! CPU usage at rest.

use egui::{Context, Id};

/// Standard easing curves for duration-based transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Easing {
    /// Constant speed (`t`).
    Linear,
    /// Smooth deceleration (`1 - (1 - t)^3`), ideal for UI entrances and hover states.
    #[default]
    EaseOutCubic,
    /// Smooth acceleration and deceleration, ideal for tab switches and layout moves.
    EaseInOutCubic,
    /// Fast initial snap with silky exponential tail (Apple UI feel).
    EaseOutExpo,
    /// Subtle springy overshoot at the end of the curve.
    EaseOutBack,
}

impl Easing {
    /// Evaluates the easing curve for normalized progress `t` in `0.0..=1.0`.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
                }
            }
            Self::EaseOutExpo => {
                if t >= 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * t)
                }
            }
            Self::EaseOutBack => {
                let c1 = 1.701_58;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
        }
    }
}

/// Damped harmonic spring parameters for physical UI motion.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpringConfig {
    /// Spring stiffness (higher = faster snap).
    pub stiffness: f32,
    /// Viscous damping coefficient (higher = less overshoot).
    pub damping: f32,
    /// Settlement threshold below which the spring snaps cleanly to its target.
    pub precision: f32,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self::smooth()
    }
}

impl SpringConfig {
    /// Fast, crisp, critically damped spring for buttons, toggles, and micro-controls.
    pub fn snappy() -> Self {
        Self {
            stiffness: 420.0,
            damping: 34.0,
            precision: 0.001,
        }
    }

    /// Fluid Apple-style spring for cards, sidebar selection pills, and panel elevation.
    pub fn smooth() -> Self {
        Self {
            stiffness: 260.0,
            damping: 28.0,
            precision: 0.001,
        }
    }

    /// Playful, responsive spring with gentle overshoot for badges and popovers.
    pub fn bouncy() -> Self {
        Self {
            stiffness: 320.0,
            damping: 18.0,
            precision: 0.001,
        }
    }

    /// Calm, spacious spring for modals, drawers, and large view transitions.
    pub fn gentle() -> Self {
        Self {
            stiffness: 150.0,
            damping: 22.0,
            precision: 0.001,
        }
    }
}

/// Persistent per-widget state of a 1D damped spring simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringState {
    /// Current animated value.
    pub value: f32,
    /// Current velocity in units per second.
    pub velocity: f32,
}

impl SpringState {
    /// Creates a new spring initialized at rest at `initial`.
    pub fn new(initial: f32) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
        }
    }

    /// Advances the spring toward `target` over `dt` seconds.
    ///
    /// Returns `true` if the spring is still in motion (caller should request repaint).
    pub fn step(&mut self, target: f32, dt: f32, config: SpringConfig) -> bool {
        let dt = dt.clamp(0.0, 0.05);
        if dt <= 0.0 {
            return (target - self.value).abs() > config.precision
                || self.velocity.abs() > config.precision;
        }

        // Two sub-steps keep stiff springs rock-solid even at 30–60 FPS.
        let sub_dt = dt * 0.5;
        for _ in 0..2 {
            let displacement = target - self.value;
            let accel = displacement * config.stiffness - self.velocity * config.damping;
            self.velocity += accel * sub_dt;
            self.value += self.velocity * sub_dt;
        }

        let settled = (target - self.value).abs() <= config.precision
            && self.velocity.abs() <= config.precision;
        if settled {
            self.value = target;
            self.velocity = 0.0;
            false
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TweenState {
    start_value: f32,
    target_value: f32,
    start_time: f64,
    current_value: f32,
}

/// Evaluates and advances a persistent spring animation in [`egui::Context`] memory.
///
/// Automatically schedules repaints while settling and stops requesting repaints once at rest.
pub fn animate_spring(ctx: &Context, id: Id, target: f32, config: SpringConfig) -> f32 {
    let dt = ctx.input(|i| i.stable_dt);
    let key = id.with("prism_spring");

    let mut state = ctx
        .data(|d| d.get_temp::<SpringState>(key))
        .unwrap_or_else(|| SpringState::new(target));

    let moving = state.step(target, dt, config);
    ctx.data_mut(|d| d.insert_temp(key, state));

    if moving {
        ctx.request_repaint();
    }
    state.value
}

/// Evaluates and advances a duration-based eased tween in [`egui::Context`] memory.
pub fn animate_tween(
    ctx: &Context,
    id: Id,
    target: f32,
    duration_secs: f32,
    easing: Easing,
) -> f32 {
    let now = ctx.input(|i| i.time);
    let duration = f64::from(duration_secs.max(0.001));
    let key = id.with("prism_tween");

    let mut state = ctx
        .data(|d| d.get_temp::<TweenState>(key))
        .unwrap_or(TweenState {
            start_value: target,
            target_value: target,
            start_time: now,
            current_value: target,
        });

    if (state.target_value - target).abs() > f32::EPSILON {
        state.start_value = state.current_value;
        state.target_value = target;
        state.start_time = now;
    }

    let elapsed = (now - state.start_time).max(0.0);
    let raw_t = (elapsed / duration).clamp(0.0, 1.0) as f32;
    let eased = easing.apply(raw_t);
    state.current_value = state.start_value + (state.target_value - state.start_value) * eased;

    ctx.data_mut(|d| d.insert_temp(key, state));

    if raw_t < 1.0 {
        ctx.request_repaint();
    }

    state.current_value
}

/// Smoothly animates a normalized `0.0..=1.0` hover/press presence dial using a crisp spring.
pub fn animate_hover(ctx: &Context, id: Id, hovered: bool) -> f32 {
    animate_spring(
        ctx,
        id.with("hover"),
        if hovered { 1.0 } else { 0.0 },
        SpringConfig::snappy(),
    )
    .clamp(0.0, 1.0)
}

/// Computes a smooth continuous breathing pulse in `0.0..=1.0` (useful for live status indicators).
pub fn animate_pulse(ctx: &Context, cycles_per_sec: f32) -> f32 {
    let t = ctx.input(|i| i.time) as f32;
    ctx.request_repaint();
    0.5 + 0.5 * (t * cycles_per_sec * std::f32::consts::TAU).sin()
}

/// Computes a smooth repeating `0.0..=1.0` sweep phase (useful for skeleton shimmer effects).
pub fn animate_shimmer(ctx: &Context, cycles_per_sec: f32) -> f32 {
    let t = ctx.input(|i| i.time) as f32;
    ctx.request_repaint();
    (t * cycles_per_sec).fract()
}

/// Ergonomic animation helpers on [`egui::Context`].
pub trait PrismAnimExt {
    /// Advances a spring animation toward `target`.
    fn prism_spring(&self, id: Id, target: f32, config: SpringConfig) -> f32;

    /// Advances an eased duration-based tween toward `target`.
    fn prism_tween(&self, id: Id, target: f32, duration_secs: f32, easing: Easing) -> f32;

    /// Smoothly animates a `0.0..=1.0` hover progress value using [`SpringConfig::snappy`].
    fn prism_hover(&self, id: Id, hovered: bool) -> f32;
}

impl PrismAnimExt for Context {
    fn prism_spring(&self, id: Id, target: f32, config: SpringConfig) -> f32 {
        animate_spring(self, id, target, config)
    }

    fn prism_tween(&self, id: Id, target: f32, duration_secs: f32, easing: Easing) -> f32 {
        animate_tween(self, id, target, duration_secs, easing)
    }

    fn prism_hover(&self, id: Id, hovered: bool) -> f32 {
        animate_hover(self, id, hovered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easings_respect_endpoints() {
        for easing in [
            Easing::Linear,
            Easing::EaseOutCubic,
            Easing::EaseInOutCubic,
            Easing::EaseOutExpo,
            Easing::EaseOutBack,
        ] {
            assert!((easing.apply(0.0) - 0.0).abs() < 1e-4);
            assert!((easing.apply(1.0) - 1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn spring_converges_to_target() {
        let mut spring = SpringState::new(0.0);
        let cfg = SpringConfig::snappy();
        let mut moving = true;
        for _ in 0..180 {
            moving = spring.step(1.0, 1.0 / 60.0, cfg);
            if !moving {
                break;
            }
        }
        assert!(!moving);
        assert!((spring.value - 1.0).abs() < 1e-3);
    }
}
