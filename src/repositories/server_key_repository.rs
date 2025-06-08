use crate::models::pagination::Pagination;
use crate::models::server_key::{
    ServerKey, ServerKeyFilter, ServerKeySortableFields, ServerKeyUpdatePayload,
};
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use anyhow::{Error, Result};
use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::Uuid;
use mongodb::bson::{Bson, doc, to_document};
use mongodb::options::{FindOptions, IndexOptions};
use mongodb::{Collection, Database, IndexModel};

/// Repository for managing ServerKey documents in MongoDB.
///
/// Provides CRUD operations for ServerKey entities.    
#[derive(Debug)]
pub struct ServerKeyRepository {
    collection: Collection<ServerKey>,
}

impl ServerKeyRepository {
    /// Creates a new ServerKeyRepository instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ServerKeyRepository or an error if initialization fails.
    pub fn new(database: Database) -> Result<Self, Error> {
        let collection = database.collection::<ServerKey>("server_keys");
        Ok(Self { collection })
    }

    pub async fn ensure_indexes(&self) -> Result<(), Error> {
        let _ = &self
            .collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "environment_id": 1, "key": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await
            .expect("Failed to create index on environment_id, key");

        let _ = &self
            .collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "environment_id": 1, "algorithm": 1 })
                    .build(),
            )
            .await
            .expect("Failed to create index on environment_id, algorithm");

        Ok(())
    }
}

#[async_trait]
impl Repository<ServerKey> for ServerKeyRepository {
    type UpdatePayload = ServerKeyUpdatePayload;
    type Filter = ServerKeyFilter;
    type Sort = ServerKeySortableFields;

    async fn create(&self, mut item: ServerKey) -> Result<ServerKey, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        item.created_at = Utc::now();
        item.updated_at = Utc::now();
        self.collection.insert_one(&item).await?;
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<ServerKey>, Error> {
        let result = self.collection.find_one(doc! { "_id": id }).await?;
        Ok(result)
    }

    async fn update(&self, id: Uuid, payload: Self::UpdatePayload) -> Result<ServerKey, Error> {
        let mut document = to_document(&payload)?;
        document.insert("updated_at", Bson::String(Utc::now().to_rfc3339()));

        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": document })
            .await?;
        let updated = self
            .read(id)
            .await?
            .ok_or_else(|| Error::msg("ServerKey not found"))?;
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
    ) -> Result<Vec<ServerKey>, Error> {
        let filter_doc = filter.into();

        // Create FindOptions
        let mut options = FindOptions::default();

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
        let items: Vec<ServerKey> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<ServerKey>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::cleanup_test_db;
    use crate::test_utils::setup_test_db;
    use jsonwebtoken::Algorithm;
    use mongodb::Database;

    async fn setup() -> (ServerKeyRepository, Database) {
        let client = setup_test_db("test_db").await.unwrap();
        let database = client.clone();
        let repository =
            ServerKeyRepository::new(database.clone()).expect("Failed to create repository");
        (repository, database)
    }

    #[tokio::test]
    async fn test_create_server_key() -> Result<()> {
        let (repository, database) = setup().await;
        let environment_id = Uuid::new();
        let server_key = ServerKey {
            id: None,
            key: "test_key".to_string(),
            environment_id,
            algorithm: Algorithm::HS256,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = repository.create(server_key).await?;
        assert!(result.id.is_some());
        assert_eq!(result.key, "test_key");
        assert_eq!(result.algorithm, Algorithm::HS256);

        // Clean up
        cleanup_test_db(database).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_server_key() -> Result<()> {
        let (repository, database) = setup().await;
        let environment_id = Uuid::new();
        let server_key = ServerKey {
            id: None,
            key: "test_key".to_string(),
            environment_id,
            algorithm: Algorithm::HS256,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = repository.create(server_key).await?;
        let read_result = repository.read(created.id.unwrap()).await?;
        assert!(read_result.is_some());
        assert_eq!(read_result.unwrap().key, "test_key");

        // Clean up
        cleanup_test_db(database).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_server_key() -> Result<()> {
        let (repository, database) = setup().await;
        let environment_id = Uuid::new();
        let server_key = ServerKey {
            id: None,
            key: "test_key".to_string(),
            environment_id,
            algorithm: Algorithm::HS256,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = repository.create(server_key).await?;
        let update_payload = ServerKeyUpdatePayload {
            key: Some("updated_key".to_string()),
            environment_id: Some(environment_id),
            algorithm: Some(Algorithm::HS512),
        };

        let updated = repository
            .update(created.id.unwrap(), update_payload)
            .await?;
        assert_eq!(updated.key, "updated_key");
        assert_eq!(updated.algorithm, Algorithm::HS512);

        // Clean up
        cleanup_test_db(database).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_server_key() -> Result<()> {
        let (repository, database) = setup().await;
        let environment_id = Uuid::new();
        let server_key = ServerKey {
            id: None,
            key: "test_key".to_string(),
            environment_id,
            algorithm: Algorithm::HS256,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = repository.create(server_key).await?;
        let deleted = repository.delete(created.id.unwrap()).await?;
        assert!(deleted);

        // Clean up
        cleanup_test_db(database).await?;
        Ok(())
    }
}
