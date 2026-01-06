# fractured-json-rs

[![CI](https://img.shields.io/github/actions/workflow/status/kljensen/fractured-json-rs/ci.yml?branch=main&style=for-the-badge&logo=github-actions&logoColor=white&label=CI)](https://github.com/kljensen/fractured-json-rs/actions/workflows/ci.yml)
[![License: Unlicense](https://img.shields.io/badge/License-Unlicense-yellow.svg?style=for-the-badge)](UNLICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
<!--
[![docs.rs](https://img.shields.io/docsrs/fractured-json-rs?style=for-the-badge)](https://docs.rs/fractured-json-rs)
[![Crates.io](https://img.shields.io/crates/v/fractured-json-rs?style=for-the-badge)](https://crates.io/crates/fractured-json-rs)
-->

A human-friendly JSON/JSONC formatter that makes JSON readable while staying compact.

Inspired by [FracturedJson](https://github.com/j-brooke/FracturedJson/), this Rust implementation formats JSON intelligently: simple structures go on one line, similar items align as tables, and complex structures expand beautifully—all while preserving comments.

## Quick Start

### CLI

```bash
cargo install fractured-json-rs
```

```bash
# Format a file in-place
fractured-json-rs input.jsonc

# Read from stdin, write to stdout
echo '{"name":"value"}' | fractured-json-rs

# Check if file is formatted (exit 1 if not)
fractured-json-rs --check input.jsonc
```

### Library

```toml
[dependencies]
fractured-json-rs = "0.1"
```

```rust
use fractured_json_rs::{format_jsonc, FracturedJsonOptions, CommentPolicy};

let input = r#"{"name":"Alice","age":30}"#;
let options = FracturedJsonOptions::default();
let formatted = format_jsonc(input, &options)?;
```

## What Makes It Different

Most formatters give you "pretty" or "compact". fractured-json-rs gives you **smart** formatting that adapts to your data:

**Simple objects stay inline:**
```json
{"name": "Alice", "age": 30, "city": "Paris"}
```

**Similar structures align as tables:**
```json
{
    "type": "turret",    "hp": 400, "loc": {"x": 47, "y":  -4}, "flags": "S"   },
    "type": "assassin",  "hp":  80, "loc": {"x": 12, "y":   6}, "flags": "Q"   },
    "type": "berserker", "hp": 150, "loc": {"x":  0, "y":   0}                 },
    "type": "pittrap",              "loc": {"x": 10, "y": -14}, "flags": "S,I" }
}
```

**Long arrays wrap efficiently:**
```json
"Locations": [
    [11,  2], [11,  3], [11,  4], [11,  5], [11,  6], [11,  7], [11,  8], [11,  9],
    [11, 10], [11, 11], [11, 12], [11, 13], [11, 14], [ 1, 14], [ 1, 13], [ 1, 12]
]
```

## Comment Preservation

Full support for JSONC (JSON with Comments):

```jsonc
{
    /* Configuration section */
    "database": {
        "host": "localhost",  // default host
        "port": 5432
    },
    // Users list
    "users": [
        {"name": "Alice", "role": "admin"},  // admin user
        {"name": "Bob",   "role": "user"}    // regular user
    ]
}
```

Remove comments with `--comment-policy remove`.

## Features

| Feature | Description |
|---------|-------------|
| **Table formatting** | Aligns similar objects/arrays when structure matches |
| **Compact arrays** | Multiple items per line for long simple arrays |
| **Number alignment** | Left-align or decimal-align numbers in lists |
| **Property alignment** | Align object property names up to a max width |
| **Comment preservation** | Keeps `//` and `/* */` comments |
| **Complexity-based** | Automatically chooses inline vs expanded |
| **Trailing commas** | Optional trailing commas support |
| **Tabs or spaces** | Configurable indentation |

## CLI Options

```
--indent <N|tab>           Indentation width (default: 4)
--max-line-length <N>      Maximum line length (default: 120)
--comment-policy <preserve|remove>  Comment handling (default: preserve)
--number-list-alignment <none|left|decimal>  Number alignment
--table-comma-placement <before|after>  Comma position in tables
--allow-trailing-commas    Add trailing commas
--simple-bracket-padding   Add space inside empty brackets []
--check                    Check formatting without modifying
```

## Library Options

```rust
use fractured_json_rs::{
    format_jsonc, FracturedJsonOptions, CommentPolicy,
    NumberListAlignment, EolStyle, TableCommaPlacement
};

let mut options = FracturedJsonOptions::default();
options.indent_spaces = 2;
options.max_total_line_length = 100;
options.max_inline_complexity = 5;
options.comment_policy = CommentPolicy::Preserve;
options.number_list_alignment = NumberListAlignment::Decimal;
options.table_comma_placement = TableCommaPlacement::BeforePadding;
options.allow_trailing_commas = true;

let formatted = format_jsonc(input, &options)?;
```

## Differences from C# FracturedJson

This is a **spirit-based implementation** focused on readable output:

- Table formatting aligns values only (ignores comments in alignment)
- Comments always force expansion (simplified heuristics)
- Number alignment supports left/decimal only (no "Normalize" mode)
- Comment attachment uses proximity-based heuristics

The goal is beautiful, readable JSON—not byte-for-byte compatibility.

## Related Projects

- [FracturedJson](https://github.com/j-brooke/FracturedJson) - Original C# implementation
- [FracturedJsonJs](https://github.com/j-brooke/FracturedJsonJs) - TypeScript/JavaScript port
- [jsonc-parser](https://github.com/jeff-hykin/jsonc-parser) - JSONC parsing library (used here)

## License

Unlicense - Public domain dedication. See [LICENSE](LICENSE) for details.
