use crate::models::service_account::{ServiceAccount, ServiceAccountUpdatePayload};
use crate::repositories::base::Repository;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, doc, to_document};
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

    async fn create(&self, mut item: ServiceAccount) -> Result<ServiceAccount, Error> {
        if item.id.is_none() {
            item.id = Some(Uuid::new());
        }
        self.collection
            .insert_one(&item)
            .await
            .expect("Failed to create service account");
        Ok(item)
    }

    async fn read(&self, id: Uuid) -> Result<Option<ServiceAccount>, Error> {
        let result = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?;
        Ok(result)
    }

    async fn replace(&self, id: Uuid, mut item: ServiceAccount) -> Result<ServiceAccount, Error> {
        if item.id.is_none() || item.id.unwrap() != id {
            item.id = Some(id);
        }
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": to_document(&item)? })
            .await
            .expect("Failed to update service account");
        let updated = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?
            .unwrap();
        Ok(updated)
    }

    async fn update(&self, id: Uuid, item: Self::UpdatePayload) -> Result<ServiceAccount, Error> {
        let document = to_document(&item)?;
        self.collection
            .update_one(
                mongodb::bson::doc! { "_id": id },
                mongodb::bson::doc! { "$set": document },
            )
            .await?;
        let updated = self
            .collection
            .find_one(mongodb::bson::doc! { "_id": id })
            .await?
            .unwrap();
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = self
            .collection
            .delete_one(mongodb::bson::doc! { "_id": id })
            .await?;
        Ok(result.deleted_count > 0)
    }

    async fn find(&self, filter: Document) -> Result<Vec<ServiceAccount>, Error> {
        let result = self.collection.find(filter).await?;
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

        let found = repo.find(doc! { "enabled": true }).await?;
        assert_eq!(found.len(), 2);

        cleanup_test_db(db).await?;
        Ok(())
    }
}
