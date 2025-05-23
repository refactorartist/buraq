use crate::models::pagination::Pagination;
use crate::models::project_scope::{
    ProjectScope, ProjectScopeFilter, ProjectScopeSortableFields, ProjectScopeUpdatePayload,
};
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use crate::repositories::project_scope_repository::ProjectScopeRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;
use std::sync::Arc;

pub struct ProjectScopeService {
    project_scope_repository: ProjectScopeRepository,
}

impl ProjectScopeService {
    pub fn new(database: Arc<Database>) -> Result<Self, Error> {
        let project_scope_repository = ProjectScopeRepository::new(database.as_ref().clone())?;
        Ok(Self {
            project_scope_repository,
        })
    }

    pub async fn create(&self, project_scope: ProjectScope) -> Result<ProjectScope, Error> {
        self.project_scope_repository.create(project_scope).await
    }

    pub async fn get_project_scope(&self, id: Uuid) -> Result<Option<ProjectScope>, Error> {
        self.project_scope_repository.read(id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        project_scope: ProjectScopeUpdatePayload,
    ) -> Result<ProjectScope, Error> {
        self.project_scope_repository
            .update(id, project_scope)
            .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        self.project_scope_repository.delete(id).await
    }

    pub async fn find(
        &self,
        filter: ProjectScopeFilter,
        sort: Option<SortBuilder<ProjectScopeSortableFields>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<ProjectScope>, Error> {
        self.project_scope_repository
            .find(filter, sort, pagination)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use chrono::Utc;

    async fn setup() -> (ProjectScopeService, Database) {
        let db = setup_test_db("project_scope_service").await.unwrap();
        let service = ProjectScopeService::new(Arc::new(db.clone())).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_project_scope() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project_id = Uuid::new();
        let scope = ProjectScope {
            id: None,
            project_id,
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(scope.clone()).await?;
        assert!(created.id.is_some());
        assert_eq!(created.project_id, project_id);
        assert_eq!(created.name, "read:users");
        assert!(created.enabled);
        assert!(created.created_at.is_some());
        assert!(created.updated_at.is_some());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project_scope() -> Result<(), Error> {
        let (service, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(scope.clone()).await?;
        let retrieved = service
            .get_project_scope(created.id.unwrap())
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
    async fn test_update_project_scope() -> Result<(), Error> {
        let (service, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(scope).await?;
        let update = ProjectScopeUpdatePayload {
            name: Some("write:users".to_string()),
            description: Some("Allows writing user data".to_string()),
            enabled: Some(false),
        };

        let updated = service.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.name, "write:users");
        assert_eq!(updated.description, "Allows writing user data");
        assert!(!updated.enabled);
        assert!(updated.updated_at.unwrap() > created.updated_at.unwrap());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project_scope() -> Result<(), Error> {
        let (service, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = service.create(scope).await?;
        let deleted = service.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = service.get_project_scope(created.id.unwrap()).await?;
        assert!(read.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_project_scopes() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project_id = Uuid::new();
        let scope1 = ProjectScope {
            id: Some(Uuid::new()),
            project_id,
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let scope2 = ProjectScope {
            id: Some(Uuid::new()),
            project_id,
            name: "write:users".to_string(),
            description: "Allows writing user data".to_string(),
            enabled: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        service.create(scope1).await?;
        service.create(scope2).await?;

        let filter = ProjectScopeFilter {
            project_id: Some(project_id),
            name: Some("read:users".to_string()),
            is_enabled: Some(true),
        };

        let found = service.find(filter, None, None).await?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "read:users");
        assert!(found[0].enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_project_scopes_with_pagination() -> Result<(), Error> {
        let (service, db) = setup().await;
        let project_id = Uuid::new();

        // Create 5 test scopes
        for i in 1..=5 {
            let scope = ProjectScope {
                id: Some(Uuid::new()),
                project_id,
                name: format!("scope:{}", i),
                description: format!("Description {}", i),
                enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            };
            service.create(scope).await?;
        }

        // Test first page
        let pagination = Pagination { page: 1, limit: 2 };
        let found = service
            .find(
                ProjectScopeFilter {
                    project_id: Some(project_id),
                    name: None,
                    is_enabled: None,
                },
                None,
                Some(pagination),
            )
            .await?;
        assert_eq!(found.len(), 2);

        // Test second page
        let pagination = Pagination { page: 2, limit: 2 };
        let found = service
            .find(
                ProjectScopeFilter {
                    project_id: Some(project_id),
                    name: None,
                    is_enabled: None,
                },
                None,
                Some(pagination),
            )
            .await?;
        assert_eq!(found.len(), 2);

        // Test last page
        let pagination = Pagination { page: 3, limit: 2 };
        let found = service
            .find(
                ProjectScopeFilter {
                    project_id: Some(project_id),
                    name: None,
                    is_enabled: None,
                },
                None,
                Some(pagination),
            )
            .await?;
        assert_eq!(found.len(), 1);

        cleanup_test_db(db).await?;
        Ok(())
    }
}
