use crate::config::AppData;
use crate::models::access_token::{
    AccessToken, AccessTokenCreatePayload, AccessTokenFilter, AccessTokenRead,
    AccessTokenSortableFields, AccessTokenUpdatePayload,
};
use crate::models::pagination::Pagination;
use crate::models::sort::{SortBuilder, SortDirection};
use crate::services::access_token_service::AccessTokenService;
use crate::utils::tokens::key_builder::KeyBuilder;
use chrono::Utc;
use mongodb::bson::uuid::Uuid;

use actix_web::{Error, HttpResponse, web};

pub async fn create(
    data: web::Data<AppData>,
    payload: web::Json<AccessTokenCreatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = AccessTokenService::new(database.clone()).unwrap();
    let private_key = KeyBuilder::new()
        .generate_key(payload.algorithm)
        .unwrap()
        .private_key;
    let access_token = AccessToken {
        id: None,
        project_access_id: payload.project_access_id,
        key: String::from_utf8(private_key).unwrap(),
        algorithm: payload.algorithm,
        expires_at: payload.expires_at,
        created_at: Utc::now(),
        enabled: true,
    };
    let access_token = service.create(access_token).await;

    match access_token {
        Ok(access_token) => Ok(HttpResponse::Ok().json(AccessTokenRead::from(access_token))),
        Err(e) => {
            println!("Error creating project: {:?}", e);
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
    let service = AccessTokenService::new(database.clone()).unwrap();
    let access_token_id = Uuid::parse_str(path.into_inner()).unwrap();
    let access_token = service.get_access_token(access_token_id).await;

    match access_token {
        Ok(Some(access_token)) => Ok(HttpResponse::Ok().json(AccessTokenRead::from(access_token))),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn update(
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
        Ok(access_token) => Ok(HttpResponse::Ok().json(AccessTokenRead::from(access_token))),
        Err(e) => {
            println!("Error updating project: {:?}", e);
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

pub async fn list(
    data: web::Data<AppData>,
    filter: Option<web::Query<AccessTokenFilter>>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;

    let service = AccessTokenService::new(database.clone()).unwrap();
    let sort = SortBuilder::new().add_sort(AccessTokenSortableFields::Id, SortDirection::Ascending);

    let filter = filter.map_or_else(AccessTokenFilter::default, |q| q.into_inner());

    let results = service
        .find(filter, Some(sort), Some(pagination.into_inner()))
        .await;

    let access_tokens = results.map(|access_tokens| {
        access_tokens
            .into_iter()
            .map(AccessTokenRead::from)
            .collect::<Vec<AccessTokenRead>>()
    });

    match access_tokens {
        Ok(access_tokens) => Ok(HttpResponse::Ok().json(access_tokens)),
        Err(e) => {
            println!("Error listing projects: {:?}", e);
            Err(actix_web::error::ErrorInternalServerError(e))
        }
    }
}

pub fn configure_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/access-tokens")
            .service(web::resource("").route(web::post().to(create)))
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
    use actix_web::{App, test};
    use chrono::{Duration, Utc};
    use jsonwebtoken::Algorithm;

    #[actix_web::test]
    async fn test_create_access_token_success() {
        // Setup
        let db = setup_test_db("access_token_routes").await.unwrap();
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

        // Test
        let payload = AccessTokenCreatePayload {
            project_access_id: Uuid::new(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
        };

        let resp = test::TestRequest::post()
            .uri("/access-tokens")
            .set_json(&payload)
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let created_token: AccessTokenRead = test::read_body_json(resp).await;

        assert_eq!(created_token.algorithm, payload.algorithm);
        assert_eq!(created_token.expires_at, payload.expires_at);
        assert!(created_token.enabled);

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
            App::new()
                .app_data(app_data.clone())
                .configure(configure_routes),
        )
        .await;

        // First create an access token
        let payload = AccessTokenCreatePayload {
            project_access_id: Uuid::new(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
        };

        let resp = test::TestRequest::post()
            .uri("/access-tokens")
            .set_json(&payload)
            .send_request(&app)
            .await;

        let created_token: AccessTokenRead = test::read_body_json(resp).await;
        let access_token_id = created_token.id.unwrap();

        // Then get the access token
        let resp = test::TestRequest::get()
            .uri(&format!("/access-tokens/{}", access_token_id))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let retrieved_token: AccessTokenRead = test::read_body_json(resp).await;

        assert_eq!(retrieved_token.id, created_token.id);
        assert_eq!(retrieved_token.algorithm, payload.algorithm);
        assert_eq!(retrieved_token.expires_at, payload.expires_at);

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
            App::new()
                .app_data(app_data.clone())
                .configure(configure_routes),
        )
        .await;

        // First create an access token
        let now = Utc::now();
        let payload = AccessTokenCreatePayload {
            project_access_id: Uuid::new(),
            algorithm: Algorithm::RS256,
            expires_at: now + Duration::hours(1),
        };

        let resp = test::TestRequest::post()
            .uri("/access-tokens")
            .set_json(&payload)
            .send_request(&app)
            .await;

        let created_token: AccessTokenRead = test::read_body_json(resp).await;
        let access_token_id = created_token.id.unwrap();

        // Then update the access token
        let new_expires = now + Duration::hours(2);
        let new_project_id = Uuid::new();
        let update_payload = AccessTokenUpdatePayload {
            expires_at: Some(new_expires),
            enabled: Some(false),
            project_access_id: Some(new_project_id),
            key: None,
        };

        let resp = test::TestRequest::patch()
            .uri(&format!("/access-tokens/{}", access_token_id))
            .set_json(&update_payload)
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let updated_token: AccessTokenRead = test::read_body_json(resp).await;

        assert_eq!(updated_token.expires_at, new_expires);
        assert!(!updated_token.enabled);
        assert_eq!(updated_token.project_access_id, new_project_id);

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
            App::new()
                .app_data(app_data.clone())
                .configure(configure_routes),
        )
        .await;

        // First create an access token
        let payload = AccessTokenCreatePayload {
            project_access_id: Uuid::new(),
            algorithm: Algorithm::RS256,
            expires_at: Utc::now() + Duration::hours(1),
        };

        let resp = test::TestRequest::post()
            .uri("/access-tokens")
            .set_json(&payload)
            .send_request(&app)
            .await;

        let created_token: AccessTokenRead = test::read_body_json(resp).await;
        let access_token_id = created_token.id.unwrap();

        // Then delete the access token
        let resp = test::TestRequest::delete()
            .uri(&format!("/access-tokens/{}", access_token_id))
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 204); // No Content

        // Verify the token was deleted
        let resp = test::TestRequest::get()
            .uri(&format!("/access-tokens/{}", access_token_id))
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 404); // Not Found

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
            App::new()
                .app_data(app_data.clone())
                .configure(configure_routes),
        )
        .await;

        // Try to get a non-existent access token
        let non_existent_id = Uuid::new();
        let resp = test::TestRequest::get()
            .uri(&format!("/access-tokens/{}", non_existent_id))
            .send_request(&app)
            .await;

        assert_eq!(resp.status(), 404);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
}
