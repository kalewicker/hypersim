//! Commands the GUI/CLI sends to a controller.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SimCommand {
    Start,
    Pause,
    Resume,
    Step(u64),
    Reset,
    SetSpeed(f32),
    Quit,
}
