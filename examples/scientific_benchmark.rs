use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::{FastSSSP, DegreeMode};
use fast_sssp::algorithm::traits::ShortestPathAlgorithm;
use fast_sssp::graph::{DirectedGraph, Graph, generate_barabasi_albert, generate_3d_grid, generate_geometric_3d};
use ordered_float::OrderedFloat;
use std::time::Instant;
use std::fs::File;
use std::io::{Write, BufWriter};
use colored::*;
use rand::prelude::*;

fn main() {
    println!("{}", "🔬 Scientific Benchmark Suite for Fast-SSSP 🔬".bright_green());
    println!("Testing specific graph families where Fast-SSSP might outperform Dijkstra");
    println!("========================================================================");
    
    // Create CSV file for results
    let mut results_file = BufWriter::new(File::create("benchmark_results.csv").unwrap());
    writeln!(results_file, "graph_type,n,m,reachable,algorithm,time_ms,delta").unwrap();
    
    // Run benchmarks for different graph families
    benchmark_scale_free(&mut results_file);
    benchmark_3d_grid(&mut results_file);
    benchmark_geometric_3d(&mut results_file);
    
    println!("\n{}", "✅ Benchmark complete! Results saved to benchmark_results.csv".bright_green());
}

fn benchmark_scale_free(results_file: &mut BufWriter<File>) {
    println!("\n{}", "📊 Scale-Free Graph Benchmarks (Barabási-Albert)".bright_yellow());
    println!("These graphs have power-law degree distribution with many vertices having few connections");
    
    let sizes = [
        (100_000, 2),
        (500_000, 2),
        (1_000_000, 2),
        (5_000_000, 2),
    ];
    
    for (n, m) in sizes {
        if n > 1_000_000 {
            println!("\n⚠️  Generating large graph with {} vertices, this may take a while...", n);
        } else {
            println!("\n📈 Testing scale-free graph with {} vertices, {} edges per new node", n, m);
        }
        
        // Generate graph
        let start = Instant::now();
        let graph = generate_barabasi_albert(n, m);
        let gen_time = start.elapsed();
        
        println!("✓ Graph generated with {} vertices and {} edges in {:.2?}", 
                 graph.vertex_count(), graph.edge_count(), gen_time);
        
        // Choose random source vertex
        let source = rand::thread_rng().gen_range(0..n);
        println!("🎯 Source vertex: {}", source);
        
        // Run benchmarks with different algorithms
        run_algorithm_comparison(&graph, source, "scale_free", results_file);
    }
}

fn benchmark_3d_grid(results_file: &mut BufWriter<File>) {
    println!("\n{}", "🧊 3D Grid Graph Benchmarks".bright_yellow());
    println!("These graphs represent regular 3D grids with 6-connectivity");
    
    let sizes = [
        (100, 100, 100),   // 1M vertices
        (200, 200, 200),   // 8M vertices
    ];
    
    for (x, y, z) in sizes {
        println!("\n📏 Testing 3D grid with dimensions {}x{}x{}", x, y, z);
        
        // Generate graph
        let start = Instant::now();
        let graph = generate_3d_grid(x, y, z);
        let gen_time = start.elapsed();
        
        println!("✓ Graph generated with {} vertices and {} edges in {:.2?}", 
                 graph.vertex_count(), graph.edge_count(), gen_time);
        
        // Choose center vertex as source
        let source = (x/2) * y * z + (y/2) * z + (z/2);
        println!("🎯 Source vertex: {} (center of grid)", source);
        
        // Run benchmarks with different algorithms
        run_algorithm_comparison(&graph, source, "grid_3d", results_file);
    }
}

fn benchmark_geometric_3d(results_file: &mut BufWriter<File>) {
    println!("\n{}", "🌐 3D Geometric Graph Benchmarks".bright_yellow());
    println!("These graphs connect points in 3D space that are within a certain radius");
    
    let sizes = [
        (500_000, 0.05),
        (1_000_000, 0.03),
    ];
    
    for (n, radius) in sizes {
        println!("\n🔍 Testing 3D geometric graph with {} vertices and radius {}", n, radius);
        
        // Generate graph
        let start = Instant::now();
        let graph = generate_geometric_3d(n, radius);
        let gen_time = start.elapsed();
        
        println!("✓ Graph generated with {} vertices and {} edges in {:.2?}", 
                 graph.vertex_count(), graph.edge_count(), gen_time);
        
        // Choose random source vertex
        let source = rand::thread_rng().gen_range(0..n);
        println!("🎯 Source vertex: {}", source);
        
        // Run benchmarks with different algorithms
        run_algorithm_comparison(&graph, source, "geometric_3d", results_file);
    }
}

