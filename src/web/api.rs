use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

use crate::algorithm::{ShortestPathAlgorithm, ShortestPathResult};
use crate::algorithm::dijkstra::Dijkstra;
use crate::algorithm::fast_sssp::{FastSSSP, DegreeMode};
use crate::algorithm::bmssp::BMSSP;
use crate::algorithm::smart_sssp::SmartSSSP;
use crate::graph::directed::DirectedGraph;
use crate::graph::generators::{generate_barabasi_albert, generate_3d_grid, generate_geometric_3d};
use crate::graph::traits::{Graph, MutableGraph};
use crate::web::models::*;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<Mutex<HashMap<Uuid, Session>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Create the API router
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/graphs/generate", post(generate_graph))
        .route("/api/graphs/:session_id", get(get_graph))
        .route("/api/algorithms/run/:session_id", post(run_algorithm))
        .route("/api/algorithms/compare/:session_id", post(compare_algorithms))
        .route("/api/benchmark", post(run_benchmark))
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/:session_id", get(get_session))
        .route("/api/health", get(health_check))
}

/// Generate a new graph
pub async fn generate_graph(
    State(state): State<AppState>,
    Json(request): Json<GraphGenerationRequest>,
) -> Result<Json<Session>, (StatusCode, Json<ErrorResponse>)> {
    let graph = match request.graph_type.as_str() {
        "scale-free" => {
            let rust_graph = generate_barabasi_albert(request.node_count, request.edges_per_node);
            convert_graph_to_web(&rust_graph)
        }
        "grid-3d" => {
            let (x, y, z) = request.grid_dimensions
                .unwrap_or_else(|| {
                    let size = (request.node_count as f64).cbrt().ceil() as usize;
                    (size, size, size)
                });
            let rust_graph = generate_3d_grid(x, y, z);
            convert_graph_to_web(&rust_graph)
        }
        "geometric-3d" => {
            let rust_graph = generate_geometric_3d(request.node_count, request.radius);
            convert_graph_to_web(&rust_graph)
        }
        _ => {
            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                error: "invalid_graph_type".to_string(),
                message: format!("Unknown graph type: {}", request.graph_type),
                details: None,
            })));
        }
    };

    let session = Session::new(graph);
    let session_id = session.id;
    
    {
        let mut sessions = state.sessions.lock().unwrap();
        sessions.insert(session_id, session.clone());
    }

    Ok(Json(session))
}

/// Get graph data for a session
pub async fn get_graph(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<WebGraph>, (StatusCode, Json<ErrorResponse>)> {
    let sessions = state.sessions.lock().unwrap();
    
    match sessions.get(&session_id) {
        Some(session) => Ok(Json(session.graph.clone())),
        None => Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: "session_not_found".to_string(),
            message: "Session not found".to_string(),
            details: None,
        }))),
    }
}

