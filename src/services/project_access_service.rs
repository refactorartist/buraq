use crate::models::project_access::{
    ProjectAccess, ProjectAccessFilter, ProjectAccessUpdatePayload,
};
use crate::repositories::base::Repository;
use crate::repositories::project_access_repository::ProjectAccessRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;

pub struct ProjectAccessService {
    project_access_repository: ProjectAccessRepository,
}

impl ProjectAccessService {
    pub fn new(database: Database) -> Result<Self, Error> {
        let project_access_repository = ProjectAccessRepository::new(database)?;
        Ok(Self {
            project_access_repository,
        })
    }

    pub async fn create(&self, project_access: ProjectAccess) -> Result<ProjectAccess, Error> {
        self.project_access_repository.create(project_access).await
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<ProjectAccess>, Error> {
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

    pub async fn find(&self, filter: ProjectAccessFilter) -> Result<Vec<ProjectAccess>, Error> {
        self.project_access_repository.find(filter.into()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};

    async fn setup() -> (ProjectAccessService, Database) {
        let db = setup_test_db("project_access_service").await.unwrap();
        let service = ProjectAccessService::new(db.clone()).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create() {
        let (service, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        let created = service.create(project_access.clone()).await.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.name, project_access.name);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get() {
        let (service, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        let created = service.create(project_access.clone()).await.unwrap();
        let retrieved = service.get(created.id.unwrap()).await.unwrap().unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, created.name);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_update() {
        let (service, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        let created = service.create(project_access).await.unwrap();
        let update = ProjectAccessUpdatePayload {
            name: Some("Updated Access".to_string()),
            project_scopes: None,
        };

        let updated = service.update(created.id.unwrap(), update).await.unwrap();
        assert_eq!(updated.name, "Updated Access");

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete() {
        let (service, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        let created = service.create(project_access).await.unwrap();
        let deleted = service.delete(created.id.unwrap()).await.unwrap();
        assert!(deleted);

        let read = service.get(created.id.unwrap()).await.unwrap();
        assert!(read.is_none());

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_by_environment() {
        let (service, db) = setup().await;
        let env_id = Uuid::new();
        let project_access1 = ProjectAccess {
            id: None,
            name: "Access 1".to_string(),
            environment_id: env_id,
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };
        let project_access2 = ProjectAccess {
            id: None,
            name: "Access 2".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        service.create(project_access1).await.unwrap();
        service.create(project_access2).await.unwrap();

        let filter = ProjectAccessFilter {
            environment_id: Some(env_id),
            project_scopes: None,
            service_account_id: None,
        };

        let found = service.find(filter).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].environment_id, env_id);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_by_service_account() {
        let (service, db) = setup().await;
        let service_account_id = Uuid::new();
        let project_access1 = ProjectAccess {
            id: None,
            name: "Access 1".to_string(),
            environment_id: Uuid::new(),
            service_account_id,
            project_scopes: vec![Uuid::new()],
        };
        let project_access2 = ProjectAccess {
            id: None,
            name: "Access 2".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        service.create(project_access1).await.unwrap();
        service.create(project_access2).await.unwrap();

        let filter = ProjectAccessFilter {
            environment_id: None,
            project_scopes: None,
            service_account_id: Some(service_account_id),
        };

        let found = service.find(filter).await.unwrap();
        dbg!(&found);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].service_account_id, service_account_id);

        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_by_project_scopes() {
        let (service, db) = setup().await;
        let scope_id = Uuid::new();
        let project_access1 = ProjectAccess {
            id: None,
            name: "Access 1".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![scope_id],
        };
        let project_access2 = ProjectAccess {
            id: None,
            name: "Access 2".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        service.create(project_access1).await.unwrap();
        service.create(project_access2).await.unwrap();

        let filter = ProjectAccessFilter {
            environment_id: None,
            project_scopes: Some(vec![scope_id]),
            service_account_id: None,
        };

        let found = service.find(filter).await.unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].project_scopes.contains(&scope_id));

        cleanup_test_db(db).await.unwrap();
    }
}
