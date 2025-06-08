use crate::config::AppData;
use crate::models::pagination::Pagination;
use crate::models::server_key::{ServerKeyCreatePayload, ServerKeyFilter, ServerKeyUpdatePayload};
use crate::services::server_key_service::ServerKeyService;
use actix_web::{Error, HttpResponse, web};
use mongodb::bson::uuid::Uuid;

/// Handler to create a new server key.
pub async fn create(
    data: web::Data<AppData>,
    payload: web::Json<ServerKeyCreatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServerKeyService::new(database.clone())
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let server_key = service.create(payload.into_inner()).await;
    match server_key {
        Ok(server_key) => Ok(HttpResponse::Ok().json(server_key)),
        Err(e) => {
            println!("Error creating server key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to retrieve a server key by its ID.
pub async fn read(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServerKeyService::new(database.clone())
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let server_key_id = Uuid::parse_str(path.into_inner())
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))?;
    let server_key = service.get(server_key_id).await;
    match server_key {
        Ok(Some(server_key_read)) => Ok(HttpResponse::Ok().json(server_key_read)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting server key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to update an existing server key.
pub async fn update(
    data: web::Data<AppData>,
    path: web::Path<String>,
    payload: web::Json<ServerKeyUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServerKeyService::new(database.clone())
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let server_key_id = Uuid::parse_str(path.into_inner())
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))?;

    let server_key = service.update(server_key_id, payload.into_inner()).await;

    match server_key {
        Ok(server_key) => Ok(HttpResponse::Ok().json(server_key)),
        Err(e) => {
            println!("Error updating server key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to delete a server key by its ID.
pub async fn delete(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServerKeyService::new(database.clone())
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let server_key_id = Uuid::parse_str(path.into_inner())
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))?;

    let result = service.delete(server_key_id).await;

    match result {
        Ok(deleted) => {
            if deleted {
                Ok(HttpResponse::NoContent().finish())
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        }
        Err(e) => {
            println!("Error deleting server key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to list server keys with filtering and pagination.
pub async fn list(
    data: web::Data<AppData>,
    filter: Option<web::Query<ServerKeyFilter>>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServerKeyService::new(database.clone())
        .map_err(actix_web::error::ErrorInternalServerError)?;

    dbg!(filter.clone());
    dbg!(pagination.clone());
    dbg!(&service);

    let pagination = pagination.into_inner();
    let filter = filter.map(|f| f.into_inner()).unwrap_or_default();

    let result = service.find(filter, None, Some(pagination)).await;

    match result {
        Ok(server_key_reads) => Ok(HttpResponse::Ok().json(server_key_reads)),
        Err(e) => {
            println!("Error listing server keys: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Configures the routes for server keys.
pub fn configure_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/server-keys")
            .service(
                web::resource("")
                    .route(web::post().to(create))
                    .route(web::get().to(list)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(read))
                    .route(web::patch().to(update))
                    .route(web::delete().to(delete)),
            ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::server_key::ServerKeyRead;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use actix_web::{App, test};
    use jsonwebtoken::Algorithm;

    #[actix_web::test]
    async fn test_list_server_keys_no_filter() {
        // Setup
        let db = setup_test_db("server_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new()
                .app_data(app_data.clone())
                .configure(configure_routes),
        )
        .await;

        // Create server keys
        for _ in 0..5 {
            let payload = ServerKeyCreatePayload {
                environment_id: Uuid::new(),
                algorithm: Algorithm::HS256,
            };
            let _ = test::TestRequest::post()
                .uri("/server-keys")
                .set_json(&payload)
                .send_request(&app)
                .await;
        }

        // Test listing
        let resp = test::TestRequest::get()
            .uri("/server-keys")
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let server_keys: Vec<ServerKeyRead> = test::read_body_json(resp).await;
        assert_eq!(server_keys.len(), 5);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_list_server_keys_with_filter() {
        // Set the master encryption key for testing
        // Setup
        let db = setup_test_db("server_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new()
                .app_data(app_data.clone())
                .configure(configure_routes),
        )
        .await;

        // Create server keys
        let environment_id = Uuid::new();
        for _ in 0..5 {
            let payload = ServerKeyCreatePayload {
                environment_id,
                algorithm: Algorithm::HS256,
            };
            let _ = test::TestRequest::post()
                .uri("/server-keys")
                .set_json(&payload)
                .send_request(&app)
                .await;
        }

        // Test listing with filter
        let filter = ServerKeyFilter {
            environment_id: Some(environment_id),
            ..Default::default()
        };

        let resp = test::TestRequest::get()
            .uri("/server-keys")
            .set_json(&filter)
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 200);
        let server_keys: Vec<ServerKeyRead> = test::read_body_json(resp).await;
        assert_eq!(server_keys.len(), 5);
        assert!(
            server_keys
                .iter()
                .all(|k| k.environment_id == environment_id)
        );

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_create_server_key_success() {
        // Setup
        let db = setup_test_db("server_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new()
                .app_data(app_data.clone())
                .configure(configure_routes),
        )
        .await;

        let environment_id = Uuid::new();

        // Create server key
        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::HS256,
        };

        let resp = test::TestRequest::post()
            .uri("/server-keys")
            .set_json(&payload)
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 200);
        let created_key: ServerKeyRead = test::read_body_json(resp).await;
        assert!(!created_key.id.to_string().is_empty());
        assert_eq!(created_key.environment_id, environment_id);
        assert_eq!(created_key.algorithm, Algorithm::HS256);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_get_server_key_success() {
        // Set the master encryption key for testing
        let db = setup_test_db("server_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new()
                .app_data(app_data.clone())
                .configure(configure_routes),
        )
        .await;

        let environment_id = Uuid::new();

        // Create server key
        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::HS256,
        };

        let resp = test::TestRequest::post()
            .uri("/server-keys")
            .set_json(&payload)
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 200);
        let created_key: ServerKeyRead = test::read_body_json(resp).await;

        // Get server key
        let resp = test::TestRequest::get()
            .uri(&format!("/server-keys/{}", created_key.id))
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 200);
        let retrieved_key: ServerKeyRead = test::read_body_json(resp).await;
        assert_eq!(retrieved_key.id, created_key.id);
        assert_eq!(retrieved_key.environment_id, environment_id);
        assert_eq!(retrieved_key.algorithm, Algorithm::HS256);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_update_server_key_success() {
        // Setup
        let db = setup_test_db("server_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new()
                .app_data(app_data.clone())
                .configure(configure_routes),
        )
        .await;

        let environment_id = Uuid::new();

        // Create server key
        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::HS256,
        };

        let resp = test::TestRequest::post()
            .uri("/server-keys")
            .set_json(&payload)
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 200);
        let created_key: ServerKeyRead = test::read_body_json(resp).await;

        // Update server key
        let update_payload = ServerKeyUpdatePayload {
            key: Some("updated-key".to_string()),
            environment_id: Some(environment_id),
            algorithm: Some(Algorithm::HS256),
        };

        let resp = test::TestRequest::patch()
            .uri(&format!("/server-keys/{}", created_key.id))
            .set_json(&update_payload)
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 200);
        let updated_key: ServerKeyRead = test::read_body_json(resp).await;
        assert_eq!(updated_key.environment_id, environment_id);
        assert_eq!(updated_key.algorithm, Algorithm::HS256);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_delete_server_key_success() {
        // Setup
        let db = setup_test_db("server_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new()
                .app_data(app_data.clone())
                .configure(configure_routes),
        )
        .await;

        // Create server key
        let environment_id = Uuid::new();
        let payload = ServerKeyCreatePayload {
            environment_id,
            algorithm: Algorithm::HS256,
        };

        let resp = test::TestRequest::post()
            .uri("/server-keys")
            .set_json(&payload)
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 200);
        let created_key: ServerKeyRead = test::read_body_json(resp).await;

        // Delete server key
        let resp = test::TestRequest::delete()
            .uri(&format!("/server-keys/{}", created_key.id))
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 204);

        // Verify deletion
        let resp = test::TestRequest::get()
            .uri(&format!("/server-keys/{}", created_key.id))
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 404);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
}
