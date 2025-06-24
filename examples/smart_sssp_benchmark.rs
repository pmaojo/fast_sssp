use std::time::Instant;
use std::fs::File;
use std::io::Write;
use ordered_float::OrderedFloat;
use colored::*;
use rand::Rng;

use fast_sssp::algorithm::ShortestPathAlgorithm;
use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::{FastSSSP, DegreeMode};
use fast_sssp::algorithm::smart_sssp::{SmartSSSP, SmartMode};
use fast_sssp::graph::DirectedGraph;
use fast_sssp::graph::generators::{generate_barabasi_albert, generate_3d_grid, generate_geometric_3d};

/// Run a comprehensive benchmark comparing SmartSSSP against Dijkstra and FastSSSP variants
fn main() {
    println!("{}", "Smart SSSP Scientific Benchmark".green().bold());
    println!("Testing algorithm performance across different graph families\n");

    // Create CSV file for results
    let mut csv_file = File::create("smart_sssp_results.csv").expect("Could not create CSV file");
    writeln!(csv_file, "graph_type,size,avg_degree,reachable_vertices,dijkstra_ms,fast_sssp_ms,fast_sssp_no_transform_ms,smart_auto_ms,smart_adaptive_ms,winner").unwrap();

    // Test on different graph families
    benchmark_scale_free_graphs(&mut csv_file);
    benchmark_3d_grid_graphs(&mut csv_file);
    benchmark_geometric_graphs(&mut csv_file);

    println!("\n{}", "Results saved to smart_sssp_results.csv".green());
    
    // Print final statistics
    let mut smart_sssp = SmartSSSP::new()
        .with_stats_collection(true)
        .with_verbose(true);
        
    println!("\n{}", smart_sssp.get_stats());
}

/// Benchmark on scale-free graphs (Barabási-Albert model)
fn benchmark_scale_free_graphs(csv_file: &mut File) {
    println!("\n{}", "Testing Scale-Free Graphs".yellow().bold());
    
    let sizes = [10_000, 50_000, 100_000, 500_000, 1_000_000];
    let edges_per_node = [2, 3, 5];
    
    for &size in &sizes {
        for &m in &edges_per_node {
            println!("\n📈 Testing scale-free graph with {} vertices, {} edges per new node", size, m);
            
            // Generate graph
            let start = Instant::now();
            let graph = generate_barabasi_albert(size, m);
            let gen_time = start.elapsed();
            println!("✓ Graph generated with {} vertices and {} edges in {:.2}s", 
                graph.vertex_count(), graph.edge_count(), gen_time.as_secs_f64());
            
            // Select random source vertex
            let mut rng = rand::thread_rng();
            let source = rng.gen_range(0..graph.vertex_count());
            println!("🎯 Source vertex: {}", source);
            
            // Run benchmark
            let result = benchmark_algorithms(&graph, source);
            
            // Save results to CSV
            writeln!(csv_file, "scale_free,{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{}",
                size, m, result.reachable, 
                result.dijkstra_ms, result.fast_sssp_ms, result.fast_sssp_no_transform_ms,
                result.smart_auto_ms, result.smart_adaptive_ms, result.winner).unwrap();
        }
    }
}

/// Benchmark on 3D grid graphs
fn benchmark_3d_grid_graphs(csv_file: &mut File) {
    println!("\n{}", "Testing 3D Grid Graphs".yellow().bold());
    
    let dimensions = [
        (20, 20, 20),   // 8,000 vertices
        (30, 30, 30),   // 27,000 vertices
        (40, 40, 40),   // 64,000 vertices
        (50, 50, 50),   // 125,000 vertices
        (60, 60, 60),   // 216,000 vertices
    ];
    
    for &(nx, ny, nz) in &dimensions {
        let size = nx * ny * nz;
        println!("\n📈 Testing 3D grid graph with dimensions {}x{}x{} ({} vertices)", nx, ny, nz, size);
        
        // Generate graph
        let start = Instant::now();
        let graph = generate_3d_grid(nx, ny, nz);
        let gen_time = start.elapsed();
        println!("✓ Graph generated with {} vertices and {} edges in {:.2}s", 
            graph.vertex_count(), graph.edge_count(), gen_time.as_secs_f64());
        
        // Select random source vertex
        let mut rng = rand::thread_rng();
        let source = rng.gen_range(0..graph.vertex_count());
        println!("🎯 Source vertex: {}", source);
        
        // Run benchmark
        let result = benchmark_algorithms(&graph, source);
        
        // Save results to CSV
        writeln!(csv_file, "3d_grid,{},6,{},{:.2},{:.2},{:.2},{:.2},{:.2},{}",
            size, result.reachable, 
            result.dijkstra_ms, result.fast_sssp_ms, result.fast_sssp_no_transform_ms,
            result.smart_auto_ms, result.smart_adaptive_ms, result.winner).unwrap();
    }
}

