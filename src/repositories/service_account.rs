use crate::models::service_account::ServiceAccount;
use crate::repositories::base::Repository;
use anyhow::Result;
use mongodb::{Collection, Database};

/// Repository for managing ServiceAccount documents in MongoDB.
///
/// Provides CRUD operations for ServiceAccount entities.
pub struct ServiceAccountRepository {
    collection: Collection<ServiceAccount>,
}

impl ServiceAccountRepository {
    /// Creates a new ServiceAccountRepository instance.
    ///
    /// # Arguments
    ///
    /// * `database` - MongoDB Database instance
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ServiceAccountRepository or an error if collection creation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use buraq::repositories::service_account::ServiceAccountRepository;
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
    ///     let repo = ServiceAccountRepository::new(db)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<ServiceAccount>("service_account");
        Ok(Self { collection })
    }
}

impl Repository<ServiceAccount> for ServiceAccountRepository {
    /// Gets the MongoDB collection for ServiceAccounts.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Collection or an error if cloning fails.
    fn collection(&self) -> Result<Collection<ServiceAccount>, anyhow::Error> {
        Ok(self.collection.clone())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use mongodb::bson::Bson;
    use tokio;

    #[tokio::test]
    async fn test_create_service_account() -> Result<()> {
        let db = setup_test_db("service_account").await?;
        let repo = ServiceAccountRepository::new(db.clone())?;

        let email = "Examples@google.com".to_string();
        let user = "Example123".to_string();
        let secret = "ExampleSercet".to_string();

        let service_account = ServiceAccount::new(email.clone(), user.clone(), secret.clone());

        let result = repo.create(service_account).await?;

        assert!(matches!(result.inserted_id, Bson::ObjectId(_)));

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_service_account() -> Result<()> {
        let db = setup_test_db("service_account").await?;
        let repo = ServiceAccountRepository::new(db.clone())?;

        let email = "Examples@google.com".to_string();
        let user = "Example123".to_string();
        let secret = "ExampleSercet".to_string();

        let service_account = ServiceAccount::new(email.clone(), user.clone(), secret.clone());
        let result = repo.create(service_account).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let read_account = repo.read(id).await?;
        assert!(read_account.is_some());
        let read_account = read_account.unwrap();
        assert_eq!(read_account.email(), email);
        assert_eq!(read_account.user(), user);
        assert_eq!(read_account.secret(), secret);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_service_account() -> Result<()> {
        let db = setup_test_db("service_account").await?;
        let repo = ServiceAccountRepository::new(db.clone())?;

        let email = "Examples@google.com".to_string();
        let user = "Example123".to_string();
        let secret = "ExampleSercet".to_string();

        let service_account = ServiceAccount::new(email.clone(), user.clone(), secret.clone());
        let result = repo.create(service_account).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let updated_email = "Updated@google.com".to_string();
        let updated_user = "Updated123".to_string();
        let updated_secret = "UpdatedSecret".to_string();

        let updated_account = ServiceAccount::new(updated_email.clone(), updated_user.clone(), updated_secret.clone());
        let update_result = repo.update(id, updated_account).await?;
        assert_eq!(update_result.modified_count, 1);

        let read_account = repo.read(id).await?.unwrap();
        assert_eq!(read_account.email(), updated_email);
        assert_eq!(read_account.user(), updated_user);
        assert_eq!(read_account.secret(), updated_secret);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_service_account() -> Result<()> {
        let db = setup_test_db("service_account").await?;
        let repo = ServiceAccountRepository::new(db.clone())?;

        let email = "Examples@google.com".to_string();
        let user = "Example123".to_string();
        let secret = "ExampleSercet".to_string();

        let service_account = ServiceAccount::new(email.clone(), user.clone(), secret.clone());
        let result = repo.create(service_account).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let delete_result = repo.delete(id).await?;
        assert_eq!(delete_result.deleted_count, 1);

        let read_account = repo.read(id).await?;
        assert!(read_account.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }
}