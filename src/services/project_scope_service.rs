use crate::models::project_scope::{ProjectScope, ProjectScopeFilter, ProjectScopeUpdatePayload};
use crate::repositories::project_scope_repository::ProjectScopeRepository;
use crate::repositories::base::Repository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;

pub struct ProjectScopeService {
    project_scope_repository: ProjectScopeRepository,
}

impl ProjectScopeService {
    pub fn new(database: Database) -> Result<Self, Error> {
        let project_scope_repository = ProjectScopeRepository::new(database)?;
        Ok(Self {
            project_scope_repository,
        })
    }

    pub async fn create(
        &self,
        project_scope: ProjectScope,
    ) -> Result<ProjectScope, Error> {
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
        self.project_scope_repository.update(id, project_scope).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        self.project_scope_repository.delete(id).await
    }

    pub async fn find(
        &self,
        filter: ProjectScopeFilter,
    ) -> Result<Vec<ProjectScope>, Error> {
        self.project_scope_repository.find(filter.into()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_db, cleanup_test_db};

    async fn setup() -> (ProjectScopeService, Database) {
        let db = setup_test_db("project_scope_service").await.unwrap();
        let service = ProjectScopeService::new(db.clone()).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_project_scope() {
        let (service, db) = setup().await;
        let project_id = Uuid::new();
        let scope = ProjectScope {
            id: None,
            project_id,
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
        };

        let created = service.create(scope.clone()).await.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.project_id, project_id);
        assert_eq!(created.name, "read:users");

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_project_scope() {
        let (service, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
        };

        let created = service.create(scope.clone()).await.unwrap();
        let retrieved = service.get_project_scope(created.id.unwrap()).await.unwrap().unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, created.name);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_project_scope() {
        let (service, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
        };

        let created = service.create(scope).await.unwrap();
        let update = ProjectScopeUpdatePayload {
            name: Some("write:users".to_string()),
            description: Some("Allows writing user data".to_string()),
        };

        let updated = service.update(created.id.unwrap(), update).await.unwrap();
        assert_eq!(updated.name, "write:users");
        assert_eq!(updated.description, "Allows writing user data");

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_project_scope() {
        let (service, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
        };

        let created = service.create(scope).await.unwrap();
        let deleted = service.delete(created.id.unwrap()).await.unwrap();
        assert!(deleted);

        let read = service.get_project_scope(created.id.unwrap()).await.unwrap();
        assert!(read.is_none());

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_project_scopes() {
        let (service, db) = setup().await;
        let project_id = Uuid::new();
        let scope1 = ProjectScope {
            id: Some(Uuid::new()),
            project_id,
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
        };
        let scope2 = ProjectScope {
            id: Some(Uuid::new()),
            project_id,
            name: "write:users".to_string(),
            description: "Allows writing user data".to_string(),
        };

        service.create(scope1).await.unwrap();
        service.create(scope2).await.unwrap();

        let filter = ProjectScopeFilter {
            project_id: Some(project_id),
            name: Some("read:users".to_string()),
        };

        let found = service.find(filter).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "read:users");

        cleanup_test_db(db).await.unwrap();
    }
}
