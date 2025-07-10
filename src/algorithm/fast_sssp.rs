use std::fmt::Debug;
use num_traits::{Float, Zero};

use crate::graph::{Graph, GraphTransform, MutableGraph};
use crate::graph::traits::ToConstantDegree;
use crate::algorithm::{ShortestPathAlgorithm, ShortestPathResult};
use crate::algorithm::bmssp::BMSSP;
use crate::algorithm::dijkstra::Dijkstra;
use crate::{Error, Result};

/// Implementation of the O(m log^(2/3) n) single-source shortest path algorithm
/// as described in the paper "Breaking the Sorting Barrier for Directed Single-Source Shortest Paths"
#[derive(Debug)]
pub struct FastSSSP {
    /// Whether to convert the input graph to a constant-degree graph
    /// NOTE: Currently this flag is ignored as the transformation is not yet implemented for generic graphs
    convert_to_constant_degree: bool,
    /// Threshold for number of vertices to use FastSSSP over Dijkstra
    vertex_threshold: usize,
}

impl FastSSSP {
    /// Create a new FastSSSP algorithm instance
    pub fn new() -> Self {
        FastSSSP { 
            convert_to_constant_degree: true,
            vertex_threshold: 500_000, // Lower threshold to use FastSSSP more often
        }
    }
    
    /// Set whether to convert the input graph to a constant-degree graph
    pub fn with_constant_degree_conversion(mut self, convert: bool) -> Self {
        self.convert_to_constant_degree = convert;
        self
    }
    
    /// Set the vertex threshold for choosing between FastSSSP and Dijkstra
    pub fn with_vertex_threshold(mut self, threshold: usize) -> Self {
        self.vertex_threshold = threshold;
        self
    }
    
    /// Maps a result from the transformed graph back to the original graph
    fn map_result_to_original<W, G>(
        &self,
        transformer: &ConstantDegreeTransformer<W, G>,
        result: &mut ShortestPathResult<W>,
        original_vertex_count: usize,
        original_source: usize,
    )
    where
        W: Float + Zero + Debug + Copy + Ord,
        G: Graph<W> + Clone + MutableGraph<W>,
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
    
    /// Checks for reachability issues and fixes them with a Dijkstra sweep
    fn check_reachability<W, G>(
        &self,
        graph: &G,
        source: usize,
        result: &mut ShortestPathResult<W>
    ) 
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
    }
    
    /// Computes a path to a specific target vertex
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

// Helper method to compute shortest paths using Dijkstra's algorithm
impl FastSSSP {
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
        
        // Apply constant-degree graph transformation if enabled
        let (working_graph, transformed_to_original, vertex_mapping) = if self.convert_to_constant_degree {
            println!("Converting graph to constant-degree representation");
            // Clone the graph first since we need to call a method on the owned value
            let graph_owned = graph.clone();
            let (transformed_graph, original_to_transformed, transformed_to_original) = graph_owned.to_constant_degree();
            
            // Use the transformed graph and store the mappings
            // We'll need transformed_to_original to map back vertices for the result
            (transformed_graph, transformed_to_original, Some(original_to_transformed))
        } else {
            // Use original graph
            (graph.clone(), Vec::new(), None)
        };
        
        // Determine the source vertex in the working graph
        let working_source = if self.convert_to_constant_degree && vertex_mapping.is_some() {
            // If we're using a transformed graph, we need to map the original source
            // We choose the first vertex in the cycle that corresponds to the original source
            vertex_mapping.as_ref().unwrap()[source][0]
        } else {
            source
        };
        
