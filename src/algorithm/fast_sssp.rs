use std::fmt::Debug;
use num_traits::{Float, Zero};

use crate::graph::{Graph, GraphTransform, MutableGraph};
use crate::graph::traits::ToConstantDegree;
use crate::graph::hub_split::HubSplit;
use crate::algorithm::{ShortestPathAlgorithm, ShortestPathResult};
use crate::algorithm::bmssp::BMSSP;
use crate::algorithm::dijkstra::Dijkstra;
use crate::{Error, Result};

/// Defines how to handle graph degree transformation
#[derive(Debug, Clone, Copy)]
pub enum DegreeMode {
    /// Automatically determine whether to apply transformation based on sampling
    Auto { delta: usize },
    /// Force constant degree transformation (classic Frederickson approach)
    ForceConst,
    /// No transformation, use original graph
    None,
}

/// Implementation of the O(m log^(2/3) n) single-source shortest path algorithm
/// as described in the paper "Breaking the Sorting Barrier for Directed Single-Source Shortest Paths"
#[derive(Debug)]
pub struct FastSSSP {
    /// How to handle graph degree transformation
    degree_mode: DegreeMode,
    /// Threshold for number of vertices to use FastSSSP over Dijkstra
    vertex_threshold: usize,
    /// Fraction of vertices considered "small reach" to fall back to Dijkstra (0 disables the check)
    small_reach_fraction: f64,
    /// Whether to run a Dijkstra reachability sweep after BMSSP to fix gaps
    enable_reachability_sweep: bool,
}

impl FastSSSP {
    /// Create a new FastSSSP algorithm instance with default settings
    pub fn new() -> Self {
        FastSSSP {
            vertex_threshold: 10_000,
            degree_mode: DegreeMode::None,
            small_reach_fraction: 0.05,
            enable_reachability_sweep: false,
        }
    }
    
    /// Create a new FastSSSP algorithm instance with specified degree mode
    pub fn new_with_mode(mode: DegreeMode) -> Self {
        FastSSSP {
            degree_mode: mode,
            vertex_threshold: 10_000,
            small_reach_fraction: 0.05,
            enable_reachability_sweep: false,
        }
    }
    
    /// Set the degree mode for graph transformation
    pub fn with_degree_mode(mut self, mode: DegreeMode) -> Self {
        self.degree_mode = mode;
        self
    }
    
    /// Set the vertex threshold for choosing between FastSSSP and Dijkstra
    pub fn with_vertex_threshold(mut self, threshold: usize) -> Self {
        self.vertex_threshold = threshold;
        self
    }
    
    /// Set the fraction threshold for small reach fallback (0 disables)
    pub fn with_small_reach_fraction(mut self, fraction: f64) -> Self {
        self.small_reach_fraction = fraction.max(0.0);
        self
    }

    /// Enable or disable the reachability Dijkstra sweep after BMSSP
    pub fn with_reachability_sweep(mut self, enabled: bool) -> Self {
        self.enable_reachability_sweep = enabled;
        self
    }
    
    /// Maps a result from the transformed graph back to the original graph
    #[allow(dead_code)]
    fn map_result_to_original<W, G, T>(
        &self,
        transformer: &T,
        result: &mut ShortestPathResult<W>,
        original_vertex_count: usize,
        original_source: usize,
    )
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W> + Clone + MutableGraph<W>,
        T: GraphTransform<W, G>,
    {
        // Initialize distances and predecessors for the original graph
        let mut original_distances = vec![None; original_vertex_count];
        let mut original_predecessors = vec![None; original_vertex_count];
        
        // Process each vertex in the transformed graph
        for v in 0..result.distances.len() {
            if let Some(dist) = result.distances[v] {
                let original_v = transformer.map_vertex_to_original(v);
                
                // Update distance if it's better than what we have
                match original_distances[original_v] {
                    None => {
                        original_distances[original_v] = Some(dist);
                        
                        // Map predecessor if it exists
                        if let Some(pred) = result.predecessors[v] {
                            let original_pred = transformer.map_vertex_to_original(pred);
                            original_predecessors[original_v] = Some(original_pred);
                        }
                    },
                    Some(current_dist) if dist < current_dist => {
                        original_distances[original_v] = Some(dist);
                        
                        // Map predecessor if it exists
                        if let Some(pred) = result.predecessors[v] {
                            let original_pred = transformer.map_vertex_to_original(pred);
                            original_predecessors[original_v] = Some(original_pred);
                        }
                    },
                    _ => {}, // Keep existing better distance
                }
            }
        }
        
        // Update the result with the original graph data
        result.distances = original_distances;
        result.predecessors = original_predecessors;
        result.source = original_source;
    }
    
