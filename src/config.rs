use dotenvy; 



pub struct DatabaseConfig {
    pub uri: String
}

pub struct AppConfig {
    pub database: DatabaseConfig
}

impl AppConfig {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        let database_uri = dotenvy::var("DATABASE_URI")?;
        Ok(
            Self { 
                database: DatabaseConfig { uri: database_uri } 
            }
        )
    }
}