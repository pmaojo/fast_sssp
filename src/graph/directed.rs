use crate::graph::traits::{Graph, MutableGraph, ToConstantDegree};
use num_traits::{Float, Zero};
use std::collections::HashMap;
use std::fmt::Debug;

/// A directed graph implementation using adjacency lists
#[derive(Debug, Clone)]
pub struct DirectedGraph<W> 
where
    W: Float + Zero + Debug + Copy,
{
    /// Number of vertices in the graph
    vertex_count: usize,
    
    /// Outgoing edges for each vertex: vertex_id -> [(target_vertex, weight)]
    outgoing_edges: HashMap<usize, Vec<(usize, W)>>,
    
    /// Incoming edges for each vertex: vertex_id -> [(source_vertex, weight)]
    incoming_edges: HashMap<usize, Vec<(usize, W)>>,
}

impl<W> DirectedGraph<W> 
where
    W: Float + Zero + Debug + Copy,
{
    /// Creates a new empty directed graph
    pub fn new() -> Self {
        DirectedGraph {
            vertex_count: 0,
            outgoing_edges: HashMap::new(),
            incoming_edges: HashMap::new(),
        }
    }
    
    /// Creates a new directed graph with the specified number of vertices
    pub fn with_capacity(vertices: usize) -> Self {
        let mut graph = DirectedGraph {
            vertex_count: vertices,
            outgoing_edges: HashMap::with_capacity(vertices),
            incoming_edges: HashMap::with_capacity(vertices),
        };
        
        // Initialize empty edge lists for each vertex
        for v in 0..vertices {
            graph.outgoing_edges.insert(v, Vec::new());
            graph.incoming_edges.insert(v, Vec::new());
        }
        
        graph
    }
    
    // Legacy method - use the trait implementation instead
    pub fn to_constant_degree_legacy(&self) -> Self {
        let (graph, _, _) = self.to_constant_degree();
        graph
    }
    
    /// Validate that the graph doesn't have negative weights
    pub fn validate_non_negative(&self) -> bool {
        for (_vertex, edges) in &self.outgoing_edges {
            for (_target, weight) in edges {
                if *weight < W::zero() {
                    return false;
                }
            }
        }
        true
    }
}

impl<W> Graph<W> for DirectedGraph<W> 
where
    W: Float + Zero + Debug + Copy,
{
    fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    
    fn edge_count(&self) -> usize {
        self.outgoing_edges.values().map(|edges| edges.len()).sum()
    }
    
    fn outgoing_edges(&self, vertex: usize) -> Box<dyn Iterator<Item = (usize, W)> + '_> {
        if let Some(edges) = self.outgoing_edges.get(&vertex) {
            Box::new(edges.iter().cloned())
        } else {
            Box::new(std::iter::empty())
        }
    }
    
    fn incoming_edges(&self, vertex: usize) -> Box<dyn Iterator<Item = (usize, W)> + '_> {
        if let Some(edges) = self.incoming_edges.get(&vertex) {
            Box::new(edges.iter().cloned())
        } else {
            Box::new(std::iter::empty())
        }
    }
    
    fn has_vertex(&self, vertex: usize) -> bool {
        vertex < self.vertex_count
    }
    
    fn has_edge(&self, from: usize, to: usize) -> bool {
        if let Some(edges) = self.outgoing_edges.get(&from) {
            edges.iter().any(|(target, _)| *target == to)
        } else {
            false
        }
    }
    
    fn get_edge_weight(&self, from: usize, to: usize) -> Option<W> {
        if let Some(edges) = self.outgoing_edges.get(&from) {
            edges.iter().find(|(target, _)| *target == to).map(|(_, weight)| *weight)
        } else {
            None
        }
    }
}

