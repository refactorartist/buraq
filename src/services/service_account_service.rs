use crate::models::service_account::{self, ServiceAccount};
use crate::repositories::base::Repository;
use crate::repositories::service_account::ServiceAccountRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

pub struct ServiceAccountService {
    service_account_repository: ServiceAccountRepository,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceAccountFilter {
    pub email: Option<String>,
    pub user: Option<String>,
    pub enable: Option<bool>,
}

impl From<ServiceAccountFilter> for mongodb::bson::Document {
    fn from(val: ServiceAccountFilter) -> Self {
        let mut doc = mongodb::bson::Document::new();

        if let Some(email) = val.email {
            doc.insert("email", email);
        }

        if let Some(user) = val.user {
            doc.insert("user", user);
        }

        if let Some(enable) = val.enable {
            doc.insert("enable", enable);
        }

        doc
    }
}

impl ServiceAccountService {
    pub fn new(database: Database) -> Result<Self, Error> {
        let service_account_repository = ServiceAccountRepository::new(database)?;
        Ok(Self {
            service_account_repository,
        })
    }

    pub async fn create_service_account(
        &self,
        service_account: ServiceAccount,
    ) -> Result<ServiceAccount, Error> {
        let result = self
            .service_account_repository
            .create(service_account.clone())
            .await?;
        let id = result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| anyhow::anyhow!("Invalid ObjectId"))?;

        // Fetch the newly created service account
        let inserted_service_account = self
            .service_account_repository
            .read(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch created service account"))?;

        Ok(inserted_service_account)
    }

    pub async fn get_service_account(&self, id: ObjectId) -> Result<Option<ServiceAccount>, Error> {
        self.service_account_repository.read(id).await
    }

