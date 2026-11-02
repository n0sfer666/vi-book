use v_ide_editor::autocmd::{DataBuilder, DataNode};

#[test]
fn data_building_rejects_unknown_edges_without_losing_the_valid_prefix() {
    let mut foreign = DataBuilder::new();
    foreign.push(DataNode::Null).unwrap();
    let unknown = foreign.push(DataNode::Null).unwrap();
    let mut builder = DataBuilder::new();
    let root = builder.push(DataNode::Boolean(true)).unwrap();
    assert!(builder.push(DataNode::Array(vec![unknown])).is_err());
    assert!(
        builder
            .push(DataNode::Dictionary(vec![(b"key".to_vec(), unknown)]))
            .is_err()
    );
    let data = builder.finish(root).unwrap();
    assert_eq!(data.root(), &DataNode::Boolean(true));
    assert!(data.get(unknown).is_none());
    assert!(DataBuilder::new().finish(root).is_err());
}

#[test]
fn deep_shared_data_is_cloned_and_dropped_without_recursive_ownership() {
    let mut builder = DataBuilder::new();
    let leaf = builder.push(DataNode::String(vec![b'a', 0, 255])).unwrap();
    let mut root = leaf;
    for _ in 0..50000 {
        root = builder.push(DataNode::Array(vec![root, leaf])).unwrap();
    }
    let data = builder.finish(root).unwrap();
    let retained = data.clone();
    drop(data);
    let mut current = retained.root();
    for _ in 0..50000 {
        let DataNode::Array(children) = current else {
            panic!("missing nesting")
        };
        assert_eq!(children.len(), 2);
        assert_eq!(
            retained.get(children[1]),
            Some(&DataNode::String(vec![b'a', 0, 255]))
        );
        current = retained.get(children[0]).unwrap();
    }
    assert_eq!(current, &DataNode::String(vec![b'a', 0, 255]));
}
