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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    name: String,
    description: String,
    enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Project {
    /// Creates a new Project with the given name and description.
    ///
    /// Automatically generates:
    /// - Current UTC timestamps for created_at and updated_at
    /// - Sets enabled to true by default
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
    /// assert!(project.enabled());
    /// ```
    pub fn new(name: String, description: String) -> Self {
        Self { 
            id: None,
            name, 
            description, 
            enabled: true,
            created_at: Utc::now(), 
            updated_at: Utc::now() 
        }
    }

    /// Returns the project's unique identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::project::Project;
    /// use mongodb::bson::oid::ObjectId;
    ///
    /// let mut project = Project::new(
    ///     "My Project".to_string(),
    ///     "Project description".to_string()
    /// );
    ///
    /// assert!(project.id().is_none());
    ///
    /// let id = ObjectId::new();
    /// project.set_id(id);
    /// assert!(project.id().is_some());
    /// ```
    pub fn id(&self) -> Option<&ObjectId> {
        self.id.as_ref()
    }

    /// Sets the project's unique identifier
    pub fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }

    /// Returns the project's name
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::project::Project;
    ///
    /// let name = "My Project".to_string();
    /// let project = Project::new(
    ///     name.clone(),
    ///     "Project description".to_string()
    /// );
    ///
    /// assert_eq!(project.name(), name);
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the project's description
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::project::Project;
    ///
    /// let description = "Project description".to_string();
    /// let project = Project::new(
    ///     "My Project".to_string(),
    ///     description.clone()
    /// );
    ///
    /// assert_eq!(project.description(), description);
    /// ```
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the project's enabled status
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::project::Project;
    ///
    /// let project = Project::new(
    ///     "My Project".to_string(),
    ///     "Project description".to_string()
    /// );
    ///
    /// // Projects are enabled by default
    /// assert!(project.enabled());
    /// ```
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the project's creation timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::project::Project;
    /// use chrono::Utc;
    ///
    /// let before = Utc::now();
    /// let project = Project::new(
    ///     "My Project".to_string(),
    ///     "Project description".to_string()
    /// );
    /// let after = Utc::now();
    ///
    /// // Verify timestamp is between before and after
    /// assert!(project.created_at() >= &before);
    /// assert!(project.created_at() <= &after);
    /// ```
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Returns the project's last update timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use buraq::models::project::Project;
    /// use chrono::Utc;
    ///
    /// let before = Utc::now();
    /// let project = Project::new(
    ///     "My Project".to_string(),
    ///     "Project description".to_string()
    /// );
    /// let after = Utc::now();
    ///
    /// // Verify timestamp is between before and after
    /// assert!(project.updated_at() >= &before);
    /// assert!(project.updated_at() <= &after);
    /// ```
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
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
    fn test_new_project() {
        let name = "Test Project".to_string();
        let description = "Test Description".to_string();
        
        let project = Project::new(name.clone(), description.clone());
        
        // Verify fields are set correctly
        assert!(project.id().is_none());
        assert_eq!(project.name(), name);
        assert_eq!(project.description(), description);
        assert!(project.enabled());
        // Verify timestamps are recent
        let now = Utc::now();
        assert!(project.created_at() <= &now);
        assert!(project.updated_at() <= &now);
    }

    #[test]
    fn test_serialization() {
        let name = "Test Project".to_string();
        let description = "Test Description".to_string();
        let project = Project::new(name.clone(), description.clone());
        
        // Test serialization
        let serialized = serde_json::to_string(&project);
        assert!(serialized.is_ok());
        
        // Test deserialization
        let deserialized: Result<Project, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
        
        let deserialized_project = deserialized.unwrap();
        assert_eq!(project.id(), deserialized_project.id());
        assert_eq!(project.name(), deserialized_project.name());
        assert_eq!(project.description(), deserialized_project.description());
        assert_eq!(project.enabled(), deserialized_project.enabled());
        assert_eq!(project.created_at(), deserialized_project.created_at());
        assert_eq!(project.updated_at(), deserialized_project.updated_at());
    }

    #[test]
    fn test_mongodb_serialization() {
        let name = "Test Project".to_string();
        let description = "Test Description".to_string();
        
        let mut project = Project::new(name.clone(), description.clone());
        let id = ObjectId::new();
        project.set_id(id);

        // Test conversion to BSON Document
        let doc = project.to_document().unwrap();
        
        // Test conversion from BSON Document
        let deserialized = Project::from_document(doc).unwrap();

        assert_eq!(deserialized.id(), project.id());
        assert_eq!(deserialized.name(), project.name());
        assert_eq!(deserialized.description(), project.description());
        assert_eq!(deserialized.enabled(), project.enabled());
        assert_eq!(deserialized.created_at(), project.created_at());
        assert_eq!(deserialized.updated_at(), project.updated_at());
    }
}
