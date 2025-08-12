use std::time::Instant;
use colored::*;
use ordered_float::OrderedFloat;
use rand::Rng;

use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::{FastSSSP, DegreeMode};
use fast_sssp::algorithm::bmssp::BMSSP;
use fast_sssp::algorithm::traits::ShortestPathAlgorithm;
use fast_sssp::graph::DirectedGraph;
use fast_sssp::graph::generators::{generate_barabasi_albert, generate_3d_grid, generate_geometric_3d};
use fast_sssp::graph::Graph;

fn main() {
    println!("{}", "BMSSP Benchmark".green().bold());

    // Generate a 3D grid graph
    let (nx, ny, nz) = (50, 50, 50);
    println!("Generating 3D grid graph {}x{}x{}...", nx, ny, nz);
    let start = Instant::now();
    let graph = generate_3d_grid(nx, ny, nz);
    let gen_time = start.elapsed();
    println!("Generated graph with {} vertices and {} edges in {:.2?}", graph.vertex_count(), graph.edge_count(), gen_time);

    // Pick random source
    let mut rng = rand::thread_rng();
    let source = rng.gen_range(0..graph.vertex_count());
    println!("Source vertex: {}", source);

    // Compare BMSSP parameters
    let params = vec![(2, 4), (3, 6), (4, 8)];

    for (k, t) in params {
        println!("\nTesting BMSSP with k={}, t={}", k, t);
        let bmssp = BMSSP::<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>::new_with_params(graph.vertex_count(), k, t);

        // Prepare containers
        let mut distances = vec![OrderedFloat(f64::INFINITY); graph.vertex_count()];
        let mut predecessors = vec![None; graph.vertex_count()];

        // Run Dijkstra baseline
        let start = Instant::now();
        let dijkstra = Dijkstra::new();
        let dijkstra_result = dijkstra.compute_shortest_paths(&graph, source).unwrap();
        let dijkstra_time = start.elapsed();
        println!("Dijkstra time: {:.2?}", dijkstra_time);

        // Run FastSSSP baseline
        let start = Instant::now();
        let fast_sssp = FastSSSP::new().with_degree_mode(DegreeMode::Auto { delta: 256 });
        let fast_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
        let fast_time = start.elapsed();
        println!("FastSSSP time: {:.2?}", fast_time);

        // Execute BMSSP base case for demonstration
        let bound = OrderedFloat(1000.0);
        let sources = vec![source];
        match bmssp.execute(&graph, 0, bound, &sources, &mut distances, &mut predecessors) {
            Ok(res) => println!("BMSSP base case: new_bound={:.2}, vertices={}", res.new_bound.0, res.vertices.len()),
            Err(e) => println!("BMSSP error: {:?}", e),
        }

        // Verify correctness with Dijkstra on a sample
        let mut mismatches = 0;
        for i in 0..graph.vertex_count().min(1000) {
            let d_fast = fast_result.distances[i].map(|x| x.0);
            let d_dij = dijkstra_result.distances[i].map(|x| x.0);
            if d_fast != d_dij { mismatches += 1; }
        }
        if mismatches == 0 {
            println!("✓ FastSSSP matches Dijkstra on sampled vertices");
        } else {
            println!("⚠️  {} mismatches between FastSSSP and Dijkstra on sample", mismatches);
        }
    }
}