impl<W> MutableGraph<W> for DirectedGraph<W> 
where
    W: Float + Zero + Debug + Copy,
{
    fn add_vertex(&mut self) -> usize {
        let new_id = self.vertex_count;
        self.outgoing_edges.insert(new_id, Vec::new());
        self.incoming_edges.insert(new_id, Vec::new());
        self.vertex_count += 1;
        new_id
    }
    
    fn remove_vertex(&mut self, vertex: usize) -> bool {
        if !self.has_vertex(vertex) {
            return false;
        }
        
        // Remove all edges connected to this vertex
        if let Some(outgoing) = self.outgoing_edges.remove(&vertex) {
            for (target, _) in outgoing {
                if let Some(incoming) = self.incoming_edges.get_mut(&target) {
                    incoming.retain(|(source, _)| *source != vertex);
                }
            }
        }
        
        if let Some(incoming) = self.incoming_edges.remove(&vertex) {
            for (source, _) in incoming {
                if let Some(outgoing) = self.outgoing_edges.get_mut(&source) {
                    outgoing.retain(|(target, _)| *target != vertex);
                }
            }
        }
        
        // Note: We don't reduce vertex_count to avoid re-indexing all vertices
        true
    }
    
    fn add_edge(&mut self, from: usize, to: usize, weight: W) -> bool {
        if !self.has_vertex(from) || !self.has_vertex(to) || weight < W::zero() {
            return false;
        }
        
        // Check if edge already exists and update it if it does
        if let Some(outgoing) = self.outgoing_edges.get_mut(&from) {
            for edge in outgoing.iter_mut() {
                if edge.0 == to {
                    edge.1 = weight;
                    
                    // Also update incoming edge
                    if let Some(incoming) = self.incoming_edges.get_mut(&to) {
                        for edge in incoming.iter_mut() {
                            if edge.0 == from {
                                edge.1 = weight;
                                return true;
                            }
                        }
                    }
                    return true;
                }
            }
            
            // Edge doesn't exist, add it
            outgoing.push((to, weight));
        } else {
            self.outgoing_edges.insert(from, vec![(to, weight)]);
        }
        
        // Update incoming edges
        if let Some(incoming) = self.incoming_edges.get_mut(&to) {
            incoming.push((from, weight));
        } else {
            self.incoming_edges.insert(to, vec![(from, weight)]);
        }
        
        true
    }
    
    fn remove_edge(&mut self, from: usize, to: usize) -> bool {
        let mut removed = false;
        
        // Remove from outgoing edges
        if let Some(outgoing) = self.outgoing_edges.get_mut(&from) {
            let len_before = outgoing.len();
            outgoing.retain(|(target, _)| *target != to);
            removed = len_before > outgoing.len();
        }
        
        // Remove from incoming edges
        if let Some(incoming) = self.incoming_edges.get_mut(&to) {
            incoming.retain(|(source, _)| *source != from);
        }
        
        removed
    }
    
    fn update_edge_weight(&mut self, from: usize, to: usize, weight: W) -> bool {
        if !self.has_vertex(from) || !self.has_vertex(to) || weight < W::zero() {
            return false;
        }
        
        let mut updated = false;
        
        // Update outgoing edge
        if let Some(outgoing) = self.outgoing_edges.get_mut(&from) {
            for edge in outgoing.iter_mut() {
                if edge.0 == to {
                    edge.1 = weight;
                    updated = true;
                    break;
                }
            }
        }
        
        // Update incoming edge
        if let Some(incoming) = self.incoming_edges.get_mut(&to) {
            for edge in incoming.iter_mut() {
                if edge.0 == from {
                    edge.1 = weight;
                    updated = true;
                    break;
                }
            }
        }
        
        updated
    }
}

