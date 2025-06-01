use chrono::{DateTime, Utc};
use jsonwebtoken::Algorithm;
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, doc, from_document, to_document};
use serde::{Deserialize, Serialize};

/// Represents a server key for API authentication
///
/// # Fields
/// - `id`: Unique identifier for the server key (MongoDB Uuid)
/// - `key`: The actual API key value
/// - `environment_id`: Foreign key reference to the associated environment
/// - `algorithm`: The algorithm used for the key
/// - `created_at`: Key creation timestamp
/// - `updated_at`: Timestamp when key was last updated
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerKey {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub key: String,
    pub environment_id: Uuid,
    #[serde(with = "crate::serializers::algorithm")]
    pub algorithm: Algorithm,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ServerKey> for Document {
    fn from(value: ServerKey) -> Self {
        to_document(&value).expect("Failed to convert ServerKey to Document")
    }
}

impl From<Document> for ServerKey {
    fn from(value: Document) -> Self {
        from_document(value.clone()).expect("Failed to convert Document to ServerKey")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerKeyUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<Algorithm>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServerKeyFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<Algorithm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<Uuid>,
}

impl From<ServerKeyFilter> for Document {
    fn from(value: ServerKeyFilter) -> Self {
        let mut doc = Document::new();
        if let Some(key) = value.key {
            doc.insert("key", key);
        }
        if let Some(algorithm) = value.algorithm {
            doc.insert("algorithm", format!("{:?}", algorithm));
        }
        if let Some(environment_id) = value.environment_id {
            doc.insert("environment_id", environment_id);
        }
        doc
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerKeySortableFields {
    Id,
    Algorithm,
    EnvironmentId,
}

impl From<ServerKeySortableFields> for String {
    fn from(value: ServerKeySortableFields) -> Self {
        match value {
            ServerKeySortableFields::Id => "id".to_string(),
            ServerKeySortableFields::Algorithm => "algorithm".to_string(),
            ServerKeySortableFields::EnvironmentId => "environment_id".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_server_key_creation() {
        let now = Utc::now();
        let server_key = ServerKey {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            environment_id: Uuid::new(),
            algorithm: Algorithm::RS256,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(server_key.key, "test-key");
        assert_eq!(server_key.algorithm, Algorithm::RS256);
    }

    #[test]
    fn test_server_key_document_conversion() {
        let server_key = ServerKey {
            id: Some(Uuid::new()),
            key: "test-key".to_string(),
            environment_id: Uuid::new(),
            algorithm: Algorithm::HS256,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let doc: Document = server_key.clone().into();
        let converted: ServerKey = doc.into();

        assert_eq!(server_key.id, converted.id);
        assert_eq!(server_key.key, converted.key);
        assert_eq!(server_key.algorithm, converted.algorithm);
        assert_eq!(server_key.environment_id, converted.environment_id);
    }

    #[test]
    fn test_server_key_update_payload() {
        let update = ServerKeyUpdatePayload {
            key: Some("new-key".to_string()),
            environment_id: Some(Uuid::new()),
            algorithm: Some(Algorithm::RS256),
        };

        assert_eq!(update.key.unwrap(), "new-key");
        assert_eq!(update.algorithm.unwrap(), Algorithm::RS256);
    }

    #[test]
    fn test_server_key_filter() {
        let filter = ServerKeyFilter {
            key: Some("test-key".to_string()),
            algorithm: Some(Algorithm::RS256),
            environment_id: Some(Uuid::new()),
        };

        let doc: Document = filter.into();

        assert_eq!(doc.get_str("key").unwrap(), "test-key");
        assert_eq!(doc.get_str("algorithm").unwrap(), "RS256");
        assert!(doc.contains_key("environment_id"));
    }

    #[test]
    fn test_server_key_sortable_fields() {
        assert_eq!(String::from(ServerKeySortableFields::Id), "id");
        assert_eq!(String::from(ServerKeySortableFields::Algorithm), "algorithm");
        assert_eq!(String::from(ServerKeySortableFields::EnvironmentId), "environment_id");
    }
}
