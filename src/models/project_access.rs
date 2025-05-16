use mongodb::bson::uuid::Uuid;
use mongodb::bson::{Document, to_document, from_document};
use serde::{Deserialize, Serialize};

/// Represents access control configuration for a project environment.
///
/// # Fields
/// - `id`: Unique identifier for the project access
/// - `name`: Name of the access configuration
/// - `environment_id`: Foreign key reference to the associated environment
/// - `service_account_id`: Foreign key reference to the associated service account
/// - `project_scopes`: Array of project scope IDs this access is granted
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectAccess {    
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub name: String,
    pub environment_id: Uuid,
    pub service_account_id: Uuid,
    pub project_scopes: Vec<Uuid>,
}


impl From<ProjectAccess> for Document {
    fn from(value: ProjectAccess) -> Self {
        to_document(&value).unwrap()
    }
}

impl From<Document> for ProjectAccess {
    fn from(value: Document) -> Self {
        from_document(value.clone()).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectAccessUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_scopes: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectAccessFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_account_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_scopes: Option<Vec<Uuid>>,
}

impl From<ProjectAccessFilter> for Document {
    fn from(value: ProjectAccessFilter) -> Self {
        let mut doc = Document::new();
        if let Some(env_id) = value.environment_id {
            doc.insert("environment_id", env_id);
        }
        if let Some(sa_id) = value.service_account_id {
            doc.insert("service_account_id", sa_id);
        }
        if let Some(scopes) = value.project_scopes {
            doc.insert("project_scopes", scopes);
        }
        doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_access_creation() {
        let name = "Test Access".to_string();
        let environment_id = Uuid::new();
        let service_account_id = Uuid::new();
        let project_scopes = vec![Uuid::new(), Uuid::new()];

        let access = ProjectAccess {
            id: None,
            name: name.clone(),
            environment_id,
            service_account_id,
            project_scopes: project_scopes.clone(),
        };

        assert!(access.id.is_none());
        assert_eq!(access.name, name);
        assert_eq!(access.environment_id, environment_id);
        assert_eq!(access.service_account_id, service_account_id);
        assert_eq!(access.project_scopes, project_scopes);
    }

    #[test]
    fn test_project_access_document_conversion() {
        let access = ProjectAccess {
            id: Some(Uuid::new()),
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
        };

        let doc: Document = access.clone().into();
        let converted: ProjectAccess = doc.into();

        assert_eq!(access.id, converted.id);
        assert_eq!(access.name, converted.name);
        assert_eq!(access.environment_id, converted.environment_id);
        assert_eq!(access.service_account_id, converted.service_account_id);
        assert_eq!(access.project_scopes, converted.project_scopes);
    }

    #[test]
    fn test_project_access_update_payload() {
        let update = ProjectAccessUpdatePayload {
            name: Some("New Name".to_string()),
            project_scopes: Some(vec![Uuid::new()]),
        };

        assert_eq!(update.name.unwrap(), "New Name");
        assert_eq!(update.project_scopes.unwrap().len(), 1);
    }

    #[test]
    fn test_project_access_filter() {
        let filter = ProjectAccessFilter {
            environment_id: Some(Uuid::new()),
            service_account_id: Some(Uuid::new()),
            project_scopes: Some(vec![Uuid::new()]),
        };

        let doc: Document = filter.into();
        
        assert!(doc.contains_key("environment_id"));
        assert!(doc.contains_key("service_account_id"));
        assert!(doc.contains_key("project_scopes"));
    }
}