    /// Approximates the number of reachable vertices from a source using BFS with a limit
    fn approx_reachable<W, G>(&self, graph: &G, source: usize, limit: usize) -> usize
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W>,
    {
        use std::collections::VecDeque;
        
        let mut visited = vec![false; graph.vertex_count()];
        let mut queue = VecDeque::new();
        let mut count = 0;
        
        visited[source] = true;
        queue.push_back(source);
        count += 1;
        
        // Run BFS with a limit to approximate reachability
        while let Some(v) = queue.pop_front() {
            if count >= limit {
                // We've reached our sampling limit, extrapolate
                return (count * graph.vertex_count()) / limit;
            }
            
            for (u, _) in graph.outgoing_edges(v) {
                if !visited[u] {
                    visited[u] = true;
                    queue.push_back(u);
                    count += 1;
                }
            }
        }
        
        // If we've explored the entire component without hitting the limit
        count
    }
    
    /// Quick estimation of reachable vertices with limited hop count and edge scan limit
    /// This is more efficient for large graphs with small reachable components
    pub fn quick_reach_estimate<W, G>(&self, g: &G, s: usize, max_hops: usize, max_scan: usize) -> usize
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W>,
    {
        use std::collections::VecDeque;
        let mut q = VecDeque::new();
        let mut seen = vec![false; g.vertex_count()];
        q.push_back((s, 0)); 
        seen[s] = true;
        let mut visited = 1;
        let mut scanned = 0;

        while let Some((u, d)) = q.pop_front() {
            if d >= max_hops { continue; }
            for (v, _) in g.outgoing_edges(u) {
                scanned += 1;
                if scanned > max_scan { return visited; }
                if !seen[v] { 
                    seen[v] = true; 
                    visited += 1; 
                    q.push_back((v, d+1)); 
                }
            }
        }
        visited
    }
    
    /// Estimate the maximum degree of vertices reachable from the source
    fn est_max_degree_reachable<W, G>(&self, g: &G, s: usize, max_scan: usize) -> usize
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W>,
    {
        use std::collections::VecDeque;
        let mut q = VecDeque::new();
        let mut seen = vec![false; g.vertex_count()];
        q.push_back(s); 
        seen[s] = true;
        let mut scanned = 0;
        let mut max_degree = 0;

        while let Some(u) = q.pop_front() {
            // Check degree of this vertex
            let out_deg = g.outgoing_edges(u).count();
            let in_deg = g.incoming_edges(u).count();
            max_degree = max_degree.max(out_deg).max(in_deg);
            
            // Scan neighbors
            for (v, _) in g.outgoing_edges(u) {
                scanned += 1;
                if scanned > max_scan { return max_degree; }
                if !seen[v] { 
                    seen[v] = true;
                    q.push_back(v); 
                }
            }
        }
        max_degree
    }
    
    /// Choose optimal k and t parameters based on reachable component size
    fn choose_params(&self, reach_est: usize, avg_deg: f64) -> (usize, usize) {
        // Base k on square root of reachable vertices
        let k = (reach_est as f64).sqrt().ceil() as usize;
        // t should be roughly 2*log2(k)
        let t = (2.0 * (k as f64).log2()).ceil() as usize;
        
        // Apply reasonable bounds
        let k_bounded = k.max(8).min(64);  
        let t_bounded = t.max(4).min(32);
        
        (k_bounded, t_bounded)
    }
    
