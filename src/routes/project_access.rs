use actix_web::{Error, HttpResponse, web};
use crate::models::project_access::{ProjectAccess, ProjectAccessUpdatePayload};
use crate::services::project_access_service::ProjectAccessService;
use crate::config::AppData;
use mongodb::bson::uuid::Uuid;

pub async fn create_project_access(
    data: web::Data<AppData>,
    project_access: web::Json<ProjectAccess>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectAccessService::new(database.clone()).unwrap();
    let project_access = service.create(project_access.into_inner()).await;

    match project_access {
        Ok(project_access) => Ok(HttpResponse::Ok().json(project_access)),
        Err(e) => {
            println!("Error creating project access: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn get_project_access(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectAccessService::new(database.clone()).unwrap();
    let project_access_id = Uuid::parse_str(path.into_inner()).unwrap();
    let project_access = service.get_project_access(project_access_id).await;

    match project_access {
        Ok(Some(project_access)) => Ok(HttpResponse::Ok().json(project_access)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting project access: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn update_project_access(
    data: web::Data<AppData>,
    path: web::Path<String>,
    payload: web::Json<ProjectAccessUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectAccessService::new(database.clone()).unwrap();
    let project_access_id = Uuid::parse_str(path.into_inner()).unwrap();

    let project_access = service.update(project_access_id, payload.into_inner()).await;

    match project_access {
        Ok(project_access) => Ok(HttpResponse::Ok().json(project_access)),
        Err(e) => {
            println!("Error updating project access: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn delete_project_access(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectAccessService::new(database.clone()).unwrap();
    let project_access_id = Uuid::parse_str(path.into_inner()).unwrap();

    let result = service.delete(project_access_id).await;

    match result {
        Ok(deleted) => {
            if deleted {
                Ok(HttpResponse::NoContent().finish())
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        }
        Err(e) => {
            println!("Error deleting project access: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub fn configure_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/project-access")
            .service(web::resource("").route(web::post().to(create_project_access)))
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get_project_access))
                    .route(web::patch().to(update_project_access))
                    .route(web::delete().to(delete_project_access)),
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
    async fn test_create_project_access_success() {
        // Setup
        let db = setup_test_db("project_access_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-access")
                    .service(web::resource("").route(web::post().to(create_project_access)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_project_access))
                            .route(web::patch().to(update_project_access))
                            .route(web::delete().to(delete_project_access)),
                    ),
            ),
        )
        .await;

        // Test
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Uuid::new(),
            project_scopes: vec![Uuid::new()],
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let resp = test::TestRequest::post()
            .uri("/project-access")
            .set_json(&project_access)
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let created_access: ProjectAccess = test::read_body_json(resp).await;
        assert_eq!(created_access.name, project_access.name);
        assert_eq!(created_access.environment_id, project_access.environment_id);
        assert!(created_access.enabled);
        assert!(created_access.id.is_some());

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
}
