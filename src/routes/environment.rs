use crate::config::AppData;
use crate::models::environment::{Environment, EnvironmentUpdatePayload};
use crate::services::environment_service::EnvironmentService;
use mongodb::bson::uuid::Uuid;
use actix_web::{Error, HttpResponse, web};

/// Handler to create a new environment.
pub async fn create_environment(
    data: web::Data<AppData>,
    environment: web::Json<Environment>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = EnvironmentService::new(database.clone())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let environment = service.create(environment.into_inner()).await;
    match environment {
        Ok(environment) => Ok(HttpResponse::Ok().json(environment)),
        Err(e) => {
            println!("Error creating environment: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to retrieve an environment by its ID.
pub async fn get_environment(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = EnvironmentService::new(database.clone())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let environment_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))?;
    let environment = service.get_environment(environment_id).await;
    match environment {
        Ok(Some(environment)) => Ok(HttpResponse::Ok().json(environment)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting environment: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to update an existing environment.
pub async fn update_environment(
    data: web::Data<AppData>,
    path: web::Path<String>,
    payload: web::Json<EnvironmentUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = EnvironmentService::new(database.clone()).unwrap();
    let environment_id = Uuid::parse_str(path.into_inner()).unwrap();

    let environment = service.update(environment_id, payload.into_inner()).await;

    match environment {
        Ok(environment) => Ok(HttpResponse::Ok().json(environment)),
        Err(e) => {
            println!("Error updating project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to delete an environment by its ID.
pub async fn delete_environment(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = EnvironmentService::new(database.clone()).unwrap();
    let environment_id = Uuid::parse_str(path.into_inner()).unwrap();

    let result = service.delete(environment_id).await;

    match result {
        Ok(deleted) => {
            if deleted {
                Ok(HttpResponse::NoContent().finish())
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        }
        Err(e) => {
            println!("Error deleting project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}
/// Configures the routes for environments.
pub fn configure_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/environments")
            .service(web::resource("").route(web::post().to(create_environment)))
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get_environment))
                    .route(web::patch().to(update_environment))
                    .route(web::delete().to(delete_environment)),
            ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use actix_web::{App, test};
    use chrono::Utc;
    

    #[actix_web::test]
    async fn test_create_environment_success() {
        // Setup
        let db = setup_test_db("environment_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/environments")
                    .service(web::resource("").route(web::post().to(create_environment)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_environment))
                            .route(web::patch().to(update_environment))
                            .route(web::delete().to(delete_environment)),
                    ),
            ),
        )
        .await;
        
        // Test
        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        let resp = test::TestRequest::post()
            .uri("/environments")
            .set_json(&environment)
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        let created_environment: Environment = test::read_body_json(resp).await;
        
        assert_eq!(created_environment.name, environment.name);
        assert_eq!(created_environment.description, environment.description);
        assert_eq!(created_environment.project_id, project_id);
        assert!(created_environment.enabled);
        assert!(created_environment.id.is_some());
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_get_environment_success() {
        // Setup
        let db = setup_test_db("environment_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/environments")
                    .service(web::resource("").route(web::post().to(create_environment)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_environment))
                            .route(web::patch().to(update_environment))
                            .route(web::delete().to(delete_environment)),
                    ),
            ),
        )
        .await;
        
        // First create an environment
        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        let resp = test::TestRequest::post()
            .uri("/environments")
            .set_json(&environment)
            .send_request(&app)
            .await;
        
        let created_environment: Environment = test::read_body_json(resp).await;
        let environment_id = created_environment.id.unwrap();
        
        // Then get the environment
        let resp = test::TestRequest::get()
            .uri(&format!("/environments/{}", environment_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        let retrieved_environment: Environment = test::read_body_json(resp).await;
        
        assert_eq!(retrieved_environment.id, created_environment.id);
        assert_eq!(retrieved_environment.name, created_environment.name);
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_get_nonexistent_environment() {
        // Setup
        let db = setup_test_db("environment_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/environments")
                    .service(web::resource("").route(web::post().to(create_environment)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_environment))
                            .route(web::patch().to(update_environment))
                            .route(web::delete().to(delete_environment)),
                    ),
            ),
        )
        .await;
        
        let nonexistent_id = Uuid::new();
        let resp = test::TestRequest::get()
            .uri(&format!("/environments/{}", nonexistent_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_client_error());
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_update_environment_success() {
        // Setup
        let db = setup_test_db("environment_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/environments")
                    .service(web::resource("").route(web::post().to(create_environment)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_environment))
                            .route(web::patch().to(update_environment))
                            .route(web::delete().to(delete_environment)),
                    ),
            ),
        )
        .await;
        
        // First create an environment
        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        let resp = test::TestRequest::post()
            .uri("/environments")
            .set_json(&environment)
            .send_request(&app)
            .await;
        
        let created_environment: Environment = test::read_body_json(resp).await;
        let environment_id = created_environment.id.unwrap();
        
        // Then update the environment
        let update_payload = EnvironmentUpdatePayload {
            name: Some("Updated Environment".to_string()),
            description: Some("Updated Description".to_string()),
            enabled: Some(false),
        };
        
        let resp = test::TestRequest::patch()
            .uri(&format!("/environments/{}", environment_id))
            .set_json(&update_payload)
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        let updated_environment: Environment = test::read_body_json(resp).await;
        
        assert_eq!(updated_environment.name, "Updated Environment");
        assert_eq!(updated_environment.description, "Updated Description");
        assert!(!updated_environment.enabled);
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_delete_environment_success() {
        // Setup
        let db = setup_test_db("environment_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/environments")
                    .service(web::resource("").route(web::post().to(create_environment)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_environment))
                            .route(web::patch().to(update_environment))
                            .route(web::delete().to(delete_environment)),
                    ),
            ),
        )
        .await;
        
        // First create an environment
        let project_id = Uuid::new();
        let environment = Environment {
            id: None,
            project_id,
            name: "Test Environment".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        let resp = test::TestRequest::post()
            .uri("/environments")
            .set_json(&environment)
            .send_request(&app)
            .await;
        
        let created_environment: Environment = test::read_body_json(resp).await;
        let environment_id = created_environment.id.unwrap();
        
        // Then delete the environment
        let resp = test::TestRequest::delete()
            .uri(&format!("/environments/{}", environment_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        
        // Verify environment is deleted
        let resp = test::TestRequest::get()
            .uri(&format!("/environments/{}", environment_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_client_error());
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_delete_nonexistent_environment() {
        // Setup
        let db = setup_test_db("environment_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/environments")
                    .service(web::resource("").route(web::post().to(create_environment)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_environment))
                            .route(web::patch().to(update_environment))
                            .route(web::delete().to(delete_environment)),
                    ),
            ),
        )
        .await;
        
        let nonexistent_id = Uuid::new();
        let resp = test::TestRequest::delete()
            .uri(&format!("/environments/{}", nonexistent_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_client_error());
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
}