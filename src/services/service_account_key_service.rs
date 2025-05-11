use crate::models::service_account_key::ServiceAccountKey;
use crate::repositories::base::Repository;
use crate::repositories::service_account_key::ServiceAccountKeyRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

pub struct ServiceAccountKeyService {
    service_account_key_repository: ServiceAccountKeyRepository,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceAccountKeyFilter {
    algorithm: Option<String>,
    key: Option<String>,
    enabled: Option<String>,
}

impl From<ServiceAccountKeyFilter> for mongodb::bson::Document {
    fn from(val: ServiceAccountKeyFilter) -> Self {
        let mut doc = mongodb::bson::Document::new();
        if let Some(algorithm) = val.algorithm {
            doc.insert("algorithm", algorithm);
        }
        if let Some(key) = val.key {
            doc.insert("key", key);
        }
        if let Some(enabled) = val.enabled {
            doc.insert("enabled", enabled);
        }
        doc
    }
}

impl ServiceAccountKeyService {
    pub async fn new(database: Database) -> Result<Self, Error> {
        let service_account_key_repository: ServiceAccountKeyRepository =
            ServiceAccountKeyRepository::new(database).await?;
        Ok(Self {
            service_account_key_repository,
        })
    }
    pub async fn create_service_account_key(
        &self,
        service_account_key: ServiceAccountKey,
    ) -> Result<ServiceAccountKey, Error> {
        let result = self
            .service_account_key_repository
            .create(service_account_key.clone())
            .await?;

        let id = result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| anyhow::anyhow!("Invalid ObjectId"))?;

