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
