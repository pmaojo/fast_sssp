use colored::*;
use ordered_float::OrderedFloat;
use std::time::Instant;

use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::{DegreeMode, FastSSSP};
use fast_sssp::algorithm::smart_sssp::{SmartMode, SmartSSSP};
use fast_sssp::algorithm::ShortestPathAlgorithm;
use fast_sssp::graph::generators::{
    generate_3d_grid, generate_barabasi_albert, generate_geometric_3d,
};
use fast_sssp::graph::DirectedGraph;
use fast_sssp::graph::Graph; // Importamos el trait Graph

fn main() {
    println!("{}", "SmartSSSP Demo".green().bold());
    println!("Demonstrating intelligent algorithm selection based on graph properties\n");

    // Test on different graph types
    test_on_small_graph();
    test_on_large_sparse_graph();
    test_on_large_dense_graph();

    // Compare modes
    compare_smart_modes();
}

/// Test on a small graph where Dijkstra should be preferred
fn test_on_small_graph() {
    println!(
        "\n{}",
        "Testing on small graph (Dijkstra should win)"
            .yellow()
            .bold()
    );

    // Generate a small graph
    let graph = generate_barabasi_albert(5_000, 3);
    println!(
        "Generated scale-free graph with {} vertices and {} edges",
        graph.vertex_count(),
        graph.edge_count()
    );

    // Source vertex
    let source = 0;

    // Run algorithms
    run_comparison(&graph, source);
}

/// Test on a large sparse graph where FastSSSP might have an advantage
fn test_on_large_sparse_graph() {
    println!("\n{}", "Testing on large sparse graph".yellow().bold());

    // Generate a large sparse graph
    let graph = generate_3d_grid(50, 50, 50);
    println!(
        "Generated 3D grid with {} vertices and {} edges",
        graph.vertex_count(),
        graph.edge_count()
    );

    // Source vertex
    let source = 0;

    // Run algorithms
    run_comparison(&graph, source);
}

/// Test on a large dense graph
fn test_on_large_dense_graph() {
    println!("\n{}", "Testing on large dense graph".yellow().bold());

    // Generate a large dense graph
    let graph = generate_geometric_3d(50_000, 0.1);
    println!(
        "Generated 3D geometric graph with {} vertices and {} edges",
        graph.vertex_count(),
        graph.edge_count()
    );

    // Source vertex
    let source = 0;

    // Run algorithms
    run_comparison(&graph, source);
}

/// Compare different SmartSSSP modes
fn compare_smart_modes() {
    println!("\n{}", "Comparing SmartSSSP modes".yellow().bold());

    // Generate a medium-sized graph
    let graph = generate_barabasi_albert(50_000, 5);
    println!(
        "Generated scale-free graph with {} vertices and {} edges",
        graph.vertex_count(),
        graph.edge_count()
    );

    // Source vertex
    let source = 0;

    // Run Dijkstra (baseline)
    println!("\n🏃 Running Dijkstra (baseline)...");
    let start = Instant::now();
    let dijkstra = Dijkstra::new();
    let dijkstra_result = dijkstra.compute_shortest_paths(&graph, source).unwrap();
    let dijkstra_time = start.elapsed();
    let dijkstra_ms = dijkstra_time.as_secs_f64() * 1000.0;
    println!("⏱️  Dijkstra time: {:.2}ms", dijkstra_ms);

    // Count reachable vertices
    let reachable = dijkstra_result
        .distances
        .iter()
        .filter(|d| d.is_some())
        .count();
    println!("📍 Reachable vertices: {}", reachable);

    // Run SmartSSSP in Auto mode
    println!("\n🏃 Running SmartSSSP in Auto mode...");
    let start = Instant::now();
    let smart_auto = SmartSSSP::with_mode(SmartMode::Auto).with_verbose(true);
    let smart_auto_result = smart_auto.compute_shortest_paths(&graph, source).unwrap();
    let smart_auto_time = start.elapsed();
    let smart_auto_ms = smart_auto_time.as_secs_f64() * 1000.0;
    println!("⏱️  SmartSSSP (Auto) time: {:.2}ms", smart_auto_ms);

    // Run SmartSSSP in Adaptive mode
    println!("\n🏃 Running SmartSSSP in Adaptive mode...");
    let start = Instant::now();
    let smart_adaptive = SmartSSSP::with_mode(SmartMode::Adaptive)
        .with_stats_collection(true)
        .with_verbose(true);
    let smart_adaptive_result = smart_adaptive
        .compute_shortest_paths(&graph, source)
        .unwrap();
    let smart_adaptive_time = start.elapsed();
    let smart_adaptive_ms = smart_adaptive_time.as_secs_f64() * 1000.0;
    println!("⏱️  SmartSSSP (Adaptive) time: {:.2}ms", smart_adaptive_ms);

    // Run SmartSSSP in SimpleFastSSSP mode
    println!("\n🏃 Running SmartSSSP in SimpleFastSSSP mode...");
    let start = Instant::now();
    let smart_simple = SmartSSSP::with_mode(SmartMode::SimpleFastSSSP).with_verbose(true);
    let smart_simple_result = smart_simple.compute_shortest_paths(&graph, source).unwrap();
    let smart_simple_time = start.elapsed();
    let smart_simple_ms = smart_simple_time.as_secs_f64() * 1000.0;
    println!(
        "⏱️  SmartSSSP (SimpleFastSSSP) time: {:.2}ms",
        smart_simple_ms
    );

    // Run SmartSSSP in ForceFastSSSP mode
    println!("\n🏃 Running SmartSSSP in ForceFastSSSP mode...");
    let start = Instant::now();
    let smart_force = SmartSSSP::with_mode(SmartMode::ForceFastSSSP).with_verbose(true);
    let smart_force_result = smart_force.compute_shortest_paths(&graph, source).unwrap();
    let smart_force_time = start.elapsed();
    let smart_force_ms = smart_force_time.as_secs_f64() * 1000.0;
    println!(
        "⏱️  SmartSSSP (ForceFastSSSP) time: {:.2}ms",
        smart_force_ms
    );

    // Compare accuracy against Dijkstra for each SmartSSSP mode
    let compare_to_dijkstra =
        |label: &str, result: &fast_sssp::algorithm::ShortestPathResult<OrderedFloat<f64>>| {
            let mismatches = result
                .distances
                .iter()
                .zip(&dijkstra_result.distances)
                .filter(|(a, b)| a != b)
                .count();
            if mismatches == 0 {
                println!(
                    "✓ {} matches Dijkstra on all {} vertices",
                    label,
                    result.distances.len()
                );
            } else {
                println!(
                    "⚠️  {} differs from Dijkstra on {} of {} vertices",
                    label,
                    mismatches,
                    result.distances.len()
                );
            }
        };

    compare_to_dijkstra("SmartSSSP (Auto)", &smart_auto_result);
    compare_to_dijkstra("SmartSSSP (Adaptive)", &smart_adaptive_result);
    compare_to_dijkstra("SmartSSSP (SimpleFastSSSP)", &smart_simple_result);
    compare_to_dijkstra("SmartSSSP (ForceFastSSSP)", &smart_force_result);

    // Print summary
    println!("\n{}", "Summary".green().bold());
    println!("Dijkstra: {:.2}ms", dijkstra_ms);
    println!("SmartSSSP (Auto): {:.2}ms", smart_auto_ms);
    println!("SmartSSSP (Adaptive): {:.2}ms", smart_adaptive_ms);
    println!("SmartSSSP (SimpleFastSSSP): {:.2}ms", smart_simple_ms);
    println!("SmartSSSP (ForceFastSSSP): {:.2}ms", smart_force_ms);

    // Print stats
    println!("\n{}", smart_adaptive.get_stats());
}

