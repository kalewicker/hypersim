//! Graph generators for initial conditions

use crate::graph::{HypergraphStore, HyperEdge, NodeData, Label, NodeId};
use smallvec::SmallVec;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

/// Determine label range for random generation
fn label_range(max_label: u32) -> u32 {
    if max_label == 0 {
        1
    } else {
        max_label + 1
    }
}

/// Generate a pure crystal (regular grid)
pub fn generate_crystal(
    store: &mut HypergraphStore,
    width: usize,
    height: usize,
    arity: usize,
    periodic: bool,
    _seed: u64,
) {
    // Create grid nodes
    let mut node_ids = vec![vec![NodeId::default(); width]; height];
    for (_y, row) in node_ids.iter_mut().enumerate().take(height) {
        for (_x, cell) in row.iter_mut().enumerate().take(width) {
            let id = store.add_node(NodeData::new(Label(0), store.step()));
            *cell = id;
        }
    }
    
    // Connect neighbors based on arity
    for (y, row) in node_ids.iter().enumerate().take(height) {
        for (x, &center) in row.iter().enumerate().take(width) {
            
            match arity {
                2 => {
                    // von Neumann neighborhood (4 neighbors)
                    let neighbors = if periodic {
                        vec![
                            (x, (y + 1) % height),
                            (x, (y + height - 1) % height),
                            ((x + 1) % width, y),
                            ((x + width - 1) % width, y),
                        ]
                    } else {
                        let mut n = Vec::new();
                        if y + 1 < height {
                            n.push((x, y + 1));
                        }
                        if y > 0 {
                            n.push((x, y - 1));
                        }
                        if x + 1 < width {
                            n.push((x + 1, y));
                        }
                        if x > 0 {
                            n.push((x - 1, y));
                        }
                        n
                    };
                    
                    for (nx, ny) in neighbors {
                        let neighbor = node_ids[ny][nx];
                        let edge = HyperEdge::new(Label(0), SmallVec::from_slice(&[center, neighbor]));
                        store.add_edge(edge);
                    }
                }
                3 => {
                    // Triangle lattice: each node connects to 6 neighbors forming triangles
                    // Simplified: connect to right, down-right, down
                    let neighbors = if periodic {
                        vec![
                            ((x + 1) % width, y),
                            (x, (y + 1) % height),
                            ((x + 1) % width, (y + 1) % height),
                        ]
                    } else {
                        let mut n = Vec::new();
                        if x + 1 < width {
                            n.push((x + 1, y));
                        }
                        if y + 1 < height {
                            n.push((x, y + 1));
                        }
                        if x + 1 < width && y + 1 < height {
                            n.push((x + 1, y + 1));
                        }
                        n
                    };
                    
                    for (nx, ny) in neighbors {
                        let neighbor = node_ids[ny][nx];
                        let edge = HyperEdge::new(Label(0), SmallVec::from_slice(&[center, neighbor]));
                        store.add_edge(edge);
                    }
                }
                _ => {
                    // Default: connect to immediate neighbors
                    let neighbors = if periodic {
                        vec![((x + 1) % width, y), (x, (y + 1) % height)]
                    } else {
                        let mut n = Vec::new();
                        if x + 1 < width {
                            n.push((x + 1, y));
                        }
                        if y + 1 < height {
                            n.push((x, y + 1));
                        }
                        n
                    };
                    
                    for (nx, ny) in neighbors {
                        let neighbor = node_ids[ny][nx];
                        let edge = HyperEdge::new(Label(0), SmallVec::from_slice(&[center, neighbor]));
                        store.add_edge(edge);
                    }
                }
            }
        }
    }
}