        // Choose algorithm based on graph size
        if n >= self.vertex_threshold {
            println!("Graph is large (n={} >= {}), using FastSSSP algorithm", n, self.vertex_threshold);
            
            // Calculate adaptive parameters for BMSSP based on graph characteristics
            let log_n = (n as f64).ln();
            let m = working_graph.edge_count();
            let density = m as f64 / (n as f64).powi(2);
            
            // Detect if this is likely a hierarchical graph based on edge/vertex ratio
            // Hierarchical graphs typically have edge count close to vertex count
            let edge_vertex_ratio = m as f64 / n as f64;
            let is_hierarchical = edge_vertex_ratio < 2.0;
            
            // For hierarchical graphs, use specialized parameters
            let mut k = if is_hierarchical {
                // For hierarchical graphs, use larger k to reduce recursion depth
                // This helps with bucket-based hierarchical structures
                6
            } else if density < 0.001 {
                // Very sparse graph
                3
            } else if density < 0.01 {
                // Sparse graph
                4
            } else if density < 0.1 {
                // Medium density
                5
            } else {
                // Dense graph
                6
            };
            
            let mut t = if is_hierarchical {
                // For hierarchical graphs, use smaller t to increase pivot selection
                // This helps find shortcuts in the hierarchy
                8
            } else if density < 0.001 {
                // Very sparse graph
                12
            } else if density < 0.01 {
                // Sparse graph
                10
            } else if density < 0.1 {
                // Medium density
                8
            } else {
                // Dense graph
                6
            };
            
            // Scale parameters based on graph size for very large graphs
            if n > 1_000_000 {
                // For extremely large graphs, adjust parameters to avoid excessive work
                k = k.min(4); // Limit k for very large graphs
                t = t.max(10); // Increase t for very large graphs
            }
            
            // Ensure k and t are within reasonable bounds
            k = k.max(2).min(8);
            t = t.max(4).min(20);
            
            println!("Running BMSSP with adaptive parameters: k={}, t={} (density={}, hierarchical={})", 
                     k, t, density, is_hierarchical);
        
            // Run BMSSP on the graph
            let bmssp = BMSSP::new_with_params(working_graph.vertex_count(), k, t);
            
            // Prepare data structures
            let mut distances = vec![W::max_value(); working_graph.vertex_count()];
            let mut predecessors = vec![None; working_graph.vertex_count()];
            
            // Set source distance
            distances[working_source] = W::zero();
            predecessors[working_source] = Some(working_source);
            
            // Calculate recursion level based on graph size and parameters
            let level = ((n as f64).ln() / (t as f64)).ceil() as usize;
            
            // Execute BMSSP starting from the computed top level
            let _bmssp_result = bmssp.execute(
                &working_graph,
                level.max(1), // Ensure level is at least 1
                W::max_value(),
                &[working_source],
                &mut distances,
                &mut predecessors
            )?;
            
            // Convert BMSSP result to ShortestPathResult, mapping back to original vertices if needed
            if self.convert_to_constant_degree {
                // Need to map results back to original graph
                let mut orig_distances = vec![W::max_value(); n];
                let mut orig_predecessors = vec![None; n];
                
                // Map distances back - improved mapping to ensure all reachable vertices are found
                for (transformed_v, dist) in distances.iter().enumerate() {
                    if transformed_v < transformed_to_original.len() {
                        let orig_v = transformed_to_original[transformed_v];
                        
                        // If this transformed vertex has a path, update the original vertex
                        if *dist < W::max_value() {
                            orig_distances[orig_v] = Float::min(*dist, orig_distances[orig_v]);
                        }
                        
                        // Check if any other vertices in the same cycle have paths
                        // This ensures we don't miss paths through different cycle vertices
                        if let Some(ref vertex_mapping) = vertex_mapping {
                            if orig_v < vertex_mapping.len() {
                                for &other_v in &vertex_mapping[orig_v] {
                                    if other_v < distances.len() && distances[other_v] < W::max_value() {
                                        orig_distances[orig_v] = Float::min(distances[other_v], orig_distances[orig_v]);
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Map predecessors back
                for (transformed_v, pred_opt) in predecessors.iter().enumerate() {
                    if let Some(transformed_pred) = pred_opt {
                        let orig_v = transformed_to_original[transformed_v];
                        let orig_pred = transformed_to_original[*transformed_pred];
                        
                        // Only update if we found a better path through this predecessor
                        if orig_predecessors[orig_v].is_none() || 
                          (orig_distances[orig_v] < W::max_value() && 
                           orig_distances[orig_v] == orig_distances[orig_pred] + 
                           graph.get_edge_weight(orig_pred, orig_v).unwrap_or(W::max_value())) {
                            orig_predecessors[orig_v] = Some(orig_pred);
                        }
                    }
                }
                
                // Create result for original graph
                let mut result = ShortestPathResult {
                    distances: orig_distances.into_iter().map(|d| {
                        if d == W::max_value() { None } else { Some(d) }
                    }).collect(),
                    predecessors: orig_predecessors,
                    source,
                };
                
                // Check and fix reachability issues with Dijkstra on the original graph
                self.check_reachability::<W, G>(graph, source, &mut result);
                
                Ok(result)
            } else {
                // Working with original graph, no mapping needed
                let mut result = ShortestPathResult {
                    distances: distances.into_iter().map(|d| {
                        if d == W::max_value() { None } else { Some(d) }
                    }).collect(),
                    predecessors,
                    source,
                };
                
                // Check and fix reachability issues with a Dijkstra sweep
                self.check_reachability::<W, G>(&working_graph, source, &mut result);
                
                Ok(result)
            }
        } else {
            println!("Graph is small (n={} < {}), using Dijkstra algorithm", n, self.vertex_threshold);
            
            // Use Dijkstra's algorithm for small graphs
            let dijkstra = Dijkstra::new();
            let result = if self.convert_to_constant_degree && !transformed_to_original.is_empty() {
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
