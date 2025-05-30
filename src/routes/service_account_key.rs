use crate::config::AppData;
use crate::models::pagination::Pagination;
use crate::models::service_account_key::{
    ServiceAccountKey, ServiceAccountKeyFilter, ServiceAccountKeySortableFields,
    ServiceAccountKeyUpdatePayload,
};
use crate::models::sort::{SortBuilder, SortDirection};
use crate::services::service_account_key_service::ServiceAccountKeyService;
use actix_web::{Error, HttpResponse, web};
use mongodb::bson::uuid::Uuid;

pub async fn create(
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

pub async fn read(
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

pub async fn update(
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

pub async fn delete(
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

pub async fn list(
    data: web::Data<AppData>,
    filter: Option<web::Query<ServiceAccountKeyFilter>>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServiceAccountKeyService::new(database.clone()).unwrap();
    let filter = filter.map_or_else(ServiceAccountKeyFilter::default, |q| q.into_inner());
    let sort = SortBuilder::new().add_sort(
        ServiceAccountKeySortableFields::Id,
        SortDirection::Ascending,
    );
    let service_account_keys = service
        .find(filter, Some(sort), Some(pagination.into_inner()))
        .await;

    match service_account_keys {
        Ok(keys) => Ok(HttpResponse::Ok().json(keys)),
        Err(e) => {
            println!("Error listing service account keys: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub fn configure_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/service_account_keys")
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
    use crate::test_utils::{cleanup_test_db, setup_test_db};
    use crate::types::Algorithm;
    use actix_web::{App, test};
    use chrono::{Duration, Utc};

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
                    .service(web::resource("").route(web::post().to(create))),
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
                    .service(web::resource("/{id}").route(web::get().to(read))),
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
            .uri(&format!(
                "/service_account_keys/{}",
                created_key.id.unwrap()
            ))
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
                    .service(web::resource("/{id}").route(web::patch().to(update))),
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
            .uri(&format!(
                "/service_account_keys/{}",
                created_key.id.unwrap()
            ))
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
                    .service(web::resource("/{id}").route(web::delete().to(delete))),
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
            .uri(&format!(
                "/service_account_keys/{}",
                created_key.id.unwrap()
            ))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());

        let resp = test::TestRequest::get()
            .uri(&format!(
                "/service_account_keys/{}",
                created_key.id.unwrap()
            ))
            .send_request(&app)
            .await;

        assert!(resp.status().is_client_error());

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_list_service_account_keys_success() {
        let db = setup_test_db("service_account_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_account_keys")
                    .service(web::resource("").route(web::get().to(list))),
            ),
        )
        .await;

        // Create multiple keys
        for i in 1..=3 {
            let key = ServiceAccountKey {
                id: None,
                service_account_id: Uuid::new(),
                algorithm: Algorithm::RSA,
                key: format!("test-key-{}", i),
                expires_at: Utc::now() + Duration::hours(1),
                enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            };
            ServiceAccountKeyService::new(app_data.database.clone().unwrap())
                .unwrap()
                .create(key)
                .await
                .unwrap();
        }

        let resp = test::TestRequest::get()
            .uri("/service_account_keys")
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let keys: Vec<ServiceAccountKey> = test::read_body_json(resp).await;
        assert_eq!(keys.len(), 3);

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_list_service_account_keys_with_pagination() {
        let db = setup_test_db("service_account_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_account_keys")
                    .service(web::resource("").route(web::get().to(list))),
            ),
        )
        .await;

        // Create multiple keys
        for i in 1..=5 {
            let key = ServiceAccountKey {
                id: None,
                service_account_id: Uuid::new(),
                algorithm: Algorithm::RSA,
                key: format!("test-key-{}", i),
                expires_at: Utc::now() + Duration::hours(1),
                enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            };
            ServiceAccountKeyService::new(app_data.database.clone().unwrap())
                .unwrap()
                .create(key)
                .await
                .unwrap();
        }

        let resp = test::TestRequest::get()
            .uri("/service_account_keys?limit=2&page=1")
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let keys: Vec<ServiceAccountKey> = test::read_body_json(resp).await;
        assert_eq!(keys.len(), 2);

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_list_service_account_keys_with_enabled_filter() {
        let db = setup_test_db("service_account_key_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_account_keys")
                    .service(web::resource("").route(web::get().to(list))),
            ),
        )
        .await;

        // Create multiple keys
        for i in 1..=5 {
            let key = ServiceAccountKey {
                id: None,
                service_account_id: Uuid::new(),
                algorithm: Algorithm::RSA,
                key: format!("test-key-{}", i),
                expires_at: Utc::now() + Duration::hours(1),
                enabled: i % 2 == 0,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            };
            ServiceAccountKeyService::new(app_data.database.clone().unwrap())
                .unwrap()
                .create(key)
                .await
                .unwrap();
        }

        let resp = test::TestRequest::get()
            .uri("/service_account_keys?is_enabled=true")
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let keys: Vec<ServiceAccountKey> = test::read_body_json(resp).await;
        assert_eq!(keys.len(), 2);

        cleanup_test_db(db).await.unwrap();
    }
}
