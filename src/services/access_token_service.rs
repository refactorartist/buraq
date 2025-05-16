use crate::models::access_token::{AccessToken, AccessTokenFilter, AccessTokenUpdatePayload};
use crate::repositories::access_token_repository::AccessTokenRepository;
use crate::repositories::base::Repository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;

pub struct AccessTokenService {
    access_token_repository: AccessTokenRepository,
}

impl AccessTokenService {
    pub fn new(database: Database) -> Result<Self, Error> {
        let access_token_repository = AccessTokenRepository::new(database)?;
        Ok(Self {
            access_token_repository,
        })
    }

    pub async fn create(&self, access_token: AccessToken) -> Result<AccessToken, Error> {
        self.access_token_repository.create(access_token).await
    }

    pub async fn get_access_token(&self, id: Uuid) -> Result<Option<AccessToken>, Error> {
        self.access_token_repository.read(id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        access_token: AccessTokenUpdatePayload,
    ) -> Result<AccessToken, Error> {
        self.access_token_repository.update(id, access_token).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        self.access_token_repository.delete(id).await
    }

    pub async fn find(&self, filter: AccessTokenFilter) -> Result<Vec<AccessToken>, Error> {
        self.access_token_repository.find(filter.into()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use crate::types::Algorithm;
    use chrono::{Duration, Utc};
    use futures::TryStreamExt;
    use mongodb::bson::doc;

    async fn setup() -> (AccessTokenService, Database) {
        let db = setup_test_db("access_token_service").await.unwrap();
        let service = AccessTokenService::new(db.clone()).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_access_token() {
        let (service, db) = setup().await;
        let now = Utc::now();
        let token = AccessToken {
            id: None,
            key: "test-key".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: now + Duration::hours(1),
            created_at: now,
            enabled: true,
        };

        let created = service.create(token.clone()).await.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.key, token.key);
        assert_eq!(created.algorithm, token.algorithm);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_access_token() {
        let (service, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };

        let created = service.create(token.clone()).await.unwrap();
        let retrieved = service
            .get_access_token(created.id.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.key, created.key);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_access_token() {
        let (service, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };

        let created = service.create(token).await.unwrap();
        let update = AccessTokenUpdatePayload {
            key: Some("new-key".to_string()),
            expires_at: Some(Utc::now() + Duration::hours(2)),
            enabled: Some(false),
        };

        let updated = service.update(created.id.unwrap(), update).await.unwrap();
        assert_eq!(updated.key, "new-key");
        assert!(!updated.enabled);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_access_token() {
        let (service, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };

        let created = service.create(token).await.unwrap();
        let deleted = service.delete(created.id.unwrap()).await.unwrap();
        assert!(deleted);

        let read = service.get_access_token(created.id.unwrap()).await.unwrap();
        assert!(read.is_none());

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_access_tokens_by_enabled_status() {
        let (service, db) = setup().await;
        let token1 = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key-1".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };
        let token2 = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key-2".to_string(),
            algorithm: Algorithm::HMAC,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: false,
        };

        let created1 = service.create(token1).await.unwrap();
        service.create(token2).await.unwrap();

        // Debug print: print all documents in the collection
        let collection = db.collection::<mongodb::bson::Document>("access_tokens");
        let all_docs: Vec<_> = collection
            .find(doc! {})
            .await
            .unwrap()
            .try_collect()
            .await
            .unwrap();
        println!("All documents in access_tokens collection: {:#?}", all_docs);

        let filter = AccessTokenFilter {
            key: None,
            algorithm: None,
            is_enabled: Some(true),
            is_active: None,
        };

        let found = service.find(filter).await.unwrap();
        dbg!(&found);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, created1.id);
        assert!(found[0].enabled);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_access_tokens_by_algorithm() {
        let (service, db) = setup().await;
        let token1 = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key-1".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };
        let token2 = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key-2".to_string(),
            algorithm: Algorithm::HMAC,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };

        service.create(token1).await.unwrap();
        service.create(token2).await.unwrap();

        let filter = AccessTokenFilter {
            key: None,
            algorithm: Some(Algorithm::RSA),
            is_enabled: None,
            is_active: None,
        };

        let found = service.find(filter).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].algorithm, Algorithm::RSA);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_access_tokens_by_key() {
        let (service, db) = setup().await;
        let token1 = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key-1".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };
        let token2 = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key-2".to_string(),
            algorithm: Algorithm::HMAC,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };

        service.create(token1).await.unwrap();
        service.create(token2).await.unwrap();

        let filter = AccessTokenFilter {
            key: Some("test-key-1".to_string()),
            algorithm: None,
            is_enabled: None,
            is_active: None,
        };

        let found = service.find(filter).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].key, "test-key-1");

        cleanup_test_db(db).await.unwrap();
    }
}
