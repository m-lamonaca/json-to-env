# json2env

Convert valid JSON to environment variables or an `.env`-line file.

## Usage

```sh
JSON to Env Var converter

Usage: json2env.exe [OPTIONS]

Options:
  -i, --input <FILE>              Input file, defaults to STDIN if not specified
  -o, --output <FILE>             Output file, defaults to STDOUT if not specified
  -s, --key-separator <STRING>    Separator for nested keys [default: __]
  -S, --array-separator <STRING>  Separator for array elements [default: ,]
  -e, --enumerate-array           Separate array elements in multiple environment variables
  -h, --help                      Print help
  -V, --version                   Print version
```
