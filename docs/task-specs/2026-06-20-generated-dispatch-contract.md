# Generated Dispatch Contract

## Why

AQAMI-generated instruction modules now expose explicit runtime validation and execution-preparation helpers.

That is a strong boundary, but the program-level generated surface still lacks an explicit dispatch contract.
Today there is no generated way to say:

- here is the instruction variant
- here are its typed args
- here is any explicit typed account-state context
- now route that through the right runtime-aware instruction boundary

The next coherent step is to generate a typed dispatch layer.
That gives AQAMI real instruction flow without pretending that raw instruction bytes or generic account deserialization are already solved.

## Scope

In scope:

- generate a program-level typed instruction enum in `instructions/mod.rs`
- generate a program-level dispatch error enum
- generate a dispatch helper that routes typed instruction variants into per-instruction `execute_with_runtime_validation(...)`
- update generated-program runtime tests to exercise dispatch rather than only per-instruction preparation
- update roadmap and architecture docs to reflect:
  - the new dispatch contract
  - the future milestone for real crate publishing and stable external versioning once AQAMI is mature

Out of scope:

- raw instruction-data decoding
- binary serialization formats for args
- automatic account-state deserialization from bytes
- crates.io publishing today

## Constraints

- keep the dispatch surface explicit and typed
- do not invent hidden byte-level conventions
- preserve deterministic code generation
- do not imply that AQAMI already has a stable external wire format when it does not

## Chosen Approach

Generate a typed instruction enum plus a typed dispatch helper in `instructions/mod.rs`.
Each variant carries references to the existing generated args and any required explicit account-data context.
Dispatch then becomes an honest generated contract:

- typed instruction choice
- typed args
- typed state context when needed
- runtime account validation and execution handoff

## Impacted Areas

- `crates/aqami-codegen/src/rust_program.rs`
- `crates/aqami-codegen/testdata/rust_program/escrow/*`
- `README.md`
- `docs/roadmap.md`
- `docs/architecture.md`

## Verification Plan

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- generate and compile the reference escrow program
- generated-program `solana-program-test` coverage for the typed dispatch layer
