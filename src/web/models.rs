use chrono::{DateTime, Utc};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a node in the graph for web visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebNode {
    pub id: usize,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>,
    #[serde(default)]
    pub distance: f64,
    #[serde(default)]
    pub is_visited: bool,
    #[serde(default)]
    pub is_frontier: bool,
    #[serde(default)]
    pub is_pivot: bool,
    #[serde(default)]
    pub is_current: bool,
}

/// Represents an edge in the graph for web visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebEdge {
    pub source: usize,
    pub target: usize,
    pub weight: f64,
    #[serde(default)]
    pub is_path: bool,
}

/// Represents a complete graph for web visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGraph {
    pub nodes: Vec<WebNode>,
    pub links: Vec<WebEdge>,
}

/// Parameters for graph generation
#[derive(Debug, Deserialize)]
pub struct GraphGenerationRequest {
    pub graph_type: String,
    pub node_count: usize,
    #[serde(default = "default_edges_per_node")]
    pub edges_per_node: usize,
    #[serde(default = "default_radius")]
    pub radius: f64,
    #[serde(default)]
    pub grid_dimensions: Option<(usize, usize, usize)>,
}

fn default_edges_per_node() -> usize { 3 }
fn default_radius() -> f64 { 0.2 }

/// Parameters for algorithm execution
#[derive(Debug, Deserialize)]
pub struct AlgorithmRequest {
    pub algorithm: String,
    pub source: usize,
    #[serde(default)]
    pub k: Option<usize>,
    #[serde(default)]
    pub t: Option<usize>,
    #[serde(default)]
    pub auto_tune: bool,
    #[serde(default)]
    pub skip_sweep: bool,
    #[serde(default)]
    pub parallel: bool,
}

/// Response containing algorithm execution results
#[derive(Debug, Clone, Serialize)]
pub struct AlgorithmResponse {
    pub execution_id: Uuid,
    pub algorithm: String,
    pub algorithm_type: String,
    pub source: usize,
    pub parameters: HashMap<String, serde_json::Value>,
    pub execution_time_ms: f64,
    pub nodes_processed: usize,
    pub edges_relaxed: usize,
    pub distances: HashMap<usize, f64>,
    pub predecessors: HashMap<usize, Option<usize>>,
    pub animation_steps: Vec<AnimationStep>,
    pub metrics: AlgorithmMetrics,
}

/// Metrics collected during algorithm execution
#[derive(Debug, Clone, Serialize)]
pub struct AlgorithmMetrics {
    pub heap_operations: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub pivot_selections: usize,
    pub base_case_calls: usize,
    pub recursive_calls: usize,
    pub work_set_sizes: Vec<usize>,
    pub memory_usage_mb: f64,
}

/// Animation step for visualizing algorithm execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationStep {
    pub step_id: usize,
    pub step_type: String,
    pub timestamp_ms: f64,
    pub node_id: Option<usize>,
    pub distance: Option<f64>,
    pub source: Option<usize>,
    pub target: Option<usize>,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Benchmark configuration
#[derive(Debug, Deserialize)]
pub struct BenchmarkRequest {
    pub algorithms: Vec<String>,
    pub graph_types: Vec<String>,
    pub node_counts: Vec<usize>,
    pub iterations: usize,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

/// Benchmark results
#[derive(Debug, Serialize)]
pub struct BenchmarkResponse {
    pub benchmark_id: Uuid,
    pub results: Vec<BenchmarkResult>,
    pub summary: BenchmarkSummary,
}

/// Individual benchmark result
#[derive(Debug, Serialize)]
pub struct BenchmarkResult {
    pub algorithm: String,
    pub graph_type: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub execution_time_ms: f64,
    pub memory_usage_mb: f64,
    pub success: bool,
    pub error_message: Option<String>,
    pub metrics: AlgorithmMetrics,
}

/// Summary statistics for benchmark
#[derive(Debug, Serialize)]
pub struct BenchmarkSummary {
    pub fastest_algorithm: String,
    pub average_speedup: f64,
    pub memory_efficiency: HashMap<String, f64>,
    pub success_rate: HashMap<String, f64>,
}

/// Error response for API
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Session containing graph data and execution history
#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: Uuid,
    pub graph: WebGraph,
    pub last_result: Option<AlgorithmResponse>,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn new(graph: WebGraph) -> Self {
        Self {
            id: Uuid::new_v4(),
            graph,
            last_result: None,
            created_at: Utc::now(),
        }
    }
}
