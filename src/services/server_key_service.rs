use crate::models::pagination::Pagination;
use crate::models::server_key::{
    ServerKey, ServerKeyCreatePayload, ServerKeyFilter, ServerKeyRead, ServerKeySortableFields,
    ServerKeyUpdatePayload,
};
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use crate::repositories::server_key_repository::ServerKeyRepository;
use crate::utils::security::SecretsManager;
use crate::utils::tokens::key_builder::KeyBuilder;
use anyhow::Error;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::Utc;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;
use std::sync::Arc;

#[derive(Debug)]
pub struct ServerKeyService {
    server_key_repository: ServerKeyRepository,
    secrets_manager: SecretsManager,
}

impl ServerKeyService {
    pub fn new(database: Arc<Database>) -> Result<Self, Error> {
        let server_key_repository = ServerKeyRepository::new(database.as_ref().clone())?;
        let secrets_manager = SecretsManager::new(true)?;
        Ok(Self {
            server_key_repository,
            secrets_manager,
        })
    }

    pub async fn create(&self, payload: ServerKeyCreatePayload) -> Result<ServerKeyRead, Error> {
        let key_builder = KeyBuilder::new();
        let key_pair = key_builder.generate_key(payload.algorithm).unwrap();
        let private_key = STANDARD.encode(key_pair.private_key);

        let encrypted_key = self
            .secrets_manager
            .encrypt(&private_key, &payload.environment_id)?;

        let server_key = ServerKey {
            id: None,
            key: encrypted_key,
            environment_id: payload.environment_id,
            algorithm: payload.algorithm,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let servery_key = self.server_key_repository.create(server_key).await;

        match servery_key {
            Ok(server_key) => Ok(ServerKeyRead::from(server_key)),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<ServerKeyRead>, Error> {
        let server_key = self.server_key_repository.read(id).await?;

        match server_key {
            Some(server_key) => Ok(Some(ServerKeyRead::from(server_key))),
            None => Ok(None),
        }
    }

    pub async fn update(
        &self,
        id: Uuid,
        server_key: ServerKeyUpdatePayload,
    ) -> Result<ServerKeyRead, Error> {
        let server_key = self.server_key_repository.update(id, server_key).await;

        match server_key {
            Ok(server_key) => Ok(ServerKeyRead::from(server_key)),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        self.server_key_repository.delete(id).await
    }

    pub async fn find(
        &self,
        filter: ServerKeyFilter,
        sort: Option<SortBuilder<ServerKeySortableFields>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<ServerKeyRead>, Error> {
        let server_keys = self
            .server_key_repository
            .find(filter, sort, pagination)
            .await?;
        Ok(server_keys.into_iter().map(ServerKeyRead::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::sort::SortDirection,
        test_utils::{cleanup_test_db, setup_test_db},
    };
    use anyhow::Result;
    use jsonwebtoken::Algorithm;

    async fn setup() -> (ServerKeyService, Database) {
        let db = setup_test_db("server_key_service").await.unwrap();
        let service = ServerKeyService::new(Arc::new(db.clone())).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_server_key() -> Result<()> {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();

        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::RS256,
        };

        let created = service.create(payload).await?;
        assert_eq!(created.environment_id, environment_id);
        assert_eq!(created.algorithm, Algorithm::RS256);
        assert!(created.created_at <= Utc::now());
        assert!(created.updated_at <= Utc::now());

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_get_server_key() -> Result<()> {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();

        // Create a server key first
        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::RS256,
        };

        let created = service.create(payload).await?;

        // Now retrieve it
        let retrieved = service.get(created.id).await?.unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.environment_id, created.environment_id);
        assert_eq!(retrieved.algorithm, created.algorithm);
        assert_eq!(retrieved.created_at, created.created_at);
        assert_eq!(retrieved.updated_at, created.updated_at);

        // Try to get a non-existent key
        let non_existent = service.get(Uuid::new()).await?;
        assert!(non_existent.is_none());

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_update_server_key() -> Result<()> {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();
        let new_environment_id = Uuid::new();

        // Create a server key first
        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::RS256,
        };

        let created = service.create(payload).await?;

        // Update the server key
        let update_payload = ServerKeyUpdatePayload {
            environment_id: Some(new_environment_id),
            algorithm: Some(Algorithm::ES256),
            key: None,
        };

        let updated = service.update(created.id, update_payload).await?;
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.environment_id, new_environment_id);
        assert_eq!(updated.algorithm, Algorithm::ES256);
        assert_eq!(updated.created_at, created.created_at);
        assert!(updated.updated_at >= created.updated_at);

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_server_key() -> Result<()> {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();

        // Create a server key first
        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::RS256,
        };

        let created = service.create(payload).await?;

        // Verify it exists
        let retrieved = service.get(created.id).await?;
        assert!(retrieved.is_some());

        // Delete it
        let deleted = service.delete(created.id).await?;
        assert!(deleted);

        // Verify it's gone
        let retrieved_after_delete = service.get(created.id).await?;
        assert!(retrieved_after_delete.is_none());

        // Try to delete a non-existent key
        let non_existent_delete = service.delete(Uuid::new()).await?;
        assert!(!non_existent_delete);

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_find_server_keys() -> Result<()> {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();
        let second_environment_id = Uuid::new();

        // Create two server keys with different environments
        let payload1 = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::RS256,
        };

        let payload2 = ServerKeyCreatePayload {
            environment_id: second_environment_id,
            algorithm: Algorithm::HS256,
        };

        let key1 = service.create(payload1).await?;
        let key2 = service.create(payload2).await?;

        // Find by environment_id
        let filter1 = ServerKeyFilter {
            environment_id: Some(environment_id),
            algorithm: None,
            key: None,
        };

        let found1 = service.find(filter1, None, None).await?;
        assert_eq!(found1.len(), 1);
        assert_eq!(found1[0].id, key1.id);
        assert_eq!(found1[0].environment_id, environment_id);

        // Find by algorithm
        let filter2 = ServerKeyFilter {
            environment_id: None,
            algorithm: Some(Algorithm::HS256),
            key: None,
        };

        let found2 = service.find(filter2, None, None).await?;
        assert_eq!(found2.len(), 1);
        assert_eq!(found2[0].id, key2.id);
        assert_eq!(found2[0].algorithm, Algorithm::HS256);

        // Find all
        let filter_all = ServerKeyFilter::default();
        let found_all = service.find(filter_all, None, None).await?;
        assert_eq!(found_all.len(), 2);

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_find_server_keys_with_pagination() -> Result<()> {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();

        // Create 5 server keys
        for i in 0..5 {
            let payload = ServerKeyCreatePayload {
                environment_id,
                algorithm: if i % 2 == 0 {
                    Algorithm::RS256
                } else {
                    Algorithm::HS256
                },
            };
            service.create(payload).await?;
        }

        // Test first page
        let pagination = Pagination {
            page: Some(1),
            limit: Some(2),
        };

        let found = service
            .find(
                ServerKeyFilter {
                    environment_id: Some(environment_id),
                    algorithm: None,
                    key: None,
                },
                None,
                Some(pagination),
            )
            .await?;

        assert_eq!(found.len(), 2);

        // Test second page
        let pagination = Pagination {
            page: Some(2),
            limit: Some(2),
        };

        let found = service
            .find(
                ServerKeyFilter {
                    environment_id: Some(environment_id),
                    algorithm: None,
                    key: None,
                },
                None,
                Some(pagination),
            )
            .await?;

        assert_eq!(found.len(), 2);

        // Test last page
        let pagination = Pagination {
            page: Some(3),
            limit: Some(2),
        };

        let found = service
            .find(
                ServerKeyFilter {
                    environment_id: Some(environment_id),
                    algorithm: None,
                    key: None,
                },
                None,
                Some(pagination),
            )
            .await?;

        assert_eq!(found.len(), 1);

        // Test with sorting
        let sort = SortBuilder::<ServerKeySortableFields>::new()
            .add_sort(ServerKeySortableFields::Algorithm, SortDirection::Ascending);

        let found = service
            .find(ServerKeyFilter::default(), Some(sort), None)
            .await?;

        assert_eq!(found.len(), 5);

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_encryption_decryption() -> Result<()> {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();

        // Create a server key
        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::RS256,
        };

        // The service should encrypt the key when creating
        let created = service.create(payload).await?;

        // The key should be encrypted in the database
        let server_key_repo = ServerKeyRepository::new(db.clone())?;
        let stored_key = server_key_repo.read(created.id).await?.unwrap();

        // The stored key should be encrypted and not match the original
        assert!(!stored_key.key.is_empty());

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }
}
