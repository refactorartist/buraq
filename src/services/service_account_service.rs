use crate::{
    models::service_account::{self, ServiceAccount},
    repositories::{base::Repository, service_account::ServiceAccountRepository},
};
use mongodb::{Database, error::Error};
use serde::{Deserialize, Serialize};

pub struct ServiceAccountService {
    service_account_repository: ServiceAccountRepository,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceAccountFilter {
    email: Option<String>,
    user: Option<String>,
    enable: Option<bool>,
}

impl Into<mongodb::bson::Document> for ServiceAccountFilter {
    fn into(self) -> mongodb::bson::Document {
        let mut doc = mongodb::bson::Document::new();

        if let Some(email) = self.email {
            doc.insert("email", email);
        }

        if let Some(user) = self.user {
            doc.insert("user", user);
        }

        if let Some(enable) = self.enable {
            doc.insert("enable", enable);
        }

        doc
    }
}

impl ServiceAccountService {
    pub async fn new(db: Database) -> Self {
        let service_account_service = ServiceAccountRepository::new(db).unwrap();
        Self {
            service_account_repository: service_account_service,
        }
    }

    pub async fn create_service_account_service(
        &self,
        service_account: ServiceAccount,
    ) -> Result<ServiceAccount, Error> {
        let result = self
            .service_account_repository
            .create(service_account.clone())
            .await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let insert_service_account = self
            .service_account_repository
            .read(id)
            .await?
            .ok_or_else(|| Error::msg("Failed to fetch created service_account"));

        Ok(insert_service_account)
    }
}

#[cfg(test)]

mod tests {

    use super::*;
    use crate::{
        models::service_account,
        services::service_account_service,
        test_utils::{cleanup_test_db, setup_test_db},
    };
    use tokio;

    async fn setup_service_account_for_filter_tests(
        service_account_service: &ServiceAccountService,
    ) -> Result<(), Error> {
    let collation = service_account_service.service_account_repository.collection()?;
    let db = collation.client().database(&collation.namespace().db);
    cleanup_test_db(db).await?;


    let service_account1 = ServiceAccount::new()


    }
}
