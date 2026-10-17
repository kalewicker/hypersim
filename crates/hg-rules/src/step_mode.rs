//! Step mode enumeration

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StepMode {
    #[default]
    MaxParallel,
    SequentialRandom,
    SequentialWeighted,
}