/// Generate crystal with random defects
pub fn generate_crystal_with_defects(
    store: &mut HypergraphStore,
    width: usize,
    height: usize,
    arity: usize,
    periodic: bool,
    defect_density: f64,
    seed: u64,
) {
    generate_crystal(store, width, height, arity, periodic, seed);
    
    let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(1));
    let total_nodes = width * height;
    let defect_count = (total_nodes as f64 * defect_density) as usize;
    
    for _ in 0..defect_count {
        let _idx = rng.gen_range(0..total_nodes);
        
        // Collect node IDs first to avoid borrow issues
        let node_ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
        if node_ids.is_empty() {
            continue;
        }
        let node_id = node_ids[rng.gen_range(0..node_ids.len())];
        
        // Remove some edges and add random ones
        let incident: Vec<_> = store.edges_incident_to(node_id).to_vec();
        for edge_id in incident.iter().take(2) {
            store.remove_edge(*edge_id);
        }
        
        // Add random edges
        for _ in 0..2 {
            if node_ids.len() > 1 {
                let target_id = node_ids[rng.gen_range(0..node_ids.len())];
                if target_id != node_id {
                    let edge = HyperEdge::new(Label(0), SmallVec::from_slice(&[node_id, target_id]));
                    store.add_edge(edge);
                }
            }
        }
    }
}

/// Generate white noise (random graph)
pub fn generate_white_noise(
    store: &mut HypergraphStore,
    node_count: usize,
    edge_count: usize,
    max_arity: usize,
    label_range: u32,
    seed: u64,
) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    
    // Create nodes with random labels
    for _ in 0..node_count {
        let label = Label(rng.gen_range(0..label_range));
        store.add_node(NodeData::new(label, store.step()));
    }
    
    // Create random edges
    let node_ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    for _ in 0..edge_count {
        if node_ids.len() < 2 {
            break;
        }
        let arity = rng.gen_range(2..=max_arity.min(node_ids.len()));
        let mut nodes = SmallVec::new();
        for _ in 0..arity {
            nodes.push(node_ids[rng.gen_range(0..node_ids.len())]);
        }
        let label = Label(rng.gen_range(0..label_range));
        store.add_edge(HyperEdge::new(label, nodes));
    }
}

/// Generate sparse soup (low density random graph)
pub fn generate_sparse_soup(
    store: &mut HypergraphStore,
    node_count: usize,
    avg_degree: f64,
    max_arity: usize,
    label_range: u32,
    seed: u64,
) {
    let edge_count = ((node_count as f64 * avg_degree) / 2.0) as usize;
    generate_white_noise(store, node_count, edge_count, max_arity, label_range, seed);
}

/// Generate minimal seed (single hyperedge)
pub fn generate_minimal_seed(store: &mut HypergraphStore, arity: usize, label: u32, seed: u64) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    
    let mut nodes = SmallVec::new();
    for _ in 0..arity {
        let node_label = Label(rng.gen_range(0..label_range(label)));
        nodes.push(store.add_node(NodeData::new(node_label, store.step())));
    }
    
    store.add_edge(HyperEdge::new(Label(label), nodes));
}

/// Generate empty graph
pub fn generate_empty(_store: &mut HypergraphStore) {
    // Nothing to do - store is already empty
}
/// Generate a glider track: a periodic ring of `n` nodes tiled by `n`
/// overlapping 3-edges `{i, i+1, i+2}` (mod n), all label 0 except node
/// 0 (the mark, label 1). Every position on the ring offers the glider
/// LHS a `{1, 0, 0}` 3-edge, so a conservative mark-transfer rule can
/// walk it indefinitely. Requires `n >= 5` (below that the overlapping
/// triples degenerate); smaller n falls back to the minimum.
pub fn generate_glider_ring(store: &mut HypergraphStore, n: usize) {
    let n = n.max(5);
    let ids: Vec<NodeId> = (0..n)
        .map(|i| {
            let label = Label(u32::from(i == 0));
            store.add_node(NodeData::new(label, store.step()))
        })
        .collect();
    for i in 0..n {
        store.add_edge(HyperEdge::new(
            Label(0),
            SmallVec::from_slice(&[ids[i], ids[(i + 1) % n], ids[(i + 2) % n]]),
        ));
    }
}
