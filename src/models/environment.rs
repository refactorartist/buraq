use mongodb::bson::{doc, from_document, to_document, Document, Uuid};
use serde::{Deserialize, Serialize};

/// Represents an environment associated with a project.
///
/// # Fields
/// - `id`: Unique identifier for the environment (MongoDB ObjectId)
/// - `project_id`: Foreign key reference to the associated project
/// - `name`: Name of the environment
/// - `description`: Description of the environment
/// - `enabled`: Whether the environment is active/enabled
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Environment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub project_id: Uuid,
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

impl From<Environment> for Document {
    fn from(value: Environment) -> Self {
        to_document(&value).unwrap()
    }
}

impl From<Document> for Environment {
    fn from(value: Document) -> Self {
        from_document(value.clone()).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
}

impl From<EnvironmentFilter> for Document {
    fn from(value: EnvironmentFilter) -> Self {
        let mut doc = Document::new();
        if let Some(project_id) = value.project_id {
            doc.insert("project_id", project_id);
        }
        if let Some(name) = value.name {
            doc.insert("name", name);
        }
        if let Some(is_enabled) = value.is_enabled {
            doc.insert("enabled", is_enabled);
        }
        doc
    }
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
    pub fn new(project_id: Uuid, name: String, description: String) -> Self {
        Self {
            id: None,
            project_id,
            name,
            description,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_environment() {
        let project_id = Uuid::new();
        let name = "Production".to_string();
        let description = "Production environment".to_string();

        let env = Environment::new(project_id, name.clone(), description.clone());

        assert!(env.id.is_none());
        assert_eq!(env.project_id, project_id);
        assert_eq!(env.name, name);
        assert_eq!(env.description, description);
        assert!(env.enabled);
    }

    #[test]
    fn test_mongodb_serialization() {
        let project_id = Uuid::new();
        let name = "example-name".to_string();
        let description = "example description".to_string();

        let mut environment = Environment::new(
            project_id,
            name.clone(),
            description.clone()
        );
        let id = Uuid::new();
        environment.id = Some(id);

        let doc: Document = environment.clone().into();
        let converted: Environment = doc.into();

        assert_eq!(environment.id, converted.id);
        assert_eq!(environment.project_id, converted.project_id);
        assert_eq!(environment.name, converted.name);
        assert_eq!(environment.description, converted.description);
        assert_eq!(environment.enabled, converted.enabled);
    }

    #[test]
    fn test_environment_update_payload() {
        let update = EnvironmentUpdatePayload {
            name: Some("new-name".to_string()),
            description: Some("new description".to_string()),
            enabled: Some(false),
        };

        assert_eq!(update.name.unwrap(), "new-name");
        assert_eq!(update.description.unwrap(), "new description");
        assert!(!update.enabled.unwrap());
    }

    #[test]
    fn test_environment_filter() {
        let project_id = Uuid::new();
        let filter = EnvironmentFilter {
            project_id: Some(project_id),
            name: Some("test-env".to_string()),
            is_enabled: Some(true),
        };

        let doc: Document = filter.into();

        // Extract UUID from the document
        let extracted_uuid = match doc.get("project_id").unwrap() {
            mongodb::bson::Bson::Binary(binary) => Uuid::from_bytes(binary.bytes.clone().try_into().unwrap()),
            _ => panic!("Expected UUID binary"),
        };

        assert_eq!(extracted_uuid, project_id);
        assert_eq!(doc.get_str("name").unwrap(), "test-env");
        assert!(doc.get_bool("enabled").unwrap());
    }
}
