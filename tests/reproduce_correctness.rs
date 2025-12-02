use fast_sssp::algorithm::{dijkstra::Dijkstra, fast_sssp::FastSSSP, ShortestPathAlgorithm};
use fast_sssp::graph::{DirectedGraph, Graph, MutableGraph};
use ordered_float::OrderedFloat;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[test]
fn reproduce_correctness_issue() {
    let num_vertices = 1000;
    let edge_factor = 2.0;
    let seed = 42;

    println!("Generating graph with seed {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let mut graph = DirectedGraph::with_capacity(num_vertices);
    for _ in 0..num_vertices {
        graph.add_vertex();
    }

    let num_edges = (edge_factor * num_vertices as f64) as usize;
    for _ in 0..num_edges {
        let u = rng.gen_range(0..num_vertices);
        let v = rng.gen_range(0..num_vertices);
        if u != v {
            let weight = OrderedFloat(rng.gen_range(1.0..100.0));
            graph.add_edge(u, v, weight);
        }
    }

    let source = 0;

    let dijkstra = Dijkstra::new();
    let d_result = dijkstra.compute_shortest_paths(&graph, source).unwrap();
    let d_reachable = d_result.distances.iter().filter(|d| d.is_some()).count();

    // Use FastSSSP with low threshold to force BMSSP usage
    let fast_sssp = FastSSSP::new().with_vertex_threshold(0);
    let f_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    let f_reachable = f_result.distances.iter().filter(|d| d.is_some()).count();

    println!("Dijkstra reachable: {}", d_reachable);
    println!("FastSSSP reachable: {}", f_reachable);

    assert_eq!(
        d_reachable, f_reachable,
        "FastSSSP should find same number of reachable vertices"
    );

    // Also check distances match
    for i in 0..num_vertices {
        assert_eq!(
            d_result.distances[i], f_result.distances[i],
            "Distance mismatch at vertex {}",
            i
        );
    }
}
