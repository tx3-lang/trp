# TRP (Transaction Resolver Protocol)

This repository hosts the OpenRPC specification for the Transaction Resolver Protocol (TRP) and notes on generating client/server bindings.

## Layout

- `specs/trp.json`: Placeholder OpenRPC schema for TRP.
- `codegen/generated/`: Output directory for generated artifacts (empty until you run the generator).
- `package.json`: Declares the OpenRPC generator dependency and a sample script for Rust.

## Generating code

The repository delegates rendering to the official [OpenRPC generator](https://github.com/open-rpc/generator). After installing the Node.js dependency (via `npm install`), run the generator directly to produce a Rust client crate:

```bash
npx @open-rpc/generator --schema ./specs/trp.json --client rust --out ./codegen/generated/rust
```

You can replace `rust` with another supported client language and adjust the `--out` path to target additional ecosystems using the same generator.
