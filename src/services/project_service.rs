use crate::repositories::project::ProjectRepository;
use crate::models::project::Project;
use crate::repositories::base::Repository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Service for managing projects
///
/// This service provides a higher-level API for project operations,
/// abstracting the repository layer and providing business logic.
pub struct ProjectService {
    project_repository: ProjectRepository,
}

impl ProjectService {
    /// Creates a new ProjectService with the given database connection
    ///
    /// # Arguments
    ///
    /// * `db` - MongoDB database connection
    ///
    /// # Returns
    ///
    /// A new ProjectService instance
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::services::project_service::ProjectService;
    /// use mongodb::{Client, Database};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    /// let db = client.database("test_db");
    /// let project_service = ProjectService::new(db);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(db: Database) -> Self {
        let project_repository = ProjectRepository::new(db).unwrap();
        Self { project_repository }
    }

    /// Creates a new project in the database
    ///
    /// # Arguments
    ///
    /// * `project` - The project to create
    ///
    /// # Returns
    ///
    /// The created project with its assigned ID
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use buraq::services::project_service::ProjectService;
    /// # use buraq::models::project::Project;
    /// # use mongodb::{Client, Database};
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// # let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    /// # let db = client.database("test_db");
    /// let project_service = ProjectService::new(db);
    /// let project = Project::new("My Project".to_string(), "A test project".to_string());
    /// let created_project = project_service.create_project(project).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_project(&self, project: Project) -> Result<Project, Error> {
        let result = self.project_repository.create(project.clone()).await?;
        let id = result.inserted_id.as_object_id().unwrap();
        
        // Fetch the newly created project
        let inserted_project = self.project_repository.read(id).await?
            .ok_or_else(|| Error::msg("Failed to fetch created project"))?;

        Ok(inserted_project)
    }

    /// Retrieves a project by its ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project to retrieve
    ///
    /// # Returns
    ///
    /// An Option containing the project if found, or None if not found
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use buraq::services::project_service::ProjectService;
    /// # use mongodb::{Client, Database, bson::oid::ObjectId};
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// # let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    /// # let db = client.database("test_db");
    /// let project_service = ProjectService::new(db);
    /// let project_id = ObjectId::new();
    /// let project = project_service.get_project(project_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_project(&self, id: ObjectId) -> Result<Option<Project>, Error> {
        self.project_repository.read(id).await
    }

    /// Updates an existing project
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project to update
    /// * `project` - The updated project data
    ///
    /// # Returns
    ///
    /// The updated project
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails or if the project is not found
    ///
    /// # Examples
    ///
    /// ```
    /// # use buraq::services::project_service::ProjectService;
    /// # use buraq::models::project::Project;
    /// # use mongodb::{Client, Database, bson::oid::ObjectId};
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// # let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    /// # let db = client.database("test_db");
    /// let project_service = ProjectService::new(db);
    /// let project_id = ObjectId::new();
    /// let updated_project = Project::new("Updated Name".to_string(), "Updated description".to_string());
    /// let result = project_service.update_project(project_id, updated_project).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_project(&self, id: ObjectId, project: Project) -> Result<Project, Error> {
        let result = self.project_repository.update(id, project).await?;

        if result.modified_count > 0 {
            log::info!("Project updated successfully: {:?}", id);
        } else {
            log::error!("Failed to update project: {:?}", id);
        }

        let updated_project = self.project_repository.read(id).await?
            .ok_or_else(|| Error::msg("Failed to fetch updated project"))?;

        Ok(updated_project)
    }

    /// Deletes a project by its ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project to delete
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the project was successfully deleted
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use buraq::services::project_service::ProjectService;
    /// # use mongodb::{Client, Database, bson::oid::ObjectId};
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// # let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    /// # let db = client.database("test_db");
    /// let project_service = ProjectService::new(db);
    /// let project_id = ObjectId::new();
    /// let was_deleted = project_service.delete_project(project_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_project(&self, id: ObjectId) -> Result<bool, Error> {
        let result = self.project_repository.delete(id).await?;
        Ok(result.deleted_count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::utils::database::create_database_client;
    use dotenvy::dotenv;
    use tokio;    

    async fn setup_test_db() -> Result<Database, Error> {
        dotenv().ok();

        let app_config = AppConfig::from_env(Some(true))?;
        let client = create_database_client(&app_config.application.database_uri).await?;
        let db = client.database("test_db__project_services");  
        Ok(db)
    }

    async fn cleanup_test_db(db: Database) -> Result<(), Error> {
        db.collection::<Project>("projects").drop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_new() {
        // Setup test database
        let db = setup_test_db().await.unwrap();
        
        // Create a new ProjectService instance
        let project_service = ProjectService::new(db.clone());
        
        // Verify the project_service was created successfully
        assert!(project_service.project_repository.collection().unwrap().name() == "projects");
        
        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_project() {
        let db = setup_test_db().await.unwrap();
        let project_service = ProjectService::new(db.clone());

        // Clean up any existing data
        cleanup_test_db(db.clone()).await.unwrap();

        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = project_service.create_project(project).await;

        assert!(result.is_ok(), "Failed to create project: {:?}", result.err());
        let created_project = result.unwrap();
        assert_eq!(created_project.name(), "Test Project");
        assert_eq!(created_project.description(), "Test Description");
        assert!(created_project.enabled());

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_project() {
        let db = setup_test_db().await.unwrap();
        cleanup_test_db(db.clone()).await.unwrap();
        let project_service = ProjectService::new(db.clone());

        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = project_service.create_project(project).await;
        assert!(result.is_ok(), "Failed to create project: {:?}", result.err());
        let created_project = result.unwrap();

        let project = project_service.get_project(created_project.id().unwrap().clone()).await;
        assert!(project.is_ok(), "Failed to get project: {:?}", project.err());
        let project = project.unwrap().expect("Project should exist");
        assert_eq!(project.name(), "Test Project");
        assert_eq!(project.description(), "Test Description");
        assert!(project.enabled());
    }

    #[tokio::test]
    async fn test_update_project() {
        let db = setup_test_db().await.unwrap();
        cleanup_test_db(db.clone()).await.unwrap();
        let project_service = ProjectService::new(db.clone());

        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = project_service.create_project(project).await;
        assert!(result.is_ok(), "Failed to create project: {:?}", result.err());
        let created_project = result.unwrap();

        let updated_project = Project::new("Updated Project".to_string(), "Updated Description".to_string());
        let result = project_service.update_project(created_project.id().unwrap().clone(), updated_project).await;
        assert!(result.is_ok(), "Failed to update project: {:?}", result.err());
        let updated_project = result.unwrap();
        assert_eq!(updated_project.name(), "Updated Project");
        assert_eq!(updated_project.description(), "Updated Description");

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_project() {
        let db = setup_test_db().await.unwrap();
        cleanup_test_db(db.clone()).await.unwrap();
        let project_service = ProjectService::new(db.clone());

        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = project_service.create_project(project).await;
        assert!(result.is_ok(), "Failed to create project: {:?}", result.err());
        let created_project = result.unwrap();

        let result = project_service.delete_project(created_project.id().unwrap().clone()).await;
        assert!(result.is_ok(), "Failed to delete project: {:?}", result.err());
        assert!(result.unwrap(), "Project should be deleted");

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }
}