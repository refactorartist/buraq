use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

//service_account {
//    ObjectID id PK "Primary Key"
//    string email "Account email"
//    string user "Username"
//    string secret "Account secret"
//    bool enabled "Account status"
//}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceAccount {
    id: ObjectId,
    email: String,
    user: String,
    secret: String,
    enabled: bool,
}

impl ServiceAccount {
    pub fn new(email: String, user: String, secret: String) -> Self {
        Self {
            id: ObjectId::new(),
            email,
            user,
            secret,
            enabled: true,
        }
    }

    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_new_service_account() {
        let email = "Examples@google.com".to_string();
        let user = "Example123".to_string();
        let secret = "ExampleSercet".to_string();

        let service_account = ServiceAccount::new(email.clone(), user.clone(), secret.clone());

        assert!(ObjectId::parse_str(service_account.id.to_string()).is_ok());
        assert_eq!(service_account.email, email);
        assert_eq!(service_account.user, user);
        assert_eq!(service_account.secret, secret);
    }
    #[test]
    fn test_serialization() {
        let email = "Examples@google.com".to_string();
        let user = "Example123".to_string();
        let secret = "ExampleSercet".to_string();
        let service_account = ServiceAccount::new(email.clone(), user.clone(), secret.clone());

        let serialized = serde_json::to_string(&service_account);
        let deserialized: Result<ServiceAccount, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
        let deserialized_service_account = deserialized.unwrap();

        assert_eq!(service_account.id, deserialized_service_account.id);
        assert_eq!(service_account.email, deserialized_service_account.email);
        assert_eq!(service_account.user, deserialized_service_account.user);
        assert_eq!(service_account.secret, deserialized_service_account.secret);
        assert_eq!(
            service_account.enabled,
            deserialized_service_account.enabled
        );
    }
}