/// Run an algorithm on a graph
pub async fn run_algorithm(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<AlgorithmRequest>,
) -> Result<Json<AlgorithmResponse>, (StatusCode, Json<ErrorResponse>)> {
    let graph = {
        let sessions = state.sessions.lock().unwrap();
        match sessions.get(&session_id) {
            Some(session) => session.graph.clone(),
            None => {
                return Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
                    error: "session_not_found".to_string(),
                    message: "Session not found".to_string(),
                    details: None,
                })));
            }
        }
    };

    // Convert web graph to Rust graph
    let rust_graph = match convert_web_graph_to_rust(&graph) {
        Ok(graph) => graph,
        Err(err) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "graph_conversion_failed".to_string(),
                message: format!("Failed to convert graph: {}", err),
                details: None,
            })));
        }
    };
    
    // Execute the algorithm
    let start_time = Instant::now();
    let result = match request.algorithm.as_str() {
        "dijkstra" => {
            let dijkstra = Dijkstra::new();
            dijkstra.compute_shortest_paths(&rust_graph, request.source)
        }
        "fast-sssp" => {
            let mut fast_sssp = FastSSSP::new();
            fast_sssp = fast_sssp.with_degree_mode(DegreeMode::None);
            fast_sssp.compute_shortest_paths(&rust_graph, request.source)
        }
        "mini-bmssp" => {
            let k = request.k.unwrap_or_else(|| {
                let n = rust_graph.vertex_count();
                ((n as f64).ln().powf(1.0 / 3.0).round() as usize).max(2)
            });
            let t = request.t.unwrap_or_else(|| {
                let n = rust_graph.vertex_count();
                ((n as f64).ln().powf(2.0 / 3.0).round() as usize).max(2)
            });
            
            let bmssp = BMSSP::new_with_params(rust_graph.vertex_count(), k, t);
            let mut distances = vec![OrderedFloat(f64::INFINITY); rust_graph.vertex_count()];
            let mut predecessors = vec![None; rust_graph.vertex_count()];
            distances[request.source] = OrderedFloat(0.0);
            
            let level = (k as f64).log2().ceil() as usize;
            bmssp.execute(
                &rust_graph,
                level,
                OrderedFloat(f64::INFINITY),
                &[request.source],
                &mut distances,
                &mut predecessors,
            ).map(|_| ShortestPathResult { 
                source: request.source,
                distances: distances.into_iter().map(Some).collect(), 
                predecessors 
            })
        }
        "smart-sssp" => {
            let smart_sssp = SmartSSSP::new();
            smart_sssp.compute_shortest_paths(&rust_graph, request.source)
        }
        _ => {
            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                error: "invalid_algorithm".to_string(),
                message: format!("Unknown algorithm: {}", request.algorithm),
                details: None,
            })));
        }
    };
    
    let execution_time = start_time.elapsed();
    
    match result {
        Ok(shortest_path_result) => {
            // Convert results to web format
            let mut distances_map = HashMap::new();
            let mut predecessors_map = HashMap::new();
            
            for (i, distance) in shortest_path_result.distances.iter().enumerate() {
                if let Some(dist) = distance {
                    distances_map.insert(i, dist.into_inner());
                }
            }
            
            for (i, pred) in shortest_path_result.predecessors.iter().enumerate() {
                predecessors_map.insert(i, *pred);
            }
            
            // Generate animation steps (simplified for now)
            let animation_steps = generate_animation_steps(&shortest_path_result, request.source);
            
            // Create parameters map with algorithm-specific parameters
            let mut parameters = HashMap::new();
            if let Some(k) = request.k {
                parameters.insert("k".to_string(), serde_json::json!(k));
            }
            if let Some(t) = request.t {
                parameters.insert("t".to_string(), serde_json::json!(t));
            }
            parameters.insert("auto_tune".to_string(), serde_json::json!(request.auto_tune));
            parameters.insert("skip_sweep".to_string(), serde_json::json!(request.skip_sweep));
            parameters.insert("parallel".to_string(), serde_json::json!(request.parallel));
            
            // Determine algorithm type based on the algorithm name
            let algorithm_type = match request.algorithm.as_str() {
                "dijkstra" => "classic",
                "bmssp" => "delta-stepping",
                "fast_sssp" => "hierarchical",
                "smart_sssp" => "adaptive",
                _ => "unknown"
            }.to_string();
            
            let response = AlgorithmResponse {
                execution_id: Uuid::new_v4(),
                algorithm: request.algorithm.clone(),
                algorithm_type,
                source: request.source,
                parameters,
                execution_time_ms: execution_time.as_secs_f64() * 1000.0,
                nodes_processed: distances_map.len(),
                edges_relaxed: rust_graph.edge_count(), // Simplified
                distances: distances_map,
                predecessors: predecessors_map,
                animation_steps,
                metrics: AlgorithmMetrics {
                    heap_operations: 0, // TODO: Implement proper metrics collection
                    cache_hits: 0,
                    cache_misses: 0,
                    pivot_selections: 0,
                    base_case_calls: 0,
                    recursive_calls: 0,
                    work_set_sizes: vec![],
                    memory_usage_mb: 0.0,
                },
            };
            
            // Update session with result
            {
                let mut sessions = state.sessions.lock().unwrap();
                if let Some(session) = sessions.get_mut(&session_id) {
                    session.last_result = Some(response.clone());
                }
            }
            
            Ok(Json(response))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "algorithm_execution_failed".to_string(),
            message: format!("Algorithm execution failed: {}", e),
            details: None,
        }))),
    }
}

