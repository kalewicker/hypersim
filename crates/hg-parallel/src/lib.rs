//! hg-parallel: parallel-matching support (Rayon), used by hg-engine
//! behind its `parallel` feature. Deliberately thin: determinism comes
//! from the engine's canonical candidate sort, not from collection order.

pub use rayon;
