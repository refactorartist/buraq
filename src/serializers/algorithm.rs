use jsonwebtoken::Algorithm;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(algorithm: &Algorithm, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let algorithm_str = match algorithm {
        Algorithm::HS256 => "HS256",
        Algorithm::HS384 => "HS384",
        Algorithm::HS512 => "HS512",
        Algorithm::ES256 => "ES256",
        Algorithm::ES384 => "ES384",
        Algorithm::RS256 => "RS256",
        Algorithm::RS384 => "RS384",
        Algorithm::RS512 => "RS512",
        Algorithm::PS256 => "PS256",
        Algorithm::PS384 => "PS384",
        Algorithm::PS512 => "PS512",
        Algorithm::EdDSA => "EdDSA",
    };
    serializer.serialize_str(algorithm_str)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Algorithm, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "HS256" => Ok(Algorithm::HS256),
        "HS384" => Ok(Algorithm::HS384),
        "HS512" => Ok(Algorithm::HS512),
        "ES256" => Ok(Algorithm::ES256),
        "ES384" => Ok(Algorithm::ES384),
        "RS256" => Ok(Algorithm::RS256),
        "RS384" => Ok(Algorithm::RS384),
        "RS512" => Ok(Algorithm::RS512),
        "PS256" => Ok(Algorithm::PS256),
        "PS384" => Ok(Algorithm::PS384),
        "PS512" => Ok(Algorithm::PS512),
        "EdDSA" => Ok(Algorithm::EdDSA),
        _ => Err(serde::de::Error::custom("Invalid algorithm type")),
    }
}

pub fn serialize_option<S>(algorithm: &Option<Algorithm>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match algorithm {
        Some(algorithm) => serialize(algorithm, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_option<'de, D>(deserializer: D) -> Result<Option<Algorithm>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => {
            // Parse the Algorithm from the string directly
            let algorithm = s.parse().map_err(serde::de::Error::custom)?;
            Ok(Some(algorithm))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json;

    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        #[serde(serialize_with = "serialize", deserialize_with = "deserialize")]
        algorithm: Algorithm,
    }

    #[derive(Serialize, Deserialize)]
    struct TestStructOption {
        #[serde(
            serialize_with = "serialize_option",
            deserialize_with = "deserialize_option"
        )]
        algorithm: Option<Algorithm>,
    }

    #[test]
    fn test_serialize_rs256() {
        let test_struct = TestStruct {
            algorithm: Algorithm::RS256,
        };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(serialized, r#"{"algorithm":"RS256"}"#);
    }

    #[test]
    fn test_serialize_hs256() {
        let test_struct = TestStruct {
            algorithm: Algorithm::HS256,
        };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(serialized, r#"{"algorithm":"HS256"}"#);
    }

    #[test]
    fn test_serialize_option_rs256() {
        let test_struct = TestStructOption {
            algorithm: Some(Algorithm::RS256),
        };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(serialized, r#"{"algorithm":"RS256"}"#);
    }

    #[test]
    fn test_serialize_option_none() {
        let test_struct = TestStructOption {
            algorithm: None,
        };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(serialized, r#"{"algorithm":null}"#);
    }

    #[test]
    fn test_deserialize_rs256() {
        let json = r#"{"algorithm":"RS256"}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert!(matches!(deserialized.algorithm, Algorithm::RS256));
    }

    #[test]
    fn test_deserialize_hs256() {
        let json = r#"{"algorithm":"HS256"}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert!(matches!(deserialized.algorithm, Algorithm::HS256));
    }

    #[test]
    fn test_deserialize_option_rs256() {
        let json = r#"{"algorithm":"RS256"}"#;
        let deserialized: TestStructOption = serde_json::from_str(json).unwrap();
        assert!(matches!(deserialized.algorithm, Some(Algorithm::RS256)));
    }

    #[test]
    fn test_deserialize_option_none() {
        let json = r#"{"algorithm":null}"#;
        let deserialized: TestStructOption = serde_json::from_str(json).unwrap();
        assert!(deserialized.algorithm.is_none());
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = r#"{"algorithm":"INVALID"}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_option_invalid() {
        let json = r#"{"algorithm":"INVALID"}"#;
        let result: Result<TestStructOption, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
