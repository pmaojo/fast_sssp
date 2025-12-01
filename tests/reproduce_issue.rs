use fast_sssp::graph::DirectedGraph;
use fast_sssp::graph::{Graph, MutableGraph};

#[test]
fn test_vertex_removal_behavior() {
    let mut graph = DirectedGraph::<f64>::new();
    let v0 = graph.add_vertex();

    assert!(graph.has_vertex(v0));

    // Remove v0
    graph.remove_vertex(v0);

    // Check if v0 is gone
    assert!(!graph.has_vertex(v0), "Vertex should be removed");

    // Check if we can add edges to it
    let added = graph.add_edge(v0, v0, 1.0);
    assert!(!added, "Should not be able to add edge to removed vertex");
}
