# Transaction Resolver Protocol (TRP)

The **Transaction Resolver Protocol (TRP)** is the wire protocol that backends speak to resolve and submit [Tx3](https://github.com/tx3-lang/tx3) transactions. It is defined as an [OpenRPC](https://open-rpc.org/) document.

A consumer with a compiled TIR envelope (produced by the Tx3 compiler) and a set of arguments calls `trp.resolve` on a TRP-speaking backend; the backend returns a transaction envelope that the consumer can sign and submit back via `trp.submit`. Additional methods cover mempool inspection and status checks.

## Spec versions

- `v1beta0/trp.json` — current stable OpenRPC spec. Self-contained (no external `$ref`s).

## Methods

- `trp.resolve` — resolve a TIR + args into a signed transaction envelope.
- `trp.submit` — submit a resolved transaction with witnesses.
- `trp.checkStatus` — check status of transactions in the mempool.
- `trp.dumpLogs` — page through finalized transaction logs.
- `trp.peekPending` / `trp.peekInflight` — inspect the mempool.

See `v1beta0/trp.json` for full schemas, parameter types, and error codes.

## Relationship to TII

The [Transaction Invocation Interface (TII)](https://github.com/tx3-lang/tii) describes *what* transactions a protocol exposes; TRP describes the wire protocol used to actually resolve and submit them. A consumer typically reads a TII document, gathers parameter values from the user, and then calls `trp.resolve` with the TIR envelope from TII and the collected `args`.

### Argument encoding

The TIR carried in the resolve request is **untyped** — the full type information lives only in the `.tii`, a client-side artifact. So for a parameter of an aggregate type (a record, `List`, `Map`, or `Tuple`), the **client is authoritative**: it uses its `.tii` type knowledge to serialize the value into the deterministic, self-describing `TaggedArg` wire form (see the `TaggedArg` schema in `v1beta0/trp.json`). Every node is tagged and struct fields are positionally ordered, so the resolver decodes and applies the value **without a schema** — it only checks that the tag's kind matches the param's flat TIR type and that tags are well-formed. Scalar arguments stay bare and type-directed. Authority for type correctness sits in the client + `.tii`; the server applies deterministically.

## Reference implementation

The `tx3-resolver` crate in [tx3-lang/tx3](https://github.com/tx3-lang/tx3/tree/main/crates/tx3-resolver) is a Rust client implementation of TRP and is the canonical consumer of this spec.

## License

Apache 2.0. See [`LICENSE`](./LICENSE).
