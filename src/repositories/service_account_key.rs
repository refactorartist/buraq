use crate::models::service_account_key::ServiceAccountKey;
use crate::repositories::base::Repository;
use anyhow::Result;
use mongodb::{Collection, Database, IndexModel};
use mongodb::options::IndexOptions;
use mongodb::bson::doc;

/// Repository for managing ServiceAccountKey documents in MongoDB.
///
/// Provides CRUD operations for ServiceAccountKey entities.
pub struct ServiceAccountKeyRepository {
    collection: Collection<ServiceAccountKey>,
}

impl ServiceAccountKeyRepository {
    /// Creates a new ServiceAccountKeyRepository instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ServiceAccountKeyRepository or an error if collection creation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use buraq::repositories::service_account_key::ServiceAccountKeyRepository;
    /// use mongodb::Client;
    /// use buraq::utils::database::create_database_client;
    /// use dotenvy::dotenv;
    /// use buraq::config::AppConfig;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     dotenv().ok();
    ///
    ///     let app_config = AppConfig::from_env(Some(true))?;
    ///     let client = create_database_client(&app_config.application.database_uri).await?;
    ///     let db = client.database("test_db");
    ///     let repo = ServiceAccountKeyRepository::new(db).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<ServiceAccountKey>("service_account_key");
        
        // Create unique index on service_account_id
        let options = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { "service_account_id": 1 })
            .options(options)
            .build();
        
        collection.create_index(index).await?;

        Ok(Self { collection })
    }
}

impl Repository<ServiceAccountKey> for ServiceAccountKeyRepository {
    /// Gets the MongoDB collection for ServiceAccountKeys.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Collection or an error if cloning fails.
    fn collection(&self) -> Result<Collection<ServiceAccountKey>, anyhow::Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::types::Algorithm;
    use crate::utils::database::create_database_client;
    use chrono::Utc;
    use dotenvy::dotenv;
    use mongodb::bson::{Bson, oid::ObjectId};
    use tokio;

    async fn setup_test_db() -> Result<Database> {
        dotenv().ok();

        let app_config = AppConfig::from_env(Some(true))?;

        let client = create_database_client(&app_config.application.database_uri).await?;
        let db = client.database("test_db__service_account_key");
        Ok(db)
    }

    async fn cleanup_test_db(db: Database) -> Result<()> {
        db.collection::<ServiceAccountKey>("service_account_key").drop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_service_account_key() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = ServiceAccountKeyRepository::new(db.clone()).await?;

        let service_account_id = ObjectId::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now();

        let service_account_key = ServiceAccountKey::new(
            service_account_id,
            algorithm,
            key.clone(),
            expires_at
        );

        let result = repo.create(service_account_key).await?;

        assert!(matches!(result.inserted_id, Bson::ObjectId(_)));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_service_account_key() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = ServiceAccountKeyRepository::new(db.clone()).await?;

        let service_account_id = ObjectId::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now();

        let service_account_key = ServiceAccountKey::new(
            service_account_id,
            algorithm,
            key.clone(),
            expires_at
        );

        let result = repo.create(service_account_key).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let read_key = repo.read(id).await?;
        assert!(read_key.is_some());
        let read_key = read_key.unwrap();
        assert_eq!(read_key.service_account_id(), &service_account_id);
        assert!(matches!(read_key.algorithm(), &Algorithm::RSA));
        assert_eq!(read_key.key(), key);
        assert_eq!(read_key.expires_at().timestamp(), expires_at.timestamp());

        cleanup_test_db(db).await?;
        Ok(())
    }
}
