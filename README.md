# json2env

Convert valid JSON to environment variables or an `.env`-line file.

## Usage

```sh
JSON to Env Var converter

Usage: json2env.exe [OPTIONS]

Options:
  -i, --input <FILE>        Input file, defaults to STDIN if not specified
  -o, --output <FILE>       Output file, defaults to STDOUT if not specified
  -s, --separator <STRING>  Separator for nested keys
  -h, --help                Print help
  -V, --version             Print version
```

## Installation

You can either install the tool with `cargo`:

```sh
cargo install --path <path/to/repo>
```

or build the executable with (output in `target/release`):

```sh
cargo build --release
```
