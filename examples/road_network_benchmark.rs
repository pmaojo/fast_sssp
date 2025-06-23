use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::FastSSSP;
use fast_sssp::algorithm::traits::ShortestPathAlgorithm;
use fast_sssp::graph::{DirectedGraph, MutableGraph};
use ordered_float::OrderedFloat;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::time::Instant;
use colored::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: road_network_benchmark <path_to_dimacs.gr> [source]");
        return;
    }
    let path = &args[1];
    let source: usize = if args.len() > 2 { args[2].parse().unwrap_or(0) } else { 0 };

    println!("\u{1F4E5} Loading DIMACS graph from {}...", path);
    let graph = load_dimacs_graph(path).expect("Failed to load graph");
    println!("✅ Loaded graph with {} vertices and {} edges", graph.vertex_count(), graph.edge_count());
    println!("🎯 Source vertex: {}", source);

    // Run FastSSSP
    let fast_sssp = FastSSSP::new();
    let start = Instant::now();
    let fast_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    let fast_time = start.elapsed();
    let fast_reachable = fast_result.distances.iter().filter(|d| d.is_some()).count();
    println!("⏱️ Fast SSSP time: {}", format!("{:?}", fast_time).bright_cyan());
    println!("📍 Vertices reachable with Fast SSSP: {}", fast_reachable);

    // Run Dijkstra
    let dijkstra = Dijkstra::new();
    let start = Instant::now();
    let dijkstra_result = dijkstra.compute_shortest_paths(&graph, source).unwrap();
    let dijkstra_time = start.elapsed();
    let dijkstra_reachable = dijkstra_result.distances.iter().filter(|d| d.is_some()).count();
    println!("⏱️ Dijkstra time: {}", format!("{:?}", dijkstra_time).bright_cyan());
    println!("📍 Vertices reachable with Dijkstra: {}", dijkstra_reachable);

    // Compare results for vertices reachable by both algorithms
    let mut mismatch = 0usize;
    for i in 0..graph.vertex_count() {
        if let (Some(a), Some(b)) = (fast_result.distances[i], dijkstra_result.distances[i]) {
            if ((a.into_inner() - b.into_inner()) as f64).abs() > 1e-6 {
                mismatch += 1;
                if mismatch <= 5 {
                    println!("⚠️ Mismatch at vertex {}: FastSSSP {:?}, Dijkstra {:?}", i, a, b);
                }
            }
        }
    }
    if mismatch == 0 {
        println!("✅ Results match for all commonly reachable vertices");
    } else {
        println!("⚠️ {} mismatched vertices", mismatch);
    }

    // Speedup
    let speedup = dijkstra_time.as_secs_f64() / fast_time.as_secs_f64();
    if speedup > 1.0 {
        println!("🚀 Fast SSSP is {:.2}x faster than Dijkstra", speedup);
    } else {
        println!("⚠️ Dijkstra is {:.2}x faster than Fast SSSP", 1.0 / speedup);
    }
}

fn load_dimacs_graph(path: &str) -> io::Result<DirectedGraph<OrderedFloat<f64>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut graph = DirectedGraph::new();
    let mut vertices = 0usize;
    for line in reader.lines() {
        let line = line?;
        if line.starts_with('c') || line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "p" => {
                if parts.len() >= 4 {
                    vertices = parts[2].parse().unwrap_or(0);
                    graph = DirectedGraph::with_capacity(vertices);
                }
            }
            "a" => {
                if parts.len() >= 4 {
                    let u: usize = parts[1].parse().unwrap();
                    let v: usize = parts[2].parse().unwrap();
                    let w: f64 = parts[3].parse().unwrap();
                    // DIMACS format is 1-indexed
                    if u > 0 && v > 0 {
                        graph.add_edge(u - 1, v - 1, OrderedFloat(w));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(graph)
}

