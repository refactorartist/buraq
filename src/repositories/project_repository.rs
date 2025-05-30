use crate::models::pagination::Pagination;
use crate::models::project::{Project, ProjectFilter, ProjectSortableFields, ProjectUpdatePayload};
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use anyhow::{Error, Result};
use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Bson, doc, to_document};
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
    type Filter = ProjectFilter;
    type Sort = ProjectSortableFields;

    async fn create(&self, mut item: Project) -> Result<Project, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        item.created_at = Some(Utc::now());
        item.updated_at = Some(Utc::now());
        self.collection.insert_one(&item).await?;
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<Project>, Error> {
        let result = self.collection.find_one(doc! { "_id": id }).await?;
        Ok(result)
    }

    async fn update(&self, id: Uuid, payload: Self::UpdatePayload) -> Result<Project, Error> {
        let mut document = to_document(&payload)?;
        document.insert("updated_at", Bson::String(Utc::now().to_rfc3339()));

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

    async fn find(
        &self,
        filter: Self::Filter,
        sort: Option<SortBuilder<Self::Sort>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<Project>, Error> {
        let filter_doc = filter.into();

        // Create FindOptions
        let mut options = mongodb::options::FindOptions::default();

        if let Some(s) = sort {
            options.sort = Some(s.to_document());
        }

        if let Some(p) = pagination {
            options.skip = Some(p.skip());
            options.limit = Some(p.limit());
        }

        let result = self
            .collection
            .find(filter_doc)
            .with_options(options)
            .await?;
        let items: Vec<Project> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<Project>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use chrono::Utc;

    async fn create_test_project(repo: &ProjectRepository) -> Result<Project> {
        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        repo.create(project).await
    }

    #[tokio::test]
    async fn test_create_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        let created = create_test_project(&repo).await?;

        assert!(created.id.is_some());
        assert_eq!(created.name, "Test Project");
        assert_eq!(created.description, "Test Description");
        assert!(created.enabled);
        assert!(created.created_at.is_some());
        assert!(created.updated_at.is_some());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        let created = create_test_project(&repo).await?;
        let read = repo.read(created.id.unwrap()).await?;

        assert!(read.is_some());
        let read = read.unwrap();
        assert_eq!(read.id, created.id);
        assert_eq!(read.name, created.name);
        assert_eq!(read.description, created.description);
        assert_eq!(read.enabled, created.enabled);
        assert_eq!(read.created_at, created.created_at);
        assert_eq!(read.updated_at, created.updated_at);

        // Test reading non-existent project
        let non_existent = repo.read(Uuid::new()).await?;
        assert!(non_existent.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        let created = create_test_project(&repo).await?;
        let project_id = created.id.unwrap();

        // Test partial update
        let update_payload = ProjectUpdatePayload {
            name: Some("Updated Project".to_string()),
            description: None,
            enabled: Some(false),
        };

        let updated = repo.update(project_id, update_payload).await?;

        // Verify updated fields
        assert_eq!(updated.name, "Updated Project");
        assert_eq!(updated.description, created.description); // Should remain unchanged
        assert!(!updated.enabled);
        assert!(updated.updated_at.unwrap() > created.updated_at.unwrap());

        // Test updating non-existent project
        let non_existent_update = repo
            .update(
                Uuid::new(),
                ProjectUpdatePayload {
                    name: Some("Test".to_string()),
                    description: None,
                    enabled: None,
                },
            )
            .await;
        assert!(non_existent_update.is_err());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        let created = create_test_project(&repo).await?;
        let project_id = created.id.unwrap();

        // Test successful deletion
        let deleted = repo.delete(project_id).await?;
        assert!(deleted);

        // Verify project no longer exists
        let read = repo.read(project_id).await?;
        assert!(read.is_none());

        // Test deleting non-existent project
        let deleted = repo.delete(Uuid::new()).await?;
        assert!(!deleted);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_projects() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        // Create multiple test projects
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
            enabled: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        repo.create(project1).await?;
        repo.create(project2).await?;

        // Test finding all projects
        let filter = ProjectFilter {
            name: None,
            is_enabled: None,
        };
        let all_projects = repo.find(filter, None, None).await?;
        assert_eq!(all_projects.len(), 2);

        // Test finding by name
        let name_filter = ProjectFilter {
            name: Some("Project 1".to_string()),
            is_enabled: None,
        };
        let projects = repo.find(name_filter, None, None).await?;
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Project 1");

        // Test finding by enabled status
        let enabled_filter = ProjectFilter {
            name: None,
            is_enabled: Some(true),
        };
        let enabled_projects = repo.find(enabled_filter, None, None).await?;
        assert_eq!(enabled_projects.len(), 1);
        assert!(enabled_projects[0].enabled);

        let disabled_filter = ProjectFilter {
            name: None,
            is_enabled: Some(false),
        };
        let disabled_projects = repo.find(disabled_filter, None, None).await?;
        assert_eq!(disabled_projects.len(), 1);
        assert!(!disabled_projects[0].enabled);

        // Test finding with non-matching criteria
        let non_matching_filter = ProjectFilter {
            name: Some("Non-existent".to_string()),
            is_enabled: None,
        };
        let non_matching = repo.find(non_matching_filter, None, None).await?;
        assert_eq!(non_matching.len(), 0);

        cleanup_test_db(db).await?;
        Ok(())
    }
}