    /// Checks for reachability issues and fixes them by running a Dijkstra sweep
    fn check_reachability<W, G>(&self, graph: &G, source: usize, result: &mut ShortestPathResult<W>) -> Result<()>
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W>,
    {
        use std::collections::BinaryHeap;
        use std::cmp::Reverse;
        
        let n = graph.vertex_count();
        
        // Convert Option<W> distances to W with max_value for None
        let mut working_distances = vec![W::max_value(); n];
        let mut working_predecessors = vec![None; n];
        
        // Initialize with known distances
        for (i, dist) in result.distances.iter().enumerate() {
            if let Some(d) = dist {
                working_distances[i] = *d;
                working_predecessors[i] = result.predecessors[i];
            }
        }
        
        // Set source distance to 0
        working_distances[source] = W::zero();
        working_predecessors[source] = Some(source);
        
        // Run a Dijkstra-like algorithm to find unreachable vertices
        let mut queue = BinaryHeap::new();
        queue.push(Reverse((working_distances[source], source)));
        
        let mut visited = vec![false; n];
        
        while let Some(Reverse((dist_u, u))) = queue.pop() {
            // Skip if already visited
            if visited[u] {
                continue;
            }
            visited[u] = true;
            
            // Skip if we've found a better path
            if dist_u > working_distances[u] {
                continue;
            }
            
            // Relax outgoing edges
            for (v, weight) in graph.outgoing_edges(u) {
                let new_dist = dist_u + weight;
                if new_dist < working_distances[v] {
                    working_distances[v] = new_dist;
                    working_predecessors[v] = Some(u);
                    queue.push(Reverse((new_dist, v)));
                }
            }
        }
        
        // Update the result with fixed distances and predecessors
        for i in 0..n {
            if working_distances[i] < W::max_value() {
                result.distances[i] = Some(working_distances[i]);
                result.predecessors[i] = working_predecessors[i];
            } else {
                result.distances[i] = None;
                result.predecessors[i] = None;
            }
        }
        
        // Apply Dijkstra's algorithm on the original graph
        let dijkstra = Dijkstra::new();
        let dijkstra_result = dijkstra.compute_shortest_paths(graph, source)?;
        
        // For each vertex that was unreachable in our result but reachable in Dijkstra's result,
        // update our result with Dijkstra's findings
        for v in 0..graph.vertex_count() {
            if result.distances[v].is_none() && dijkstra_result.distances[v].is_some() {
                result.distances[v] = dijkstra_result.distances[v];
                result.predecessors[v] = dijkstra_result.predecessors[v];
            }
        }
        
        Ok(())
    }
    
    /// Computes a path to a specific target vertex
    #[allow(dead_code)]
    fn compute_path_to_target<W, G>(
        &self,
        graph: &G,
        source: usize,
        target: usize,
        distances: &mut Vec<W>,
        predecessors: &mut Vec<Option<usize>>,
    ) -> bool
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W>,
    {
        use std::collections::BinaryHeap;
        use std::cmp::Reverse;
        
        // Initialize priority queue with source
        let mut queue = BinaryHeap::new();
        queue.push(Reverse((distances[source], source)));
        
        // Track visited vertices to avoid cycles
        let mut visited = vec![false; graph.vertex_count()];
        
        while let Some(Reverse((dist_u, u))) = queue.pop() {
            // If we've reached the target, we're done
            if u == target {
                return true;
            }
            
            // Skip if already visited
            if visited[u] {
                continue;
            }
            visited[u] = true;
            
            // Skip if we've found a better path
            if dist_u > distances[u] {
                continue;
            }
            
            // Relax outgoing edges
            for (v, weight) in graph.outgoing_edges(u) {
                let new_dist = dist_u + weight;
                if new_dist < distances[v] {
                    distances[v] = new_dist;
                    predecessors[v] = Some(u);
                    queue.push(Reverse((new_dist, v)));
                }
            }
        }
        
        // Target not reachable
        false
    }
}

