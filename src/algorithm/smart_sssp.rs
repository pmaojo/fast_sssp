use std::fmt::Debug;
use std::time::Instant;
use std::cell::RefCell;
use num_traits::{Float, Zero};

use crate::graph::Graph;
use crate::algorithm::{ShortestPathAlgorithm, ShortestPathResult};
use crate::algorithm::dijkstra::Dijkstra;
use crate::algorithm::fast_sssp::{FastSSSP, DegreeMode};
use crate::{Error, Result};

/// Smart algorithm selection mode
#[derive(Debug, Clone, Copy)]
pub enum SmartMode {
    /// Always use Dijkstra
    ForceDijkstra,
    /// Always use FastSSSP
    ForceFastSSSP,
    /// Use simplified FastSSSP (no transformations)
    SimpleFastSSSP,
    /// Automatically select best algorithm based on graph properties
    Auto,
    /// Adaptive mode that learns from previous runs
    Adaptive,
}

/// Performance statistics for algorithm selection
#[derive(Debug, Default, Clone)]
struct AlgorithmStats {
    /// Number of times Dijkstra was faster
    dijkstra_wins: usize,
    /// Number of times FastSSSP was faster
    fast_sssp_wins: usize,
    /// Number of times simplified FastSSSP was faster
    simple_fast_sssp_wins: usize,
    /// Average speedup when Dijkstra wins
    avg_dijkstra_speedup: f64,
    /// Average speedup when FastSSSP wins
    avg_fast_sssp_speedup: f64,
    /// Average speedup when simplified FastSSSP wins
    avg_simple_speedup: f64,
    /// Total runs
    total_runs: usize,
}

/// Smart Single-Source Shortest Path algorithm that selects the best algorithm based on graph properties
#[derive(Debug)]
pub struct SmartSSSP {
    /// Mode for algorithm selection
    mode: SmartMode,
    /// Threshold for number of vertices to consider FastSSSP
    vertex_threshold: usize,
    /// Threshold for reachable component size (as percentage of total vertices)
    reachable_threshold: f64,
    /// Maximum degree for considering FastSSSP without transformation
    max_degree_threshold: usize,
    /// Delta parameter for HubSplit transformation
    hub_split_delta: usize,
    /// Whether to collect performance statistics
    collect_stats: bool,
    /// Performance statistics
    stats: RefCell<AlgorithmStats>,
    /// Whether to print debug information
    verbose: bool,
}

impl SmartSSSP {
    /// Create a new SmartSSSP algorithm instance with default settings
    pub fn new() -> Self {
        SmartSSSP {
            mode: SmartMode::Auto,
            vertex_threshold: 50_000,
            reachable_threshold: 0.05,
            max_degree_threshold: 256,
            hub_split_delta: 256,
            collect_stats: false,
            stats: RefCell::new(AlgorithmStats::default()),
            verbose: false,
        }
    }
    
    /// Create a new SmartSSSP algorithm instance with specified mode
    pub fn with_mode(mode: SmartMode) -> Self {
        let mut sssp = SmartSSSP::new();
        sssp.mode = mode;
        sssp
    }
    
    /// Set the vertex threshold for considering FastSSSP
    pub fn with_vertex_threshold(mut self, threshold: usize) -> Self {
        self.vertex_threshold = threshold;
        self
    }
    
    /// Set the reachable component threshold (as percentage of total vertices)
    pub fn with_reachable_threshold(mut self, threshold: f64) -> Self {
        self.reachable_threshold = threshold;
        self
    }
    
    /// Set the maximum degree threshold for considering FastSSSP without transformation
    pub fn with_max_degree_threshold(mut self, threshold: usize) -> Self {
        self.max_degree_threshold = threshold;
        self
    }
    
    /// Set the delta parameter for HubSplit transformation
    pub fn with_hub_split_delta(mut self, delta: usize) -> Self {
        self.hub_split_delta = delta;
        self
    }
    
    /// Enable or disable collection of performance statistics
    pub fn with_stats_collection(mut self, collect: bool) -> Self {
        self.collect_stats = collect;
        self
    }
    
