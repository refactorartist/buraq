use dotenvy;
use mongodb;
use std::env;
use std::sync::Arc;

/// Configuration for the application, including host, port, and database URI.
#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    /// The host address of the application.
    pub host: String,
    /// The port number the application listens on.
    pub port: u16,
    /// The URI for the application's database.
    pub database_uri: String,
    /// The name of the database.
    pub database_name: String,
}

/// Wrapper for application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// The application-specific configuration.
    pub application: ApplicationConfig,
}

/// Holds the application configuration and a MongoDB client.
#[derive(Debug, Default)]
pub struct AppData {
    /// The application configuration.
    pub config: Option<AppConfig>,
    /// The MongoDB client wrapped in an `Arc`.
    pub mongo_client: Option<Arc<mongodb::Client>>,
    pub database: Option<Arc<mongodb::Database>>,
}

impl AppConfig {
    /// Constructs an `AppConfig` from environment variables.
    ///
    /// # Arguments
    ///
    /// * `load_env` - An optional boolean to determine if the environment variables should be loaded using dotenv.
    ///
    /// # Returns
    ///
    /// * `Result<AppConfig, anyhow::Error>` - On success, returns an `AppConfig` instance.
    ///   On failure, returns an `anyhow::Error` indicating the missing environment variable.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the required environment variables (`BURAQ_DATABASE_URI`, `BURAQ_HOST`, `BURAQ_PORT`) are not set.
    ///
    pub fn from_env(load_env: Option<bool>) -> Result<Self, anyhow::Error> {
        if load_env == Some(true) {
            dotenvy::dotenv().ok();
        }

        let database_uri = match env::var("BURAQ_DATABASE_URI") {
            Ok(uri) => uri,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "BURAQ_DATABASE_URI environment variable is not set"
                ));
            }
        };

        let database_name = match env::var("BURAQ_DATABASE_NAME") {
            Ok(name) => name,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "BURAQ_DATABASE_NAME environment variable is not set"
                ));
            }
        };

        let host = match env::var("BURAQ_HOST") {
            Ok(h) => h,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "BURAQ_HOST environment variable is not set"
                ));
            }
        };

        let port = match env::var("BURAQ_PORT") {
            Ok(p) => p,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "BURAQ_PORT environment variable is not set"
                ));
            }
        };

        Ok(Self {
            application: ApplicationConfig {
                host,
                port: port.parse()?,
                database_uri,
                database_name,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_from_env() {
        temp_env::with_vars(
            vec![
                ("BURAQ_DATABASE_URI", Some("mongodb://localhost:27017")),
                ("BURAQ_DATABASE_NAME", Some("test_db")),
                ("BURAQ_HOST", Some("127.0.0.1")),
                ("BURAQ_PORT", Some("8080")),
            ],
            || {
                let config = AppConfig::from_env(Some(false)).expect("Failed to load config");

                assert_eq!(config.application.host, "127.0.0.1");
                assert_eq!(config.application.port, 8080);
                assert_eq!(config.application.database_uri, "mongodb://localhost:27017");
                assert_eq!(config.application.database_name, "test_db");
            },
        );
    }

    #[test]
    fn test_app_config_missing_database_uri() {
        temp_env::with_vars(
            vec![
                ("BURAQ_DATABASE_NAME", Some("test_db")),
                ("BURAQ_HOST", Some("127.0.0.1")),
                ("BURAQ_PORT", Some("8080")),
            ],
            || {
                let result = AppConfig::from_env(Some(false));
                assert!(result.is_err());
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "BURAQ_DATABASE_URI environment variable is not set"
                );
            },
        );
    }

    #[test]
    fn test_app_config_missing_database_name() {
        temp_env::with_vars(
            vec![
                ("BURAQ_DATABASE_URI", Some("mongodb://localhost:27017")),
                ("BURAQ_HOST", Some("127.0.0.1")),
                ("BURAQ_PORT", Some("8080")),
            ],
            || {
                let result = AppConfig::from_env(Some(false));
                assert!(result.is_err());
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "BURAQ_DATABASE_NAME environment variable is not set"
                );
            },
        );
    }

    #[test]
    fn test_app_config_missing_host() {
        temp_env::with_vars(
            vec![
                ("BURAQ_DATABASE_URI", Some("mongodb://localhost:27017")),
                ("BURAQ_DATABASE_NAME", Some("test_db")),
                ("BURAQ_PORT", Some("8080")),
            ],
            || {
                let result = AppConfig::from_env(Some(false));
                assert!(result.is_err());
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "BURAQ_HOST environment variable is not set"
                );
            },
        );
    }

    #[test]
    fn test_app_config_missing_port() {
        temp_env::with_vars(
            vec![
                ("BURAQ_DATABASE_URI", Some("mongodb://localhost:27017")),
                ("BURAQ_DATABASE_NAME", Some("test_db")),
                ("BURAQ_HOST", Some("127.0.0.1")),
            ],
            || {
                let result = AppConfig::from_env(Some(false));
                assert!(result.is_err());
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "BURAQ_PORT environment variable is not set"
                );
            },
        );
    }
}
