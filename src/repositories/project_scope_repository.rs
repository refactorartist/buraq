use crate::models::project_scope::{ProjectScope, ProjectScopeUpdatePayload};
use crate::repositories::base::Repository;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, doc, to_document};
use mongodb::{Collection, Database};

/// Repository for managing ProjectScope documents in MongoDB.
///
/// Provides CRUD operations for ProjectScope entities.
pub struct ProjectScopeRepository {
    collection: Collection<ProjectScope>,
}

impl ProjectScopeRepository {
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<ProjectScope>("project_scopes");
        Ok(Self { collection })
    }
}

#[async_trait]
impl Repository<ProjectScope> for ProjectScopeRepository {
    type UpdatePayload = ProjectScopeUpdatePayload;

    async fn create(&self, mut item: ProjectScope) -> Result<ProjectScope, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        self.collection
            .insert_one(&item)
            .await
            .expect("Failed to create project scope");
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<ProjectScope>, Error> {
        let result = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?;
        Ok(result)
    }

    async fn replace(&self, id: Uuid, mut item: ProjectScope) -> Result<ProjectScope, Error> {
        if item.id.is_none() || item.id.unwrap() != id {
            item.id = Some(id);
        }
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": to_document(&item)? })
            .await
            .expect("Failed to update project scope");
        let updated = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?
            .unwrap();
        Ok(updated)
    }

    async fn update(&self, id: Uuid, item: Self::UpdatePayload) -> Result<ProjectScope, Error> {
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

    async fn find(&self, filter: Document) -> Result<Vec<ProjectScope>, Error> {
        let result = self.collection.find(filter).await?;
        let items: Vec<ProjectScope> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<ProjectScope>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};

    async fn setup() -> (ProjectScopeRepository, Database) {
        let db = setup_test_db("project_scope").await.unwrap();
        let repo = ProjectScopeRepository::new(db.clone()).expect("Failed to create repository");
        (repo, db)
    }

    #[tokio::test]
    async fn test_create_project_scope() -> Result<(), Error> {
        let (repo, db) = setup().await;
        let project_id = Uuid::new();
        let scope = ProjectScope {
            id: None,
            project_id,
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
        };

        let created = repo.create(scope.clone()).await?;
        assert!(created.id.is_some());
        assert_eq!(created.project_id, project_id);
        assert_eq!(created.name, "read:users");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_project_scope() -> Result<(), Error> {
        let (repo, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
        };

        let created = repo.create(scope.clone()).await?;
        let read = repo.read(created.id.unwrap()).await?.unwrap();
        assert_eq!(read.id, created.id);
        assert_eq!(read.name, created.name);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project_scope() -> Result<(), Error> {
        let (repo, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
        };

        let created = repo.create(scope).await?;
        let update = ProjectScopeUpdatePayload {
            name: Some("write:users".to_string()),
            description: Some("Allows writing user data".to_string()),
        };

        let updated = repo.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.name, "write:users");
        assert_eq!(updated.description, "Allows writing user data");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project_scope() -> Result<(), Error> {
        let (repo, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
        };

        let created = repo.create(scope).await?;
        let deleted = repo.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = repo.read(created.id.unwrap()).await?;
        assert!(read.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_project_scopes() -> Result<(), Error> {
        let (repo, db) = setup().await;
        let scope1 = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
        };
        let scope2 = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "write:users".to_string(),
            description: "Allows writing user data".to_string(),
        };

        repo.create(scope1).await?;
        repo.create(scope2).await?;

        let found = repo.find(doc! { "name": "read:users" }).await?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "read:users");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
