use crate::models::project_scope::{ProjectScope, ProjectScopeUpdatePayload, ProjectScopeFilter, ProjectScopeSortableFields};
use crate::models::sort::SortBuilder;
use crate::models::pagination::Pagination;
use crate::repositories::base::Repository;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Bson, doc, to_document};
use mongodb::{Collection, Database};

/// Repository for managing ProjectScope documents in MongoDB.
///
/// Provides CRUD operations for ProjectScope entities.
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
    /// Returns a Result containing the ProjectScopeRepository or an error if initialization fails.
    pub fn new(database: Database) -> Result<Self, Error> {
        let collection = database.collection::<ProjectScope>("project_scopes");
        Ok(Self { collection })
    }
}

#[async_trait]
impl Repository<ProjectScope> for ProjectScopeRepository {
    type UpdatePayload = ProjectScopeUpdatePayload;
    type Filter = ProjectScopeFilter;
    type Sort = ProjectScopeSortableFields;

    async fn create(&self, mut item: ProjectScope) -> Result<ProjectScope, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        item.created_at = Some(Utc::now());
        item.updated_at = Some(Utc::now());
        self.collection.insert_one(&item).await?;
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<ProjectScope>, Error> {
        let result = self.collection.find_one(doc! { "_id": id }).await?;
        Ok(result)
    }

    async fn replace(&self, id: Uuid, mut item: ProjectScope) -> Result<ProjectScope, Error> {
        if item.id.is_none() || item.id.unwrap() != id {
            item.id = Some(id);
        }
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": to_document(&item)? })
            .await?;
        let updated = self.collection.find_one(doc! { "_id": id }).await?.unwrap();
        Ok(updated)
    }

    async fn update(&self, id: Uuid, payload: Self::UpdatePayload) -> Result<ProjectScope, Error> {
        let mut document = to_document(&payload)?;
        document.insert("updated_at", Bson::String(Utc::now().to_rfc3339()));

        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": document })
            .await?;
        let updated = self
            .read(id)
            .await?
            .ok_or_else(|| Error::msg("Project scope not found"))?;
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(&self, filter: Self::Filter, sort: Option<SortBuilder<Self::Sort>>, pagination: Option<Pagination>) -> Result<Vec<ProjectScope>, Error> {
        let filter_doc = filter.into();
        
        // Create FindOptions
        let mut options = mongodb::options::FindOptions::default();
        
        if let Some(s) = sort {
            options.sort = Some(s.to_document());
        }
        
        if let Some(p) = pagination {
            options.skip = Some(((p.page - 1) * p.limit) as u64);
            options.limit = Some(p.limit as i64);
        }
        
        let result = self.collection.find(filter_doc).with_options(options).await?;
        let items: Vec<ProjectScope> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<ProjectScope>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use chrono::Utc;

    async fn setup() -> (ProjectScopeRepository, Database) {
        let db = setup_test_db("project_scope").await.unwrap();
        let repo = ProjectScopeRepository::new(db.clone()).expect("Failed to create repository");
        (repo, db)
    }

    #[tokio::test]
    async fn test_create_project_scope() -> Result<(), Error> {
        let (repo, db) = setup().await;
        let project_id = Uuid::new();
        let scope = ProjectScope {
            id: None,
            project_id,
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = repo.create(scope.clone()).await?;
        assert!(created.id.is_some());
        assert_eq!(created.project_id, project_id);
        assert_eq!(created.name, "read:users");
        assert!(created.enabled);
        assert!(created.created_at.is_some());
        assert!(created.updated_at.is_some());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_project_scope() -> Result<(), Error> {
        let (repo, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = repo.create(scope.clone()).await?;
        let read = repo.read(created.id.unwrap()).await?.unwrap();
        assert_eq!(read.id, created.id);
        assert_eq!(read.name, created.name);
        assert_eq!(read.enabled, created.enabled);
        assert_eq!(read.created_at, created.created_at);
        assert_eq!(read.updated_at, created.updated_at);

        // Test reading non-existent scope
        let non_existent = repo.read(Uuid::new()).await?;
        assert!(non_existent.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project_scope() -> Result<(), Error> {
        let (repo, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = repo.create(scope).await?;
        let update = ProjectScopeUpdatePayload {
            name: Some("write:users".to_string()),
            description: Some("Allows writing user data".to_string()),
            enabled: Some(false),
        };

        let updated = repo.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.name, "write:users");
        assert_eq!(updated.description, "Allows writing user data");
        assert!(!updated.enabled);
        assert!(updated.updated_at.unwrap() > created.updated_at.unwrap());

        // Test updating non-existent scope
        let non_existent_update = repo
            .update(
                Uuid::new(),
                ProjectScopeUpdatePayload {
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
    async fn test_delete_project_scope() -> Result<(), Error> {
        let (repo, db) = setup().await;
        let scope = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = repo.create(scope).await?;
        let deleted = repo.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = repo.read(created.id.unwrap()).await?;
        assert!(read.is_none());

        // Test deleting non-existent scope
        let deleted = repo.delete(Uuid::new()).await?;
        assert!(!deleted);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_project_scopes() -> Result<(), Error> {
        let (repo, db) = setup().await;
        let project_id = Uuid::new();
        let scope1 = ProjectScope {
            id: Some(Uuid::new()),
            project_id,
            name: "read:users".to_string(),
            description: "Allows reading user data".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let scope2 = ProjectScope {
            id: Some(Uuid::new()),
            project_id: Uuid::new(),
            name: "write:users".to_string(),
            description: "Allows writing user data".to_string(),
            enabled: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        repo.create(scope1).await?;
        repo.create(scope2).await?;

        // Test finding by project_id
        let filter = ProjectScopeFilter {
            project_id: Some(project_id),
            name: None,
            is_enabled: None,
        };
        let found = repo.find(filter, None, None).await?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].project_id, project_id);

        // Test finding by enabled status
        let enabled_filter = ProjectScopeFilter {
            project_id: None,
            name: None,
            is_enabled: Some(true),
        };
        let enabled_scopes = repo.find(enabled_filter, None, None).await?;
        assert_eq!(enabled_scopes.len(), 1);
        assert!(enabled_scopes[0].enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }
}
