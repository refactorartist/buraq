use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

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
    id: ObjectId,
    project_id: ObjectId,
    name: String,
    description: String,
    enabled: bool,
}

impl Environment {
    /// Creates a new Environment with the given project ID, name and description.
    ///
    /// Automatically generates:
    /// - A new ObjectId
    /// - Sets enabled to true by default
    ///
    /// # Arguments
    ///
    /// * `project_id` - ID of the associated project
    /// * `name` - Name of the environment
    /// * `description` - Description of the environment
    pub fn new(project_id: ObjectId, name: String, description: String) -> Self {
        Self {
            id: ObjectId::new(),
            project_id,
            name,
            description,
            enabled: true,
        }
    }

    /// Returns the environment's unique identifier
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    /// Returns the associated project ID
    pub fn project_id(&self) -> &ObjectId {
        &self.project_id
    }

    /// Returns the environment's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the environment's description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns whether the environment is enabled
    pub fn enabled(&self) -> bool {
        self.enabled
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

        let env = Environment::new(project_id.clone(), name.clone(), description.clone());

        assert!(ObjectId::parse_str(env.id().to_hex()).is_ok());
        assert_eq!(env.project_id(), &project_id);
        assert_eq!(env.name(), name);
        assert_eq!(env.description(), description);
        assert!(env.enabled());
    }

    #[test]
    fn test_serialization() {
        let project_id = ObjectId::new();
        let name = "Staging".to_string();
        let description = "Staging environment".to_string();
        let env = Environment::new(project_id, name, description);

        let serialized = serde_json::to_string(&env).unwrap();
        let deserialized: Environment = serde_json::from_str(&serialized).unwrap();

        assert_eq!(env.id(), deserialized.id());
        assert_eq!(env.project_id(), deserialized.project_id());
        assert_eq!(env.name(), deserialized.name());
        assert_eq!(env.description(), deserialized.description());
        assert_eq!(env.enabled(), deserialized.enabled());
    }
}
