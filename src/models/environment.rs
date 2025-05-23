use chrono::{DateTime, Utc};
use mongodb::bson::{Document, Uuid, doc, from_document, to_document};
use serde::{Deserialize, Serialize};

/// Represents an environment associated with a project.
///
/// # Fields
/// - `id`: Unique identifier for the environment (MongoDB Uuid)
/// - `project_id`: Foreign key reference to the associated project
/// - `name`: Name of the environment
/// - `description`: Description of the environment
/// - `enabled`: Whether the environment is active/enabled
/// - `created_at`: Timestamp when environment was created
/// - `updated_at`: Timestamp when environment was last updated
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Environment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub project_id: Uuid,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EnvironmentSortableFields {
    Id,
    Name,
    UpdatedAt,
    CreatedAt,
}

impl From<EnvironmentSortableFields> for String {
    fn from(value: EnvironmentSortableFields) -> Self {
        match value {
            EnvironmentSortableFields::Id => "id".to_string(),
            EnvironmentSortableFields::Name => "name".to_string(),
            EnvironmentSortableFields::UpdatedAt => "updated_at".to_string(),
            EnvironmentSortableFields::CreatedAt => "created_at".to_string(),
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

        let env = Environment {
            id: None,
            project_id,
            name: name.clone(),
            description: description.clone(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        assert!(env.id.is_none());
        assert_eq!(env.project_id, project_id);
        assert_eq!(env.name, name);
        assert_eq!(env.description, description);
        assert!(env.enabled);
        // Verify timestamps are recent
        let now = Utc::now();
        assert!(env.created_at.unwrap() <= now);
        assert!(env.updated_at.unwrap() <= now);
    }

    #[test]
    fn test_serialization() {
        let project_id = Uuid::new();
        let name = "Production".to_string();
        let description = "Production environment".to_string();
        let env = Environment {
            id: None,
            project_id,
            name: name.clone(),
            description: description.clone(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        // Test serialization
        let serialized = serde_json::to_string(&env);
        assert!(serialized.is_ok());

        // Test deserialization
        let deserialized: Result<Environment, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());

        let deserialized_env = deserialized.unwrap();
        assert_eq!(env.id, deserialized_env.id);
        assert_eq!(env.project_id, deserialized_env.project_id);
        assert_eq!(env.name, deserialized_env.name);
        assert_eq!(env.description, deserialized_env.description);
        assert_eq!(env.enabled, deserialized_env.enabled);
        assert_eq!(env.created_at, deserialized_env.created_at);
        assert_eq!(env.updated_at, deserialized_env.updated_at);
    }

    #[test]
    fn test_mongodb_serialization() {
        let project_id = Uuid::new();
        let name = "example-name".to_string();
        let description = "example description".to_string();

        let mut environment = Environment {
            id: None,
            project_id,
            name,
            description,
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let id = Uuid::new();
        environment.id = Some(id);

        let doc: Document = environment.clone().into();
        let converted: Environment = doc.into();

        assert_eq!(environment.id, converted.id);
        assert_eq!(environment.project_id, converted.project_id);
        assert_eq!(environment.name, converted.name);
        assert_eq!(environment.description, converted.description);
        assert_eq!(environment.enabled, converted.enabled);
        assert_eq!(environment.created_at, converted.created_at);
        assert_eq!(environment.updated_at, converted.updated_at);
    }
}
