use fast_sssp::algorithm::dijkstra::Dijkstra;
use fast_sssp::algorithm::fast_sssp::FastSSSP;
use fast_sssp::algorithm::traits::ShortestPathAlgorithm;
use fast_sssp::graph::DirectedGraph;
use fast_sssp::graph::{Graph, MutableGraph};
use ordered_float::OrderedFloat;
use std::collections::HashMap;

// Test helper function to create a simple grid graph
fn create_test_grid(width: usize, height: usize) -> DirectedGraph<OrderedFloat<f64>> {
    let mut graph = DirectedGraph::new();
    
    // Add vertices for all positions in the grid
    for _ in 0..(width * height) {
        graph.add_vertex();
    }
    
    // Connect adjacent vertices (including diagonals)
    for y in 0..height {
        for x in 0..width {
            let vertex = y * width + x;
            
            // Define possible moves (8 directions)
            let directions = [
                // Cardinal directions (N, E, S, W)
                (0, -1, 1.0), (1, 0, 1.0), (0, 1, 1.0), (-1, 0, 1.0),
                // Diagonal directions (NE, SE, SW, NW)
                (1, -1, 1.4), (1, 1, 1.4), (-1, 1, 1.4), (-1, -1, 1.4),
            ];
            
            for (dx, dy, cost) in directions {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                if nx >= 0 && ny >= 0 && nx < width as i32 && ny < height as i32 {
                    let neighbor = ny as usize * width + nx as usize;
                    graph.add_edge(vertex, neighbor, OrderedFloat(cost));
                }
            }
        }
    }
    
    graph
}

// Test that paths can be found in a simple grid
#[test]
fn test_path_finding_simple_grid() {
    let graph = create_test_grid(10, 10);
    
    // Test source and target vertices
    let source = 0; // Top-left corner (0,0)
    let target = 99; // Bottom-right corner (9,9)
    
    // Test with Dijkstra
    let dijkstra = Dijkstra::new();
    let dijkstra_result = dijkstra.compute_shortest_paths(&graph, source).unwrap();
    
    // Test with FastSSSP
    let fast_sssp = FastSSSP::new();
    let fast_sssp_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    
    // Verify distances
    assert!(dijkstra_result.distances[target].is_some(), "Dijkstra should find a path");
    assert!(fast_sssp_result.distances[target].is_some(), "FastSSSP should find a path");
    
    // Verify paths
    let dijkstra_path = <Dijkstra as ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>>::get_path(&dijkstra, &dijkstra_result, target);
    let fast_sssp_path = <FastSSSP as ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>>::get_path(&fast_sssp, &fast_sssp_result, target);
    
    assert!(dijkstra_path.is_some(), "Dijkstra should construct a path");
    assert!(fast_sssp_path.is_some(), "FastSSSP should construct a path");
    
    // Verify path properties
    let dijkstra_path = dijkstra_path.unwrap();
    let fast_sssp_path = fast_sssp_path.unwrap();
    
    assert_eq!(dijkstra_path[0], source, "Path should start at source");
    assert_eq!(dijkstra_path[dijkstra_path.len() - 1], target, "Path should end at target");
    
    assert_eq!(fast_sssp_path[0], source, "Path should start at source");
    assert_eq!(fast_sssp_path[fast_sssp_path.len() - 1], target, "Path should end at target");
}

