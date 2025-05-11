use crate::models::access_token::AccessToken;
use crate::repositories::access_token::AccessTokenRepository;
use crate::repositories::base::Repository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

pub struct AccessTokenService {
    access_token_repository: AccessTokenRepository,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenFilter {
    pub key: Option<String>,
    pub algorithm: Option<String>,
    pub enabled: Option<bool>,
}

impl From<AccessTokenFilter> for mongodb::bson::Document {
    fn from(val: AccessTokenFilter) -> Self {
        let mut doc = mongodb::bson::Document::new();
        if let Some(key) = val.key {
            doc.insert("key", key);
        }
        if let Some(algorithm) = val.algorithm {
            doc.insert("algorithm", algorithm);
        }
        if let Some(enabled) = val.enabled {
            doc.insert("enabled", enabled);
        }
        doc
    }
}

impl AccessTokenService {
    pub fn new(database: Database) -> Result<Self, Error> {
        let access_token_repository = AccessTokenRepository::new(database)?;
        Ok(Self {
            access_token_repository,
        })
    }

    pub async fn create_access_token(
        &self,
        access_token: AccessToken,
    ) -> Result<AccessToken, Error> {
        let result = self
            .access_token_repository
            .create(access_token.clone())
            .await?;
        let id = result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| anyhow::anyhow!("Invalid ObjectId"))?;

        // Fetch the newly created access token
        let inserted_access_token = self
            .access_token_repository
            .read(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch created access token"))?;

        Ok(inserted_access_token)
    }

    pub async fn get_access_token(&self, id: ObjectId) -> Result<Option<AccessToken>, Error> {
        self.access_token_repository.read(id).await
    }

