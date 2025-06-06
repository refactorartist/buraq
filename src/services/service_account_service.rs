use crate::models::pagination::Pagination;
use crate::models::service_account::{
    ServiceAccount, ServiceAccountFilter, ServiceAccountSortableFields, ServiceAccountUpdatePayload,
};
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use crate::repositories::service_account_repository::ServiceAccountRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::uuid::Uuid;
use std::sync::Arc;

pub struct ServiceAccountService {
    service_account_repository: ServiceAccountRepository,
}

impl ServiceAccountService {
    pub fn new(database: Arc<Database>) -> Result<Self, Error> {
        let service_account_repository = ServiceAccountRepository::new(database.as_ref().clone())?;
        Ok(Self {
            service_account_repository,
        })
    }

    pub async fn create(&self, service_account: ServiceAccount) -> Result<ServiceAccount, Error> {
        self.service_account_repository
            .create(service_account)
            .await
    }

    pub async fn get_service_account(&self, id: Uuid) -> Result<Option<ServiceAccount>, Error> {
        self.service_account_repository.read(id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        service_account: ServiceAccountUpdatePayload,
    ) -> Result<ServiceAccount, Error> {
        self.service_account_repository
            .update(id, service_account)
            .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        self.service_account_repository.delete(id).await
    }

    pub async fn find(
        &self,
        filter: ServiceAccountFilter,
        sort: Option<SortBuilder<ServiceAccountSortableFields>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<ServiceAccount>, Error> {
        self.service_account_repository
            .find(filter, sort, pagination)
            .await
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};

    async fn setup() -> (ServiceAccountService, Database) {
        let db = setup_test_db("service_account_service").await.unwrap();
        let service = ServiceAccountService::new(Arc::new(db.clone())).unwrap();
        (service, db)
    }

    #[tokio::test]
    async fn test_create_service_account() -> Result<(), Error> {
        let (service, db) = setup().await;
        let account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created = service.create(account.clone()).await?;
        assert!(created.id.is_some());
        assert_eq!(created.email, account.email);
        assert_eq!(created.user, account.user);
        assert_eq!(created.secret, account.secret);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_service_account() -> Result<(), Error> {
        let (service, db) = setup().await;
        let account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created = service.create(account.clone()).await?;
        let retrieved = service
            .get_service_account(created.id.unwrap())
            .await?
            .unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.email, created.email);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_service_account() -> Result<(), Error> {
        let (service, db) = setup().await;
        let account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created = service.create(account).await?;
        let update = ServiceAccountUpdatePayload {
            email: Some("new@example.com".to_string()),
            user: Some("newuser".to_string()),
            secret: Some("newsecret".to_string()),
            enabled: Some(false),
        };

        let updated = service.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.email, "new@example.com");
        assert_eq!(updated.user, "newuser");
        assert_eq!(updated.secret, "newsecret");
        assert!(!updated.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_service_account() -> Result<(), Error> {
        let (service, db) = setup().await;
        let account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created = service.create(account).await?;
        let deleted = service.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = service.get_service_account(created.id.unwrap()).await?;
        assert!(read.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_service_accounts() -> Result<(), Error> {
        let (service, db) = setup().await;
        let account1 = ServiceAccount::new(
            "test1@example.com".to_string(),
            "testuser1".to_string(),
            "secret1".to_string(),
        );
        let account2 = ServiceAccount::new(
            "test2@example.com".to_string(),
            "testuser2".to_string(),
            "secret2".to_string(),
        );

        service.create(account1).await?;
        let created2 = service.create(account2).await?;

        let filter = ServiceAccountFilter {
            email: Some("test2@example.com".to_string()),
            user: None,
            is_enabled: None,
        };

        let found = service.find(filter, None, None).await?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, created2.id);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_service_accounts_with_pagination() -> Result<(), Error> {
        let (service, db) = setup().await;

        // Create 5 test accounts
        for i in 1..=5 {
            let account = ServiceAccount::new(
                format!("test{}@example.com", i),
                format!("testuser{}", i),
                format!("secret{}", i),
            );
            service.create(account).await?;
        }

        // Test first page
        let pagination = Pagination {
            page: Some(1),
            limit: Some(2),
        };
        let found = service
            .find(
                ServiceAccountFilter {
                    email: None,
                    user: None,
                    is_enabled: None,
                },
                None,
                Some(pagination),
            )
            .await?;
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].user, "testuser1");
        assert_eq!(found[1].user, "testuser2");

        // Test second page
        let pagination = Pagination {
            page: Some(2),
            limit: Some(2),
        };
        let found = service
            .find(
                ServiceAccountFilter {
                    email: None,
                    user: None,
                    is_enabled: None,
                },
                None,
                Some(pagination),
            )
            .await?;
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].user, "testuser3");
        assert_eq!(found[1].user, "testuser4");

        cleanup_test_db(db).await?;
        Ok(())
    }
}
