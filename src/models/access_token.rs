use crate::serializers::algorithm;
use crate::types::Algorithm;
use chrono::{DateTime, Utc};
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, doc, from_document, to_document};
use serde::{Deserialize, Serialize};

/// Represents an access token for API authentication
///
/// # Fields
/// - `id`: Unique identifier for the access token (MongoDB ObjectId)
/// - `key`: The actual API key value
/// - `algorithm`: The algorithm used for the token
/// - `expires_at`: Token expiration timestamp
/// - `created_at`: Token creation timestamp
/// - `enabled`: Whether the token is currently active
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessToken {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub key: String,
    #[serde(with = "algorithm")]
    pub algorithm: Algorithm,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub enabled: bool,
}

impl From<AccessToken> for Document {
    fn from(value: AccessToken) -> Self {
        to_document(&value).expect("Failed to convert AccessToken to Document")
    }
}

impl From<Document> for AccessToken {
    fn from(value: Document) -> Self {
        from_document(value.clone()).expect("Failed to convert Document to AccessToken")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone,Default)]
pub struct AccessTokenFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<Algorithm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

impl From<AccessTokenFilter> for Document {
    fn from(value: AccessTokenFilter) -> Self {
        let mut doc = Document::new();
        if let Some(key) = value.key {
            doc.insert("key", key);
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AccessTokenSortableFields {
    Id,
    Key,
    Algorithm,
    ExpiresAt,
    CreatedAt,
}

impl From<AccessTokenSortableFields> for String {
    fn from(value: AccessTokenSortableFields) -> Self {
        match value {
            AccessTokenSortableFields::Id => "id".to_string(),
            AccessTokenSortableFields::Key => "key".to_string(),
            AccessTokenSortableFields::Algorithm => "algorithm".to_string(),
            AccessTokenSortableFields::ExpiresAt => "expires_at".to_string(),
            AccessTokenSortableFields::CreatedAt => "created_at".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use mongodb::bson::doc;

    #[test]
    fn test_access_token_creation() {
        let now = Utc::now();
        let expires = now + Duration::hours(1);

        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: expires,
            created_at: now,
            enabled: true,
        };

        assert_eq!(token.key, "test-key");
        assert_eq!(token.algorithm, Algorithm::RSA);
        assert!(token.enabled);
    }

    #[test]
    fn test_access_token_document_conversion() {
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::HMAC,
            expires_at: Utc::now(),
            created_at: Utc::now(),
            enabled: true,
        };

        let doc: Document = token.clone().into();
        let converted: AccessToken = doc.into();

        assert_eq!(token.id, converted.id);
        assert_eq!(token.key, converted.key);
        assert_eq!(token.algorithm, converted.algorithm);
        assert_eq!(token.enabled, converted.enabled);
    }

    #[test]
    fn test_access_token_document_conversion_error() {
        let doc = doc! {
            "key": "test-key",
            "expires_at": Utc::now().to_rfc3339(),
            "created_at": Utc::now().to_rfc3339(),
            "algorithm": "InvalidAlgorithm", // Invalid algorithm
            "enabled": true,
        };

        let result: Result<AccessToken, _> = from_document(doc.clone());
        assert!(result.is_err(), "Expected conversion to fail due to invalid algorithm");
    }

    #[test]
    fn test_access_token_update_payload() {
        let update = AccessTokenUpdatePayload {
            key: Some("new-key".to_string()),
            expires_at: Some(Utc::now()),
            enabled: Some(false),
        };

        assert_eq!(update.key.unwrap(), "new-key");
        assert!(!update.enabled.unwrap());
    }

    #[test]
    fn test_access_token_filter() {
        let filter = AccessTokenFilter {
            key: Some("test-key".to_string()),
            algorithm: Some(Algorithm::RSA),
            is_enabled: Some(true),
            is_active: Some(true),
        };

        let doc: Document = filter.into();

        assert_eq!(doc.get_str("key").unwrap(), "test-key");
        assert_eq!(doc.get_str("algorithm").unwrap(), "RSA");
        assert!(doc.get_bool("enabled").unwrap());
        assert!(doc.get_document("expires_at").unwrap().contains_key("$gt"));
    }

    #[test]
    fn test_access_token_sortable_fields() {
        assert_eq!(String::from(AccessTokenSortableFields::Id), "id");
        assert_eq!(String::from(AccessTokenSortableFields::Key), "key");
        assert_eq!(
            String::from(AccessTokenSortableFields::Algorithm),
            "algorithm"
        );
        assert_eq!(
            String::from(AccessTokenSortableFields::ExpiresAt),
            "expires_at"
        );
        assert_eq!(
            String::from(AccessTokenSortableFields::CreatedAt),
            "created_at"
        );
    }
}
