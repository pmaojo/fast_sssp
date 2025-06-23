use fast_sssp::data_structures::BlockList;
use ordered_float::OrderedFloat;

#[test]
fn test_block_list_insert_and_pull() {
    let mut bl: BlockList<usize, OrderedFloat<f64>> = BlockList::new(2, OrderedFloat(f64::INFINITY));
    bl.insert(1, OrderedFloat(10.0));
    bl.insert(2, OrderedFloat(5.0));
    // update with smaller value
    bl.insert(1, OrderedFloat(8.0));
    assert_eq!(bl.get(&1), Some(OrderedFloat(8.0)));
    assert_eq!(bl.len(), 2);

    let (keys, next_bound) = bl.pull(2);
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&1));
    assert!(keys.contains(&2));
    assert!(next_bound > OrderedFloat(8.0));
}

#[test]
fn test_block_list_batch_prepend() {
    let mut bl: BlockList<usize, OrderedFloat<f64>> = BlockList::new(2, OrderedFloat(f64::INFINITY));
    bl.insert(1, OrderedFloat(10.0));
    bl.insert(2, OrderedFloat(20.0));

    bl.batch_prepend(vec![(3, OrderedFloat(2.0)), (4, OrderedFloat(1.0))]);
    assert_eq!(bl.len(), 4);

    let (first_keys, _) = bl.pull(2);
    assert!(first_keys.contains(&3));
    assert!(first_keys.contains(&4));
}
