use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a service account for API authentication
///
/// # Fields
/// - `id`: Unique identifier for the service account (MongoDB ObjectId)
/// - `email`: Email address associated with the account
/// - `user`: Username for the account
/// - `secret`: Secret key for authentication
/// - `enabled`: Whether the account is currently active
/// - `created_at`: Account creation timestamp
/// - `updated_at`: Last update timestamp
#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct ServiceAccount {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    email: String,
    user: String,
    secret: String,
    enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ServiceAccount {
    pub fn new(email: String, user: String, secret: String) -> Self {
        Self {
            id: None,
            email,
            user,
            secret,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn id(&self) -> Option<&ObjectId> {
        self.id.as_ref()
    }

    /// Sets the account's unique identifier
    pub fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
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

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    // Convert to MongoDB Document
    pub fn to_document(&self) -> Result<mongodb::bson::Document, mongodb::bson::ser::Error> {
        mongodb::bson::to_document(self)
    }

    // Create from MongoDB Document
    pub fn from_document(doc: mongodb::bson::Document) -> Result<Self, mongodb::bson::de::Error> {
        mongodb::bson::from_document(doc)
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

        assert!(service_account.id().is_none());
        assert_eq!(service_account.email(), email);
        assert_eq!(service_account.user(), user);
        assert_eq!(service_account.secret(), secret);
        assert!(service_account.enabled());
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

        assert_eq!(service_account.id(), deserialized_service_account.id());
        assert_eq!(service_account.email(), deserialized_service_account.email());
        assert_eq!(service_account.user(), deserialized_service_account.user());
        assert_eq!(service_account.secret(), deserialized_service_account.secret());
        assert_eq!(
            service_account.enabled(),
            deserialized_service_account.enabled()
        );
    }

    #[test]
    fn test_mongodb_serialization() {
        let email = "Examples@google.com".to_string();
        let user = "Example123".to_string();
        let secret = "ExampleSercet".to_string();
        let service_account = ServiceAccount::new(email.clone(), user.clone(), secret.clone());

        // Test conversion to BSON Document
        let doc = service_account.to_document().unwrap();
        
        // Test conversion from BSON Document
        let deserialized = ServiceAccount::from_document(doc).unwrap();

        assert_eq!(service_account.id(), deserialized.id());
        assert_eq!(service_account.email(), deserialized.email());
        assert_eq!(service_account.user(), deserialized.user());
        assert_eq!(service_account.secret(), deserialized.secret());
        assert_eq!(service_account.enabled(), deserialized.enabled());
    }
}
