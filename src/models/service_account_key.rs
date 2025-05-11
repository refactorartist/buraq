use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::Algorithm;
use crate::serializers::algorithm;

/// Represents a service account key for API authentication
///
/// # Fields
/// - `id`: Unique identifier for the service account key (MongoDB ObjectId)
/// - `service_account_id`: Foreign key reference to the associated service account
/// - `algorithm`: The algorithm used for the key
/// - `key`: The actual key value
/// - `expires_at`: Key expiration timestamp
#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct ServiceAccountKey {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    service_account_id: ObjectId,
    #[serde(with = "algorithm")]
    algorithm: Algorithm,
    key: String,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    enabled: bool,
}

impl ServiceAccountKey {
    /// Creates a new ServiceAccountKey with the given parameters
    ///
    /// Automatically generates:
    /// - Current UTC timestamp for created_at
    /// - Sets enabled to true by default
    ///
    /// # Arguments
    /// * `service_account_id` - ID of the associated service account
    /// * `algorithm` - The algorithm used for the key
    /// * `key` - The key value
    /// * `expires_at` - When the key expires
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::service_account_key::ServiceAccountKey;
    /// use buraq::types::Algorithm;
    /// use mongodb::bson::oid::ObjectId;
    /// use chrono::{Utc, Duration};
    ///
    /// let service_account_id = ObjectId::new();
    /// let algorithm = Algorithm::RSA;
    /// let key = "service-key-123456".to_string();
    /// let expires_at = Utc::now() + Duration::days(30);
    ///
    /// let service_key = ServiceAccountKey::new(
    ///     service_account_id,
    ///     algorithm,
    ///     key.clone(),
    ///     expires_at
    /// );
    /// assert_eq!(service_key.key(), "service-key-123456");
    /// assert!(matches!(service_key.algorithm(), Algorithm::RSA));
    /// assert!(!service_key.is_expired());
    /// ```
    pub fn new(service_account_id: ObjectId, algorithm: Algorithm, key: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: None,
            service_account_id,
            algorithm,
            key,
            expires_at,
            created_at: Utc::now(),
            enabled: true,
        }
    }

    /// Returns the key's unique identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::service_account_key::ServiceAccountKey;
    /// use buraq::types::Algorithm;
    /// use mongodb::bson::oid::ObjectId;
    /// use chrono::{Utc, Duration};
    ///
    /// let service_account_id = ObjectId::new();
    /// let mut key = ServiceAccountKey::new(
    ///     service_account_id,
    ///     Algorithm::RSA,
    ///     "service-key".to_string(),
    ///     Utc::now() + Duration::days(7)
    /// );
    ///
    /// assert!(key.id().is_none());
    ///
    /// let id = ObjectId::new();
    /// key.set_id(id);
    /// assert!(key.id().is_some());
    /// ```
    pub fn id(&self) -> Option<&ObjectId> {
        self.id.as_ref()
    }

    /// Sets the key's unique identifier
    pub fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }

    /// Returns the associated service account ID
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::service_account_key::ServiceAccountKey;
    /// use buraq::types::Algorithm;
    /// use mongodb::bson::oid::ObjectId;
    /// use chrono::{Utc, Duration};
    ///
    /// let service_account_id = ObjectId::new();
    /// let key = ServiceAccountKey::new(
    ///     service_account_id,
    ///     Algorithm::RSA,
    ///     "service-key".to_string(),
    ///     Utc::now() + Duration::days(7)
    /// );
    ///
    /// assert_eq!(key.service_account_id(), &service_account_id);
    /// ```
    pub fn service_account_id(&self) -> &ObjectId {
        &self.service_account_id
    }

    /// Returns the algorithm used
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::service_account_key::ServiceAccountKey;
    /// use buraq::types::Algorithm;
    /// use mongodb::bson::oid::ObjectId;
    /// use chrono::{Utc, Duration};
    ///
    /// let key = ServiceAccountKey::new(
    ///     ObjectId::new(),
    ///     Algorithm::HMAC,
    ///     "service-key".to_string(),
    ///     Utc::now() + Duration::days(7)
    /// );
    ///
    /// assert!(matches!(key.algorithm(), Algorithm::HMAC));
    /// ```
    pub fn algorithm(&self) -> &Algorithm {
        &self.algorithm
    }

    /// Returns the key value
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::service_account_key::ServiceAccountKey;
    /// use buraq::types::Algorithm;
    /// use mongodb::bson::oid::ObjectId;
    /// use chrono::{Utc, Duration};
    ///
    /// let key_value = "my-secret-service-key".to_string();
    /// let key = ServiceAccountKey::new(
    ///     ObjectId::new(),
    ///     Algorithm::RSA,
    ///     key_value.clone(),
    ///     Utc::now() + Duration::days(7)
    /// );
    ///
    /// assert_eq!(key.key(), key_value);
    /// ```
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the expiration timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::service_account_key::ServiceAccountKey;
    /// use buraq::types::Algorithm;
    /// use mongodb::bson::oid::ObjectId;
    /// use chrono::{Utc, Duration};
    ///
    /// let expires_at = Utc::now() + Duration::days(30);
    /// let key = ServiceAccountKey::new(
    ///     ObjectId::new(),
    ///     Algorithm::RSA,
    ///     "service-key".to_string(),
    ///     expires_at
    /// );
    ///
    /// assert_eq!(key.expires_at(), &expires_at);
    /// ```
    pub fn expires_at(&self) -> &DateTime<Utc> {
        &self.expires_at
    }

    /// Returns the creation timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::service_account_key::ServiceAccountKey;
    /// use buraq::types::Algorithm;
    /// use mongodb::bson::oid::ObjectId;
    /// use chrono::{Utc, Duration};
    ///
    /// let before = Utc::now();
    /// let key = ServiceAccountKey::new(
    ///     ObjectId::new(),
    ///     Algorithm::RSA,
    ///     "service-key".to_string(),
    ///     Utc::now() + Duration::days(7)
    /// );
    /// let after = Utc::now();
    ///
    /// assert!(before <= *key.created_at() && *key.created_at() <= after);
    /// ```
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Returns whether the key is enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::service_account_key::ServiceAccountKey;
    /// use buraq::types::Algorithm;
    /// use mongodb::bson::oid::ObjectId;
    /// use chrono::{Utc, Duration};
    ///
    /// let key = ServiceAccountKey::new(
    ///     ObjectId::new(),
    ///     Algorithm::RSA,
    ///     "service-key".to_string(),
    ///     Utc::now() + Duration::days(7)
    /// );
    ///
    /// assert!(key.enabled());
    /// ```
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Checks if the key has expired
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::service_account_key::ServiceAccountKey;
    /// use buraq::types::Algorithm;
    /// use mongodb::bson::oid::ObjectId;
    /// use chrono::{Utc, Duration};
    ///
    /// // Create a key that expires in the future
    /// let valid_key = ServiceAccountKey::new(
    ///     ObjectId::new(),
    ///     Algorithm::RSA,
    ///     "valid-key".to_string(),
    ///     Utc::now() + Duration::days(7)
    /// );
    /// assert!(!valid_key.is_expired());
    ///
    /// // Create a key that has already expired
    /// let expired_key = ServiceAccountKey::new(
    ///     ObjectId::new(),
    ///     Algorithm::RSA,
    ///     "expired-key".to_string(),
    ///     Utc::now() - Duration::hours(1)
    /// );
    /// assert!(expired_key.is_expired());
    /// ```
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
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
    fn test_new_service_account_key() {
        let service_account_id = ObjectId::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        let service_account_key = ServiceAccountKey::new(
            service_account_id,
            algorithm,
            key.clone(),
            expires_at
        );

        assert!(service_account_key.id().is_none());
        assert_eq!(service_account_key.service_account_id(), &service_account_id);
        assert!(matches!(service_account_key.algorithm(), Algorithm::RSA));
        assert_eq!(service_account_key.key(), key);
        assert_eq!(service_account_key.expires_at(), &expires_at);
        assert!(service_account_key.enabled());
        assert!(!service_account_key.is_expired());
    }

    #[test]
    fn test_mongodb_serialization() {
        let service_account_id = ObjectId::new();
        let algorithm = Algorithm::RSA;
        let key = "test-key-value".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

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

        assert_eq!(service_account_key.id(), deserialized.id());
        assert_eq!(service_account_key.service_account_id(), deserialized.service_account_id());
        assert!(matches!(deserialized.algorithm(), Algorithm::RSA));
        assert_eq!(service_account_key.key(), deserialized.key());
        assert_eq!(service_account_key.expires_at(), deserialized.expires_at());
        assert_eq!(service_account_key.created_at(), deserialized.created_at());
        assert_eq!(service_account_key.enabled(), deserialized.enabled());
    }

    #[test]
    fn test_algorithm_serialization() {
        let service_account_id = ObjectId::new();
        let key = "test-key-value".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

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

    #[test]
    fn test_expiration() {
        let service_account_id = ObjectId::new();
        let expired_time = Utc::now() - chrono::Duration::hours(1);
        let key = ServiceAccountKey::new(
            service_account_id,
            Algorithm::HMAC,
            "test-key".to_string(),
            expired_time
        );

        assert!(key.is_expired());
    }
}
