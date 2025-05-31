use crate::serializers::algorithm;
use jsonwebtoken::Algorithm;
use chrono::{DateTime, Utc};
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, doc, from_document, to_document};
use serde::{Deserialize, Serialize};

/// Represents a service account key for API authentication
///
/// # Fields
/// - `id`: Unique identifier for the service account key (MongoDB UUID)
/// - `service_account_id`: Foreign key reference to the associated service account
/// - `algorithm`: The algorithm used for the key
/// - `key`: The actual key value
/// - `expires_at`: Key expiration timestamp
/// - `created_at`: Timestamp when key was created
/// - `updated_at`: Timestamp when key was last updated
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAccountKey {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub service_account_id: Uuid,
    #[serde(with = "algorithm")]
    pub algorithm: Algorithm,
    pub key: String,
    pub expires_at: DateTime<Utc>,
    pub enabled: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<ServiceAccountKey> for Document {
    fn from(value: ServiceAccountKey) -> Self {
        to_document(&value).unwrap()
    }
}

impl From<Document> for ServiceAccountKey {
    fn from(value: Document) -> Self {
        from_document(value.clone()).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAccountKeyUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServiceAccountKeyFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_account_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<Algorithm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

impl From<ServiceAccountKeyFilter> for Document {
    fn from(value: ServiceAccountKeyFilter) -> Self {
        let mut doc = Document::new();
        if let Some(service_account_id) = value.service_account_id {
            doc.insert("service_account_id", service_account_id);
        }
        if let Some(algorithm) = value.algorithm {
            doc.insert("algorithm", format!("{:?}", algorithm));
        }
        if let Some(is_enabled) = value.is_enabled {
            doc.insert("enabled", is_enabled);
        }
        if let Some(is_active) = value.is_active {
            if is_active {
                doc.insert("expires_at", doc! { "$gt": mongodb::bson::DateTime::now() });
            }
        }
        doc
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServiceAccountKeySortableFields {
    Id,
    ServiceAccountId,
    Algorithm,
    ExpiresAt,
    UpdatedAt,
    CreatedAt,
}

impl From<ServiceAccountKeySortableFields> for String {
    fn from(value: ServiceAccountKeySortableFields) -> Self {
        match value {
            ServiceAccountKeySortableFields::Id => "id".to_string(),
            ServiceAccountKeySortableFields::ServiceAccountId => "service_account_id".to_string(),
            ServiceAccountKeySortableFields::Algorithm => "algorithm".to_string(),
            ServiceAccountKeySortableFields::ExpiresAt => "expires_at".to_string(),
            ServiceAccountKeySortableFields::UpdatedAt => "updated_at".to_string(),
            ServiceAccountKeySortableFields::CreatedAt => "created_at".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_service_account_key() {
        let service_account_id = Uuid::new();
        let algorithm = Algorithm::RS256;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        let service_account_key = ServiceAccountKey {
            id: None,
            service_account_id,
            algorithm,
            key: key.clone(),
            expires_at,
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        assert!(service_account_key.id.is_none());
        assert_eq!(service_account_key.service_account_id, service_account_id);
        assert!(matches!(service_account_key.algorithm, Algorithm::RS256));
        assert_eq!(service_account_key.key, key);
        assert_eq!(service_account_key.expires_at, expires_at);
        assert!(service_account_key.enabled);
        let now = Utc::now();
        assert!(service_account_key.created_at.unwrap() <= now);
        assert!(service_account_key.updated_at.unwrap() <= now);
    }

    #[test]
    fn test_serialization() {
        let service_account_id = Uuid::new();
        let algorithm = Algorithm::RS256;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        let service_account_key = ServiceAccountKey {
            id: None,
            service_account_id,
            algorithm,
            key: key.clone(),
            expires_at,
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        // Test serialization
        let serialized = serde_json::to_string(&service_account_key);
        assert!(serialized.is_ok());

        // Test deserialization
        let deserialized: Result<ServiceAccountKey, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());

        let deserialized_key = deserialized.unwrap();
        assert_eq!(service_account_key.id, deserialized_key.id);
        assert_eq!(
            service_account_key.service_account_id,
            deserialized_key.service_account_id
        );
        assert_eq!(service_account_key.algorithm, deserialized_key.algorithm);
        assert_eq!(service_account_key.key, deserialized_key.key);
        assert_eq!(service_account_key.expires_at, deserialized_key.expires_at);
        assert_eq!(service_account_key.enabled, deserialized_key.enabled);
        assert_eq!(service_account_key.created_at, deserialized_key.created_at);
        assert_eq!(service_account_key.updated_at, deserialized_key.updated_at);
    }

    #[test]
    fn test_mongodb_serialization() {
        let service_account_id = Uuid::new();
        let algorithm = Algorithm::RS256;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        let mut service_account_key = ServiceAccountKey {
            id: None,
            service_account_id,
            algorithm,
            key: key.clone(),
            expires_at,
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let id = Uuid::new();
        service_account_key.id = Some(id);

        let doc: Document = service_account_key.clone().into();
        let converted: ServiceAccountKey = doc.into();

        assert_eq!(converted.id, service_account_key.id);
        assert_eq!(
            converted.service_account_id,
            service_account_key.service_account_id
        );
        assert_eq!(converted.algorithm, service_account_key.algorithm);
        assert_eq!(converted.key, service_account_key.key);
        assert_eq!(converted.expires_at, service_account_key.expires_at);
        assert_eq!(converted.enabled, service_account_key.enabled);
        assert_eq!(converted.created_at, service_account_key.created_at);
        assert_eq!(converted.updated_at, service_account_key.updated_at);
    }
}
