use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Algorithm {
    RSA,
    HMAC,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::RSA => write!(f, "RSA"),
            Algorithm::HMAC => write!(f, "HMAC"),
        }
    }
}
