use crate::models::environment::Environment;
use crate::repositories::base::Repository;
use anyhow::Result;
use mongodb::{Collection, Database};

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
    /// Returns a Result containing the EnvironmentRepository or an error if collection creation fails.
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<Environment>("environments");
        Ok(Self { collection })
    }
}

impl Repository<Environment> for EnvironmentRepository {
    /// Gets the MongoDB collection for Environments.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Collection or an error if cloning fails.
    fn collection(&self) -> Result<Collection<Environment>, anyhow::Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::utils::database::create_database_client;
    use dotenvy::dotenv;
    use mongodb::bson::{Bson, doc};
    use tokio;

    async fn setup_test_db() -> Result<Database> {
        dotenv().ok();

        let app_config = AppConfig::from_env(Some(true))?;

        let client = create_database_client(&app_config.application.database_uri).await?;
        let db = client.database("test_db__environments");
        Ok(db)
    }

    async fn cleanup_test_db(db: Database) -> Result<()> {
        db.collection::<Environment>("environments").drop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_environment() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = EnvironmentRepository::new(db.clone())?;

        let project_id = mongodb::bson::oid::ObjectId::new();
        let environment = Environment::new(
            project_id,
            "Test Environment".to_string(),
            "Test Description".to_string(),
        );
        let result = repo.create(environment).await?;

        assert!(matches!(result.inserted_id, Bson::ObjectId(_)));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_environment() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = EnvironmentRepository::new(db.clone())?;

        let project_id = mongodb::bson::oid::ObjectId::new();
        let environment = Environment::new(
            project_id,
            "Test Environment".to_string(),
            "Test Description".to_string(),
        );
        let result = repo.create(environment).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let read_environment = repo.read(id).await?;
        assert!(read_environment.is_some());
        let read_environment = read_environment.unwrap();
        assert_eq!(read_environment.name(), "Test Environment");
        assert_eq!(read_environment.description(), "Test Description");
        assert!(read_environment.enabled());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_environment() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = EnvironmentRepository::new(db.clone())?;

        let project_id = mongodb::bson::oid::ObjectId::new();
        let environment = Environment::new(
            project_id,
            "Test Environment".to_string(),
            "Test Description".to_string(),
        );
        let result = repo.create(environment).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let updated_environment = Environment::new(
            project_id,
            "Updated Environment".to_string(),
            "Updated Description".to_string(),
        );
        let update_result = repo.update(id, updated_environment).await?;
        assert_eq!(update_result.modified_count, 1);

        let read_environment = repo.read(id).await?.unwrap();
        assert_eq!(read_environment.name(), "Updated Environment");
        assert_eq!(read_environment.description(), "Updated Description");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_environment() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = EnvironmentRepository::new(db.clone())?;

        let project_id = mongodb::bson::oid::ObjectId::new();
        let environment = Environment::new(
            project_id,
            "Test Environment".to_string(),
            "Test Description".to_string(),
        );
        let result = repo.create(environment).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let delete_result = repo.delete(id).await?;
        assert_eq!(delete_result.deleted_count, 1);

        let read_environment = repo.read(id).await?;
        assert!(read_environment.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_environments() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = EnvironmentRepository::new(db.clone())?;

        let project_id = mongodb::bson::oid::ObjectId::new();
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
        assert_eq!(environments[0].name(), "Environment 1");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_environments_by_project() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = EnvironmentRepository::new(db.clone())?;

        let project_id_1 = mongodb::bson::oid::ObjectId::new();
        let project_id_2 = mongodb::bson::oid::ObjectId::new();

        // Create environments for project 1
        let environment1 = Environment::new(
            project_id_1,
            "Project 1 Environment 1".to_string(),
            "Description 1".to_string(),
        );
        let environment2 = Environment::new(
            project_id_1,
            "Project 1 Environment 2".to_string(),
            "Description 2".to_string(),
        );
        repo.create(environment1).await?;
        repo.create(environment2).await?;

        // Create environment for project 2
        let environment3 = Environment::new(
            project_id_2,
            "Project 2 Environment 1".to_string(),
            "Description 3".to_string(),
        );
        repo.create(environment3).await?;

        // Find environments for project 1
        let environments = repo.find(doc! { "project_id": project_id_1 }).await?;
        assert_eq!(environments.len(), 2);
        assert!(environments.iter().all(|env| env.project_id() == &project_id_1));
        assert!(environments.iter().any(|env| env.name() == "Project 1 Environment 1"));
        assert!(environments.iter().any(|env| env.name() == "Project 1 Environment 2"));

        // Find environments for project 2
        let environments = repo.find(doc! { "project_id": project_id_2 }).await?;
        assert_eq!(environments.len(), 1);
        assert_eq!(environments[0].project_id(), &project_id_2);
        assert_eq!(environments[0].name(), "Project 2 Environment 1");

        // Find environments for non-existent project
        let non_existent_project_id = mongodb::bson::oid::ObjectId::new();
        let environments = repo.find(doc! { "project_id": non_existent_project_id }).await?;
        assert_eq!(environments.len(), 0);

        cleanup_test_db(db).await?;
        Ok(())
    }
}
