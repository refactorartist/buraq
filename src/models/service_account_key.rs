use crate::serializers::algorithm;
use crate::types::Algorithm;
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAccountKey {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub service_account_id: Uuid,
    #[serde(with = "algorithm")]
    pub algorithm: Algorithm,
    pub key: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub enabled: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            doc.insert("algorithm", algorithm.to_string());
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_service_account_key() {
        let service_account_id = Uuid::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        let service_account_key = ServiceAccountKey {
            id: None,
            service_account_id,
            algorithm,
            key: key.clone(),
            expires_at,
            created_at: Utc::now(),
            enabled: true,
        };

        assert!(service_account_key.id.is_none());
        assert_eq!(service_account_key.service_account_id, service_account_id);
        assert!(matches!(service_account_key.algorithm, Algorithm::RSA));
        assert_eq!(service_account_key.key, key);
        assert_eq!(service_account_key.expires_at, expires_at);
        assert!(service_account_key.enabled);
    }

    #[test]
    fn test_mongodb_serialization() {
        let service_account_id = Uuid::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        let service_account_key = ServiceAccountKey {
            id: None,
            service_account_id,
            algorithm,
            key: key.clone(),
            expires_at,
            created_at: Utc::now(),
            enabled: true,
        };

        let doc: Document = service_account_key.clone().into();
        let converted: ServiceAccountKey = doc.into();

        assert_eq!(service_account_key.id, converted.id);
        assert_eq!(
            service_account_key.service_account_id,
            converted.service_account_id
        );
        assert_eq!(service_account_key.algorithm, converted.algorithm);
        assert_eq!(service_account_key.key, converted.key);
        assert_eq!(service_account_key.expires_at, converted.expires_at);
        assert_eq!(service_account_key.created_at, converted.created_at);
        assert_eq!(service_account_key.enabled, converted.enabled);
    }

    #[test]
    fn test_algorithm_serialization() {
        let service_account_id = Uuid::new();
        let key = "test-key-value".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        // Test RSA algorithm
        let rsa_key = ServiceAccountKey {
            id: None,
            service_account_id,
            algorithm: Algorithm::RSA,
            key: key.clone(),
            expires_at,
            created_at: Utc::now(),
            enabled: true,
        };
        let doc = Document::from(rsa_key);
        assert_eq!(doc.get_str("algorithm").unwrap(), "RSA");

        // Test HMAC algorithm
        let hmac_key = ServiceAccountKey {
            id: None,
            service_account_id,
            algorithm: Algorithm::HMAC,
            key: key.clone(),
            expires_at,
            created_at: Utc::now(),
            enabled: true,
        };
        let doc = Document::from(hmac_key);
        assert_eq!(doc.get_str("algorithm").unwrap(), "HMAC");
    }

    #[test]
    fn test_expiration() {
        let service_account_id = Uuid::new();
        let expired_time = Utc::now() + chrono::Duration::hours(1);
        let key = ServiceAccountKey {
            id: None,
            service_account_id,
            algorithm: Algorithm::HMAC,
            key: "test-key".to_string(),
            expires_at: expired_time,
            created_at: Utc::now(),
            enabled: true,
        };

        assert!(key.expires_at > Utc::now());
    }
}
