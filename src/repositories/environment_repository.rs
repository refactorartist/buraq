use crate::models::environment::{
    Environment, EnvironmentFilter, EnvironmentSortableFields, EnvironmentUpdatePayload,
};
use crate::models::pagination::Pagination;
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use anyhow::{Error, Result};
use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::Uuid;
use mongodb::bson::{Bson, doc, to_document};
use mongodb::options::IndexOptions;
use mongodb::{Collection, Database, IndexModel};

/// Repository for managing Environment documents in MongoDB.
///
/// Provides CRUD operations for Environment entities.
pub struct EnvironmentRepository {
    collection: Collection<Environment>,
}

impl EnvironmentRepository {
    /// Creates a new EnvironmentRepository instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the EnvironmentRepository or an error if initialization fails.
    pub fn new(database: Database) -> Result<Self, Error> {
        let collection = database.collection::<Environment>("environments");
        Ok(Self { collection })
    }

    pub async fn ensure_indexes(&self) -> Result<(), Error> {
        let _ = &self
            .collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "project_id": 1, "name": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await
            .expect("Failed to create index on project_id, name");

        let _ = &self
            .collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "project_id": 1, "enabled": 1 })
                    .build(),
            )
            .await
            .expect("Failed to create index on project_id, enabled");

        Ok(())
    }
}

#[async_trait]
impl Repository<Environment> for EnvironmentRepository {
    type UpdatePayload = EnvironmentUpdatePayload;
    type Filter = EnvironmentFilter;
    type Sort = EnvironmentSortableFields;

    async fn create(&self, mut item: Environment) -> Result<Environment, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        item.created_at = Some(Utc::now());
        item.updated_at = Some(Utc::now());
        self.collection.insert_one(&item).await?;
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<Environment>, Error> {
        let result = self.collection.find_one(doc! { "_id": id }).await?;
        Ok(result)
    }

