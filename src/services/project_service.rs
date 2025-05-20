use crate::models::project::{Project, ProjectFilter, ProjectUpdatePayload};
use crate::repositories::base::Repository;
use crate::repositories::project_repository::ProjectRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;

pub struct ProjectService {
    project_repository: ProjectRepository,
}

impl ProjectService {
    pub fn new(database: Database) -> Result<Self, Error> {
        let project_repository = ProjectRepository::new(database)?;
        Ok(Self { project_repository })
    }

    pub async fn create(&self, project: Project) -> Result<Project, Error> {
        self.project_repository.create(project).await
    }

    pub async fn get_project(&self, id: Uuid) -> Result<Option<Project>, Error> {
        self.project_repository.read(id).await
    }

    pub async fn update(&self, id: Uuid, project: ProjectUpdatePayload) -> Result<Project, Error> {
        self.project_repository.update(id, project).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        self.project_repository.delete(id).await
    }

    pub async fn find(&self, filter: ProjectFilter) -> Result<Vec<Project>, Error> {
        self.project_repository.find(filter.into()).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};

    async fn setup() -> (ProjectService, Database) {
        let db = setup_test_db("project_service").await.unwrap();
        let service = ProjectService::new(db.clone()).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_project() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(project).await?;
        assert!(created.id.is_some());
        assert_eq!(created.name, "Test Project");
        assert_eq!(created.description, "Test Description");
        assert!(created.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let created = service.create(project).await?;
        let retrieved = service.get_project(created.id.unwrap()).await?.unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Test Project");
        assert_eq!(retrieved.description, "Test Description");
        assert!(retrieved.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let created = service.create(project).await?;
        let update = ProjectUpdatePayload {
            name: Some("Updated Project".to_string()),
            description: Some("Updated Description".to_string()),
            enabled: Some(false),
        };

        let updated = service.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.name, "Updated Project");
        assert_eq!(updated.description, "Updated Description");
        assert!(!updated.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let created = service.create(project).await?;
        let deleted = service.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = service.get_project(created.id.unwrap()).await?;
        assert!(read.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_projects() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project1 = Project {
            id: None,
            name: "Project 1".to_string(),
            description: "Description 1".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let project2 = Project {
            id: None,
            name: "Project 2".to_string(),
            description: "Description 2".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        service.create(project1).await?;
        service.create(project2).await?;

        let filter = ProjectFilter {
            name: Some("Project 1".to_string()),
            is_enabled: None,
        };

        let found = service.find(filter).await?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Project 1");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
