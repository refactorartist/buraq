use crate::models::pagination::Pagination;
use crate::models::service_account::{ServiceAccount, ServiceAccountUpdatePayload, ServiceAccountFilter, ServiceAccountSortableFields};
use crate::models::sort::SortBuilder;
use crate::repositories::base::Repository;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Bson, Document, doc, to_document};
use mongodb::{Collection, Database};

/// Repository for managing ServiceAccount documents in MongoDB.
///
/// Provides CRUD operations for ServiceAccount entities.
pub struct ServiceAccountRepository {
    collection: Collection<ServiceAccount>,
}

impl ServiceAccountRepository {
    pub fn new(database: Database) -> Result<Self, anyhow::Error> {
        let collection = database.collection::<ServiceAccount>("service_accounts");
        Ok(Self { collection })
    }
}

#[async_trait]
impl Repository<ServiceAccount> for ServiceAccountRepository {
    type UpdatePayload = ServiceAccountUpdatePayload;
    type Filter = ServiceAccountFilter;
    type Sort = ServiceAccountSortableFields;

    async fn create(&self, mut item: ServiceAccount) -> Result<ServiceAccount, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        item.created_at = Some(Utc::now());
        item.updated_at = Some(Utc::now());
        self.collection.insert_one(&item).await?;
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<ServiceAccount>, Error> {
        let result = self.collection.find_one(doc! { "_id": id }).await?;
        Ok(result)
    }

    async fn replace(&self, id: Uuid, mut item: ServiceAccount) -> Result<ServiceAccount, Error> {
        if item.id.is_none() || item.id.unwrap() != id {
            item.id = Some(id);
        }
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": to_document(&item)? })
            .await?;
        let updated = self.collection.find_one(doc! { "_id": id }).await?.unwrap();
        Ok(updated)
    }

    async fn update(&self, id: Uuid, payload: Self::UpdatePayload) -> Result<ServiceAccount, Error> {
        let mut document = to_document(&payload)?;
        document.insert("updated_at", Bson::String(Utc::now().to_rfc3339()));

        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": document })
            .await?;
        let updated = self
            .read(id)
            .await?
            .ok_or_else(|| Error::msg("ServiceAccount not found"))?;
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(&self, filter: Self::Filter, sort: Option<SortBuilder<Self::Sort>>, pagination: Option<Pagination>) -> Result<Vec<ServiceAccount>, Error> {
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
        let items: Vec<ServiceAccount> = result.try_collect().await?;
        Ok(items)
    }

    fn collection(&self) -> Result<Collection<ServiceAccount>, Error> {
        Ok(self.collection.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};

    async fn setup() -> (ServiceAccountRepository, Database) {
        let db = setup_test_db("service_account").await.unwrap();
        let repo = ServiceAccountRepository::new(db.clone()).expect("Failed to create repository");
        (repo, db)
    }

    #[tokio::test]
    async fn test_create_service_account() -> Result<()> {
        let (repo, db) = setup().await;
        let service_account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created = repo.create(service_account.clone()).await?;
        assert!(created.id.is_some());
        assert_eq!(created.email, service_account.email);
        assert_eq!(created.user, service_account.user);
        assert_eq!(created.secret, service_account.secret);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_service_account() -> Result<()> {
        let (repo, db) = setup().await;
        let service_account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created = repo.create(service_account.clone()).await?;
        let read = repo.read(created.id.unwrap()).await?.unwrap();
        assert_eq!(read.id, created.id);
        assert_eq!(read.email, created.email);
        assert_eq!(read.user, created.user);
        assert_eq!(read.secret, created.secret);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_service_account() -> Result<()> {
        let (repo, db) = setup().await;
        let service_account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created = repo.create(service_account).await?;
        let update = ServiceAccountUpdatePayload {
            email: Some("new@example.com".to_string()),
            user: Some("newuser".to_string()),
            secret: Some("newsecret".to_string()),
            enabled: Some(false),
        };

        let updated = repo.update(created.id.unwrap(), update).await?;
        assert_eq!(updated.email, "new@example.com");
        assert_eq!(updated.user, "newuser");
        assert_eq!(updated.secret, "newsecret");
        assert!(!updated.enabled);

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_service_account() -> Result<()> {
        let (repo, db) = setup().await;
        let service_account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created = repo.create(service_account).await?;
        let deleted = repo.delete(created.id.unwrap()).await?;
        assert!(deleted);

        let read = repo.read(created.id.unwrap()).await?;
        assert!(read.is_none());

        cleanup_test_db(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_service_accounts() -> Result<()> {
        let (repo, db) = setup().await;
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

        repo.create(account1).await?;
        repo.create(account2).await?;

        // Test finding all accounts
        let filter = ServiceAccountFilter {
            email: None,
            user: None,
            is_enabled: None,
        };
        let all_accounts = repo.find(filter, None, None).await?;
        assert_eq!(all_accounts.len(), 2);

        // Test finding by email
        let email_filter = ServiceAccountFilter {
            email: Some("test1@example.com".to_string()),
            user: None,
            is_enabled: None,
        };
        let accounts = repo.find(email_filter, None, None).await?;
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].email, "test1@example.com");

        // Test finding by enabled status
        let enabled_filter = ServiceAccountFilter {
            email: None,
            user: None,
            is_enabled: Some(true),
        };
        let enabled_accounts = repo.find(enabled_filter, None, None).await?;
        assert_eq!(enabled_accounts.len(), 2);

        cleanup_test_db(db).await?;
        Ok(())
    }
}
