use dotenvy;
use mongodb;
use std::sync::Arc;

pub struct AppConfig {
    pub database: Arc<mongodb::Client>,
}

impl AppConfig {
    pub async fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        let database_uri = dotenvy::var("DATABASE_URI")?;
        let client = mongodb::Client::with_uri_str(&database_uri)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create MongoDB client: {}", e))?;

        Ok(Self {
            database: Arc::new(client),
        })
    }
}
