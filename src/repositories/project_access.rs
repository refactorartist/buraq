use crate::models::project_access::ProjectAccess;
use crate::repositories::base::Repository;
use anyhow::Result;
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
    /// Returns a Result containing the ProjectAccessRepository or an error if collection creation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use buraq::repositories::project_access::ProjectAccessRepository;
    /// use mongodb::Client;
    /// use buraq::utils::database::create_database_client;
    /// use dotenvy::dotenv;
    /// use buraq::config::AppConfig;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     dotenv().ok();
    ///
    ///     let app_config = AppConfig::from_env(Some(true))?;
    ///     let client = create_database_client(&app_config.application.database_uri).await?;
    ///     let db = client.database("test_db");
    ///     let repo = ProjectAccessRepository::new(db)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<ProjectAccess>("project_access");
        Ok(Self { collection })
    }
}

impl Repository<ProjectAccess> for ProjectAccessRepository {
    /// Gets the MongoDB collection for ProjectAccess.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Collection or an error if cloning fails.
    fn collection(&self) -> Result<Collection<ProjectAccess>, anyhow::Error> {
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
        let db = client.database("test_db__project_access");
        Ok(db)
    }

    async fn cleanup_test_db(db: Database) -> Result<()> {
        db.collection::<ProjectAccess>("project_access").drop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_project_access() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = ProjectAccessRepository::new(db.clone())?;

        let project_access = ProjectAccess::new(
            "Test Access".to_string(),
            mongodb::bson::oid::ObjectId::new(),
            mongodb::bson::oid::ObjectId::new(),
            vec![mongodb::bson::oid::ObjectId::new()],
        );
        let result = repo.create(project_access).await?;

        assert!(matches!(result.inserted_id, Bson::ObjectId(_)));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_project_access() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = ProjectAccessRepository::new(db.clone())?;

        let project_access = ProjectAccess::new(
            "Test Access".to_string(),
            mongodb::bson::oid::ObjectId::new(),
            mongodb::bson::oid::ObjectId::new(),
            vec![mongodb::bson::oid::ObjectId::new()],
        );
        let result = repo.create(project_access).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let read_access = repo.read(id).await?;
        assert!(read_access.is_some());
        let read_access = read_access.unwrap();
        assert_eq!(read_access.name(), "Test Access");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project_access() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = ProjectAccessRepository::new(db.clone())?;

        let project_access = ProjectAccess::new(
            "Test Access".to_string(),
            mongodb::bson::oid::ObjectId::new(),
            mongodb::bson::oid::ObjectId::new(),
            vec![mongodb::bson::oid::ObjectId::new()],
        );
        let result = repo.create(project_access).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let updated_access = ProjectAccess::new(
            "Updated Access".to_string(),
            mongodb::bson::oid::ObjectId::new(),
            mongodb::bson::oid::ObjectId::new(),
            vec![mongodb::bson::oid::ObjectId::new()],
        );
        let update_result = repo.update(id, updated_access).await?;
        assert_eq!(update_result.modified_count, 1);

        let read_access = repo.read(id).await?.unwrap();
        assert_eq!(read_access.name(), "Updated Access");

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project_access() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = ProjectAccessRepository::new(db.clone())?;

        let project_access = ProjectAccess::new(
            "Test Access".to_string(),
            mongodb::bson::oid::ObjectId::new(),
            mongodb::bson::oid::ObjectId::new(),
            vec![mongodb::bson::oid::ObjectId::new()],
        );
        let result = repo.create(project_access).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let delete_result = repo.delete(id).await?;
        assert_eq!(delete_result.deleted_count, 1);

        let read_access = repo.read(id).await?;
        assert!(read_access.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_project_access() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = ProjectAccessRepository::new(db.clone())?;

        let environment_id = mongodb::bson::oid::ObjectId::new();
        let service_account_id = mongodb::bson::oid::ObjectId::new();

        let access1 = ProjectAccess::new(
            "Access 1".to_string(),
            environment_id,
            service_account_id,
            vec![mongodb::bson::oid::ObjectId::new()],
        );
        let access2 = ProjectAccess::new(
            "Access 2".to_string(),
            environment_id,
            service_account_id,
            vec![mongodb::bson::oid::ObjectId::new()],
        );
        repo.create(access1).await?;
        repo.create(access2).await?;

        let access_list = repo.find(doc! {}).await?;
        assert_eq!(access_list.len(), 2);

        let filtered_list = repo.find(doc! { "name": "Access 1" }).await?;
        assert_eq!(filtered_list.len(), 1);
        assert_eq!(filtered_list[0].name(), "Access 1");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
