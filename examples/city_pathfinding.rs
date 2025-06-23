use fast_sssp::{DirectedGraph, FastSSSP, Dijkstra, ShortestPathAlgorithm};
use fast_sssp::graph::{Graph, MutableGraph};
use ordered_float::OrderedFloat;
// Removed unused import
use std::collections::HashMap;
use std::io::{self, Write};

/// Represents a city grid with buildings and streets
#[derive(Clone)]
struct CityGrid {
    width: usize,
    height: usize,
    buildings: Vec<Vec<bool>>, // true = building, false = street
    locations: HashMap<String, (usize, usize)>,
}

impl CityGrid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buildings: vec![vec![false; width]; height],
            locations: HashMap::new(),
        }
    }

    fn add_building(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.buildings[y][x] = true;
        }
    }

    fn add_location(&mut self, name: String, x: usize, y: usize) {
        if x < self.width && y < self.height && !self.buildings[y][x] {
            self.locations.insert(name, (x, y));
        }
    }

    fn is_walkable(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height && !self.buildings[y][x]
    }

    fn get_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize, f64)> {
        let mut neighbors = Vec::new();
        let directions = [
            (0, 1, 1.0),   // North
            (1, 0, 1.0),   // East
            (0, -1, 1.0),  // South
            (-1, 0, 1.0),  // West
            (1, 1, 1.4),   // Northeast (diagonal)
            (1, -1, 1.4),  // Southeast (diagonal)
            (-1, 1, 1.4),  // Northwest (diagonal)
            (-1, -1, 1.4), // Southwest (diagonal)
        ];

        for (dx, dy, cost) in directions {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;
            
            if new_x >= 0 && new_y >= 0 {
                let new_x = new_x as usize;
                let new_y = new_y as usize;
                
                if self.is_walkable(new_x, new_y) {
                    neighbors.push((new_x, new_y, cost));
                }
            }
        }
        
        neighbors
    }

    fn coord_to_vertex(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn vertex_to_coord(&self, vertex: usize) -> (usize, usize) {
        (vertex % self.width, vertex / self.width)
    }
}

/// Converts a city grid to a directed graph
fn city_to_graph(city: &CityGrid) -> DirectedGraph<OrderedFloat<f64>> {
    let mut graph = DirectedGraph::new();
    
    // Add vertices for each cell
    for _ in 0..(city.width * city.height) {
        graph.add_vertex();
    }
    
    // Add edges between walkable adjacent cells
    for y in 0..city.height {
        for x in 0..city.width {
            if city.is_walkable(x, y) {
                let vertex = city.coord_to_vertex(x, y);
                
                for (nx, ny, cost) in city.get_neighbors(x, y) {
                    let neighbor_vertex = city.coord_to_vertex(nx, ny);
                    graph.add_edge(vertex, neighbor_vertex, OrderedFloat(cost));
                }
            }
        }
    }
    
    graph
}

/// Visualizes the city grid with paths
fn visualize_city_with_path(city: &CityGrid, path: Option<&[usize]>) {
    let mut grid = vec![vec![' '; city.width]; city.height];
    
    // Mark buildings
    for y in 0..city.height {
        for x in 0..city.width {
            if city.buildings[y][x] {
                grid[y][x] = '█';
            } else {
                grid[y][x] = '·';
            }
        }
    }
    
    // Mark locations
    for (name, &(x, y)) in &city.locations {
        grid[y][x] = name.chars().next().unwrap().to_ascii_uppercase();
    }
    
    // Mark path
    if let Some(path) = path {
        for &vertex in path {
            let (x, y) = city.vertex_to_coord(vertex);
            if grid[y][x] == '·' {
                grid[y][x] = '*';
            }
        }
    }
    
    // Print the grid
    println!("┌{}┐", "─".repeat(city.width));
    for row in &grid {
        print!("│");
        for &cell in row {
            print!("{}", cell);
        }
        println!("│");
    }
    println!("└{}┘", "─".repeat(city.width));
    
    // Print legend
    println!("\nLegend:");
    println!("█ = Building  · = Street  * = Path");
    for (name, _) in &city.locations {
        println!("{} = {}", name.chars().next().unwrap().to_ascii_uppercase(), name);
    }
}

