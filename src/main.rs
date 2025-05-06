use actix_web::{web, App, HttpServer};
use buraq::config::AppConfig; 
use tokio::signal;
use std::time::Duration;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv()?;
    env_logger::init();  

    let app_config = AppConfig::from_env().await?; 
    let host = app_config.application.host.clone();
    let port = app_config.application.port;
    let app_data = web::Data::new(app_config);

    println!("Starting the server on {}:{}", &host, &port);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .route("/", web::get().to(|| async { "Hello, World!" }))
    })
    .bind((host, port))?
    .shutdown_timeout(30) // 30 seconds graceful shutdown timeout
    .workers(4) // Set number of workers
    .keep_alive(Duration::from_secs(75)) // Keep-alive timeout
    .run();

    // Create a handle to the server
    let server_handle = server.handle();

    // Wait for the server to finish or for ctrl+c
    tokio::select! {
        _ = server => {
            println!("Server finished");
        }
        _ = signal::ctrl_c() => {
            println!("Received ctrl+c signal, shutting down gracefully");
            
            // Stop accepting new connections
            server_handle.stop(true).await;
            
            // Give some time for cleanup
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("Cleanup completed, server shutting down");
        }
    }
    
    Ok(())
}