/// Benchmark on 3D geometric graphs
fn benchmark_geometric_graphs(csv_file: &mut File) {
    println!("\n{}", "Testing 3D Geometric Graphs".yellow().bold());
    
    let sizes = [10_000, 50_000, 100_000, 200_000];
    let radii = [0.05, 0.1, 0.2];
    
    for &size in &sizes {
        for &radius in &radii {
            println!("\n📈 Testing 3D geometric graph with {} vertices, radius {}", size, radius);
            
            // Generate graph
            let start = Instant::now();
            let graph = generate_geometric_3d(size, radius);
            let gen_time = start.elapsed();
            println!("✓ Graph generated with {} vertices and {} edges in {:.2}s", 
                graph.vertex_count(), graph.edge_count(), gen_time.as_secs_f64());
            
            let avg_degree = graph.edge_count() as f64 / graph.vertex_count() as f64;
            println!("📊 Average degree: {:.2}", avg_degree);
            
            // Select random source vertex
            let mut rng = rand::thread_rng();
            let source = rng.gen_range(0..graph.vertex_count());
            println!("🎯 Source vertex: {}", source);
            
            // Run benchmark
            let result = benchmark_algorithms(&graph, source);
            
            // Save results to CSV
            writeln!(csv_file, "geometric_3d,{},{:.2},{},{:.2},{:.2},{:.2},{:.2},{:.2},{}",
                size, avg_degree, result.reachable, 
                result.dijkstra_ms, result.fast_sssp_ms, result.fast_sssp_no_transform_ms,
                result.smart_auto_ms, result.smart_adaptive_ms, result.winner).unwrap();
        }
    }
}

/// Result of benchmarking algorithms
struct BenchmarkResult {
    reachable: usize,
    dijkstra_ms: f64,
    fast_sssp_ms: f64,
    fast_sssp_no_transform_ms: f64,
    smart_auto_ms: f64,
    smart_adaptive_ms: f64,
    winner: String,
}

