use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::Algorithm;
use crate::serializers::algorithm;

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
    pub id: Option<ObjectId>,
    key: String,
    #[serde(with = "algorithm")]
    algorithm: Algorithm,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    enabled: bool,
}

impl AccessToken {
    /// Creates a new AccessToken with the given parameters
    ///
    /// Automatically generates:
    /// - Current UTC timestamp for created_at
    /// - Sets enabled to true by default
    ///
    /// # Arguments
    /// * `key` - The API key value
    /// * `algorithm` - The algorithm used for the token
    /// * `expires_at` - When the token expires
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::access_token::AccessToken;
    /// use buraq::types::Algorithm;
    /// use chrono::{Utc, Duration};
    ///
    /// let key = "api-key-123456".to_string();
    /// let algorithm = Algorithm::HMAC;
    /// let expires_at = Utc::now() + Duration::days(30);
    ///
    /// let token = AccessToken::new(key, algorithm, expires_at);
    /// assert_eq!(token.key(), "api-key-123456");
    /// assert!(matches!(token.algorithm(), Algorithm::HMAC));
    /// assert!(!token.is_expired());
    /// ```
    pub fn new(key: String, algorithm: Algorithm, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: None,
            key,
            algorithm,
            expires_at,
            created_at: Utc::now(),
            enabled: true,
        }
    }

    /// Returns the token's unique identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::access_token::AccessToken;
    /// use buraq::types::Algorithm;
    /// use chrono::{Utc, Duration};
    /// use mongodb::bson::oid::ObjectId;
    ///
    /// let mut token = AccessToken::new(
    ///     "api-key".to_string(),
    ///     Algorithm::RSA,
    ///     Utc::now() + Duration::days(7)
    /// );
    ///
    /// assert!(token.id().is_none());
    ///
    /// let id = ObjectId::new();
    /// token.set_id(id);
    /// assert!(token.id().is_some());
    /// ```
    pub fn id(&self) -> Option<&ObjectId> {
        self.id.as_ref()
    }

    /// Sets the token's unique identifier
    pub fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }

    /// Returns the API key value
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::access_token::AccessToken;
    /// use buraq::types::Algorithm;
    /// use chrono::{Utc, Duration};
    ///
    /// let key = "my-secret-key".to_string();
    /// let token = AccessToken::new(
    ///     key.clone(),
    ///     Algorithm::HMAC,
    ///     Utc::now() + Duration::days(7)
    /// );
    ///
    /// assert_eq!(token.key(), key);
    /// ```
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the algorithm used
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::access_token::AccessToken;
    /// use buraq::types::Algorithm;
    /// use chrono::{Utc, Duration};
    ///
    /// let token = AccessToken::new(
    ///     "api-key".to_string(),
    ///     Algorithm::RSA,
    ///     Utc::now() + Duration::days(7)
    /// );
    ///
    /// assert!(matches!(token.algorithm(), Algorithm::RSA));
    /// ```
    pub fn algorithm(&self) -> &Algorithm {
        &self.algorithm
    }

    /// Returns the expiration timestamp
    pub fn expires_at(&self) -> &DateTime<Utc> {
        &self.expires_at
    }

    /// Returns the creation timestamp
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Returns whether the token is enabled
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Checks if the token has expired
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::access_token::AccessToken;
    /// use buraq::types::Algorithm;
    /// use chrono::{Utc, Duration};
    ///
    /// // Create a token that expires in the future
    /// let valid_token = AccessToken::new(
    ///     "valid-key".to_string(),
    ///     Algorithm::HMAC,
    ///     Utc::now() + Duration::days(7)
    /// );
    /// assert!(!valid_token.is_expired());
    ///
    /// // Create a token that has already expired
    /// let expired_token = AccessToken::new(
    ///     "expired-key".to_string(),
    ///     Algorithm::HMAC,
    ///     Utc::now() - Duration::hours(1)
    /// );
    /// assert!(expired_token.is_expired());
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
mod tests {
    use super::*;

    #[test]
    fn test_new_access_token() {
        let key = "test-key".to_string();
        let algorithm = Algorithm::HMAC;
        let expires_at = Utc::now() + chrono::Duration::days(7);

        let token = AccessToken::new(key.clone(), algorithm, expires_at);

        assert!(token.id().is_none());
        assert_eq!(token.key(), key);
        assert!(matches!(token.algorithm(), Algorithm::HMAC));
        assert_eq!(token.expires_at(), &expires_at);
        assert!(token.enabled());
        assert!(!token.is_expired());
    }

    #[test]
    fn test_mongodb_serialization() {
        let key = "test-key".to_string();
        let algorithm = Algorithm::RSA;
        let expires_at = Utc::now() + chrono::Duration::days(7);

        let mut token = AccessToken::new(key.clone(), algorithm, expires_at);
        let id = ObjectId::new();
        token.set_id(id);

        // Test conversion to BSON Document
        let doc = token.to_document().unwrap();
        
        // Test conversion from BSON Document
        let deserialized = AccessToken::from_document(doc).unwrap();

        assert_eq!(token.id(), deserialized.id());
        assert_eq!(token.key(), deserialized.key());
        assert!(matches!(deserialized.algorithm(), Algorithm::RSA));
        assert_eq!(token.expires_at(), deserialized.expires_at());
        assert_eq!(token.created_at(), deserialized.created_at());
        assert_eq!(token.enabled(), deserialized.enabled());
    }

    #[test]
    fn test_algorithm_serialization() {
        let key = "test-key".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        // Test RSA algorithm
        let rsa_token = AccessToken::new(
            key.clone(),
            Algorithm::RSA,
            expires_at
        );
        let doc = rsa_token.to_document().unwrap();
        assert_eq!(doc.get_str("algorithm").unwrap(), "RSA");

        // Test HMAC algorithm
        let hmac_token = AccessToken::new(
            key,
            Algorithm::HMAC,
            expires_at
        );
        let doc = hmac_token.to_document().unwrap();
        assert_eq!(doc.get_str("algorithm").unwrap(), "HMAC");
    }

    #[test]
    fn test_expiration() {
        let expired_time = Utc::now() - chrono::Duration::hours(1);
        let token = AccessToken::new(
            "test-key".to_string(),
            Algorithm::HMAC,
            expired_time
        );

        assert!(token.is_expired());
    }
}
