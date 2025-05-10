use crate::models::service_account::ServiceAccount;
use crate::repositories::base::Repository;
use anyhow::Result;
use mongodb::{Collection, Database};

pub struct ServiceAccountRepository {
    collection: Collection<ServiceAccount>,
}

impl ServiceAccountRepository {
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<ServiceAccount>("service_account");
        Ok(Self { collection })
    }
}

impl Repository<ServiceAccount> for ServiceAccountRepository {
    /// Gets the MongoDB collection for Projects.
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
    use crate::config::AppConfig;
    use crate::utils::database::create_database_client;
    use dotenvy::dotenv;
    use mongodb::bson::Bson;
    use tokio;

    async fn setup_test_db() -> Result<Database> {
        dotenv().ok();

        let app_config = AppConfig::from_env(Some(true))?;

        let client = create_database_client(&app_config.application.database_uri).await?;
        let db = client.database("test_db");
        Ok(db)
    }

    async fn cleanup_test_db(db: Database) -> Result<()> {
        db.collection::<ServiceAccount>("projects").drop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_project() -> Result<()> {
        let db = setup_test_db().await?;
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
    async fn test_read_project() -> Result<()> {
        let db = setup_test_db().await?;
        let repo = ServiceAccountRepository::new(db.clone())?;

        // Create a project first
        let email = "Examples@google.com".to_string();
        let user = "Example123".to_string();
        let secret = "ExampleSercet".to_string();

        let service_account = ServiceAccount::new(email.clone(), user.clone(), secret.clone());

        let result = repo.create(service_account).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        // Read the project
        let read_service_account = repo.read(id).await?;
        assert!(read_service_account.is_some());
        let read_service_account = read_service_account.unwrap();
        assert_eq!(read_service_account.email(), email);
        assert_eq!(read_service_account.user(), user);
        assert_eq!(read_service_account.secret(),secret);
        assert_eq!(read_service_account.enabled(), true);

        cleanup_test_db(db).await?;
        Ok(())
    }



}