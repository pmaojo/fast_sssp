use fast_sssp::{DirectedGraph, FastSSSP, Dijkstra, ShortestPathAlgorithm, ShortestPathResult};
use fast_sssp::graph::{Graph, MutableGraph};
use ordered_float::OrderedFloat;
use colored::*;
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use std::collections::HashSet;
use std::fmt::Debug;
use num_traits::{Float, Zero};

/// Represents a city grid with buildings and streets
#[derive(Clone)]
struct CityGrid {
    width: usize,
    height: usize,
    buildings: Vec<Vec<bool>>,
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
        // Cardinal directions (N, E, S, W) with cost 1.0
        // Diagonal directions (NE, SE, SW, NW) with cost 1.4
        let directions = [
            // Cardinal directions: N, E, S, W
            (0, -1, 1.0), (1, 0, 1.0), (0, 1, 1.0), (-1, 0, 1.0),
            // Diagonal directions: NE, SE, SW, NW
            (1, -1, 1.4), (1, 1, 1.4), (-1, 1, 1.4), (-1, -1, 1.4),
        ];

        for (dx, dy, cost) in directions {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;
            
            // Check if the new coordinates are within bounds
            if new_x >= 0 && new_y >= 0 && new_x < self.width as i32 && new_y < self.height as i32 {
                let new_x = new_x as usize;
                let new_y = new_y as usize;
                
                // Check if the position is walkable (not a building)
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

fn city_to_graph(city: &CityGrid) -> DirectedGraph<OrderedFloat<f64>> {
    let mut graph = DirectedGraph::new();
    
    // Add vertices for all positions in the grid
    for _ in 0..(city.width * city.height) {
        graph.add_vertex();
    }
    
    // Add edges between walkable positions
    for y in 0..city.height {
        for x in 0..city.width {
            if city.is_walkable(x, y) {
                let vertex = city.coord_to_vertex(x, y);
                
                // Get all walkable neighbors and add edges
                for (nx, ny, cost) in city.get_neighbors(x, y) {
                    let neighbor_vertex = city.coord_to_vertex(nx, ny);
                    graph.add_edge(vertex, neighbor_vertex, OrderedFloat(cost));
                }
            }
        }
    }
    
    // Debug output to verify graph connectivity
    println!("Graph created with {} vertices and {} edges", 
             graph.vertex_count(), 
             graph.edge_count());
    
    // Verify that all locations are walkable
    let mut all_walkable = true;
    for (name, &(x, y)) in &city.locations {
        if !city.is_walkable(x, y) {
            println!("WARNING: Location '{}' at ({}, {}) is not walkable!", name, x, y);
            all_walkable = false;
        }
    }
    
    if all_walkable {
        println!("All locations are properly placed on walkable tiles.");
    }
    
    // Check if any locations are in the same cell (would cause visualization issues)
    let mut location_coords = HashMap::new();
    for (name, &(x, y)) in &city.locations {
        let coord = (x, y);
        if let Some(existing) = location_coords.get(&coord) {
            println!("WARNING: Multiple locations at ({}, {}): '{}' and '{}'", 
                     x, y, existing, name);
        } else {
            location_coords.insert(coord, name);
        }
    }
    
    graph
}

fn visualize_city_with_animated_path(city: &CityGrid, path: Option<&[usize]>, animate: bool) {
    if animate && path.is_some() {
        // Show animated pathfinding
        for step in 0..=path.unwrap().len() {
            print!("\x1B[2J\x1B[1;1H"); // Clear screen
            println!("{}", "🏙️  CITY PATHFINDING VISUALIZATION  🏙️".bright_cyan().bold());
            println!();
            
            let current_path = if step == 0 { 
                None 
            } else { 
                Some(&path.unwrap()[..step.min(path.unwrap().len())]) 
            };
            
            visualize_city_static(city, current_path);
            
            if step < path.unwrap().len() {
                println!("\n{} Step {}/{}", "🚶".bright_yellow(), step, path.unwrap().len() - 1);
                thread::sleep(Duration::from_millis(300));
            }
        }
        println!("\n{} {}", "✅".bright_green(), "Path complete!".bright_green().bold());
    } else {
        visualize_city_static(city, path);
    }
}

fn visualize_city_static(city: &CityGrid, path: Option<&[usize]>) {
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
    
    // Mark path first (so locations can override)
    if let Some(path) = path {
        for &vertex in path {
            let (x, y) = city.vertex_to_coord(vertex);
            if grid[y][x] == '·' {
                grid[y][x] = '*';
            }
        }
    }
    
    // Mark locations (these override path markers)
    for (name, &(x, y)) in &city.locations {
        grid[y][x] = name.chars().next().unwrap().to_ascii_uppercase();
    }
    
    // Print the grid with colors
    println!("{}", format!("┌{}┐", "─".repeat(city.width)).bright_white());
    for row in &grid {
        print!("{}", "│".bright_white());
        for &cell in row {
            let colored_cell = match cell {
                '█' => cell.to_string().on_bright_black().white(),
                '·' => cell.to_string().bright_black(),
                '*' => cell.to_string().bright_yellow().bold(),
                'H' => cell.to_string().bright_green().bold(),
                'W' => cell.to_string().bright_red().bold(),
                'C' => cell.to_string().bright_magenta().bold(),
                'G' => cell.to_string().bright_blue().bold(),
                'P' => cell.to_string().bright_cyan().bold(),
                'S' => cell.to_string().yellow().bold(),
                _ => cell.to_string().normal(),
            };
            print!("{}", colored_cell);
        }
        println!("{}", "│".bright_white());
    }
    println!("{}", format!("└{}┘", "─".repeat(city.width)).bright_white());
    
    // Print colorful legend
    println!("\n{}", "Legend:".bright_white().bold());
    println!("{} = Building  {} = Street  {} = Path", 
             "█".on_bright_black().white(), 
             "·".bright_black(), 
             "*".bright_yellow().bold());
    
    for (name, _) in &city.locations {
        let symbol = name.chars().next().unwrap().to_ascii_uppercase();
        let colored_symbol = match symbol {
            'H' => symbol.to_string().bright_green().bold(),
            'W' => symbol.to_string().bright_red().bold(),
            'C' => symbol.to_string().bright_magenta().bold(),
            'G' => symbol.to_string().bright_blue().bold(),
            'P' => symbol.to_string().bright_cyan().bold(),
            'S' => symbol.to_string().yellow().bold(),
            _ => symbol.to_string().normal(),
        };
        println!("{} = {}", colored_symbol, name.bright_white());
    }
}

fn create_sample_city() -> CityGrid {
    let mut city = CityGrid::new(25, 18);
    
    // Create a more interesting city layout
    
    // Office district
    for x in 3..7 {
        for y in 2..8 {
            city.add_building(x, y);
        }
    }
    
    // Shopping center
    for x in 10..15 {
        for y in 4..9 {
            city.add_building(x, y);
        }
    }
    
    // Residential area
    for x in 18..22 {
        for y in 10..15 {
            city.add_building(x, y);
        }
    }
    
    // Hospital complex
    for x in 1..4 {
        for y in 12..16 {
            city.add_building(x, y);
        }
    }
    
    // School
    for x in 8..11 {
        for y in 13..16 {
            city.add_building(x, y);
        }
    }
    
    // Random smaller buildings
    let small_buildings = [
        (16, 2), (17, 2), (20, 4), (23, 7), (15, 12), 
        (6, 10), (13, 11), (2, 9), (24, 15), (12, 2)
    ];
    
    for &(x, y) in &small_buildings {
        city.add_building(x, y);
    }
    
    // Add interesting locations
    city.add_location("home".to_string(), 0, 0);
    city.add_location("work".to_string(), 24, 17);
    city.add_location("cafe".to_string(), 8, 1);
    city.add_location("gym".to_string(), 16, 8);
    city.add_location("park".to_string(), 12, 10);
    city.add_location("store".to_string(), 20, 6);
    
    city
}

fn show_welcome_screen() {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen
    println!("{}", "🏙️".repeat(20));
    println!("{}", "    FAST SSSP CITY PATHFINDING DEMO    ".bright_cyan().bold().on_black());
    println!("{}", "🏙️".repeat(20));
    println!();
    println!("{}", "Welcome to the interactive city pathfinding demonstration!".bright_white());
    println!("{}", "This demo showcases the O(m log^(2/3) n) Fast SSSP algorithm".bright_yellow());
    println!("{}", "finding optimal paths through a city with buildings and obstacles.".bright_yellow());
    println!();
    println!("{} {}", "🚀".bright_red(), "Features:".bright_white().bold());
    println!("  • Interactive city exploration");
    println!("  • Animated pathfinding visualization");
    println!("  • Performance comparison with Dijkstra");
    println!("  • Real-time path calculation");
    println!();
    print!("Press Enter to start exploring... ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

fn interactive_demo() {
    show_welcome_screen();
    
    let city = create_sample_city();
    let graph = city_to_graph(&city);
    
    loop {
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        println!("{}", "🗺️  CITY MAP  🗺️".bright_cyan().bold());
        println!();
        visualize_city_static(&city, None);
        
        println!("\n{}", "Available locations:".bright_white().bold());
        for (name, &(x, y)) in &city.locations {
            let symbol = name.chars().next().unwrap().to_ascii_uppercase();
            let colored_symbol = match symbol {
                'H' => symbol.to_string().bright_green().bold(),
                'W' => symbol.to_string().bright_red().bold(),
                'C' => symbol.to_string().bright_magenta().bold(),
                'G' => symbol.to_string().bright_blue().bold(),
                'P' => symbol.to_string().bright_cyan().bold(),
                'S' => symbol.to_string().yellow().bold(),
                _ => symbol.to_string().normal(),
            };
            println!("  {} {} at ({}, {})", colored_symbol, name.bright_white(), x, y);
        }
        
        println!("\n{}", "=".repeat(60).bright_white());
        println!("{}", "🚶 Choose your journey:".bright_yellow().bold());
        println!("{}. {} (morning commute)", "1".bright_green(), "Home → Work".bright_white());
        println!("{}. {} (lunch break)", "2".bright_green(), "Work → Cafe".bright_white());
        println!("{}. {} (evening workout)", "3".bright_green(), "Home → Gym".bright_white());
        println!("{}. {} (weekend stroll)", "4".bright_green(), "Cafe → Park".bright_white());
        println!("{}. {} (shopping trip)", "5".bright_green(), "Home → Store".bright_white());
        println!("{}. {}", "6".bright_blue(), "Custom route".bright_white());
        println!("{}. {}", "7".bright_magenta(), "Performance comparison".bright_white());
        println!("{}. {}", "8".bright_red(), "Exit".bright_white());
        
        print!("\n{} ", "Enter your choice (1-8):".bright_yellow());
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        match input.trim() {
            "1" => find_and_display_path(&city, &graph, "home", "work", true),
            "2" => find_and_display_path(&city, &graph, "work", "cafe", true),
            "3" => find_and_display_path(&city, &graph, "home", "gym", true),
            "4" => find_and_display_path(&city, &graph, "cafe", "park", true),
            "5" => find_and_display_path(&city, &graph, "home", "store", true),
            "6" => custom_route(&city, &graph),
            "7" => performance_comparison(&city, &graph),
            "8" => {
                println!("\n{} {}", "👋".bright_yellow(), "Thanks for exploring the city!".bright_green().bold());
                break;
            }
            _ => {
                println!("{} {}", "❌".bright_red(), "Invalid choice. Please enter 1-8.".bright_red());
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn find_and_display_path(city: &CityGrid, graph: &DirectedGraph<OrderedFloat<f64>>, from: &str, to: &str, animate: bool) {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen
    
    // Normalize location names to lowercase for case-insensitive matching
    let from_lower = from.to_lowercase();
    let to_lower = to.to_lowercase();
    
    // Find the closest match if exact match not found
    let from_pos = city.locations.get(&from_lower).or_else(|| {
        // Try to find a case-insensitive match
        city.locations.keys()
            .find(|k| k.to_lowercase() == from_lower)
            .and_then(|k| city.locations.get(k))
    });
    
    let to_pos = city.locations.get(&to_lower).or_else(|| {
        // Try to find a case-insensitive match
        city.locations.keys()
            .find(|k| k.to_lowercase() == to_lower)
            .and_then(|k| city.locations.get(k))
    });
    
    // Check if locations exist
    if from_pos.is_none() {
        println!("{} Location '{}' not found!", "❌".bright_red(), from.bright_yellow());
        println!("\n{} Available locations: {}", "📍".bright_blue(), 
                city.locations.keys().map(|k| format!("'{}'", k)).collect::<Vec<_>>().join(", ").bright_cyan());
        println!("\n{} {}", "👉".bright_yellow(), "Press Enter to continue...".bright_yellow().bold());
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        return;
    }
    
    if to_pos.is_none() {
        println!("{} Location '{}' not found!", "❌".bright_red(), to.bright_yellow());
        println!("\n{} Available locations: {}", "📍".bright_blue(), 
                city.locations.keys().map(|k| format!("'{}'", k)).collect::<Vec<_>>().join(", ").bright_cyan());
        println!("\n{} {}", "👉".bright_yellow(), "Press Enter to continue...".bright_yellow().bold());
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        return;
    }
    
    let (fx, fy) = *from_pos.unwrap();
    let (tx, ty) = *to_pos.unwrap();
    let source = city.coord_to_vertex(fx, fy);
    let target = city.coord_to_vertex(tx, ty);
    
    println!("{} Finding path from {} to {}...", 
             "🗺️".bright_blue(), 
             from.bright_green().bold(), 
             to.bright_red().bold());
    
    let fast_sssp = FastSSSP::new();
    let start_time = std::time::Instant::now();
    
    // Add debug output to track algorithm progress
    println!("Computing shortest path from {} ({},{}) to {} ({},{})", 
             from, fx, fy, to, tx, ty);
             
    let result = fast_sssp.compute_shortest_paths(graph, source).unwrap();
    let computation_time = start_time.elapsed();
    
    // Debug output for result - Safely handle Option with map
    if let Some(dist) = result.distances[target] {
        println!("Path computation complete. Distance to target: {}", dist.into_inner());
    } else {
        println!("Path computation complete. No path found to target.");
    }
    
    println!("Checking if path exists...");
    if let Some(distance) = result.distances[target] {
        println!("Path exists with distance: {}", distance.into_inner());
        println!("Getting path from source to target...");
        // Custom path reconstruction to avoid infinite loops
        println!("Using custom path reconstruction...");
        let path = match reconstruct_path(&result, target) {
            Some(p) => {
                println!("Path found with {} vertices", p.len());
                p
            },
            None => {
                println!("ERROR: Failed to get path even though distance exists!");
                println!("This could be due to a cycle in the path or missing predecessors.");
                return;
            }
        };
        
        println!("About to display path details...");
        println!("{} {}", "✅".bright_green(), "Path found!".bright_green().bold());
        println!("{} {:.2} blocks", "📏 Distance:".bright_white(), distance.into_inner().to_string().bright_yellow());
        println!("{} {} moves", "👣 Steps:".bright_white(), (path.len() - 1).to_string().bright_yellow());
        println!("{} {:?}", "⚡ Computation time:".bright_white(), format!("{:?}", computation_time).bright_cyan());
        
        println!("Sleeping for 1 second before visualization...");
        thread::sleep(Duration::from_secs(1));
        
        println!("Starting path visualization...");
        visualize_city_with_animated_path(city, Some(&path), animate);
        println!("Path visualization complete.");
        
        println!("\n{}", "📋 Step-by-step directions:".bright_white().bold());
        for i in 1..path.len() {
            let (px, py) = city.vertex_to_coord(path[i-1]);
            let (cx, cy) = city.vertex_to_coord(path[i]);
            let direction = match (cx as i32 - px as i32, cy as i32 - py as i32) {
                (0, 1) => "South ⬇️",
                (1, 0) => "East ➡️", 
                (0, -1) => "North ⬆️",
                (-1, 0) => "West ⬅️",
                (1, 1) => "Southeast ↘️",
                (1, -1) => "Northeast ↗️",
                (-1, 1) => "Southwest ↙️",
                (-1, -1) => "Northwest ↖️",
                _ => "Unknown ❓",
            };
            println!("  {}. Go {} to ({}, {})", 
                     i.to_string().bright_blue(), 
                     direction.bright_white(), 
                     cx.to_string().bright_yellow(), 
                     cy.to_string().bright_yellow());
        }
    } else {
        println!("No path exists from source to target!");
        println!("{} No path found from {} to {}!", 
                 "❌".bright_red(), 
                 from.bright_green(), 
                 to.bright_red());
    }
    
    println!("\n{} {}", "👉".bright_yellow(), "Press Enter to continue...".bright_yellow().bold());
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

fn custom_route(city: &CityGrid, graph: &DirectedGraph<OrderedFloat<f64>>) {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen
    println!("{}", "📍 CUSTOM ROUTE PLANNER".bright_magenta().bold());
    println!();
    
    // Sort locations for better display
    let mut location_names: Vec<_> = city.locations.keys().collect();
    location_names.sort();
    
    println!("{}", "Available locations:".bright_white().bold());
    for (i, name) in location_names.iter().enumerate() {
        let (x, y) = city.locations.get(*name).unwrap();
        println!("  {}. {} at ({}, {})", 
                 (i+1).to_string().bright_blue(), 
                 name.bright_cyan(),
                 x.to_string().bright_yellow(),
                 y.to_string().bright_yellow());
    }
    
    print!("\n{} ", "Enter starting location (name or number):".bright_yellow());
    io::stdout().flush().unwrap();
    let mut from = String::new();
    io::stdin().read_line(&mut from).unwrap();
    let from = from.trim();
    
    // Handle numeric input
    let from_location = if let Ok(num) = from.parse::<usize>() {
        if num > 0 && num <= location_names.len() {
            location_names[num-1].to_string()
        } else {
            from.to_string()
        }
    } else {
        from.to_string()
    };
    
    print!("{} ", "Enter destination (name or number):".bright_yellow());
    io::stdout().flush().unwrap();
    let mut to = String::new();
    io::stdin().read_line(&mut to).unwrap();
    let to = to.trim();
    
    // Handle numeric input
    let to_location = if let Ok(num) = to.parse::<usize>() {
        if num > 0 && num <= location_names.len() {
            location_names[num-1].to_string()
        } else {
            to.to_string()
        }
    } else {
        to.to_string()
    };
    
    find_and_display_path(city, graph, &from_location, &to_location, true);
}

fn performance_comparison(city: &CityGrid, graph: &DirectedGraph<OrderedFloat<f64>>) {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen
    println!("{}", "⚡ PERFORMANCE COMPARISON".bright_magenta().bold());
    println!("{}", "Fast SSSP vs Dijkstra Algorithm".bright_white());
    println!();
    
    let source = city.coord_to_vertex(0, 0); // home position
    
    println!("{} Running algorithms...", "🔄".bright_blue());
    
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
    
    println!("\n{}", "📊 RESULTS".bright_green().bold());
    println!("{}", "=".repeat(50).bright_white());
    println!("{} {} vertices, {} edges", 
             "🏗️  Graph size:".bright_white(), 
             graph.vertex_count().to_string().bright_yellow(), 
             graph.edge_count().to_string().bright_yellow());
    println!("{} {:?}", "⏱️ Fast SSSP time:".bright_white(), format!("{:?}", fast_time).bright_cyan());
    println!("{} {:?}", "⏱️ Dijkstra time:".bright_white(), format!("{:?}", dijkstra_time).bright_cyan());
    
    if fast_time < dijkstra_time {
        let speedup = dijkstra_time.as_nanos() as f64 / fast_time.as_nanos() as f64;
        println!("{} Fast SSSP is {:.2}x faster!", "🏆".bright_yellow(), speedup.to_string().bright_green().bold());
    } else {
        println!("{} {}", "📝".bright_yellow(), "Note: For small graphs, overhead may make Dijkstra appear faster".bright_white());
    }
    
    println!("\n{}", "🎯 Distance verification:".bright_white().bold());
    let mut all_match = true;
    for (name, &(x, y)) in &city.locations {
        if name != "home" {
            let target = city.coord_to_vertex(x, y);
            let fast_dist = fast_result.distances[target];
            let dijkstra_dist = dijkstra_result.distances[target];
            
            match (fast_dist, dijkstra_dist) {
                (Some(fd), Some(dd)) => {
                    let fd_val = fd.into_inner();
                    let dd_val = dd.into_inner();
                    let match_symbol = if (fd_val - dd_val).abs() < 1e-10 { "✅".bright_green() } else { "❌".bright_red() };
                    println!("  {} {}: Fast={:.2}, Dijkstra={:.2} {}", 
                             name.bright_white(), match_symbol, fd_val.to_string().bright_yellow(), dd_val.to_string().bright_cyan(), 
                             if (fd_val - dd_val).abs() < 1e-10 { "✓".bright_green() } else { "✗".bright_red() });
                    if (fd_val - dd_val).abs() >= 1e-10 { all_match = false; }
                }
                _ => println!("  {} {}: Unreachable", "⚫".bright_black(), name.bright_white()),
            }
        }
    }
    
    if all_match {
        println!("\n{} {}", "🎉".bright_green(), "All distances match perfectly!".bright_green().bold());
    }
    
    println!("\n{}", "Press Enter to continue...".bright_black());
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

/// Custom path reconstruction function that avoids infinite loops
fn reconstruct_path<W>(result: &ShortestPathResult<W>, target: usize) -> Option<Vec<usize>>
where
    W: Float + Zero + Debug + Copy + Ord,
{
    if target >= result.predecessors.len() || result.distances[target].is_none() {
        println!("Target is out of bounds or unreachable");
        return None;
    }
    
    let mut path = Vec::new();
    let mut current = target;
    let mut visited = HashSet::new();
    
    // Build path in reverse order
    while current != result.source {
        // Safety check to prevent infinite loops
        if !visited.insert(current) {
            println!("WARNING: Cycle detected in path reconstruction at vertex {}", current);
            return None;
        }
        
        path.push(current);
        match result.predecessors[current] {
            Some(pred) => current = pred,
            None => {
                println!("No predecessor for vertex {}", current);
                return None;
            }
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

fn main() {
    interactive_demo();
}
