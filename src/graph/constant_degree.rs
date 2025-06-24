use std::collections::HashMap;
use std::fmt::Debug;
use num_traits::{Float, Zero};

use crate::graph::{Graph, GraphTransform, MutableGraph};
use crate::DirectedGraph;

/// Transforms a graph into one with constant bounded degree by creating cycles
/// of vertices for each original vertex, as described in the paper.
#[derive(Debug)]
pub struct ConstantDegreeTransform<W> 
where 
    W: Float + Zero + Debug + Copy,
{
    // Map of original vertex to list of transformed vertices
    original_to_transformed: HashMap<usize, Vec<usize>>,
    // Map of transformed vertex to original vertex
    transformed_to_original: HashMap<usize, usize>,
    _weight_type: std::marker::PhantomData<W>,
}

impl<W> ConstantDegreeTransform<W>
where
    W: Float + Zero + Debug + Copy,
{
    /// Creates a new constant degree transform
    pub fn new() -> Self {
        ConstantDegreeTransform {
            original_to_transformed: HashMap::new(),
            transformed_to_original: HashMap::new(),
            _weight_type: std::marker::PhantomData,
        }
    }
}

impl<W> GraphTransform<W, DirectedGraph<W>> for ConstantDegreeTransform<W>
where
    W: Float + Zero + Debug + Copy,
{
    fn transform(&self, graph: &DirectedGraph<W>) -> DirectedGraph<W> {
        let mut result = DirectedGraph::new();
        let mut original_to_transformed = HashMap::new();
        let mut transformed_to_original = HashMap::new();
        
        // Create cycle vertices for each original vertex
        for v in 0..graph.vertex_count() {
            // For each original vertex, create vertices for each incoming/outgoing edge
            let mut cycle_vertices = Vec::new();
            
            // Count all neighbors (incoming and outgoing)
            let outgoing_count = graph.outgoing_edges(v).count();
            let incoming_count = graph.incoming_edges(v).count();
            
            // Create cycle vertices
            for _ in 0..(outgoing_count + incoming_count).max(1) {
                let new_vertex = result.add_vertex();
                cycle_vertices.push(new_vertex);
                transformed_to_original.insert(new_vertex, v);
            }
            
            // Connect cycle vertices with zero-weight edges
            for i in 0..cycle_vertices.len() {
                let from = cycle_vertices[i];
                let to = cycle_vertices[(i + 1) % cycle_vertices.len()];
                result.add_edge(from, to, W::zero());
            }
            
            original_to_transformed.insert(v, cycle_vertices);
        }
        
        // Add edges between cycles
        for u in 0..graph.vertex_count() {
            let u_vertices = original_to_transformed.get(&u).unwrap();
            
            // Create a vector of outgoing edges
            let mut outgoing_edges = Vec::new();
            for (v, weight) in graph.outgoing_edges(u) {
                outgoing_edges.push((v, weight));
            }
            
            // Add edges to the transformed graph
            for (idx, (v, weight)) in outgoing_edges.iter().enumerate() {
                let v_vertices = original_to_transformed.get(v).unwrap();
                
                // Use modulo to handle cases where we have more edges than vertices
                let u_idx = idx % u_vertices.len();
                let v_idx = idx % v_vertices.len();
                
                result.add_edge(u_vertices[u_idx], v_vertices[v_idx], *weight);
            }
        }

        // Save mappings for later use
        unsafe {
            let this = self as *const _ as *mut Self;
            (*this).original_to_transformed = original_to_transformed;
            (*this).transformed_to_original = transformed_to_original;
        }
        
        result
    }

    fn map_vertex_to_original(&self, transformed_vertex: usize) -> usize {
        *self.transformed_to_original.get(&transformed_vertex).unwrap_or(&transformed_vertex)
    }

    fn map_vertex_from_original(&self, original_vertex: usize) -> Vec<usize> {
        self.original_to_transformed.get(&original_vertex)
            .cloned()
            .unwrap_or_else(|| vec![original_vertex])
    }
}
