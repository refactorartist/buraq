use mongodb::Client;
use std::sync::Arc;

pub async fn create_database_client(database_uri: &str) -> Result<Arc<Client>, anyhow::Error> {
    let client = mongodb::Client::with_uri_str(&database_uri)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create MongoDB client: {}", e))?;

    Ok(Arc::new(client))
}
