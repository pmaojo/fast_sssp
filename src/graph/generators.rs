use crate::graph::{DirectedGraph, Graph, MutableGraph};
use ordered_float::OrderedFloat;
use rand::prelude::*;
use std::collections::HashSet;

/// Generates a Barabási-Albert scale-free graph with n vertices and m edges per new vertex
/// Returns a directed graph with OrderedFloat<f64> weights
pub fn generate_barabasi_albert(n: usize, m: usize) -> DirectedGraph<OrderedFloat<f64>> {
    assert!(m > 0, "m must be positive");
    assert!(n > m, "n must be greater than m");
    
    let mut graph = DirectedGraph::new();
    let mut rng = rand::thread_rng();
    
    // Add initial m vertices
    for _ in 0..m {
        graph.add_vertex();
    }
    
    // Connect initial vertices (complete graph)
    for i in 0..m {
        for j in 0..m {
            if i != j && !graph.has_edge(i, j) {
                let weight = OrderedFloat(rng.gen_range(1.0..100.0));
                graph.add_edge(i, j, weight);
            }
        }
    }
    
    // Vector to track the degree of each vertex for preferential attachment
    let mut degrees = vec![m - 1; m];
    let mut total_degree = m * (m - 1);
    
    // Add remaining vertices with preferential attachment
    for i in m..n {
        graph.add_vertex();
        let mut added_edges = HashSet::new();
        
        // Add m edges from the new vertex to existing vertices
        while added_edges.len() < m {
            // Select target based on degree (preferential attachment)
            let mut target_value = rng.gen_range(0..total_degree);
            let mut target = 0;
            
            // Find the target vertex based on cumulative degree
            while target < i && target_value >= degrees[target] {
                target_value -= degrees[target];
                target += 1;
            }
            
            if target < i && !added_edges.contains(&target) {
                // Add edge
                let weight = OrderedFloat(rng.gen_range(1.0..100.0));
                graph.add_edge(i, target, weight);
                added_edges.insert(target);
                
                // Update degrees
                degrees[target] += 1;
                total_degree += 1;
            }
        }
        
        // Update degree of the new vertex
        degrees.push(m);
        total_degree += m;
    }
    
    graph
}

/// Generates a 3D grid graph with dimensions x*y*z
/// Returns a directed graph with OrderedFloat<f64> weights
pub fn generate_3d_grid(x: usize, y: usize, z: usize) -> DirectedGraph<OrderedFloat<f64>> {
    let mut graph = DirectedGraph::new();
    // No random generator needed for grid graph
    
    // Total number of vertices
    let n = x * y * z;
    
    // Add vertices
    for _ in 0..n {
        graph.add_vertex();
    }
    
    // Helper function to get vertex index from 3D coordinates
    let get_index = |i: usize, j: usize, k: usize| -> usize {
        i * y * z + j * z + k
    };
    
    // Add edges (6-connectivity)
    for i in 0..x {
        for j in 0..y {
            for k in 0..z {
                let current = get_index(i, j, k);
                
                // Connect to neighbors
                if i > 0 {
                    let neighbor = get_index(i-1, j, k);
                    let weight = OrderedFloat(1.0);
                    graph.add_edge(current, neighbor, weight);
                }
                if i < x-1 {
                    let neighbor = get_index(i+1, j, k);
                    let weight = OrderedFloat(1.0);
                    graph.add_edge(current, neighbor, weight);
                }
                if j > 0 {
                    let neighbor = get_index(i, j-1, k);
                    let weight = OrderedFloat(1.0);
                    graph.add_edge(current, neighbor, weight);
                }
                if j < y-1 {
                    let neighbor = get_index(i, j+1, k);
                    let weight = OrderedFloat(1.0);
                    graph.add_edge(current, neighbor, weight);
                }
                if k > 0 {
                    let neighbor = get_index(i, j, k-1);
                    let weight = OrderedFloat(1.0);
                    graph.add_edge(current, neighbor, weight);
                }
                if k < z-1 {
                    let neighbor = get_index(i, j, k+1);
                    let weight = OrderedFloat(1.0);
                    graph.add_edge(current, neighbor, weight);
                }
            }
        }
    }
    
    graph
}

/// Generates a random geometric graph in 3D space
/// n: number of vertices
/// r: connection radius (vertices within distance r are connected)
pub fn generate_geometric_3d(n: usize, r: f64) -> DirectedGraph<OrderedFloat<f64>> {
    let mut graph = DirectedGraph::new();
    let mut rng = rand::thread_rng();
    
    // Generate random points in 3D space
    let mut points = Vec::with_capacity(n);
    for _ in 0..n {
        let x = rng.gen_range(0.0..1.0);
        let y = rng.gen_range(0.0..1.0);
        let z = rng.gen_range(0.0..1.0);
        points.push((x, y, z));
        graph.add_vertex();
    }
    
    // Connect points that are within distance r
    for i in 0..n {
        for j in 0..n {
            if i != j {
                let (x1, y1, z1) = points[i];
                let (x2, y2, z2) = points[j];
                
                // Calculate Euclidean distance
                let dx = x1 - x2;
                let dy = y1 - y2;
                let dz = z1 - z2;
                let dist = f64::sqrt(dx*dx + dy*dy + dz*dz);
                
                if dist <= r {
                    // Add edge with weight equal to the distance
                    graph.add_edge(i, j, OrderedFloat(dist));
                }
            }
        }
    }
    
    graph
}
