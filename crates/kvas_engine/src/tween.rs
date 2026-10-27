//! Easing curves and script-free local-position animation.
//!
//! [`MoveTween`] runs after ordinary modifiers and before physics, making it
//! useful for kinematic lifts and animated scene objects. Avoid assigning multiple
//! independent position writers to one entity: the tween writes last.
//!
//! ```
//! use kvas_engine::{scene::Scene, runtime::Runtime, tween::{Ease, MoveTween}};
//! use kvas_engine::components::Transform;
//! use glam::Vec3;
//! let mut scene = Scene::default();
//! let entity = scene.spawn("lift");
//! let mut tween = MoveTween::to(Vec3::Y, 1.0).unwrap();
//! tween.ease = Ease::Linear;
//! scene.world.insert(entity, tween);
//! let mut runtime = Runtime::new(scene, None);
//! runtime.tick(0.1);
//! assert!((runtime.scene.world.get::<Transform>(entity).unwrap().position.y - 0.1).abs() < 1e-5);
//! ```

use crate::{
    choice_enum,
    components::{Component, Transform},
    reflect::Field,
};
use glam::Vec3;
use kvas_ecs::World;
use serde::{Deserialize, Serialize};

choice_enum!(
    /// Normalized interpolation curves. BackOut overshoots; BounceOut rebounds.
    Ease { Linear, SmoothStep, SmootherStep, QuadIn, QuadOut, QuadInOut, CubicIn, CubicOut, CubicInOut, SineIn, SineOut, SineInOut, BackOut, BounceOut } default SmoothStep
);

impl Ease {
    /// Samples a curve at time clamped to 0..1. Non-finite time selects zero.
    /// Endpoints return exactly zero/one; BackOut may exceed one in between.
    pub fn sample(self, time: f32) -> f32 {
        let t = if time.is_finite() { time.clamp(0.0, 1.0) } else { 0.0 };
        if t == 0.0 || t == 1.0 {
            return t;
        }
        match self {
            Self::Linear => t,
            Self::SmoothStep => t * t * (3.0 - 2.0 * t),
            Self::SmootherStep => t * t * t * (t * (t * 6.0 - 15.0) + 10.0),
            Self::QuadIn => t * t,
            Self::QuadOut => 1.0 - (1.0 - t).powi(2),
            Self::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) * 0.5
                }
            }
            Self::CubicIn => t * t * t,
            Self::CubicOut => 1.0 - (1.0 - t).powi(3),
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
                }
            }
            Self::SineIn => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
            Self::SineOut => (t * std::f32::consts::FRAC_PI_2).sin(),
            Self::SineInOut => -((std::f32::consts::PI * t).cos() - 1.0) * 0.5,
            Self::BackOut => {
                let t = t - 1.0;
                1.0 + 2.70158 * t.powi(3) + 1.70158 * t.powi(2)
            }
            Self::BounceOut => {
                let (t, offset) = if t < 1.0 / 2.75 {
                    (t, 0.0)
                } else if t < 2.0 / 2.75 {
                    (t - 1.5 / 2.75, 0.75)
                } else if t < 2.5 / 2.75 {
                    (t - 2.25 / 2.75, 0.9375)
                } else {
                    (t - 2.625 / 2.75, 0.984375)
                };
                7.5625 * t * t + offset
            }
        }
    }
}

choice_enum!(
    /// Once stops at the target; Repeat jumps to the origin; PingPong reverses each leg.
    TweenMode { Once, Repeat, PingPong } default Once
);

/// Local-position tween from the first observed pose to a parent-local target.
/// Runtime progress is omitted from serde; loaded/duplicated components start afresh.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MoveTween {
    /// Absolute target position in parent-local metres.
    pub target: Vec3,
    /// Duration of one leg in simulation seconds. Zero snaps once, in any mode.
    pub duration: f32,
    /// Initial delay in simulation seconds; negative values behave as zero.
    pub delay: f32,
    /// Interpolation curve for each leg.
    pub ease: Ease,
    /// Completion/repetition policy.
    pub mode: TweenMode,
    /// Whether the update system advances this component.
    pub playing: bool,
    #[serde(skip)]
    origin: Option<Vec3>,
    #[serde(skip)]
    elapsed: f64,
    #[serde(skip)]
    finished: bool,
}

impl Default for MoveTween {
    fn default() -> Self {
        Self {
            target: Vec3::Y,
            duration: 1.0,
            delay: 0.0,
            ease: Ease::SmoothStep,
            mode: TweenMode::Once,
            playing: true,
            origin: None,
            elapsed: 0.0,
            finished: false,
        }
    }
}

