use crate::repositories::project_access::ProjectAccessRepository;
use crate::models::project_access::ProjectAccess;
use crate::repositories::base::Repository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Service for managing project access
///
/// This service provides a higher-level API for project access operations,
/// abstracting the repository layer and providing business logic.
pub struct ProjectAccessService {
    project_access_repository: ProjectAccessRepository,
}

/// Filter for querying project access
///
/// This struct is used to filter project access based on specific criteria.
/// It can be used to filter by name, environment_id, service_account_id, or project_scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAccessFilter {
    pub name: Option<String>,
    pub environment_id: Option<ObjectId>,
    pub service_account_id: Option<ObjectId>,
    pub project_scope: Option<ObjectId>,
}

impl From<ProjectAccessFilter> for mongodb::bson::Document {
    fn from(val: ProjectAccessFilter) -> Self {
        let mut doc = mongodb::bson::Document::new();
        
        if let Some(name) = val.name {
            doc.insert("name", name);
        }
        
        if let Some(environment_id) = val.environment_id {
            doc.insert("environment_id", environment_id);
        }
        
        if let Some(service_account_id) = val.service_account_id {
            doc.insert("service_account_id", service_account_id);
        }
        
        if let Some(project_scope) = val.project_scope {
            doc.insert("project_scopes", project_scope);
        }
        
        doc
    }
}

impl ProjectAccessService {
    /// Creates a new ProjectAccessService instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ProjectAccessService or an error if initialization fails.
    pub fn new(database: Database) -> Result<Self, Error> {
        let project_access_repository = ProjectAccessRepository::new(database)?;
        Ok(Self { project_access_repository })
    }

    /// Creates a new project access in the database
    ///
    /// # Arguments
    ///
    /// * `project_access` - The project access to create
    ///
    /// # Returns
    ///
    /// The created project access with its assigned ID
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn create_project_access(&self, project_access: ProjectAccess) -> Result<ProjectAccess, Error> {
        let result = self.project_access_repository.create(project_access.clone()).await?;
        let id = result.inserted_id.as_object_id().unwrap();
        
        // Fetch the newly created project access
        let inserted_project_access = self.project_access_repository.read(id).await?
            .ok_or_else(|| Error::msg("Failed to fetch created project access"))?;

        Ok(inserted_project_access)
    }

    /// Retrieves a project access by its ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project access to retrieve
    ///
    /// # Returns
    ///
    /// An Option containing the project access if found, or None if not found
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn get_project_access(&self, id: ObjectId) -> Result<Option<ProjectAccess>, Error> {
        self.project_access_repository.read(id).await
    }

    /// Updates an existing project access
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project access to update
    /// * `project_access` - The updated project access data
    ///
    /// # Returns
    ///
    /// The updated project access
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails or if the project access is not found
    pub async fn update_project_access(&self, id: ObjectId, project_access: ProjectAccess) -> Result<ProjectAccess, Error> {
        let result = self.project_access_repository.update(id, project_access).await?;

        if result.modified_count > 0 {
            log::info!("Project access updated successfully: {:?}", id);
        } else {
            log::error!("Failed to update project access: {:?}", id);
        }

        let updated_project_access = self.project_access_repository.read(id).await?
            .ok_or_else(|| Error::msg("Failed to fetch updated project access"))?;

        Ok(updated_project_access)
    }

    /// Deletes a project access by its ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project access to delete
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the project access was successfully deleted
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn delete_project_access(&self, id: ObjectId) -> Result<bool, Error> {
        let result = self.project_access_repository.delete(id).await?;
        Ok(result.deleted_count > 0)
    }

    /// Retrieves project access configurations based on the provided filter criteria
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter criteria to apply when retrieving project access
    ///
    /// # Returns
    ///
    /// A vector of project access configurations that match the filter criteria
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn get_project_access_list(&self, filter: ProjectAccessFilter) -> Result<Vec<ProjectAccess>, Error> {
        let filter_doc = filter.into();
        let project_access_list = self.project_access_repository.find(filter_doc).await?;
        Ok(project_access_list)
    }