        // Fetch the newly created service account key
        let inserted_service_account_key = self
            .service_account_key_repository
            .read(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch created service account key"))?;

        Ok(inserted_service_account_key)
    }
    pub async fn get_service_account_key(
        &self,
        id: ObjectId,
    ) -> Result<Option<ServiceAccountKey>, Error> {
        self.service_account_key_repository.read(id).await
    }
    pub async fn update_service_account_key(
        &self,
        id: ObjectId,
        service_account_key: ServiceAccountKey,
    ) -> Result<ServiceAccountKey, Error> {
        let result = self
            .service_account_key_repository
            .update(id, service_account_key)
            .await?;
        if result.modified_count > 0 {
            log::info!("Service account key updated successfully: {:?}", id);
        } else {
            log::error!("Failed to update service account key: {:?}", id);
        }
        let updated_service_account_key = self
            .service_account_key_repository
            .read(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch updated service account key"))?;
        Ok(updated_service_account_key)
    }

    pub async fn delete_service_account_key(&self, id: ObjectId) -> Result<bool, Error> {
        let result = self.service_account_key_repository.delete(id).await?;
        Ok(result.deleted_count > 0)
    }
    pub async fn get_service_account_keys(
        &self,
        filter: ServiceAccountKeyFilter,
    ) -> Result<Vec<ServiceAccountKey>, Error> {
        let filter_doc = filter.into();
        let service_account_keys = self.service_account_key_repository.find(filter_doc).await?;
        Ok(service_account_keys)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use crate::types::Algorithm;
    use anyhow::Ok;
    use chrono::Utc;
    use mongodb::bson::oid::ObjectId;
    async fn setup_service_account_keys_for_filter_tests(
        service_account_key_service: &ServiceAccountKeyService,
    ) -> Result<(), Error> {
        // Clean up any existing data first
        let collection = service_account_key_service
            .service_account_key_repository
            .collection()?;
        let db = collection.client().database(&collection.namespace().db);
        cleanup_test_db(db).await?;
        // Create multiple service account keys for testing filters
        let service_account_key1 = ServiceAccountKey::new(
            ObjectId::new(),
            Algorithm::RSA,
            "key".to_string(),
            Utc::now() + chrono::Duration::days(7),
        );
        let service_account_key2 = ServiceAccountKey::new(
            ObjectId::new(),
            Algorithm::HMAC,
            "key1".to_string(),
            Utc::now() + chrono::Duration::days(3),
        );
        let service_account_key3 = ServiceAccountKey::new(
            ObjectId::new(),
            Algorithm::RSA,
            "key3".to_string(),
            Utc::now() + chrono::Duration::days(2),
        );
        service_account_key_service
            .create_service_account_key(service_account_key1)
            .await?;
        service_account_key_service
            .create_service_account_key(service_account_key2)
            .await?;
        service_account_key_service
            .create_service_account_key(service_account_key3)
            .await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_create_service_account_key() -> Result<(), Error> {
        let db = setup_test_db("service_account_key_service").await?;
        let service_account_key_service = ServiceAccountKeyService::new(db.clone()).await?;
        let service_account_key = ServiceAccountKey::new(
            ObjectId::new(),
            Algorithm::RSA,
            "test_key".to_string(),
            Utc::now() + chrono::Duration::days(7),
        );
        let result = service_account_key_service
            .create_service_account_key(service_account_key)
            .await?;
        assert!(result.id().is_some());
        assert_eq!(result.algorithm(), Algorithm::RSA);
        assert_eq!(result.key(), "test_key");
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_new() -> Result<(), Error> {
        let db = setup_test_db("service_account_key_service").await?;
        let service_account_key_service = ServiceAccountKeyService::new(db.clone()).await?;
        assert!(
            service_account_key_service
                .service_account_key_repository
                .collection()
                .is_ok()
        );
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_service_account_key() -> Result<(), Error> {
        let db = setup_test_db("service_account_key_service").await?;
        let service_account_key_service = ServiceAccountKeyService::new(db.clone()).await?;
        let service_account_key = ServiceAccountKey::new(
            ObjectId::new(),
            Algorithm::RSA,
            "test_key".to_string(),
            Utc::now() + chrono::Duration::days(7),
        );
        let result = service_account_key_service
            .create_service_account_key(service_account_key)
            .await?;
        assert!(result.id().is_some());
        let service_account_key = service_account_key_service
            .get_service_account_key(*result.id().unwrap())
            .await?;
        assert!(service_account_key.is_some());
        let service_account_key = service_account_key.unwrap();
        assert_eq!(service_account_key.algorithm(), Algorithm::RSA);
        assert_eq!(service_account_key.key(), "test_key");
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_service_account_key() -> Result<(), Error> {
        let db = setup_test_db("service_account_key_service").await?;
        let service_account_key_service = ServiceAccountKeyService::new(db.clone()).await?;
        let service_account_key = ServiceAccountKey::new(
            ObjectId::new(),
            Algorithm::RSA,
            "original_key".to_string(),
            Utc::now() + chrono::Duration::days(7),
        );
        let result = service_account_key_service
            .create_service_account_key(service_account_key)
            .await?;
        assert!(result.id().is_some());
        let updated_service_account_key = ServiceAccountKey::new(
            *result.id().unwrap(),
            Algorithm::HMAC,
            "updated_key".to_string(),
            Utc::now() + chrono::Duration::days(14),
        );
        let result = service_account_key_service
            .update_service_account_key(*result.id().unwrap(), updated_service_account_key)
            .await?;
        assert!(result.id().is_some());
        assert_eq!(result.algorithm(), Algorithm::HMAC);
        assert_eq!(result.key(), "updated_key");
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_service_account_key() -> Result<(), Error> {
        let db = setup_test_db("service_account_key_service").await?;
        let service_account_key_service = ServiceAccountKeyService::new(db.clone()).await?;
        let service_account_key = ServiceAccountKey::new(
            ObjectId::new(),
            Algorithm::RSA,
            "test_key".to_string(),
            Utc::now() + chrono::Duration::days(7),
        );
        let result = service_account_key_service
            .create_service_account_key(service_account_key)
            .await?;
        assert!(result.id().is_some());
        let result = service_account_key_service
            .delete_service_account_key(*result.id().unwrap())
            .await?;
        assert!(result);
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_service_account_keys_no_filter() -> Result<(), Error> {
        let db = setup_test_db("service_account_key_service").await?;
        let service_account_key_service = ServiceAccountKeyService::new(db.clone()).await?;
        // Setup test data
        setup_service_account_keys_for_filter_tests(&service_account_key_service).await?;
        // Test with empty filter (should return all service account keys)
        let filter = ServiceAccountKeyFilter {
            algorithm: None,
            key: None,
            enabled: None,
        };
        let service_account_keys = service_account_key_service
            .get_service_account_keys(filter)
            .await?;
        assert_eq!(
            service_account_keys.len(),
            3,
            "Should have retrieved all 3 service account keys with no filter"
        );
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_service_account_keys_filter_by_algorithm() -> Result<(), Error> {
        let db = setup_test_db("service_account_key_service").await?;
        let service_account_key_service = ServiceAccountKeyService::new(db.clone()).await?;
        // Setup test data
        setup_service_account_keys_for_filter_tests(&service_account_key_service).await?;
        // Test filtering by algorithm
        let filter = ServiceAccountKeyFilter {
            algorithm: Some("RSA".to_string()),
            key: None,
            enabled: None,
        };
        let service_account_keys = service_account_key_service
            .get_service_account_keys(filter)
            .await?;
        assert_eq!(
            service_account_keys.len(),
            2,
            "Should have retrieved 2 service account keys when filtering by algorithm"
        );
        assert!(
            service_account_keys
                .iter()
                .all(|key| *key.algorithm() == Algorithm::RSA)
        );
        cleanup_test_db(db).await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_get_service_account_keys_filter_by_key() -> Result<(), Error> {
        let db = setup_test_db("service_account_key_service").await?;
        let service_account_key_service = ServiceAccountKeyService::new(db.clone()).await?;
        // Setup test data
        setup_service_account_keys_for_filter_tests(&service_account_key_service).await?;
        // Test filtering by key
        let filter = ServiceAccountKeyFilter {
            algorithm: None,
            key: Some("key1".to_string()),
            enabled: None,
        };
        let service_account_keys = service_account_key_service
            .get_service_account_keys(filter)
            .await?;
        assert_eq!(
            service_account_keys.len(),
            1,
            "Should have retrieved 1 service account key when filtering by key"
        );
        assert_eq!(service_account_keys[0].key(), "key1");
        cleanup_test_db(db).await?;
        Ok(())
    }
}
