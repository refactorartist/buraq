use dotenvy;
use mongodb;
use std::sync::Arc;


#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    pub host: String,
    pub port: u16,
}

pub struct AppConfig {
    pub database: Arc<mongodb::Client>,
    pub application: ApplicationConfig,
}

impl AppConfig {
    pub async fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        let database_uri = dotenvy::var("BURAQ_DATABASE_URI")?;
        let client = mongodb::Client::with_uri_str(&database_uri)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create MongoDB client: {}", e))?;

        let host = dotenvy::var("BURAQ_HOST")?;
        let port = dotenvy::var("BURAQ_PORT").map_err(|e| anyhow::anyhow!("Failed to parse port: {}", e))?;

        Ok(Self {
            database: Arc::new(client),
            application: ApplicationConfig {
                host: host,
                port: port.parse()?,
            },
        })
    }
}
