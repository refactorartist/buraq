use crate::models::project::Project;
use crate::repositories::base::Repository;
use anyhow::Result;
use mongodb::{Collection, Database};

/// Repository for managing Project documents in MongoDB.
///
/// Provides CRUD operations for Project entities.
///
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
    /// Returns a Result containing the ProjectRepository or an error if collection creation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use buraq::repositories::project::ProjectRepository;
    /// use mongodb::Client;
    /// use buraq::utils::database::create_database_client;
    /// use dotenvy::dotenv;
    /// use buraq::config::AppConfig;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     dotenv().ok();
    ///
    ///     let app_config = AppConfig::from_env(Some(true))?;
    ///     let client = create_database_client(&app_config.application.database_uri).await?;
    ///     let db = client.database("test_db");
    ///     let repo = ProjectRepository::new(db)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<Project>("projects");
        Ok(Self { collection })
    }
}

impl Repository<Project> for ProjectRepository {
    /// Gets the MongoDB collection for Projects.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Collection or an error if cloning fails.
    fn collection(&self) -> Result<Collection<Project>, anyhow::Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use mongodb::bson::{Bson, doc};
    use tokio;

    #[tokio::test]
    async fn test_create_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = repo.create(project).await?;

        assert!(matches!(result.inserted_id, Bson::ObjectId(_)));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        // Create a project first
        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = repo.create(project).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        // Read the project
        let read_project = repo.read(id).await?;
        assert!(read_project.is_some());
        let read_project = read_project.unwrap();
        assert_eq!(read_project.name(), "Test Project");
        assert_eq!(read_project.description(), "Test Description");
        assert!(read_project.enabled());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        // Create a project first
        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = repo.create(project).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        // Update the project
        let updated_project = Project::new("Updated Project".to_string(), "Updated Description".to_string());
        let update_result = repo.update(id, updated_project).await?;
        assert_eq!(update_result.modified_count, 1);

        // Read the updated project
        let read_project = repo.read(id).await?;
        assert!(read_project.is_some());
        let read_project = read_project.unwrap();
        assert_eq!(read_project.name(), "Updated Project");
        assert_eq!(read_project.description(), "Updated Description");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        // Create a project first
        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = repo.create(project).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        // Delete the project
        let delete_result = repo.delete(id).await?;
        assert_eq!(delete_result.deleted_count, 1);

        // Try to read the deleted project
        let read_project = repo.read(id).await?;
        assert!(read_project.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_projects() -> Result<()> {
        let db = setup_test_db("project").await?;
        let repo = ProjectRepository::new(db.clone())?;

        // Clean up any existing data
        cleanup_test_db(db.clone()).await?;

        // Create multiple projects
        let project1 = Project::new("Project 1".to_string(), "Description 1".to_string());
        let project2 = Project::new("Project 2".to_string(), "Description 2".to_string());
        let result1 = repo.create(project1).await?;
        let result2 = repo.create(project2).await?;

        // Verify both projects were created
        assert!(result1.inserted_id.as_object_id().is_some());
        assert!(result2.inserted_id.as_object_id().is_some());

        // Find all projects
        let projects = repo.find(doc! {}).await?;
        assert_eq!(projects.len(), 2, "Expected 2 projects, found {}", projects.len());

        // Find projects with specific name
        let projects = repo.find(doc! { "name": "Project 1" }).await?;
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name(), "Project 1");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
