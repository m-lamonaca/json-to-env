use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
};

use clap::Parser;
use serde_json::Value;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut reader: Box<dyn BufRead> = match args.input {
        None => Box::new(std::io::stdin().lock()),
        Some(ref filename) => {
            let file = File::open(filename)
                .inspect_err(|_| eprintln!("Error: Could not open file `{filename}`"))?;

            Box::new(BufReader::new(file))
        }
    };

    let mut buffer = String::new();

    let input = args.input.unwrap_or("STDIN".to_string());
    reader
        .read_to_string(&mut buffer)
        .inspect_err(|_| eprintln!("Error: Could not read `{input}`"))?;

    let json: Value = serde_json::from_str(&buffer)
        .inspect_err(|_| eprintln!("Error: `{input}` does not contain valid JSON"))?;

    let mut vars: Vec<EnvVar> = vec![];
    JsonParser::parse(&mut vars, "", &json, &args.separator);

    let environ = vars
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("\n");

    let mut writer: Box<dyn Write> = match args.output {
        None => Box::new(std::io::stdout().lock()),
        Some(ref filename) => {
            let file = File::create(filename)
                .inspect_err(|_| eprintln!("Error: Could not open file `{filename}`"))?;

            Box::new(BufWriter::new(file))
        }
    };

    let output = args.output.unwrap_or("STDOUT".to_string());
    writer
        .write_all(environ.as_bytes())
        .inspect_err(|_| eprintln!("Error: Could not write to `{output}`"))?;

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

    /// Separator for nested keys
    #[arg(short, long, value_name = "STRING", default_value = "__")]
    separator: String,
}

struct JsonParser;

impl JsonParser {
    fn parse(lines: &mut Vec<EnvVar>, key: &str, value: &Value, separator: &str) {
        match value {
            Value::Array(array) => {
                for (index, item) in array.iter().enumerate() {
                    let key = Self::build_key(key, index.to_string().as_str(), separator);
                    Self::parse(lines, &key, item, separator)
                }
            }
            Value::Object(object) => {
                for (name, value) in object {
                    let key = Self::build_key(key, name.as_str(), separator);
                    Self::parse(lines, &key, value, separator)
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

struct EnvVar(String, Value);

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
}
