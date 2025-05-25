use crate::models::pagination::Pagination;
use crate::models::service_account_key::{
    ServiceAccountKey, ServiceAccountKeyFilter, ServiceAccountKeySortableFields,
    ServiceAccountKeyUpdatePayload,
};
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Bson, doc, to_document};
use mongodb::{Collection, Database};

/// Repository for managing ServiceAccountKey documents in MongoDB.
///
/// Provides CRUD operations for ServiceAccountKey entities.
pub struct ServiceAccountKeyRepository {
    collection: Collection<ServiceAccountKey>,
}

impl ServiceAccountKeyRepository {
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<ServiceAccountKey>("service_account_keys");
        Ok(Self { collection })
    }
}

#[async_trait]
impl Repository<ServiceAccountKey> for ServiceAccountKeyRepository {
    type UpdatePayload = ServiceAccountKeyUpdatePayload;
    type Filter = ServiceAccountKeyFilter;
    type Sort = ServiceAccountKeySortableFields;

    async fn create(&self, mut item: ServiceAccountKey) -> Result<ServiceAccountKey, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        item.created_at = Some(Utc::now());
        item.updated_at = Some(Utc::now());
        self.collection.insert_one(&item).await?;
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<ServiceAccountKey>, Error> {
        let result = self.collection.find_one(doc! { "_id": id }).await?;
        Ok(result)
    }


    async fn update(
        &self,
        id: Uuid,
        payload: Self::UpdatePayload,
    ) -> Result<ServiceAccountKey, Error> {
        let mut document = to_document(&payload)?;
        document.insert("updated_at", Bson::String(Utc::now().to_rfc3339()));

        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": document })
            .await?;
        let updated = self
            .read(id)
            .await?
            .ok_or_else(|| Error::msg("ServiceAccountKey not found"))?;
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(
        &self,
        filter: Self::Filter,
        sort: Option<SortBuilder<Self::Sort>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<ServiceAccountKey>, Error> {
        let filter_doc = filter.into();

        // Create FindOptions
        let mut options = mongodb::options::FindOptions::default();

        if let Some(s) = sort {
            options.sort = Some(s.to_document());
        }

        if let Some(p) = pagination {
            options.skip = Some(((p.page - 1) * p.limit) as u64);
            options.limit = Some(p.limit as i64);
        }

        let result = self
            .collection
            .find(filter_doc)
            .with_options(options)
            .await?;
        let items: Vec<ServiceAccountKey> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<ServiceAccountKey>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use crate::types::Algorithm;
    use chrono::{Duration, Utc};

    async fn setup() -> (ServiceAccountKeyRepository, Database) {
        let db = setup_test_db("service_account_key").await.unwrap();
        let repo =
            ServiceAccountKeyRepository::new(db.clone()).expect("Failed to create repository");
        (repo, db)
    }

    #[tokio::test]
    async fn test_create_service_account_key() {
        let (repo, db) = setup().await;
        let service_account_id = Uuid::new();
        let now = Utc::now();
        let key = ServiceAccountKey {
            id: None,
            service_account_id,
            algorithm: Algorithm::RSA,
            key: "test-key".to_string(),
            expires_at: now + Duration::hours(1),
            enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        };

        let created = repo.create(key.clone()).await.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.service_account_id, service_account_id);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_read_service_account_key() {
        let (repo, db) = setup().await;
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

        let created = repo.create(key.clone()).await.unwrap();
        let read = repo.read(created.id.unwrap()).await.unwrap().unwrap();
        assert_eq!(read.id, created.id);
        assert_eq!(read.key, created.key);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_service_account_key() {
        let (repo, db) = setup().await;
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

        let created = repo.create(key).await.unwrap();
        let update = ServiceAccountKeyUpdatePayload {
            key: Some("new-key".to_string()),
            expires_at: Some(Utc::now() + Duration::hours(2)),
            enabled: Some(false),
        };

        let updated = repo.update(created.id.unwrap(), update).await.unwrap();
        assert_eq!(updated.key, "new-key");
        assert!(!updated.enabled);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_service_account_key() {
        let (repo, db) = setup().await;
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

        let created = repo.create(key).await.unwrap();
        let deleted = repo.delete(created.id.unwrap()).await.unwrap();
        assert!(deleted);

        let read = repo.read(created.id.unwrap()).await.unwrap();
        assert!(read.is_none());

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_service_account_keys() {
        let (repo, db) = setup().await;
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

        repo.create(key1).await.unwrap();
        repo.create(key2).await.unwrap();

        // Test finding all keys
        let filter = ServiceAccountKeyFilter {
            service_account_id: None,
            algorithm: None,
            is_enabled: None,
            is_active: None,
        };
        let all_keys = repo.find(filter, None, None).await.unwrap();
        assert_eq!(all_keys.len(), 2);

        // Test finding by enabled status
        let enabled_filter = ServiceAccountKeyFilter {
            service_account_id: None,
            algorithm: None,
            is_enabled: Some(true),
            is_active: None,
        };
        let enabled_keys = repo.find(enabled_filter, None, None).await.unwrap();
        assert_eq!(enabled_keys.len(), 2);

        cleanup_test_db(db).await.unwrap();
    }
}
