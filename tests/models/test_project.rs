use buraq::models::project::Project;
use mongodb::bson::oid::ObjectId;
use chrono::{DateTime, Utc};

#[test]
fn test_project_new() {
    let name = "Test Project".to_string();
    let description = "Test Description".to_string();
    
    let project = Project::new(name.clone(), description.clone());
    
    // Verify fields are set correctly
    assert!(ObjectId::parse_str(&project.id.to_string()).is_ok());
    assert_eq!(project.name, name);
    assert_eq!(project.description, description);
    
    // Verify timestamps are recent
    let now = Utc::now();
    assert!(project.created_at <= now);
    assert!(project.updated_at <= now);
}

#[test]
fn test_project_serialization() {
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
