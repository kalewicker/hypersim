//! S0.1 — cross-process WL determinism, permanently.
//!
//! A same-process rerun cannot detect a RandomState-seeded hasher (it seeds
//! once per process), so this test re-execs ITSELF as two separate OS processes
//! and asserts the WL digests match. Each child writes its digests to a file
//! named by `WL_PROBE_OUT`; the parent (env unset) spawns two children and
//! diffs the files. If the WL hasher ever regresses to a non-deterministic
//! seed, the two files differ and this fails.

use hg_core::graph::{HyperEdge, HypergraphStore, Label, NodeData};
use hg_core::hash::{compute_wl_class_map, compute_wl_hash};
use hg_core::property::PropertyValue;
use smallvec::SmallVec;
use std::process::Command;

fn fixture() -> HypergraphStore {
    let mut s = HypergraphStore::new();
    let ids: Vec<_> = (0..12)
        .map(|i| {
            let n = s.add_node(NodeData::new(Label((i % 3) as u32), 0));
            let d = s.get_node_mut(n).unwrap();
            d.energy = (i as f32) * 0.25;
            d.set_property(1, PropertyValue::Float(i as f64));
            d.set_property(11, PropertyValue::Float(0.5 + i as f64));
            d.set_property(20, PropertyValue::Bool(i % 2 == 0));
            n
        })
        .collect();
    for w in ids.windows(2) {
        s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[w[0], w[1]])));
    }
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[ids[11], ids[0]])));
    s.add_edge(HyperEdge::new(Label(1), SmallVec::from_slice(&[ids[0], ids[4], ids[8]])));
    s.add_edge(HyperEdge::new(Label(1), SmallVec::from_slice(&[ids[1], ids[5], ids[9]])));
    s
}

fn fold(hashes: &mut [u64]) -> u64 {
    hashes.sort_unstable();
    hashes
        .iter()
        .fold(0xcbf29ce484222325u64, |acc, &h| (acc ^ h).wrapping_mul(0x100000001b3))
}

fn digests() -> String {
    let s = fixture();
    let mut out = String::new();
    for radius in [1u8, 2, 3] {
        let mut per_node: Vec<u64> =
            s.nodes().map(|(id, _)| compute_wl_hash(&s, id, radius).0).collect();
        let mut map: Vec<u64> =
            compute_wl_class_map(&s, radius).into_values().map(|w| w.0).collect();
        out.push_str(&format!(
            "r={radius} node={:#018x} map={:#018x}\n",
            fold(&mut per_node),
            fold(&mut map)
        ));
    }
    out
}

#[test]
fn cross_process_wl_is_deterministic() {
    // Child leg: write digests to the requested file and exit before the
    // harness can run anything else.
    if let Ok(path) = std::env::var("WL_PROBE_OUT") {
        std::fs::write(&path, digests()).expect("child writes digest file");
        std::process::exit(0);
    }

    let exe = std::env::current_exe().expect("current test exe");
    let base = std::env::temp_dir();
    let pid = std::process::id();
    let run_child = |tag: &str| -> String {
        let out = base.join(format!("hypersim_wl_xproc_{pid}_{tag}.txt"));
        let _ = std::fs::remove_file(&out);
        let status = Command::new(&exe)
            .arg("--exact")
            .arg("cross_process_wl_is_deterministic")
            .env("WL_PROBE_OUT", &out)
            .status()
            .expect("spawn child");
        assert!(status.success(), "child {tag} exited non-zero");
        std::fs::read_to_string(&out).expect("read child digest")
    };

    let a = run_child("a");
    let b = run_child("b");
    assert!(!a.trim().is_empty(), "child produced no digests");
    assert_eq!(
        a, b,
        "WL digests differ across processes — the hasher is not process-stable \
(a RandomState-seeded hasher leaked back in); every cross-run WL claim is invalid"
    );
}
