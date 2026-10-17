//! `SimProgress` — atomics, not RwLock (v4 §9.2).
//!
//! `Ordering::Relaxed` throughout, deliberately, and ONLY here: this is
//! purely advisory display data; a one-store-stale read is fine. This is
//! explicitly not extended to MetricsHistory/EvalState/knowledge-graph
//! state, which keep RwLock consistency because that data is
//! scientifically load-bearing.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};

#[derive(Debug)]
pub struct SimProgress {
    step: AtomicU64,
    max_steps: AtomicU64,
    run_state: AtomicU8, // RunState as a small integer discriminant
    sim_hz_bits: AtomicU32, // f32 via to_bits()/from_bits()
}

impl SimProgress {
    pub fn new(max_steps: u64) -> Self {
        Self {
            step: AtomicU64::new(0),
            max_steps: AtomicU64::new(max_steps),
            run_state: AtomicU8::new(0),
            sim_hz_bits: AtomicU32::new(0f32.to_bits()),
        }
    }

    pub fn step(&self) -> u64 {
        self.step.load(Ordering::Relaxed)
    }
    pub fn set_step(&self, v: u64) {
        self.step.store(v, Ordering::Relaxed);
    }
    pub fn max_steps(&self) -> u64 {
        self.max_steps.load(Ordering::Relaxed)
    }
    pub fn set_max_steps(&self, v: u64) {
        self.max_steps.store(v, Ordering::Relaxed);
    }
    pub fn run_state_u8(&self) -> u8 {
        self.run_state.load(Ordering::Relaxed)
    }
    pub fn set_run_state_u8(&self, v: u8) {
        self.run_state.store(v, Ordering::Relaxed);
    }
    pub fn sim_hz(&self) -> f32 {
        f32::from_bits(self.sim_hz_bits.load(Ordering::Relaxed))
    }
    pub fn set_sim_hz(&self, v: f32) {
        self.sim_hz_bits.store(v.to_bits(), Ordering::Relaxed);
    }
}

impl Default for SimProgress {
    fn default() -> Self {
        Self::new(0)
    }
}