    pub async fn update_service_account(
        &self,
        id: ObjectId,
        service_account: ServiceAccount,
    ) -> Result<ServiceAccount, Error> {
        let result = self
            .service_account_repository
            .update(id, service_account)
            .await?;
        if result.modified_count > 0 {
            log::info!("Service account updated successfully: {:?}", id);
        } else {
            log::error!("Failed to update service account: {:?}", id);
        }
        let updated_service_account = self
            .service_account_repository
            .read(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch updated service account"))?;
        Ok(updated_service_account)
    }
    pub async fn delete_service_account(&self, id: ObjectId) -> Result<bool, Error> {
        let result = self.service_account_repository.delete(id).await?;
        Ok(result.deleted_count > 0)
    }
    pub async fn get_service_accounts(
        &self,
        filter: ServiceAccountFilter,
    ) -> Result<Vec<ServiceAccount>, Error> {
        let filter_doc = filter.into();
        let service_accounts = self.service_account_repository.find(filter_doc).await?;
        Ok(service_accounts)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use anyhow::Ok;
    use tokio;
    async fn setup_service_accounts_for_filter_tests(
        service_account_service: &ServiceAccountService,
    ) -> Result<(), Error> {
        // Clean up any existing data first
        let collection = service_account_service.service_account_repository.collection()?;
        let db = collection.client().database(&collection.namespace().db);
        cleanup_test_db(db).await?;
        
        // Create multiple service accounts for testing filters
        let service_account1 = ServiceAccount::new(
            "user1@example.com".to_string(), 
            "User 1".to_string(), 
            "secret".to_string()
        );
        let service_account2 = ServiceAccount::new(
            "user2@example.com".to_string(), 
            "User 2".to_string(), 
            "secret".to_string()
        );
        let service_account3 = ServiceAccount::new(
            "user3@example.com".to_string(), 
            "User 3".to_string(), 
            "secret".to_string()
        );
        
        service_account_service.create_service_account(service_account1).await?;
        service_account_service.create_service_account(service_account2).await?;
        service_account_service.create_service_account(service_account3).await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_create_service_account() -> Result<(), Error> {
        let db = setup_test_db("service_account_service").await?;
        let service_account_service = ServiceAccountService::new(db.clone())?;
        let service_account = ServiceAccount::new(
            "test@example.com".to_string(), 
            "Test User".to_string(), 
            "secret".to_string()
        );
        let result = service_account_service.create_service_account(service_account).await?;
        assert!(result.id().is_some());
        assert_eq!(result.email(), "test@example.com");
        assert_eq!(result.user(), "Test User");
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_new() -> Result<(), Error> {
        let db = setup_test_db("service_account_service").await?;
        let service_account_service = ServiceAccountService::new(db.clone())?;
        assert!(service_account_service.service_account_repository.collection().is_ok());
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_service_account() -> Result<(), Error> {
        let db = setup_test_db("service_account_service").await?;
        let service_account_service = ServiceAccountService::new(db.clone())?;
        let service_account = ServiceAccount::new(
            "test@example.com".to_string(), 
            "Test User".to_string(), 
            "secret".to_string()
        );
        let result = service_account_service.create_service_account(service_account).await?;
        assert!(result.id().is_some());
        let service_account = service_account_service.get_service_account(*result.id().unwrap()).await?;
        assert!(service_account.is_some());
        let service_account = service_account.unwrap();
        assert_eq!(service_account.email(), "test@example.com");
        assert_eq!(service_account.user(), "Test User");
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_service_account() -> Result<(), Error> {
        let db = setup_test_db("service_account_service").await?;
        let service_account_service = ServiceAccountService::new(db.clone())?;
        let service_account = ServiceAccount::new(
            "test@example.com".to_string(), 
            "Test User".to_string(), 
            "secret".to_string()
        );
        let result = service_account_service.create_service_account(service_account).await?;
        assert!(result.id().is_some());
        let updated_service_account = ServiceAccount::new(
            "updated@example.com".to_string(), 
            "Updated User".to_string(), 
            "new_secret".to_string()
        );
        let result = service_account_service.update_service_account(*result.id().unwrap(), updated_service_account).await?;
        assert!(result.id().is_some());
        assert_eq!(result.email(), "updated@example.com");
        assert_eq!(result.user(), "Updated User");
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_service_account() -> Result<(), Error> {
        let db = setup_test_db("service_account_service").await?;
        let service_account_service = ServiceAccountService::new(db.clone())?;
        let service_account = ServiceAccount::new(
            "test@example.com".to_string(), 
            "Test User".to_string(), 
            "secret".to_string()
        );
        let result = service_account_service.create_service_account(service_account).await?;
        assert!(result.id().is_some());
        let result = service_account_service.delete_service_account(*result.id().unwrap()).await?;
        assert!(result);
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_service_accounts_no_filter() -> Result<(), Error> {
        let db = setup_test_db("service_account_service").await?;
        let service_account_service = ServiceAccountService::new(db.clone())?;
        // Setup test data
        setup_service_accounts_for_filter_tests(&service_account_service).await?;
        // Test with empty filter (should return all service accounts)
        let filter = ServiceAccountFilter {
            email: None,
            user: None,
            enable: None,
        };
        
        let service_accounts = service_account_service.get_service_accounts(filter).await?;
        assert_eq!(service_accounts.len(), 3, "Should have retrieved all 3 service accounts with no filter");
        
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_service_accounts_filter_by_email() -> Result<(), Error> {
        let db = setup_test_db("service_account_service").await?;
        let service_account_service = ServiceAccountService::new(db.clone())?;
        // Setup test data
        setup_service_accounts_for_filter_tests(&service_account_service).await?;
        // Test filtering by email
        let filter = ServiceAccountFilter {
            email: Some("user1@example.com".to_string()),
            user: None,
            enable: None,
        };
        
        let service_accounts = service_account_service.get_service_accounts(filter).await?;
        assert_eq!(service_accounts.len(), 1, "Should have retrieved 1 service account when filtering by email");
        assert_eq!(service_accounts[0].email(), "user1@example.com");
        
        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_service_accounts_filter_by_user() -> Result<(), Error> {
        let db = setup_test_db("service_account_service").await?;
        let service_account_service = ServiceAccountService::new(db.clone())?;
        // Setup test data
        setup_service_accounts_for_filter_tests(&service_account_service).await?;
        // Test filtering by user
        let filter = ServiceAccountFilter {
            email: None,
            user: Some("User 2".to_string()),
            enable: None,
        };
        
        let service_accounts = service_account_service.get_service_accounts(filter).await?;
        assert_eq!(service_accounts.len(), 1, "Should have retrieved 1 service account when filtering by user");
        assert_eq!(service_accounts[0].user(), "User 2");
        
        cleanup_test_db(db).await?;
        Ok(())
    }

}