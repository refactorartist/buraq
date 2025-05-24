use actix_web::{App, HttpServer, web};
use buraq::config::{AppConfig, AppData};
use buraq::utils::database::create_database_client;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

/// The main entry point for the application.
///
/// This function initializes the environment, sets up the application configuration,
/// and starts the Actix web server. It also handles graceful shutdown on receiving
/// a Ctrl+C signal.
///
/// # Returns
///
/// * `Result<(), anyhow::Error>` - On success, returns an empty tuple. On failure,
///   returns an `anyhow::Error` indicating the reason for failure.
///
/// # Errors
///
/// This function will return an error if the environment variables cannot be loaded,
/// the application configuration cannot be created, or the server fails to start.
#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load environment variables from a .env file
    dotenvy::dotenv()?;
    // Initialize the logger
    env_logger::init();

    // Create application configuration from environment variables
    let app_config = AppConfig::from_env(Some(true))?;
    let host = app_config.application.host.clone();
    let port = app_config.application.port;

    // Create application data including the MongoDB client
    let mongo_client = create_database_client(&app_config.application.database_uri).await?;
    let database = Arc::new(mongo_client.database(&app_config.application.database_name));
    let app_data = web::Data::new(AppData {
        config: Some(app_config.clone()),
        mongo_client: Some(mongo_client),
        database: Some(database),
    });

    println!("Starting the server on {}:{}", &host, &port);

    // Configure and start the Actix web server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .configure(buraq::routes::project::configure_routes)
            .configure(buraq::routes::access_token::configure_routes)
            .configure(buraq::routes::service_account::configure_routes)
            .configure(buraq::routes::environment::configure_routes)
            .configure(buraq::routes::project_access::configure_routes)
            .configure(buraq::routes::project_scope::configure_routes)
    })
    .bind((host, port))?
    .shutdown_timeout(30) // 30 seconds graceful shutdown timeout
    .workers(4) // Set number of workers
    .keep_alive(Duration::from_secs(75)) // Keep-alive timeout
    .run();

    // Create a handle to the server for managing shutdown
    let server_handle = server.handle();

    // Wait for the server to finish or for a Ctrl+C signal
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
