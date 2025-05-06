use dotenvy;
use mongodb;
use std::env;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    pub host: String,
    pub port: u16,
    pub database_uri: String,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub application: ApplicationConfig,
}


#[derive(Debug)]
pub struct AppData {
    pub config: AppConfig,
    pub database: Arc<mongodb::Client>,
}

impl AppConfig {
    pub fn from_env(load_env: Option<bool>) -> Result<Self, anyhow::Error> {
        if load_env == Some(true) {
            dotenvy::dotenv().ok();
        }

        println!(
            "env::var(\"BURAQ_DATABASE_URI\"): {:?}",
            env::var("BURAQ_DATABASE_URI")
        );

        let database_uri = match env::var("BURAQ_DATABASE_URI") {
            Ok(uri) => uri,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "BURAQ_DATABASE_URI environment variable is not set"
                ));
            }
        };

        println!("database_uri: {}", database_uri);

        let host = match env::var("BURAQ_HOST") {
            Ok(h) => h,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "BURAQ_HOST environment variable is not set"
                ));
            }
        };

        println!("host: {}", host);

        let port = match env::var("BURAQ_PORT") {
            Ok(p) => p,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "BURAQ_PORT environment variable is not set"
                ));
            }
        };

        println!("port: {}", port);

        Ok(Self {
            application: ApplicationConfig {
                host,
                port: port.parse()?,
                database_uri,
            },
        })
    }

}