/// Run a comparison of algorithms on the given graph
fn run_comparison(graph: &DirectedGraph<OrderedFloat<f64>>, source: usize) {
    // Run Dijkstra
    println!("\n🏃 Running Dijkstra...");
    let start = Instant::now();
    let dijkstra = Dijkstra::new();
    let dijkstra_result = dijkstra.compute_shortest_paths(graph, source).unwrap();
    let dijkstra_time = start.elapsed();
    println!("⏱️  Time: {:.2}ms", dijkstra_time.as_secs_f64() * 1000.0);

    // Count reachable vertices
    let reachable = dijkstra_result
        .distances
        .iter()
        .filter(|d| d.is_some())
        .count();
    println!("📍 Reachable vertices: {}", reachable);

    // Run FastSSSP with HubSplit
    println!("\n🏃 Running FastSSSP with HubSplit...");
    let start = Instant::now();
    let mut fast_sssp = FastSSSP::new();
    fast_sssp = fast_sssp.with_degree_mode(DegreeMode::Auto { delta: 256 });
    let _fast_sssp_result = fast_sssp.compute_shortest_paths(graph, source).unwrap();
    let fast_sssp_time = start.elapsed();
    println!("⏱️  Time: {:.2}ms", fast_sssp_time.as_secs_f64() * 1000.0);

    // Run FastSSSP without transformation
    println!("\n🏃 Running FastSSSP without transformation...");
    let start = Instant::now();
    let mut fast_sssp_no_transform = FastSSSP::new();
    fast_sssp_no_transform = fast_sssp_no_transform.with_degree_mode(DegreeMode::None);
    let _fast_sssp_no_transform_result = fast_sssp_no_transform
        .compute_shortest_paths(graph, source)
        .unwrap();
    let fast_sssp_no_transform_time = start.elapsed();
    println!(
        "⏱️  Time: {:.2}ms",
        fast_sssp_no_transform_time.as_secs_f64() * 1000.0
    );

    // Run SmartSSSP
    println!("\n🏃 Running SmartSSSP...");
    let start = Instant::now();
    let smart_sssp = SmartSSSP::new().with_verbose(true);
    let _smart_sssp_result = smart_sssp.compute_shortest_paths(graph, source).unwrap();
    let smart_sssp_time = start.elapsed();
    println!("⏱️  Time: {:.2}ms", smart_sssp_time.as_secs_f64() * 1000.0);

    // Print summary
    println!("\n{}", "Summary".green().bold());
    println!("Dijkstra: {:.2}ms", dijkstra_time.as_secs_f64() * 1000.0);
    println!(
        "FastSSSP with HubSplit: {:.2}ms",
        fast_sssp_time.as_secs_f64() * 1000.0
    );
    println!(
        "FastSSSP without transformation: {:.2}ms",
        fast_sssp_no_transform_time.as_secs_f64() * 1000.0
    );
    println!("SmartSSSP: {:.2}ms", smart_sssp_time.as_secs_f64() * 1000.0);
}
