// src/main.rs
use std::env;
use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_files::Files;
use env_logger::init;

mod errors;
mod model;
mod config;
mod login;
mod scraper;
mod handlers;

use handlers::{health, scrape};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print statement to show the main function has started.
    println!("[main] Application starting... 🚀");

    // Initialize the logger.
    init();
    println!("[main] Logger initialized.");

    // Print statement before creating the HttpServer instance.
    println!("[main] Configuring HTTP server...");
    let port_str = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    
    // Parse the port string into a number
    let port = port_str.parse::<u16>().expect("PORT must be a valid number");

    // Bind to 0.0.0.0 (to listen on all interfaces) and the port
    let bind_address = format!("0.0.0.0:{}", port);
    // Create the server but don't start it yet.
    let server = HttpServer::new(|| {
        
        // This closure runs for each worker thread, so you might see this print multiple times.
        println!("[HttpServer] Creating new App instance for a worker. 🏭");
        App::new()
            .wrap(Logger::default())
            .route("/health", web::get().to(health))
            .route("/scrape", web::post().to(scrape))
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(bind_address)?;
    
    // Print statement after successfully binding to the address.
    println!("[main] Server successfully bound to 127.0.0.1:8000. Listening... 🎧");

    // Start the server. The .await call will block here until the server is stopped.
    server.run().await?;

    // This line will only be reached after the server has been shut down gracefully.
    println!("[main] Server has been shut down. Goodbye! 👋");

    Ok(())
}