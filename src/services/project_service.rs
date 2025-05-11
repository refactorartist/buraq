use crate::repositories::project::ProjectRepository;
use crate::models::project::Project;
use crate::repositories::base::Repository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::oid::ObjectId;

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

    pub async fn create_project(&self, project: Project) -> Result<Project, Error> {
        let result = self.project_repository.create(project.clone()).await?;
        let id = result.inserted_id.as_object_id().unwrap();
        
        // Fetch the newly created project
        let inserted_project = self.project_repository.read(id).await?
            .ok_or_else(|| Error::msg("Failed to fetch created project"))?;

        Ok(inserted_project)
    }

    pub async fn get_project(&self, id: ObjectId) -> Result<Option<Project>, Error> {
        self.project_repository.read(id).await
    }

    pub async fn update_project(&self, id: ObjectId, project: Project) -> Result<bool, Error> {
        let result = self.project_repository.update(id, project).await?;
        Ok(result.modified_count > 0)
    }

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
        let db = client.database("test_db__projects");  
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
        // TODO: Implement test for update_project
    }

    #[tokio::test]
    async fn test_delete_project() {
        // TODO: Implement test for delete_project
    }
}