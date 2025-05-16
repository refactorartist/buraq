use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, to_document, from_document, doc};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a service account for API authentication
///
/// # Fields
/// - `id`: Unique identifier for the service account (UUID)
/// - `email`: Email address associated with the account
/// - `user`: Username for the account
/// - `secret`: Secret key for authentication
/// - `enabled`: Whether the account is currently active
/// - `created_at`: Account creation timestamp
/// - `updated_at`: Last update timestamp
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAccount {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub email: String,
    pub user: String,
    pub secret: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
}

impl From<ServiceAccount> for Document {
    fn from(value: ServiceAccount) -> Self {
        to_document(&value).unwrap()
    }
}

impl From<Document> for ServiceAccount {
    fn from(value: Document) -> Self {
        from_document(value.clone()).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAccountUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAccountFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
}

impl From<ServiceAccountFilter> for Document {
    fn from(value: ServiceAccountFilter) -> Self {
        let mut doc = Document::new();
        if let Some(email) = value.email {
            doc.insert("email", email);
        }
        if let Some(user) = value.user {
            doc.insert("user", user);
        }
        if let Some(is_enabled) = value.is_enabled {
            doc.insert("enabled", is_enabled);
        }
        doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_account_creation() {
        let account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string()
        );

        assert_eq!(account.email, "test@example.com");
        assert_eq!(account.user, "testuser");
        assert_eq!(account.secret, "secret123");
        assert!(account.enabled);
    }

    #[test]
    fn test_service_account_document_conversion() {
        let account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string()
        );

        let doc: Document = account.clone().into();
        let converted: ServiceAccount = doc.into();

        assert_eq!(account.id, converted.id);
        assert_eq!(account.email, converted.email);
        assert_eq!(account.user, converted.user);
        assert_eq!(account.secret, converted.secret);
        assert_eq!(account.enabled, converted.enabled);
    }

    #[test]
    fn test_service_account_filter() {
        let filter = ServiceAccountFilter {
            email: Some("test@example.com".to_string()),
            user: Some("testuser".to_string()),
            is_enabled: Some(true),
        };

        let doc: Document = filter.into();
        
        assert_eq!(doc.get_str("email").unwrap(), "test@example.com");
        assert_eq!(doc.get_str("user").unwrap(), "testuser");
        assert!(doc.get_bool("enabled").unwrap());
    }
}
