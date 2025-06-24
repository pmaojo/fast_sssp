use fast_sssp::web::server::{start_server, ServerConfig};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let port = if args.len() > 1 {
        args[1].parse().unwrap_or(3005)
    } else {
        3005
    };
    
    let config = ServerConfig {
        port,
        ..Default::default()
    };
    
    println!("🔧 Starting FastSSSP Web Server...");
    println!("⚙️  Configuration:");
    println!("   📡 Port: {}", config.port);
    println!("   📁 Static files: {}", config.static_dir);
    println!("   🌐 CORS enabled: {}", config.enable_cors);
    println!("   👥 Max sessions: {}", config.max_sessions);
    println!("   ⏰ Session timeout: {} minutes", config.session_timeout_minutes);
    println!();
    
    // Start the server
    start_server(port).await?;
    
    Ok(())
}