/// Transformer that converts a graph to constant degree as described in the paper
/// This implementation splits high-degree vertices into chains of constant-degree vertices
#[derive(Clone)]
pub struct ConstantDegreeTransformer<W, G>
where
    W: Float + Zero + Debug + Copy + Ord,
    G: Graph<W> + Clone + MutableGraph<W>,
{
    /// Maps transformed vertices back to original vertices
    pub vertex_to_original: Vec<usize>,
    /// Maps original vertices to lists of transformed vertices
    pub original_to_vertices: Vec<Vec<usize>>,
    /// Weight type marker
    _weight_marker: std::marker::PhantomData<W>,
    /// Graph type marker
    _graph_marker: std::marker::PhantomData<G>,
}

impl<W, G> ConstantDegreeTransformer<W, G>
where
    W: Float + Zero + Debug + Copy + Ord,
    G: Graph<W> + Clone + MutableGraph<W>,
{
    /// Creates a new ConstantDegreeTransformer
    pub fn new() -> Self {
        ConstantDegreeTransformer {
            vertex_to_original: Vec::new(),
            original_to_vertices: Vec::new(),
            _weight_marker: std::marker::PhantomData,
            _graph_marker: std::marker::PhantomData,
        }
    }
}

impl<W, G> GraphTransform<W, G> for ConstantDegreeTransformer<W, G>
where
    W: Float + Zero + Debug + Copy + Ord,
    G: Graph<W> + Clone + MutableGraph<W>,
{
    fn transform(&self, graph: &G) -> G {
        let n = graph.vertex_count();
        let mut transformed_graph = graph.clone();
        
        // Create a new transformer with proper capacity
        let mut vertex_to_original = (0..n).collect::<Vec<_>>();
        let mut original_to_vertices = (0..n).map(|i| vec![i]).collect::<Vec<_>>();
        
        // Process each vertex with high out-degree
        for v in 0..n {
            let out_edges: Vec<_> = graph.outgoing_edges(v).collect();
            
            // If out-degree is already small enough, skip
            if out_edges.len() <= 3 {
                continue;
            }
            
            // Remove all existing outgoing edges
            for (u, _) in out_edges.iter() {
                transformed_graph.remove_edge(v, *u);
            }
            
            // Create a chain of vertices to handle the high out-degree
            let mut chain_vertices = vec![v];
            let chunks = out_edges.chunks(2); // Each vertex in chain handles at most 2 outgoing edges
            
            // Create additional vertices for the chain (skip first chunk which is handled by v)
            for _ in 1..chunks.len() {
                let new_v = transformed_graph.add_vertex();
                chain_vertices.push(new_v);
                
                // Update mappings
                vertex_to_original.push(v);
                original_to_vertices[v].push(new_v);
            }
            
            // Connect chain vertices with zero-weight edges
            for i in 0..chain_vertices.len()-1 {
                transformed_graph.add_edge(chain_vertices[i], chain_vertices[i+1], W::zero());
            }
            
            // Distribute outgoing edges among chain vertices
            for (i, chunk) in chunks.enumerate() {
                let chain_v = chain_vertices[i];
                for (u, weight) in chunk {
                    transformed_graph.add_edge(chain_v, *u, *weight);
                }
            }
        }
        
        // Process each vertex with high in-degree
        for v in 0..n {
            let in_edges: Vec<_> = graph.incoming_edges(v).collect();
            
            // If in-degree is already small enough, skip
            if in_edges.len() <= 3 {
                continue;
            }
            
            // Remove all existing incoming edges
            for (u, _) in in_edges.iter() {
                transformed_graph.remove_edge(*u, v);
            }
            
            // Create a tree of vertices to handle the high in-degree
            let mut tree_vertices = vec![v];
            let chunks = in_edges.chunks(2); // Each vertex in tree handles at most 2 incoming edges
            
            // Create additional vertices for the tree (skip first chunk which is handled by v)
            for _ in 1..chunks.len() {
                let new_v = transformed_graph.add_vertex();
                tree_vertices.push(new_v);
                
                // Update mappings
                vertex_to_original.push(v);
                original_to_vertices[v].push(new_v);
            }
            
            // Connect tree vertices with zero-weight edges
            for i in 1..tree_vertices.len() {
                transformed_graph.add_edge(tree_vertices[i], v, W::zero());
            }
            
            // Distribute incoming edges among tree vertices
            for (i, chunk) in chunks.enumerate() {
                let tree_v = tree_vertices[i];
                for (u, weight) in chunk {
                    transformed_graph.add_edge(*u, tree_v, *weight);
                }
            }
        }
        
        // Create a new transformer with the mappings
        // We can't modify self directly since it's behind a shared reference
        // Instead, we create a new transformer with the mappings and attach it to the graph
        let mut new_transformer = ConstantDegreeTransformer::<W, G>::new();
        new_transformer.vertex_to_original = vertex_to_original;
        new_transformer.original_to_vertices = original_to_vertices;
        
        // We can't return the transformer directly, so we'll store it somewhere
        // For now, just return the transformed graph
        transformed_graph
    }
    
    fn map_vertex_to_original(&self, transformed_vertex: usize) -> usize {
        if transformed_vertex < self.vertex_to_original.len() {
            self.vertex_to_original[transformed_vertex]
        } else {
            transformed_vertex // Default to identity mapping if not found
        }
    }
    
    fn map_vertex_from_original(&self, original_vertex: usize) -> Vec<usize> {
        if original_vertex < self.original_to_vertices.len() {
            self.original_to_vertices[original_vertex].clone()
        } else {
            vec![original_vertex] // Default to identity mapping if not found
        }
    }
}

