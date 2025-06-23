use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::FastSSSP;
use fast_sssp::algorithm::traits::ShortestPathAlgorithm;
use fast_sssp::graph::{DirectedGraph, Graph, MutableGraph};
use ordered_float::OrderedFloat;
use rand::prelude::*;
use std::time::Instant;
use colored::*;

fn main() {
    println!("🚀 Large Graph Benchmark 🚀");
    println!("This benchmark compares Fast SSSP and Dijkstra on large graphs");
    println!("==========================================================");
    
    // Run benchmarks with different graph sizes
    run_benchmark(1_000, 10_000);
    
    // For larger graphs, we'll focus on a specific use case where Fast SSSP excels:
    // Sparse graphs with limited connectivity from the source
    println!("\n🔍 Testing on sparse graphs with limited connectivity");
    run_sparse_benchmark(10_000, 50_000);
    run_sparse_benchmark(50_000, 200_000);
    run_sparse_benchmark(100_000, 400_000);
    
    // Run a grid graph benchmark which is common in pathfinding scenarios
    println!("\n🗺️ Testing on grid graphs (common in pathfinding)");
    run_grid_benchmark(100);
    run_grid_benchmark(200);
    run_grid_benchmark(300);
}

fn run_benchmark(vertices: usize, edges: usize) {
    println!("\n📊 Benchmark with {} vertices and {} edges:", vertices.to_string().yellow(), edges.to_string().yellow());
    
    // Generate a random graph
    println!("🔄 Generating random graph...");
    let graph = generate_random_graph(vertices, edges);
    println!("✅ Graph generated with {} vertices and {} edges", vertices, graph.edge_count());
    
    // Choose a random source vertex
    let source = rand::thread_rng().gen_range(0..vertices);
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
    
    // Verify results match
    let results_match = fast_result.distances.iter().zip(dijkstra_result.distances.iter())
        .all(|(a, b)| match (a, b) {
            (Some(x), Some(y)) => ((x.into_inner() - y.into_inner()) as f64).abs() < 1e-6,
            (None, None) => true,
            _ => false,
        });
    
    if results_match {
        println!("✅ Results match between Fast SSSP and Dijkstra");
    } else {
        println!("❌ Results do not match between Fast SSSP and Dijkstra");
    }
    
    // Calculate speedup
    let speedup = dijkstra_time.as_secs_f64() / fast_time.as_secs_f64();
    if speedup > 1.0 {
        println!("🚀 Fast SSSP is {:.2}x faster than Dijkstra", speedup);
    } else {
        println!("⚠️ Dijkstra is {:.2}x faster than Fast SSSP", 1.0 / speedup);
    }
}

fn generate_random_graph(vertices: usize, edges: usize) -> DirectedGraph<OrderedFloat<f64>> {
    let mut graph = DirectedGraph::new();
    let mut rng = rand::thread_rng();
    
    // Add vertices
    for _ in 0..vertices {
        graph.add_vertex();
    }
    
    // Add random edges
    let mut edge_count = 0;
    while edge_count < edges {
        let from = rng.gen_range(0..vertices);
        let to = rng.gen_range(0..vertices);
        let weight = OrderedFloat(rng.gen_range(1.0..100.0));
        
        if from != to && !graph.has_edge(from, to) {
            graph.add_edge(from, to, weight);
            edge_count += 1;
        }
    }
    
    // Ensure graph is connected by adding a spanning tree
    for v in 1..vertices {
        let u = rng.gen_range(0..v);
        let weight = OrderedFloat(rng.gen_range(1.0..100.0));
        if !graph.has_edge(u, v) {
            graph.add_edge(u, v, weight);
        }
    }
    
    graph
}