/// Benchmark all algorithms on a graph and return results
fn benchmark_algorithms(graph: &DirectedGraph<OrderedFloat<f64>>, source: usize) -> BenchmarkResult {
    // Run Dijkstra (baseline)
    println!("🏃 Running Dijkstra (baseline)...");
    let start = Instant::now();
    let dijkstra = Dijkstra::new();
    let dijkstra_result = dijkstra.compute_shortest_paths(graph, source).unwrap();
    let dijkstra_time = start.elapsed();
    let dijkstra_ms = dijkstra_time.as_secs_f64() * 1000.0;
    println!("⏱️  Dijkstra time: {:.2}ms", dijkstra_ms);
    
    // Count reachable vertices
    let reachable = dijkstra_result.distances.iter()
        .filter(|d| d.is_some())
        .count();
    println!("📍 Reachable vertices: {}", reachable);
    
    // Run FastSSSP with HubSplit
    println!("🏃 Running FastSSSP with HubSplit...");
    let start = Instant::now();
    let fast_sssp = FastSSSP::with_degree_mode(DegreeMode::Auto { delta: 256 });
    let fast_sssp_result = fast_sssp.compute_shortest_paths(graph, source).unwrap();
    let fast_sssp_time = start.elapsed();
    let fast_sssp_ms = fast_sssp_time.as_secs_f64() * 1000.0;
    println!("⏱️  Time: {:.2}ms", fast_sssp_ms);
    
    // Verify correctness
    let fast_sssp_correct = verify_results(&dijkstra_result.distances, &fast_sssp_result.distances);
    if fast_sssp_correct {
        println!("✓ FastSSSP with HubSplit correctly found all {} reachable vertices", reachable);
    } else {
        println!("❌ FastSSSP with HubSplit produced incorrect results");
    }
    
    // Run FastSSSP without transformation
    println!("🏃 Running FastSSSP without transformation...");
    let start = Instant::now();
    let fast_sssp_no_transform = FastSSSP::with_degree_mode(DegreeMode::None);
    let fast_sssp_no_transform_result = fast_sssp_no_transform.compute_shortest_paths(graph, source).unwrap();
    let fast_sssp_no_transform_time = start.elapsed();
    let fast_sssp_no_transform_ms = fast_sssp_no_transform_time.as_secs_f64() * 1000.0;
    println!("⏱️  Time: {:.2}ms", fast_sssp_no_transform_ms);
    
    // Verify correctness
    let fast_sssp_no_transform_correct = verify_results(&dijkstra_result.distances, &fast_sssp_no_transform_result.distances);
    if fast_sssp_no_transform_correct {
        println!("✓ FastSSSP without transformation correctly found all {} reachable vertices", reachable);
    } else {
        println!("❌ FastSSSP without transformation produced incorrect results");
    }
    
    // Run SmartSSSP in Auto mode
    println!("🏃 Running SmartSSSP in Auto mode...");
    let start = Instant::now();
    let mut smart_auto = SmartSSSP::with_mode(SmartMode::Auto);
    let smart_auto_result = smart_auto.compute_shortest_paths(graph, source).unwrap();
    let smart_auto_time = start.elapsed();
    let smart_auto_ms = smart_auto_time.as_secs_f64() * 1000.0;
    println!("⏱️  Time: {:.2}ms", smart_auto_ms);
    
    // Verify correctness
    let smart_auto_correct = verify_results(&dijkstra_result.distances, &smart_auto_result.distances);
    if smart_auto_correct {
        println!("✓ SmartSSSP (Auto) correctly found all {} reachable vertices", reachable);
    } else {
        println!("❌ SmartSSSP (Auto) produced incorrect results");
    }
    
    // Run SmartSSSP in Adaptive mode
    println!("🏃 Running SmartSSSP in Adaptive mode...");
    let start = Instant::now();
    let mut smart_adaptive = SmartSSSP::with_mode(SmartMode::Adaptive)
        .with_stats_collection(true);
    let smart_adaptive_result = smart_adaptive.compute_shortest_paths(graph, source).unwrap();
    let smart_adaptive_time = start.elapsed();
    let smart_adaptive_ms = smart_adaptive_time.as_secs_f64() * 1000.0;
    println!("⏱️  Time: {:.2}ms", smart_adaptive_ms);
    
    // Verify correctness
    let smart_adaptive_correct = verify_results(&dijkstra_result.distances, &smart_adaptive_result.distances);
    if smart_adaptive_correct {
        println!("✓ SmartSSSP (Adaptive) correctly found all {} reachable vertices", reachable);
    } else {
        println!("❌ SmartSSSP (Adaptive) produced incorrect results");
    }
    
    // Determine winner
    let mut winner = "dijkstra";
    let mut min_time = dijkstra_ms;
    
    if fast_sssp_ms < min_time && fast_sssp_correct {
        winner = "fast_sssp";
        min_time = fast_sssp_ms;
    }
    
    if fast_sssp_no_transform_ms < min_time && fast_sssp_no_transform_correct {
        winner = "fast_sssp_no_transform";
        min_time = fast_sssp_no_transform_ms;
    }
    
    if smart_auto_ms < min_time && smart_auto_correct {
        winner = "smart_auto";
        min_time = smart_auto_ms;
    }
    
    if smart_adaptive_ms < min_time && smart_adaptive_correct {
        winner = "smart_adaptive";
        min_time = smart_adaptive_ms;
    }
    
    println!("🏆 Winner: {} ({:.2}ms)", winner, min_time);
    
    BenchmarkResult {
        reachable,
        dijkstra_ms,
        fast_sssp_ms,
        fast_sssp_no_transform_ms,
        smart_auto_ms,
        smart_adaptive_ms,
        winner: winner.to_string(),
    }
}

/// Verify that two sets of distances match
fn verify_results(
    baseline: &[Option<OrderedFloat<f64>>], 
    result: &[Option<OrderedFloat<f64>>]
) -> bool {
    if baseline.len() != result.len() {
        return false;
    }
    
    for (i, (b, r)) in baseline.iter().zip(result.iter()).enumerate() {
        match (b, r) {
            (Some(b_val), Some(r_val)) => {
                // Allow small floating point differences
                let diff = (*b_val - *r_val).abs();
                if diff > 1e-6 {
                    println!("Distance mismatch at vertex {}: baseline={}, result={}", i, b_val, r_val);
                    return false;
                }
            },
            (None, None) => {
                // Both unreachable, good
            },
            _ => {
                println!("Reachability mismatch at vertex {}: baseline={:?}, result={:?}", 
                    i, b, r);
                return false;
            }
        }
    }
    
    true
}