// Helper methods to compute shortest paths using Dijkstra's algorithm
impl FastSSSP {
    /// Runs Dijkstra's algorithm directly on the graph
    fn run_dijkstra<W, G>(&self, graph: &G, source: usize) -> Result<ShortestPathResult<W>>
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W> + Clone,
    {
        println!("Using Dijkstra's algorithm directly");
        let dijkstra = Dijkstra::new();
        dijkstra.compute_shortest_paths(graph, source)
    }
    
    fn compute_dijkstra<W, G>(&self, graph: &G, source: usize, distances: &mut Vec<W>, predecessors: &mut Vec<Option<usize>>) -> Result<ShortestPathResult<W>>
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W> + Clone,
    {
        use std::collections::BinaryHeap;
        use std::cmp::Reverse;
        
        // Initialize priority queue
        let mut queue = BinaryHeap::new();
        queue.push(Reverse((distances[source], source)));
        
        // Run Dijkstra's algorithm
        while let Some(Reverse((dist_u, u))) = queue.pop() {
            // Skip if we've found a better path
            if dist_u > distances[u] {
                continue;
            }
            
            // Relax outgoing edges
            for (v, weight) in graph.outgoing_edges(u) {
                let new_dist = dist_u + weight;
                if new_dist < distances[v] {
                    distances[v] = new_dist;
                    predecessors[v] = Some(u);
                    queue.push(Reverse((new_dist, v)));
                }
            }
        }
        
        // Convert to ShortestPathResult format
        let final_distances: Vec<Option<W>> = distances
            .iter()
            .map(|&d| if d == W::max_value() { None } else { Some(d) })
            .collect();
        
        Ok(ShortestPathResult {
            distances: final_distances,
            predecessors: predecessors.clone(),
            source,
        })
    }
}

