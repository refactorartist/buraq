use crate::models::access_token::{
    AccessToken, AccessTokenFilter, AccessTokenSortableFields, AccessTokenUpdatePayload,
};
use crate::models::pagination::Pagination;
use crate::models::sort::SortBuilder;
use crate::repositories::access_token_repository::AccessTokenRepository;
use crate::repositories::base::Repository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;
use std::sync::Arc;

pub struct AccessTokenService {
    access_token_repository: AccessTokenRepository,
}

impl AccessTokenService {
    pub fn new(database: Arc<Database>) -> Result<Self, Error> {
        let access_token_repository = AccessTokenRepository::new(database.as_ref().clone())?;
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

    pub async fn find(
        &self,
        filter: AccessTokenFilter,
        sort: Option<SortBuilder<AccessTokenSortableFields>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<AccessToken>, Error> {
        self.access_token_repository
            .find(filter, sort, pagination)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use jsonwebtoken::Algorithm;
    use chrono::{Duration, Utc};

    async fn setup() -> (AccessTokenService, Database) {
        let db = setup_test_db("access_token_service").await.unwrap();
        let service = AccessTokenService::new(Arc::new(db.clone())).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_access_token() -> Result<(), Error> {
        let (service, db) = setup().await;
        let now = Utc::now();
        let token = AccessToken {
            id: None,
            key: "test-key".to_string(),
            algorithm: Algorithm::RS256,
            expires_at: now + Duration::hours(1),
            created_at: now,
            enabled: true,
            project_access_id: Uuid::new(),
        };

        let created = service.create(token.clone()).await?;
        assert!(created.id.is_some());
        assert_eq!(created.key, token.key);
        assert_eq!(created.algorithm, token.algorithm);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_access_token() -> Result<(), Error> {
        let (service, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };

        let created = service.create(token.clone()).await?;
        let retrieved = service
            .get_access_token(created.id.unwrap())
            .await?
            .unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.key, created.key);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_access_token() -> Result<(), Error> {
        let (service, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };

        let created = service.create(token).await?;
        let update = AccessTokenUpdatePayload {
            key: Some("new-key".to_string()),
            expires_at: Some(Utc::now() + Duration::hours(2)),
            enabled: Some(false),
            project_access_id: Some(Uuid::new()),
        };

        let updated = service.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.key, "new-key");
        assert!(!updated.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_access_token() -> Result<(), Error> {
        let (service, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };

        let created = service.create(token).await?;
        let deleted = service.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = service.get_access_token(created.id.unwrap()).await?;
        assert!(read.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_access_tokens() -> Result<(), Error> {
        let (service, db) = setup().await;
        let token1 = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key-1".to_string(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };
        let token2 = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key-2".to_string(),
            algorithm: Algorithm::HS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };

        service.create(token1).await?;
        service.create(token2).await?;

        let filter = AccessTokenFilter {
            key: None,
            algorithm: None,
            is_enabled: Some(true),
            is_active: None,
            project_access_id: None,
        };

        let found = service.find(filter, None, None).await?;
        assert_eq!(found.len(), 2);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_access_tokens_with_pagination() -> Result<(), Error> {
        let (service, db) = setup().await;

        // Create 5 test tokens
        for i in 1..=5 {
            let token = AccessToken {
                id: None,
                key: format!("test-key-{}", i),
                algorithm: Algorithm::RS256,
                expires_at: Utc::now() + Duration::hours(1),
                created_at: Utc::now(),
                enabled: true,
                project_access_id: Uuid::new(),
            };
            service.create(token).await?;
        }

        // Test first page
        let pagination = Pagination {
            page: Some(1),
            limit: Some(2),
        };
        let found = service
            .find(
                AccessTokenFilter {
                    key: None,
                    algorithm: None,
                    is_enabled: None,
                    is_active: None,
                    project_access_id: None,
                },
                None,
                Some(pagination),
            )
            .await?;
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].key, "test-key-1");
        assert_eq!(found[1].key, "test-key-2");

        // Test second page
        let pagination = Pagination {
            page: Some(2),
            limit: Some(2),
        };
        let found = service
            .find(
                AccessTokenFilter {
                    key: None,
                    algorithm: None,
                    is_enabled: None,
                    is_active: None,
                    project_access_id: None,
                },
                None,
                Some(pagination),
            )
            .await?;
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].key, "test-key-3");
        assert_eq!(found[1].key, "test-key-4");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
