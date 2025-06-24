use std::time::Instant;
use ordered_float::OrderedFloat;
use colored::*;
use std::marker::PhantomData;

use fast_sssp::algorithm::ShortestPathAlgorithm;
use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::{FastSSSP, DegreeMode};
use fast_sssp::algorithm::bmssp::BMSSP;
use fast_sssp::graph::DirectedGraph;
use fast_sssp::graph::Graph;
use fast_sssp::graph::generators::{generate_barabasi_albert, generate_3d_grid, generate_geometric_3d};

fn main() {
    println!("{}", "BMSSP Base Case Benchmark".green().bold());
    println!("Evaluando la eficiencia del caso base optimizado de BMSSP\n");

    // Benchmark en diferentes tipos de grafos
    benchmark_on_scale_free_graphs();
    benchmark_on_grid_graphs();
    benchmark_on_geometric_graphs();
}

/// Benchmark en grafos scale-free con diferentes tamaños
fn benchmark_on_scale_free_graphs() {
    println!("\n{}", "Benchmark en grafos scale-free".yellow().bold());
    
    let sizes = [10_000, 50_000, 100_000];
    let edges_per_node = 5;
    
    for &size in &sizes {
        println!("\nGenerando grafo scale-free con {} nodos...", size);
        let graph = generate_barabasi_albert(size, edges_per_node);
        
        println!("Grafo generado con {} vértices y {} aristas", 
            graph.vertex_count(), graph.edge_count());
        
        // Ejecutar FastSSSP con diferentes configuraciones
        run_benchmark(&graph, 0);
    }
}

/// Benchmark en grafos de rejilla 3D
fn benchmark_on_grid_graphs() {
    println!("\n{}", "Benchmark en grafos de rejilla 3D".yellow().bold());
    
    let sizes = [(20, 20, 20), (30, 30, 30), (40, 40, 40)];
    
    for &(x, y, z) in &sizes {
        println!("\nGenerando grafo de rejilla 3D {}x{}x{}...", x, y, z);
        let graph = generate_3d_grid(x, y, z);
        
        println!("Grafo generado con {} vértices y {} aristas", 
            graph.vertex_count(), graph.edge_count());
        
        // Ejecutar FastSSSP con diferentes configuraciones
        run_benchmark(&graph, 0);
    }
}

/// Benchmark en grafos geométricos 3D
fn benchmark_on_geometric_graphs() {
    println!("\n{}", "Benchmark en grafos geométricos 3D".yellow().bold());
    
    let sizes = [10_000, 20_000, 30_000];
    let radius = 0.1;
    
    for &size in &sizes {
        println!("\nGenerando grafo geométrico 3D con {} nodos...", size);
        let graph = generate_geometric_3d(size, radius);
        
        println!("Grafo generado con {} vértices y {} aristas", 
            graph.vertex_count(), graph.edge_count());
        
        // Ejecutar FastSSSP con diferentes configuraciones
        run_benchmark(&graph, 0);
    }
}

/// Ejecutar benchmark con diferentes configuraciones de FastSSSP
fn run_benchmark(graph: &DirectedGraph<OrderedFloat<f64>>, source: usize) {
    // Parámetros para BMSSP
    let n = graph.vertex_count();
    let ln = (n as f64).ln();
    let k_values = [
        ln.powf(1.0 / 3.0).round() as usize,
        ln.powf(1.0 / 3.0).round() as usize * 2,
        ln.powf(1.0 / 3.0).round() as usize / 2,
    ];
    let t_values = [
        ln.powf(2.0 / 3.0).round() as usize,
        ln.powf(2.0 / 3.0).round() as usize * 2,
        ln.powf(2.0 / 3.0).round() as usize / 2,
    ];
    
    // Ejecutar Dijkstra como referencia
    println!("\n🏃 Ejecutando Dijkstra (referencia)...");
    let start = Instant::now();
    let dijkstra = Dijkstra::new();
    let dijkstra_result = dijkstra.compute_shortest_paths(graph, source).unwrap();
    let dijkstra_time = start.elapsed();
    let dijkstra_ms = dijkstra_time.as_secs_f64() * 1000.0;
    println!("⏱️  Tiempo: {:.2}ms", dijkstra_ms);
    
    // Contar vértices alcanzables
    let reachable = dijkstra_result.distances.iter()
        .filter(|d| d.is_some())
        .count();
    println!("📍 Vértices alcanzables: {}", reachable);
    
    // Ejecutar FastSSSP con diferentes parámetros
    println!("\n🔍 Probando diferentes configuraciones de parámetros k y t:");
    
    for &k in &k_values {
        for &t in &t_values {
            // Asegurar valores mínimos
            let k = k.max(2);
            let t = t.max(2);
            
            println!("\n🏃 Ejecutando FastSSSP con k={}, t={}...", k, t);
            let start = Instant::now();
            
            // Usar FastSSSP con los parámetros estándar
            let mut fast_sssp = FastSSSP::new();
            fast_sssp = fast_sssp.with_degree_mode(DegreeMode::None);
            let _fast_sssp_result = fast_sssp.compute_shortest_paths(graph, source).unwrap();
            let fast_sssp_time = start.elapsed();
            let fast_sssp_ms = fast_sssp_time.as_secs_f64() * 1000.0;
            println!("⏱️  Tiempo FastSSSP estándar: {:.2}ms", fast_sssp_ms);
            
            // Ahora probar directamente con BMSSP usando los parámetros personalizados
            println!("\n🔍 Ejecutando BMSSP directamente con k={}, t={}...", k, t);
            let start = Instant::now();
            
            // Crear instancia de BMSSP con parámetros personalizados
            let bmssp = BMSSP::new_with_params(graph.vertex_count(), k, t);
            
            // Preparar estructuras de datos para BMSSP
            let mut distances = vec![OrderedFloat(f64::INFINITY); graph.vertex_count()];
            let mut predecessors = vec![None; graph.vertex_count()];
            distances[source] = OrderedFloat(0.0);
            
            // Ejecutar BMSSP
            let level = (k as f64).log2().ceil() as usize;
            let _bmssp_result = bmssp.execute(
                graph,
                level,
                OrderedFloat(f64::INFINITY),
                &[source],
                &mut distances,
                &mut predecessors
            ).unwrap();
            
            let bmssp_time = start.elapsed();
            let bmssp_ms = bmssp_time.as_secs_f64() * 1000.0;
            println!("⏱️  Tiempo BMSSP directo: {:.2}ms", bmssp_ms);
            println!("📊 Mejora vs FastSSSP: {:.2}x", fast_sssp_ms / bmssp_ms);
            println!("📊 Ratio vs Dijkstra: {:.2}x", bmssp_ms / dijkstra_ms);
        }
    }
}