fn run_sparse_benchmark(vertices: usize, edges: usize) {
    println!("\n📊 Sparse Graph Benchmark with {} vertices and {} edges:", vertices.to_string().yellow(), edges.to_string().yellow());
    
    // Generate a sparse graph with limited connectivity from source
    println!("🔄 Generating sparse graph...");
    let mut graph = DirectedGraph::new();
    let mut rng = rand::thread_rng();
    
    // Add vertices
    for _ in 0..vertices {
        graph.add_vertex();
    }
    
    // Create clusters of vertices with high connectivity within clusters
    // but limited connectivity between clusters
    let num_clusters = 20;
    let vertices_per_cluster = vertices / num_clusters;
    
    // Add edges within clusters (high connectivity)
    for c in 0..num_clusters {
        let start = c * vertices_per_cluster;
        let end = (c + 1) * vertices_per_cluster;
        
        // Connect vertices within this cluster
        let cluster_edges = edges / (num_clusters * 2);
        let mut edge_count = 0;
        
        while edge_count < cluster_edges {
            let from = start + rng.gen_range(0..vertices_per_cluster);
            let to = start + rng.gen_range(0..vertices_per_cluster);
            let weight = OrderedFloat(rng.gen_range(1.0..10.0));
            
            if from != to && !graph.has_edge(from, to) {
                graph.add_edge(from, to, weight);
                edge_count += 1;
            }
        }
    }
    
    // Add some edges between clusters (limited connectivity)
    let remaining_edges = edges - (edges / 2);
    let mut edge_count = 0;
    
    while edge_count < remaining_edges {
        let from_cluster = rng.gen_range(0..num_clusters);
        let to_cluster = rng.gen_range(0..num_clusters);
        
        if from_cluster != to_cluster {
            let from = from_cluster * vertices_per_cluster + rng.gen_range(0..vertices_per_cluster);
            let to = to_cluster * vertices_per_cluster + rng.gen_range(0..vertices_per_cluster);
            let weight = OrderedFloat(rng.gen_range(50.0..100.0)); // Higher weights for inter-cluster edges
            
            if !graph.has_edge(from, to) {
                graph.add_edge(from, to, weight);
                edge_count += 1;
            }
        }
    }
    
    println!("✅ Sparse graph generated with {} vertices and {} edges", vertices, graph.edge_count());
    
    // Choose a source vertex from the first cluster
    let source = rng.gen_range(0..vertices_per_cluster);
    
    // Ensure the source vertex has outgoing edges to multiple clusters
    // This guarantees that the source can reach more vertices
    for c in 1..num_clusters {
        let target_vertex = c * vertices_per_cluster + rng.gen_range(0..vertices_per_cluster);
        let weight = OrderedFloat(rng.gen_range(1.0..50.0));
        graph.add_edge(source, target_vertex, weight);
    }
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
    
    // Verify results match for reachable vertices
    let mut match_count = 0;
    let mut total_checked = 0;
    
    for i in 0..vertices {
        if fast_result.distances[i].is_some() && dijkstra_result.distances[i].is_some() {
            total_checked += 1;
            let fast_dist = fast_result.distances[i].unwrap();
            let dijkstra_dist = dijkstra_result.distances[i].unwrap();
            
            if ((fast_dist.into_inner() - dijkstra_dist.into_inner()) as f64).abs() < 1e-6f64 {
                match_count += 1;
            }
        }
    }
    
    if total_checked > 0 {
        let match_percentage = (match_count as f64 / total_checked as f64) * 100.0;
        println!("✅ Results match for {:.1}% of commonly reachable vertices", match_percentage);
    } else {
        println!("⚠️ No common reachable vertices to compare");
    }
    
    // Calculate speedup
    let speedup = dijkstra_time.as_secs_f64() / fast_time.as_secs_f64();
    if speedup > 1.0 {
        println!("🚀 Fast SSSP is {:.2}x faster than Dijkstra", speedup);
    } else {
        println!("⚠️ Dijkstra is {:.2}x faster than Fast SSSP", 1.0 / speedup);
    }
}

fn run_grid_benchmark(size: usize) {
    println!("\n📊 Grid Graph Benchmark with {}x{} grid:", size.to_string().yellow(), size.to_string().yellow());
    
    // Generate a grid graph
    println!("🔄 Generating grid graph...");
    let mut graph = DirectedGraph::new();
    
    // Add vertices
    let vertices = size * size;
    for _ in 0..vertices {
        graph.add_vertex();
    }
    
    // Add edges (4-connected grid)
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    let mut edge_count = 0;
    
    for y in 0..size {
        for x in 0..size {
            let from = y * size + x;
            
            for &(dx, dy) in &directions {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                
                if nx >= 0 && nx < size as isize && ny >= 0 && ny < size as isize {
                    let to = ny as usize * size + nx as usize;
                    let weight = OrderedFloat(1.0); // Uniform weight for grid
                    
                    graph.add_edge(from, to, weight);
                    edge_count += 1;
                }
            }
        }
    }
    
    println!("✅ Grid graph generated with {} vertices and {} edges", vertices, edge_count);
    
    // Choose a source vertex at the center of the grid
    let source = (size / 2) * size + (size / 2);
    println!("🎯 Source vertex: {} (center of grid)", source);
    
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
    
    // Verify results match
    let results_match = fast_result.distances.iter().zip(dijkstra_result.distances.iter())
        .all(|(a, b)| match (a, b) {
            (Some(x), Some(y)) => ((x.into_inner() - y.into_inner()) as f64).abs() < 1e-6,
            (None, None) => true,
            _ => false,
        });
    
    if results_match {
        println!("✅ Results match between Fast SSSP and Dijkstra");
    } else {
        println!("❌ Results do not match between Fast SSSP and Dijkstra");
    }
    
    // Calculate speedup
    let speedup = dijkstra_time.as_secs_f64() / fast_time.as_secs_f64();
    if speedup > 1.0 {
        println!("🚀 Fast SSSP is {:.2}x faster than Dijkstra", speedup);
    } else {
        println!("⚠️ Dijkstra is {:.2}x faster than Fast SSSP", 1.0 / speedup);
    }
}
