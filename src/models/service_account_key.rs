use mongodb::bson::{oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::Algorithm;

//service_account_keys {
//    ObjectID service_account_id PK "Primary Key"
//    string algorithm FK "Foreign Key to ALGORITHM"
//    string key "Key value" 
//    date expires_at "Expiration date"
//}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceAccountKey {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    service_account_id: ObjectId,
    #[serde(with = "algorithm_serializer")]
    algorithm: Algorithm,
    key: String,
    expires_at: DateTime<Utc>,
}

// Custom serializer for Algorithm enum to store as string in MongoDB
mod algorithm_serializer {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(algorithm: &Algorithm, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let algorithm_str = match algorithm {
            Algorithm::RSA => "RSA",
            Algorithm::HMAC => "HMAC",
        };
        serializer.serialize_str(algorithm_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Algorithm, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "RSA" => Ok(Algorithm::RSA),
            "HMAC" => Ok(Algorithm::HMAC),
            _ => Err(serde::de::Error::custom("Invalid algorithm type")),
        }
    }
}

impl ServiceAccountKey {
    pub fn new(service_account_id: ObjectId, algorithm: Algorithm, key: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: None,
            service_account_id,
            algorithm,
            key,
            expires_at,
        }
    }

    pub fn service_account_id(&self) -> &ObjectId {
        &self.service_account_id
    }

    pub fn algorithm(&self) -> &Algorithm {
        &self.algorithm
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    // Convert to MongoDB Document
    pub fn to_document(&self) -> Result<Document, mongodb::bson::ser::Error> {
        mongodb::bson::to_document(self)
    }

    // Create from MongoDB Document
    pub fn from_document(doc: Document) -> Result<Self, mongodb::bson::de::Error> {
        mongodb::bson::from_document(doc)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_service_account_key() {
        let service_account_id = ObjectId::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now();

        let service_account_key = ServiceAccountKey::new(
            service_account_id,
            algorithm,
            key.clone(),
            expires_at
        );

        assert_eq!(service_account_key.service_account_id, service_account_id);
        assert!(matches!(service_account_key.algorithm, Algorithm::RSA));
        assert_eq!(service_account_key.key, key);
        assert_eq!(service_account_key.expires_at, expires_at);
    }

    #[test]
    fn test_mongodb_serialization() {
        let service_account_id = ObjectId::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now();

        let service_account_key = ServiceAccountKey::new(
            service_account_id,
            algorithm,
            key.clone(),
            expires_at
        );

        // Test conversion to BSON Document
        let doc = service_account_key.to_document().unwrap();
        
        // Test conversion from BSON Document
        let deserialized = ServiceAccountKey::from_document(doc).unwrap();

        assert_eq!(deserialized.service_account_id, service_account_id);
        assert!(matches!(deserialized.algorithm, Algorithm::RSA));
        assert_eq!(deserialized.key, key);
        assert_eq!(deserialized.expires_at.timestamp(), expires_at.timestamp());
    }

    #[test]
    fn test_algorithm_serialization() {
        let service_account_id = ObjectId::new();
        let key = "test-key-value".to_string();
        let expires_at = Utc::now();

        // Test RSA algorithm
        let rsa_key = ServiceAccountKey::new(
            service_account_id,
            Algorithm::RSA,
            key.clone(),
            expires_at
        );
        let doc = rsa_key.to_document().unwrap();
        assert_eq!(doc.get_str("algorithm").unwrap(), "RSA");

        // Test HMAC algorithm
        let hmac_key = ServiceAccountKey::new(
            service_account_id,
            Algorithm::HMAC,
            key,
            expires_at
        );
        let doc = hmac_key.to_document().unwrap();
        assert_eq!(doc.get_str("algorithm").unwrap(), "HMAC");
    }
}




