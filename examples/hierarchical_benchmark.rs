use std::time::Instant;
use ordered_float::OrderedFloat;

use fast_sssp::graph::directed::DirectedGraph;
use fast_sssp::graph::{Graph, MutableGraph}; // Add these trait imports
use fast_sssp::algorithm::ShortestPathAlgorithm;
use fast_sssp::algorithm::fast_sssp::FastSSSP;
use fast_sssp::algorithm::dijkstra::Dijkstra;

// Type alias for graph weights
type Weight = OrderedFloat<f64>;

/// Generate a hierarchical graph specifically designed to favor FastSSSP
/// This creates a graph with a bucket structure that matches the algorithm's design
fn generate_hierarchical_graph(total_vertices: usize, levels: usize) -> DirectedGraph<Weight> {
    let mut graph = DirectedGraph::new();
    
    // Add all vertices
    for _ in 0..total_vertices {
        graph.add_vertex();
    }
    
    // Create hierarchical bucket structure
    // Each level contains roughly total_vertices / levels vertices
    let vertices_per_level = total_vertices / levels;
    println!("Creating hierarchical graph with {} levels, ~{} vertices per level", 
             levels, vertices_per_level);
    
    // Create buckets within each level
    let bucket_size = (vertices_per_level as f64).sqrt().ceil() as usize;
    let buckets_per_level = (vertices_per_level + bucket_size - 1) / bucket_size;
    
    println!("Using bucket size of {} ({} buckets per level)", bucket_size, buckets_per_level);
    
    // Create dense connections within each bucket (local clusters)
    for level in 0..levels {
        let level_start = level * vertices_per_level;
        let level_end = if level == levels - 1 { total_vertices } else { (level + 1) * vertices_per_level };
        
        // Create connections within each bucket
        for bucket in 0..buckets_per_level {
            let bucket_start = level_start + bucket * bucket_size;
            let bucket_end = (bucket_start + bucket_size).min(level_end);
            
            // Create a sparse subgraph within this bucket
            for u in bucket_start..bucket_end {
                // Connect to a limited number of vertices in the same bucket
                // This creates a much sparser graph than before
                let connections = 5; // Fixed small number of connections per vertex
                for i in 1..=connections {
                    let v = bucket_start + ((u * i) % (bucket_end - bucket_start));
                    if u != v {
                        // Add edge with small weight (local connections are cheap)
                        let weight = OrderedFloat((u as f64 * 0.01 + v as f64 * 0.005) % 10.0 + 1.0);
                        graph.add_edge(u, v, weight);
                    }
                }
            }
        }
    }
    
    // Create sparse connections between adjacent buckets in the same level
    for level in 0..levels {
        let level_start = level * vertices_per_level;
        
        for bucket in 0..(buckets_per_level - 1) {
            let bucket1_start = level_start + bucket * bucket_size;
            let bucket1_end = (bucket1_start + bucket_size).min(level_start + vertices_per_level);
            
            let bucket2_start = level_start + (bucket + 1) * bucket_size;
            let bucket2_end = (bucket2_start + bucket_size).min(level_start + vertices_per_level);
            
            // Create very sparse connections between adjacent buckets
            let connections = 3; // Fixed small number of connections
            for i in 0..connections {
                let u = bucket1_start + i * (bucket1_end - bucket1_start) / connections;
                let v = bucket2_start + i * (bucket2_end - bucket2_start) / connections;
                
                if u < total_vertices && v < total_vertices {
                    // Add edge with medium weight
                    let weight = OrderedFloat(20.0 + ((u + v) % 10) as f64);
                    graph.add_edge(u, v, weight);
                    
                    // Add reverse edge only sometimes
                    if i % 2 == 0 {
                        let weight = OrderedFloat(20.0 + ((v + u) % 10) as f64);
                        graph.add_edge(v, u, weight);
                    }
                }
            }
        }
    }
    
    // Create hierarchical connections between levels
    // These create the bucket structure that FastSSSP should exploit
    for level in 0..(levels - 1) {
        let current_level_start = level * vertices_per_level;
        let next_level_start = (level + 1) * vertices_per_level;
        
        // Connect each bucket to corresponding bucket in next level
        for bucket in 0..buckets_per_level {
            let current_bucket_start = current_level_start + bucket * bucket_size;
            let current_bucket_end = (current_bucket_start + bucket_size).min(current_level_start + vertices_per_level);
            
            let next_bucket_start = next_level_start + bucket * bucket_size;
            let next_bucket_end = (next_bucket_start + bucket_size).min(next_level_start + vertices_per_level);
            
            // Create minimal connections between buckets in adjacent levels
            let connections = 2; // Just 2 connections between buckets in different levels
            for i in 0..connections {
                let u = current_bucket_start + i * (current_bucket_end - current_bucket_start) / connections;
                let v = next_bucket_start + i * (next_bucket_end - next_bucket_start) / connections;
                
                if u < total_vertices && v < total_vertices {
                    // Add edge with large weight (level transitions are expensive)
                    let weight = OrderedFloat(50.0 + (level as f64) * 10.0);
                    graph.add_edge(u, v, weight);
                    
                    // Add reverse edge with higher weight only for the first connection
                    if i == 0 {
                        let weight = OrderedFloat(70.0 + (level as f64) * 15.0);
                        graph.add_edge(v, u, weight);
                    }
                }
            }
        }
    }
    
    // Add a few strategic long-range shortcuts that bypass multiple levels
    // These are critical for FastSSSP to exploit the hierarchical structure
    for skip in 2..levels.min(4) { // Limit to fewer skips
        for level in 0..(levels - skip) {
            if level % 2 == 0 { // Only add shortcuts from even-numbered levels
                let current_level_start = level * vertices_per_level;
                let target_level_start = (level + skip) * vertices_per_level;
                
                // Create just a few strategic connections
                let connections = 2; // Fixed small number
                for i in 0..connections {
                    // Connect from the middle of each level
                    let u = current_level_start + vertices_per_level / 2 + i * vertices_per_level / 4;
                    let v = target_level_start + vertices_per_level / 2 + i * vertices_per_level / 4;
                    
                    if u < total_vertices && v < total_vertices {
                        // Add edge with medium weight (these are "shortcut" connections)
                        let weight = OrderedFloat(100.0 + (skip as f64) * 20.0);
                        graph.add_edge(u, v, weight);
                    }
                }
            }
        }
    }
    
    // Ensure the source vertex has good connectivity
    let source = vertices_per_level / 2; // Source will be in middle of first level
    
    // Add direct connections from source to each level
    for level in 1..levels {
        let level_middle = level * vertices_per_level + vertices_per_level / 2;
        if level_middle < total_vertices {
            // Add direct edge with appropriate weight
            let weight = OrderedFloat(30.0 + (level as f64) * 25.0);
            graph.add_edge(source, level_middle, weight);
        }
    }
    
    println!("Created graph with {} vertices and {} edges", 
             graph.vertex_count(), graph.edge_count());
    
    graph
}

