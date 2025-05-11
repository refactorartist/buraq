use crate::models::access_token::AccessToken;
use crate::repositories::base::Repository;
use anyhow::Result;
use mongodb::{Collection, Database};

/// Repository for managing AccessToken documents in MongoDB.
///
/// Provides CRUD operations for AccessToken entities.
pub struct AccessTokenRepository {
    collection: Collection<AccessToken>,
}

impl AccessTokenRepository {
    /// Creates a new AccessTokenRepository instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the AccessTokenRepository or an error if collection creation fails.
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<AccessToken>("access_tokens");
        Ok(Self { collection })
    }
}

impl Repository<AccessToken> for AccessTokenRepository {
    /// Gets the MongoDB collection for AccessTokens.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Collection or an error if cloning fails.
    fn collection(&self) -> Result<Collection<AccessToken>, anyhow::Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::utils::database::create_database_client;
    use crate::types::Algorithm;
    use dotenvy::dotenv;
    use mongodb::bson::{Bson, doc};
    use tokio;
    use chrono::{Utc, Duration};

    async fn setup_test_db() -> Result<Database> {
        dotenv().ok();

        let app_config = AppConfig::from_env(Some(true))?;

        let client = create_database_client(&app_config.application.database_uri).await?;
        let db = client.database("test_db__access_tokens");
        
        // Ensure collection is empty before test
        db.collection::<AccessToken>("access_tokens").drop().await?;
        
        Ok(db)
    }

    async fn cleanup_test_db(db: Database) -> Result<()> {
        db.collection::<AccessToken>("access_tokens").drop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_access_token() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = AccessTokenRepository::new(db.clone())?;

        let token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7)
        );
        let result = repo.create(token).await?;

        assert!(matches!(result.inserted_id, Bson::ObjectId(_)));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_access_token() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = AccessTokenRepository::new(db.clone())?;

        let token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::RSA,
            Utc::now() + Duration::days(7)
        );
        let result = repo.create(token).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let read_token = repo.read(id).await?;
        assert!(read_token.is_some());
        let read_token = read_token.unwrap();
        assert_eq!(read_token.key(), "test-key");
        assert!(matches!(read_token.algorithm(), Algorithm::RSA));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_access_token() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = AccessTokenRepository::new(db.clone())?;

        let token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7)
        );
        let result = repo.create(token).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let mut updated_token = AccessToken::new(
            "updated-key".to_string(),
            Algorithm::RSA,
            Utc::now() + Duration::days(14)
        );
        updated_token.set_id(id);
        let update_result = repo.update(id, updated_token).await?;
        assert_eq!(update_result.modified_count, 1);

        let read_token = repo.read(id).await?.unwrap();
        assert_eq!(read_token.key(), "updated-key");
        assert!(matches!(read_token.algorithm(), Algorithm::RSA));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_access_token() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = AccessTokenRepository::new(db.clone())?;

        let token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7)
        );
        let result = repo.create(token).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let delete_result = repo.delete(id).await?;
        assert_eq!(delete_result.deleted_count, 1);

        let read_token = repo.read(id).await?;
        assert!(read_token.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_access_tokens() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = AccessTokenRepository::new(db.clone())?;

        // Create first token
        let token1 = AccessToken::new(
            "key-1".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7)
        );
        let result1 = repo.create(token1).await?;
        println!("Created first token with id: {:?}", result1.inserted_id);

        // Create second token
        let token2 = AccessToken::new(
            "key-2".to_string(),
            Algorithm::RSA,
            Utc::now() + Duration::days(7)
        );
        let result2 = repo.create(token2).await?;
        println!("Created second token with id: {:?}", result2.inserted_id);

        // Find all tokens
        let tokens = repo.find(doc! {}).await?;
        println!("Found {} tokens", tokens.len());
        for token in &tokens {
            println!("Token key: {}", token.key());
        }
        assert_eq!(tokens.len(), 2);

        // Find specific token
        let filtered_tokens = repo.find(doc! { "key": "key-1" }).await?;
        println!("Found {} filtered tokens", filtered_tokens.len());
        assert_eq!(filtered_tokens.len(), 1);
        assert_eq!(filtered_tokens[0].key(), "key-1");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
