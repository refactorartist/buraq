use crate::models::project::Project;
use crate::repositories::base::Repository;
use crate::repositories::project::ProjectRepository;
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

/// Filter for querying projects
///
/// This struct is used to filter projects based on specific criteria.
/// It can be used to filter projects by name, description, or enabled status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFilter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

impl Into<mongodb::bson::Document> for ProjectFilter {
    fn into(self) -> mongodb::bson::Document {
        let mut doc = mongodb::bson::Document::new();

        if let Some(name) = self.name {
            doc.insert("name", name);
        }

        if let Some(description) = self.description {
            doc.insert("description", description);
        }

        if let Some(enabled) = self.enabled {
            doc.insert("enabled", enabled);
        }

        doc
    }
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
        let inserted_project = self
            .project_repository
            .read(id)
            .await?
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

        let updated_project = self
            .project_repository
            .read(id)
            .await?
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

    /// Retrieves projects based on the provided filter criteria
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter criteria to apply when retrieving projects
    ///
    /// # Returns
    ///
    /// A vector of projects that match the filter criteria
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use buraq::services::project_service::{ProjectService, ProjectFilter};
    /// # use mongodb::{Client, Database};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// # let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    /// # let db = client.database("test_db");
    /// let project_service = ProjectService::new(db);
    ///
    /// // Create a filter to find enabled projects
    /// let filter = ProjectFilter {
    ///     name: None,
    ///     description: None,
    ///     enabled: Some(true),
    /// };
    ///
    /// let projects = project_service.get_projects(filter).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_projects(&self, filter: ProjectFilter) -> Result<Vec<Project>, Error> {
        let filter_doc = filter.into();
        let projects = self.project_repository.find(filter_doc).await?;
        Ok(projects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::utils::database::create_database_client;
    use dotenvy::dotenv;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio;

    static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);

    async fn setup_test_db() -> Result<Database, Error> {
        dotenv().ok();

        let app_config = AppConfig::from_env(Some(true))?;
        let client = create_database_client(&app_config.application.database_uri).await?;

        // Create a unique database name for each test
        let db_num = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let db = client.database(&format!("test_db__project_services_{}", db_num));

        // Ensure the database is clean before starting
        cleanup_test_db(db.clone()).await?;

        Ok(db)
    }

    async fn cleanup_test_db(db: Database) -> Result<(), Error> {
        db.collection::<Project>("projects").drop().await?;
        db.drop().await?;
        Ok(())
    }