impl MoveTween {
    /// Creates a one-shot tween; rejects invalid targets and negative/non-finite durations.
    pub fn to(target: Vec3, duration: f32) -> Option<Self> {
        (target.is_finite() && duration.is_finite() && duration >= 0.0).then_some(Self { target, duration, ..Self::default() })
    }
    /// Whether a one-shot (including zero duration) has reached its target.
    pub fn is_finished(&self) -> bool {
        self.finished
    }
    /// Total simulated seconds since the current start, including initial delay.
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }
    /// Restarts the clock and resumes, keeping the captured original position.
    /// The next update may jump back toward the original position.
    pub fn restart(&mut self) {
        self.elapsed = 0.0;
        self.finished = false;
        self.playing = true;
    }
    /// Restarts from the position observed on the next update.
    pub fn restart_from_current(&mut self) {
        self.origin = None;
        self.restart();
    }
    /// Suspends progress without resetting it.
    pub fn pause(&mut self) {
        self.playing = false;
    }
    /// Resumes an unfinished tween. Completed one-shots require restart.
    pub fn resume(&mut self) {
        if !self.finished {
            self.playing = true;
        }
    }

    fn advance(&mut self, position: Vec3, dt: f32) -> Option<Vec3> {
        if !self.playing || self.finished {
            return None;
        }
        if !position.is_finite() || !self.target.is_finite() || !self.duration.is_finite() || self.duration < 0.0 || !self.delay.is_finite()
        {
            self.playing = false;
            return None;
        }
        let origin = *self.origin.get_or_insert(position);
        self.elapsed += dt as f64;
        let active = self.elapsed - self.delay.max(0.0) as f64;
        if active < 0.0 {
            return None;
        }
        if self.duration == 0.0 {
            self.finished = true;
            self.playing = false;
            return Some(self.target);
        }
        let phase = active / self.duration as f64;
        let progress = match self.mode {
            TweenMode::Once => {
                if phase >= 1.0 {
                    self.finished = true;
                    self.playing = false;
                }
                phase.min(1.0)
            }
            TweenMode::Repeat => phase.fract(),
            TweenMode::PingPong => {
                let cycle = phase.rem_euclid(2.0);
                if cycle <= 1.0 {
                    cycle
                } else {
                    2.0 - cycle
                }
            }
        } as f32;
        Some(origin.lerp(self.target, self.ease.sample(progress)))
    }
}

impl Component for MoveTween {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::vec3("target", &mut self.target),
            Field::f32("duration", &mut self.duration),
            Field::f32("delay", &mut self.delay),
            Field::choice("ease", &mut self.ease),
            Field::choice("mode", &mut self.mode),
            Field::bool("playing", &mut self.playing),
        ]
    }
}

/// Advances every position tween by a finite positive simulation delta.
/// Entities without a transform are ignored. Invalid tween data pauses the tween.
pub fn update(world: &mut World, dt: f32) {
    if !dt.is_finite() || dt <= 0.0 {
        return;
    }
    let positions: Vec<_> = world.iter2::<MoveTween, Transform>().map(|(e, _, t)| (e, t.position)).collect();
    for (entity, position) in positions {
        let target = world.get_mut::<MoveTween>(entity).and_then(|tween| tween.advance(position, dt));
        if let Some(target) = target {
            if let Some(transform) = world.get_mut::<Transform>(entity) {
                transform.position = target;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reflect::Choice;
    #[test]
    fn curves_have_exact_endpoints_and_finite_samples() {
        for index in 0..Ease::OPTIONS.len() {
            let ease = Ease::from_index(index);
            assert_eq!(ease.sample(-1.0), 0.0);
            assert_eq!(ease.sample(2.0), 1.0);
            assert_eq!(ease.sample(f32::NAN), 0.0);
            for i in 0..=100 {
                assert!(ease.sample(i as f32 / 100.0).is_finite());
            }
        }
    }
    #[test]
    fn delay_pause_restart_and_pingpong() {
        let mut tween = MoveTween::to(Vec3::X, 1.0).unwrap();
        tween.ease = Ease::Linear;
        tween.delay = 0.5;
        assert!(tween.advance(Vec3::ZERO, 0.25).is_none());
        assert_eq!(tween.advance(Vec3::ZERO, 0.75), Some(Vec3::X * 0.5));
        tween.pause();
        assert!(tween.advance(Vec3::ZERO, 0.25).is_none());
        tween.resume();
        assert_eq!(tween.advance(Vec3::ZERO, 0.5), Some(Vec3::X));
        assert!(tween.is_finished());
        tween.restart();
        tween.mode = TweenMode::PingPong;
        tween.delay = 0.0;
        assert_eq!(tween.advance(Vec3::ZERO, 1.5), Some(Vec3::X * 0.5));
        assert!(!tween.is_finished());
        assert_eq!(tween.advance(Vec3::ZERO, 0.5), Some(Vec3::ZERO));
        tween.restart_from_current();
        assert_eq!(tween.advance(Vec3::Y, 0.5), Some((Vec3::X + Vec3::Y) * 0.5));
    }
}
