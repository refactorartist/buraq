use crate::repositories::environment::EnvironmentRepository;
use crate::models::environment::Environment;
use crate::repositories::base::Repository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Service for managing environments
///
/// This service provides a higher-level API for environment operations,
/// abstracting the repository layer and providing business logic.
pub struct EnvironmentService {
    environment_repository: EnvironmentRepository,
}

/// Filter for querying environments
///
/// This struct is used to filter environments based on specific criteria.
/// It can be used to filter environments by project_id, name, or enabled status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentFilter {
    pub project_id: Option<ObjectId>,
    pub name: Option<String>,
    pub enabled: Option<bool>,
}

impl From<EnvironmentFilter> for mongodb::bson::Document {
    fn from(val: EnvironmentFilter) -> Self {
        let mut doc = mongodb::bson::Document::new();
        
        if let Some(project_id) = val.project_id {
            doc.insert("project_id", project_id);
        }
        
        if let Some(name) = val.name {
            doc.insert("name", name);
        }
        
        if let Some(enabled) = val.enabled {
            doc.insert("enabled", enabled);
        }
        
        doc
    }
}

impl EnvironmentService {
    /// Creates a new EnvironmentService instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the EnvironmentService or an error if initialization fails.
    pub fn new(database: Database) -> Result<Self, Error> {
        let environment_repository = EnvironmentRepository::new(database)?;
        Ok(Self { environment_repository })
    }

    pub async fn create_environment(&self, environment: Environment) -> Result<Environment, Error> {
        let result = self.environment_repository.create(environment).await?;
        let id = result.inserted_id.as_object_id().unwrap();
        
        // Fetch the newly created environment
        let inserted_environment = self.environment_repository.read(id).await?
            .ok_or_else(|| Error::msg("Failed to fetch created environment"))?;

        Ok(inserted_environment)
    }

    pub async fn get_environment(&self, id: ObjectId) -> Result<Option<Environment>, Error> {
        self.environment_repository.read(id).await
    }

    pub async fn update_environment(&self, id: ObjectId, environment: Environment) -> Result<Environment, Error> {
        let result = self.environment_repository.update(id, environment).await?;

        if result.modified_count > 0 {
            log::info!("Environment updated successfully: {:?}", id);
        } else {
            log::error!("Failed to update environment: {:?}", id);
        }

        let updated_environment = self.environment_repository.read(id).await?
            .ok_or_else(|| Error::msg("Failed to fetch updated environment"))?;

        Ok(updated_environment)
    }

