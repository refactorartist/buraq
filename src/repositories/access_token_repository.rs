use crate::models::access_token::{
    AccessToken, AccessTokenFilter, AccessTokenSortableFields, AccessTokenUpdatePayload,
};
use crate::models::pagination::Pagination;
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::IndexModel;
use mongodb::bson::to_document;
use mongodb::bson::uuid::Uuid;
use mongodb::{Collection, Database};

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

    pub async fn ensure_indexes(&self) -> Result<(), Error> {
        let _ = &self.collection.create_index(
            IndexModel::builder()
                .keys(mongodb::bson::doc! { "project_access_id": 1, "algorithm": 1, "expires_at": 1 })
                .build()
        ).await.expect("Failed to create index on project_access_id, algorithm, expires_at");

        let _ = &self
            .collection
            .create_index(
                IndexModel::builder()
                    .keys(
                        mongodb::bson::doc! { "project_access_id": 1, "algorithm": 1, "active": 1 },
                    )
                    .build(),
            )
            .await
            .expect("Failed to create index on project_access_id, algorithm, active");

        let _ = &self
            .collection
            .create_index(
                IndexModel::builder()
                    .keys(mongodb::bson::doc! { "project_access_id": 1, "enabled": 1 })
                    .build(),
            )
            .await
            .expect("Failed to create index on project_access_id, enabled");

        Ok(())
    }
}

#[async_trait]
impl Repository<AccessToken> for AccessTokenRepository {
    type UpdatePayload = AccessTokenUpdatePayload;
    type Filter = AccessTokenFilter;
    type Sort = AccessTokenSortableFields;

    async fn create(&self, mut item: AccessToken) -> Result<AccessToken, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }

        self.collection
            .insert_one(&item)
            .await
            .expect("Failed to create access token");

        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<AccessToken>, Error> {
        let result = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?;
        Ok(result)
    }

    async fn update(&self, id: Uuid, item: Self::UpdatePayload) -> Result<AccessToken, Error> {
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

    async fn find(
        &self,
        filter: Self::Filter,
        sort: Option<SortBuilder<Self::Sort>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<AccessToken>, Error> {
        let filter_doc = filter.into();

        // Create FindOptions
        let mut options = mongodb::options::FindOptions::default();

        if let Some(s) = sort {
            options.sort = Some(s.to_document());
        }

        if let Some(p) = pagination {
            options.skip = Some(p.skip());
            options.limit = Some(p.limit());
        }

        let result = self
            .collection
            .find(filter_doc)
            .with_options(options)
            .await?;
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
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use chrono::{Duration, Utc};
    use jsonwebtoken::Algorithm;

    async fn setup() -> (AccessTokenRepository, Database) {
        let db = setup_test_db("access_token").await.unwrap();
        let repo = AccessTokenRepository::new(db.clone()).expect("Failed to create repository");
        repo.ensure_indexes()
            .await
            .expect("Failed to create indexes");
        (repo, db)
    }

    #[tokio::test]
    async fn test_create_access_token() {
        let (repo, db) = setup().await;
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
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
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
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };

        let created = repo.create(token).await.unwrap();
        let update = AccessTokenUpdatePayload {
            key: Some("new-key".to_string()),
            expires_at: Some(Utc::now() + Duration::hours(2)),
            enabled: Some(false),
            project_access_id: Some(Uuid::new()),
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
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };

        let created = repo.create(token).await.unwrap();
        let deleted = repo.delete(created.id.unwrap()).await.unwrap();
        assert!(deleted);

        let read = repo.read(created.id.unwrap()).await.unwrap();
        assert!(read.is_none());

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_access_tokens_by_key() {
        let (repo, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };

        repo.create(token).await.unwrap();

        let filter = AccessTokenFilter {
            key: Some("test-key".to_string()),
            algorithm: None,
            is_enabled: None,
            is_active: None,
            project_access_id: None,
        };

        let found = repo.find(filter, None, None).await.unwrap();
        assert_eq!(found.len(), 1);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_access_tokens_by_algorithm() {
        let (repo, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };

        repo.create(token).await.unwrap();

        let filter = AccessTokenFilter {
            key: None,
            algorithm: Some(Algorithm::RS256),
            is_enabled: None,
            is_active: None,
            project_access_id: None,
        };

        let found = repo.find(filter, None, None).await.unwrap();
        assert_eq!(found.len(), 1);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_access_tokens_by_is_enabled() {
        let (repo, db) = setup().await;
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };

        repo.create(token).await.unwrap();

        let filter = AccessTokenFilter {
            key: None,
            algorithm: None,
            is_enabled: Some(true),
            is_active: None,
            project_access_id: None,
        };

        let found = repo.find(filter, None, None).await.unwrap();
        assert_eq!(found.len(), 1);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_access_tokens_by_project_access_id() {
        let (repo, db) = setup().await;
        let project_access_id = Uuid::new();
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            enabled: true,
            project_access_id,
        };

        repo.create(token).await.unwrap();

        let filter = AccessTokenFilter {
            key: None,
            algorithm: None,
            is_enabled: None,
            is_active: None,
            project_access_id: Some(project_access_id),
        };

        let found = repo.find(filter, None, None).await.unwrap();
        assert_eq!(found.len(), 1);

        cleanup_test_db(db).await.unwrap();
    }
}