    async fn setup_projects_for_filter_tests(
        project_service: &ProjectService,
    ) -> Result<(), Error> {
        // Clean up any existing data first
        let collection = project_service.project_repository.collection()?;
        let db = collection.client().database(&collection.namespace().db);
        cleanup_test_db(db).await?;

        // Create multiple projects for testing filters
        let project1 = Project::new("Project 1".to_string(), "Description 1".to_string());
        let project2 = Project::new("Project 2".to_string(), "Description 2".to_string());
        let project3 = Project::new("Project 3".to_string(), "Description 3".to_string());

        project_service.create_project(project1).await?;
        project_service.create_project(project2).await?;
        project_service.create_project(project3).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_new() {
        // Setup test database
        let db = setup_test_db().await.unwrap();

        // Create a new ProjectService instance
        let project_service = ProjectService::new(db.clone());

        // Verify the project_service was created successfully
        assert!(
            project_service
                .project_repository
                .collection()
                .unwrap()
                .name()
                == "projects"
        );

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_project() {
        let db = setup_test_db().await.unwrap();
        let project_service = ProjectService::new(db.clone());

        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = project_service.create_project(project).await;

        assert!(
            result.is_ok(),
            "Failed to create project: {:?}",
            result.err()
        );
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
        let project_service = ProjectService::new(db.clone());

        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = project_service.create_project(project).await;
        assert!(
            result.is_ok(),
            "Failed to create project: {:?}",
            result.err()
        );
        let created_project = result.unwrap();

        let project = project_service
            .get_project(*created_project.id().unwrap())
            .await;
        assert!(
            project.is_ok(),
            "Failed to get project: {:?}",
            project.err()
        );
        let project = project.unwrap().expect("Project should exist");
        assert_eq!(project.name(), "Test Project");
        assert_eq!(project.description(), "Test Description");
        assert!(project.enabled());

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_project() {
        let db = setup_test_db().await.unwrap();
        let project_service = ProjectService::new(db.clone());

        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = project_service.create_project(project).await;
        assert!(
            result.is_ok(),
            "Failed to create project: {:?}",
            result.err()
        );
        let created_project = result.unwrap();

        let updated_project = Project::new(
            "Updated Project".to_string(),
            "Updated Description".to_string(),
        );
        let result = project_service
            .update_project(*created_project.id().unwrap(), updated_project)
            .await;
        assert!(
            result.is_ok(),
            "Failed to update project: {:?}",
            result.err()
        );
        let updated_project = result.unwrap();
        assert_eq!(updated_project.name(), "Updated Project");
        assert_eq!(updated_project.description(), "Updated Description");

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_project() {
        let db = setup_test_db().await.unwrap();
        let project_service = ProjectService::new(db.clone());

        let project = Project::new("Test Project".to_string(), "Test Description".to_string());
        let result = project_service.create_project(project).await;
        assert!(
            result.is_ok(),
            "Failed to create project: {:?}",
            result.err()
        );
        let created_project = result.unwrap();

        let result = project_service
            .delete_project(*created_project.id().unwrap())
            .await;
        assert!(
            result.is_ok(),
            "Failed to delete project: {:?}",
            result.err()
        );
        assert!(result.unwrap(), "Project should be deleted");

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_projects_no_filter() {
        let db = setup_test_db().await.unwrap();
        let project_service = ProjectService::new(db.clone());

        // Setup test data
        setup_projects_for_filter_tests(&project_service)
            .await
            .unwrap();

        // Test with empty filter (should return all projects)
        let filter = ProjectFilter {
            name: None,
            description: None,
            enabled: None,
        };

        let result = project_service.get_projects(filter).await;
        assert!(result.is_ok(), "Failed to get projects: {:?}", result.err());

        let projects = result.unwrap();
        assert_eq!(
            projects.len(),
            3,
            "Should have retrieved all 3 projects with no filter"
        );

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_projects_filter_by_name() {
        let db = setup_test_db().await.unwrap();
        let project_service = ProjectService::new(db.clone());

        // Setup test data
        setup_projects_for_filter_tests(&project_service)
            .await
            .unwrap();

        // Test filtering by name
        let filter = ProjectFilter {
            name: Some("Project 1".to_string()),
            description: None,
            enabled: None,
        };

        let result = project_service.get_projects(filter).await;
        assert!(result.is_ok(), "Failed to get projects: {:?}", result.err());

        let projects = result.unwrap();
        assert_eq!(
            projects.len(),
            1,
            "Should have retrieved 1 project when filtering by name"
        );
        assert_eq!(projects[0].name(), "Project 1");

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_projects_filter_by_description() {
        let db = setup_test_db().await.unwrap();
        let project_service = ProjectService::new(db.clone());

        // Setup test data
        setup_projects_for_filter_tests(&project_service)
            .await
            .unwrap();

        // Test filtering by description
        let filter = ProjectFilter {
            name: None,
            description: Some("Description 2".to_string()),
            enabled: None,
        };

        let result = project_service.get_projects(filter).await;
        assert!(result.is_ok(), "Failed to get projects: {:?}", result.err());

        let projects = result.unwrap();
        assert_eq!(
            projects.len(),
            1,
            "Should have retrieved 1 project when filtering by description"
        );
        assert_eq!(projects[0].description(), "Description 2");

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_projects_filter_by_enabled() {
        let db = setup_test_db().await.unwrap();
        let project_service = ProjectService::new(db.clone());

        // Setup test data
        setup_projects_for_filter_tests(&project_service)
            .await
            .unwrap();

        // Test filtering by enabled status
        let filter = ProjectFilter {
            name: None,
            description: None,
            enabled: Some(true),
        };

        let result = project_service.get_projects(filter).await;
        assert!(result.is_ok(), "Failed to get projects: {:?}", result.err());

        let projects = result.unwrap();
        assert_eq!(
            projects.len(),
            3,
            "Should have retrieved all 3 projects when filtering by enabled=true"
        );

        // Clean up test database
        cleanup_test_db(db).await.unwrap();
    }
}
