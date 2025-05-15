use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Represents an environment associated with a project.
///
/// # Fields
/// - `id`: Unique identifier for the environment (MongoDB ObjectId)
/// - `project_id`: Foreign key reference to the associated project
/// - `name`: Name of the environment
/// - `description`: Description of the environment
/// - `enabled`: Whether the environment is active/enabled
#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    project_id: ObjectId,
    name: String,
    description: String,
    enabled: bool,
}

impl Environment {
    /// Creates a new Environment with the given project ID, name and description.
    ///
    /// Automatically generates:
    /// - Sets enabled to true by default
    ///
    /// # Arguments
    ///
    /// * `project_id` - ID of the associated project
    /// * `name` - Name of the environment
    /// * `description` - Description of the environment
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::environment::Environment;
    /// use mongodb::bson::oid::ObjectId;
    ///
    /// let project_id = ObjectId::new();
    /// let name = "Production".to_string();
    /// let description = "Production environment".to_string();
    ///
    /// let env = Environment::new(project_id, name.clone(), description.clone());
    /// assert_eq!(env.name(), "Production");
    /// assert_eq!(env.description(), "Production environment");
    /// assert!(env.enabled());
    /// ```
    pub fn new(project_id: ObjectId, name: String, description: String) -> Self {
        Self {
            id: None,
            project_id,
            name,
            description,
            enabled: true,
        }
    }

    pub fn id(&self) -> Option<&ObjectId> {
        self.id.as_ref()
    }

    /// Sets the environment's unique identifier
    pub fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }

    pub fn project_id(&self) -> &ObjectId {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn enabled(&self) -> bool {
        self.enabled
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
    fn test_new_environment() {
        let project_id = ObjectId::new();
        let name = "Production".to_string();
        let description = "Production environment".to_string();

        let env = Environment::new(project_id, name.clone(), description.clone());

        assert!(env.id().is_none());
        assert_eq!(env.project_id(), &project_id);
        assert_eq!(env.name(), name);
        assert_eq!(env.description(), description);
        assert!(env.enabled());
    }

    #[test]
    fn test_mongodb_serialization() {
        let project_id = ObjectId::new();
        let name = "example-name".to_string();
        let description = "example description".to_string();
        let enable = true;

        let mut environment = Environment::new(
            project_id,
            name.clone(),
            description.clone()
        );
        let id = ObjectId::new();
        environment.set_id(id);

        let doc = environment.to_document().unwrap();

        let deserialized = Environment::from_document(doc).unwrap();
        assert_eq!(deserialized.id(), environment.id());
        assert_eq!(deserialized.project_id, project_id);
        assert_eq!(deserialized.name, name);
        assert_eq!(deserialized.description, description);
        assert_eq!(deserialized.enabled, enable);
    }

    #[test]
    fn test_serialization() {
        let project_id = ObjectId::new();
        let name = "Staging".to_string();
        let description = "Staging environment".to_string();
        let mut env = Environment::new(project_id, name, description);
        let id = ObjectId::new();
        env.set_id(id);

        let serialized = serde_json::to_string(&env).unwrap();
        let deserialized: Environment = serde_json::from_str(&serialized).unwrap();

        assert_eq!(env.id(), deserialized.id());
        assert_eq!(env.project_id(), deserialized.project_id());
        assert_eq!(env.name(), deserialized.name());
        assert_eq!(env.description(), deserialized.description());
        assert_eq!(env.enabled(), deserialized.enabled());
    }
}
