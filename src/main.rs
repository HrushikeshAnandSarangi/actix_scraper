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
    init();
    let port_str = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let port = port_str.parse::<u16>().expect("PORT must be a valid number");
    let bind_address = format!("0.0.0.0:{}", port);
    let server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/health", web::get().to(health))
            .route("/scrape", web::post().to(scrape))
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(bind_address)?;
    
    server.run().await?;
    Ok(())
}