    /// Enable or disable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    /// Get the current performance statistics
    pub fn get_stats(&self) -> String {
        let stats = self.stats.borrow();
        if stats.total_runs == 0 {
            return "No statistics collected yet".to_string();
        }
        
        format!(
            "Algorithm performance statistics:\n\
            Total runs: {}\n\
            Dijkstra wins: {} ({:.1}%), avg speedup: {:.2}x\n\
            FastSSSP wins: {} ({:.1}%), avg speedup: {:.2}x\n\
            Simple FastSSSP wins: {} ({:.1}%), avg speedup: {:.2}x",
            stats.total_runs,
            stats.dijkstra_wins,
            100.0 * stats.dijkstra_wins as f64 / stats.total_runs as f64,
            stats.avg_dijkstra_speedup,
            stats.fast_sssp_wins,
            100.0 * stats.fast_sssp_wins as f64 / stats.total_runs as f64,
            stats.avg_fast_sssp_speedup,
            stats.simple_fast_sssp_wins,
            100.0 * stats.simple_fast_sssp_wins as f64 / stats.total_runs as f64,
            stats.avg_simple_speedup
        )
    }
    
    /// Reset the performance statistics
    pub fn reset_stats(&self) {
        *self.stats.borrow_mut() = AlgorithmStats::default();
    }
    
    /// Estimate the reachable component size using BFS sampling
    fn approx_reachable<W, G>(&self, graph: &G, source: usize, sample_size: usize) -> usize
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W>,
    {
        let n = graph.vertex_count();
        let mut visited = vec![false; n];
        let mut queue = std::collections::VecDeque::new();
        
        visited[source] = true;
        queue.push_back(source);
        
        let mut count = 1;
        while let Some(u) = queue.pop_front() {
            if count >= sample_size {
                // Extrapolate based on sample
                let sample_fraction = sample_size as f64 / n as f64;
                return (count as f64 / sample_fraction).round() as usize;
            }
            
            for (v, _) in graph.outgoing_edges(u) {
                if !visited[v] {
                    visited[v] = true;
                    queue.push_back(v);
                    count += 1;
                }
            }
        }
        
        // If we visited fewer than sample_size vertices, return exact count
        count
    }
    
    /// Estimate the maximum degree in the graph using sampling
    fn approx_max_degree<W, G>(&self, graph: &G, sample_size: usize) -> usize
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W>,
    {
        let n = graph.vertex_count();
        let mut rng = rand::thread_rng();
        let mut max_degree = 0;
        
        use rand::seq::SliceRandom;
        let mut vertices: Vec<usize> = (0..n).collect();
        vertices.shuffle(&mut rng);
        
        for &v in vertices.iter().take(sample_size.min(n)) {
            let out_degree = graph.outgoing_edges(v).count();
            let in_degree = graph.incoming_edges(v).count();
            max_degree = max_degree.max(out_degree).max(in_degree);
        }
        
        max_degree
    }
    
