use crate::models::pagination::Pagination;
use crate::models::sort::SortBuilder;
use crate::models::server_key::{
    ServerKey, ServerKeyFilter, ServerKeySortableFields, ServerKeyUpdatePayload,
};
use crate::repositories::base::Repository;
use crate::repositories::server_key_repository::ServerKeyRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;
use std::sync::Arc;

pub struct ServerKeyService {
    server_key_repository: ServerKeyRepository,
}

impl ServerKeyService {
    pub fn new(database: Arc<Database>) -> Result<Self, Error> {
        let server_key_repository = ServerKeyRepository::new(database.as_ref().clone())?;
        Ok(Self {
            server_key_repository,
        })
    }

    pub async fn create(&self, server_key: ServerKey) -> Result<ServerKey, Error> {
        self.server_key_repository.create(server_key).await
    }

    pub async fn get_server_key(&self, id: Uuid) -> Result<Option<ServerKey>, Error> {
        self.server_key_repository.read(id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        server_key: ServerKeyUpdatePayload,
    ) -> Result<ServerKey, Error> {
        self.server_key_repository.update(id, server_key).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        self.server_key_repository.delete(id).await
    }

    pub async fn find(
        &self,
        filter: ServerKeyFilter,
        sort: Option<SortBuilder<ServerKeySortableFields>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<ServerKey>, Error> {
        self.server_key_repository.find(filter, sort, pagination).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use jsonwebtoken::Algorithm;
    use chrono::Utc;

    async fn setup() -> (ServerKeyService, Database) {
        let db = setup_test_db("server_key_service").await.unwrap();
        let service = ServerKeyService::new(Arc::new(db.clone())).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_server_key() {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();
        let server_key = ServerKey {
            id: None,
            key: "test_key".to_string(),
            environment_id,
            algorithm: Algorithm::HS256,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = service.create(server_key).await.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.key, "test_key");
        assert_eq!(created.algorithm, Algorithm::HS256);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_server_key() {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();
        let server_key = ServerKey {
            id: None,
            key: "test_key".to_string(),
            environment_id,
            algorithm: Algorithm::HS256,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = service.create(server_key).await.unwrap();
        let retrieved = service.get_server_key(created.id.unwrap()).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().key, "test_key");

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_server_key() {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();
        let server_key = ServerKey {
            id: None,
            key: "test_key".to_string(),
            environment_id,
            algorithm: Algorithm::HS256,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = service.create(server_key).await.unwrap();
        let update_payload = ServerKeyUpdatePayload {
            key: Some("updated_key".to_string()),
            environment_id: Some(environment_id),
            algorithm: Some(Algorithm::HS512),
        };

        let updated = service.update(created.id.unwrap(), update_payload).await.unwrap();
        assert_eq!(updated.key, "updated_key");
        assert_eq!(updated.algorithm, Algorithm::HS512);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_server_key() {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();
        let server_key = ServerKey {
            id: None,
            key: "test_key".to_string(),
            environment_id,
            algorithm: Algorithm::HS256,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = service.create(server_key).await.unwrap();
        let deleted = service.delete(created.id.unwrap()).await.unwrap();
        assert!(deleted);

        let retrieved = service.get_server_key(created.id.unwrap()).await.unwrap();
        assert!(retrieved.is_none());

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_server_keys() {
        let (service, db) = setup().await;
        let environment_id = Uuid::new();
        
        // Create multiple server keys
        for i in 0..3 {
            let server_key = ServerKey {
                id: None,
                key: format!("test_key_{}", i),
                environment_id,
                algorithm: Algorithm::HS256,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            service.create(server_key).await.unwrap();
        }

        // Find without filter
        let all_keys = service.find(ServerKeyFilter::default(), None, None).await.unwrap();
        assert_eq!(all_keys.len(), 3);

        // Find with filter
        let filter = ServerKeyFilter {
            environment_id: Some(environment_id),
            ..Default::default()
        };
        let filtered_keys = service.find(filter, None, None).await.unwrap();
        assert_eq!(filtered_keys.len(), 3);

        cleanup_test_db(db).await.unwrap();
    }
}
