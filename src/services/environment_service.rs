use crate::models::environment::{
    Environment, EnvironmentFilter, EnvironmentSortableFields, EnvironmentUpdatePayload,
};
use crate::models::pagination::Pagination;
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use crate::repositories::environment_repository::EnvironmentRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;
use std::sync::Arc;

pub struct EnvironmentService {
    environment_repository: EnvironmentRepository,
}

impl EnvironmentService {
    pub fn new(database: Arc<Database>) -> Result<Self, Error> {
        let environment_repository = EnvironmentRepository::new(database.as_ref().clone())?;
        Ok(Self {
            environment_repository,
        })
    }

    pub async fn create(&self, environment: Environment) -> Result<Environment, Error> {
        self.environment_repository.create(environment).await
    }

    pub async fn get_environment(&self, id: Uuid) -> Result<Option<Environment>, Error> {
        self.environment_repository.read(id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        environment: EnvironmentUpdatePayload,
    ) -> Result<Environment, Error> {
        self.environment_repository.update(id, environment).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        self.environment_repository.delete(id).await
    }

    pub async fn find(
        &self,
        filter: EnvironmentFilter,
        sort: Option<SortBuilder<EnvironmentSortableFields>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<Environment>, Error> {
        self.environment_repository
            .find(filter, sort, pagination)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use chrono::Utc;

    async fn setup() -> (EnvironmentService, Database) {
        let db = setup_test_db("environment_service").await.unwrap();
        let service = EnvironmentService::new(Arc::new(db.clone())).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_environment() {
        let (service, db) = setup().await;
        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(environment).await.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.name, "Test Environment");
        assert_eq!(created.description, "Test Description");
        assert!(created.enabled);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_environment() {
        let (service, db) = setup().await;
        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(environment).await.unwrap();
        let retrieved = service
            .get_environment(created.id.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, created.name);
        assert_eq!(retrieved.description, created.description);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_environment() {
        let (service, db) = setup().await;
        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(environment).await.unwrap();
        let update = EnvironmentUpdatePayload {
            name: Some("Updated Environment".to_string()),
            description: Some("Updated Description".to_string()),
            enabled: Some(false),
        };

        let updated = service.update(created.id.unwrap(), update).await.unwrap();
        assert_eq!(updated.name, "Updated Environment");
        assert_eq!(updated.description, "Updated Description");
        assert!(!updated.enabled);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_environment() {
        let (service, db) = setup().await;
        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(environment).await.unwrap();
        let deleted = service.delete(created.id.unwrap()).await.unwrap();
        assert!(deleted);

        let read = service.get_environment(created.id.unwrap()).await.unwrap();
        assert!(read.is_none());

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_environments() {
        let (service, db) = setup().await;
        let project_id = Uuid::new();
        let environment1 = Environment {
            id: None,
            project_id,
            name: "Environment 1".to_string(),
            description: "Description 1".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let environment2 = Environment {
            id: None,
            project_id,
            name: "Environment 2".to_string(),
            description: "Description 2".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        service.create(environment1).await.unwrap();
        service.create(environment2).await.unwrap();

        let filter = EnvironmentFilter {
            project_id: Some(project_id),
            name: None,
            is_enabled: None,
        };

        let found = service.find(filter, None, None).await.unwrap();
        assert_eq!(found.len(), 2);

        let filter = EnvironmentFilter {
            project_id: None,
            name: Some("Environment 1".to_string()),
            is_enabled: None,
        };

        let found = service.find(filter, None, None).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Environment 1");

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_environments_with_pagination() {
        let (service, db) = setup().await;
        let project_id = Uuid::new();

        // Create 5 test environments
        for i in 1..=5 {
            let environment = Environment {
                id: None,
                project_id,
                name: format!("Environment {}", i),
                description: format!("Description {}", i),
                enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            };
            service.create(environment).await.unwrap();
        }

        // Test first page
        let pagination = Pagination {
            page: Some(1),
            limit: Some(2),
        };
        let found = service
            .find(
                EnvironmentFilter {
                    project_id: Some(project_id),
                    name: None,
                    is_enabled: None,
                },
                None,
                Some(pagination),
            )
            .await
            .unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "Environment 1");
        assert_eq!(found[1].name, "Environment 2");

        // Test second page
        let pagination = Pagination {
            page: Some(2),
            limit: Some(2),
        };
        let found = service
            .find(
                EnvironmentFilter {
                    project_id: Some(project_id),
                    name: None,
                    is_enabled: None,
                },
                None,
                Some(pagination),
            )
            .await
            .unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "Environment 3");
        assert_eq!(found[1].name, "Environment 4");

        // Test last page
        let pagination = Pagination {
            page: Some(3),
            limit: Some(2),
        };
        let found = service
            .find(
                EnvironmentFilter {
                    project_id: Some(project_id),
                    name: None,
                    is_enabled: None,
                },
                None,
                Some(pagination),
            )
            .await
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Environment 5");

        cleanup_test_db(db).await.unwrap();
    }
}
