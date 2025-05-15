use crate::models::access_token::{AccessToken, AccessTokenUpdatePayload};
use crate::repositories::base::Repository;
use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, to_document, doc};
use mongodb::{Collection, Database};
use anyhow::Error;

/// Repository for managing AccessToken documents in MongoDB.
///
/// Provides CRUD operations for AccessToken entities.
pub struct AccessTokenRepository {
    collection: Collection<AccessToken>,
}

impl AccessTokenRepository {
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<AccessToken>("access_tokens");
        Ok(Self { collection })
    }
}

#[async_trait]
impl Repository<AccessToken> for AccessTokenRepository {
    type UpdatePayload = AccessTokenUpdatePayload;

    async fn create(&self, mut item: AccessToken) -> Result<AccessToken, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        self.collection.insert_one(&item).await.expect("Failed to create access token");
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<AccessToken>, Error> {
        let result = self.collection.find_one(mongodb::bson::doc! { "_id": id }).await?;
        Ok(result)
    }
    async fn replace(&self, id: Uuid, mut item: AccessToken) -> Result<AccessToken, Error> {
        if item.id.is_none() || item.id.unwrap() != id {
            item.id = Some(id);
        }
        self.collection
            .update_one(
                doc! { "_id": id }, 
                doc! { "$set": to_document(&item)? }
            )
            .await
            .expect("Failed to update access token");
        let updated = self.collection.find_one(mongodb::bson::doc! { "_id": id }).await?.unwrap();
        Ok(updated)
    }

    async fn update(&self, id: Uuid, item: Self::UpdatePayload) -> Result<AccessToken, Error> {        
        let document = to_document(&item)?;
        self.collection.update_one(mongodb::bson::doc! { "_id": id }, mongodb::bson::doc! { "$set": document }).await?;
        let updated = self.collection.find_one(mongodb::bson::doc! { "_id": id }).await?.unwrap();
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self.collection.delete_one(mongodb::bson::doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(&self, filter: Document) -> Result<Vec<AccessToken>, Error> {
        let result = self.collection.find(filter).await?;
        let items: Vec<AccessToken> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<AccessToken>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, Duration};
    use crate::types::Algorithm;
    use crate::test_utils::{setup_test_db, cleanup_test_db};

    async fn setup() -> (AccessTokenRepository, Database) {
        let db = setup_test_db("access_token").await.unwrap();
        let repo = AccessTokenRepository::new(db.clone()).expect("Failed to create repository");
        (repo, db)
    }

    #[tokio::test]
    async fn test_create_access_token() {
        let (repo, db) = setup().await;
        let now = Utc::now();
        let token = AccessToken {
            id: None,
            key: "test-key".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: now + Duration::hours(1),
            created_at: now,
            enabled: true,
        };

        let created = repo.create(token.clone()).await.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.key, token.key);
        assert_eq!(created.algorithm, token.algorithm);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_read_access_token() {
        let (repo, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };

        let created = repo.create(token.clone()).await.unwrap();
        let read = repo.read(created.id.unwrap()).await.unwrap().unwrap();
        assert_eq!(read.id, created.id);
        assert_eq!(read.key, created.key);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_access_token() {
        let (repo, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };

        let created = repo.create(token).await.unwrap();
        let update = AccessTokenUpdatePayload {
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
    async fn test_delete_access_token() {
        let (repo, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
        };

        let created = repo.create(token).await.unwrap();
        let deleted = repo.delete(created.id.unwrap()).await.unwrap();
        assert!(deleted);

        let read = repo.read(created.id.unwrap()).await.unwrap();
        assert!(read.is_none());

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_access_tokens() {
        let (repo, db) = setup().await;
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

        repo.create(token1).await.unwrap();
        repo.create(token2).await.unwrap();

        let found = repo.find(doc! { "enabled": true }).await.unwrap();
        assert_eq!(found.len(), 2);

        cleanup_test_db(db).await.unwrap();
    }
}