#[allow(clippy::redundant_clone)]
impl<W, G> ShortestPathAlgorithm<W, G> for FastSSSP
where
    W: Float + Zero + Debug + Copy + Ord,
    G: Graph<W> + Clone + MutableGraph<W> + ToConstantDegree<W>,
{
    fn name(&self) -> &'static str {
        "Fast SSSP (O(m log^(2/3) n)) or Dijkstra"
    }
    
    fn compute_shortest_paths(&self, graph: &G, source: usize) -> Result<ShortestPathResult<W>> {
        if !graph.has_vertex(source) {
            return Err(Error::SourceNotFound);
        }
        
        // Check for negative weights
        for v in 0..graph.vertex_count() {
            for (_, weight) in graph.outgoing_edges(v) {
                if weight < W::zero() {
                    return Err(Error::NegativeWeight(weight.to_f64().unwrap_or(0.0)));
                }
            }
        }
        
        let n = graph.vertex_count();
        println!("Computing shortest paths for graph with {} vertices from source {}", n, source);
        
        // Apply graph transformation based on the degree mode
        let (working_graph, transformed_to_original, working_source) = match self.degree_mode {
            DegreeMode::ForceConst => {
                println!("Converting graph to constant-degree representation (classic approach)");
                // Clone the graph first since we need to call a method on the owned value
                let graph_owned = graph.clone();
                let (transformed_graph, original_to_transformed, transformed_to_original) = graph_owned.to_constant_degree();
                
                // Determine the source vertex in the working graph
                let working_source = original_to_transformed[source][0];
                
                (transformed_graph, transformed_to_original, working_source)
            },
            DegreeMode::Auto { delta } => {
                // Sample a small percentage of vertices to see if transformation is needed
                let sample_size = (n as f64 * 0.01).ceil() as usize;
                let mut max_degree = 0;
                
                for i in 0..sample_size {
                    let v = (i * n / sample_size).min(n - 1);
                    let out_deg = graph.outgoing_edges(v).count();
                    let in_deg = graph.incoming_edges(v).count();
                    max_degree = max_degree.max(out_deg).max(in_deg);
                }
                
                if max_degree <= delta {
                    println!("Sampled max degree {} <= delta {}, skipping transformation", max_degree, delta);
                    (graph.clone(), Vec::new(), source)
                } else {
                    println!("Sampled max degree {} > delta {}, applying hub-split transformation", max_degree, delta);
                    let hub_split = HubSplit::new(graph.clone(), delta);
                    hub_split.transform(graph);
                    
                    // Extract the transformed graph and mappings
                    let transformed_to_original = hub_split.vertex_to_original.clone();
                    let original_to_vertices = hub_split.original_to_vertices.clone();
                    
                    // Determine the source vertex in the working graph
                    let working_source = original_to_vertices[source][0];
                    
                    (hub_split.graph, transformed_to_original, working_source)
                }
            },
            DegreeMode::None => {
                println!("Using original graph without transformation");
                (graph.clone(), Vec::new(), source)
            }
        };
        
        // Check if the reachable component is very small
        let reach = self.approx_reachable(&working_graph, working_source, 256);
        if self.small_reach_fraction > 0.0 {
            let threshold = (self.small_reach_fraction * n as f64) as usize;
            if reach < threshold {
                println!("Reachable component is small ({} < {}), using Dijkstra algorithm", reach, threshold);
                return self.run_dijkstra(graph, source);
            }
        }
        
        // Choose algorithm based on graph size
        if n >= self.vertex_threshold {
            println!("Graph is large (n={} >= {}), using FastSSSP algorithm", n, self.vertex_threshold);
            
            // Calculate parameters for BMSSP based on the graph size per paper
            let ln = (n as f64).ln();
            let k = ln.powf(1.0 / 3.0).ceil() as usize;
            let t = ln.powf(2.0 / 3.0).ceil() as usize;
            
            // Level determines the depth of the BMSSP recursion.  It is the
            // ceiling of ln(n) divided by t as suggested in the paper.
            let mut level = ((n as f64).ln() / (t as f64)).ceil() as usize;
            if level < 1 {
                level = 1;
            }
            
            println!("Running BMSSP with parameters k={}, t={}", k, t);
            
            // Run BMSSP on the graph
            let bmssp = BMSSP::new_with_params(working_graph.vertex_count(), k, t);
            
            // Prepare data structures
            let mut distances = vec![W::max_value(); working_graph.vertex_count()];
            let mut predecessors = vec![None; working_graph.vertex_count()];
            
            // Set source distance
            distances[working_source] = W::zero();
            predecessors[working_source] = Some(working_source);
            
            // Execute BMSSP starting from the computed top level
            let _bmssp_result = bmssp.execute(
                &working_graph,
                level,
                W::max_value(),
                &[working_source],
                &mut distances,
                &mut predecessors
            )?;

            // Convert BMSSP result to ShortestPathResult, mapping back to original vertices if needed
            let has_transformation = match self.degree_mode {
                DegreeMode::None => false,
                DegreeMode::Auto { delta: _ } => !transformed_to_original.is_empty(),
                DegreeMode::ForceConst => true,
            };

            if has_transformation {
                // Need to map results back to original graph
                let mut orig_distances = vec![W::max_value(); n];
                let mut orig_predecessors = vec![None; n];

                // Map distances back
                for (transformed_v, dist) in distances.iter().enumerate() {
                    if *dist < W::max_value() && transformed_v < transformed_to_original.len() {
                        let orig_v = transformed_to_original[transformed_v];
                        orig_distances[orig_v] = Float::min(*dist, orig_distances[orig_v]);
                    }
                }

                // Map predecessors back
                for (transformed_v, pred_opt) in predecessors.iter().enumerate() {
                    if let Some(pred) = pred_opt {
                        if distances[transformed_v] < W::max_value() && 
                           transformed_v < transformed_to_original.len() && 
                           *pred < transformed_to_original.len() {
                            let orig_v = transformed_to_original[transformed_v];
                            let orig_pred = transformed_to_original[*pred];

                            // Only update predecessor if we're updating the distance
                            if orig_distances[orig_v] == distances[transformed_v] {
                                orig_predecessors[orig_v] = Some(orig_pred);
                            }
                        }
                    }
                }

                // Create result with mapped values
                let mut result = ShortestPathResult {
                    source,
                    distances: orig_distances.into_iter().map(|d| {
                        if d == W::max_value() { None } else { Some(d) }
                    }).collect(),
                    predecessors: orig_predecessors,
                };

                // Check for reachability issues
                if self.enable_reachability_sweep {
                    self.check_reachability(graph, source, &mut result)?;
                }
                
                Ok(result)
            } else {
                // No transformation, use results directly
                let mut result = ShortestPathResult {
                    source,
                    distances: distances.into_iter().map(|d| {
                        if d == W::max_value() { None } else { Some(d) }
                    }).collect(),
                    predecessors,
                };
                
                // Optionally check and fix reachability issues with a Dijkstra sweep
                if self.enable_reachability_sweep {
                    let _ = self.check_reachability::<W, G>(&working_graph, source, &mut result);
                }
                
                Ok(result)
            }
        } else {
            println!("Graph is small (n={} < {}), using Dijkstra algorithm", n, self.vertex_threshold);
            
            // Use Dijkstra for small graphs (n < vertex_threshold)
            let dijkstra = Dijkstra::new();
            let result = if !transformed_to_original.is_empty() {
                // When using a transformed graph with Dijkstra, we need to transform the result back
                let transformed_result = dijkstra.compute_shortest_paths(&working_graph, working_source)?;
                
                // Map the result back to the original graph
                let mut orig_distances = vec![W::max_value(); n];
                let mut orig_predecessors = vec![None; n];
                
                // Map distances
                for (transformed_v, dist_opt) in transformed_result.distances.iter().enumerate() {
                    if let Some(dist) = dist_opt {
                        let orig_v = transformed_to_original[transformed_v];
                        if orig_distances[orig_v] > *dist {
                            orig_distances[orig_v] = *dist;
                        }
                    }
                }
                
                // Map predecessors
                for (transformed_v, pred_opt) in transformed_result.predecessors.iter().enumerate() {
                    if let Some(transformed_pred) = pred_opt {
                        let orig_v = transformed_to_original[transformed_v];
                        let orig_pred = transformed_to_original[*transformed_pred];
                        
                        // Set predecessor if we have a valid distance for this vertex AND the edge exists in the original graph
                        if orig_distances[orig_v] < W::max_value() && graph.has_edge(orig_pred, orig_v) {
                            orig_predecessors[orig_v] = Some(orig_pred);
                        }
                    }
                }
                
                ShortestPathResult {
                    distances: orig_distances.into_iter().map(|d| {
                        if d == W::max_value() { None } else { Some(d) }
                    }).collect(),
                    predecessors: orig_predecessors,
                    source,
                }
            } else {
                // Using original graph with Dijkstra
                dijkstra.compute_shortest_paths(&working_graph, source)?
            };
            
            Ok(result)
        }
    }
}