    /// Run the algorithm with the best strategy based on graph properties
    fn run_best_algorithm<W, G>(&self, graph: &G, source: usize) -> Result<ShortestPathResult<W>>
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W> + Clone + crate::graph::traits::MutableGraph<W> + crate::graph::traits::ToConstantDegree<W>,
    {
        let n = graph.vertex_count();
        
        // For small graphs, always use Dijkstra
        if n < self.vertex_threshold {
            if self.verbose {
                println!("Small graph (n={}), using Dijkstra algorithm", n);
            }
            return self.run_dijkstra(graph, source);
        }
        
        // Estimate reachable component size
        let sample_size = (n as f64).sqrt().round() as usize * 4;
        let reach = self.approx_reachable(graph, source, sample_size);
        let reach_ratio = reach as f64 / n as f64;
        
        if reach_ratio < self.reachable_threshold {
            if self.verbose {
                println!("Reachable component is small ({} < {}), using Dijkstra algorithm", 
                    reach, (self.reachable_threshold * n as f64) as usize);
            }
            return self.run_dijkstra(graph, source);
        }
        
        // Estimate maximum degree
        let degree_sample_size = (n as f64).sqrt().round() as usize;
        let max_degree = self.approx_max_degree(graph, degree_sample_size);
        
        if max_degree > self.max_degree_threshold {
            if self.verbose {
                println!("High maximum degree ({}), using FastSSSP with HubSplit", max_degree);
            }
            
            // Use FastSSSP with HubSplit
            let mut fast_sssp = FastSSSP::new();
            fast_sssp = fast_sssp.with_degree_mode(DegreeMode::Auto { delta: self.hub_split_delta })
                .with_vertex_threshold(self.vertex_threshold);
            return fast_sssp.compute_shortest_paths(graph, source);
        } else {
            if self.verbose {
                println!("Low maximum degree ({}), using simplified FastSSSP", max_degree);
            }
            
            // Use simplified FastSSSP (no transformation)
            let mut fast_sssp = FastSSSP::new();
            fast_sssp = fast_sssp.with_degree_mode(DegreeMode::None)
                .with_vertex_threshold(self.vertex_threshold);
            return fast_sssp.compute_shortest_paths(graph, source);
        }
    }
    
    /// Run Dijkstra's algorithm
    fn run_dijkstra<W, G>(&self, graph: &G, source: usize) -> Result<ShortestPathResult<W>>
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W>,
    {
        if self.verbose {
            println!("Using Dijkstra's algorithm directly");
        }
        
        let dijkstra = Dijkstra::new();
        dijkstra.compute_shortest_paths(graph, source)
    }
    
    /// Run FastSSSP algorithm with HubSplit transformation
    fn run_fast_sssp_with_hubsplit<W, G>(&self, graph: &G, source: usize) -> Result<ShortestPathResult<W>>
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W> + Clone + crate::graph::traits::MutableGraph<W> + crate::graph::traits::ToConstantDegree<W>,
    {
        if self.verbose {
            println!("Using FastSSSP with HubSplit transformation");
        }
        
        let mut fast_sssp = FastSSSP::new();
        fast_sssp = fast_sssp.with_degree_mode(DegreeMode::Auto { delta: self.hub_split_delta })
            .with_vertex_threshold(self.vertex_threshold);
        fast_sssp.compute_shortest_paths(graph, source)
    }
    
    /// Run simplified FastSSSP algorithm (no transformation)
    fn run_simplified_fast_sssp<W, G>(&self, graph: &G, source: usize) -> Result<ShortestPathResult<W>>
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W> + Clone + crate::graph::traits::MutableGraph<W> + crate::graph::traits::ToConstantDegree<W>,
    {
        if self.verbose {
            println!("Using simplified FastSSSP (no transformation)");
        }
        
        let mut fast_sssp = FastSSSP::new();
        fast_sssp = fast_sssp.with_degree_mode(DegreeMode::None)
            .with_vertex_threshold(self.vertex_threshold);
        fast_sssp.compute_shortest_paths(graph, source)
    }
    