    async fn update(&self, id: Uuid, payload: Self::UpdatePayload) -> Result<Environment, Error> {
        let mut document = to_document(&payload)?;
        document.insert("updated_at", Bson::String(Utc::now().to_rfc3339()));

        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": document })
            .await?;
        let updated = self
            .read(id)
            .await?
            .ok_or_else(|| Error::msg("Environment not found"))?;
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(
        &self,
        filter: Self::Filter,
        sort: Option<SortBuilder<Self::Sort>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<Environment>, Error> {
        let filter_doc = filter.into();

        // Create FindOptions
        let mut options = mongodb::options::FindOptions::default();

        if let Some(s) = sort {
            options.sort = Some(s.to_document());
        }

        if let Some(p) = pagination {
            options.skip = Some(p.skip());
            options.limit = Some(p.limit());
        }

        let result = self
            .collection
            .find(filter_doc)
            .with_options(options)
            .await?;
        let items: Vec<Environment> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<Environment>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};

    async fn setup() -> (EnvironmentRepository, Database) {
        let db = setup_test_db("environment").await.unwrap();
        let repo = EnvironmentRepository::new(db.clone()).expect("Failed to create repository");
        repo.ensure_indexes()
            .await
            .expect("Failed to create indexes");
        (repo, db)
    }

    #[tokio::test]
    async fn test_create_environment() -> Result<()> {
        let (repo, db) = setup().await;

        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let created = repo.create(environment).await?;
        assert!(created.id.is_some());
        assert_eq!(created.name, "Test Environment");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_duplicate_environment() -> Result<()> {
        let (repo, db) = setup().await;

        let project_id = Uuid::new();
        let name = "Test Environment".to_string();

        let environment1 = Environment {
            id: None,
            project_id,
            name: name.clone(),
            description: "Test Description 1".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        repo.create(environment1).await?;

        let environment2 = Environment {
            id: None,
            project_id,
            name,
            description: "Test Description 2".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let result = repo.create(environment2).await;

        assert!(result.is_err());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_environment() -> Result<()> {
        let (repo, db) = setup().await;

        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let created = repo.create(environment).await?;

        let read = repo.read(created.id.unwrap()).await?;
        assert!(read.is_some());
        let read = read.unwrap();
        assert_eq!(read.name, "Test Environment");
        assert_eq!(read.description, "Test Description");
        assert!(read.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_environment() -> Result<()> {
        let (repo, db) = setup().await;

        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let created = repo.create(environment).await?;

        let update = EnvironmentUpdatePayload {
            name: Some("Updated Environment".to_string()),
            description: Some("Updated Description".to_string()),
            enabled: Some(false),
        };

        let updated = repo.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.name, "Updated Environment");
        assert_eq!(updated.description, "Updated Description");
        assert!(!updated.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_environment() -> Result<()> {
        let (repo, db) = setup().await;

        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let created = repo.create(environment).await?;

        let deleted = repo.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = repo.read(created.id.unwrap()).await?;
        assert!(read.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_environments() -> Result<()> {
        let (repo, db) = setup().await;

        let project_id = Uuid::new();
        let environment1 = Environment {
            id: None,
            project_id,
            name: "Environment 1".to_string(),
            description: "Description 1".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let environment2 = Environment {
            id: None,
            project_id,
            name: "Environment 2".to_string(),
            description: "Description 2".to_string(),
            enabled: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        repo.create(environment1).await?;
        repo.create(environment2).await?;

        // Test finding all environments
        let filter = EnvironmentFilter {
            project_id: None,
            name: None,
            is_enabled: None,
        };
        let all_environments = repo.find(filter, None, None).await?;
        assert_eq!(all_environments.len(), 2);

        // Test finding by name
        let name_filter = EnvironmentFilter {
            project_id: None,
            name: Some("Environment 1".to_string()),
            is_enabled: None,
        };
        let environments = repo.find(name_filter, None, None).await?;
        assert_eq!(environments.len(), 1);
        assert_eq!(environments[0].name, "Environment 1");

        // Test finding by enabled status
        let enabled_filter = EnvironmentFilter {
            project_id: None,
            name: None,
            is_enabled: Some(true),
        };
        let enabled_environments = repo.find(enabled_filter, None, None).await?;
        assert_eq!(enabled_environments.len(), 1);
        assert!(enabled_environments[0].enabled);

        let disabled_filter = EnvironmentFilter {
            project_id: None,
            name: None,
            is_enabled: Some(false),
        };
        let disabled_environments = repo.find(disabled_filter, None, None).await?;
        assert_eq!(disabled_environments.len(), 1);
        assert!(!disabled_environments[0].enabled);

        // Test finding with non-matching criteria
        let non_matching_filter = EnvironmentFilter {
            project_id: None,
            name: Some("Non-existent".to_string()),
            is_enabled: None,
        };
        let non_matching = repo.find(non_matching_filter, None, None).await?;
        assert_eq!(non_matching.len(), 0);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_environment_filter_by_project_id() -> Result<()> {
        let (repo, db) = setup().await;

        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Environment 1".to_string(),
            description: "Description 1".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        repo.create(environment).await?;

        let filter = EnvironmentFilter {
            project_id: Some(project_id),
            name: None,
            is_enabled: None,
        };
        let environments = repo.find(filter, None, None).await?;
        assert_eq!(environments.len(), 1);
        assert_eq!(environments[0].project_id, project_id);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_environment_filter_by_name() -> Result<()> {
        let (repo, db) = setup().await;

        let environment = Environment {
            id: None,
            project_id: Uuid::new(),
            name: "Test Environment".to_string(),
            description: "Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        repo.create(environment).await?;

        let filter = EnvironmentFilter {
            project_id: None,
            name: Some("Test Environment".to_string()),
            is_enabled: None,
        };
        let environments = repo.find(filter, None, None).await?;
        assert_eq!(environments.len(), 1);
        assert_eq!(environments[0].name, "Test Environment");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_environment_filter_by_enabled() -> Result<()> {
        let (repo, db) = setup().await;

        let environment = Environment {
            id: None,
            project_id: Uuid::new(),
            name: "Environment".to_string(),
            description: "Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        repo.create(environment).await?;

        let filter = EnvironmentFilter {
            project_id: None,
            name: None,
            is_enabled: Some(true),
        };
        let environments = repo.find(filter, None, None).await?;
        assert_eq!(environments.len(), 1);
        assert!(environments[0].enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }
}
