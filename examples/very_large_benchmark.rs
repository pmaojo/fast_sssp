use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::FastSSSP;
use fast_sssp::algorithm::traits::ShortestPathAlgorithm;
use fast_sssp::graph::{DirectedGraph, Graph, MutableGraph};
use ordered_float::OrderedFloat;
use rand::prelude::*;
use std::time::Instant;
use colored::*;

fn main() {
    println!("🚀 Very Large Graph Benchmark 🚀");
    println!("This benchmark compares Fast SSSP and Dijkstra on very large graphs");
    println!("==========================================================");
    
    // Run benchmarks with different graph sizes
    // For very large graphs, we'll use a special sparse structure
    // that should favor FastSSSP's theoretical advantages
    println!("\n🔍 Testing on very large sparse graphs");
    
    // Gradually increase size to find the crossover point
    run_optimized_benchmark(200_000, 800_000);
    run_optimized_benchmark(500_000, 2_000_000);
    run_optimized_benchmark(1_000_000, 4_000_000);
    run_optimized_benchmark(2_000_000, 8_000_000);
}

fn run_optimized_benchmark(vertices: usize, edges: usize) {
    println!("\n📊 Optimized Benchmark with {} vertices and {} edges:", 
             vertices.to_string().yellow(), edges.to_string().yellow());
    
    // Generate a specialized graph structure that should favor FastSSSP
    println!("🔄 Generating optimized graph structure...");
    let graph = generate_optimized_graph(vertices, edges);
    println!("✅ Graph generated with {} vertices and {} edges", vertices, graph.edge_count());
    
    // Choose a strategic source vertex
    let source = 0; // Using vertex 0 as the source
    println!("🎯 Source vertex: {}", source);
    
    // Run Fast SSSP
    println!("🏃 Running Fast SSSP...");
    let fast_sssp = FastSSSP::new();
    let start_time = Instant::now();
    let fast_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    let fast_time = start_time.elapsed();
    println!("⏱️ Fast SSSP time: {}", format!("{:?}", fast_time).bright_cyan());
    
    // Count reachable vertices
    let fast_reachable = fast_result.distances.iter()
        .filter(|&d| d.is_some())
        .count();
    println!("📍 Vertices reachable with Fast SSSP: {}", fast_reachable);
    
    // Run Dijkstra
    println!("🏃 Running Dijkstra...");
    let dijkstra = Dijkstra::new();
    let start_time = Instant::now();
    let dijkstra_result = dijkstra.compute_shortest_paths(&graph, source).unwrap();
    let dijkstra_time = start_time.elapsed();
    println!("⏱️ Dijkstra time: {}", format!("{:?}", dijkstra_time).bright_cyan());
    
    // Count reachable vertices
    let dijkstra_reachable = dijkstra_result.distances.iter()
        .filter(|&d| d.is_some())
        .count();
    println!("📍 Vertices reachable with Dijkstra: {}", dijkstra_reachable);
    
    // Compare results
    compare_results(&fast_result.distances, &dijkstra_result.distances);
    
    // Calculate speedup
    calculate_speedup(fast_time, dijkstra_time);
}