    /// Run all algorithms and compare their performance
    fn run_adaptive<W, G>(&self, graph: &G, source: usize) -> Result<ShortestPathResult<W>>
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W> + Clone + crate::graph::traits::MutableGraph<W> + crate::graph::traits::ToConstantDegree<W>,
    {
        if self.verbose {
            println!("Running adaptive mode with performance comparison");
        }
        
        // Run Dijkstra
        let start = Instant::now();
        let dijkstra_result = self.run_dijkstra(graph, source)?;
        let dijkstra_time = start.elapsed();
        
        // Run FastSSSP with HubSplit
        let start = Instant::now();
        let fast_sssp_result = self.run_fast_sssp_with_hubsplit(graph, source)?;
        let fast_sssp_time = start.elapsed();
        
        // Run simplified FastSSSP
        let start = Instant::now();
        let simple_result = self.run_simplified_fast_sssp(graph, source)?;
        let simple_time = start.elapsed();
        
        // Compare times and update statistics
        let dijkstra_ms = dijkstra_time.as_secs_f64() * 1000.0;
        let fast_sssp_ms = fast_sssp_time.as_secs_f64() * 1000.0;
        let simple_ms = simple_time.as_secs_f64() * 1000.0;
        
        if self.verbose {
            println!("Performance comparison:");
            println!("  Dijkstra: {:.2} ms", dijkstra_ms);
            println!("  FastSSSP with HubSplit: {:.2} ms", fast_sssp_ms);
            println!("  Simplified FastSSSP: {:.2} ms", simple_ms);
        }
        
        // Update statistics
        if self.collect_stats {
            let mut stats = self.stats.borrow_mut();
            stats.total_runs += 1;
            
            if dijkstra_ms <= fast_sssp_ms && dijkstra_ms <= simple_ms {
                stats.dijkstra_wins += 1;
                let speedup = fast_sssp_ms.min(simple_ms) / dijkstra_ms;
                stats.avg_dijkstra_speedup = 
                    (stats.avg_dijkstra_speedup * (stats.dijkstra_wins - 1) as f64 + speedup) 
                    / stats.dijkstra_wins as f64;
                
                if self.verbose {
                    println!("  Winner: Dijkstra (speedup: {:.2}x)", speedup);
                }
                
                return Ok(dijkstra_result);
            } else if fast_sssp_ms <= dijkstra_ms && fast_sssp_ms <= simple_ms {
                stats.fast_sssp_wins += 1;
                let speedup = dijkstra_ms / fast_sssp_ms;
                stats.avg_fast_sssp_speedup = 
                    (stats.avg_fast_sssp_speedup * (stats.fast_sssp_wins - 1) as f64 + speedup) 
                    / stats.fast_sssp_wins as f64;
                
                if self.verbose {
                    println!("  Winner: FastSSSP with HubSplit (speedup: {:.2}x)", speedup);
                }
                
                return Ok(fast_sssp_result);
            } else {
                stats.simple_fast_sssp_wins += 1;
                let speedup = dijkstra_ms / simple_ms;
                stats.avg_simple_speedup = 
                    (stats.avg_simple_speedup * (stats.simple_fast_sssp_wins - 1) as f64 + speedup) 
                    / stats.simple_fast_sssp_wins as f64;
                
                if self.verbose {
                    println!("  Winner: Simplified FastSSSP (speedup: {:.2}x)", speedup);
                }
                
                return Ok(simple_result);
            }
        }
        
        // Default to returning the fastest result
        if dijkstra_ms <= fast_sssp_ms && dijkstra_ms <= simple_ms {
            return Ok(dijkstra_result);
        } else if fast_sssp_ms <= dijkstra_ms && fast_sssp_ms <= simple_ms {
            return Ok(fast_sssp_result);
        } else {
            return Ok(simple_result);
        }
    }
}

impl<W, G> ShortestPathAlgorithm<W, G> for SmartSSSP
where
    W: Float + Zero + Debug + Copy + Ord,
    G: Graph<W> + Clone + crate::graph::traits::MutableGraph<W> + crate::graph::traits::ToConstantDegree<W>,
{
    fn name(&self) -> &'static str {
        "SmartSSSP"
    }
    
    fn compute_shortest_paths(&self, graph: &G, source: usize) -> Result<ShortestPathResult<W>> {
        if !graph.has_vertex(source) {
            return Err(Error::SourceNotFound);
        }
        
        match self.mode {
            SmartMode::ForceDijkstra => self.run_dijkstra(graph, source),
            SmartMode::ForceFastSSSP => self.run_fast_sssp_with_hubsplit(graph, source),
            SmartMode::SimpleFastSSSP => self.run_simplified_fast_sssp(graph, source),
            SmartMode::Auto => self.run_best_algorithm(graph, source),
            SmartMode::Adaptive => self.run_adaptive(graph, source),
        }
    }
}

// Note: We removed the specialized implementation for OrderedFloat<f64> as it conflicts with the generic one
