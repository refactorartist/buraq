use crate::models::environment::{Environment, EnvironmentUpdatePayload, EnvironmentFilter};
use crate::repositories::environment_repository::EnvironmentRepository;
use crate::repositories::base::Repository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;

pub struct EnvironmentService {
    environment_repository: EnvironmentRepository,
}

impl EnvironmentService {
    pub fn new(database: Database) -> Result<Self, Error> {
        let environment_repository = EnvironmentRepository::new(database)?;
        Ok(Self { environment_repository })
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

    pub async fn find(&self, filter: EnvironmentFilter) -> Result<Vec<Environment>, Error> {
        self.environment_repository.find(filter.into()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_db, cleanup_test_db};

    async fn setup() -> (EnvironmentService, Database) {
        let db = setup_test_db("environment_service").await.unwrap();
        let service = EnvironmentService::new(db.clone()).unwrap();
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
        };

        let created = service.create(environment).await.unwrap();
        let retrieved = service.get_environment(created.id.unwrap()).await.unwrap().unwrap();
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
        };

        let environment2 = Environment {
            id: None,
            project_id,
            name: "Environment 2".to_string(),
            description: "Description 2".to_string(),
            enabled: true,
        };

        service.create(environment1).await.unwrap();
        service.create(environment2).await.unwrap();

        let filter = EnvironmentFilter {
            project_id: Some(project_id),
            name: None,
            is_enabled: None,
        };

        let found = service.find(filter).await.unwrap();
        assert_eq!(found.len(), 2);

        let filter = EnvironmentFilter {
            project_id: None,
            name: Some("Environment 1".to_string()),
            is_enabled: None,
        };

        let found = service.find(filter).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Environment 1");

        cleanup_test_db(db).await.unwrap();
    }
}
