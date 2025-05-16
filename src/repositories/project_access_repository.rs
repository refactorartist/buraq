use crate::models::project_access::{ProjectAccess, ProjectAccessUpdatePayload};
use crate::repositories::base::Repository;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, doc, to_document};
use mongodb::{Collection, Database};

/// Repository for managing ProjectAccess documents in MongoDB.
///
/// Provides CRUD operations for ProjectAccess entities.
pub struct ProjectAccessRepository {
    collection: Collection<ProjectAccess>,
}

impl ProjectAccessRepository {
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<ProjectAccess>("project_access");
        Ok(Self { collection })
    }
}

#[async_trait]
impl Repository<ProjectAccess> for ProjectAccessRepository {
    type UpdatePayload = ProjectAccessUpdatePayload;

    async fn create(&self, mut item: ProjectAccess) -> Result<ProjectAccess, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        self.collection
            .insert_one(&item)
            .await
            .expect("Failed to create project access");
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<ProjectAccess>, Error> {
        let result = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?;
        Ok(result)
    }

    async fn replace(&self, id: Uuid, mut item: ProjectAccess) -> Result<ProjectAccess, Error> {
        if item.id.is_none() || item.id.unwrap() != id {
            item.id = Some(id);
        }
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": to_document(&item)? })
            .await
            .expect("Failed to update project access");
        let updated = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?
            .unwrap();
        Ok(updated)
    }

    async fn update(&self, id: Uuid, item: Self::UpdatePayload) -> Result<ProjectAccess, Error> {
        let document = to_document(&item)?;
        self.collection
            .update_one(
                mongodb::bson::doc! { "_id": id },
                mongodb::bson::doc! { "$set": document },
            )
            .await?;
        let updated = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?
            .unwrap();
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self
            .collection
            .delete_one(mongodb::bson::doc! { "_id": id })
            .await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(&self, filter: Document) -> Result<Vec<ProjectAccess>, Error> {
        let result = self.collection.find(filter).await?;
        let items: Vec<ProjectAccess> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<ProjectAccess>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};

    async fn setup() -> (ProjectAccessRepository, Database) {
        let db = setup_test_db("project_access").await.unwrap();
        let repo = ProjectAccessRepository::new(db.clone()).expect("Failed to create repository");
        (repo, db)
    }

    #[tokio::test]
    async fn test_create_project_access() -> Result<()> {
        let (repo, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        let created = repo.create(project_access.clone()).await.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.name, project_access.name);

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_read_project_access() -> Result<()> {
        let (repo, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        let created = repo.create(project_access.clone()).await.unwrap();
        let read = repo.read(created.id.unwrap()).await.unwrap().unwrap();
        assert_eq!(read.id, created.id);
        assert_eq!(read.name, created.name);

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project_access() -> Result<()> {
        let (repo, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        let created = repo.create(project_access).await.unwrap();
        let update = ProjectAccessUpdatePayload {
            name: Some("Updated Access".to_string()),
            project_scopes: Some(vec![Uuid::new()]),
        };

        let updated = repo.update(created.id.unwrap(), update).await.unwrap();
        assert_eq!(updated.name, "Updated Access");

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project_access() -> Result<()> {
        let (repo, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        let created = repo.create(project_access).await.unwrap();
        let deleted = repo.delete(created.id.unwrap()).await.unwrap();
        assert!(deleted);

        let read = repo.read(created.id.unwrap()).await.unwrap();
        assert!(read.is_none());

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_find_project_access() -> Result<()> {
        let (repo, db) = setup().await;
        let environment_id = Uuid::new();
        let service_account_id = Uuid::new();

        let access1 = ProjectAccess {
            id: None,
            name: "Access 1".to_string(),
            environment_id,
            service_account_id,
            project_scopes: vec![Uuid::new()],
        };
        let access2 = ProjectAccess {
            id: None,
            name: "Access 2".to_string(),
            environment_id,
            service_account_id,
            project_scopes: vec![Uuid::new()],
        };

        repo.create(access1).await.unwrap();
        repo.create(access2).await.unwrap();

        let found = repo
            .find(doc! { "environment_id": environment_id })
            .await
            .unwrap();
        assert_eq!(found.len(), 2);

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }
}
