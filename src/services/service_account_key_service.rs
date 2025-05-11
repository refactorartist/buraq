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
    use crate::test_utils::cleanup_test_db;
    use crate::types::Algorithm;
    use anyhow::Ok;
    use chrono::Utc;

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
}
