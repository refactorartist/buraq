use crate::models::access_token::AccessToken;
use crate::repositories::base::Repository;
use crate::repositories::access_token::AccessTokenRepository;
use anyhow::Error;
use mongodb::Database;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};


pub struct AccessTokenService {
    access_token_service:AccessTokenRepository
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenFilter {
    key: Option<String>,
    algorithm: Option<String>,
    enabled: bool,
}

