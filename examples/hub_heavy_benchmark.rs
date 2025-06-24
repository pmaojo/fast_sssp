use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::{FastSSSP, DegreeMode};
use fast_sssp::algorithm::traits::ShortestPathAlgorithm;
use fast_sssp::graph::{DirectedGraph, Graph, MutableGraph};
use ordered_float::OrderedFloat;
use rand::prelude::*;
use std::time::Instant;
use colored::*;
use num_traits::Zero;

fn main() {
    println!("🚀 Hub-Heavy Graph Benchmark 🚀");
    println!("This benchmark compares different degree modes on graphs with significant hubs");
    println!("==========================================================");
    
    // Run benchmarks with different graph sizes
    run_hub_benchmark(100_000, 500_000, 20, 1000);
    run_hub_benchmark(500_000, 2_000_000, 50, 2000);
    run_hub_benchmark(1_000_000, 4_000_000, 100, 5000);
}

fn run_hub_benchmark(vertices: usize, edges: usize, num_hubs: usize, hub_degree: usize) {
    println!("\n📊 Hub-Heavy Benchmark with {} vertices, {} edges, {} hubs with ~{} connections each:", 
             vertices.to_string().yellow(), 
             edges.to_string().yellow(),
             num_hubs.to_string().yellow(),
             hub_degree.to_string().yellow());
    
    // Generate a hub-heavy graph
    println!("🔄 Generating hub-heavy graph...");
    let graph = generate_hub_graph(vertices, edges, num_hubs, hub_degree);
    println!("✅ Graph generated with {} vertices and {} edges", vertices, graph.edge_count());
    
    // Choose a random source vertex
    let source = rand::thread_rng().gen_range(0..vertices);
    println!("🎯 Source vertex: {}", source);
    
    // Run Dijkstra (baseline)
    println!("🏃 Running Dijkstra (baseline)...");
    let dijkstra = Dijkstra::new();
    let start_time = Instant::now();
    let dijkstra_result = dijkstra.compute_shortest_paths(&graph, source).unwrap();
    let dijkstra_time = start_time.elapsed();
    println!("⏱️ Dijkstra time: {}", format!("{:?}", dijkstra_time).bright_cyan());
    
    // Count reachable vertices
    let dijkstra_reachable = dijkstra_result.distances.iter()
        .filter(|&d| d.is_some())
        .count();
    println!("📍 Reachable vertices: {}", dijkstra_reachable);
    
    // Run FastSSSP with ForceConst mode
    println!("🏃 Running FastSSSP with ForceConst mode...");
    let fast_sssp = FastSSSP::new_with_mode(DegreeMode::ForceConst);
    let start_time = Instant::now();
    let classic_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    let classic_time = start_time.elapsed();
    println!("⏱️ Time: {:?}", classic_time);
    println!("  Slowdown vs Dijkstra: {:.2}x", classic_time.as_secs_f64() / dijkstra_time.as_secs_f64());
    
    // Run FastSSSP with Auto mode (HubSplit)
    println!("🏃 Running FastSSSP with Auto mode (delta=64)...");
    let fast_sssp = FastSSSP::new_with_mode(DegreeMode::Auto { delta: 64 });
    let start_time = Instant::now();
    let auto_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    let auto_time = start_time.elapsed();
    println!("⏱️ Time: {:?}", auto_time);
    println!("  Speedup vs classic: {:.2}x", classic_time.as_secs_f64() / auto_time.as_secs_f64());
    println!("  Ratio vs Dijkstra: {:.2}x", auto_time.as_secs_f64() / dijkstra_time.as_secs_f64());
    
    // Run FastSSSP with None mode
    println!("🏃 Running FastSSSP with None mode...");
    let fast_sssp = FastSSSP::new_with_mode(DegreeMode::None);
    let start_time = Instant::now();
    let none_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    let none_time = start_time.elapsed();
    println!("⏱️ Time: {:?}", none_time);
    println!("  Speedup vs classic: {:.2}x", classic_time.as_secs_f64() / none_time.as_secs_f64());
    println!("  Ratio vs Dijkstra: {:.2}x", none_time.as_secs_f64() / dijkstra_time.as_secs_f64());
    
    // Verify results
    verify_results(&classic_result, &dijkstra_result, "classic mode");
    verify_results(&auto_result, &dijkstra_result, "auto mode");
    verify_results(&none_result, &dijkstra_result, "none mode");
}

fn generate_hub_graph(vertices: usize, edges: usize, num_hubs: usize, hub_degree: usize) -> DirectedGraph<OrderedFloat<f64>> {
    let mut graph = DirectedGraph::new();
    let mut rng = rand::thread_rng();
    
    // Add vertices
    for _ in 0..vertices {
        graph.add_vertex();
    }
    
    // Select hub vertices
    let hub_vertices: Vec<usize> = (0..vertices).choose_multiple(&mut rng, num_hubs);
    
    // Connect hubs with high degree
    let mut edge_count = 0;
    for &hub in &hub_vertices {
        let connections = hub_degree.min(vertices - 1);
        let targets: Vec<usize> = (0..vertices)
            .filter(|&v| v != hub)
            .choose_multiple(&mut rng, connections);
        
        for target in targets {
            let weight = OrderedFloat(rng.gen_range(1.0..100.0));
            if !graph.has_edge(hub, target) {
                graph.add_edge(hub, target, weight);
                edge_count += 1;
            }
        }
    }
    
    // Add remaining random edges
    while edge_count < edges {
        let from = rng.gen_range(0..vertices);
        let to = rng.gen_range(0..vertices);
        let weight = OrderedFloat(rng.gen_range(1.0..100.0));
        
        if from != to && !graph.has_edge(from, to) {
            graph.add_edge(from, to, weight);
            edge_count += 1;
        }
    }
    
    graph
}

fn verify_results<W>(result: &fast_sssp::algorithm::traits::ShortestPathResult<W>, 
                    baseline: &fast_sssp::algorithm::traits::ShortestPathResult<W>,
                    mode_name: &str)
where
    W: Clone + PartialEq + std::fmt::Debug + Zero + num_traits::Float,
{
    // Count reachable vertices
    let result_reachable = result.distances.iter().filter(|&d| d.is_some()).count();
    let baseline_reachable = baseline.distances.iter().filter(|&d| d.is_some()).count();
    
    // Check if all reachable vertices match
    if result_reachable == baseline_reachable {
        println!("  ✓ {} correctly found all {} reachable vertices", mode_name, baseline_reachable);
    } else {
        println!("  ❌ {} found {} reachable vertices, but baseline found {}", 
                mode_name, result_reachable, baseline_reachable);
    }
    
    // Check distances
    let mut mismatches = 0;
    let mut checked = 0;
    
    for (i, (result_dist, baseline_dist)) in result.distances.iter().zip(baseline.distances.iter()).enumerate() {
        if let (Some(rd), Some(bd)) = (result_dist, baseline_dist) {
            checked += 1;
            if rd != bd {
                mismatches += 1;
                if mismatches <= 3 {
                    println!("  ⚠️ Mismatch at vertex {}: {}: {:?}, Dijkstra: {:?}", 
                            i, mode_name, rd, bd);
                }
            }
        }
    }
    
    if mismatches == 0 {
        println!("  ✓ {} distances match baseline for all reachable vertices", mode_name);
    } else {
        let match_percentage = 100.0 * (checked - mismatches) as f64 / checked as f64;
        println!("  ⚠️ {} had {} distance mismatches with baseline ({:.1}% match)", 
                mode_name, mismatches, match_percentage);
    }
}
