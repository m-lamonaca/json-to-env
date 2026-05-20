use std::{
    error::Error,
    io::{Read, Write},
};

use clap::Parser;
use json2env::{JsonParser, ParseOptions};
use serde_json::Value;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut buffer = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut buffer)
        .inspect_err(|_| eprintln!("Error: Could not read input"))?;

    let json: Value = serde_json::from_str(&buffer)
        .inspect_err(|_| eprintln!("Error: input does not contain valid JSON"))?;

    let options = ParseOptions::new(
        args.key_separator,
        args.array_separator,
        args.enumerate_array,
    );

    let mut parser = JsonParser::new(options);
    let keys = parser.parse(&json);

    let environ = keys
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("\n");

    std::io::stdout()
        .lock()
        .write_all(environ.as_bytes())
        .inspect_err(|_| eprintln!("Error: Could not write to stdout"))?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "json2env", version, about)]
struct Args {
    /// Separator for nested keys
    #[arg(short = 's', long, value_name = "STRING", default_value = "__")]
    key_separator: String,

    /// Separator for array elements
    #[arg(short = 'S', long, value_name = "STRING", default_value = ",")]
    array_separator: String,

    /// Separate array elements in multiple environment variables
    #[arg(short, long)]
    enumerate_array: bool,
}
