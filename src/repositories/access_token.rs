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
    ///
    /// # Example
    ///
    /// ```no_run
    /// use buraq::repositories::access_token::AccessTokenRepository;
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
    ///     let repo = AccessTokenRepository::new(db)?;
    ///     Ok(())
    /// }
    /// ```
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
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use crate::types::Algorithm;
    use chrono::{Utc, Duration};
    use mongodb::bson::{Bson, doc};
    use tokio;

    #[tokio::test]
    async fn test_create_access_token() -> Result<()> {
        let db = setup_test_db("access_token").await?;
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
        let db = setup_test_db("access_token").await?;
        let repo = AccessTokenRepository::new(db.clone())?;

        let token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7)
        );
        let result = repo.create(token).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let read_token = repo.read(id).await?;
        assert!(read_token.is_some());
        let read_token = read_token.unwrap();
        assert_eq!(read_token.key(), "test-key");
        assert_eq!(read_token.algorithm(), &Algorithm::HMAC);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_access_token() -> Result<()> {
        let db = setup_test_db("access_token").await?;
        let repo = AccessTokenRepository::new(db.clone())?;

        let token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7)
        );
        let result = repo.create(token).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let updated_token = AccessToken::new(
            "updated-key".to_string(),
            Algorithm::RSA,
            Utc::now() + Duration::days(14)
        );
        let update_result = repo.update(id, updated_token).await?;
        assert_eq!(update_result.modified_count, 1);

        let read_token = repo.read(id).await?.unwrap();
        assert_eq!(read_token.key(), "updated-key");
        assert_eq!(read_token.algorithm(), &Algorithm::RSA);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_access_token() -> Result<()> {
        let db = setup_test_db("access_token").await?;
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
        let db = setup_test_db("access_token").await?;
        let repo = AccessTokenRepository::new(db.clone())?;

        let token1 = AccessToken::new(
            "key-1".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7)
        );
        let token2 = AccessToken::new(
            "key-2".to_string(),
            Algorithm::RSA,
            Utc::now() + Duration::days(14)
        );
        repo.create(token1).await?;
        repo.create(token2).await?;

        let tokens = repo.find(doc! {}).await?;
        assert_eq!(tokens.len(), 2);

        let filtered_tokens = repo.find(doc! { "key": "key-1" }).await?;
        assert_eq!(filtered_tokens.len(), 1);
        assert_eq!(filtered_tokens[0].key(), "key-1");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