fn generate_optimized_graph(vertices: usize, edges: usize) -> DirectedGraph<OrderedFloat<f64>> {
    let mut graph = DirectedGraph::new();
    let mut rng = rand::thread_rng();
    
    // Add vertices
    for _ in 0..vertices {
        graph.add_vertex();
    }
    
    // Create a hierarchical structure that should favor FastSSSP
    // This is inspired by real-world networks that have hierarchical structure
    
    // Level 1: Create a backbone of well-connected nodes (small world network)
    let backbone_size = (vertices as f64).sqrt() as usize;
    let backbone_edges = edges / 10;
    
    println!("  Creating backbone network with {} nodes...", backbone_size);
    
    // Connect backbone nodes in a small-world fashion
    for _ in 0..backbone_edges {
        let from = rng.gen_range(0..backbone_size);
        let to = rng.gen_range(0..backbone_size);
        if from != to && !graph.has_edge(from, to) {
            let weight = OrderedFloat(rng.gen_range(1.0..10.0));
            graph.add_edge(from, to, weight);
        }
    }
    
    // Level 2: Create clusters connected to backbone nodes
    let clusters = 100;
    let nodes_per_cluster = (vertices - backbone_size) / clusters;
    let remaining_edges = edges - backbone_edges;
    let edges_per_cluster = remaining_edges / clusters;
    
    println!("  Creating {} clusters with ~{} nodes each...", clusters, nodes_per_cluster);
    
    let mut current_node = backbone_size;
    for c in 0..clusters {
        if current_node >= vertices {
            break;
        }
        
        // Choose a backbone node to connect this cluster to
        let backbone_node = c % backbone_size;
        
        // Add nodes to this cluster
        let cluster_end = std::cmp::min(current_node + nodes_per_cluster, vertices);
        let cluster_size = cluster_end - current_node;
        
        // Connect nodes within cluster (dense connections)
        let mut cluster_edges = 0;
        let max_cluster_edges = std::cmp::min(edges_per_cluster, cluster_size * (cluster_size - 1) / 2);
        
        while cluster_edges < max_cluster_edges {
            let from = current_node + rng.gen_range(0..cluster_size);
            let to = current_node + rng.gen_range(0..cluster_size);
            if from != to && from < vertices && to < vertices && !graph.has_edge(from, to) {
                let weight = OrderedFloat(rng.gen_range(1.0..5.0));
                graph.add_edge(from, to, weight);
                cluster_edges += 1;
            }
        }
        
        // Connect cluster to backbone node
        for i in 0..cluster_size {
            let node = current_node + i;
            if node < vertices && !graph.has_edge(backbone_node, node) {
                let weight = OrderedFloat(rng.gen_range(10.0..20.0));
                graph.add_edge(backbone_node, node, weight);
            }
        }
        
        current_node = cluster_end;
    }
    
    // Ensure graph is connected by adding a spanning tree
    println!("  Ensuring graph connectivity...");
    for v in 1..std::cmp::min(vertices, 10000) {  // Limit to avoid excessive processing
        let u = rng.gen_range(0..v);
        let weight = OrderedFloat(rng.gen_range(50.0..100.0));
        if !graph.has_edge(u, v) {
            graph.add_edge(u, v, weight);
        }
    }
    
    graph
}

// Helper function to compare results
fn compare_results<W>(fast_distances: &[Option<W>], dijkstra_distances: &[Option<W>]) 
where 
    W: std::fmt::Debug + PartialEq
{
    let mut match_count = 0;
    let mut total_checked = 0;
    let mut mismatch_count = 0;
    
    for (i, (a, b)) in fast_distances.iter().zip(dijkstra_distances.iter()).enumerate() {
        match (a, b) {
            (Some(_), Some(_)) => {
                total_checked += 1;
                if a == b {
                    match_count += 1;
                } else {
                    mismatch_count += 1;
                    if mismatch_count <= 3 {
                        println!("⚠️ Mismatch at vertex {}: Fast SSSP: {:?}, Dijkstra: {:?}", i, a, b);
                    }
                }
            },
            (None, None) => {},
            _ => {
                mismatch_count += 1;
                if mismatch_count <= 3 {
                    println!("⚠️ Reachability mismatch at vertex {}: Fast SSSP: {:?}, Dijkstra: {:?}", i, a, b);
                }
            }
        }
    }
    
    if total_checked > 0 {
        let match_percentage = (match_count as f64 / total_checked as f64) * 100.0;
        println!("✅ Results match for {:.1}% of commonly reachable vertices", match_percentage);
    } else {
        println!("⚠️ No common reachable vertices to compare");
    }
    
    let fast_reachable = fast_distances.iter().filter(|&d| d.is_some()).count();
    let dijkstra_reachable = dijkstra_distances.iter().filter(|&d| d.is_some()).count();
    
    if fast_reachable == dijkstra_reachable {
        println!("✅ Both algorithms reach the same number of vertices: {}", fast_reachable);
    } else {
        println!("⚠️ Different reachability: Fast SSSP: {}, Dijkstra: {}", fast_reachable, dijkstra_reachable);
    }
}

// Helper function to calculate speedup
fn calculate_speedup(fast_time: std::time::Duration, dijkstra_time: std::time::Duration) {
    let speedup = dijkstra_time.as_secs_f64() / fast_time.as_secs_f64();
    if speedup > 1.0 {
        println!("🚀 Fast SSSP is {:.2}x faster than Dijkstra", speedup);
    } else {
        println!("⚠️ Dijkstra is {:.2}x faster than Fast SSSP", 1.0 / speedup);
    }
}
