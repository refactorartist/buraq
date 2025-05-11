use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Represents a project scope that defines permissions within a project.
///
/// # Fields
/// - `id`: Unique identifier for the project scope (MongoDB ObjectId)
/// - `project_id`: Foreign key reference to the associated project
/// - `name`: Name of the scope
/// - `description`: Description of what the scope allows
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectScope {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    project_id: ObjectId,
    name: String,
    description: String,
}

impl ProjectScope {
    /// Creates a new ProjectScope with the given project ID, name and description.
    ///
    /// # Arguments
    ///
    /// * `project_id` - ID of the associated project
    /// * `name` - Name of the scope
    /// * `description` - Description of the scope
    pub fn new(project_id: ObjectId, name: String, description: String) -> Self {
        Self {
            id: None,
            project_id,
            name,
            description,
        }
    }

    /// Returns the project scope's unique identifier
    pub fn id(&self) -> Option<&ObjectId> {
        self.id.as_ref()
    }

    /// Sets the project scope's unique identifier
    pub fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }

    /// Returns the associated project ID
    pub fn project_id(&self) -> &ObjectId {
        &self.project_id
    }

    /// Returns the scope's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the scope's description
    pub fn description(&self) -> &str {
        &self.description
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
    fn test_new_project_scope() {
        let project_id = ObjectId::new();
        let name = "read:users".to_string();
        let description = "Allows reading user data".to_string();

        let scope = ProjectScope::new(project_id, name.clone(), description.clone());

        assert!(scope.id().is_none());
        assert_eq!(scope.project_id(), &project_id);
        assert_eq!(scope.name(), name);
        assert_eq!(scope.description(), description);
    }

    #[test]
    fn test_mongodb_serialization() {
        let project_id = ObjectId::new();
        let name = "write:posts".to_string();
        let description = "Allows creating and updating posts".to_string();

        let mut scope = ProjectScope::new(project_id, name.clone(), description.clone());
        let id = ObjectId::new();
        scope.set_id(id);

        // Test conversion to BSON Document
        let doc = scope.to_document().unwrap();
        
        // Test conversion from BSON Document
        let deserialized = ProjectScope::from_document(doc).unwrap();

        assert_eq!(deserialized.id(), scope.id());
        assert_eq!(deserialized.project_id, project_id);
        assert_eq!(deserialized.name, name);
        assert_eq!(deserialized.description, description);
    }

    #[test]
    fn test_serialization() {
        let project_id = ObjectId::new();
        let name = "example-scope".to_string();
        let description = "example description".to_string();

        let mut scope = ProjectScope::new(
            project_id,
            name.clone(),
            description.clone()
        );
        let id = ObjectId::new();
        scope.set_id(id);

        let doc = scope.to_document().unwrap();

        let deserialized = ProjectScope::from_document(doc).unwrap();
        assert_eq!(deserialized.id(), scope.id());
        assert_eq!(deserialized.project_id, project_id);
        assert_eq!(deserialized.name, name);
        assert_eq!(deserialized.description, description);
    }
}
