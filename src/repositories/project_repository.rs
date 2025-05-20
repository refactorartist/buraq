use crate::models::project::{Project, ProjectUpdatePayload};
use crate::repositories::base::Repository;
use anyhow::{Error, Result};
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, doc, to_document};
use mongodb::{Collection, Database};

/// Repository for managing Project documents in MongoDB.
///
/// Provides CRUD operations for Project entities.
pub struct ProjectRepository {
    collection: Collection<Project>,
}

impl ProjectRepository {
    /// Creates a new ProjectRepository instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ProjectRepository or an error if initialization fails.
    pub fn new(database: Database) -> Result<Self, Error> {
        let collection = database.collection::<Project>("projects");
        Ok(Self { collection })
    }
}

#[async_trait]
impl Repository<Project> for ProjectRepository {
    type UpdatePayload = ProjectUpdatePayload;

    async fn create(&self, mut item: Project) -> Result<Project, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        self.collection.insert_one(&item).await?;
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<Project>, Error> {
        let result = self.collection.find_one(doc! { "_id": id }).await?;
        Ok(result)
    }

    async fn replace(&self, id: Uuid, mut item: Project) -> Result<Project, Error> {
        if item.id.is_none() || item.id.unwrap() != id {
            item.id = Some(id);
        }
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": to_document(&item)? })
            .await?;
        let updated = self.collection.find_one(doc! { "_id": id }).await?.unwrap();
        Ok(updated)
    }

    async fn update(&self, id: Uuid, payload: Self::UpdatePayload) -> Result<Project, Error> {
        let document = to_document(&payload)?;
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": document })
            .await?;
        let updated = self
            .read(id)
            .await?
            .ok_or_else(|| Error::msg("Project not found"))?;
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(&self, filter: Document) -> Result<Vec<Project>, Error> {
        let result = self.collection.find(filter).await?;
        let items: Vec<Project> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<Project>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};

    #[tokio::test]
    async fn test_create_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let created = repo.create(project).await?;

        assert!(created.id.is_some());
        assert_eq!(created.name, "Test Project");
        assert_eq!(created.description, "Test Description");
        assert!(created.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let created = repo.create(project).await?;

        let read = repo.read(created.id.unwrap()).await?;
        assert!(read.is_some());
        let read = read.unwrap();
        assert_eq!(read.name, "Test Project");
        assert_eq!(read.description, "Test Description");
        assert!(read.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let created = repo.create(project).await?;

        let update = ProjectUpdatePayload {
            name: Some("Updated Project".to_string()),
            description: Some("Updated Description".to_string()),
            enabled: Some(false),
        };

        let updated = repo.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.name, "Updated Project");
        assert_eq!(updated.description, "Updated Description");
        assert!(!updated.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let created = repo.create(project).await?;

        let deleted = repo.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = repo.read(created.id.unwrap()).await?;
        assert!(read.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_projects() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

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

        repo.create(project1).await?;
        repo.create(project2).await?;

        let projects = repo.find(doc! {}).await?;
        assert_eq!(projects.len(), 2);

        let projects = repo.find(doc! { "name": "Project 1" }).await?;
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Project 1");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
