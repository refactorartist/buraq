use actix_web::{Error, HttpResponse, web};
use crate::config::AppData;
use crate::models::project_scope::{ProjectScope, ProjectScopeFilter, ProjectScopeSortableFields, ProjectScopeUpdatePayload};
use crate::models::sort::{SortBuilder, SortDirection};
use crate::services::project_scope_service::ProjectScopeService;
use mongodb::bson::uuid::Uuid;
use crate::models::pagination::Pagination;

pub async fn create(
    data: web::Data<AppData>,
    project_scope: web::Json<ProjectScope>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectScopeService::new(database.clone()).unwrap();
    let project_scope = service.create(project_scope.into_inner()).await;

    match project_scope {
        Ok(project_scope) => Ok(HttpResponse::Ok().json(project_scope)),
        Err(e) => {
            println!("Error creating project scope: {:?}", e);
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
    let service = ProjectScopeService::new(database.clone()).unwrap();
    let scope_id = Uuid::parse_str(path.into_inner()).unwrap();
    let project_scope = service.get_project_scope(scope_id).await;

    match project_scope {
        Ok(Some(project_scope)) => Ok(HttpResponse::Ok().json(project_scope)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting project scope: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn update(
    data: web::Data<AppData>,
    path: web::Path<String>,
    update: web::Json<ProjectScopeUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectScopeService::new(database.clone()).unwrap();
    let scope_id = Uuid::parse_str(path.into_inner()).unwrap();

    let result = service.update(scope_id, update.into_inner()).await;

    match result {
        Ok(updated) => Ok(HttpResponse::Ok().json(updated)),
        Err(e) => {
            println!("Error updating project scope: {:?}", e);
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
    let service = ProjectScopeService::new(database.clone()).unwrap();
    let scope_id = Uuid::parse_str(path.into_inner()).unwrap();

    let result = service.delete(scope_id).await;

    match result {
        Ok(deleted) => {
            if deleted {
                Ok(HttpResponse::NoContent().finish())
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        }
        Err(e) => {
            println!("Error deleting project scope: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn list(
    data: web::Data<AppData>,
    query: web::Query<ProjectScopeFilter>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectScopeService::new(database.clone()).unwrap();
    let sort = SortBuilder::new().add_sort(ProjectScopeSortableFields::Id, SortDirection::Ascending);
    let project_scopes = service.find(query.into_inner(), Some(sort), Some(pagination.into_inner())).await;

    match project_scopes {
        Ok(scopes) => Ok(HttpResponse::Ok().json(scopes)),
        Err(e) => {
            println!("Error listing project scopes: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub fn configure_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/project-scopes")
            .service(web::resource("").route(web::post().to(create)).route(web::get().to(list)))
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
    async fn test_create_project_scope_success() {
        let db = setup_test_db("project_scope_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-scopes")
                    .service(web::resource("").route(web::post().to(create)).route(web::get().to(list)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(read))
                            .route(web::patch().to(update))
                            .route(web::delete().to(delete)),
                    ),
            ),
        )
        .await;

        let project_scope = ProjectScope {
            id: None,
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let resp = test::TestRequest::post()
            .uri("/project-scopes")
            .set_json(&project_scope)
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let created_scope: ProjectScope = test::read_body_json(resp).await;
        assert_eq!(created_scope.name, project_scope.name);
        assert_eq!(created_scope.description, project_scope.description);
        assert!(created_scope.enabled);
        assert!(created_scope.id.is_some());

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_get_project_scope_success() {
        let db = setup_test_db("project_scope_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-scopes")
                    .service(web::resource("").route(web::post().to(create)).route(web::get().to(list)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(read))
                            .route(web::patch().to(update))
                            .route(web::delete().to(delete)),
                    ),
            ),
        )
        .await;

        let project_scope = ProjectScope {
            id: None,
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let resp = test::TestRequest::post()
            .uri("/project-scopes")
            .set_json(&project_scope)
            .send_request(&app)
            .await;

        let created_scope: ProjectScope = test::read_body_json(resp).await;
        let scope_id = created_scope.id.unwrap();

        let resp = test::TestRequest::get()
            .uri(&format!("/project-scopes/{}", scope_id))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let retrieved_scope: ProjectScope = test::read_body_json(resp).await;
        assert_eq!(retrieved_scope.id, created_scope.id);
        assert_eq!(retrieved_scope.name, created_scope.name);

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_update_project_scope_success() {
        let db = setup_test_db("project_scope_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-scopes")
                    .service(web::resource("").route(web::post().to(create)).route(web::get().to(list)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(read))
                            .route(web::patch().to(update))
                            .route(web::delete().to(delete)),
                    ),
            ),
        )
        .await;

        let project_scope = ProjectScope {
            id: None,
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let resp = test::TestRequest::post()
            .uri("/project-scopes")
            .set_json(&project_scope)
            .send_request(&app)
            .await;

        let created_scope: ProjectScope = test::read_body_json(resp).await;
        let scope_id = created_scope.id.unwrap();

        let update_payload = ProjectScopeUpdatePayload {
            name: Some("write:users".to_string()),
            description: Some("Updated Description".to_string()),
            enabled: Some(false),
        };

        let resp = test::TestRequest::patch()
            .uri(&format!("/project-scopes/{}", scope_id))
            .set_json(&update_payload)
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let updated_scope: ProjectScope = test::read_body_json(resp).await;
        assert_eq!(updated_scope.name, "write:users");
        assert_eq!(updated_scope.description, "Updated Description");
        assert!(!updated_scope.enabled);

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_delete_project_scope_success() {
        let db = setup_test_db("project_scope_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-scopes")
                    .service(web::resource("").route(web::post().to(create)).route(web::get().to(list)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(read))
                            .route(web::patch().to(update))
                            .route(web::delete().to(delete)),
                    ),
            ),
        )
        .await;

        let project_scope = ProjectScope {
            id: None,
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let resp = test::TestRequest::post()
            .uri("/project-scopes")
            .set_json(&project_scope)
            .send_request(&app)
            .await;

        let created_scope: ProjectScope = test::read_body_json(resp).await;
        let scope_id = created_scope.id.unwrap();

        let resp = test::TestRequest::delete()
            .uri(&format!("/project-scopes/{}", scope_id))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());

        let resp = test::TestRequest::get()
            .uri(&format!("/project-scopes/{}", scope_id))
            .send_request(&app)
            .await;

        assert!(resp.status().is_client_error());

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_list_project_scopes_pagination() {
        let db = setup_test_db("project_scope_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-scopes")
                    .service(web::resource("").route(web::post().to(create)).route(web::get().to(list)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(read))
                            .route(web::patch().to(update))
                            .route(web::delete().to(delete)),
                    ),
            ),
        )
        .await;

        let project_scope1 = ProjectScope {
            id: None,
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Test Description 1".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let project_scope2 = ProjectScope {
            id: None,
            project_id: Uuid::new(),
            name: "write:users".to_string(),
            description: "Test Description 2".to_string(),
            enabled: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        test::TestRequest::post()
            .uri("/project-scopes")
            .set_json(&project_scope1)
            .send_request(&app)
            .await;

        test::TestRequest::post()
            .uri("/project-scopes")
            .set_json(&project_scope2)
            .send_request(&app)
            .await;

        let resp = test::TestRequest::get()
            .uri("/project-scopes?page=1&limit=1")
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let scopes: Vec<ProjectScope> = test::read_body_json(resp).await;
        assert_eq!(scopes.len(), 1);

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_list_project_scopes_filter_by_name() {
        let db = setup_test_db("project_scope_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-scopes")
                    .service(web::resource("").route(web::post().to(create)).route(web::get().to(list)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(read))
                            .route(web::patch().to(update))
                            .route(web::delete().to(delete)),
                    ),
            ),
        )
        .await;

        let project_scope1 = ProjectScope {
            id: None,
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Test Description 1".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let project_scope2 = ProjectScope {
            id: None,
            project_id: Uuid::new(),
            name: "write:users".to_string(),
            description: "Test Description 2".to_string(),
            enabled: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        test::TestRequest::post()
            .uri("/project-scopes")
            .set_json(&project_scope1)
            .send_request(&app)
            .await;

        test::TestRequest::post()
            .uri("/project-scopes")
            .set_json(&project_scope2)
            .send_request(&app)
            .await;

        let resp = test::TestRequest::get()
            .uri("/project-scopes?name=read:users&page=1&limit=10")
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let scopes: Vec<ProjectScope> = test::read_body_json(resp).await;
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].name, "read:users");

        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_list_project_scopes_filter_by_enabled() {
        let db = setup_test_db("project_scope_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/project-scopes")
                    .service(web::resource("").route(web::post().to(create)).route(web::get().to(list)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(read))
                            .route(web::patch().to(update))
                            .route(web::delete().to(delete)),
                    ),
            ),
        )
        .await;

        let project_scope1 = ProjectScope {
            id: None,
            project_id: Uuid::new(),
            name: "read:users".to_string(),
            description: "Test Description 1".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let project_scope2 = ProjectScope {
            id: None,
            project_id: Uuid::new(),
            name: "write:users".to_string(),
            description: "Test Description 2".to_string(),
            enabled: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        test::TestRequest::post()
            .uri("/project-scopes")
            .set_json(&project_scope1)
            .send_request(&app)
            .await;

        test::TestRequest::post()
            .uri("/project-scopes")
            .set_json(&project_scope2)
            .send_request(&app)
            .await;

        let resp = test::TestRequest::get()
            .uri("/project-scopes?is_enabled=true&page=1&limit=10")
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let scopes: Vec<ProjectScope> = test::read_body_json(resp).await;
        assert_eq!(scopes.len(), 1);
        assert!(scopes[0].enabled);

        cleanup_test_db(db).await.unwrap();
    }
}
