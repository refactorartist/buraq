use crate::models::pagination::Pagination;
use crate::models::service_account_key::{
    ServiceAccountKey, ServiceAccountKeyFilter, ServiceAccountKeySortableFields, ServiceAccountKeyUpdatePayload,
};
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use crate::repositories::service_account_key_repository::ServiceAccountKeyRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;
use std::sync::Arc;

pub struct ServiceAccountKeyService {
    service_account_key_repository: ServiceAccountKeyRepository,
}

impl ServiceAccountKeyService {
    pub fn new(database: Arc<Database>) -> Result<Self, Error> {
        let service_account_key_repository = ServiceAccountKeyRepository::new(database.as_ref().clone())?;
        Ok(Self {
            service_account_key_repository,
        })
    }

    pub async fn create(
        &self,
        service_account_key: ServiceAccountKey,
    ) -> Result<ServiceAccountKey, Error> {
        self.service_account_key_repository
            .create(service_account_key)
            .await
    }

    pub async fn get_service_account_key(
        &self,
        id: Uuid,
    ) -> Result<Option<ServiceAccountKey>, Error> {
        self.service_account_key_repository.read(id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        service_account_key: ServiceAccountKeyUpdatePayload,
    ) -> Result<ServiceAccountKey, Error> {
        self.service_account_key_repository
            .update(id, service_account_key)
            .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        self.service_account_key_repository.delete(id).await
    }

    pub async fn find(
        &self,
        filter: ServiceAccountKeyFilter,
        sort: Option<SortBuilder<ServiceAccountKeySortableFields>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<ServiceAccountKey>, Error> {
        self.service_account_key_repository
            .find(filter, sort, pagination)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use crate::types::Algorithm;
    use chrono::{Duration, Utc};

    async fn setup() -> (ServiceAccountKeyService, Database) {
        let db = setup_test_db("service_account_key_service").await.unwrap();
        let service = ServiceAccountKeyService::new(Arc::new(db.clone())).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_service_account_key() -> Result<(), Error> {
        let (service, db) = setup().await;
        let now = Utc::now();
        let key = ServiceAccountKey {
            id: None,
            service_account_id: Uuid::new(),
            algorithm: Algorithm::RSA,
            key: "test-key".to_string(),
            expires_at: now + Duration::hours(1),
            enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        };

        let created = service.create(key.clone()).await?;
        assert!(created.id.is_some());
        assert_eq!(created.key, key.key);
        assert_eq!(created.algorithm, key.algorithm);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_service_account_key() -> Result<(), Error> {
        let (service, db) = setup().await;
        let key = ServiceAccountKey {
            id: Some(Uuid::new()),
            service_account_id: Uuid::new(),
            algorithm: Algorithm::RSA,
            key: "test-key".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(key.clone()).await?;
        let retrieved = service
            .get_service_account_key(created.id.unwrap())
            .await?
            .unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.key, created.key);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_service_account_key() -> Result<(), Error> {
        let (service, db) = setup().await;
        let key = ServiceAccountKey {
            id: Some(Uuid::new()),
            service_account_id: Uuid::new(),
            algorithm: Algorithm::RSA,
            key: "test-key".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(key).await?;
        let update = ServiceAccountKeyUpdatePayload {
            key: Some("new-key".to_string()),
            expires_at: Some(Utc::now() + Duration::hours(2)),
            enabled: Some(false),
        };

        let updated = service.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.key, "new-key");
        assert!(!updated.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_service_account_key() -> Result<(), Error> {
        let (service, db) = setup().await;
        let key = ServiceAccountKey {
            id: Some(Uuid::new()),
            service_account_id: Uuid::new(),
            algorithm: Algorithm::RSA,
            key: "test-key".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(key).await?;
        let deleted = service.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = service
            .get_service_account_key(created.id.unwrap())
            .await?;
        assert!(read.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_service_account_keys_by_enabled_status() -> Result<(), Error> {
        let (service, db) = setup().await;
        let key1 = ServiceAccountKey {
            id: Some(Uuid::new()),
            service_account_id: Uuid::new(),
            algorithm: Algorithm::RSA,
            key: "test-key-1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let key2 = ServiceAccountKey {
            id: Some(Uuid::new()),
            service_account_id: Uuid::new(),
            algorithm: Algorithm::HMAC,
            key: "test-key-2".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            enabled: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        service.create(key1.clone()).await?;
        service.create(key2).await?;

        let filter = ServiceAccountKeyFilter {
            service_account_id: None,
            algorithm: None,
            is_enabled: Some(true),
            is_active: None,
        };

        let found = service.find(filter, None, None).await?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, key1.id);
        assert!(found[0].enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_service_account_keys_by_algorithm() -> Result<(), Error> {
        let (service, db) = setup().await;
        let key1 = ServiceAccountKey {
            id: Some(Uuid::new()),
            service_account_id: Uuid::new(),
            algorithm: Algorithm::RSA,
            key: "test-key-1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let key2 = ServiceAccountKey {
            id: Some(Uuid::new()),
            service_account_id: Uuid::new(),
            algorithm: Algorithm::HMAC,
            key: "test-key-2".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        service.create(key1).await?;
        service.create(key2).await?;

        let filter = ServiceAccountKeyFilter {
            service_account_id: None,
            algorithm: Some(Algorithm::RSA),
            is_enabled: None,
            is_active: None,
        };

        let found = service.find(filter, None, None).await?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].algorithm, Algorithm::RSA);

        cleanup_test_db(db).await?;
        Ok(())
    }
}
