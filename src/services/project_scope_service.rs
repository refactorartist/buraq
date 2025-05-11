use crate::repositories::project_scope::ProjectScopeRepository;
use crate::models::project_scope::ProjectScope;
use crate::repositories::base::Repository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Filter for querying project scopes
///
/// This struct is used to filter project scopes based on specific criteria.
/// It can be used to filter scopes by project ID, name, or description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScopeFilter {
    pub project_id: Option<ObjectId>,
    pub name: Option<String>,
    pub description: Option<String>,
}

impl From<ProjectScopeFilter> for mongodb::bson::Document {
    fn from(val: ProjectScopeFilter) -> Self {
        let mut doc = mongodb::bson::Document::new();
        
        if let Some(project_id) = val.project_id {
            doc.insert("project_id", project_id);
        }
        
        if let Some(name) = val.name {
            doc.insert("name", name);
        }
        
        if let Some(description) = val.description {
            doc.insert("description", description);
        }
        
        doc
    }
}

/// Service for managing project scopes
///
/// This service provides a higher-level API for project scope operations,
/// abstracting the repository layer and providing business logic.
pub struct ProjectScopeService {
    project_scope_repository: ProjectScopeRepository,
}

impl ProjectScopeService {
    /// Creates a new ProjectScopeService instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ProjectScopeService or an error if initialization fails.
    pub fn new(database: Database) -> Result<Self, Error> {
        let project_scope_repository = ProjectScopeRepository::new(database)?;
        Ok(Self { project_scope_repository })
    }

    /// Creates a new project scope in the database
    ///
    /// # Arguments
    ///
    /// * `project_scope` - The project scope to create
    ///
    /// # Returns
    ///
    /// The created project scope with its assigned ID
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn create_project_scope(&self, project_scope: ProjectScope) -> Result<ProjectScope, Error> {
        let result = self.project_scope_repository.create(project_scope.clone()).await?;
        let id = result.inserted_id.as_object_id().unwrap();
        
        // Fetch the newly created project scope
        let inserted_scope = self.project_scope_repository.read(id).await?
            .ok_or_else(|| Error::msg("Failed to fetch created project scope"))?;

        Ok(inserted_scope)
    }

    /// Retrieves a project scope by its ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project scope to retrieve
    ///
    /// # Returns
    ///
    /// An Option containing the project scope if found, or None if not found
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn get_project_scope(&self, id: ObjectId) -> Result<Option<ProjectScope>, Error> {
        self.project_scope_repository.read(id).await
    }

    /// Updates an existing project scope
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project scope to update
    /// * `project_scope` - The updated project scope data
    ///
    /// # Returns
    ///
    /// The updated project scope
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails or if the project scope is not found
    pub async fn update_project_scope(&self, id: ObjectId, project_scope: ProjectScope) -> Result<ProjectScope, Error> {
        let result = self.project_scope_repository.update(id, project_scope).await?;

        if result.modified_count > 0 {
            log::info!("Project scope updated successfully: {:?}", id);
        } else {
            log::error!("Failed to update project scope: {:?}", id);
        }

        let updated_scope = self.project_scope_repository.read(id).await?
            .ok_or_else(|| Error::msg("Failed to fetch updated project scope"))?;

        Ok(updated_scope)
    }

    /// Deletes a project scope by its ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ObjectId of the project scope to delete
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the project scope was successfully deleted
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn delete_project_scope(&self, id: ObjectId) -> Result<bool, Error> {
        let result = self.project_scope_repository.delete(id).await?;
        Ok(result.deleted_count > 0)
    }

    /// Retrieves project scopes based on the provided filter criteria
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter criteria to apply when retrieving project scopes
    ///
    /// # Returns
    ///
    /// A vector of project scopes that match the filter criteria
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn get_project_scopes(&self, filter: ProjectScopeFilter) -> Result<Vec<ProjectScope>, Error> {
        let filter_doc = filter.into();
        let scopes = self.project_scope_repository.find(filter_doc).await?;
        Ok(scopes)
    }

