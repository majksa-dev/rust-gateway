use anyhow::{anyhow, bail, Context, Ok, Result};
use serde_json::Value;

pub trait ClaimParser {
    fn parse(&self, path: &str) -> Result<String>;

    fn parse_value(&self, path: &str) -> Result<&Value>;
}

impl ClaimParser for Value {
    fn parse(&self, path: &str) -> Result<String> {
        match self.parse_value(path)? {
            Value::Null => bail!("Null value at key '{}'", path),
            Value::Bool(bool) => Ok(bool.to_string()),
            Value::Number(number) => Ok(number.to_string()),
            Value::String(string) => Ok(string.clone()),
            Value::Array(_) => bail!("Array value at key '{}'", path),
            Value::Object(_) => bail!("Object value at key '{}'", path),
        }
    }

    fn parse_value(&self, path: &str) -> Result<&Value> {
        match path.split_once('.') {
            Some((key, path)) => match self {
                Value::Object(map) => {
                    let value = map
                        .get(key)
                        .ok_or_else(|| anyhow!("Expected key '{}' not found in {:?}", key, self))?;
                    value
                        .parse_value(path)
                        .with_context(|| format!("at key '{}'", key))
                }
                _ => Err(anyhow!("Expected object but found {:?}", self)),
            },
            None => match self {
                Value::Object(map) => map
                    .get(path)
                    .ok_or_else(|| anyhow!("Expected key '{}' not found in {:?}", path, self)),
                _ => Err(anyhow!("Expected object but found {:?}", self)),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_claim_parser() {
        let claims = json!({
            "sub": "1234567890",
            "name": "John Doe",
            "admin": true,
            "iat": 1516239022,
            "extra": {
                "email": "john@doe.com"
            }
        });
        assert_eq!(claims.parse("sub").unwrap(), "1234567890");
        assert_eq!(claims.parse("name").unwrap(), "John Doe");
        assert_eq!(claims.parse("admin").unwrap(), "true");
        assert_eq!(claims.parse("iat").unwrap(), "1516239022");
        assert!(claims.parse("missing").is_err());
        assert!(claims.parse("sub.missing").is_err());
        assert_eq!(claims.parse("extra.email").unwrap(), "john@doe.com");
        assert!(claims.parse("extra.missing").is_err());
    }
}