/// Compare multiple algorithms on the same graph
pub async fn compare_algorithms(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(algorithms): Json<Vec<AlgorithmRequest>>,
) -> Result<Json<Vec<AlgorithmResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let mut results = Vec::new();
    
    for algorithm_request in algorithms {
        let result = run_algorithm(State(state.clone()), Path(session_id), Json(algorithm_request)).await?;
        results.push(result.0);
    }
    
    Ok(Json(results))
}

/// Run comprehensive benchmark
pub async fn run_benchmark(
    State(_state): State<AppState>,
    Json(request): Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut results = Vec::new();
    
    for graph_type in &request.graph_types {
        for &node_count in &request.node_counts {
            for algorithm in &request.algorithms {
                // Generate graph
                let rust_graph = match graph_type.as_str() {
                    "scale-free" => generate_barabasi_albert(node_count, 3),
                    "grid-3d" => {
                        let size = (node_count as f64).cbrt().ceil() as usize;
                        generate_3d_grid(size, size, size)
                    }
                    "geometric-3d" => generate_geometric_3d(node_count, 0.2),
                    _ => continue,
                };
                
                // Run algorithm multiple times
                let mut total_time = 0.0;
                let mut success_count = 0;
                
                for _ in 0..request.iterations {
                    let start_time = Instant::now();
                    
                    let result = match algorithm.as_str() {
                        "dijkstra" => {
                            let dijkstra = Dijkstra::new();
                            dijkstra.compute_shortest_paths(&rust_graph, 0)
                        }
                        "fast-sssp" => {
                            let mut fast_sssp = FastSSSP::new();
                            fast_sssp = fast_sssp.with_degree_mode(DegreeMode::None);
                            fast_sssp.compute_shortest_paths(&rust_graph, 0)
                        }
                        "smart-sssp" => {
                            let smart_sssp = SmartSSSP::new();
                            smart_sssp.compute_shortest_paths(&rust_graph, 0)
                        }
                        _ => continue,
                    };
                    
                    let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
                    
                    if result.is_ok() {
                        total_time += execution_time;
                        success_count += 1;
                    }
                }
                
                if success_count > 0 {
                    results.push(BenchmarkResult {
                        algorithm: algorithm.clone(),
                        graph_type: graph_type.clone(),
                        node_count,
                        edge_count: rust_graph.edge_count(),
                        execution_time_ms: total_time / success_count as f64,
                        memory_usage_mb: 0.0, // TODO: Implement memory tracking
                        success: true,
                        error_message: None,
                        metrics: AlgorithmMetrics {
                            heap_operations: 0,
                            cache_hits: 0,
                            cache_misses: 0,
                            pivot_selections: 0,
                            base_case_calls: 0,
                            recursive_calls: 0,
                            work_set_sizes: vec![],
                            memory_usage_mb: 0.0,
                        },
                    });
                }
            }
        }
    }
    
    // Generate summary
    let summary = generate_benchmark_summary(&results);
    
    Ok(Json(BenchmarkResponse {
        benchmark_id: Uuid::new_v4(),
        results,
        summary,
    }))
}

/// List all active sessions
pub async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Uuid>>, (StatusCode, Json<ErrorResponse>)> {
    let sessions = state.sessions.lock().unwrap();
    let session_ids: Vec<Uuid> = sessions.keys().cloned().collect();
    Ok(Json(session_ids))
}

/// Get session information
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Session>, (StatusCode, Json<ErrorResponse>)> {
    let sessions = state.sessions.lock().unwrap();
    
    match sessions.get(&session_id) {
        Some(session) => Ok(Json(session.clone())),
        None => Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: "session_not_found".to_string(),
            message: "Session not found".to_string(),
            details: None,
        }))),
    }
}

/// Health check endpoint
pub async fn health_check() -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    })))
}

// Helper functions

fn convert_graph_to_web(graph: &DirectedGraph<OrderedFloat<f64>>) -> WebGraph {
    let nodes = (0..graph.vertex_count())
        .map(|i| WebNode {
            id: i,
            label: format!("Node {}", i),
            x: None,
            y: None,
            z: None,
            distance: f64::INFINITY,
            is_visited: false,
            is_frontier: false,
            is_pivot: false,
            is_current: false,
        })
        .collect();
    
    let mut links = Vec::new();
    for u in 0..graph.vertex_count() {
        for (v, weight) in graph.outgoing_edges(u) {
            links.push(WebEdge {
                source: u,
                target: v,
                weight: weight.into_inner(),
                is_path: false,
            });
        }
    }
    
    WebGraph { nodes, links }
}

