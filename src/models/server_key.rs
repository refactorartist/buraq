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
pub struct ServerKeyRead {
    pub id: Uuid,
    pub environment_id: Uuid,
    #[serde(with = "crate::serializers::algorithm")]
    pub algorithm: Algorithm,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ServerKey> for ServerKeyRead {
    fn from(value: ServerKey) -> Self {
        ServerKeyRead {
            id: value.id.unwrap(),
            environment_id: value.environment_id,
            algorithm: value.algorithm,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerKeyUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none", with = "crate::serializers::option_algorithm")]
    pub algorithm: Option<Algorithm>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerKeyCreatePayload {
    pub environment_id: Uuid,
    #[serde(with = "crate::serializers::algorithm")]
    pub algorithm: Algorithm,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServerKeyFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", with = "crate::serializers::option_algorithm")]
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
    use serde_json::{to_value, from_value};
    use chrono::TimeZone;

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
        assert_eq!(
            String::from(ServerKeySortableFields::Algorithm),
            "algorithm"
        );
        assert_eq!(
            String::from(ServerKeySortableFields::EnvironmentId),
            "environment_id"
        );
    }
    
    #[test]
    fn test_server_key_create_payload() {
        let environment_id = Uuid::new();
        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::HS384,
        };
        
        assert_eq!(payload.environment_id, environment_id);
        assert_eq!(payload.algorithm, Algorithm::HS384);
    }
    
    #[test]
    fn test_server_key_create_payload_serialization() {
        let environment_id = Uuid::new();
        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::HS384,
        };
        
        let json = to_value(&payload).unwrap();
        assert_eq!(json["algorithm"], "HS384");
        assert_eq!(json["environment_id"], environment_id.to_string());
        
        let deserialized: ServerKeyCreatePayload = from_value(json).unwrap();
        assert_eq!(deserialized.environment_id, environment_id);
        assert_eq!(deserialized.algorithm, Algorithm::HS384);
    }
    
    #[test]
    fn test_server_key_update_payload_serialization() {
        let environment_id = Uuid::new();
        let update = ServerKeyUpdatePayload {
            key: Some("new-key".to_string()),
            environment_id: Some(environment_id),
            algorithm: Some(Algorithm::RS256),
        };
        
        let json = to_value(&update).unwrap();
        assert_eq!(json["key"], "new-key");
        assert_eq!(json["algorithm"], "RS256");
        assert_eq!(json["environment_id"], environment_id.to_string());
        
        let deserialized: ServerKeyUpdatePayload = from_value(json).unwrap();
        assert_eq!(deserialized.key.unwrap(), "new-key");
        assert_eq!(deserialized.algorithm.unwrap(), Algorithm::RS256);
        assert_eq!(deserialized.environment_id.unwrap(), environment_id);
    }
    
    #[test]
    fn test_server_key_update_payload_partial_serialization() {
        // Test with only some fields present
        let update = ServerKeyUpdatePayload {
            key: None,
            environment_id: None,
            algorithm: Some(Algorithm::RS256),
        };
        
        let json = to_value(&update).unwrap();
        assert!(!json.as_object().unwrap().contains_key("key"));
        assert!(!json.as_object().unwrap().contains_key("environment_id"));
        assert_eq!(json["algorithm"], "RS256");
        
        let deserialized: ServerKeyUpdatePayload = from_value(json).unwrap();
        assert!(deserialized.key.is_none());
        assert!(deserialized.environment_id.is_none());
        assert_eq!(deserialized.algorithm.unwrap(), Algorithm::RS256);
    }
    
    #[test]
    fn test_server_key_filter_serialization() {
        let environment_id = Uuid::new();
        let filter = ServerKeyFilter {
            key: Some("test-key".to_string()),
            algorithm: Some(Algorithm::RS256),
            environment_id: Some(environment_id),
        };
        
        let json = to_value(&filter).unwrap();
        assert_eq!(json["key"], "test-key");
        assert_eq!(json["algorithm"], "RS256");
        assert_eq!(json["environment_id"], environment_id.to_string());
        
        let deserialized: ServerKeyFilter = from_value(json).unwrap();
        assert_eq!(deserialized.key.unwrap(), "test-key");
        assert_eq!(deserialized.algorithm.unwrap(), Algorithm::RS256);
        assert_eq!(deserialized.environment_id.unwrap(), environment_id);
    }
    
    #[test]
    fn test_server_key_filter_partial_serialization() {
        // Test with only some fields present
        let filter = ServerKeyFilter {
            key: None,
            algorithm: Some(Algorithm::RS256),
            environment_id: None,
        };
        
        let json = to_value(&filter).unwrap();
        assert!(!json.as_object().unwrap().contains_key("key"));
        assert!(!json.as_object().unwrap().contains_key("environment_id"));
        assert_eq!(json["algorithm"], "RS256");
        
        let deserialized: ServerKeyFilter = from_value(json).unwrap();
        assert!(deserialized.key.is_none());
        assert!(deserialized.environment_id.is_none());
        assert_eq!(deserialized.algorithm.unwrap(), Algorithm::RS256);
    }
    
    #[test]
    fn test_server_key_filter_empty() {
        let filter = ServerKeyFilter::default();
        
        let json = to_value(&filter).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.is_empty() || (!obj.contains_key("key") && !obj.contains_key("algorithm") && !obj.contains_key("environment_id")));
        
        let doc: Document = filter.into();
        assert!(doc.is_empty());
    }
    