    pub async fn delete_environment(&self, id: ObjectId) -> Result<bool, Error> {
        let result = self.environment_repository.delete(id).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn get_environments(&self, filter: EnvironmentFilter) -> Result<Vec<Environment>, Error> {
        let filter_doc = filter.into();
        let environments = self.environment_repository.find(filter_doc).await?;
        Ok(environments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use tokio;

    async fn setup_environments_for_filter_tests(environment_service: &EnvironmentService) -> Result<(), Error> {
        // Clean up any existing data first
        let collection = environment_service.environment_repository.collection()?;
        let db = collection.client().database(&collection.namespace().db);
        cleanup_test_db(db).await?;

        // Create multiple environments for testing filters
        let project_id = ObjectId::new();
        let environment1 = Environment::new(project_id, "Environment 1".to_string(), "Description 1".to_string());
        let environment2 = Environment::new(project_id, "Environment 2".to_string(), "Description 2".to_string());
        let environment3 = Environment::new(ObjectId::new(), "Environment 3".to_string(), "Description 3".to_string());

        environment_service.create_environment(environment1).await?;
        environment_service.create_environment(environment2).await?;
        environment_service.create_environment(environment3).await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_create_environment() -> Result<(), Error> {
        let db = setup_test_db("environment_service").await?;
        let environment_service = EnvironmentService::new(db.clone())?;

        let project_id = ObjectId::new();
        let environment = Environment::new(project_id, "Test Environment".to_string(), "Test Description".to_string());
        let result = environment_service.create_environment(environment).await?;

        assert!(result.id().is_some());
        assert_eq!(result.name(), "Test Environment");
        assert_eq!(result.description(), "Test Description");
        assert!(result.enabled());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_environment() -> Result<(), Error> {
        let db = setup_test_db("environment_service").await?;
        let environment_service = EnvironmentService::new(db.clone())?;

        let project_id = ObjectId::new();
        let environment = Environment::new(project_id, "Test Environment".to_string(), "Test Description".to_string());
        let result = environment_service.create_environment(environment).await?;
        assert!(result.id().is_some());

        let environment = environment_service.get_environment(*result.id().unwrap()).await?;
        assert!(environment.is_some());
        let environment = environment.unwrap();
        assert_eq!(environment.name(), "Test Environment");
        assert_eq!(environment.description(), "Test Description");
        assert!(environment.enabled());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_environment() -> Result<(), Error> {
        let db = setup_test_db("environment_service").await?;
        let environment_service = EnvironmentService::new(db.clone())?;

        let project_id = ObjectId::new();
        let environment = Environment::new(project_id, "Test Environment".to_string(), "Test Description".to_string());
        let result = environment_service.create_environment(environment).await?;
        assert!(result.id().is_some());

        let updated_environment = Environment::new(project_id, "Updated Environment".to_string(), "Updated Description".to_string());
        let result = environment_service.update_environment(*result.id().unwrap(), updated_environment).await?;
        assert!(result.id().is_some());
        assert_eq!(result.name(), "Updated Environment");
        assert_eq!(result.description(), "Updated Description");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_environment() -> Result<(), Error> {
        let db = setup_test_db("environment_service").await?;
        let environment_service = EnvironmentService::new(db.clone())?;

        let project_id = ObjectId::new();
        let environment = Environment::new(project_id, "Test Environment".to_string(), "Test Description".to_string());
        let result = environment_service.create_environment(environment).await?;
        assert!(result.id().is_some());

        let result = environment_service.delete_environment(*result.id().unwrap()).await?;
        assert!(result);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_environments_no_filter() -> Result<(), Error> {
        let db = setup_test_db("environment_service").await?;
        let environment_service = EnvironmentService::new(db.clone())?;

        // Setup test data
        setup_environments_for_filter_tests(&environment_service).await?;

        // Test with empty filter (should return all environments)
        let filter = EnvironmentFilter {
            project_id: None,
            name: None,
            enabled: None,
        };
        
        let environments = environment_service.get_environments(filter).await?;
        assert_eq!(environments.len(), 3, "Should have retrieved all 3 environments with no filter");
        
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_environments_filter_by_project_id() -> Result<(), Error> {
        let db = setup_test_db("environment_service").await?;
        let environment_service = EnvironmentService::new(db.clone())?;

        // Setup test data
        let project_id = ObjectId::new();
        let environment1 = Environment::new(project_id, "Environment 1".to_string(), "Description 1".to_string());
        let environment2 = Environment::new(project_id, "Environment 2".to_string(), "Description 2".to_string());
        let environment3 = Environment::new(ObjectId::new(), "Environment 3".to_string(), "Description 3".to_string());

        environment_service.create_environment(environment1).await?;
        environment_service.create_environment(environment2).await?;
        environment_service.create_environment(environment3).await?;

        // Test filtering by project_id
        let filter = EnvironmentFilter {
            project_id: Some(project_id),
            name: None,
            enabled: None,
        };
        
        let environments = environment_service.get_environments(filter).await?;
        assert_eq!(environments.len(), 2, "Should have retrieved 2 environments when filtering by project_id");
        
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_environments_filter_by_name() -> Result<(), Error> {
        let db = setup_test_db("environment_service").await?;
        let environment_service = EnvironmentService::new(db.clone())?;

        // Setup test data
        setup_environments_for_filter_tests(&environment_service).await?;

        // Test filtering by name
        let filter = EnvironmentFilter {
            project_id: None,
            name: Some("Environment 1".to_string()),
            enabled: None,
        };
        
        let environments = environment_service.get_environments(filter).await?;
        assert_eq!(environments.len(), 1, "Should have retrieved 1 environment when filtering by name");
        assert_eq!(environments[0].name(), "Environment 1");
        
        cleanup_test_db(db).await?;
        Ok(())
    }
}

