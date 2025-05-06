use mongodb::Client;
use std::sync::Arc;

/// Creates a MongoDB client wrapped in an `Arc`.
///
/// # Arguments
///
/// * `database_uri` - A string slice that holds the URI of the MongoDB database.
///
/// # Returns
///
/// * `Result<Arc<Client>, anyhow::Error>` - On success, returns an `Arc` containing the MongoDB `Client`.
///   On failure, returns an `anyhow::Error` indicating the reason for failure.
///
/// # Errors
///
/// This function will return an error if the MongoDB client cannot be created with the provided URI.
///
/// # Examples
///
/// ```no_run
/// use buraq::utils::database::create_database_client;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///     let database_uri = "mongodb://localhost:27017";
///     let client_result = create_database_client(database_uri).await;
///     match client_result {
///         Ok(client) => println!("Successfully created client with strong count: {}", Arc::strong_count(&client)),
///         Err(e) => eprintln!("Error creating client: {}", e),
///     }
/// }
/// ```
pub async fn create_database_client(database_uri: &str) -> Result<Arc<Client>, anyhow::Error> {
    let client = mongodb::Client::with_uri_str(&database_uri)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create MongoDB client: {}", e))?;

    Ok(Arc::new(client))
}
