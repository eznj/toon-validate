# toon-validate

Command-line TOON Validator for structure validation, token analysis, and file profiling.

The `tval` binary provides fast validation and analysis of TOON and JSON files.

## Requirements

- Rust toolchain (stable)

## Build

```sh
cargo build --release
```

Binary: `target/release/tval`

## Commands

```sh
tval analyze <file>     # analyze single file
tval profile <dir>      # analyze directory  
tval check <file>       # validate structure
```

## Flags

- `--in=toon|json` - input format (default: auto)
- `--json` - JSON output
- `--ext=<list>` - file extensions for profile (default: .toon,.json)

## Exit codes

- 0: success
- 1: IO/parse error  
- 2: validation error

## Tests

```sh
cargo test
./test.sh
```