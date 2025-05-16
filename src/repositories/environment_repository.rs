use crate::models::environment::{Environment, EnvironmentUpdatePayload};
use crate::repositories::base::Repository;
use anyhow::{Result, Error};
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::bson::Uuid;
use mongodb::{Collection, Database};
use mongodb::bson::{Document, doc, to_document};

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
}

#[async_trait]
impl Repository<Environment> for EnvironmentRepository {
    type UpdatePayload = EnvironmentUpdatePayload;

    async fn create(&self, mut item: Environment) -> Result<Environment, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        self.collection.insert_one(&item).await?;
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<Environment>, Error> {
        let result = self.collection.find_one(doc! { "_id": id }).await?;
        Ok(result)
    }

    async fn replace(&self, id: Uuid, mut item: Environment) -> Result<Environment, Error> {
        if item.id.is_none() || item.id.unwrap() != id {
            item.id = Some(id);
        }
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! { "$set": to_document(&item)? }
            )
            .await?;
        let updated = self.collection.find_one(doc! { "_id": id }).await?.unwrap();
        Ok(updated)
    }

    async fn update(&self, id: Uuid, payload: Self::UpdatePayload) -> Result<Environment, Error> {
        let document = to_document(&payload)?;
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": document })
            .await?;
        let updated = self.read(id).await?.ok_or_else(|| Error::msg("Environment not found"))?;
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(&self, filter: Document) -> Result<Vec<Environment>, Error> {
        let result = self.collection.find(filter).await?;
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
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use mongodb::options::IndexOptions;
    use mongodb::IndexModel;

    async fn ensure_unique_index(db: &Database) -> Result<()> {
        let collection = db.collection::<Environment>("environments");
        let options = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { "project_id": 1, "name": 1 })
            .options(options)
            .build();
        collection.create_index(index).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_environment() -> Result<()> {
        let db = setup_test_db("environment").await?;
        ensure_unique_index(&db).await?;
        let repo = EnvironmentRepository::new(db.clone())?;
        
        let project_id = Uuid::new();
        let environment = Environment::new(
            project_id,
            "Test Environment".to_string(),
            "Test Description".to_string(),
        );
        
        let created = repo.create(environment).await?;
        assert!(created.id.is_some());
        assert_eq!(created.name, "Test Environment");
        
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_duplicate_environment() -> Result<()> {
        let db = setup_test_db("environment").await?;
        ensure_unique_index(&db).await?;
        let repo = EnvironmentRepository::new(db.clone())?;
        
        let project_id = Uuid::new();
        let name = "Test Environment".to_string();
        
        let environment1 = Environment::new(
            project_id,
            name.clone(),
            "Test Description 1".to_string(),
        );
        repo.create(environment1).await?;
        
        let environment2 = Environment::new(
            project_id,
            name,
            "Test Description 2".to_string(),
        );
        let result = repo.create(environment2).await;
        
        assert!(result.is_err());
        
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_environment() -> Result<()> {
        let db = setup_test_db("environment").await?;
        ensure_unique_index(&db).await?;
        let repo = EnvironmentRepository::new(db.clone())?;

        let project_id = Uuid::new();
        let environment = Environment::new(
            project_id,
            "Test Environment".to_string(),
            "Test Description".to_string(),
        );
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
        let db = setup_test_db("environment").await?;
        ensure_unique_index(&db).await?;
        let repo = EnvironmentRepository::new(db.clone())?;

        let project_id = Uuid::new();
        let environment = Environment::new(
            project_id,
            "Test Environment".to_string(),
            "Test Description".to_string(),
        );
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
        let db = setup_test_db("environment").await?;
        ensure_unique_index(&db).await?;
        let repo = EnvironmentRepository::new(db.clone())?;

        let project_id = Uuid::new();
        let environment = Environment::new(
            project_id,
            "Test Environment".to_string(),
            "Test Description".to_string(),
        );
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
        let db = setup_test_db("environment").await?;
        ensure_unique_index(&db).await?;
        let repo = EnvironmentRepository::new(db.clone())?;

        let project_id = Uuid::new();
        let environment1 = Environment::new(
            project_id,
            "Environment 1".to_string(),
            "Description 1".to_string(),
        );
        let environment2 = Environment::new(
            project_id,
            "Environment 2".to_string(),
            "Description 2".to_string(),
        );
        repo.create(environment1).await?;
        repo.create(environment2).await?;

        let environments = repo.find(doc! {}).await?;
        assert_eq!(environments.len(), 2);

        let environments = repo.find(doc! { "name": "Environment 1" }).await?;
        assert_eq!(environments.len(), 1);
        assert_eq!(environments[0].name, "Environment 1");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
