//! A-track binding rule 4 — **firewall: zero biology.**
//!
//! This is bare-substrate characterisation. If a future edit pulls `hg-alife`
//! (or an `alife` feature) into this crate's dependency graph, this test is the
//! thing that should stop it, rather than a reviewer noticing.
//!
//! The check covers this crate's manifest AND the manifests of all four crates
//! it depends on, so a transitively introduced dependency fails here too.

const SELF: &str = include_str!("../Cargo.toml");
const CORE: &str = include_str!("../../hg-core/Cargo.toml");
const RULES: &str = include_str!("../../hg-rules/Cargo.toml");
const ENGINE: &str = include_str!("../../hg-engine/Cargo.toml");
const OBSERVE: &str = include_str!("../../hg-observe/Cargo.toml");

/// Strip comment lines: this crate's own manifest names `hg-alife` in the
/// comment that explains why it is absent, and that comment must not be what
/// the guard trips on — a guard that fires on its own documentation is a guard
/// that gets deleted.
fn code_of(manifest: &str) -> String {
    manifest
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn no_biology_in_the_alphabet_track_dependency_graph() {
    for (name, manifest) in [
        ("hg-alphabet", SELF),
        ("hg-core", CORE),
        ("hg-rules", RULES),
        ("hg-engine", ENGINE),
        ("hg-observe", OBSERVE),
    ] {
        let code = code_of(manifest);
        assert!(
            !code.contains("hg-alife"),
            "FIREWALL BREACH: {name} declares a dependency on hg-alife; the A-track is \
             bare-substrate characterisation and must run with zero biology"
        );
        assert!(
            !code.contains("alife"),
            "FIREWALL BREACH: {name} mentions an `alife` feature or dependency"
        );
    }
}

/// The firing test for the guard above (binding rule 6): the same predicate,
/// applied to a manifest that DOES declare the dependency, must reject. Without
/// this, a guard that silently stopped reading manifests would still pass.
#[test]
fn firewall_guard_fires_on_a_breaching_manifest() {
    let breaching = "[dependencies]\nhg-core = { path = \"../hg-core\" }\nhg-alife = { path = \"../hg-alife\" }\n";
    assert!(
        code_of(breaching).contains("hg-alife"),
        "the firewall predicate failed to detect a manifest that plainly breaches it"
    );
    // And the comment-stripping must not be a loophole in the other direction:
    // a commented-out dependency is genuinely not a dependency.
    let commented = "[dependencies]\n# hg-alife = { path = \"../hg-alife\" }\n";
    assert!(!code_of(commented).contains("hg-alife"));
}
