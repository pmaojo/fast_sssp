use std::time::Instant;
use colored::*;
use ordered_float::OrderedFloat;
use fast_sssp::algorithm::ShortestPathAlgorithm;
use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::{FastSSSP, DegreeMode};
use fast_sssp::graph::DirectedGraph;
use fast_sssp::graph::generators::{generate_3d_grid, generate_barabasi_albert};
use fast_sssp::graph::Graph;

fn main() {
    println!("{}", "Fast SSSP Simple Example".green().bold());

    // Create a simple graph
    let start = Instant::now();
    let graph = generate_3d_grid(20, 20, 20);
    let gen_time = start.elapsed().as_secs_f64();

    println!("Graph has {} vertices and {} edges", graph.vertex_count(), graph.edge_count());
    println!("Graph generated in {:.2} seconds", gen_time);

    // Run Dijkstra
    let dijkstra = Dijkstra::new();
    let source = 0;
    let result = dijkstra.compute_shortest_paths(&graph, source).unwrap();

    // Print distances to first 10 vertices
    for target in 1..graph.vertex_count() {
        if target > 10 { break; }
        if let Some(dist) = result.distances[target] {
            println!("Distance from {} to {}: {:.2}", source, target, dist.0);
        }
    }

    // Run FastSSSP with HubSplit transformation enabled automatically
    let fast_sssp = FastSSSP::new().with_degree_mode(DegreeMode::Auto { delta: 256 });
    let start = Instant::now();
    let result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    let elapsed = start.elapsed().as_secs_f64();

    println!("FastSSSP computed shortest paths in {:.2} seconds", elapsed);

    // Print path to the 10th vertex
    use fast_sssp::algorithm::ShortestPathAlgorithm as _;
    let path = <FastSSSP as fast_sssp::algorithm::traits::ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>>::get_path(&fast_sssp, &result, 10);
    if let Some(path) = path {
        println!("Path from 0 to 10: {:?}", path);
    }
}
