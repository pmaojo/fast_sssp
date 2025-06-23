use std::fmt::Debug;
use num_traits::{Float, Zero};

/// Trait representing a weighted directed graph
pub trait Graph<W>: Debug 
where
    W: Float + Zero + Debug + Copy,
{
    /// Returns the number of vertices in the graph
    fn vertex_count(&self) -> usize;
    
    /// Returns the number of edges in the graph
    fn edge_count(&self) -> usize;
    
    /// Returns an iterator over the outgoing edges from a vertex
    fn outgoing_edges(&self, vertex: usize) -> Box<dyn Iterator<Item = (usize, W)> + '_>;
    
    /// Returns an iterator over the incoming edges to a vertex
    fn incoming_edges(&self, vertex: usize) -> Box<dyn Iterator<Item = (usize, W)> + '_>;
    
    /// Returns true if the vertex exists in the graph
    fn has_vertex(&self, vertex: usize) -> bool;
    
    /// Returns true if there's an edge between the two vertices
    fn has_edge(&self, from: usize, to: usize) -> bool;
    
    /// Gets the weight of an edge if it exists
    fn get_edge_weight(&self, from: usize, to: usize) -> Option<W>;
}

/// Trait for mutable graph operations
pub trait MutableGraph<W>: Graph<W> 
where
    W: Float + Zero + Debug + Copy,
{
    /// Adds a vertex to the graph and returns its ID
    fn add_vertex(&mut self) -> usize;
    
    /// Removes a vertex and its connected edges from the graph
    fn remove_vertex(&mut self, vertex: usize) -> bool;
    
    /// Adds a directed edge between vertices with the given weight
    fn add_edge(&mut self, from: usize, to: usize, weight: W) -> bool;
    
    /// Removes an edge from the graph
    fn remove_edge(&mut self, from: usize, to: usize) -> bool;
    
    /// Updates the weight of an existing edge
    fn update_edge_weight(&mut self, from: usize, to: usize, weight: W) -> bool;
}

/// Trait for transforming a graph (e.g., creating a constant-degree representation)
pub trait GraphTransform<W, G>
where
    W: Float + Zero + Debug + Copy,
    G: Graph<W>,
{
    /// Transforms a graph into a different representation
    fn transform(&self, graph: &G) -> G;
    
    /// Maps a vertex from the transformed graph back to the original graph
    fn map_vertex_to_original(&self, transformed_vertex: usize) -> usize;
    
    /// Maps a vertex from the original graph to the transformed graph
    /// Returns a list of vertices in the transformed graph that correspond to the original vertex
    fn map_vertex_from_original(&self, original_vertex: usize) -> Vec<usize>;
}

/// Trait for graphs that can be transformed to constant-degree graphs
pub trait ToConstantDegree<W>
where
    W: Float + Zero + Debug + Copy,
{
    /// Converts this graph to a constant-degree representation
    /// Returns the transformed graph and mappings between original and transformed vertices
    fn to_constant_degree(&self) -> (Self, Vec<Vec<usize>>, Vec<usize>) where Self: Sized;
}
