//! Animation helpers for UI transitions.
//!
//! Provides easing functions and animation state management following
//! the Disney principle: "Does it move like it has weight and intent?"

use std::time::{Duration, Instant};

/// Easing function type for animation curves
pub type EasingFn = fn(f32) -> f32;

/// Animation state with configurable easing
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub start: Instant,
    pub duration: Duration,
    pub from: f32,
    pub to: f32,
    pub easing: EasingFn,
}

impl AnimationState {
    /// Create animation with default ease_in_out_quad easing
    pub fn new(from: f32, to: f32, duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
            from,
            to,
            easing: ease_in_out_quad,
        }
    }

    /// Create animation with custom easing function
    pub fn with_easing(from: f32, to: f32, duration: Duration, easing: EasingFn) -> Self {
        Self {
            start: Instant::now(),
            duration,
            from,
            to,
            easing,
        }
    }

    /// Create velocity-aware animation (duration scales with distance)
    /// Min duration prevents jank on small movements, max prevents sluggishness
    pub fn velocity_aware(from: f32, to: f32, pixels_per_sec: f32) -> Self {
        let distance = (to - from).abs();
        let raw_duration = distance / pixels_per_sec;
        // Clamp between 120ms and 350ms for natural feel
        let clamped = raw_duration.clamp(0.12, 0.35);
        Self::with_easing(from, to, Duration::from_secs_f32(clamped), ease_out_quint)
    }

    pub fn progress(&self) -> f32 {
        let elapsed = self.start.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32().max(0.0001);
        (elapsed / total).min(1.0)
    }

    pub fn value(&self) -> f32 {
        let t = (self.easing)(self.progress());
        self.from + (self.to - self.from) * t
    }

    pub fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Easing Functions
// ══════════════════════════════════════════════════════════════════════════════

/// Linear - no easing (rarely used, feels mechanical)
pub fn linear(t: f32) -> f32 {
    t
}

/// Ease out quad - smooth deceleration (good for enters)
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(2)
}

/// Ease in quad - smooth acceleration (good for exits)
pub fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Ease in-out quad - smooth both ends (default for panels)
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Ease out cubic - snappier deceleration
pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Ease out quint - very snappy start, gentle settle (premium feel)
pub fn ease_out_quint(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(5)
}

/// Ease out back - slight overshoot for micro-interactions
pub fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
}

/// Ease out elastic - bouncy settle (use sparingly for delight)
#[allow(dead_code)]
pub fn ease_out_elastic(t: f32) -> f32 {
    if t == 0.0 || t == 1.0 {
        return t;
    }
    const C4: f32 = std::f32::consts::TAU / 3.0;
    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0
}

// ══════════════════════════════════════════════════════════════════════════════
// Specialized Animation Types
// ══════════════════════════════════════════════════════════════════════════════

/// Toast animation state (combines slide + fade)
#[derive(Debug, Clone)]
pub struct ToastAnimation {
    pub start: Instant,
    pub duration: Duration,
    pub entering: bool, // true = entering, false = exiting
}

impl ToastAnimation {
    pub fn enter(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
            entering: true,
        }
    }

    pub fn exit(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
            entering: false,
        }
    }

    pub fn progress(&self) -> f32 {
        let elapsed = self.start.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32().max(0.0001);
        (elapsed / total).min(1.0)
    }

    pub fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }

    /// Returns (opacity, y_offset) for rendering
    pub fn values(&self) -> (f32, f32) {
        let t = self.progress();
        if self.entering {
            // Enter: slide up from 20px below, fade in
            let eased = ease_out_quint(t);
            let opacity = eased;
            let y_offset = 20.0 * (1.0 - eased);
            (opacity, y_offset)
        } else {
            // Exit: fade out quickly, slight slide down
            let eased = ease_in_quad(t);
            let opacity = 1.0 - eased;
            let y_offset = 8.0 * eased;
            (opacity, y_offset)
        }
    }
}

/// Modal fade animation
#[derive(Debug, Clone)]
pub struct ModalAnimation {
    pub start: Instant,
    pub duration: Duration,
    pub opening: bool,
}

impl ModalAnimation {
    pub fn open(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
            opening: true,
        }
    }

    pub fn close(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
            opening: false,
        }
    }

    pub fn progress(&self) -> f32 {
        let elapsed = self.start.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32().max(0.0001);
        (elapsed / total).min(1.0)
    }

    pub fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }

    /// Returns (overlay_opacity, content_scale, content_opacity)
    pub fn values(&self) -> (f32, f32, f32) {
        let t = self.progress();
        if self.opening {
            let eased = ease_out_quint(t);
            let overlay_opacity = 0.85 * eased;
            let content_scale = 0.95 + 0.05 * eased; // 95% -> 100%
            let content_opacity = eased;
            (overlay_opacity, content_scale, content_opacity)
        } else {
            let eased = ease_in_quad(t);
            let overlay_opacity = 0.85 * (1.0 - eased);
            let content_scale = 1.0 - 0.03 * eased; // 100% -> 97%
            let content_opacity = 1.0 - eased;
            (overlay_opacity, content_scale, content_opacity)
        }
    }
}

/// Button press micro-animation
#[derive(Debug, Clone)]
pub struct PressAnimation {
    pub start: Instant,
    pub duration: Duration,
    pub pressing: bool,
}

impl PressAnimation {
    pub fn press() -> Self {
        Self {
            start: Instant::now(),
            duration: Duration::from_millis(80),
            pressing: true,
        }
    }

    pub fn release() -> Self {
        Self {
            start: Instant::now(),
            duration: Duration::from_millis(150),
            pressing: false,
        }
    }

    pub fn progress(&self) -> f32 {
        let elapsed = self.start.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32().max(0.0001);
        (elapsed / total).min(1.0)
    }

    pub fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }

    /// Returns scale factor (1.0 = normal, <1.0 = pressed)
    pub fn scale(&self) -> f32 {
        let t = self.progress();
        if self.pressing {
            // Quick scale down
            1.0 - 0.02 * ease_out_quad(t)
        } else {
            // Bounce back with slight overshoot
            0.98 + 0.02 * ease_out_back(t)
        }
    }
}
