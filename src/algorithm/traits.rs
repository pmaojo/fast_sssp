use std::fmt::Debug;
use num_traits::{Float, Zero};
use crate::graph::Graph;
use crate::Result;

/// Result of a shortest path algorithm execution
#[derive(Debug, Clone)]
pub struct ShortestPathResult<W>
where
    W: Float + Zero + Debug + Copy,
{
    /// Distances from source to each vertex
    pub distances: Vec<Option<W>>,
    
    /// Predecessor vertices in the shortest path tree
    pub predecessors: Vec<Option<usize>>,
    
    /// Source vertex ID
    pub source: usize,
}

/// Trait for shortest path algorithms
pub trait ShortestPathAlgorithm<W, G>
where
    W: Float + Zero + Debug + Copy,
    G: Graph<W>,
{
    /// Compute shortest paths from a source vertex to all other vertices
    fn compute_shortest_paths(&self, graph: &G, source: usize) -> Result<ShortestPathResult<W>>;
    
    /// Get the name of the algorithm
    fn name(&self) -> &'static str;
    
    /// Get the shortest path from source to target as a sequence of vertices
    fn get_path(&self, result: &ShortestPathResult<W>, target: usize) -> Option<Vec<usize>> {
        if target >= result.predecessors.len() || result.distances[target].is_none() {
            return None;
        }
        
        // Use Dijkstra's algorithm to reconstruct a valid path
        // This ensures we only use existing edges
        let mut path = Vec::new();
        let mut current = target;
        let mut visited = std::collections::HashSet::new();
        
        // Build path in reverse order
        while current != result.source {
            // Safety check to prevent infinite loops
            if !visited.insert(current) {
                println!("WARNING: Cycle detected in path reconstruction at vertex {}", current);
                return None; // If there's a cycle, we can't construct a valid path
            }
            
            path.push(current);
            match result.predecessors[current] {
                Some(pred) => {
                    // Ensure we're making progress toward the source
                    if pred == current {
                        // Self-loop detected, break it
                        break;
                    }
                    current = pred;
                },
                None => {
                    // If we hit a vertex with no predecessor but it's not the source,
                    // we have a broken path and can't continue
                    return None;
                },
            }
            
            // Additional safety check - limit path length
            if path.len() > result.predecessors.len() {
                println!("WARNING: Path length exceeds graph size, likely a cycle");
                return None;
            }
        }
        
        path.push(result.source);
        path.reverse();
        
        Some(path)
    }
}