fn run_algorithm_comparison(
    graph: &DirectedGraph<OrderedFloat<f64>>, 
    source: usize,
    graph_type: &str,
    results_file: &mut BufWriter<File>
)
{
    // Run Dijkstra (baseline)
    println!("🏃 Running Dijkstra (baseline)...");
    let dijkstra = Dijkstra::new();
    let start_time = Instant::now();
    let dijkstra_result = dijkstra.compute_shortest_paths(graph, source).unwrap();
    let dijkstra_time = start_time.elapsed();
    
    // Count reachable vertices
    let dijkstra_reachable = dijkstra_result.distances.iter()
        .filter(|&d| d.is_some())
        .count();
    
    println!("⏱️  Dijkstra time: {}", format!("{:.2?}", dijkstra_time).bright_cyan());
    println!("📍 Reachable vertices: {}", dijkstra_reachable);
    
    // Write Dijkstra results to CSV
    writeln!(
        results_file, 
        "{},{},{},{},{},{},{}",
        graph_type,
        graph.vertex_count(),
        graph.edge_count(),
        dijkstra_reachable,
        "dijkstra",
        dijkstra_time.as_millis(),
        "N/A"
    ).unwrap();
    
    // Test different delta values for HubSplit
    let delta_values = [64, 128, 256, usize::MAX];
    
    for &delta in &delta_values {
        let delta_str = if delta == usize::MAX { "∞".to_string() } else { delta.to_string() };
        
        println!("🏃 Running FastSSSP with Auto mode (delta={})...", delta_str);
        let fast_sssp = FastSSSP::new_with_mode(DegreeMode::Auto { delta });
        
        let start_time = Instant::now();
        let fast_result = fast_sssp.compute_shortest_paths(graph, source).unwrap();
        let fast_time = start_time.elapsed();
        
        // Count reachable vertices
        let fast_reachable = fast_result.distances.iter()
            .filter(|&d| d.is_some())
            .count();
        
        println!("⏱️  Time: {}", format!("{:.2?}", fast_time).bright_cyan());
        
        // Calculate speedup
        let speedup = dijkstra_time.as_secs_f64() / fast_time.as_secs_f64();
        if speedup > 1.0 {
            println!("🚀 Speedup vs Dijkstra: {:.2}x", speedup);
        } else {
            println!("⚠️  Slowdown vs Dijkstra: {:.2}x", 1.0 / speedup);
        }
        
        // Verify results
        verify_results(&fast_result, &dijkstra_result, &format!("FastSSSP (delta={})", delta_str));
        
        // Write FastSSSP results to CSV
        writeln!(
            results_file, 
            "{},{},{},{},{},{},{}",
            graph_type,
            graph.vertex_count(),
            graph.edge_count(),
            fast_reachable,
            "fast_sssp_auto",
            fast_time.as_millis(),
            delta
        ).unwrap();
    }
    
    // Test FastSSSP with None mode (no transformation)
    println!("🏃 Running FastSSSP with None mode...");
    let fast_sssp = FastSSSP::new_with_mode(DegreeMode::None);
    
    let start_time = Instant::now();
    let fast_result = fast_sssp.compute_shortest_paths(graph, source).unwrap();
    let fast_time = start_time.elapsed();
    
    // Count reachable vertices
    let fast_reachable = fast_result.distances.iter()
        .filter(|&d| d.is_some())
        .count();
    
    println!("⏱️  Time: {}", format!("{:.2?}", fast_time).bright_cyan());
    
    // Calculate speedup
    let speedup = dijkstra_time.as_secs_f64() / fast_time.as_secs_f64();
    if speedup > 1.0 {
        println!("🚀 Speedup vs Dijkstra: {:.2}x", speedup);
    } else {
        println!("⚠️  Slowdown vs Dijkstra: {:.2}x", 1.0 / speedup);
    }
    
    // Verify results
    verify_results(&fast_result, &dijkstra_result, "FastSSSP (None)");
    
    // Write FastSSSP results to CSV
    writeln!(
        results_file, 
        "{},{},{},{},{},{},{}",
        graph_type,
        graph.vertex_count(),
        graph.edge_count(),
        fast_reachable,
        "fast_sssp_none",
        fast_time.as_millis(),
        "N/A"
    ).unwrap();
}

fn verify_results<W>(
    result: &fast_sssp::algorithm::traits::ShortestPathResult<W>, 
    baseline: &fast_sssp::algorithm::traits::ShortestPathResult<W>,
    algo_name: &str
) where 
    W: Clone + PartialEq + std::fmt::Debug + num_traits::Zero + num_traits::Float
{
    // Count reachable vertices
    let result_reachable = result.distances.iter().filter(|&d| d.is_some()).count();
    let baseline_reachable = baseline.distances.iter().filter(|&d| d.is_some()).count();
    
    // Check if all reachable vertices match
    if result_reachable == baseline_reachable {
        println!("✓ {} correctly found all {} reachable vertices", algo_name, baseline_reachable);
    } else {
        println!("❌ {} found {} reachable vertices, but baseline found {}", 
                algo_name, result_reachable, baseline_reachable);
    }
    
    // Check distances (sample a few for large graphs)
    let mut mismatches = 0;
    let mut checked = 0;
    let max_to_check = if result.distances.len() > 1_000_000 { 1000 } else { result.distances.len() };
    let step = result.distances.len() / max_to_check.max(1);
    
    for i in (0..result.distances.len()).step_by(step.max(1)) {
        if let (Some(rd), Some(bd)) = (&result.distances[i], &baseline.distances[i]) {
            checked += 1;
            
            // Allow for small floating point differences
            let diff = (*rd - *bd).abs();
            if diff > W::from(1e-6).unwrap() {
                mismatches += 1;
                if mismatches <= 3 {
                    println!("⚠️  Mismatch at vertex {}: {}: {:?}, Dijkstra: {:?}", 
                            i, algo_name, rd, bd);
                }
            }
        }
    }
    
    if mismatches == 0 {
        println!("✓ {} distances match baseline for all sampled vertices", algo_name);
    } else {
        let match_percentage = 100.0 * (checked - mismatches) as f64 / checked as f64;
        println!("⚠️  {} had {} distance mismatches with baseline ({:.1}% match)", 
                algo_name, mismatches, match_percentage);
    }
}