/// Creates a sample city with interesting locations
fn create_sample_city() -> CityGrid {
    let mut city = CityGrid::new(20, 15);
    
    // Add some buildings to create interesting paths
    // Office building
    for x in 2..5 {
        for y in 2..6 {
            city.add_building(x, y);
        }
    }
    
    // Shopping mall
    for x in 8..12 {
        for y in 3..7 {
            city.add_building(x, y);
        }
    }
    
    // Residential block
    for x in 15..18 {
        for y in 8..12 {
            city.add_building(x, y);
        }
    }
    
    // Park (no buildings, but we'll mark it)
    for x in 6..8 {
        for y in 9..12 {
            city.add_building(x, y); // Actually, let's make it walkable
        }
    }
    
    // Small buildings scattered around
    city.add_building(1, 8);
    city.add_building(13, 2);
    city.add_building(16, 4);
    city.add_building(3, 12);
    city.add_building(11, 10);
    
    // Add interesting locations
    city.add_location("home".to_string(), 0, 0);
    city.add_location("work".to_string(), 19, 14);
    city.add_location("cafe".to_string(), 6, 1);
    city.add_location("gym".to_string(), 13, 8);
    city.add_location("park".to_string(), 7, 10);
    city.add_location("store".to_string(), 14, 3);
    
    city
}

fn interactive_demo() {
    println!("🏙️  Welcome to the City Pathfinding Demo! 🏙️");
    println!("This demo shows how the Fast SSSP algorithm finds shortest paths in a city grid.\n");
    
    let city = create_sample_city();
    let graph = city_to_graph(&city);
    
    println!("City Map:");
    visualize_city_with_path(&city, None);
    
    println!("\nAvailable locations:");
    for (name, &(x, y)) in &city.locations {
        println!("  {} at ({}, {})", name, x, y);
    }
    
    loop {
        println!("\n{}", "=".repeat(50));
        println!("Choose your journey:");
        println!("1. Home to Work (morning commute)");
        println!("2. Work to Cafe (lunch break)");
        println!("3. Home to Gym (evening workout)");
        println!("4. Cafe to Park (weekend stroll)");
        println!("5. Custom route");
        println!("6. Performance comparison");
        println!("7. Exit");
        
        print!("\nEnter your choice (1-7): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        match input.trim() {
            "1" => find_and_display_path(&city, &graph, "home", "work"),
            "2" => find_and_display_path(&city, &graph, "work", "cafe"),
            "3" => find_and_display_path(&city, &graph, "home", "gym"),
            "4" => find_and_display_path(&city, &graph, "cafe", "park"),
            "5" => custom_route(&city, &graph),
            "6" => performance_comparison(&city, &graph),
            "7" => {
                println!("Thanks for exploring the city! 🚶‍♂️");
                break;
            }
            _ => println!("Invalid choice. Please enter 1-7."),
        }
    }
}

fn find_and_display_path(city: &CityGrid, graph: &DirectedGraph<OrderedFloat<f64>>, from: &str, to: &str) {
    let from_pos = city.locations.get(from);
    let to_pos = city.locations.get(to);
    
    if let (Some(&(fx, fy)), Some(&(tx, ty))) = (from_pos, to_pos) {
        let source = city.coord_to_vertex(fx, fy);
        let target = city.coord_to_vertex(tx, ty);
        
        println!("\n🗺️  Finding path from {} to {}...", from, to);
        
        // Use Fast SSSP algorithm
        let fast_sssp = FastSSSP::new();
        let result = fast_sssp.compute_shortest_paths(graph, source).unwrap();
        
        if let Some(distance) = result.distances[target] {
            let path = <FastSSSP as ShortestPathAlgorithm<OrderedFloat<f64>, DirectedGraph<OrderedFloat<f64>>>>::get_path(&fast_sssp, &result, target).unwrap();
            
            println!("✅ Path found!");
            println!("Distance: {:.2} blocks", distance.into_inner());
            println!("Steps: {} moves", path.len() - 1);
            
            println!("\nPath visualization:");
            visualize_city_with_path(city, Some(&path));
            
            // Show step-by-step directions
            println!("\nStep-by-step directions:");
            for i in 1..path.len() {
                let (px, py) = city.vertex_to_coord(path[i-1]);
                let (cx, cy) = city.vertex_to_coord(path[i]);
                let direction = match (cx as i32 - px as i32, cy as i32 - py as i32) {
                    (0, 1) => "North",
                    (1, 0) => "East", 
                    (0, -1) => "South",
                    (-1, 0) => "West",
                    (1, 1) => "Northeast",
                    (1, -1) => "Southeast",
                    (-1, 1) => "Northwest",
                    (-1, -1) => "Southwest",
                    _ => "Unknown",
                };
                println!("  {}. Go {} to ({}, {})", i, direction, cx, cy);
            }
        } else {
            println!("❌ No path found from {} to {}!", from, to);
        }
    } else {
        println!("❌ Invalid locations specified!");
    }
}

