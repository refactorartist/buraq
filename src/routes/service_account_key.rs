use actix_web::{web, HttpResponse, Error};
use crate::config::AppData;
use crate::models::service_account_key::{ServiceAccountKey, ServiceAccountKeyUpdatePayload};
use crate::services::service_account_key_service::ServiceAccountKeyService;
use mongodb::bson::uuid::Uuid;

pub async fn create_service_account_key(
    data: web::Data<AppData>,
    service_account_key: web::Json<ServiceAccountKey>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServiceAccountKeyService::new(database.clone()).unwrap();
    let service_account_key = service.create(service_account_key.into_inner()).await;

    match service_account_key {
        Ok(key) => Ok(HttpResponse::Ok().json(key)),
        Err(e) => {
            println!("Error creating service account key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn get_service_account_key(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServiceAccountKeyService::new(database.clone()).unwrap();
    let key_id = Uuid::parse_str(path.into_inner()).unwrap();
    let service_account_key = service.get_service_account_key(key_id).await;

    match service_account_key {
        Ok(Some(key)) => Ok(HttpResponse::Ok().json(key)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting service account key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn update_service_account_key(
    data: web::Data<AppData>,
    path: web::Path<String>,
    payload: web::Json<ServiceAccountKeyUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServiceAccountKeyService::new(database.clone()).unwrap();
    let key_id = Uuid::parse_str(path.into_inner()).unwrap();

    let service_account_key = service.update(key_id, payload.into_inner()).await;

    match service_account_key {
        Ok(key) => Ok(HttpResponse::Ok().json(key)),
        Err(e) => {
            println!("Error updating service account key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn delete_service_account_key(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServiceAccountKeyService::new(database.clone()).unwrap();
    let key_id = Uuid::parse_str(path.into_inner()).unwrap();

    let result = service.delete(key_id).await;

    match result {
        Ok(deleted) => {
            if deleted {
                Ok(HttpResponse::NoContent().finish())
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        }
        Err(e) => {
            println!("Error deleting service account key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub fn configure_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/service_account_keys")
            .service(web::resource("").route(web::post().to(create_service_account_key)))
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get_service_account_key))
                    .route(web::patch().to(update_service_account_key))
                    .route(web::delete().to(delete_service_account_key)),
            ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use actix_web::{App, test};
    use chrono::{Duration, Utc};
    use crate::types::Algorithm;

    #[actix_web::test]
    async fn test_create_service_account_key_success() {
        let db = setup_test_db("service_account_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_account_keys")
                    .service(web::resource("").route(web::post().to(create_service_account_key))),
            ),
        )
        .await;

        let now = Utc::now();
        let key = ServiceAccountKey {
            id: None,
            service_account_id: Uuid::new(),
            algorithm: Algorithm::RSA,
            key: "test-key".to_string(),
            expires_at: now + Duration::hours(1),
            enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        };

        let resp = test::TestRequest::post()
            .uri("/service_account_keys")
            .set_json(&key)
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let created_key: ServiceAccountKey = test::read_body_json(resp).await;
        assert_eq!(created_key.key, key.key);
        assert!(created_key.id.is_some());

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_get_service_account_key_success() {
        let db = setup_test_db("service_account_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_account_keys")
                    .service(web::resource("/{id}").route(web::get().to(get_service_account_key))),
            ),
        )
        .await;

        let now = Utc::now();
        let key = ServiceAccountKey {
            id: None,
            service_account_id: Uuid::new(),
            algorithm: Algorithm::RSA,
            key: "test-key".to_string(),
            expires_at: now + Duration::hours(1),
            enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        };

        let created_key = ServiceAccountKeyService::new(app_data.database.clone().unwrap())
            .unwrap()
            .create(key.clone())
            .await
            .unwrap();

        let resp = test::TestRequest::get()
            .uri(&format!("/service_account_keys/{}", created_key.id.unwrap()))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let retrieved_key: ServiceAccountKey = test::read_body_json(resp).await;
        assert_eq!(retrieved_key.id, created_key.id);

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_update_service_account_key_success() {
        let db = setup_test_db("service_account_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_account_keys")
                    .service(web::resource("/{id}").route(web::patch().to(update_service_account_key))),
            ),
        )
        .await;

        let now = Utc::now();
        let key = ServiceAccountKey {
            id: None,
            service_account_id: Uuid::new(),
            algorithm: Algorithm::RSA,
            key: "test-key".to_string(),
            expires_at: now + Duration::hours(1),
            enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        };

        let created_key = ServiceAccountKeyService::new(app_data.database.clone().unwrap())
            .unwrap()
            .create(key.clone())
            .await
            .unwrap();

        let update_payload = ServiceAccountKeyUpdatePayload {
            key: Some("new-key".to_string()),
            expires_at: Some(now + Duration::hours(2)),
            enabled: Some(false),
        };

        let resp = test::TestRequest::patch()
            .uri(&format!("/service_account_keys/{}", created_key.id.unwrap()))
            .set_json(&update_payload)
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let updated_key: ServiceAccountKey = test::read_body_json(resp).await;
        assert_eq!(updated_key.key, "new-key");
        assert!(!updated_key.enabled);

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_delete_service_account_key_success() {
        let db = setup_test_db("service_account_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_account_keys")
                    .service(web::resource("/{id}").route(web::delete().to(delete_service_account_key))),
            ),
        )
        .await;

        let now = Utc::now();
        let key = ServiceAccountKey {
            id: None,
            service_account_id: Uuid::new(),
            algorithm: Algorithm::RSA,
            key: "test-key".to_string(),
            expires_at: now + Duration::hours(1),
            enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        };

        let created_key = ServiceAccountKeyService::new(app_data.database.clone().unwrap())
            .unwrap()
            .create(key.clone())
            .await
            .unwrap();

        let resp = test::TestRequest::delete()
            .uri(&format!("/service_account_keys/{}", created_key.id.unwrap()))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());

        let resp = test::TestRequest::get()
            .uri(&format!("/service_account_keys/{}", created_key.id.unwrap()))
            .send_request(&app)
            .await;

        assert!(resp.status().is_client_error());

        cleanup_test_db(db).await.unwrap();
    }
}

