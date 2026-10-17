//! Headless tests for the canvas camera and force layout — the parts of
//! the GUI-improvements work that don't need a window.

use egui::{Pos2, Rect, Vec2};
use hg_gui::camera::Camera2D;
use hg_gui::layout::ForceLayout;
use hg_app_api::{CompactEdge, CompactGraphSnapshot, CompactNode, EdgeId, Label, NodeId};
use slotmap::KeyData;

fn node_id(i: u64) -> NodeId {
    // Distinct synthetic keys; only identity matters to the layout.
    NodeId::from(KeyData::from_ffi((1 << 32) | i))
}

fn edge_id(i: u64) -> EdgeId {
    EdgeId::from(KeyData::from_ffi((1 << 32) | i))
}

fn snapshot(n_nodes: u64, edges: &[Vec<u64>]) -> CompactGraphSnapshot {
    CompactGraphSnapshot {
        step: 0,
        nodes: (0..n_nodes)
            .map(|i| CompactNode {
                id: node_id(i),
                label: Label(0),
                generation: 0,
                energy: 0.0,
            })
            .collect(),
        edges: edges
            .iter()
            .enumerate()
            .map(|(i, members)| CompactEdge {
                id: edge_id(i as u64),
                label: Label(0),
                nodes: members.iter().map(|&m| node_id(m)).collect(),
            })
            .collect(),
    }
}

#[test]
fn camera_round_trip_world_screen() {
    let cam = Camera2D {
        center: Vec2::new(37.0, -12.0),
        zoom: 2.5,
    };
    let viewport = Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(800.0, 600.0));
    let world = Vec2::new(-3.0, 91.0);
    let screen = cam.world_to_screen(world, viewport);
    let back = cam.screen_to_world(screen, viewport);
    assert!((back - world).length() < 1e-3, "{back:?} != {world:?}");
}

#[test]
fn camera_fit_frames_bounds_within_viewport() {
    let mut cam = Camera2D::default();
    let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 700.0));
    let bounds = Rect::from_min_max(Pos2::new(-400.0, 200.0), Pos2::new(900.0, 1600.0));
    cam.fit_to_bounds(bounds, viewport);
    for corner in [
        bounds.min,
        bounds.max,
        Pos2::new(bounds.min.x, bounds.max.y),
        Pos2::new(bounds.max.x, bounds.min.y),
    ] {
        let s = cam.world_to_screen(corner.to_vec2(), viewport);
        assert!(viewport.contains(s), "corner {corner:?} -> {s:?} outside viewport");
    }
}

#[test]
fn camera_fit_degenerate_bounds_is_safe() {
    let mut cam = Camera2D::default();
    let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
    cam.fit_to_bounds(Rect::from_min_max(Pos2::new(5.0, 5.0), Pos2::new(5.0, 5.0)), viewport);
    assert!(cam.zoom.is_finite() && cam.zoom > 0.0);
    assert_eq!(cam.center, Vec2::new(5.0, 5.0));
}

#[test]
fn layout_positions_are_finite_and_distinct() {
    // Triangle with a center node fanned to all corners.
    let graph = snapshot(4, &[vec![0, 1], vec![1, 2], vec![0, 2], vec![3, 0], vec![3, 1], vec![3, 2]]);
    let mut layout = ForceLayout::default();
    layout.sync(&graph);
    for _ in 0..200 {
        layout.step(&graph);
    }
    let ps: Vec<Vec2> = graph.nodes.iter().map(|n| layout.position(n.id).unwrap()).collect();
    for p in &ps {
        assert!(p.is_finite());
    }
    for i in 0..ps.len() {
        for j in (i + 1)..ps.len() {
            assert!((ps[i] - ps[j]).length() > 1.0, "nodes {i},{j} collapsed");
        }
    }
    let b = layout.bounds().unwrap();
    assert!(b.width() > 1.0 && b.height() > 1.0, "layout degenerate: {b:?}");
}

/// The bare-ring regression: in a wheel graph (hub connected to every rim
/// node), the hub must settle visibly INSIDE the rim — the old index-ring
/// pseudo-layout placed it on the circle with everything else.
#[test]
fn layout_places_wheel_hub_inside_rim() {
    const RIM: u64 = 12;
    let mut edges: Vec<Vec<u64>> = (0..RIM).map(|i| vec![1 + i, 1 + (i + 1) % RIM]).collect();
    for i in 0..RIM {
        edges.push(vec![0, 1 + i]); // hub spokes
    }
    let graph = snapshot(1 + RIM, &edges);
    let mut layout = ForceLayout::default();
    layout.sync(&graph);
    for _ in 0..400 {
        layout.step(&graph);
    }
    let hub = layout.position(node_id(0)).unwrap();
    let rim: Vec<Vec2> = (1..=RIM).map(|i| layout.position(node_id(i)).unwrap()).collect();
    let centroid = rim.iter().fold(Vec2::ZERO, |a, &p| a + p) / RIM as f32;
    let mean_r = rim.iter().map(|p| (*p - centroid).length()).sum::<f32>() / RIM as f32;
    let hub_r = (hub - centroid).length();
    assert!(
        hub_r < mean_r * 0.5,
        "hub not interior: hub_r={hub_r:.1} mean_rim_r={mean_r:.1}"
    );
}

/// New nodes appearing mid-run (the subdivision case) seed near their
/// placed neighbors and get separated by relaxation, not stacked.
#[test]
fn layout_seeds_new_nodes_near_neighbors() {
    let graph = snapshot(3, &[vec![0, 1], vec![1, 2], vec![0, 2]]);
    let mut layout = ForceLayout::default();
    layout.sync(&graph);
    for _ in 0..100 {
        layout.step(&graph);
    }
    let spread_before = layout.bounds().unwrap();

    // Subdivide: node 3 arrives connected to all of the triangle.
    let graph2 = snapshot(4, &[vec![0, 1], vec![1, 2], vec![0, 2], vec![3, 0], vec![3, 1], vec![3, 2]]);
    layout.sync(&graph2);
    let seeded = layout.position(node_id(3)).unwrap();
    assert!(
        spread_before.expand(60.0).contains(seeded.to_pos2()),
        "new node seeded far from its neighborhood: {seeded:?} vs {spread_before:?}"
    );
    layout.step(&graph2);
    for i in 0..3 {
        let p = layout.position(node_id(i)).unwrap();
        assert!((p - seeded).length() > 0.5, "seeded node stacked on node {i}");
    }
}

/// Dead nodes are dropped on sync so hover/bounds never reference them.
#[test]
fn layout_drops_dead_nodes() {
    let graph = snapshot(3, &[vec![0, 1], vec![1, 2]]);
    let mut layout = ForceLayout::default();
    layout.sync(&graph);
    assert!(layout.position(node_id(2)).is_some());
    let shrunk = snapshot(2, &[vec![0, 1]]);
    layout.sync(&shrunk);
    assert!(layout.position(node_id(2)).is_none());
}
