use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};

#[test]
fn explicit_allocator_history_preserves_reuse_without_changing_legacy_wire() {
    for isolated in [false, true] {
        let mut original = if isolated {
            HypergraphStore::new_metadata_isolated()
        } else {
            HypergraphStore::new()
        };
        let nodes: Vec<_> = (0..8)
            .map(|i| original.add_node(NodeData::new(Label(i), 0)))
            .collect();
        let edges: Vec<_> = (0..5)
            .map(|i| {
                original.add_edge(HyperEdge::new(
                    Label(0),
                    [nodes[i], nodes[i + 1]].into_iter().collect(),
                ))
            })
            .collect();
        for i in [3, 0, 2] {
            original.remove_edge(edges[i]);
        }
        for i in [5, 1, 7] {
            original.remove_node(nodes[i]);
        }
        let bytes = bincode::serialize(&original).unwrap();
        let mut restored: HypergraphStore = bincode::deserialize(&bytes).unwrap();
        restored
            .restore_allocation_state(original.allocation_state())
            .unwrap();
        assert_eq!(bytes, bincode::serialize(&restored).unwrap());
        for i in 0..10 {
            assert_eq!(
                original.add_node(NodeData::new(Label(i), 0)),
                restored.add_node(NodeData::new(Label(i), 0))
            );
            assert_eq!(
                original.add_edge(HyperEdge::new(
                    Label(0),
                    [nodes[0], nodes[2]].into_iter().collect()
                )),
                restored.add_edge(HyperEdge::new(
                    Label(0),
                    [nodes[0], nodes[2]].into_iter().collect()
                ))
            );
        }
    }
}
#[test]
fn invalid_inventory_is_rejected_before_either_allocator_changes() {
    let mut store = HypergraphStore::new();
    let n = store.add_node(NodeData::new(Label(0), 0));
    store.remove_node(n);
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Arena {
        free: [Vec<u32>; 2],
        next: [u64; 2],
        isolated: bool,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    struct State {
        nodes: Arena,
        edges: Arena,
    }
    let before = bincode::serialize(&store.allocation_state()).unwrap();
    let mut wire: State = bincode::deserialize(&before).unwrap();
    wire.edges.free[0] = vec![999];
    let invalid = bincode::deserialize(&bincode::serialize(&wire).unwrap()).unwrap();
    assert!(store.restore_allocation_state(invalid).is_err());
    assert_eq!(
        before,
        bincode::serialize(&store.allocation_state()).unwrap()
    );
}
