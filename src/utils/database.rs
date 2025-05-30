use anyhow::Error;
use mongodb::{Client, Database};
use std::sync::Arc;

use crate::repositories::{
    access_token_repository::AccessTokenRepository, environment_repository::EnvironmentRepository,
    project_access_repository::ProjectAccessRepository, project_repository::ProjectRepository,
    project_scope_repository::ProjectScopeRepository,
    service_account_key_repository::ServiceAccountKeyRepository,
    service_account_repository::ServiceAccountRepository,
};

pub async fn create_database_client(database_uri: &str) -> Result<Arc<Client>, anyhow::Error> {
    let client = mongodb::Client::with_uri_str(&database_uri)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create MongoDB client: {}", e))?;

    Ok(Arc::new(client))
}

pub async fn setup_database(database: Database) -> Result<(), Error> {
    AccessTokenRepository::new(database.clone())
        .unwrap()
        .ensure_indexes()
        .await?;
    EnvironmentRepository::new(database.clone())
        .unwrap()
        .ensure_indexes()
        .await?;
    ProjectAccessRepository::new(database.clone())
        .unwrap()
        .ensure_indexes()
        .await?;
    ProjectRepository::new(database.clone())
        .unwrap()
        .ensure_indexes()
        .await?;
    ProjectScopeRepository::new(database.clone())
        .unwrap()
        .ensure_indexes()
        .await?;
    ServiceAccountKeyRepository::new(database.clone())
        .unwrap()
        .ensure_indexes()
        .await?;
    ServiceAccountRepository::new(database.clone())
        .unwrap()
        .ensure_indexes()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use std::sync::Arc;
    use tokio;

    #[tokio::test]
    async fn test_create_database_client_success() {
        let database_uri = "mongodb://localhost:27017";
        let client_result = create_database_client(database_uri).await;
        assert!(client_result.is_ok());

        let client = client_result.unwrap();
        assert!(Arc::strong_count(&client) > 0);
    }

    #[tokio::test]
    async fn test_create_database_client_invalid_uri() {
        let invalid_database_uri = "invalid_uri";
        let client_result = create_database_client(invalid_database_uri).await;
        assert!(client_result.is_err());
    }

    #[tokio::test]
    async fn test_setup_database_success() {
        // Setup test database
        let db = setup_test_db("setup_database_test").await.unwrap();

        // Test setup_database
        let result = setup_database(db.clone()).await;
        assert!(result.is_ok());

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_setup_database_duplicate_indexes() {
        // Setup test database
        let db = setup_test_db("setup_database_duplicate_test")
            .await
            .unwrap();

        // First setup should succeed
        let result1 = setup_database(db.clone()).await;
        assert!(result1.is_ok());

        // Second setup should also succeed (idempotent)
        let result2 = setup_database(db.clone()).await;
        assert!(result2.is_ok());

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
}
