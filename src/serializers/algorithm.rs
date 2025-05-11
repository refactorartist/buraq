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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    use serde_json;

    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        #[serde(serialize_with = "serialize", deserialize_with = "deserialize")]
        algorithm: Algorithm,
    }

    #[test]
    fn test_serialize_rsa() {
        let test_struct = TestStruct {
            algorithm: Algorithm::RSA,
        };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(serialized, r#"{"algorithm":"RSA"}"#);
    }

    #[test]
    fn test_serialize_hmac() {
        let test_struct = TestStruct {
            algorithm: Algorithm::HMAC,
        };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(serialized, r#"{"algorithm":"HMAC"}"#);
    }

    #[test]
    fn test_deserialize_rsa() {
        let json = r#"{"algorithm":"RSA"}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert!(matches!(deserialized.algorithm, Algorithm::RSA));
    }

    #[test]
    fn test_deserialize_hmac() {
        let json = r#"{"algorithm":"HMAC"}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert!(matches!(deserialized.algorithm, Algorithm::HMAC));
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = r#"{"algorithm":"INVALID"}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}