use crate::config::AppData;
use crate::models::server_key::{
    ServerKey, ServerKeyFilter, ServerKeySortableFields, ServerKeyUpdatePayload,
};
use crate::models::pagination::Pagination;
use crate::models::sort::{SortBuilder, SortDirection};
use crate::services::server_key_service::ServerKeyService;
use actix_web::{Error, HttpResponse, web};
use mongodb::bson::uuid::Uuid;

/// Handler to create a new server key.
pub async fn create(
    data: web::Data<AppData>,
    server_key: web::Json<ServerKey>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServerKeyService::new(database.clone())
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let server_key = service.create(server_key.into_inner()).await;
    match server_key {
        Ok(server_key) => Ok(HttpResponse::Ok().json(server_key)),
        Err(e) => {
            println!("Error creating server key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to retrieve a server key by its ID.
pub async fn read(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServerKeyService::new(database.clone())
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let server_key_id = Uuid::parse_str(path.into_inner())
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))?;
    let server_key = service.get_server_key(server_key_id).await;
    match server_key {
        Ok(Some(server_key)) => Ok(HttpResponse::Ok().json(server_key)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            println!("Error getting server key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to update an existing server key.
pub async fn update(
    data: web::Data<AppData>,
    path: web::Path<String>,
    payload: web::Json<ServerKeyUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServerKeyService::new(database.clone())
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let server_key_id = Uuid::parse_str(path.into_inner())
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))?;

    let server_key = service.update(server_key_id, payload.into_inner()).await;

    match server_key {
        Ok(server_key) => Ok(HttpResponse::Ok().json(server_key)),
        Err(e) => {
            println!("Error updating server key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to delete a server key by its ID.
pub async fn delete(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServerKeyService::new(database.clone())
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let server_key_id = Uuid::parse_str(path.into_inner())
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))?;

    let result = service.delete(server_key_id).await;

    match result {
        Ok(deleted) => {
            if deleted {
                Ok(HttpResponse::NoContent().finish())
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        }
        Err(e) => {
            println!("Error deleting server key: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Handler to list server keys with filtering and pagination.
pub async fn list(
    data: web::Data<AppData>,
    filter: Option<web::Query<ServerKeyFilter>>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let database = data
        .database
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not initialized"))?;
    let service = ServerKeyService::new(database.clone())
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let filter = filter.map(|f| f.into_inner());
    let pagination = pagination.into_inner();

    let result = service.list(filter, pagination).await;

    match result {
        Ok(server_keys) => Ok(HttpResponse::Ok().json(server_keys)),
        Err(e) => {
            println!("Error listing server keys: {:?}", e);
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

/// Configures the routes for server keys.
pub fn configure_routes(config: &mut web::ServiceConfig) {
    config
        .route("/", web::post().to(create))
        .route("/{id}", web::get().to(read))
        .route("/{id}", web::put().to(update))
        .route("/{id}", web::delete().to(delete))
        .route("/list", web::get().to(list));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use actix_web::{App, HttpResponse};
    use mongodb::bson::uuid::Uuid;
    use std::sync::Arc;

    #[actix_web::test]
    async fn test_list_server_keys_no_filter() {
        // TODO: Implement test
    }

    #[actix_web::test]
    async fn test_list_server_keys_with_filter() {
        // TODO: Implement test
    }

    #[actix_web::test]
    async fn test_list_server_keys_with_filter_and_pagination() {
        // TODO: Implement test
    }

    #[actix_web::test]
    async fn test_create_server_key_success() {
        // TODO: Implement test
    }

    #[actix_web::test]
    async fn test_get_server_key_success() {
        // TODO: Implement test
    }

    #[actix_web::test]
    async fn test_get_nonexistent_server_key() {
        // TODO: Implement test
    }

    #[actix_web::test]
    async fn test_update_server_key_success() {
        // TODO: Implement test
    }

    #[actix_web::test]
    async fn test_delete_server_key_success() {
        // TODO: Implement test
    }

    #[actix_web::test]
    async fn test_delete_nonexistent_server_key() {
        // TODO: Implement test
    }
}
