use std::time::Instant;
use rand::prelude::*;
use ordered_float::OrderedFloat;
use num_traits::Zero;
use fast_sssp::{
    graph::{DirectedGraph, Graph, MutableGraph, ToConstantDegree},
    algorithm::{ShortestPathAlgorithm, ShortestPathResult},
    algorithm::fast_sssp::{FastSSSP, DegreeMode},
    algorithm::dijkstra::Dijkstra,
};

// Use OrderedFloat<f64> as our weight type to satisfy Ord trait
type Weight = OrderedFloat<f64>;

fn main() {
    println!("Hub-Split Benchmark - Comparing different degree modes");
    println!("=====================================================");
    
    // Test with different graph types
    benchmark_random_graph(10_000, 50_000);
    benchmark_hub_graph(10_000, 50_000, 10);
    benchmark_grid_graph(100, 100);
    
    // Test with a larger graph to see if the pivot issue is fixed
    println!("\nLarge Graph Test (100,000 vertices)");
    println!("=================================");
    benchmark_hub_graph(100_000, 500_000, 20);
}

fn benchmark_random_graph(n: usize, m: usize) {
    println!("\nRandom Graph: {} vertices, {} edges", n, m);
    println!("--------------------------------------");
    
    // Generate random graph
    let mut graph = DirectedGraph::new();
    for _ in 0..n {
        graph.add_vertex();
    }
    
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..m {
        let u = rng.gen_range(0..n);
        let v = rng.gen_range(0..n);
        let w = OrderedFloat(rng.gen_range(1.0..100.0));
        graph.add_edge(u, v, w);
    }
    
    // Run benchmarks
    run_benchmarks(&graph);
}

fn benchmark_hub_graph(n: usize, m: usize, num_hubs: usize) {
    println!("\nHub Graph: {} vertices, {} edges, {} hubs", n, m, num_hubs);
    println!("----------------------------------------------");
    
    // Generate graph with hubs
    let mut graph = DirectedGraph::new();
    for _ in 0..n {
        graph.add_vertex();
    }
    
    let mut rng = StdRng::seed_from_u64(42);
    
    // Create hub vertices with high degree
    let hubs: Vec<usize> = (0..num_hubs).collect();
    
    // Connect regular vertices to hubs
    for _ in 0..m {
        let u = rng.gen_range(0..n);
        let hub_idx = rng.gen_range(0..num_hubs);
        let hub = hubs[hub_idx];
        let w = OrderedFloat(rng.gen_range(1.0..100.0));
        
        // Connect in both directions to create high in/out degree
        if rng.gen_bool(0.5) {
            graph.add_edge(u, hub, w);
        } else {
            graph.add_edge(hub, u, w);
        }
    }
    
    // Run benchmarks
    run_benchmarks(&graph);
}

fn benchmark_grid_graph(width: usize, height: usize) {
    let n = width * height;
    println!("\nGrid Graph: {}x{} ({} vertices)", width, height, n);
    println!("----------------------------------");
    
    // Generate grid graph
    let mut graph = DirectedGraph::new();
    for _ in 0..n {
        graph.add_vertex();
    }
    
    // Connect grid cells
    for y in 0..height {
        for x in 0..width {
            let v = y * width + x;
            
            // Connect to right neighbor
            if x + 1 < width {
                graph.add_edge(v, v + 1, OrderedFloat(1.0));
            }
            
            // Connect to bottom neighbor
            if y + 1 < height {
                graph.add_edge(v, v + width, OrderedFloat(1.0));
            }
            
            // Connect to left neighbor
            if x > 0 {
                graph.add_edge(v, v - 1, OrderedFloat(1.0));
            }
            
            // Connect to top neighbor
            if y > 0 {
                graph.add_edge(v, v - width, OrderedFloat(1.0));
            }
        }
    }
    
    // Run benchmarks
    run_benchmarks(&graph);
}

