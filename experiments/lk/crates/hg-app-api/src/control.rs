//! Cooperative run control: the live runner polls this each step, so
//! stop/pause take effect at a step boundary with proper manifest
//! transitions — no killed-mid-step lies in manifest.json. The speed
//! throttle lives here too: a target steps-per-second the runner paces
//! itself against (non-finite or <= 0 means uncapped, the default).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[derive(Debug)]
pub struct RunControl {
    stop: AtomicBool,
    pause: AtomicBool,
    /// f32 bits of the target steps/sec; INFINITY = uncapped.
    speed_bits: AtomicU32,
}

impl Default for RunControl {
    fn default() -> Self {
        Self {
            stop: AtomicBool::new(false),
            pause: AtomicBool::new(false),
            speed_bits: AtomicU32::new(f32::INFINITY.to_bits()),
        }
    }
}

impl RunControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
    pub fn stop_requested(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }
    pub fn request_pause(&self) {
        self.pause.store(true, Ordering::Relaxed);
    }
    pub fn request_resume(&self) {
        self.pause.store(false, Ordering::Relaxed);
    }
    pub fn pause_requested(&self) -> bool {
        self.pause.load(Ordering::Relaxed)
    }

    /// Target steps/sec; pass a non-finite or non-positive value to
    /// uncap. Takes effect at the next step boundary.
    pub fn set_speed_steps_per_sec(&self, v: f32) {
        let v = if v.is_finite() && v > 0.0 { v } else { f32::INFINITY };
        self.speed_bits.store(v.to_bits(), Ordering::Relaxed);
    }
    /// Current target steps/sec (INFINITY = uncapped).
    pub fn speed_steps_per_sec(&self) -> f32 {
        f32::from_bits(self.speed_bits.load(Ordering::Relaxed))
    }
}
