use crate::models::project_scope::ProjectScope;
use crate::repositories::base::Repository;
use anyhow::Result;
use mongodb::{Collection, Database};

/// Repository for managing ProjectScope documents in MongoDB.
///
/// Provides CRUD operations for ProjectScope entities.
///
pub struct ProjectScopeRepository {
    collection: Collection<ProjectScope>,
}

impl ProjectScopeRepository {
    /// Creates a new ProjectScopeRepository instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ProjectScopeRepository or an error if collection creation fails.
    ///
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<ProjectScope>("project_scopes");
        Ok(Self { collection })
    }
}

impl Repository<ProjectScope> for ProjectScopeRepository {
    /// Gets the MongoDB collection for ProjectScopes.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Collection or an error if cloning fails.
    fn collection(&self) -> Result<Collection<ProjectScope>, anyhow::Error> {
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
    async fn test_create_project_scope() -> Result<()> {
        let db = setup_test_db("project_scope").await?;
        let repo = ProjectScopeRepository::new(db.clone())?;

        let project_id = mongodb::bson::oid::ObjectId::new();
        let scope = ProjectScope::new(
            project_id,
            "read:users".to_string(),
            "Allows reading user data".to_string()
        );
        let result = repo.create(scope).await?;

        assert!(matches!(result.inserted_id, Bson::ObjectId(_)));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_project_scope() -> Result<()> {
        let db = setup_test_db("project_scope").await?;
        let repo = ProjectScopeRepository::new(db.clone())?;

        let project_id = mongodb::bson::oid::ObjectId::new();
        let scope = ProjectScope::new(
            project_id,
            "read:users".to_string(),
            "Allows reading user data".to_string()
        );
        let result = repo.create(scope).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let read_scope = repo.read(id).await?;
        assert!(read_scope.is_some());
        let read_scope = read_scope.unwrap();
        assert_eq!(read_scope.name(), "read:users");
        assert_eq!(read_scope.description(), "Allows reading user data");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project_scope() -> Result<()> {
        let db = setup_test_db("project_scope").await?;
        let repo = ProjectScopeRepository::new(db.clone())?;

        let project_id = mongodb::bson::oid::ObjectId::new();
        let scope = ProjectScope::new(
            project_id,
            "read:users".to_string(),
            "Allows reading user data".to_string()
        );
        let result = repo.create(scope).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let updated_scope = ProjectScope::new(
            project_id,
            "write:users".to_string(),
            "Allows writing user data".to_string()
        );
        let update_result = repo.update(id, updated_scope).await?;
        assert_eq!(update_result.modified_count, 1);

        let read_scope = repo.read(id).await?.unwrap();
        assert_eq!(read_scope.name(), "write:users");
        assert_eq!(read_scope.description(), "Allows writing user data");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project_scope() -> Result<()> {
        let db = setup_test_db("project_scope").await?;
        let repo = ProjectScopeRepository::new(db.clone())?;

        let project_id = mongodb::bson::oid::ObjectId::new();
        let scope = ProjectScope::new(
            project_id,
            "read:users".to_string(),
            "Allows reading user data".to_string()
        );
        let result = repo.create(scope).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let delete_result = repo.delete(id).await?;
        assert_eq!(delete_result.deleted_count, 1);

        let read_scope = repo.read(id).await?;
        assert!(read_scope.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_project_scopes() -> Result<()> {
        let db = setup_test_db("project_scope").await?;
        let repo = ProjectScopeRepository::new(db.clone())?;

        let project_id = mongodb::bson::oid::ObjectId::new();
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
        repo.create(scope1).await?;
        repo.create(scope2).await?;

        let scopes = repo.find(doc! {}).await?;
        assert_eq!(scopes.len(), 2);

        let filtered_scopes = repo.find(doc! { "name": "read:users" }).await?;
        assert_eq!(filtered_scopes.len(), 1);
        assert_eq!(filtered_scopes[0].name(), "read:users");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
