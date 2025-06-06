use crate::serializers::algorithm::serialize as algorithm_serialize;
use jsonwebtoken::Algorithm;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(algorithm: &Option<Algorithm>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match algorithm {
        Some(algorithm) => algorithm_serialize(algorithm, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Algorithm>, D::Error>
where
    D: Deserializer<'de>,
{
    // Use Option::deserialize with a default value of None for missing fields
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "deserialize_opt")] Option<Algorithm>);

    let wrapper = Wrapper::deserialize(deserializer)?;
    Ok(wrapper.0)
}

fn deserialize_opt<'de, D>(deserializer: D) -> Result<Option<Algorithm>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => {
            // Parse the Algorithm from the string
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
    use serde_json::json;
    use serde_json::{from_value, to_value};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(with = "crate::serializers::option_algorithm")]
        algorithm: Option<Algorithm>,
    }

    #[test]
    fn test_serialize_some() {
        let test = TestStruct {
            algorithm: Some(Algorithm::HS256),
        };
        let value = to_value(&test).unwrap();
        assert_eq!(value, json!({ "algorithm": "HS256" }));
    }

    #[test]
    fn test_serialize_none() {
        let test = TestStruct { algorithm: None };
        let value = to_value(&test).unwrap();
        assert_eq!(value, json!({ "algorithm": null }));
    }

    #[test]
    fn test_deserialize_some() {
        let json = json!({ "algorithm": "HS256" });
        let test: TestStruct = from_value(json).unwrap();
        assert_eq!(
            test,
            TestStruct {
                algorithm: Some(Algorithm::HS256)
            }
        );
    }

    #[test]
    fn test_deserialize_none() {
        let json = json!({ "algorithm": null });
        let test: TestStruct = from_value(json).unwrap();
        assert_eq!(test, TestStruct { algorithm: None });
    }

    #[test]
    fn test_deserialize_missing() {
        let json = json!({});
        let result: Result<TestStruct, _> = from_value(json);
        assert!(result.is_err(), "Expected error for missing field");
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!({ "algorithm": "INVALID_ALGO" });
        let result: Result<TestStruct, _> = from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let test = TestStruct {
            algorithm: Some(Algorithm::RS256),
        };
        let serialized = to_value(&test).unwrap();
        let deserialized: TestStruct = from_value(serialized).unwrap();
        assert_eq!(test, deserialized);
    }
}
