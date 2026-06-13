# Close Semantics

## Why

AQAMI already models initialization and relationship constraints, but close behavior is still under-specified.

`closeTo` should not remain a decorative field.
It affects lamport flow, writable-account requirements, and future runtime enforcement.

## Scope

In scope:

- strengthen semantic validation around `constraints.closeTo`
- strengthen normalization diagnostics around close behavior
- add typed runtime validation for close constraints
- update the example spec to use explicit close semantics

Out of scope:

- actual lamport-transfer execution logic
- rent reclamation implementation
- runtime deserialization or account owner enforcement at close time

## Rules

- instruction accounts with `constraints.closeTo` must be mutable
- `constraints.closeTo` must reference a known instruction account
- the close target account must be mutable
- `constraints.init` and `constraints.closeTo` must not both be set on the same account binding

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- state-level close authorization rules
- explicit close authority conventions
- `solana-program-test` lifecycle tests
