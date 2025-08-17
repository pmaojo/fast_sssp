use std::time::Instant;
use colored::*;
use rand::Rng;
use ordered_float::OrderedFloat;

use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::{FastSSSP, DegreeMode};
use fast_sssp::algorithm::smart_sssp::{SmartSSSP, SmartMode};
use fast_sssp::algorithm::traits::ShortestPathAlgorithm;
use fast_sssp::graph::DirectedGraph;
use fast_sssp::graph::generators::{generate_barabasi_albert, generate_3d_grid, generate_geometric_3d};
use fast_sssp::graph::Graph;

fn main() {
    println!("{}", "Scientific Benchmark for SSSP Algorithms".green().bold());

    // Test sizes for graphs
    let sizes = [10_000, 50_000, 100_000];

    // Test 1: Scale-free graphs (Barabási–Albert model)
    for &size in &sizes {
        println!("\n{} {}", "Testing scale-free graph of size".yellow(), size);
        let start = Instant::now();
        let graph = generate_barabasi_albert(size, 3);
        let gen_time = start.elapsed();
        println!("Generated scale-free graph: {} vertices, {} edges in {:.2?}", graph.vertex_count(), graph.edge_count(), gen_time);

        run_algorithms_on_graph(&graph);
    }

    // Test 2: 3D grid graphs
    for &(nx, ny, nz) in &[(30, 30, 30), (40, 40, 40)] {
        println!("\n{} {}x{}x{}", "Testing 3D grid graph".yellow(), nx, ny, nz);
        let start = Instant::now();
        let graph = generate_3d_grid(nx, ny, nz);
        let gen_time = start.elapsed();
        println!("Generated 3D grid graph: {} vertices, {} edges in {:.2?}", graph.vertex_count(), graph.edge_count(), gen_time);

        run_algorithms_on_graph(&graph);
    }

    // Test 3: 3D geometric graphs
    for &size in &sizes {
        println!("\n{} {} {}", "Testing 3D geometric graph of size".yellow(), size, "with radius 0.1");
        let start = Instant::now();
        let graph = generate_geometric_3d(size, 0.1);
        let gen_time = start.elapsed();
        println!("Generated 3D geometric graph: {} vertices, {} edges in {:.2?}", graph.vertex_count(), graph.edge_count(), gen_time);

        run_algorithms_on_graph(&graph);
    }
}

fn run_algorithms_on_graph(graph: &DirectedGraph<OrderedFloat<f64>>) {
    // Choose random source vertex
    let mut rng = rand::thread_rng();
    let source = rng.gen_range(0..graph.vertex_count());
    println!("Source vertex: {}", source);

    // Dijkstra baseline
    let dijkstra = Dijkstra::new();
    let start = Instant::now();
    let dijkstra_result = dijkstra.compute_shortest_paths(graph, source).unwrap();
    let dijkstra_time = start.elapsed();
    println!("Dijkstra time: {:.2?}", dijkstra_time);

    // FastSSSP with automatic mode
    let fast_sssp = FastSSSP::new().with_degree_mode(DegreeMode::Auto { delta: 256 });
    let start = Instant::now();
    let fast_result = fast_sssp.compute_shortest_paths(graph, source).unwrap();
    let fast_time = start.elapsed();
    println!("FastSSSP time: {:.2?}", fast_time);

    // SmartSSSP in Auto mode
    let mut smart_sssp = SmartSSSP::with_mode(SmartMode::Auto);
    let start = Instant::now();
    let smart_result = smart_sssp.compute_shortest_paths(graph, source).unwrap();
    let smart_time = start.elapsed();
    println!("SmartSSSP (Auto) time: {:.2?}", smart_time);

    // Compare correctness
    let correct_fast = verify_results(&dijkstra_result.distances, &fast_result.distances);
    let correct_smart = verify_results(&dijkstra_result.distances, &smart_result.distances);

    println!("FastSSSP correct: {}", correct_fast);
    println!("SmartSSSP correct: {}", correct_smart);
}

fn verify_results(
    dist1: &Vec<Option<OrderedFloat<f64>>>,
    dist2: &Vec<Option<OrderedFloat<f64>>>,
) -> bool {
    if dist1.len() != dist2.len() {
        return false;
    }
    for i in 0..dist1.len() {
        match (&dist1[i], &dist2[i]) {
            (Some(d1), Some(d2)) => {
                if (d1.0 - d2.0).abs() > 1e-9 {
                    return false;
                }
            }
            (None, None) => {}
            _ => return false,
        }
    }
    true
}