fn run_benchmarks<G>(graph: &G) 
where 
    G: Graph<Weight> + Clone + MutableGraph<Weight> + ToConstantDegree<Weight>,
{
    let source = 0;
    
    // Get baseline with Dijkstra
    println!("Running Dijkstra (baseline)...");
    let dijkstra = Dijkstra::new();
    let start = Instant::now();
    let dijkstra_result = dijkstra.compute_shortest_paths(graph, source).unwrap();
    let dijkstra_time = start.elapsed();
    let dijkstra_reachable = count_reachable(&dijkstra_result);
    println!("  Time: {:.2?}, Reachable vertices: {}", dijkstra_time, dijkstra_reachable);
    
    // Run FastSSSP with classic constant degree transformation
    println!("Running FastSSSP with ForceConst mode...");
    let fast_sssp_classic = FastSSSP::new_with_mode(DegreeMode::ForceConst);
    let start = Instant::now();
    let classic_result = fast_sssp_classic.compute_shortest_paths(graph, source).unwrap();
    let classic_time = start.elapsed();
    let classic_reachable = count_reachable(&classic_result);
    println!("  Time: {:.2?}, Reachable vertices: {}", classic_time, classic_reachable);
    println!("  Slowdown vs Dijkstra: {:.2}x", classic_time.as_secs_f64() / dijkstra_time.as_secs_f64());
    
    // Run FastSSSP with hub-split transformation
    println!("Running FastSSSP with Auto mode (delta=64)...");
    let fast_sssp_auto = FastSSSP::new_with_mode(DegreeMode::Auto { delta: 64 });
    let start = Instant::now();
    let auto_result = fast_sssp_auto.compute_shortest_paths(graph, source).unwrap();
    let auto_time = start.elapsed();
    let auto_reachable = count_reachable(&auto_result);
    println!("  Time: {:.2?}, Reachable vertices: {}", auto_time, auto_reachable);
    println!("  Speedup vs classic: {:.2}x", classic_time.as_secs_f64() / auto_time.as_secs_f64());
    println!("  Slowdown vs Dijkstra: {:.2}x", auto_time.as_secs_f64() / dijkstra_time.as_secs_f64());
    
    // Run FastSSSP with no transformation
    println!("Running FastSSSP with None mode...");
    let fast_sssp_none = FastSSSP::new_with_mode(DegreeMode::None);
    let start = Instant::now();
    let none_result = fast_sssp_none.compute_shortest_paths(graph, source).unwrap();
    let none_time = start.elapsed();
    let none_reachable = count_reachable(&none_result);
    println!("  Time: {:.2?}, Reachable vertices: {}", none_time, none_reachable);
    println!("  Speedup vs classic: {:.2}x", classic_time.as_secs_f64() / none_time.as_secs_f64());
    println!("  Slowdown vs Dijkstra: {:.2}x", none_time.as_secs_f64() / dijkstra_time.as_secs_f64());
    
    // Verify correctness
    verify_results(&dijkstra_result, &classic_result, "classic");
    verify_results(&dijkstra_result, &auto_result, "auto");
    verify_results(&dijkstra_result, &none_result, "none");
}

fn count_reachable<W: ordered_float::Float + std::fmt::Debug + Copy + Zero>(result: &ShortestPathResult<W>) -> usize {
    result.distances.iter().filter(|d| d.is_some()).count()
}

fn verify_results<W: std::cmp::PartialEq + std::fmt::Debug + Copy + ordered_float::Float>(
    baseline: &ShortestPathResult<W>,
    result: &ShortestPathResult<W>,
    mode_name: &str
) {
    let baseline_reachable = count_reachable(baseline);
    let result_reachable = count_reachable(result);
    
    if baseline_reachable != result_reachable {
        println!("  WARNING: {} mode found {} reachable vertices, but baseline found {}",
                mode_name, result_reachable, baseline_reachable);
    } else {
        println!("  ✓ {} mode correctly found all {} reachable vertices", 
                mode_name, result_reachable);
    }
    
    // Check that distances match for reachable vertices
    let mut mismatches = 0;
    for i in 0..baseline.distances.len() {
        if baseline.distances[i].is_some() && result.distances[i].is_some() {
            if baseline.distances[i] != result.distances[i] {
                mismatches += 1;
            }
        }
    }
    
    if mismatches > 0 {
        println!("  WARNING: {} mode had {} distance mismatches with baseline",
                mode_name, mismatches);
    } else {
        println!("  ✓ {} mode distances match baseline for all reachable vertices", mode_name);
    }
}