    #[test]
    fn test_server_key_read_from_server_key() {
        // Arrange
        let id = Uuid::new();
        let environment_id = Uuid::new();
        let now = Utc::now();
        let server_key = ServerKey {
            id: Some(id),
            key: "test-key".to_string(),
            environment_id,
            algorithm: Algorithm::RS256,
            created_at: now,
            updated_at: now,
        };
        
        // Act
        let server_key_read: ServerKeyRead = server_key.into();
        
        // Assert
        assert_eq!(server_key_read.id, id);
        assert_eq!(server_key_read.environment_id, environment_id);
        assert_eq!(server_key_read.algorithm, Algorithm::RS256);
        assert_eq!(server_key_read.created_at, now);
        assert_eq!(server_key_read.updated_at, now);
    }
    
    #[test]
    fn test_server_key_read_serialization() {
        // Arrange
        let id = Uuid::new();
        let environment_id = Uuid::new();
        let created_at = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let updated_at = Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap();
        
        let server_key_read = ServerKeyRead {
            id,
            environment_id,
            algorithm: Algorithm::RS256,
            created_at,
            updated_at,
        };
        
        // Act
        let json = to_value(&server_key_read).unwrap();
        
        // Assert
        assert_eq!(json["id"], id.to_string());
        assert_eq!(json["environment_id"], environment_id.to_string());
        assert_eq!(json["algorithm"], "RS256");
        assert_eq!(json["created_at"], "2023-01-01T00:00:00Z");
        assert_eq!(json["updated_at"], "2023-01-02T00:00:00Z");
        
        // Act - Deserialization
        let deserialized: ServerKeyRead = from_value(json).unwrap();
        
        // Assert
        assert_eq!(deserialized.id, id);
        assert_eq!(deserialized.environment_id, environment_id);
        assert_eq!(deserialized.algorithm, Algorithm::RS256);
        assert_eq!(deserialized.created_at, created_at);
        assert_eq!(deserialized.updated_at, updated_at);
    }
    
    #[test]
    fn test_server_key_read_missing_id() {
        // Arrange
        let server_key = ServerKey {
            id: None, // Missing ID should cause a panic
            key: "test-key".to_string(),
            environment_id: Uuid::new(),
            algorithm: Algorithm::RS256,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Act & Assert
        // This should panic because id is None
        let result = std::panic::catch_unwind(|| {
            let _: ServerKeyRead = server_key.into();
        });
        
        assert!(result.is_err(), "Expected panic when converting ServerKey with None id to ServerKeyRead");
    }
}
