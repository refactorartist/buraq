use crate::config::AppData;
use crate::models::project::{Project, ProjectUpdatePayload};
use crate::services::project_service::ProjectService;
use mongodb::bson::uuid::Uuid;

use actix_web::{Error, HttpResponse, web};

pub async fn create_project(
    data: web::Data<AppData>,
    project: web::Json<Project>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectService::new(database.clone()).unwrap();
    let project = service.create(project.into_inner()).await;

    match project {
        Ok(project) => Ok(HttpResponse::Ok().json(project)),
        Err(e) => {
            println!("Error creating project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn get_project(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectService::new(database.clone()).unwrap();
    let project_id = Uuid::parse_str(path.into_inner()).unwrap();
    let project = service.get_project(project_id).await;

    match project {
        Ok(Some(project)) => Ok(HttpResponse::Ok().json(project)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn update_project(
    data: web::Data<AppData>,
    path: web::Path<String>,
    payload: web::Json<ProjectUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectService::new(database.clone()).unwrap();
    let project_id = Uuid::parse_str(path.into_inner()).unwrap();

    let project = service.update(project_id, payload.into_inner()).await;

    match project {
        Ok(project) => Ok(HttpResponse::Ok().json(project)),
        Err(e) => {
            println!("Error updating project: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

pub async fn delete_project(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ProjectService::new(database.clone()).unwrap();
    let project_id = Uuid::parse_str(path.into_inner()).unwrap();

    let result = service.delete(project_id).await;

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
            .service(web::resource("").route(web::post().to(create_project)))
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get_project))
                    .route(web::patch().to(update_project))
                    .route(web::delete().to(delete_project)),
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
    async fn test_create_project_success() {
        // Setup
        let db = setup_test_db("project_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/projects")
                    .service(web::resource("").route(web::post().to(create_project)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_project))
                            .route(web::patch().to(update_project))
                            .route(web::delete().to(delete_project)),
                    ),
            ),
        )
        .await;

        // Test
        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let resp = test::TestRequest::post()
            .uri("/projects")
            .set_json(&project)
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let created_project: Project = test::read_body_json(resp).await;
        assert_eq!(created_project.name, project.name);
        assert_eq!(created_project.description, project.description);
        assert!(created_project.enabled);
        assert!(created_project.id.is_some());

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_get_project_success() {
        // Setup
        let db = setup_test_db("project_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/projects")
                    .service(web::resource("").route(web::post().to(create_project)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_project))
                            .route(web::patch().to(update_project))
                            .route(web::delete().to(delete_project)),
                    ),
            ),
        )
        .await;

        // First create a project
        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let resp = test::TestRequest::post()
            .uri("/projects")
            .set_json(&project)
            .send_request(&app)
            .await;

        let created_project: Project = test::read_body_json(resp).await;
        let project_id = created_project.id.unwrap();

        // Then get the project
        let resp = test::TestRequest::get()
            .uri(&format!("/projects/{}", project_id))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let retrieved_project: Project = test::read_body_json(resp).await;
        assert_eq!(retrieved_project.id, created_project.id);
        assert_eq!(retrieved_project.name, created_project.name);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_get_nonexistent_project() {
        // Setup
        let db = setup_test_db("project_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/projects")
                    .service(web::resource("").route(web::post().to(create_project)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_project))
                            .route(web::patch().to(update_project))
                            .route(web::delete().to(delete_project)),
                    ),
            ),
        )
        .await;

        let nonexistent_id = Uuid::new();
        let resp = test::TestRequest::get()
            .uri(&format!("/projects/{}", nonexistent_id))
            .send_request(&app)
            .await;

        assert!(resp.status().is_client_error());

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_update_project_success() {
        // Setup
        let db = setup_test_db("project_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/projects")
                    .service(web::resource("").route(web::post().to(create_project)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_project))
                            .route(web::patch().to(update_project))
                            .route(web::delete().to(delete_project)),
                    ),
            ),
        )
        .await;

        // First create a project
        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let resp = test::TestRequest::post()
            .uri("/projects")
            .set_json(&project)
            .send_request(&app)
            .await;

        let created_project: Project = test::read_body_json(resp).await;
        let project_id = created_project.id.unwrap();

        // Then update the project
        let update_payload = ProjectUpdatePayload {
            name: Some("Updated Project".to_string()),
            description: Some("Updated Description".to_string()),
            enabled: Some(false),
        };

        let resp = test::TestRequest::patch()
            .uri(&format!("/projects/{}", project_id))
            .set_json(&update_payload)
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());
        let updated_project: Project = test::read_body_json(resp).await;
        assert_eq!(updated_project.name, "Updated Project");
        assert_eq!(updated_project.description, "Updated Description");
        assert!(!updated_project.enabled);

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_update_nonexistent_project() {
        // Setup
        let db = setup_test_db("project_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/projects")
                    .service(web::resource("").route(web::post().to(create_project)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_project))
                            .route(web::patch().to(update_project))
                            .route(web::delete().to(delete_project)),
                    ),
            ),
        )
        .await;

        let nonexistent_id = Uuid::new();
        let update_payload = ProjectUpdatePayload {
            name: Some("Updated Project".to_string()),
            description: Some("Updated Description".to_string()),
            enabled: Some(false),
        };

        let resp = test::TestRequest::patch()
            .uri(&format!("/projects/{}", nonexistent_id))
            .set_json(&update_payload)
            .send_request(&app)
            .await;

        assert!(resp.status().is_client_error());

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_delete_project_success() {
        // Setup
        let db = setup_test_db("project_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/projects")
                    .service(web::resource("").route(web::post().to(create_project)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_project))
                            .route(web::patch().to(update_project))
                            .route(web::delete().to(delete_project)),
                    ),
            ),
        )
        .await;

        // First create a project
        let project = Project {
            id: None,
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            enabled: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let resp = test::TestRequest::post()
            .uri("/projects")
            .set_json(&project)
            .send_request(&app)
            .await;

        let created_project: Project = test::read_body_json(resp).await;
        let project_id = created_project.id.unwrap();

        // Then delete the project
        let resp = test::TestRequest::delete()
            .uri(&format!("/projects/{}", project_id))
            .send_request(&app)
            .await;

        assert!(resp.status().is_success());

        // Verify project is deleted
        let resp = test::TestRequest::get()
            .uri(&format!("/projects/{}", project_id))
            .send_request(&app)
            .await;

        assert!(resp.status().is_client_error());

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_delete_nonexistent_project() {
        // Setup
        let db = setup_test_db("project_routes").await.unwrap();
        let app_data = web::Data::new(AppData {
            database: Some(std::sync::Arc::new(db.clone())),
            ..Default::default()
        });

        let app = test::init_service(
            App::new().app_data(app_data.clone()).service(
                web::scope("/projects")
                    .service(web::resource("").route(web::post().to(create_project)))
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(get_project))
                            .route(web::patch().to(update_project))
                            .route(web::delete().to(delete_project)),
                    ),
            ),
        )
        .await;

        let nonexistent_id = Uuid::new();
        let resp = test::TestRequest::delete()
            .uri(&format!("/projects/{}", nonexistent_id))
            .send_request(&app)
            .await;

        assert!(resp.status().is_client_error());

        // Cleanup
        cleanup_test_db(db).await.unwrap();
    }
}
