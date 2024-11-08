use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
};

use clap::Parser;
use json2env::{JsonParser, ParseOptions};
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
    #[arg(short = 's', long, value_name = "STRING", default_value = "__")]
    key_separator: String,

    /// Separator for array elements
    #[arg(short = 'S', long, value_name = "STRING", default_value = ",")]
    array_separator: String,

    /// Separate array elements in multiple environment variables
    #[arg(short, long)]
    enumerate_array: bool,
}
