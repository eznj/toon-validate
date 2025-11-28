use crate::{error::Result, value::Value};
use serde_json;

pub struct JsonParser;

impl JsonParser {
    pub fn parse(input: &str) -> Result<Value> {
        let json_value: serde_json::Value = serde_json::from_str(input)?;
        Ok(Self::convert_json_to_value(json_value))
    }
    
    fn convert_json_to_value(json: serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                Value::Number(n.as_f64().unwrap_or(0.0))
            }
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.into_iter().map(Self::convert_json_to_value).collect())
            }
            serde_json::Value::Object(obj) => {
                Value::Object(
                    obj.into_iter()
                        .map(|(k, v)| (k, Self::convert_json_to_value(v)))
                        .collect()
                )
            }
        }
    }
}