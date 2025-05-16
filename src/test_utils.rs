use crate::config::AppConfig;
use crate::utils::database::create_database_client;
use anyhow::Result;
use dotenvy::dotenv;
use mongodb::Database;
use std::sync::atomic::{AtomicUsize, Ordering};

static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Sets up a test database with a unique name for each test.
///
/// # Arguments
///
/// * `prefix` - A string prefix to use in the database name to identify the test module
///
/// # Returns
///
/// Returns a Result containing the MongoDB Database instance or an error if setup fails.
pub async fn setup_test_db(prefix: &str) -> Result<Database> {
    dotenv().ok();

    let app_config = AppConfig::from_env(Some(true))?;
    let client = create_database_client(&app_config.application.database_uri).await?;

    // Create a unique database name for each test
    let db_num = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
    let db = client.database(&format!("test_db__{}_{}", prefix, db_num));

    // Ensure the database is clean before starting
    cleanup_test_db(db.clone()).await?;

    Ok(db)
}

/// Cleans up a test database by dropping all collections and the database itself.
///
/// # Arguments
///
/// * `db` - The MongoDB Database instance to clean up
///
/// # Returns
///
/// Returns a Result indicating success or failure of the cleanup operation.
pub async fn cleanup_test_db(db: Database) -> Result<()> {
    // Drop all collections in the database
    let collection_names: Vec<String> = db.list_collection_names().await?;
    for collection_name in collection_names {
        let collection = db.collection::<mongodb::bson::Document>(&collection_name);
        collection.drop().await?;
    }

    // Drop the database itself
    db.drop().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::Document;
    use mongodb::bson::doc;

    #[tokio::test]
    async fn test_setup_and_cleanup_test_db() -> Result<()> {
        let db = setup_test_db("test_utils").await?;
        assert!(db.name().starts_with("test_db__test_utils_"));

        // Verify we can create and drop collections
        let collection = db.collection::<Document>("test_collection");
        collection.insert_one(doc! { "test": "data" }).await?;

        // Cleanup should succeed
        cleanup_test_db(db).await?;
        Ok(())
    }
}
