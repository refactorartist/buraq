use crate::serializers::algorithm;
use chrono::{DateTime, Utc};
use jsonwebtoken::Algorithm;
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
/// - `project_access_id`: Identifier for the project access
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessToken {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub project_access_id: Uuid,
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

/// Represents an access token for API reading (without the key field)
///
/// This is a read-only version of AccessToken that doesn't include the sensitive key field.
/// Use this when returning token information to clients where the key shouldn't be exposed.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenRead {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub project_access_id: Uuid,
    #[serde(with = "algorithm")]
    pub algorithm: Algorithm,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub enabled: bool,
}

impl From<AccessToken> for AccessTokenRead {
    fn from(token: AccessToken) -> Self {
        AccessTokenRead {
            id: token.id,
            project_access_id: token.project_access_id,
            algorithm: token.algorithm,
            expires_at: token.expires_at,
            created_at: token.created_at,
            enabled: token.enabled,
        }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_access_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenCreatePayload {
    #[serde(with = "algorithm")]
    pub algorithm: Algorithm,
    pub expires_at: DateTime<Utc>,
    pub project_access_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AccessTokenFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<Algorithm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_access_id: Option<Uuid>,
}

impl From<AccessTokenFilter> for Document {
    fn from(value: AccessTokenFilter) -> Self {
        let mut doc = Document::new();
        if let Some(key) = value.key {
            doc.insert("key", key);
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
        if let Some(project_access_id) = value.project_access_id {
            doc.insert("project_access_id", project_access_id);
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
    ProjectAccessId,
}

impl From<AccessTokenSortableFields> for String {
    fn from(value: AccessTokenSortableFields) -> Self {
        match value {
            AccessTokenSortableFields::Id => "id".to_string(),
            AccessTokenSortableFields::Key => "key".to_string(),
            AccessTokenSortableFields::Algorithm => "algorithm".to_string(),
            AccessTokenSortableFields::ExpiresAt => "expires_at".to_string(),
            AccessTokenSortableFields::CreatedAt => "created_at".to_string(),
            AccessTokenSortableFields::ProjectAccessId => "project_access_id".to_string(),
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
            algorithm: Algorithm::RS256,
            expires_at: expires,
            created_at: now,
            enabled: true,
            project_access_id: Uuid::new(),
        };

        assert_eq!(token.key, "test-key");
        assert_eq!(token.algorithm, Algorithm::RS256);
        assert!(token.enabled);
    }

    #[test]
    fn test_access_token_document_conversion() {
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::HS256,
            expires_at: Utc::now(),
            created_at: Utc::now(),
            enabled: true,
            project_access_id: Uuid::new(),
        };

        let doc: Document = token.clone().into();
        let converted: AccessToken = doc.into();

        assert_eq!(token.id, converted.id);
        assert_eq!(token.key, converted.key);
        assert_eq!(token.algorithm, converted.algorithm);
        assert_eq!(token.enabled, converted.enabled);
        assert_eq!(token.project_access_id, converted.project_access_id);
    }

    #[test]
    fn test_access_token_document_conversion_error() {
        let doc = doc! {
            "key": "test-key",
            "expires_at": Utc::now().to_rfc3339(),
            "created_at": Utc::now().to_rfc3339(),
            "algorithm": "InvalidAlgorithm", // Invalid algorithm
            "enabled": true,
            "project_access_id": Uuid::new(),
        };

        let result: Result<AccessToken, _> = from_document(doc.clone());
        assert!(
            result.is_err(),
            "Expected conversion to fail due to invalid algorithm"
        );
    }

    #[test]
    fn test_access_token_update_payload() {
        let update = AccessTokenUpdatePayload {
            key: Some("new-key".to_string()),
            expires_at: Some(Utc::now()),
            enabled: Some(false),
            project_access_id: Some(Uuid::new()),
        };

        assert_eq!(update.key.unwrap(), "new-key");
        assert!(!update.enabled.unwrap());
    }

    #[test]
    fn test_access_token_filter() {
        let filter = AccessTokenFilter {
            key: Some("test-key".to_string()),
            algorithm: Some(Algorithm::RS256),
            is_enabled: Some(true),
            is_active: Some(true),
            project_access_id: Some(Uuid::new()),
        };

        let doc: Document = filter.into();

        assert_eq!(doc.get_str("key").unwrap(), "test-key");
        assert_eq!(doc.get_str("algorithm").unwrap(), "RS256");
        assert!(doc.get_bool("enabled").unwrap());
        assert!(doc.get_document("expires_at").unwrap().contains_key("$gt"));
        assert!(doc.contains_key("project_access_id"));
    }

    #[test]
    fn test_access_token_create_payload() {
        // Test creation and basic properties
        let project_id = Uuid::new();
        let expires_at = Utc::now() + Duration::hours(1);

        let payload = AccessTokenCreatePayload {
            algorithm: Algorithm::HS256,
            expires_at,
            project_access_id: project_id,
        };

        assert_eq!(payload.algorithm, Algorithm::HS256);
        assert_eq!(payload.expires_at, expires_at);
        assert_eq!(payload.project_access_id, project_id);
    }

    #[test]
    fn test_access_token_create_payload_serialization() {
        // Test serialization to JSON
        let project_id = Uuid::new();
        let expires_at = Utc::now() + Duration::hours(1);

        let payload = AccessTokenCreatePayload {
            algorithm: Algorithm::RS256,
            expires_at,
            project_access_id: project_id,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains(&"\"algorithm\":\"RS256\"".to_string()));
        assert!(json.contains(&format!("\"project_access_id\":\"{}\"", project_id)));
    }

    #[test]
    fn test_access_token_create_payload_deserialization() {
        // Test deserialization from JSON
        let project_id = Uuid::new();
        let expires_at = Utc::now() + Duration::hours(1);
        let expires_at_str = expires_at.to_rfc3339();

        let json = format!(
            r#"
            {{
                "algorithm": "HS512",
                "expires_at": "{expires_at_str}",
                "project_access_id": "{project_id}"
            }}
            "#
        );

        let payload: AccessTokenCreatePayload = serde_json::from_str(&json).unwrap();

        assert_eq!(payload.algorithm, Algorithm::HS512);
        assert_eq!(payload.project_access_id, project_id);
        assert_eq!(payload.expires_at.to_rfc3339(), expires_at_str);
    }

    #[test]
    fn test_access_token_create_payload_invalid_algorithm() {
        // Test handling of invalid algorithm during deserialization
        let project_id = Uuid::new();
        let expires_at = (Utc::now() + Duration::hours(1)).to_rfc3339();

        let json = format!(
            r#"
            {{
                "algorithm": "INVALID_ALGO",
                "expires_at": "{expires_at}",
                "project_access_id": "{project_id}"
            }}
            "#
        );

        let result: Result<AccessTokenCreatePayload, _> = serde_json::from_str(&json);
        assert!(result.is_err(), "Should fail with invalid algorithm");
    }

    #[test]
    fn test_access_token_read_conversion() {
        let now = Utc::now();
        let expires = now + Duration::hours(1);
        let token = AccessToken {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            algorithm: Algorithm::RS256,
            expires_at: expires,
            created_at: now,
            enabled: true,
            project_access_id: Uuid::new(),
        };

        let read_token = AccessTokenRead::from(token.clone());

        assert_eq!(read_token.id, token.id);
        assert_eq!(read_token.project_access_id, token.project_access_id);
        assert_eq!(read_token.algorithm, token.algorithm);
        assert_eq!(read_token.expires_at, token.expires_at);
        assert_eq!(read_token.created_at, token.created_at);
        assert_eq!(read_token.enabled, token.enabled);
    }

    #[test]
    fn test_access_token_read_serialization() {
        let now = Utc::now();
        let read_token = AccessTokenRead {
            id: Some(Uuid::new()),
            project_access_id: Uuid::new(),
            algorithm: Algorithm::HS256,
            expires_at: now + Duration::hours(1),
            created_at: now,
            enabled: true,
        };

        let serialized = serde_json::to_string(&read_token).unwrap();
        let deserialized: AccessTokenRead = serde_json::from_str(&serialized).unwrap();

        assert_eq!(read_token.id, deserialized.id);
        assert_eq!(read_token.project_access_id, deserialized.project_access_id);
        assert_eq!(read_token.algorithm, deserialized.algorithm);
        assert_eq!(read_token.expires_at, deserialized.expires_at);
        assert_eq!(read_token.created_at, deserialized.created_at);
        assert_eq!(read_token.enabled, deserialized.enabled);
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
        assert_eq!(
            String::from(AccessTokenSortableFields::ProjectAccessId),
            "project_access_id"
        );
    }
}