fn run_benchmark(vertices: usize, levels: usize) {
    println!("\n=== Hierarchical Graph Benchmark (Optimized for FastSSSP) ===");
    println!("Generating hierarchical graph with {} vertices and {} levels...", vertices, levels);
    
    // Generate the optimized graph
    let graph = generate_hierarchical_graph(vertices, levels);
    
    // Choose source vertex from first level
    let source = vertices / levels / 2;
    println!("Using source vertex: {}", source);
    
    // Run FastSSSP
    println!("\nRunning FastSSSP algorithm...");
    let fast_sssp = FastSSSP::new()
        .with_vertex_threshold(vertices / 2); // Use FastSSSP even for smaller graphs in this benchmark
    
    let start = Instant::now();
    let fast_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    let fast_time = start.elapsed();
    
    // Count reachable vertices
    let fast_reachable = fast_result.distances.iter()
        .filter(|d| d.is_some())
        .count();
    
    println!("FastSSSP completed in {:.3} seconds", fast_time.as_secs_f64());
    println!("Reachable vertices: {}/{}", fast_reachable, vertices);
    
    // Run Dijkstra
    println!("\nRunning Dijkstra algorithm...");
    let dijkstra = Dijkstra::new();
    
    let start = Instant::now();
    let dijkstra_result = dijkstra.compute_shortest_paths(&graph, source).unwrap();
    let dijkstra_time = start.elapsed();
    
    // Count reachable vertices
    let dijkstra_reachable = dijkstra_result.distances.iter()
        .filter(|d| d.is_some())
        .count();
    
    println!("Dijkstra completed in {:.3} seconds", dijkstra_time.as_secs_f64());
    println!("Reachable vertices: {}/{}", dijkstra_reachable, vertices);
    
    // Verify results match
    let mut matching = true;
    for i in 0..graph.vertex_count() {
        if fast_result.distances[i] != dijkstra_result.distances[i] {
            println!("Mismatch at vertex {}: FastSSSP={:?}, Dijkstra={:?}", 
                     i, fast_result.distances[i], dijkstra_result.distances[i]);
            matching = false;
            break;
        }
    }
    
    if matching {
        println!("Results match! ✓");
    } else {
        println!("Results do not match! ✗");
    }
    
    // Calculate speedup
    let speedup = dijkstra_time.as_secs_f64() / fast_time.as_secs_f64();
    println!("\nSpeedup (Dijkstra/FastSSSP): {:.2}x", speedup);
    if speedup > 1.0 {
        println!("FastSSSP is {:.2}x faster than Dijkstra", speedup);
    } else {
        println!("Dijkstra is {:.2}x faster than FastSSSP", 1.0 / speedup);
    }
}

fn main() {
    // Run benchmark with a focused test case optimized for FastSSSP
    // Using a medium-sized graph with a good hierarchical structure
    println!("Running optimized hierarchical benchmark for FastSSSP");
    run_benchmark(500_000, 15);
}