    /// Retrieves all project scopes for a specific project
    ///
    /// # Arguments
    ///
    /// * `project_id` - The ObjectId of the project to retrieve scopes for
    ///
    /// # Returns
    ///
    /// A vector of project scopes associated with the specified project
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn get_scopes_by_project(&self, project_id: ObjectId) -> Result<Vec<ProjectScope>, Error> {
        let filter = ProjectScopeFilter {
            project_id: Some(project_id),
            name: None,
            description: None,
        };
        self.get_project_scopes(filter).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use tokio;

    #[tokio::test]
    async fn test_create_project_scope() -> Result<(), Error> {
        let db = setup_test_db("project_scope_service").await?;
        let service = ProjectScopeService::new(db.clone())?;

        let project_id = ObjectId::new();
        let scope = ProjectScope::new(
            project_id,
            "read:users".to_string(),
            "Allows reading user data".to_string()
        );
        
        let result = service.create_project_scope(scope).await?;
        assert!(result.id().is_some());
        assert_eq!(result.name(), "read:users");
        assert_eq!(result.description(), "Allows reading user data");
        assert_eq!(result.project_id(), &project_id);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project_scope() -> Result<(), Error> {
        let db = setup_test_db("project_scope_service").await?;
        let service = ProjectScopeService::new(db.clone())?;

        let project_id = ObjectId::new();
        let scope = ProjectScope::new(
            project_id,
            "read:users".to_string(),
            "Allows reading user data".to_string()
        );
        
        let created = service.create_project_scope(scope).await?;
        let retrieved = service.get_project_scope(*created.id().unwrap()).await?;
        
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name(), "read:users");
        assert_eq!(retrieved.description(), "Allows reading user data");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project_scope() -> Result<(), Error> {
        let db = setup_test_db("project_scope_service").await?;
        let service = ProjectScopeService::new(db.clone())?;

        let project_id = ObjectId::new();
        let scope = ProjectScope::new(
            project_id,
            "read:users".to_string(),
            "Allows reading user data".to_string()
        );
        
        let created = service.create_project_scope(scope).await?;
        
        let updated_scope = ProjectScope::new(
            project_id,
            "write:users".to_string(),
            "Allows writing user data".to_string()
        );
        
        let updated = service.update_project_scope(*created.id().unwrap(), updated_scope).await?;
        assert_eq!(updated.name(), "write:users");
        assert_eq!(updated.description(), "Allows writing user data");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project_scope() -> Result<(), Error> {
        let db = setup_test_db("project_scope_service").await?;
        let service = ProjectScopeService::new(db.clone())?;

        let project_id = ObjectId::new();
        let scope = ProjectScope::new(
            project_id,
            "read:users".to_string(),
            "Allows reading user data".to_string()
        );
        
        let created = service.create_project_scope(scope).await?;
        let deleted = service.delete_project_scope(*created.id().unwrap()).await?;
        
        assert!(deleted);
        let retrieved = service.get_project_scope(*created.id().unwrap()).await?;
        assert!(retrieved.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_scopes_by_project() -> Result<(), Error> {
        let db = setup_test_db("project_scope_service").await?;
        let service = ProjectScopeService::new(db.clone())?;

        let project_id1 = ObjectId::new();
        let project_id2 = ObjectId::new();
        
        // Create scopes for project 1
        let scope1 = ProjectScope::new(
            project_id1,
            "read:users".to_string(),
            "Allows reading user data".to_string()
        );
        
        let scope2 = ProjectScope::new(
            project_id1,
            "write:users".to_string(),
            "Allows writing user data".to_string()
        );
        
        // Create scope for project 2
        let scope3 = ProjectScope::new(
            project_id2,
            "delete:users".to_string(),
            "Allows deleting user data".to_string()
        );
        
        service.create_project_scope(scope1).await?;
        service.create_project_scope(scope2).await?;
        service.create_project_scope(scope3).await?;
        
        // Test filtering by project_id
        let scopes1 = service.get_scopes_by_project(project_id1).await?;
        assert_eq!(scopes1.len(), 2);
        
        let scopes2 = service.get_scopes_by_project(project_id2).await?;
        assert_eq!(scopes2.len(), 1);
        assert_eq!(scopes2[0].name(), "delete:users");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project_scopes_with_filter() -> Result<(), Error> {
        let db = setup_test_db("project_scope_service").await?;
        let service = ProjectScopeService::new(db.clone())?;

        let project_id = ObjectId::new();
        
        let scope1 = ProjectScope::new(
            project_id,
            "read:users".to_string(),
            "Allows reading user data".to_string()
        );
        
        let scope2 = ProjectScope::new(
            project_id,
            "write:users".to_string(),
            "Allows writing user data".to_string()
        );
        
        service.create_project_scope(scope1).await?;
        service.create_project_scope(scope2).await?;
        
        // Test filtering by name
        let filter = ProjectScopeFilter {
            project_id: Some(project_id),
            name: Some("read:users".to_string()),
            description: None,
        };
        
        let scopes = service.get_project_scopes(filter).await?;
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].name(), "read:users");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
