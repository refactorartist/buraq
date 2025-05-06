use buraq::utils::database::create_database_client;
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_create_database_client_success() {
    let database_uri = "mongodb://localhost:27017";
    let client_result = create_database_client(database_uri).await;
    assert!(client_result.is_ok());

    let client = client_result.unwrap();
    assert!(Arc::strong_count(&client) > 0);
}

#[tokio::test]
async fn test_create_database_client_invalid_uri() {
    let invalid_database_uri = "invalid_uri";
    let client_result = create_database_client(invalid_database_uri).await;
    assert!(client_result.is_err());
}