// Implementation of ToConstantDegree for DirectedGraph
impl<W> ToConstantDegree<W> for DirectedGraph<W> 
where
    W: Float + Zero + Debug + Copy,
{
    fn to_constant_degree(&self) -> (Self, Vec<Vec<usize>>, Vec<usize>) {
        let mut result = DirectedGraph::new();
        let mut original_to_transformed = Vec::with_capacity(self.vertex_count());
        let mut transformed_to_original = Vec::new();
        
        // Optimization: Pre-allocate with estimated size
        let estimated_transformed_size = self.vertex_count() * 3; // Rough estimate
        transformed_to_original.reserve(estimated_transformed_size);
        
        // Create cycle vertices only for very high-degree vertices
        // Increasing threshold to reduce transformation overhead
        const DEGREE_THRESHOLD: usize = 16; // Only create cycles for vertices with degree > threshold
        
        // Pre-calculate degrees for all vertices to avoid repeated lookups
        let mut vertex_degrees = Vec::with_capacity(self.vertex_count());
        for v in 0..self.vertex_count() {
            let outgoing_count = self.outgoing_edges.get(&v).map_or(0, |e| e.len());
            let incoming_count = self.incoming_edges.get(&v).map_or(0, |e| e.len());
            vertex_degrees.push(outgoing_count + incoming_count);
        }
        
        // First pass: create all vertices
        for v in 0..self.vertex_count() {
            let total_degree = vertex_degrees[v];
            let mut cycle_vertices = Vec::new();
            
            if total_degree > DEGREE_THRESHOLD {
                // For high-degree vertices, create a cycle with optimized size
                // Use logarithmic scaling to reduce the number of cycle vertices for very high degrees
                let degree_factor = if total_degree > 100 {
                    (total_degree as f64).log2().ceil() as usize
                } else {
                    (total_degree as f64).sqrt().ceil() as usize / 2
                };
                
                let cycle_size = degree_factor.max(2).min(16); // Limit max cycle size
                
                // Create cycle vertices in bulk
                let start_idx = result.vertex_count();
                for _ in 0..cycle_size {
                    result.add_vertex();
                }
                
                // Record cycle vertices
                for i in 0..cycle_size {
                    let vertex_idx = start_idx + i;
                    cycle_vertices.push(vertex_idx);
                }
                
                // Extend transformed_to_original in one operation
                if result.vertex_count() > transformed_to_original.len() {
                    transformed_to_original.resize(result.vertex_count(), 0);
                }
                
                // Set all mappings at once
                for &vertex_idx in &cycle_vertices {
                    transformed_to_original[vertex_idx] = v;
                }
            } else {
                // For low-degree vertices, just create a single vertex (no cycle)
                let new_vertex = result.add_vertex();
                cycle_vertices.push(new_vertex);
                
                if new_vertex >= transformed_to_original.len() {
                    transformed_to_original.resize(new_vertex + 1, 0);
                }
                transformed_to_original[new_vertex] = v;
            }
            
            original_to_transformed.push(cycle_vertices);
        }
        
        // Second pass: add cycle edges (zero-weight) for high-degree vertices
        for v in 0..self.vertex_count() {
            let cycle_vertices = &original_to_transformed[v];
            if cycle_vertices.len() > 1 {
                // Connect cycle vertices with zero-weight edges
                // Use a more efficient loop for adding cycle edges
                let cycle_len = cycle_vertices.len();
                for i in 0..cycle_len {
                    let from = cycle_vertices[i];
                    let to = cycle_vertices[(i + 1) % cycle_len];
                    result.add_edge(from, to, W::zero());
                }
            }
        }
        
        // Third pass: add edges between cycles using larger batches
        let mut edge_batch = Vec::with_capacity(10000); // Larger batch size for better performance
        
        for u in 0..self.vertex_count() {
            let u_vertices = &original_to_transformed[u];
            
            // Get outgoing edges
            if let Some(edges) = self.outgoing_edges.get(&u) {
                // Skip processing if no edges
                if edges.is_empty() {
                    continue;
                }
                
                for &(v, weight) in edges {
                    let v_vertices = &original_to_transformed[v];
                    
                    // Optimization: For single-vertex mappings, use direct connection
                    if u_vertices.len() == 1 && v_vertices.len() == 1 {
                        edge_batch.push((u_vertices[0], v_vertices[0], weight));
                    } else {
                        // For cycle vertices, use hash-based distribution
                        let hash_val = u.wrapping_mul(31).wrapping_add(v);
                        let u_idx = hash_val % u_vertices.len();
                        let v_idx = (hash_val >> 4) % v_vertices.len();
                        
                        edge_batch.push((u_vertices[u_idx], v_vertices[v_idx], weight));
                    }
                    
                    // Process batch when it gets large
                    if edge_batch.len() >= 10000 {
                        for (from, to, w) in edge_batch.drain(..) {
                            result.add_edge(from, to, w);
                        }
                    }
                }
            }
        }
        
        // Process any remaining edges in the batch
        for (from, to, w) in edge_batch {
            result.add_edge(from, to, w);
        }
        
        (result, original_to_transformed, transformed_to_original)
    }
}
