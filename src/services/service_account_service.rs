use crate::{models::service_account::{self, ServiceAccount}, repositories::service_account::ServiceAccountRepository};
use mongodb::Database;
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
    pub async  fn new(db: Database) -> Self {
        let service_account_service = ServiceAccountRepository::new(db).unwrap();
        Self {
            service_account_repository: service_account_service,
        }
    }


    pub async fn create_service_account_service(&self,service_account:ServiceAccount) -> Result<ServiceAccount,Error> {
        let 
        

    }






}




