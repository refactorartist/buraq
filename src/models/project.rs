use chrono::{DateTime, Utc};
use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, doc, from_document, to_document};
use serde::{Deserialize, Serialize};

/// Represents a project with metadata and timestamps.
///
/// # Fields
/// - `id`: Unique identifier for the project (MongoDB Uuid)
/// - `name`: Name of the project
/// - `description`: Description of the project
/// - `created_at`: Timestamp when project was created
/// - `updated_at`: Timestamp when project was last updated
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Project> for Document {
    fn from(value: Project) -> Self {
        to_document(&value).unwrap()
    }
}

impl From<Document> for Project {
    fn from(value: Document) -> Self {
        from_document(value.clone()).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProjectFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
}

impl From<ProjectFilter> for Document {
    fn from(value: ProjectFilter) -> Self {
        let mut doc = Document::new();
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
pub enum ProjectSortableFields {
    Id,
    Name,
    UpdatedAt,
    CreatedAt,
}

impl From<ProjectSortableFields> for String {
    fn from(value: ProjectSortableFields) -> Self {
        match value {
            ProjectSortableFields::Id => "id".to_string(),
            ProjectSortableFields::Name => "name".to_string(),
            ProjectSortableFields::UpdatedAt => "updated_at".to_string(),
            ProjectSortableFields::CreatedAt => "created_at".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_project() {
        let name = "Test Project".to_string();
        let description = "Test Description".to_string();

        let project = Project {
            id: None,
            name: name.clone(),
            description: description.clone(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        // Verify fields are set correctly
        assert!(project.id.is_none());
        assert_eq!(project.name, name);
        assert_eq!(project.description, description);
        assert!(project.enabled);
        // Verify timestamps are recent
        let now = Utc::now();
        assert!(project.created_at.unwrap() <= now);
        assert!(project.updated_at.unwrap() <= now);
    }

    #[test]
    fn test_serialization() {
        let name = "Test Project".to_string();
        let description = "Test Description".to_string();
        let project = Project {
            id: None,
            name: name.clone(),
            description: description.clone(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        // Test serialization
        let serialized = serde_json::to_string(&project);
        assert!(serialized.is_ok());

        // Test deserialization
        let deserialized: Result<Project, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());

        let deserialized_project = deserialized.unwrap();
        assert_eq!(project.id, deserialized_project.id);
        assert_eq!(project.name, deserialized_project.name);
        assert_eq!(project.description, deserialized_project.description);
        assert_eq!(project.enabled, deserialized_project.enabled);
        assert_eq!(project.created_at, deserialized_project.created_at);
        assert_eq!(project.updated_at, deserialized_project.updated_at);
    }

    #[test]
    fn test_mongodb_serialization() {
        let name = "Test Project".to_string();
        let description = "Test Description".to_string();

        let mut project = Project {
            id: None,
            name: name.clone(),
            description: description.clone(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let id = Uuid::new();
        project.id = Some(id);

        let doc: Document = project.clone().into();
        let converted: Project = doc.into();

        assert_eq!(converted.id, project.id);
        assert_eq!(converted.name, project.name);
        assert_eq!(converted.description, project.description);
        assert_eq!(converted.enabled, project.enabled);
        assert_eq!(converted.created_at, project.created_at);
        assert_eq!(converted.updated_at, project.updated_at);
    }
}
