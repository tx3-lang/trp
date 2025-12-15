# TRP (Transaction Resolver Protocol)

This repository hosts the OpenRPC specification for the Transaction Resolver Protocol (TRP) and code generation for types and stubs in each of the supported languages.

## Layout

- `specs/trp.json`: OpenRPC schema for the Transation Resolver Protocol (TRP).
- `codegen/{lang}`: Output directory for generated artifacts (empty until you run the generator).
- `xtask`: Rust crate to serve as CLI for code generation

## Code Generation

To generate language bindings from the OpenRPC specification, use the `xtask gen` command:

```bash
cargo run --package xtask -- gen --lang ts,python,go,rust
```

### Options

- `--openrpc <path>`: Path to the OpenRPC specification file (default: `specs/trp.json`)
- `--lang <languages>`: Comma-separated list of languages to generate. Supported languages: `ts`, `python`, `go`, `rust` (e.g., `--lang ts,python`)
- `--out <path>`: Output directory for generated files (default: `bindings`)
- `--clean`: Clean the output directory before generating new files

### Examples

Generate bindings for all supported languages:
```bash
cargo run --package xtask -- gen --lang ts,python,go,rust
```

The generated files will be placed in `bindings/{lang}/types.{ext}` (e.g., `bindings/ts/types.ts`, `bindings/python/types.py`).

