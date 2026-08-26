use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use futures_util::future::{ok, Ready};
use serde_json::Value;
use std::collections::HashMap;

/// Extracts typed query parameters matching Moonraker's `?key:type=value` syntax.
/// Types supported: `int`, `float`, `bool`, `json`, `str` (default).
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct TypedQuery {
    pub params: HashMap<String, Value>,
}

#[allow(dead_code)]
impl TypedQuery {
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.params.get(key).and_then(|v| v.as_str())
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.params.get(key).and_then(|v| v.as_i64())
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.params.get(key).and_then(|v| v.as_f64())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.params.get(key).and_then(|v| v.as_bool())
    }

    pub fn get_json(&self, key: &str) -> Option<&Value> {
        self.params.get(key)
    }
}

impl FromRequest for TypedQuery {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let mut map = HashMap::new();
        let query_str = req.query_string();

        if !query_str.is_empty() {
            for pair in query_str.split('&') {
                if pair.is_empty() {
                    continue;
                }
                let mut parts = pair.splitn(2, '=');
                let raw_key = parts.next().unwrap_or("");
                let raw_val = parts.next().unwrap_or("");
                let val_decoded = urlencoding::decode(raw_val).unwrap_or_default().to_string();

                let key_parts: Vec<&str> = raw_key.splitn(2, ':').collect();
                let key_name = key_parts[0];
                let type_hint = key_parts.get(1).copied().unwrap_or("str");

                let val = match type_hint {
                    "int" => val_decoded
                        .parse::<i64>()
                        .map(Value::from)
                        .unwrap_or(Value::String(val_decoded)),
                    "float" => val_decoded
                        .parse::<f64>()
                        .map(Value::from)
                        .unwrap_or(Value::String(val_decoded)),
                    "bool" => match val_decoded.to_lowercase().as_str() {
                        "true" | "1" => Value::Bool(true),
                        "false" | "0" => Value::Bool(false),
                        _ => Value::String(val_decoded),
                    },
                    "json" => serde_json::from_str(&val_decoded)
                        .unwrap_or(Value::String(val_decoded)),
                    _ => Value::String(val_decoded),
                };

                map.insert(key_name.to_string(), val);
            }
        }

        ok(TypedQuery { params: map })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_query_logic() {
        let mut map = HashMap::new();
        map.insert("limit".to_string(), Value::from(10));
        map.insert("temp".to_string(), Value::from(215.5));
        map.insert("verbose".to_string(), Value::Bool(true));

        let tq = TypedQuery { params: map };
        assert_eq!(tq.get_i64("limit"), Some(10));
        assert_eq!(tq.get_f64("temp"), Some(215.5));
        assert_eq!(tq.get_bool("verbose"), Some(true));
    }
}
