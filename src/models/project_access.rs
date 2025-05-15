use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Represents access control configuration for a project environment.
///
/// # Fields
/// - `id`: Unique identifier for the project access (MongoDB ObjectId)
/// - `name`: Name of the access configuration
/// - `environment_id`: Foreign key reference to the associated environment
/// - `service_account_id`: Foreign key reference to the associated service account
/// - `project_scopes`: Array of project scope IDs this access is granted
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectAccess {    
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    name: String,
    environment_id: ObjectId,
    service_account_id: ObjectId,
    project_scopes: Vec<ObjectId>,
}

impl ProjectAccess {
    /// Creates a new ProjectAccess with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the access configuration
    /// * `environment_id` - ID of the associated environment
    /// * `service_account_id` - ID of the associated service account
    /// * `project_scopes` - Vector of project scope IDs
    ///
    pub fn new(
        name: String,
        environment_id: ObjectId,
        service_account_id: ObjectId,
        project_scopes: Vec<ObjectId>,
    ) -> Self {
        Self {
            id: None,
            name,
            environment_id,
            service_account_id,
            project_scopes,
        }
    }

    // Convert to MongoDB Document
    pub fn to_document(&self) -> Result<mongodb::bson::Document, mongodb::bson::ser::Error> {
        mongodb::bson::to_document(self)
    }

    // Create from MongoDB Document
    pub fn from_document(doc: mongodb::bson::Document) -> Result<Self, mongodb::bson::de::Error> {
        mongodb::bson::from_document(doc)
    }

    pub fn id(&self) -> Option<&ObjectId> {
        self.id.as_ref()
    }

    /// Sets the access configuration's unique identifier
    pub fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn environment_id(&self) -> &ObjectId {
        &self.environment_id
    }

    pub fn service_account_id(&self) -> &ObjectId {
        &self.service_account_id
    }

    pub fn project_scopes(&self) -> &Vec<ObjectId> {
        &self.project_scopes
    }

    pub fn add_project_scope(&mut self, scope_id: ObjectId) -> bool {
        if !self.project_scopes.contains(&scope_id) {
            self.project_scopes.push(scope_id);
            true
        } else {
            false
        }
    }

    pub fn remove_project_scope(&mut self, scope_id: &ObjectId) -> bool {
        if let Some(pos) = self.project_scopes.iter().position(|x| x == scope_id) {
            self.project_scopes.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_project_access() {
        let name = "Test Access".to_string();
        let environment_id = ObjectId::new();
        let service_account_id = ObjectId::new();
        let project_scopes = vec![ObjectId::new(), ObjectId::new()];

        let access = ProjectAccess::new(
            name.clone(),
            environment_id,
            service_account_id,
            project_scopes.clone(),
        );

        assert!(access.id().is_none());
        assert_eq!(access.name(), name);
        assert_eq!(access.environment_id(), &environment_id);
        assert_eq!(access.service_account_id(), &service_account_id);
        assert_eq!(access.project_scopes(), &project_scopes);
    }

    #[test]
    fn test_document_conversion() {
        let mut access = ProjectAccess::new(
            "Test Access".to_string(),
            ObjectId::new(),
            ObjectId::new(),
            vec![ObjectId::new()],
        );
        let id = ObjectId::new();
        access.set_id(id);

        // Test conversion to document
        let doc = access.to_document().unwrap();
        
        // Test conversion from document
        let converted = ProjectAccess::from_document(doc).unwrap();

        assert_eq!(access.id(), converted.id());
        assert_eq!(access.name(), converted.name());
        assert_eq!(access.environment_id(), converted.environment_id());
        assert_eq!(access.service_account_id(), converted.service_account_id());
        assert_eq!(access.project_scopes(), converted.project_scopes());
    }

    #[test]
    fn test_add_remove_project_scopes() {
        let mut access = ProjectAccess::new(
            "Test Access".to_string(),
            ObjectId::new(),
            ObjectId::new(),
            vec![],
        );

        // Test adding new scope
        let scope1 = ObjectId::new();
        assert!(access.add_project_scope(scope1));
        assert_eq!(access.project_scopes().len(), 1);
        assert!(access.project_scopes().contains(&scope1));

        // Test adding duplicate scope
        assert!(!access.add_project_scope(scope1));
        assert_eq!(access.project_scopes().len(), 1);

        // Test adding another scope
        let scope2 = ObjectId::new();
        assert!(access.add_project_scope(scope2));
        assert_eq!(access.project_scopes().len(), 2);
        assert!(access.project_scopes().contains(&scope2));

        // Test removing existing scope
        assert!(access.remove_project_scope(&scope1));
        assert_eq!(access.project_scopes().len(), 1);
        assert!(!access.project_scopes().contains(&scope1));

        // Test removing non-existent scope
        let non_existent_scope = ObjectId::new();
        assert!(!access.remove_project_scope(&non_existent_scope));
        assert_eq!(access.project_scopes().len(), 1);
    }

    #[test]
    fn test_serialization() {
        let name = "Test Access".to_string();
        let environment_id = ObjectId::new();
        let service_account_id = ObjectId::new();
        let project_scopes = vec![ObjectId::new(), ObjectId::new()];

        let mut access = ProjectAccess::new(
            name,
            environment_id,
            service_account_id,
            project_scopes,
        );
        let id = ObjectId::new();
        access.set_id(id);

        let serialized = serde_json::to_string(&access).unwrap();
        let deserialized: ProjectAccess = serde_json::from_str(&serialized).unwrap();

        assert_eq!(access.id(), deserialized.id());
        assert_eq!(access.name(), deserialized.name());
        assert_eq!(access.environment_id(), deserialized.environment_id());
        assert_eq!(access.service_account_id(), deserialized.service_account_id());
        assert_eq!(access.project_scopes(), deserialized.project_scopes());
    }
}