    /// Adds a project scope to an existing project access
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project access to update
    /// * `scope_id` - The ObjectId of the project scope to add
    ///
    /// # Returns
    ///
    /// The updated project access with the new scope added
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails or if the project access is not found
    pub async fn add_project_scope(&self, id: ObjectId, scope_id: ObjectId) -> Result<ProjectAccess, Error> {
        let mut project_access = self.get_project_access(id).await?
            .ok_or_else(|| Error::msg("Project access not found"))?;
        
        if project_access.add_project_scope(scope_id) {
            self.update_project_access(id, project_access).await
        } else {
            // Scope already exists, just return the current project access
            Ok(project_access)
        }
    }

    /// Removes a project scope from an existing project access
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project access to update
    /// * `scope_id` - The ObjectId of the project scope to remove
    ///
    /// # Returns
    ///
    /// The updated project access with the scope removed
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails or if the project access is not found
    pub async fn remove_project_scope(&self, id: ObjectId, scope_id: &ObjectId) -> Result<ProjectAccess, Error> {
        let mut project_access = self.get_project_access(id).await?
            .ok_or_else(|| Error::msg("Project access not found"))?;
        
        if project_access.remove_project_scope(scope_id) {
            self.update_project_access(id, project_access).await
        } else {
            // Scope didn't exist, just return the current project access
            Ok(project_access)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use tokio;

    async fn setup_project_access_for_filter_tests(service: &ProjectAccessService) -> Result<(), Error> {
        // Clean up any existing data first
        let collection = service.project_access_repository.collection()?;
        let db = collection.client().database(&collection.namespace().db);
        cleanup_test_db(db).await?;

        // Create multiple project access entries for testing filters
        let env_id1 = ObjectId::new();
        let env_id2 = ObjectId::new();
        let service_account_id1 = ObjectId::new();
        let service_account_id2 = ObjectId::new();
        let scope1 = ObjectId::new();
        let scope2 = ObjectId::new();
        let scope3 = ObjectId::new();

        let access1 = ProjectAccess::new(
            "Access 1".to_string(),
            env_id1,
            service_account_id1,
            vec![scope1, scope2],
        );
        
        let access2 = ProjectAccess::new(
            "Access 2".to_string(),
            env_id2,
            service_account_id1,
            vec![scope2, scope3],
        );
        
        let access3 = ProjectAccess::new(
            "Access 3".to_string(),
            env_id1,
            service_account_id2,
            vec![scope1, scope3],
        );

        service.create_project_access(access1).await?;
        service.create_project_access(access2).await?;
        service.create_project_access(access3).await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_create_project_access() -> Result<(), Error> {
        let db = setup_test_db("project_access_service").await?;
        let service = ProjectAccessService::new(db.clone())?;

        let project_access = ProjectAccess::new(
            "Test Access".to_string(),
            ObjectId::new(),
            ObjectId::new(),
            vec![ObjectId::new()],
        );
        
        let result = service.create_project_access(project_access).await?;

        assert!(result.id().is_some());
        assert_eq!(result.name(), "Test Access");
        assert_eq!(result.project_scopes().len(), 1);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project_access() -> Result<(), Error> {
        let db = setup_test_db("project_access_service").await?;
        let service = ProjectAccessService::new(db.clone())?;

        let project_access = ProjectAccess::new(
            "Test Access".to_string(),
            ObjectId::new(),
            ObjectId::new(),
            vec![ObjectId::new()],
        );
        
        let created = service.create_project_access(project_access).await?;
        let id = created.id().unwrap();

        let retrieved = service.get_project_access(*id).await?;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name(), "Test Access");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project_access() -> Result<(), Error> {
        let db = setup_test_db("project_access_service").await?;
        let service = ProjectAccessService::new(db.clone())?;

        let project_access = ProjectAccess::new(
            "Test Access".to_string(),
            ObjectId::new(),
            ObjectId::new(),
            vec![ObjectId::new()],
        );
        
        let created = service.create_project_access(project_access).await?;
        let id = created.id().unwrap();

        let updated_access = ProjectAccess::new(
            "Updated Access".to_string(),
            *created.environment_id(),
            *created.service_account_id(),
            created.project_scopes().clone(),
        );
        
        let updated = service.update_project_access(*id, updated_access).await?;
        assert_eq!(updated.name(), "Updated Access");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project_access() -> Result<(), Error> {
        let db = setup_test_db("project_access_service").await?;
        let service = ProjectAccessService::new(db.clone())?;

        let project_access = ProjectAccess::new(
            "Test Access".to_string(),
            ObjectId::new(),
            ObjectId::new(),
            vec![ObjectId::new()],
        );
        
        let created = service.create_project_access(project_access).await?;
        let id = created.id().unwrap();

        let deleted = service.delete_project_access(*id).await?;
        assert!(deleted);

        let retrieved = service.get_project_access(*id).await?;
        assert!(retrieved.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project_access_list_no_filter() -> Result<(), Error> {
        let db = setup_test_db("project_access_service").await?;
        let service = ProjectAccessService::new(db.clone())?;

        // Setup test data
        setup_project_access_for_filter_tests(&service).await?;

        // Test with no filter
        let filter = ProjectAccessFilter {
            name: None,
            environment_id: None,
            service_account_id: None,
            project_scope: None,
        };
        
        let access_list = service.get_project_access_list(filter).await?;
        assert_eq!(access_list.len(), 3, "Should have retrieved all 3 project access entries with no filter");
        
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project_access_list_filter_by_name() -> Result<(), Error> {
        let db = setup_test_db("project_access_service").await?;
        let service = ProjectAccessService::new(db.clone())?;

        // Setup test data
        setup_project_access_for_filter_tests(&service).await?;

        // Test filtering by name
        let filter = ProjectAccessFilter {
            name: Some("Access 1".to_string()),
            environment_id: None,
            service_account_id: None,
            project_scope: None,
        };
        
        let access_list = service.get_project_access_list(filter).await?;
        assert_eq!(access_list.len(), 1, "Should have retrieved 1 project access when filtering by name");
        assert_eq!(access_list[0].name(), "Access 1");
        
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project_access_list_filter_by_project_scope() -> Result<(), Error> {
        let db = setup_test_db("project_access_service").await?;
        let service = ProjectAccessService::new(db.clone())?;

        // Create test data with known scope IDs
        let scope1 = ObjectId::new();
        let scope2 = ObjectId::new();
        
        let access1 = ProjectAccess::new(
            "Access 1".to_string(),
            ObjectId::new(),
            ObjectId::new(),
            vec![scope1.clone()],
        );
        
        let access2 = ProjectAccess::new(
            "Access 2".to_string(),
            ObjectId::new(),
            ObjectId::new(),
            vec![scope1.clone(), scope2.clone()],
        );
        
        service.create_project_access(access1).await?;
        service.create_project_access(access2).await?;

        // Test filtering by project scope
        let filter = ProjectAccessFilter {
            name: None,
            environment_id: None,
            service_account_id: None,
            project_scope: Some(scope2),
        };
        
        let access_list = service.get_project_access_list(filter).await?;
        assert_eq!(access_list.len(), 1, "Should have retrieved 1 project access when filtering by scope");
        assert_eq!(access_list[0].name(), "Access 2");
        
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_add_remove_project_scope() -> Result<(), Error> {
        let db = setup_test_db("project_access_service").await?;
        let service = ProjectAccessService::new(db.clone())?;

        let project_access = ProjectAccess::new(
            "Test Access".to_string(),
            ObjectId::new(),
            ObjectId::new(),
            vec![],
        );
        
        let created = service.create_project_access(project_access).await?;
        let id = created.id().unwrap();
        
        // Test adding a scope
        let scope_id = ObjectId::new();
        let updated = service.add_project_scope(*id, scope_id).await?;
        assert_eq!(updated.project_scopes().len(), 1);
        assert!(updated.project_scopes().contains(&scope_id));
        
        // Test adding the same scope again (should be idempotent)
        let updated = service.add_project_scope(*id, scope_id).await?;
        assert_eq!(updated.project_scopes().len(), 1);
        
        // Test removing a scope
        let updated = service.remove_project_scope(*id, &scope_id).await?;
        assert_eq!(updated.project_scopes().len(), 0);
        
        cleanup_test_db(db).await?;
        Ok(())
    }
}


