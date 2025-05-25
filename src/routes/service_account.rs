use crate::config::AppData;
use crate::models::service_account::{ServiceAccount, ServiceAccountUpdatePayload};
use crate::services::service_account_service::ServiceAccountService;
use mongodb::bson::uuid::Uuid;
use actix_web::{Error, HttpResponse, web};

pub async fn create(
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
            println!("Error creating service account: {:?}", e);
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
    let service = ServiceAccountService::new(database.clone()).unwrap();
    let service_account_id = Uuid::parse_str(path.into_inner()).unwrap();
    let service_account = service.get_service_account(service_account_id).await;

    match service_account {
        Ok(Some(service_account)) => Ok(HttpResponse::Ok().json(service_account)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting service account: {:?}", e);
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
            println!("Error updating service account: {:?}", e);
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
            println!("Error deleting service account: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub fn configure_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/service_accounts")
            .service(web::resource("").route(web::post().to(create)))
            .service(
                web::resource("/{id}")
                    .route(web::get().to(read))
                    .route(web::patch().to(update_service_account))
                    .route(web::delete().to(delete)),
            ),
    );
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use actix_web::{test, App};
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use crate::models::service_account::ServiceAccount;

    #[actix_web::test]
    async fn test_create_service_account_success() {
        let db = setup_test_db("service_account_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_accounts")
                    .service(web::resource("").route(web::post().to(create))),
            ),
        )
        .await;

        let service_account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

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

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_get_service_account_success() {
        let db = setup_test_db("service_account_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_accounts")
                    .service(web::resource("/{id}").route(web::get().to(read))),
            ),
        )
        .await;

        let service_account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created_service_account = ServiceAccountService::new(Arc::new(db.clone()))
            .unwrap()
            .create(service_account.clone())
            .await
            .unwrap();

        let resp = test::TestRequest::get()
            .uri(&format!("/service_accounts/{}", created_service_account.id.unwrap()))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let retrieved_service_account: ServiceAccount = test::read_body_json(resp).await;
        assert_eq!(retrieved_service_account.email, service_account.email);
        assert_eq!(retrieved_service_account.user, service_account.user);

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_update_service_account_success() {
        let db = setup_test_db("service_account_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_accounts")
                    .service(web::resource("/{id}").route(web::patch().to(update_service_account))),
            ),
        )
        .await;

        let service_account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created_service_account = ServiceAccountService::new(Arc::new(db.clone()))
            .unwrap()
            .create(service_account.clone())
            .await
            .unwrap();

        let update_payload = ServiceAccountUpdatePayload {
            email: Some("new@example.com".to_string()),
            user: Some("newuser".to_string()),
            secret: Some("newsecret".to_string()),
            enabled: Some(false),
        };

        let resp = test::TestRequest::patch()
            .uri(&format!("/service_accounts/{}", created_service_account.id.unwrap()))
            .set_json(&update_payload)
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let updated_service_account: ServiceAccount = test::read_body_json(resp).await;
        assert_eq!(updated_service_account.email, "new@example.com");
        assert_eq!(updated_service_account.user, "newuser");
        assert_eq!(updated_service_account.secret, "newsecret");
        assert!(!updated_service_account.enabled);

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_delete_service_account_success() {
        let db = setup_test_db("service_account_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/service_accounts")
                    .service(web::resource("/{id}").route(web::delete().to(delete))),
            ),
        )
        .await;

        let service_account = ServiceAccount::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "secret123".to_string(),
        );

        let created_service_account = ServiceAccountService::new(Arc::new(db.clone()))
            .unwrap()
            .create(service_account.clone())
            .await
            .unwrap();

        let resp = test::TestRequest::delete()
            .uri(&format!("/service_accounts/{}", created_service_account.id.unwrap()))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());

        let resp = test::TestRequest::get()
            .uri(&format!("/service_accounts/{}", created_service_account.id.unwrap()))
            .send_request(&app)
            .await;

        assert!(resp.status().is_client_error());

        cleanup_test_db(db).await.unwrap();
    }
}
