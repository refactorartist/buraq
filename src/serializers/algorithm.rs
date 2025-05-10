use crate::types::Algorithm;
use serde::{Deserializer, Serializer, Deserialize};

pub fn serialize<S>(algorithm: &Algorithm, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let algorithm_str = match algorithm {
        Algorithm::RSA => "RSA",
        Algorithm::HMAC => "HMAC",
    };
    serializer.serialize_str(algorithm_str)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Algorithm, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "RSA" => Ok(Algorithm::RSA),
        "HMAC" => Ok(Algorithm::HMAC),
        _ => Err(serde::de::Error::custom("Invalid algorithm type")),
    }
}