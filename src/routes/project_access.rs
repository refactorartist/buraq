use crate::config::AppData;
use crate::models::pagination::Pagination;
use crate::models::project_access::{
    ProjectAccess, ProjectAccessFilter, ProjectAccessSortableFields, ProjectAccessUpdatePayload,
};
use crate::models::sort::{SortBuilder, SortDirection};
use crate::services::project_access_service::ProjectAccessService;
use actix_web::{Error, HttpResponse, web};
use mongodb::bson::uuid::Uuid;

pub async fn create(
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

pub async fn read(
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

pub async fn update(
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

    let project_access = service
        .update(project_access_id, payload.into_inner())
        .await;

    match project_access {
        Ok(project_access) => Ok(HttpResponse::Ok().json(project_access)),
        Err(e) => {
            println!("Error updating project access: {:?}", e);
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

pub async fn list(
    data: web::Data<AppData>,
    query: web::Query<ProjectAccessFilter>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectAccessService::new(database.clone()).unwrap();
    let sort =
        SortBuilder::new().add_sort(ProjectAccessSortableFields::Id, SortDirection::Ascending);
    let project_accesses = service
        .find(
            query.into_inner(),
            Some(sort),
            Some(pagination.into_inner()),
        )
        .await;

    match project_accesses {
        Ok(project_accesses) => Ok(HttpResponse::Ok().json(project_accesses)),
        Err(e) => {
            println!("Error listing project accesses: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub fn configure_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/project-access")
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
            ),
        )
        .await;

        // Test
        let project_access = ProjectAccess {
            id: None,
            name: "Test Access".to_string(),
            environment_id: Uuid::new(),
            service_account_id: Some(Uuid::new()),
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

    #[actix_web::test]
    async fn test_list_project_access_success() {
        // Setup
        let db = setup_test_db("project_access_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-access")
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
            ),
        )
        .await;

        // Create multiple project accesses
        for i in 1..=3 {
            let project_access = ProjectAccess {
                id: None,
                name: format!("Test Access {}", i),
                environment_id: Uuid::new(),
                service_account_id: Some(Uuid::new()),
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
        }

        // Test list
        let resp = test::TestRequest::get()
            .uri("/project-access")
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let project_accesses: Vec<ProjectAccess> = test::read_body_json(resp).await;
        assert_eq!(project_accesses.len(), 3);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_list_project_access_with_pagination() {
        // Setup
        let db = setup_test_db("project_access_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-access")
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
            ),
        )
        .await;

        // Create multiple project accesses
        for i in 1..=5 {
            let project_access = ProjectAccess {
                id: None,
                name: format!("Test Access {}", i),
                environment_id: Uuid::new(),
                service_account_id: Some(Uuid::new()),
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
        }

        // Test pagination
        let resp = test::TestRequest::get()
            .uri("/project-access?page=1&limit=2")
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let project_accesses: Vec<ProjectAccess> = test::read_body_json(resp).await;
        assert_eq!(project_accesses.len(), 2);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_list_project_access_with_filters() {
        // Setup
        let db = setup_test_db("project_access_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-access")
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
            ),
        )
        .await;

        // Create multiple project accesses
        let env_id = Uuid::new();
        for i in 1..=3 {
            let project_access = ProjectAccess {
                id: None,
                name: format!("Test Access {}", i),
                environment_id: if i == 1 { env_id } else { Uuid::new() },
                service_account_id: Some(Uuid::new()),
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
        }

        // Test filters
        let resp = test::TestRequest::get()
            .uri(&format!("/project-access?environment_id={}", env_id))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let project_accesses: Vec<ProjectAccess> = test::read_body_json(resp).await;
        assert_eq!(project_accesses.len(), 1);
        assert_eq!(project_accesses[0].environment_id, env_id);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
}
