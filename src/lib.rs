use std::fmt::Display;

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ParseOptions {
    key_separator: String,
    array_separator: String,
    enumerate_array: bool,
}

impl ParseOptions {
    pub fn new(key_separator: String, array_separator: String, enumerate_array: bool) -> Self {
        Self {
            key_separator,
            array_separator,
            enumerate_array,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsonParser {
    options: ParseOptions,
    vars: Vec<EnvVar>,
}

impl JsonParser {
    pub fn new(options: ParseOptions) -> Self {
        Self {
            options,
            vars: vec![],
        }
    }

    pub fn parse(&mut self, json: &Value) -> &Vec<EnvVar> {
        Self::parse_keys(&mut self.vars, "", json, &self.options);
        &self.vars
    }

    fn parse_keys(lines: &mut Vec<EnvVar>, key: &str, value: &Value, options: &ParseOptions) {
        match value {
            Value::Array(array) => {
                if options.enumerate_array {
                    for (index, item) in array.iter().enumerate() {
                        let key = Self::build_key(key, &index.to_string(), &options.key_separator);
                        Self::parse_keys(lines, &key, item, options)
                    }
                } else {
                    let value = array
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(&options.array_separator);

                    let item = serde_json::Value::String(value);

                    Self::parse_keys(lines, key, &item, options)
                }
            }
            Value::Object(object) => {
                for (name, value) in object {
                    let key = Self::build_key(key, name, &options.key_separator);
                    Self::parse_keys(lines, &key, value, options)
                }
            }
            _ => lines.push(EnvVar(key.trim().to_owned(), value.clone())),
        }
    }

    fn build_key(prefix: &str, key: &str, separator: &str) -> String {
        match prefix.is_empty() {
            true => key.to_string(),
            false => format!("{prefix}{separator}{key}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVar(String, Value);

impl Display for EnvVar {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.1 {
            Value::Null => write!(fmt, "{key}=null", key = self.0),
            Value::Bool(bool) => write!(fmt, "{key}={bool}", key = self.0),
            Value::Number(ref number) => write!(fmt, "{key}={number}", key = self.0),
            Value::String(ref string) => write!(
                fmt,
                r#"{key}="{value}""#,
                key = self.0,
                value = string.replace('"', r#"\""#)
            ),
            _ => write!(fmt, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use crate::{EnvVar, JsonParser, ParseOptions};

    const KEY: &str = r#""key""#;

    #[test]
    fn build_key_should_leave_key_unchanged_when_prefix_is_empty() {
        // ARRANGE
        let separator = "";
        let input = KEY.to_owned();
        let expected = KEY;

        // ACT
        let result = JsonParser::build_key("", &input, separator);

        // ASSERT
        assert_eq!(result, expected);
    }

    #[test]
    fn build_key_should_leave_prepend_prefix_with_separator() {
        // ARRANGE
        let separator = "_";
        let input = KEY.to_owned();
        let expected = format!("prefix{separator}{KEY}");

        // ACT
        let actual = JsonParser::build_key("prefix", &input, separator);

        // ASSERT
        assert_eq!(actual, expected);
    }

    #[test]
    fn bool_env_var_should_be_formatted_correctly() {
        // ARRANGE
        let input = EnvVar(KEY.to_owned(), json!(true));

        // ACT
        let result = input.to_string();

        // ASSERT
        assert_eq!(result, r#""key"=true"#)
    }

    #[test]
    fn numeric_env_var_should_be_formatted_correctly() {
        // ARRANGE
        let input = EnvVar(KEY.to_owned(), json!(1.0));

        // ACT
        let result = input.to_string();

        // ASSERT
        assert_eq!(result, r#""key"=1.0"#)
    }

    #[test]
    fn string_env_var_should_be_formatted_correctly() {
        // ARRANGE
        let input = EnvVar(KEY.to_owned(), json!("hello"));

        // ACT
        let result = input.to_string();

        // ASSERT
        assert_eq!(result, r#""key"="hello""#)
    }

    #[test]
    fn array_env_var_should_be_formatted_correctly() {
        // ARRANGE
        let input = EnvVar(KEY.to_owned(), json!([1, 2]));

        // ACT
        let result = input.to_string();

        // ASSERT
        assert_eq!(result, "")
    }

    #[test]
    fn object_env_var_should_be_formatted_correctly() {
        // ARRANGE
        let input = EnvVar(KEY.to_owned(), json!({ "key": "value" }));

        // ACT
        let result = input.to_string();

        // ASSERT
        assert_eq!(result, "")
    }

    #[test]
    fn parse_array_not_enumerated() {
        // ARRANGE
        let json = json!({ "array": [1, 2, 3] });
        let options = ParseOptions::new("__".to_string(), ",".to_string(), false);
        let mut parser = JsonParser::new(options);

        // ACT
        let environ = parser.parse(&json);

        // ASSERT
        assert_eq!(
            *environ,
            vec![EnvVar(
                "array".to_string(),
                Value::String("1,2,3".to_string())
            )]
        )
    }

    #[test]
    fn parse_array_enumerated() {
        // ARRANGE
        let json = json!({ "array": [1, 2, 3] });
        let options = ParseOptions::new("__".to_string(), ",".to_string(), true);
        let mut parser = JsonParser::new(options);

        // ACT
        let environ = parser.parse(&json);

        // ASSERT
        assert_eq!(
            *environ,
            vec![
                EnvVar("array__0".to_string(), Value::Number(1.into())),
                EnvVar("array__1".to_string(), Value::Number(2.into())),
                EnvVar("array__2".to_string(), Value::Number(3.into()))
            ]
        )
    }
}
