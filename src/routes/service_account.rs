use crate::config::AppData;
use crate::models::service_account::{ServiceAccount,ServiceAccountUpdatePayload};
use crate::services::service_account_service::ServiceAccountService;
use mongodb::bson::uuid::Uuid;

use actix_web::{Error,HttpResponse,web};


pub async fn create_service_account(
    data: web::Data<AppData>,
    service_account: web::Json<ServiceAccount>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServiceAccountService::new(database.clone()).unwrap();
    let service_account = service.create(service_account.into_inner()).await;

    match service_account {
        Ok(service_account) => Ok(HttpResponse::Ok().json(service_account)),
        Err(e) => {
            println!("Error creating project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}



pub async fn get_service_account(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServiceAccountService::new(database.clone()).unwrap();
    let service_account_id = Uuid::parse_str(path.into_inner()).unwrap();
    let service_account = service.get_service_account(service_account_id).await;

    match service_account {
        Ok(Some(project)) => Ok(HttpResponse::Ok().json(project)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}


pub async fn update_service_account(
    data: web::Data<AppData>,
    path: web::Path<String>,
    payload: web::Json<ServiceAccountUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServiceAccountService::new(database.clone()).unwrap();
    let service_account_id = Uuid::parse_str(path.into_inner()).unwrap();

    let service_account = service.update(service_account_id, payload.into_inner()).await;

    match service_account {
        Ok(service_account) => Ok(HttpResponse::Ok().json(service_account)),
        Err(e) => {
            println!("Error updating project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}


pub async fn delete_service_account(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServiceAccountService::new(database.clone()).unwrap();
    let service_account_id = Uuid::parse_str(path.into_inner()).unwrap();

    let result = service.delete(service_account_id).await;

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
            .service(web::resource("").route(web::post().to(create_service_account)))
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get_service_account))
                    .route(web::patch().to(update_service_account))
                    .route(web::delete().to(delete_service_account)),
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
    async fn test_create_service_account_success() {
        // Setup
        let db = setup_test_db("service_account_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_accounts")
                    .service(web::resource("").route(web::post().to(create_service_account)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_service_account))
                            .route(web::patch().to(update_service_account))
                            .route(web::delete().to(delete_service_account)),
                    ),
            ),
        )
        .await;
        
        // Test
        let service_account = ServiceAccount {
            id: None,
            email: "test-create@example.com".to_string(),
            user: "test_create_user".to_string(),
            secret: "test_create_secret".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        let resp = test::TestRequest::post()
            .uri("/service_accounts")
            .set_json(&service_account)
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        let created_service_account: ServiceAccount = test::read_body_json(resp).await;
        
        assert_eq!(created_service_account.email, service_account.email);
        assert_eq!(created_service_account.user, service_account.user);
        assert!(created_service_account.id.is_some());
        assert!(created_service_account.enabled);
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
    
    #[actix_web::test]
    async fn test_get_service_account_success() {
        // Setup
        let db = setup_test_db("service_account_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_accounts")
                    .service(web::resource("").route(web::post().to(create_service_account)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_service_account))
                            .route(web::patch().to(update_service_account))
                            .route(web::delete().to(delete_service_account)),
                    ),
            ),
        )
        .await;
        
        // First create a service account
        let service_account = ServiceAccount {
            id: None,
            email: "test-get@example.com".to_string(),
            user: "test_get_user".to_string(),
            secret: "test_get_secret".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        let resp = test::TestRequest::post()
            .uri("/service_accounts")
            .set_json(&service_account)
            .send_request(&app)
            .await;
        
        let created_service_account: ServiceAccount = test::read_body_json(resp).await;
        let service_account_id = created_service_account.id.unwrap();
        
        // Then get the service account
        let resp = test::TestRequest::get()
            .uri(&format!("/service_accounts/{}", service_account_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        let retrieved_service_account: ServiceAccount = test::read_body_json(resp).await;
        
        assert_eq!(retrieved_service_account.id, created_service_account.id);
        assert_eq!(retrieved_service_account.email, created_service_account.email);
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
    
    #[actix_web::test]
    async fn test_update_service_account_success() {
        // Setup
        let db = setup_test_db("service_account_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_accounts")
                    .service(web::resource("").route(web::post().to(create_service_account)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_service_account))
                            .route(web::patch().to(update_service_account))
                            .route(web::delete().to(delete_service_account)),
                    ),
            ),
        )
        .await;
        
        // First create a service account
        let service_account = ServiceAccount {
            id: None,
            email: "test-update@example.com".to_string(),
            user: "test_update_user".to_string(),
            secret: "test_update_secret".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        let resp = test::TestRequest::post()
            .uri("/service_accounts")
            .set_json(&service_account)
            .send_request(&app)
            .await;
        
        let created_service_account: ServiceAccount = test::read_body_json(resp).await;
        let service_account_id = created_service_account.id.unwrap();
        
        // Then update the service account
        let update_payload = ServiceAccountUpdatePayload {
            email: Some("updated-email@example.com".to_string()),
            user: Some("updated_user".to_string()),
            secret: Some("updated_secret".to_string()),
            enabled: Some(false),
        };
        
        let resp = test::TestRequest::patch()
            .uri(&format!("/service_accounts/{}", service_account_id))
            .set_json(&update_payload)
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        let updated_service_account: ServiceAccount = test::read_body_json(resp).await;
        
        assert_eq!(updated_service_account.email, "updated-email@example.com");
        assert_eq!(updated_service_account.user, "updated_user");
        assert!(!updated_service_account.enabled);
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
    
    #[actix_web::test]
    async fn test_delete_service_account_success() {
        // Setup
        let db = setup_test_db("service_account_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_accounts")
                    .service(web::resource("").route(web::post().to(create_service_account)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_service_account))
                            .route(web::patch().to(update_service_account))
                            .route(web::delete().to(delete_service_account)),
                    ),
            ),
        )
        .await;
        
        // First create a service account
        let service_account = ServiceAccount {
            id: None,
            email: "test-delete@example.com".to_string(),
            user: "test_delete_user".to_string(),
            secret: "test_delete_secret".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        let resp = test::TestRequest::post()
            .uri("/service_accounts")
            .set_json(&service_account)
            .send_request(&app)
            .await;
        
        let created_service_account: ServiceAccount = test::read_body_json(resp).await;
        let service_account_id = created_service_account.id.unwrap();
        
        // Then delete the service account
        let resp = test::TestRequest::delete()
            .uri(&format!("/service_accounts/{}", service_account_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_success());
        
        // Verify service account is deleted
        let resp = test::TestRequest::get()
            .uri(&format!("/service_accounts/{}", service_account_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_client_error());
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
    
    #[actix_web::test]
    async fn test_get_nonexistent_service_account() {
        // Setup
        let db = setup_test_db("service_account_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });
        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_accounts")
                    .service(web::resource("").route(web::post().to(create_service_account)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_service_account))
                            .route(web::patch().to(update_service_account))
                            .route(web::delete().to(delete_service_account)),
                    ),
            ),
        )
        .await;
        
        let nonexistent_id = Uuid::new();
        let resp = test::TestRequest::get()
            .uri(&format!("/service_accounts/{}", nonexistent_id))
            .send_request(&app)
            .await;
        
        assert!(resp.status().is_client_error());
        
        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
}