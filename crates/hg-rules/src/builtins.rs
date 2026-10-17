//! Builtin rule sets — the fixture rule files embedded at compile time.

use crate::{parse_rules_toml, ParseError, Rule};

pub const BUILTIN_NAMES: [&str; 5] = [
    "triangle_subdivide",
    "binary_edge",
    "tetrahedral",
    "glider",
    "growth_treadmill",
];

/// Load a builtin rule set by name.
pub fn builtin_rules(name: &str) -> Result<Vec<Rule>, ParseError> {
    let src = match name {
        "triangle_subdivide" => include_str!("../../../rules/triangle_subdivide.toml"),
        "binary_edge" => include_str!("../../../rules/binary_edge.toml"),
        "tetrahedral" => include_str!("../../../rules/tetrahedral.toml"),
        "glider" => include_str!("../../../rules/glider.toml"),
        "growth_treadmill" => include_str!("../../../rules/growth_treadmill.toml"),
        other => {
            return Err(ParseError::InvalidGuard(format!(
                "unknown builtin rule set '{other}'"
            )))
        }
    };
    parse_rules_toml(src)
}