fn convert_web_graph_to_rust(web_graph: &WebGraph) -> Result<DirectedGraph<OrderedFloat<f64>>, String> {
    let mut graph = DirectedGraph::with_capacity(web_graph.nodes.len());
    
    for edge in &web_graph.links {
        if edge.source >= web_graph.nodes.len() || edge.target >= web_graph.nodes.len() {
            return Err(format!("Edge references invalid node: {} -> {}", edge.source, edge.target));
        }
        graph.add_edge(edge.source, edge.target, OrderedFloat(edge.weight));
    }
    
    Ok(graph)
}

fn generate_animation_steps(
    result: &ShortestPathResult<OrderedFloat<f64>>,
    source: usize,
) -> Vec<AnimationStep> {
    let mut steps = Vec::new();
    let mut step_id = 0;
    
    // Add source node as first step
    steps.push(AnimationStep {
        step_id,
        step_type: "source".to_string(),
        timestamp_ms: 0.0,
        node_id: Some(source),
        distance: Some(0.0),
        source: None,
        target: None,
        description: format!("Starting from source node {}", source),
        metadata: HashMap::new(),
    });
    step_id += 1;
    
    // Add visited nodes in order of distance
    let mut visited_nodes: Vec<_> = result.distances
        .iter()
        .enumerate()
        .filter_map(|(i, dist)| dist.map(|d| (i, d.into_inner())))
        .collect();
    
    visited_nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
    for (i, (node_id, distance)) in visited_nodes.iter().enumerate() {
        if *node_id != source {
            steps.push(AnimationStep {
                step_id,
                step_type: "visit".to_string(),
                timestamp_ms: (i + 1) as f64 * 100.0,
                node_id: Some(*node_id),
                distance: Some(*distance),
                source: None,
                target: None,
                description: format!("Visited node {} with distance {:.2}", node_id, distance),
                metadata: HashMap::new(),
            });
            step_id += 1;
        }
    }
    
    steps
}

fn generate_benchmark_summary(results: &[BenchmarkResult]) -> BenchmarkSummary {
    let mut algorithm_times: HashMap<String, Vec<f64>> = HashMap::new();
    let mut algorithm_success: HashMap<String, (usize, usize)> = HashMap::new();
    
    for result in results {
        algorithm_times
            .entry(result.algorithm.clone())
            .or_default()
            .push(result.execution_time_ms);
        
        let (success, total) = algorithm_success
            .entry(result.algorithm.clone())
            .or_insert((0, 0));
        
        if result.success {
            *success += 1;
        }
        *total += 1;
    }
    
    // Find fastest algorithm
    let fastest_algorithm = algorithm_times
        .iter()
        .min_by(|a, b| {
            let avg_a = a.1.iter().sum::<f64>() / a.1.len() as f64;
            let avg_b = b.1.iter().sum::<f64>() / b.1.len() as f64;
            avg_a.partial_cmp(&avg_b).unwrap()
        })
        .map(|(name, _)| name.clone())
        .unwrap_or_default();
    
    // Calculate average speedup
    let dijkstra_avg = algorithm_times
        .get("dijkstra")
        .map(|times| times.iter().sum::<f64>() / times.len() as f64)
        .unwrap_or(1.0);
    
    let average_speedup = algorithm_times
        .iter()
        .filter(|(name, _)| *name != "dijkstra")
        .map(|(_, times)| {
            let avg_time = times.iter().sum::<f64>() / times.len() as f64;
            dijkstra_avg / avg_time
        })
        .sum::<f64>() / (algorithm_times.len() - 1).max(1) as f64;
    
    // Calculate success rates
    let success_rate = algorithm_success
        .iter()
        .map(|(name, (success, total))| {
            (name.clone(), *success as f64 / *total as f64)
        })
        .collect();
    
    BenchmarkSummary {
        fastest_algorithm,
        average_speedup,
        memory_efficiency: HashMap::new(), // TODO: Implement memory tracking
        success_rate,
    }
}