// Test path finding with obstacles
#[test]
fn test_path_finding_with_obstacles() {
    let mut graph = create_test_grid(10, 10);
    
    // Create a wall of obstacles in the middle
    for y in 0..8 {
        let obstacle = y * 10 + 5; // Column 5
        
        // Remove all edges to and from this vertex
        let mut edges_to_remove = Vec::new();
        for v in 0..graph.vertex_count() {
            if graph.has_edge(v, obstacle) {
                edges_to_remove.push((v, obstacle));
            }
            if graph.has_edge(obstacle, v) {
                edges_to_remove.push((obstacle, v));
            }
        }
        
        for (from, to) in edges_to_remove {
            graph.remove_edge(from, to);
        }
    }
    
    // Test source and target vertices
    let source = 0; // Top-left corner (0,0)
    let target = 99; // Bottom-right corner (9,9)
    
    // Test with Dijkstra
    let dijkstra = Dijkstra::new();
    let dijkstra_result = dijkstra.compute_shortest_paths(&graph, source).unwrap();
    
    // Test with FastSSSP
    let fast_sssp = FastSSSP::new();
    let fast_sssp_result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
    
    // Verify distances
    assert!(dijkstra_result.distances[target].is_some(), "Dijkstra should find a path around obstacles");
    assert!(fast_sssp_result.distances[target].is_some(), "FastSSSP should find a path around obstacles");
    
    // Verify paths
    let dijkstra_path = <Dijkstra as ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>>::get_path(&dijkstra, &dijkstra_result, target);
    let fast_sssp_path = <FastSSSP as ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>>::get_path(&fast_sssp, &fast_sssp_result, target);
    
    assert!(dijkstra_path.is_some(), "Dijkstra should construct a path around obstacles");
    assert!(fast_sssp_path.is_some(), "FastSSSP should construct a path around obstacles");
    
    // Verify path properties
    let dijkstra_path = dijkstra_path.unwrap();
    let fast_sssp_path = fast_sssp_path.unwrap();
    
    assert_eq!(dijkstra_path[0], source, "Path should start at source");
    assert_eq!(dijkstra_path[dijkstra_path.len() - 1], target, "Path should end at target");
    
    assert_eq!(fast_sssp_path[0], source, "Path should start at source");
    assert_eq!(fast_sssp_path[fast_sssp_path.len() - 1], target, "Path should end at target");
}

// Test the city pathfinding scenario
#[test]
fn test_city_pathfinding() {
    // Create a simple city grid
    let width = 25;
    let height = 18;
    let mut graph = DirectedGraph::new();
    
    // Add vertices for all positions in the grid
    for _ in 0..(width * height) {
        graph.add_vertex();
    }
    
    // Create a mapping of buildings (obstacles)
    let mut buildings = vec![vec![false; width]; height];
    
    // Add some buildings as obstacles
    let building_positions = [
        (3, 3), (4, 3), (5, 3),
        (3, 4), (4, 4), (5, 4),
        (3, 5), (4, 5), (5, 5),
        (10, 10), (11, 10), (12, 10),
        (10, 11), (11, 11), (12, 11),
        (10, 12), (11, 12), (12, 12),
    ];
    
    for &(x, y) in &building_positions {
        buildings[y][x] = true;
    }
    
    // Connect walkable positions
    for y in 0..height {
        for x in 0..width {
            if !buildings[y][x] {
                let vertex = y * width + x;
                
                // Define possible moves (8 directions)
                let directions = [
                    // Cardinal directions (N, E, S, W)
                    (0, -1, 1.0), (1, 0, 1.0), (0, 1, 1.0), (-1, 0, 1.0),
                    // Diagonal directions (NE, SE, SW, NW)
                    (1, -1, 1.4), (1, 1, 1.4), (-1, 1, 1.4), (-1, -1, 1.4),
                ];
                
                for (dx, dy, cost) in directions {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    
                    if nx >= 0 && ny >= 0 && nx < width as i32 && ny < height as i32 {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        
                        if !buildings[ny][nx] {
                            let neighbor = ny * width + nx;
                            graph.add_edge(vertex, neighbor, OrderedFloat(cost));
                        }
                    }
                }
            }
        }
    }
    
    // Define some key locations
    let locations = HashMap::from([
        ("home".to_string(), (0, 0)),
        ("work".to_string(), (20, 15)),
        ("gym".to_string(), (15, 8)),
        ("park".to_string(), (8, 12)),
    ]);
    
    // Test path finding between locations
    for (from_name, &(fx, fy)) in &locations {
        for (to_name, &(tx, ty)) in &locations {
            if from_name != to_name {
                let source = fy * width + fx;
                let target = ty * width + tx;
                
                // Test with FastSSSP
                let fast_sssp = FastSSSP::new();
                let result = fast_sssp.compute_shortest_paths(&graph, source).unwrap();
                
                // Verify distance and path
                assert!(result.distances[target].is_some(), 
                       "Should find a path from {} to {}", from_name, to_name);
                
                let path = <FastSSSP as ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>>::get_path(&fast_sssp, &result, target);
                assert!(path.is_some(), 
                       "Should construct a path from {} to {}", from_name, to_name);
                
                let path = path.unwrap();
                assert_eq!(path[0], source, "Path should start at source");
                assert_eq!(path[path.len() - 1], target, "Path should end at target");
                
                // Verify path continuity
                for i in 1..path.len() {
                    assert!(graph.has_edge(path[i-1], path[i]), 
                           "Path should only use existing edges");
                }
            }
        }
    }
}
