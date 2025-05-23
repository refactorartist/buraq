use crate::models::project_access::{ProjectAccess, ProjectAccessUpdatePayload, ProjectAccessFilter, ProjectAccessSortableFields};
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

/// Repository for managing ProjectAccess documents in MongoDB.
///
/// Provides CRUD operations for ProjectAccess entities.
pub struct ProjectAccessRepository {
    collection: Collection<ProjectAccess>,
}

impl ProjectAccessRepository {
    /// Creates a new ProjectAccessRepository instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ProjectAccessRepository or an error if initialization fails.
    pub fn new(database: Database) -> Result<Self, Error> {
        let collection = database.collection::<ProjectAccess>("project_access");
        Ok(Self { collection })
    }
}

#[async_trait]
impl Repository<ProjectAccess> for ProjectAccessRepository {
    type UpdatePayload = ProjectAccessUpdatePayload;
    type Filter = ProjectAccessFilter;
    type Sort = ProjectAccessSortableFields;

    async fn create(&self, mut item: ProjectAccess) -> Result<ProjectAccess, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        item.created_at = Some(Utc::now());
        item.updated_at = Some(Utc::now());
        self.collection.insert_one(&item).await?;
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<ProjectAccess>, Error> {
        let result = self.collection.find_one(doc! { "_id": id }).await?;
        Ok(result)
    }

    async fn replace(&self, id: Uuid, mut item: ProjectAccess) -> Result<ProjectAccess, Error> {
        if item.id.is_none() || item.id.unwrap() != id {
            item.id = Some(id);
        }
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": to_document(&item)? })
            .await?;
        let updated = self.collection.find_one(doc! { "_id": id }).await?.unwrap();
        Ok(updated)
    }

    async fn update(&self, id: Uuid, payload: Self::UpdatePayload) -> Result<ProjectAccess, Error> {
        let mut document = to_document(&payload)?;
        document.insert("updated_at", Bson::String(Utc::now().to_rfc3339()));

        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": document })
            .await?;
        let updated = self
            .read(id)
            .await?
            .ok_or_else(|| Error::msg("ProjectAccess not found"))?;
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(&self, filter: Self::Filter, sort: Option<SortBuilder<Self::Sort>>, pagination: Option<Pagination>) -> Result<Vec<ProjectAccess>, Error> {
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
        let items: Vec<ProjectAccess> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<ProjectAccess>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use chrono::Utc;

    async fn setup() -> (ProjectAccessRepository, Database) {
        let db = setup_test_db("project_access").await.unwrap();
        let repo = ProjectAccessRepository::new(db.clone()).expect("Failed to create repository");
        (repo, db)
    }

    #[tokio::test]
    async fn test_create_project_access() -> Result<()> {
        let (repo, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = repo.create(project_access.clone()).await.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.name, project_access.name);
        assert!(created.enabled);
        assert!(created.created_at.is_some());
        assert!(created.updated_at.is_some());

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_read_project_access() -> Result<()> {
        let (repo, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = repo.create(project_access.clone()).await.unwrap();
        let read = repo.read(created.id.unwrap()).await.unwrap().unwrap();
        assert_eq!(read.id, created.id);
        assert_eq!(read.name, created.name);
        assert_eq!(read.enabled, created.enabled);
        assert_eq!(read.created_at, created.created_at);
        assert_eq!(read.updated_at, created.updated_at);

        // Test reading non-existent access
        let non_existent = repo.read(Uuid::new()).await.unwrap();
        assert!(non_existent.is_none());

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project_access() -> Result<()> {
        let (repo, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = repo.create(project_access).await.unwrap();
        let update = ProjectAccessUpdatePayload {
            name: Some("Updated Access".to_string()),
            project_scopes: Some(vec![Uuid::new()]),
            enabled: Some(false),
        };

        let updated = repo.update(created.id.unwrap(), update).await.unwrap();
        assert_eq!(updated.name, "Updated Access");
        assert!(!updated.enabled);
        assert!(updated.updated_at.unwrap() > created.updated_at.unwrap());

        // Test updating non-existent access
        let non_existent_update = repo
            .update(
                Uuid::new(),
                ProjectAccessUpdatePayload {
                    name: Some("Test".to_string()),
                    project_scopes: None,
                    enabled: None,
                },
            )
            .await;
        assert!(non_existent_update.is_err());

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project_access() -> Result<()> {
        let (repo, db) = setup().await;
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = repo.create(project_access).await.unwrap();
        let deleted = repo.delete(created.id.unwrap()).await.unwrap();
        assert!(deleted);

        let read = repo.read(created.id.unwrap()).await.unwrap();
        assert!(read.is_none());

        // Test deleting non-existent access
        let deleted = repo.delete(Uuid::new()).await.unwrap();
        assert!(!deleted);

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_find_project_access() -> Result<()> {
        let (repo, db) = setup().await;
        let environment_id = Uuid::new();
        let service_account_id = Uuid::new();

        let access1 = ProjectAccess {
            id: None,
            name: "Access 1".to_string(),
            environment_id,
            service_account_id,
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let access2 = ProjectAccess {
            id: None,
            name: "Access 2".to_string(),
            environment_id,
            service_account_id,
            project_scopes: vec![Uuid::new()],
            enabled: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        repo.create(access1).await.unwrap();
        repo.create(access2).await.unwrap();

        // Test finding all access
        let filter = ProjectAccessFilter {
            environment_id: None,
            service_account_id: None,
            project_scopes: None,
            is_enabled: None,
        };
        let all_access = repo.find(filter, None, None).await.unwrap();
        assert_eq!(all_access.len(), 2);

        // Test finding by environment_id
        let env_filter = ProjectAccessFilter {
            environment_id: Some(environment_id),
            service_account_id: None,
            project_scopes: None,
            is_enabled: None,
        };
        let env_access = repo.find(env_filter, None, None).await.unwrap();
        assert_eq!(env_access.len(), 2);

        // Test finding by enabled status
        let enabled_filter = ProjectAccessFilter {
            environment_id: None,
            service_account_id: None,
            project_scopes: None,
            is_enabled: Some(true),
        };
        let enabled_access = repo.find(enabled_filter, None, None).await.unwrap();
        assert_eq!(enabled_access.len(), 1);
        assert!(enabled_access[0].enabled);

        cleanup_test_db(db).await.unwrap();
        Ok(())
    }
}
