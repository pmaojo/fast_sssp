use std::time::{Duration, Instant};
use rand::Rng;
use ordered_float::OrderedFloat;
use fast_sssp::algorithm::{ShortestPathAlgorithm, dijkstra::Dijkstra, fast_sssp::FastSSSP};
use fast_sssp::graph::{DirectedGraph, Graph, MutableGraph};

// Function to generate a random directed graph with specified parameters
fn generate_random_graph(num_vertices: usize, edge_factor: f64) -> DirectedGraph<OrderedFloat<f64>> {
    let mut graph = DirectedGraph::with_capacity(num_vertices);
    let mut rng = rand::thread_rng();
    
    // Approximately edge_factor * n edges
    let num_edges = (edge_factor * num_vertices as f64) as usize;
    
    for _ in 0..num_edges {
        let u = rng.gen_range(0..num_vertices);
        let v = rng.gen_range(0..num_vertices);
        // Avoid self-loops and ensure positive weights
        if u != v {
            let weight = OrderedFloat(rng.gen_range(1.0..100.0));
            graph.add_edge(u, v, weight);
        }
    }
    
    graph
}

// Function to benchmark an algorithm on a graph
fn benchmark_algorithm<A>(name: &str, algorithm: &A, graph: &DirectedGraph<OrderedFloat<f64>>, source: usize) -> Duration 
where
    A: ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>
{
    println!("Running {} on graph with {} vertices...", name, graph.vertex_count());
    
    let start = Instant::now();
    let result = algorithm.compute_shortest_paths(graph, source).unwrap();
    let duration = start.elapsed();
    
    // Count reachable vertices
    let reachable = result.distances.iter().filter(|d| d.is_some()).count();
    println!("  - Found {} reachable vertices in {:?}", reachable, duration);
    
    duration
}

fn main() {
    // Define graph sizes to test
    let graph_sizes = vec![
        // Small graphs
        1_000, 
        10_000, 
        // Medium graphs - might start to see benefits
        50_000,
        // Large graphs - should see definite benefits
        100_000, 
        200_000,
        // Very large graphs - if memory allows
        500_000, 
    ];
    
    // Edge factor: average number of edges per vertex
    let edge_factor = 2.0;
    
    println!("=====================================================");
    println!("Benchmark: Dijkstra vs FastSSSP");
    println!("Edge factor: {} edges per vertex (on average)", edge_factor);
    println!("=====================================================");
    
    // Create algorithm instances
    let dijkstra = Dijkstra::new();
    
    // Create FastSSSP with different thresholds for comparison
    let fast_sssp_default = FastSSSP::new();
    let fast_sssp_always = FastSSSP::new().with_vertex_threshold(0);
    
    // Results storage
    let mut results = Vec::new();
    
    // Run benchmarks for each graph size
    for &size in &graph_sizes {
        println!("\nGenerating random graph with {} vertices...", size);
        let graph = generate_random_graph(size, edge_factor);
        let source = 0; // Use vertex 0 as source
        
        println!("Graph has {} vertices and ~{} edges", graph.vertex_count(), (size as f64 * edge_factor) as usize);
        
        // Run benchmarks
        let dijkstra_time = benchmark_algorithm("Dijkstra", &dijkstra, &graph, source);
        let fastssp_default_time = benchmark_algorithm("FastSSSP (default)", &fast_sssp_default, &graph, source);
        let fastssp_always_time = benchmark_algorithm("FastSSSP (always)", &fast_sssp_always, &graph, source);
        
        // Store results
        results.push((size, dijkstra_time, fastssp_default_time, fastssp_always_time));
        
        // Print comparison
        let speedup_default = dijkstra_time.as_secs_f64() / fastssp_default_time.as_secs_f64();
        let speedup_always = dijkstra_time.as_secs_f64() / fastssp_always_time.as_secs_f64();
        
        println!("Speedup - FastSSSP (default) vs Dijkstra: {:.2}x", speedup_default);
        println!("Speedup - FastSSSP (always) vs Dijkstra: {:.2}x", speedup_always);
    }
    
    // Print summary table
    println!("\n=====================================================");
    println!("Summary of Results");
    println!("=====================================================");
    println!("{:<10} | {:<15} | {:<15} | {:<15} | {:<10} | {:<10}", 
             "Vertices", "Dijkstra (ms)", "FastSSP-Def (ms)", "FastSSP-Alw (ms)", "SpeedUp-Def", "SpeedUp-Alw");
    println!("-----------------------------------------------------");
    
    for (size, dijkstra_time, fastssp_default_time, fastssp_always_time) in &results {
        let speedup_default = dijkstra_time.as_secs_f64() / fastssp_default_time.as_secs_f64();
        let speedup_always = dijkstra_time.as_secs_f64() / fastssp_always_time.as_secs_f64();
        
        println!("{:<10} | {:<15.2} | {:<15.2} | {:<15.2} | {:<10.2} | {:<10.2}", 
                 size,
                 dijkstra_time.as_millis(),
                 fastssp_default_time.as_millis(),
                 fastssp_always_time.as_millis(),
                 speedup_default,
                 speedup_always);
    }
}