    pub async fn update_access_token(
        &self,
        id: ObjectId,
        access_token: AccessToken,
    ) -> Result<AccessToken, Error> {
        let result = self
            .access_token_repository
            .update(id, access_token)
            .await?;

        if result.modified_count > 0 {
            log::info!("Access token updated successfully: {:?}", id);
        } else {
            log::error!("Failed to update access token: {:?}", id);
        }

        let updated_access_token = self
            .access_token_repository
            .read(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch updated access token"))?;

        Ok(updated_access_token)
    }

    pub async fn delete_access_token(&self, id: ObjectId) -> Result<bool, Error> {
        let result = self.access_token_repository.delete(id).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn get_access_tokens(
        &self,
        filter: AccessTokenFilter,
    ) -> Result<Vec<AccessToken>, Error> {
        let filter_doc = filter.into();
        let access_tokens = self.access_token_repository.find(filter_doc).await?;
        Ok(access_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use crate::types::Algorithm;
    use anyhow::Result;
    use chrono::{Duration, Utc};
    use tokio;

    async fn setup_access_tokens_for_filter_tests(
        access_token_service: &AccessTokenService,
    ) -> Result<(), Error> {
        // Clean up any existing data first
        let collection = access_token_service.access_token_repository.collection()?;
        let db = collection.client().database(&collection.namespace().db);
        cleanup_test_db(db).await?;

        // Create multiple access tokens for testing filters
        let access_token1 = AccessToken::new(
            "key-1".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7),
        );
        let access_token2 = AccessToken::new(
            "key-2".to_string(),
            Algorithm::RSA,
            Utc::now() + Duration::days(14),
        );
        let access_token3 = AccessToken::new(
            "key-3".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(21),
        );

        access_token_service
            .create_access_token(access_token1)
            .await?;
        access_token_service
            .create_access_token(access_token2)
            .await?;
        access_token_service
            .create_access_token(access_token3)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_create_access_token() -> Result<(), Error> {
        let db = setup_test_db("access_token_service").await?;
        let access_token_service = AccessTokenService::new(db.clone())?;

        let access_token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7),
        );

        let result = access_token_service
            .create_access_token(access_token)
            .await?;

        assert!(result.id().is_some());
        assert_eq!(result.key(), "test-key");
        assert!(matches!(result.algorithm(), Algorithm::HMAC));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_new() -> Result<(), Error> {
        let db = setup_test_db("access_token_service").await?;
        let access_token_service = AccessTokenService::new(db.clone())?;

        assert!(
            access_token_service
                .access_token_repository
                .collection()
                .is_ok()
        );

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_access_token() -> Result<(), Error> {
        let db = setup_test_db("access_token_service").await?;
        let access_token_service = AccessTokenService::new(db.clone())?;

        let access_token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7),
        );

        let result = access_token_service
            .create_access_token(access_token)
            .await?;
        assert!(result.id().is_some());

        let access_token = access_token_service
            .get_access_token(*result.id().unwrap())
            .await?;

        assert!(access_token.is_some());
        let access_token = access_token.unwrap();
        assert_eq!(access_token.key(), "test-key");
        assert!(matches!(access_token.algorithm(), Algorithm::HMAC));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_access_token() -> Result<(), Error> {
        let db = setup_test_db("access_token_service").await?;
        let access_token_service = AccessTokenService::new(db.clone())?;

        let access_token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7),
        );

        let result = access_token_service
            .create_access_token(access_token)
            .await?;
        assert!(result.id().is_some());

        let updated_access_token = AccessToken::new(
            "updated-key".to_string(),
            Algorithm::RSA,
            Utc::now() + Duration::days(14),
        );

        let result = access_token_service
            .update_access_token(*result.id().unwrap(), updated_access_token)
            .await?;

        assert!(result.id().is_some());
        assert_eq!(result.key(), "updated-key");
        assert!(matches!(result.algorithm(), Algorithm::RSA));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_access_token() -> Result<(), Error> {
        let db = setup_test_db("access_token_service").await?;
        let access_token_service = AccessTokenService::new(db.clone())?;

        let access_token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::HMAC,
            Utc::now() + Duration::days(7),
        );

        let result = access_token_service
            .create_access_token(access_token)
            .await?;
        assert!(result.id().is_some());

        let result = access_token_service
            .delete_access_token(*result.id().unwrap())
            .await?;

        assert!(result);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_access_tokens_no_filter() -> Result<(), Error> {
        let db = setup_test_db("access_token_service").await?;
        let access_token_service = AccessTokenService::new(db.clone())?;

        // Setup test data
        setup_access_tokens_for_filter_tests(&access_token_service).await?;

        // Test with empty filter (should return all access tokens)
        let filter = AccessTokenFilter {
            key: None,
            algorithm: None,
            enabled: None,
        };

        let access_tokens = access_token_service.get_access_tokens(filter).await?;

        assert_eq!(
            access_tokens.len(),
            3,
            "Should have retrieved all 3 access tokens with no filter"
        );

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_access_tokens_filter_by_key() -> Result<(), Error> {
        let db = setup_test_db("access_token_service").await?;
        let access_token_service = AccessTokenService::new(db.clone())?;

        // Setup test data
        setup_access_tokens_for_filter_tests(&access_token_service).await?;

        // Test filtering by key
        let filter = AccessTokenFilter {
            key: Some("key-1".to_string()),
            algorithm: None,
            enabled: None,
        };

        let access_tokens = access_token_service.get_access_tokens(filter).await?;

        assert_eq!(
            access_tokens.len(),
            1,
            "Should have retrieved 1 access token when filtering by key"
        );
        assert_eq!(access_tokens[0].key(), "key-1");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_access_tokens_filter_by_algorithm() -> Result<(), Error> {
        let db = setup_test_db("access_token_service").await?;
        let access_token_service = AccessTokenService::new(db.clone())?;

        // Setup test data
        setup_access_tokens_for_filter_tests(&access_token_service).await?;

        // Test filtering by algorithm
        let filter = AccessTokenFilter {
            key: None,
            algorithm: Some("HMAC".to_string()),
            enabled: None,
        };

        let access_tokens = access_token_service.get_access_tokens(filter).await?;

        assert_eq!(
            access_tokens.len(),
            2,
            "Should have retrieved 2 access tokens when filtering by algorithm"
        );
        assert!(
            access_tokens
                .iter()
                .all(|token| token.algorithm() == &Algorithm::HMAC)
        );

        cleanup_test_db(db).await?;
        Ok(())
    }
}
