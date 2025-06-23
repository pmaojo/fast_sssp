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
        
        // Create cycle vertices for each original vertex
        for v in 0..self.vertex_count() {
            // For each original vertex, create vertices for each incoming/outgoing edge
            let mut cycle_vertices = Vec::new();
            
            // Count all neighbors (incoming and outgoing)
            let outgoing_count = self.outgoing_edges.get(&v).map_or(0, |e| e.len());
            let incoming_count = self.incoming_edges.get(&v).map_or(0, |e| e.len());
            
            // Create cycle vertices
            for _ in 0..(outgoing_count + incoming_count).max(1) {
                let new_vertex = result.add_vertex();
                cycle_vertices.push(new_vertex);
                transformed_to_original.resize(new_vertex + 1, 0);
                transformed_to_original[new_vertex] = v;
            }
            
            // Connect cycle vertices with zero-weight edges
            for i in 0..cycle_vertices.len() {
                let from = cycle_vertices[i];
                let to = cycle_vertices[(i + 1) % cycle_vertices.len()];
                result.add_edge(from, to, W::zero());
            }
            
            original_to_transformed.push(cycle_vertices);
        }
        
        // Add edges between cycles
        for u in 0..self.vertex_count() {
            let u_vertices = &original_to_transformed[u];
            
            // Create a vector of outgoing edges
            let mut outgoing_edges = Vec::new();
            if let Some(edges) = self.outgoing_edges.get(&u) {
                for &(v, weight) in edges {
                    outgoing_edges.push((v, weight));
                }
            }
            
            // Add edges to the transformed graph
            for (idx, (v, weight)) in outgoing_edges.iter().enumerate() {
                let v_vertices = &original_to_transformed[*v];
                
                // Use modulo to handle cases where we have more edges than vertices
                let u_idx = idx % u_vertices.len();
                let v_idx = idx % v_vertices.len();
                
                result.add_edge(u_vertices[u_idx], v_vertices[v_idx], *weight);
            }
        }
        
        (result, original_to_transformed, transformed_to_original)
    }
}
