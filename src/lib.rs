//! Fast SSSP - O(m log^(2/3) n) Single-Source Shortest Path Algorithm
//! 
//! This library implements the algorithm described in "Breaking the Sorting Barrier
//! for Directed Single-Source Shortest Paths" by Duan et al. (2025).
//! 
//! The algorithm provides a deterministic O(m log^(2/3) n) solution for single-source
//! shortest paths (SSSP) on directed graphs with real non-negative edge weights.

pub mod graph;
pub mod algorithm;
pub mod data_structures;

/// Re-export main types for convenient use
pub use graph::directed::DirectedGraph;
pub use algorithm::{
    fast_sssp::FastSSSP,
    dijkstra::Dijkstra,
    ShortestPathAlgorithm,
    ShortestPathResult,
};

/// Error types for the library
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid vertex ID: {0}")]
    InvalidVertex(usize),
    
    #[error("Invalid edge: from {0} to {1}")]
    InvalidEdge(usize, usize),
    
    #[error("Negative edge weight: {0}")]
    NegativeWeight(f64),
    
    #[error("Source vertex not found in graph")]
    SourceNotFound,
    
    #[error("Algorithm execution error: {0}")]
    AlgorithmError(String),
}

/// Result type for the library
pub type Result<T> = std::result::Result<T, Error>;
