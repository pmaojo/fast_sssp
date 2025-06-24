use axum::{
    http::{header, Method},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::web::api::{create_router, AppState};

/// Start the web server
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState::new();
    
    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);
    
    // Build the application with middleware
    let app = Router::new()
        // API routes
        .merge(create_router())
        // Static file serving for the web frontend
        .nest_service("/", ServeDir::new("web"))
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .into_inner(),
        )
        .with_state(app_state);
    
    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🚀 FastSSSP Web Server starting on http://{}", addr);
    println!("📊 API documentation available at http://{}/api", addr);
    println!("🎨 Web interface available at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Configuration for the web server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub static_dir: String,
    pub enable_cors: bool,
    pub max_sessions: usize,
    pub session_timeout_minutes: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3005,
            static_dir: "web".to_string(),
            enable_cors: true,
            max_sessions: 1000,
            session_timeout_minutes: 60,
        }
    }
}

/// Start the web server with custom configuration
pub async fn start_server_with_config(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState::new();
    
    let mut app = Router::new()
        .merge(create_router())
        .nest_service("/", ServeDir::new(&config.static_dir))
        .with_state(app_state);
    
    if config.enable_cors {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);
        
        app = app.layer(cors);
    }
    
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    println!("🚀 FastSSSP Web Server starting on http://{}", addr);
    println!("📊 API documentation available at http://{}/api", addr);
    println!("🎨 Web interface available at http://{}", addr);
    println!("📁 Serving static files from: {}", config.static_dir);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
