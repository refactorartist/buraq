use crate::models::pagination::Pagination;
use crate::models::project_access::{
    ProjectAccess, ProjectAccessFilter, ProjectAccessSortableFields, ProjectAccessUpdatePayload,
};
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use crate::repositories::project_access_repository::ProjectAccessRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;
use std::sync::Arc;

pub struct ProjectAccessService {
    project_access_repository: ProjectAccessRepository,
}

impl ProjectAccessService {
    pub fn new(database: Arc<Database>) -> Result<Self, Error> {
        let project_access_repository = ProjectAccessRepository::new(database.as_ref().clone())?;
        Ok(Self {
            project_access_repository,
        })
    }

    pub async fn create(&self, project_access: ProjectAccess) -> Result<ProjectAccess, Error> {
        self.project_access_repository.create(project_access).await
    }

    pub async fn get_project_access(&self, id: Uuid) -> Result<Option<ProjectAccess>, Error> {
        self.project_access_repository.read(id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        project_access: ProjectAccessUpdatePayload,
    ) -> Result<ProjectAccess, Error> {
        self.project_access_repository
            .update(id, project_access)
            .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        self.project_access_repository.delete(id).await
    }

    pub async fn find(
        &self,
        filter: ProjectAccessFilter,
        sort: Option<SortBuilder<ProjectAccessSortableFields>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<ProjectAccess>, Error> {
        self.project_access_repository
            .find(filter, sort, pagination)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use chrono::Utc;

    async fn setup() -> (ProjectAccessService, Database) {
        let db = setup_test_db("project_access_service").await.unwrap();
        let service = ProjectAccessService::new(Arc::new(db.clone())).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_project_access() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(project_access.clone()).await?;
        assert!(created.id.is_some());
        assert_eq!(created.name, project_access.name);
        assert!(created.enabled);
        assert!(created.created_at.is_some());
        assert!(created.updated_at.is_some());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project_access() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(project_access.clone()).await?;
        let retrieved = service
            .get_project_access(created.id.unwrap())
            .await?
            .unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, created.name);
        assert_eq!(retrieved.enabled, created.enabled);
        assert_eq!(retrieved.created_at, created.created_at);
        assert_eq!(retrieved.updated_at, created.updated_at);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project_access() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(project_access).await?;
        let update = ProjectAccessUpdatePayload {
            name: Some("Updated Access".to_string()),
            project_scopes: Some(vec![Uuid::new()]),
            enabled: Some(false),
        };

        let updated = service.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.name, "Updated Access");
        assert!(!updated.enabled);
        assert!(updated.updated_at.unwrap() > created.updated_at.unwrap());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project_access() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(project_access).await?;
        let deleted = service.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = service.get_project_access(created.id.unwrap()).await?;
        assert!(read.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_project_access_with_filter() -> Result<(), Error> {
        let (service, db) = setup().await;
        let env_id = Uuid::new();
        let project_access1 = ProjectAccess {
            id: None,
            name: "Access 1".to_string(),
            environment_id: env_id,
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let project_access2 = ProjectAccess {
            id: None,
            name: "Access 2".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        service.create(project_access1).await?;
        service.create(project_access2).await?;

        let filter = ProjectAccessFilter {
            environment_id: Some(env_id),
            service_account_id: None,
            project_scopes: None,
            is_enabled: None,
        };

        let found = service.find(filter, None, None).await?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].environment_id, env_id);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_project_access_with_pagination() -> Result<(), Error> {
        let (service, db) = setup().await;

        // Create 5 test project accesses
        for i in 1..=5 {
            let project_access = ProjectAccess {
                id: None,
                name: format!("Access {}", i),
                environment_id: Uuid::new(),
                service_account_id: Uuid::new(),
                project_scopes: vec![Uuid::new()],
                enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            };
            service.create(project_access).await?;
        }

        // Test first page
        let pagination = Pagination { page: 1, limit: 2 };
        let found = service
            .find(
                ProjectAccessFilter {
                    environment_id: None,
                    service_account_id: None,
                    project_scopes: None,
                    is_enabled: None,
                },
                None,
                Some(pagination),
            )
            .await?;
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "Access 1");
        assert_eq!(found[1].name, "Access 2");

        // Test second page
        let pagination = Pagination { page: 2, limit: 2 };
        let found = service
            .find(
                ProjectAccessFilter {
                    environment_id: None,
                    service_account_id: None,
                    project_scopes: None,
                    is_enabled: None,
                },
                None,
                Some(pagination),
            )
            .await?;
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "Access 3");
        assert_eq!(found[1].name, "Access 4");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
