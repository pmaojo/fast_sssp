use fast_sssp::{DirectedGraph, FastSSSP, Dijkstra, ShortestPathAlgorithm};
use fast_sssp::graph::{Graph, MutableGraph};
use ordered_float::OrderedFloat;

fn main() {
    // Create a simple directed graph
    let mut graph = DirectedGraph::new();
    
    // Add vertices (0-4)
    for _ in 0..5 {
        graph.add_vertex();
    }
    
    // Add edges with weights
    graph.add_edge(0, 1, OrderedFloat(10.0));
    graph.add_edge(0, 2, OrderedFloat(5.0));
    graph.add_edge(1, 3, OrderedFloat(1.0));
    graph.add_edge(2, 1, OrderedFloat(3.0));
    graph.add_edge(2, 3, OrderedFloat(9.0));
    graph.add_edge(2, 4, OrderedFloat(2.0));
    graph.add_edge(3, 4, OrderedFloat(4.0));
    graph.add_edge(4, 0, OrderedFloat(7.0));
    graph.add_edge(4, 3, OrderedFloat(6.0));
    
    // Source vertex
    let source = 0;
    
    // Compare FastSSSP with classic Dijkstra
    println!("--- Testing on a simple graph ---");
    println!("Graph has {} vertices and {} edges", graph.vertex_count(), graph.edge_count());
    
    // Run the Fast SSSP algorithm
    let fast_sssp = FastSSSP::new();
    let fast_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    
    println!("\n{} algorithm results:", <FastSSSP as ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>>::name(&fast_sssp));
    for target in 1..graph.vertex_count() {
        match fast_result.distances[target] {
            None => println!("  No path to {}", target),
            Some(dist) => println!("  Distance to {}: {:.1}", target, dist.into_inner()),
        }
    }
    
    // Run classic Dijkstra's algorithm
    let dijkstra = Dijkstra::new();
    let dijkstra_result = dijkstra.compute_shortest_paths(&graph, source).unwrap();
    
    println!("\n{} algorithm results:", <Dijkstra as ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>>::name(&dijkstra));
    for v in 0..graph.vertex_count() {
        if let Some(dist) = dijkstra_result.distances[v] {
            let path = <Dijkstra as ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>>::get_path(&dijkstra, &dijkstra_result, v).unwrap();
            println!("Vertex {}: distance = {:.1}, path = {:?}", v, dist.into_inner(), path);
        } else {
            println!("Vertex {}: unreachable", v);
        }
    }
}