fn custom_route(city: &CityGrid, graph: &DirectedGraph<OrderedFloat<f64>>) {
    println!("\n📍 Custom Route Planner");
    println!("Available locations: {:?}", city.locations.keys().collect::<Vec<_>>());
    
    print!("Enter starting location: ");
    io::stdout().flush().unwrap();
    let mut from = String::new();
    io::stdin().read_line(&mut from).unwrap();
    let from = from.trim();
    
    print!("Enter destination: ");
    io::stdout().flush().unwrap();
    let mut to = String::new();
    io::stdin().read_line(&mut to).unwrap();
    let to = to.trim();
    
    find_and_display_path(city, graph, from, to);
}

fn performance_comparison(city: &CityGrid, graph: &DirectedGraph<OrderedFloat<f64>>) {
    println!("\n⚡ Performance Comparison: Fast SSSP vs Dijkstra");
    println!("Running both algorithms from 'home' to all destinations...\n");
    
    let source = city.coord_to_vertex(0, 0); // home position
    
    // Fast SSSP
    let start = std::time::Instant::now();
    let fast_sssp = FastSSSP::new();
    let fast_result = fast_sssp.compute_shortest_paths(graph, source).unwrap();
    let fast_time = start.elapsed();
    
    // Dijkstra
    let start = std::time::Instant::now();
    let dijkstra = Dijkstra::new();
    let dijkstra_result = dijkstra.compute_shortest_paths(graph, source).unwrap();
    let dijkstra_time = start.elapsed();
    
    println!("📊 Results:");
    println!("Graph size: {} vertices, {} edges", graph.vertex_count(), graph.edge_count());
    println!("Fast SSSP time: {:?}", fast_time);
    println!("Dijkstra time: {:?}", dijkstra_time);
    
    if fast_time < dijkstra_time {
        let speedup = dijkstra_time.as_nanos() as f64 / fast_time.as_nanos() as f64;
        println!("🚀 Fast SSSP is {:.2}x faster!", speedup);
    } else {
        println!("📝 Note: For small graphs, overhead may make Dijkstra appear faster");
    }
    
    println!("\n🎯 Distance comparison to key locations:");
    for (name, &(x, y)) in &city.locations {
        if name != "home" {
            let target = city.coord_to_vertex(x, y);
            let fast_dist = fast_result.distances[target];
            let dijkstra_dist = dijkstra_result.distances[target];
            
            match (fast_dist, dijkstra_dist) {
                (Some(fd), Some(dd)) => {
                    let fd_val = fd.into_inner();
                    let dd_val = dd.into_inner();
                    let match_symbol = if (fd_val - dd_val).abs() < 1e-10 { "✅" } else { "❌" };
                    println!("  {} {}: Fast={:.2}, Dijkstra={:.2} {}", 
                             name, match_symbol, fd_val, dd_val, 
                             if (fd_val - dd_val).abs() < 1e-10 { "✓" } else { "✗" });
                }
                _ => println!("  {}: Unreachable", name),
            }
        }
    }
}

fn main() {
    // Run the interactive demo
    interactive_demo();
}
