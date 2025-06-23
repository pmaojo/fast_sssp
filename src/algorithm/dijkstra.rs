use std::fmt::Debug;
use num_traits::{Float, Zero};

use crate::graph::Graph;
use crate::algorithm::{ShortestPathAlgorithm, ShortestPathResult};
use crate::data_structures::BinaryHeapWrapper;
use crate::{Error, Result};

/// Classic Dijkstra's algorithm implementation
#[derive(Debug, Default)]
pub struct Dijkstra;

impl Dijkstra {
    /// Creates a new Dijkstra algorithm instance
    pub fn new() -> Self {
        Dijkstra
    }
}

impl<W, G> ShortestPathAlgorithm<W, G> for Dijkstra
where
    W: Float + Zero + Debug + Copy + Ord,
    G: Graph<W>,
{
    fn name(&self) -> &'static str {
        "Dijkstra"
    }
    
    fn compute_shortest_paths(&self, graph: &G, source: usize) -> Result<ShortestPathResult<W>> {
        if !graph.has_vertex(source) {
            return Err(Error::SourceNotFound);
        }
        
        let n = graph.vertex_count();
        
        // Initialize distances and predecessors
        let mut distances: Vec<Option<W>> = vec![None; n];
        let mut predecessors: Vec<Option<usize>> = vec![None; n];
        
        // Distance to source is 0
        distances[source] = Some(W::zero());
        
        // Initialize priority queue
        let mut queue = BinaryHeapWrapper::new();
        queue.push(source, W::zero());
        
        // Main Dijkstra loop
        while let Some((u, dist_u)) = queue.pop() {
            // If we've already found a shorter path to u, skip
            if let Some(current_dist) = distances[u] {
                if current_dist < dist_u {
                    continue;
                }
            }
            
            // Relax all outgoing edges
            for (v, weight) in graph.outgoing_edges(u) {
                let new_dist = dist_u + weight;
                
                let should_update = match distances[v] {
                    None => true,
                    Some(current_dist) => new_dist < current_dist,
                };
                
                if should_update {
                    distances[v] = Some(new_dist);
                    predecessors[v] = Some(u);
                    queue.push(v, new_dist);
                }
            }
        }
        
        Ok(ShortestPathResult {
            distances,
            predecessors,
            source,
        })
    }
}
