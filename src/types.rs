use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Algorithm {
    RSA,
    HMAC,
}