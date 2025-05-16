use crate::models::service_account_key::{ServiceAccountKey, ServiceAccountKeyUpdatePayload};
use crate::repositories::base::Repository;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, doc, to_document};
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

    async fn create(&self, mut item: ServiceAccountKey) -> Result<ServiceAccountKey, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        self.collection
            .insert_one(&item)
            .await
            .expect("Failed to create service account key");
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<ServiceAccountKey>, Error> {
        let result = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?;
        Ok(result)
    }

    async fn replace(
        &self,
        id: Uuid,
        mut item: ServiceAccountKey,
    ) -> Result<ServiceAccountKey, Error> {
        if item.id.is_none() || item.id.unwrap() != id {
            item.id = Some(id);
        }
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": to_document(&item)? })
            .await
            .expect("Failed to update service account key");
        let updated = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?
            .unwrap();
        Ok(updated)
    }

    async fn update(
        &self,
        id: Uuid,
        item: Self::UpdatePayload,
    ) -> Result<ServiceAccountKey, Error> {
        let document = to_document(&item)?;
        self.collection
            .update_one(
                mongodb::bson::doc! { "_id": id },
                mongodb::bson::doc! { "$set": document },
            )
            .await?;
        let updated = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?
            .unwrap();
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self
            .collection
            .delete_one(mongodb::bson::doc! { "_id": id })
            .await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(&self, filter: Document) -> Result<Vec<ServiceAccountKey>, Error> {
        let result = self.collection.find(filter).await?;
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
            created_at: now,
            enabled: true,
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
            created_at: Utc::now(),
            enabled: true,
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
            created_at: Utc::now(),
            enabled: true,
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
            created_at: Utc::now(),
            enabled: true,
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
            created_at: Utc::now(),
            enabled: true,
        };
        let key2 = ServiceAccountKey {
            id: Some(Uuid::new()),
            service_account_id: Uuid::new(),
            algorithm: Algorithm::HMAC,
            key: "test-key-2".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };

        repo.create(key1).await.unwrap();
        repo.create(key2).await.unwrap();

        let found = repo.find(doc! { "enabled": true }).await.unwrap();
        assert_eq!(found.len(), 2);

        cleanup_test_db(db).await.unwrap();
    }
}
