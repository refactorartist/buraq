use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;
use chrono::{DateTime, Utc};

/// Represents a project with metadata and timestamps.
///
/// # Fields
/// - `id`: Unique identifier for the project (MongoDB ObjectId)
/// - `name`: Name of the project
/// - `description`: Description of the project
/// - `created_at`: Timestamp when project was created
/// - `updated_at`: Timestamp when project was last updated
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    id: ObjectId,
    name: String,
    description: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Project {
    /// Creates a new Project with the given name and description.
    ///
    /// Automatically generates:
    /// - A new ObjectId
    /// - Current UTC timestamps for created_at and updated_at
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the project
    /// * `description` - Description of the project
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::project::Project;
    ///
    /// let project = Project::new("My Project".to_string(), "Project description".to_string());
    /// assert_eq!(project.name(), "My Project");
    /// assert_eq!(project.description(), "Project description");
    /// ```
    pub fn new(name: String, description: String) -> Self {
        Self { 
            id: ObjectId::new(), 
            name, 
            description, 
            created_at: Utc::now(), 
            updated_at: Utc::now() 
        }
    }

    /// Returns the project's unique identifier
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    /// Returns the project's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the project's description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the project's creation timestamp
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Returns the project's last update timestamp
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_project() {
        let name = "Test Project".to_string();
        let description = "Test Description".to_string();
        
        let project = Project::new(name.clone(), description.clone());
        
        // Verify fields are set correctly
        assert!(ObjectId::parse_str(project.id.to_string()).is_ok());
        assert_eq!(project.name, name);
        assert_eq!(project.description, description);
        
        // Verify timestamps are recent
        let now = Utc::now();
        assert!(project.created_at <= now);
        assert!(project.updated_at <= now);
    }

    #[test]
    fn test_serialization() {
        let project = Project::new("Test".to_string(), "Description".to_string());
        
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
    }
}
