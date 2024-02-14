use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
};

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::Value;

fn main() -> Result<()> {
    let args = Args::parse();

    let mut reader: Box<dyn Read> = match args.input {
        Some(ref filename) => Box::new(
            File::open(filename).with_context(|| format!("Could not open file `{filename}`"))?,
        ),
        None => Box::new(std::io::stdin()),
    };

    let mut buffer = String::new();
    let filename = args.input.unwrap_or("STDIN".to_string());
    reader
        .read_to_string(&mut buffer)
        .with_context(|| format!("Could not read `{filename}`"))?;

    let json: Value = serde_json::from_str(&buffer)
        .with_context(|| format!("`{filename}` does not contain valid JSON"))?;

    let mut vars: Vec<EnvVar> = vec![];
    let separator = args.separator.unwrap_or("__".to_string());
    JsonParser::parse(&mut vars, "", json, &separator);

    let environ = vars
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    let mut writer: Box<dyn Write> = match args.output {
        Some(ref filename) => Box::new(
            File::create(filename).with_context(|| format!("Could not open file `{filename}`"))?,
        ),
        None => Box::new(std::io::stdout()),
    };

    writer
        .write_all(environ.as_bytes())
        .with_context(|| "".to_string())?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "json2env", version, about)]
struct Args {
    /// Input file, defaults to STDIN if not specified
    #[arg(short, long, value_name = "FILE")]
    input: Option<String>,

    /// Output file, defaults to STDOUT if not specified
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Separator for nested keys, defaults to double underscore (__)
    #[arg(short, long, value_name = "STRING")]
    separator: Option<String>,
}

struct JsonParser;

impl JsonParser {
    fn parse(lines: &mut Vec<EnvVar>, key: &str, value: Value, separator: &str) {
        match value {
            Value::Array(array) => {
                for (index, item) in array.into_iter().enumerate() {
                    let key = Self::build_key(key, index.to_string(), separator);
                    Self::parse(lines, key.as_str(), item, separator)
                }
            }
            Value::Object(object) => {
                for (name, value) in object {
                    let key = Self::build_key(key, name, separator);
                    Self::parse(lines, key.as_str(), value, separator)
                }
            }
            _ => lines.push(EnvVar::new(key.trim().to_owned(), value)),
        }
    }

    fn build_key(prefix: &str, key: String, separator: &str) -> String {
        match prefix.is_empty() {
            true => key,
            false => format!("{prefix}{separator}{key}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct EnvVar {
    name: String,
    value: Value,
}

impl EnvVar {
    fn new(name: String, value: Value) -> Self {
        Self { name, value }
    }
}

impl Display for EnvVar {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Value::Null => write!(fmt, "{key}=null", key = self.name),
            Value::Bool(bool) => write!(fmt, "{key}={bool}", key = self.name),
            Value::Number(ref number) => write!(fmt, "{key}={number}", key = self.name),
            Value::String(ref string) => write!(
                fmt,
                r#"{key}="{value}""#,
                key = self.name,
                value = string.replace('"', r#"\""#)
            ),
            _ => write!(fmt, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{EnvVar, JsonParser};

    const KEY: &str = r#""key""#;

    #[test]
    fn build_key_should_leave_key_unchanged_when_prefix_is_empty() {
        // ARRANGE
        let separator = "";
        let input = KEY.to_owned();
        let expected = KEY;

        // ACT
        let result = JsonParser::build_key("", input, separator);

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
        let actual = JsonParser::build_key("prefix", input, separator);

        // ASSERT
        assert_eq!(actual, expected);
    }

    #[test]
    fn bool_env_var_should_be_formatted_correctly() {
        // ARRANGE
        let input = EnvVar::new(KEY.to_owned(), json!(true));

        // ACT
        let result = input.to_string();

        // ASSERT
        assert_eq!(result, r#""key"=true"#)
    }

    #[test]
    fn numeric_env_var_should_be_formatted_correctly() {
        // ARRANGE
        let input = EnvVar::new(KEY.to_owned(), json!(1.0));

        // ACT
        let result = input.to_string();

        // ASSERT
        assert_eq!(result, r#""key"=1.0"#)
    }

    #[test]
    fn string_env_var_should_be_formatted_correctly() {
        // ARRANGE
        let input = EnvVar::new(KEY.to_owned(), json!("hello"));

        // ACT
        let result = input.to_string();

        // ASSERT
        assert_eq!(result, r#""key"="hello""#)
    }

    #[test]
    fn array_env_var_should_be_formatted_correctly() {
        // ARRANGE
        let input = EnvVar::new(KEY.to_owned(), json!([1, 2]));

        // ACT
        let result = input.to_string();

        // ASSERT
        assert_eq!(result, "")
    }

    #[test]
    fn object_env_var_should_be_formatted_correctly() {
        // ARRANGE
        let input = EnvVar::new(KEY.to_owned(), json!({ "key": "value" }));

        // ACT
        let result = input.to_string();

        // ASSERT
        assert_eq!(result, "")
    }
}
