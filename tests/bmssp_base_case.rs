use fast_sssp::algorithm::bmssp::BMSSP;
use fast_sssp::graph::{DirectedGraph, MutableGraph, Graph};
use ordered_float::OrderedFloat;

#[test]
fn test_bmssp_base_case() {
    let mut g: DirectedGraph<OrderedFloat<f64>> = DirectedGraph::new();
    for _ in 0..5 { g.add_vertex(); }
    g.add_edge(0,1,OrderedFloat(1.0));
    g.add_edge(1,2,OrderedFloat(1.0));
    g.add_edge(0,2,OrderedFloat(3.0));
    g.add_edge(2,3,OrderedFloat(1.0));
    g.add_edge(1,3,OrderedFloat(4.0));
    g.add_edge(3,4,OrderedFloat(1.0));
    g.add_edge(0,4,OrderedFloat(10.0));

    let n = g.vertex_count();
    let mut dist = vec![OrderedFloat(f64::INFINITY); n];
    let mut pred = vec![None; n];
    dist[0] = OrderedFloat(0.0);

    let bmssp = BMSSP::<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>::new(n);
    let result = bmssp.execute(&g, 0, OrderedFloat(f64::INFINITY), &[0], &mut dist, &mut pred).unwrap();

    assert_eq!(result.new_bound, OrderedFloat(2.0));
    assert_eq!(result.vertices.len(), 2);
    assert!(result.vertices.contains(&0));
    assert!(result.vertices.contains(&1));
}
