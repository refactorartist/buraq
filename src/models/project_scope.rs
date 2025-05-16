use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, from_document, to_document};
use serde::{Deserialize, Serialize};

/// Represents a project scope that defines permissions within a project.
///
/// # Fields
/// - `id`: Unique identifier for the project scope (UUID)
/// - `project_id`: Foreign key reference to the associated project
/// - `name`: Name of the scope
/// - `description`: Description of what the scope allows
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectScope {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub project_id: Uuid,
    pub name: String,
    pub description: String,
}

impl From<ProjectScope> for Document {
    fn from(value: ProjectScope) -> Self {
        to_document(&value).unwrap()
    }
}

impl From<Document> for ProjectScope {
    fn from(value: Document) -> Self {
        from_document(value.clone()).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectScopeUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectScopeFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl From<ProjectScopeFilter> for Document {
    fn from(value: ProjectScopeFilter) -> Self {
        let mut doc = Document::new();
        if let Some(project_id) = value.project_id {
            doc.insert("project_id", project_id);
        }
        if let Some(name) = value.name {
            doc.insert("name", name);
        }
        doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_project_scope() {
        let project_id = Uuid::new();
        let name = "read:users".to_string();
        let description = "Allows reading user data".to_string();

        let scope = ProjectScope {
            id: None,
            project_id,
            name: name.clone(),
            description: description.clone(),
        };

        assert!(scope.id.is_none());
        assert_eq!(scope.project_id, project_id);
        assert_eq!(scope.name, name);
        assert_eq!(scope.description, description);
    }

    #[test]
    fn test_document_conversion() {
        let project_id = Uuid::new();
        let name = "write:posts".to_string();
        let description = "Allows creating and updating posts".to_string();

        let mut scope = ProjectScope {
            id: None,
            project_id: project_id,
            name: name.clone(),
            description: description.clone(),
        };
        let id = Uuid::new();
        scope.id = Some(id);

        let doc: Document = scope.clone().into();
        let converted: ProjectScope = doc.into();

        assert_eq!(converted.id, scope.id);
        assert_eq!(converted.project_id, project_id);
        assert_eq!(converted.name, name);
        assert_eq!(converted.description, description);
    }

    #[test]
    fn test_project_scope_filter() {
        let project_id = Uuid::new();
        let filter = ProjectScopeFilter {
            project_id: Some(project_id),
            name: Some("test-scope".to_string()),
        };

        let doc: Document = filter.into();

        assert!(doc.contains_key("project_id"));
        assert!(doc.contains_key("name"));
    }
}
