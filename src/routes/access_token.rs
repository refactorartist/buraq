use crate::config::AppData;
use crate::models::access_token::{AccessToken,AccessTokenUpdatePayload};
use crate::services::access_token_service::AccessTokenService;
use mongodb::bson::uuid::Uuid;

use actix_web::{Error, HttpResponse, web};


pub async fn create_access_token(
    data: web::Data<AppData>,
    access_token: web::Json<AccessToken>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = AccessTokenService::new(database.clone()).unwrap();
    let access_token = service.create(access_token.into_inner()).await;

    match access_token {
        Ok(access_token) => Ok(HttpResponse::Ok().json(access_token)),
        Err(e) => {
            println!("Error creating project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}


pub async fn get_access_token(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = AccessTokenService::new(database.clone()).unwrap();
    let access_token_id = Uuid::parse_str(path.into_inner()).unwrap();
    let access_token = service.get_access_token(access_token_id).await;

    match access_token {
        Ok(Some(access_token)) => Ok(HttpResponse::Ok().json(access_token)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn update_access_token(
    data: web::Data<AppData>,
    path: web::Path<String>,
    payload: web::Json<AccessTokenUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = AccessTokenService::new(database.clone()).unwrap();
    let access_token_id = Uuid::parse_str(path.into_inner()).unwrap();

    let access_token = service.update(access_token_id, payload.into_inner()).await;

    match access_token {
        Ok(access_token) => Ok(HttpResponse::Ok().json(access_token)),
        Err(e) => {
            println!("Error updating project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}


pub async fn delete_access_token(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = AccessTokenService::new(database.clone()).unwrap();
    let access_token_id = Uuid::parse_str(path.into_inner()).unwrap();

    let result = service.delete(access_token_id).await;

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




pub fn configure_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/projects")
            .service(web::resource("").route(web::post().to(create_access_token)))
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get_access_token))
                    .route(web::patch().to(update_access_token))
                    .route(web::delete().to(delete_access_token)),
            ),
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use actix_web::{App, test};
    use chrono::{Utc, Duration};
    use crate::types::Algorithm;
    
    #[actix_web::test]
    async fn test_create_access_token_success() {
        // Setup
        let db = setup_test_db("access_token_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/access_tokens")
                    .service(web::resource("").route(web::post().to(create_access_token)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_access_token))
                            .route(web::patch().to(update_access_token))
                            .route(web::delete().to(delete_access_token)),
                    ),
            ),
        )
        .await;
        
        // Test
        let now = Utc::now();
        let expires = now + Duration::hours(1);
        let access_token = AccessToken {
            id: None,
            key: "test-key-create".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: expires,
            created_at: now,
            enabled: true,
        };
        
        let resp = test::TestRequest::post()
            .uri("/access_tokens")
            .set_json(&access_token)
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        let created_access_token: AccessToken = test::read_body_json(resp).await;
        
        assert_eq!(created_access_token.key, access_token.key);
        assert_eq!(created_access_token.algorithm, access_token.algorithm);
        assert!(created_access_token.id.is_some());
        assert!(created_access_token.enabled);
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
    
    #[actix_web::test]
    async fn test_get_access_token_success() {
        // Setup
        let db = setup_test_db("access_token_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/access_tokens")
                    .service(web::resource("").route(web::post().to(create_access_token)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_access_token))
                            .route(web::patch().to(update_access_token))
                            .route(web::delete().to(delete_access_token)),
                    ),
            ),
        )
        .await;
        
        // First create an access token
        let now = Utc::now();
        let expires = now + Duration::hours(1);
        let access_token = AccessToken {
            id: None,
            key: "test-key-get".to_string(),
            algorithm: Algorithm::HMAC,
            expires_at: expires,
            created_at: now,
            enabled: true,
        };
        
        let resp = test::TestRequest::post()
            .uri("/access_tokens")
            .set_json(&access_token)
            .send_request(&app)
            .await;
        
        let created_access_token: AccessToken = test::read_body_json(resp).await;
        let access_token_id = created_access_token.id.unwrap();
        
        // Then get the access token
        let resp = test::TestRequest::get()
            .uri(&format!("/access_tokens/{}", access_token_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        let retrieved_access_token: AccessToken = test::read_body_json(resp).await;
        
        assert_eq!(retrieved_access_token.id, created_access_token.id);
        assert_eq!(retrieved_access_token.key, created_access_token.key);
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
    
    #[actix_web::test]
    async fn test_update_access_token_success() {
        // Setup
        let db = setup_test_db("access_token_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/access_tokens")
                    .service(web::resource("").route(web::post().to(create_access_token)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_access_token))
                            .route(web::patch().to(update_access_token))
                            .route(web::delete().to(delete_access_token)),
                    ),
            ),
        )
        .await;
        
        // First create an access token
        let now = Utc::now();
        let expires = now + Duration::hours(1);
        let access_token = AccessToken {
            id: None,
            key: "test-key-update".to_string(),
            algorithm: Algorithm::RSA,
            expires_at: expires,
            created_at: now,
            enabled: true,
        };
        
        let resp = test::TestRequest::post()
            .uri("/access_tokens")
            .set_json(&access_token)
            .send_request(&app)
            .await;
        
        let created_access_token: AccessToken = test::read_body_json(resp).await;
        let access_token_id = created_access_token.id.unwrap();
        
        // Then update the access token
        let new_expires = now + Duration::hours(2);
        let update_payload = AccessTokenUpdatePayload {
            key: Some("updated-key".to_string()),
            expires_at: Some(new_expires),
            enabled: Some(false),
        };
        
        let resp = test::TestRequest::patch()
            .uri(&format!("/access_tokens/{}", access_token_id))
            .set_json(&update_payload)
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        let updated_access_token: AccessToken = test::read_body_json(resp).await;
        
        assert_eq!(updated_access_token.key, "updated-key");
        assert_eq!(updated_access_token.expires_at, new_expires);
        assert!(!updated_access_token.enabled);
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
    
    #[actix_web::test]
    async fn test_delete_access_token_success() {
        // Setup
        let db = setup_test_db("access_token_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/access_tokens")
                    .service(web::resource("").route(web::post().to(create_access_token)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_access_token))
                            .route(web::patch().to(update_access_token))
                            .route(web::delete().to(delete_access_token)),
                    ),
            ),
        )
        .await;
        
        // First create an access token
        let now = Utc::now();
        let expires = now + Duration::hours(1);
        let access_token = AccessToken {
            id: None,
            key: "test-key-delete".to_string(),
            algorithm: Algorithm::HMAC,
            expires_at: expires,
            created_at: now,
            enabled: true,
        };
        
        let resp = test::TestRequest::post()
            .uri("/access_tokens")
            .set_json(&access_token)
            .send_request(&app)
            .await;
        
        let created_access_token: AccessToken = test::read_body_json(resp).await;
        let access_token_id = created_access_token.id.unwrap();
        
        // Then delete the access token
        let resp = test::TestRequest::delete()
            .uri(&format!("/access_tokens/{}", access_token_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        
        // Verify access token is deleted
        let resp = test::TestRequest::get()
            .uri(&format!("/access_tokens/{}", access_token_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_client_error());
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
    
    #[actix_web::test]
    async fn test_get_nonexistent_access_token() {
        // Setup
        let db = setup_test_db("access_token_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/access_tokens")
                    .service(web::resource("").route(web::post().to(create_access_token)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_access_token))
                            .route(web::patch().to(update_access_token))
                            .route(web::delete().to(delete_access_token)),
                    ),
            ),
        )
        .await;
        
        let nonexistent_id = Uuid::new();
        let resp = test::TestRequest::get()
            .uri(&format!("/access_tokens/{}", nonexistent_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_client_error());
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
}