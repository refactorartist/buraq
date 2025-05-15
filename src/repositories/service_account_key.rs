use crate::models::service_account_key::ServiceAccountKey;
use crate::repositories::base::Repository;
use anyhow::Result;
use mongodb::{Collection, Database, IndexModel};
use mongodb::options::IndexOptions;
use mongodb::bson::doc;

/// Repository for managing ServiceAccountKey documents in MongoDB.
///
/// Provides CRUD operations for ServiceAccountKey entities.
pub struct ServiceAccountKeyRepository {
    collection: Collection<ServiceAccountKey>,
}

impl ServiceAccountKeyRepository {
    /// Creates a new ServiceAccountKeyRepository instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ServiceAccountKeyRepository or an error if collection creation fails.
    ///
    pub async fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<ServiceAccountKey>("service_account_key");
        
        // Create unique index on service_account_id
        let options = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { "service_account_id": 1 })
            .options(options)
            .build();
        
        collection.create_index(index).await?;

        Ok(Self { collection })
    }
}

impl Repository<ServiceAccountKey> for ServiceAccountKeyRepository {
    /// Gets the MongoDB collection for ServiceAccountKeys.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Collection or an error if cloning fails.
    fn collection(&self) -> Result<Collection<ServiceAccountKey>, anyhow::Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use crate::types::Algorithm;
    use chrono::Utc;
    use mongodb::bson::{Bson, oid::ObjectId};
    use tokio;

    #[tokio::test]
    async fn test_create_service_account_key() -> Result<()> {
        let db = setup_test_db("service_account_key").await?;
        let repo = ServiceAccountKeyRepository::new(db.clone()).await?;

        let service_account_id = ObjectId::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now();

        let service_account_key = ServiceAccountKey::new(
            service_account_id,
            algorithm.clone(),
            key.clone(),
            expires_at
        );

        let result = repo.create(service_account_key).await?;

        assert!(matches!(result.inserted_id, Bson::ObjectId(_)));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_service_account_key() -> Result<()> {
        let db = setup_test_db("service_account_key").await?;
        let repo = ServiceAccountKeyRepository::new(db.clone()).await?;

        let service_account_id = ObjectId::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now();

        let service_account_key = ServiceAccountKey::new(
            service_account_id,
            algorithm.clone(),
            key.clone(),
            expires_at
        );
        let result = repo.create(service_account_key).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let read_key = repo.read(id).await?;
        assert!(read_key.is_some());
        let read_key = read_key.unwrap();
        assert_eq!(read_key.service_account_id(), &service_account_id);
        assert_eq!(read_key.algorithm(), &algorithm);
        assert_eq!(read_key.key(), key);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_service_account_key() -> Result<()> {
        let db = setup_test_db("service_account_key").await?;
        let repo = ServiceAccountKeyRepository::new(db.clone()).await?;

        let service_account_id = ObjectId::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now();

        let service_account_key = ServiceAccountKey::new(
            service_account_id,
            algorithm.clone(),
            key.clone(),
            expires_at
        );
        let result = repo.create(service_account_key).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let updated_algorithm = Algorithm::HMAC;
        let updated_key = "updated-key-value".to_string();
        let updated_expires_at = Utc::now();

        let updated_service_account_key = ServiceAccountKey::new(
            service_account_id,
            updated_algorithm.clone(),
            updated_key.clone(),
            updated_expires_at
        );
        let update_result = repo.update(id, updated_service_account_key).await?;
        assert_eq!(update_result.modified_count, 1);

        let read_key = repo.read(id).await?.unwrap();
        assert_eq!(read_key.service_account_id(), &service_account_id);
        assert_eq!(read_key.algorithm(), &updated_algorithm);
        assert_eq!(read_key.key(), updated_key);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_service_account_key() -> Result<()> {
        let db = setup_test_db("service_account_key").await?;
        let repo = ServiceAccountKeyRepository::new(db.clone()).await?;

        let service_account_id = ObjectId::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now();

        let service_account_key = ServiceAccountKey::new(
            service_account_id,
            algorithm.clone(),
            key.clone(),
            expires_at
        );
        let result = repo.create(service_account_key).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let delete_result = repo.delete(id).await?;
        assert_eq!(delete_result.deleted_count, 1);

        let read_key = repo.read(id).await?;
        assert!(read_key.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_service_account_keys() -> Result<()> {
        let db = setup_test_db("service_account_key").await?;
        let repo = ServiceAccountKeyRepository::new(db.clone()).await?;

        let service_account_id1 = ObjectId::new();
        let service_account_id2 = ObjectId::new();
        let key1 = ServiceAccountKey::new(
            service_account_id1,
            Algorithm::RSA,
            "key-1".to_string(),
            Utc::now()
        );
        let key2 = ServiceAccountKey::new(
            service_account_id2,
            Algorithm::HMAC,
            "key-2".to_string(),
            Utc::now()
        );
        repo.create(key1).await?;
        repo.create(key2).await?;

        let keys = repo.find(doc! {}).await?;
        assert_eq!(keys.len(), 2);

        let filtered_keys = repo.find(doc! { "key": "key-1" }).await?;
        assert_eq!(filtered_keys.len(), 1);
        assert_eq!(filtered_keys[0].key(), "key-1");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
