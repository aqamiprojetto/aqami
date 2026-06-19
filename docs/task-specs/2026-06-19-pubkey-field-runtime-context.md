# Pubkey Field Runtime Context

## Why

AQAMI's spec can already express two important relationships that depend on account state rather than just account metadata:

- PDA seeds from `account_field`
- identity checks through `constraints.hasOne`

But AQAMI runtime validation still cannot enforce either surface safely because it has no explicit account-data context.

The next coherent step is to add a narrow, typed runtime field context for pubkey-backed account fields.
That lets AQAMI validate the highest-value state-dependent invariants without inventing hidden serialization or byte-layout conventions prematurely.

## Scope

In scope:

- add explicit runtime context types for typed instruction-account pubkey fields
- add runtime validation for `account_field` PDA seeds when the referenced field value is supplied explicitly
- add runtime validation for `constraints.hasOne` against supplied pubkey field values
- teach generated Rust instruction modules to request typed account-state inputs only when these validations are needed
- add semantic validation that `account_field` seeds currently require:
  - a source instruction account with explicit `accountType`
  - a known field on that account type
  - AQAMI type `pubkey`
- update tests, fixtures, and high-level docs

Out of scope:

- raw byte deserialization of account data
- generic account layout offsets or packed binary decoding rules
- non-pubkey `account_field` seed support
- runtime enforcement of arbitrary field comparisons beyond the current `hasOne` and PDA seed surfaces

## Constraints

- do not introduce hidden serialization conventions just to inspect account state
- keep the runtime surface explicit and typed
- generated code should expose reviewable helper structs rather than opaque callbacks or erased maps
- preserve the existing metadata-only and arg-only runtime helpers as compatible wrappers
- keep the first field-context slice narrow: pubkey fields only

## Options Considered

### 1. Decode account bytes directly in `aqami-runtime`

Pros:

- closer to eventual on-chain execution flow

Cons:

- AQAMI has not defined a stable serialization boundary yet
- would force hidden layout assumptions into runtime validation
- would make the source of truth less explicit

### 2. Add typed pubkey field context and validate only against supplied values

Pros:

- keeps the contract explicit and machine-reviewable
- supports both `account_field` PDA seeds and `hasOne` with one narrow surface
- avoids committing AQAMI to premature raw-byte layout rules

Cons:

- callers must provide account-state context explicitly

## Chosen Approach

Option 2.

AQAMI should accept explicit typed pubkey field context now and use that for the first state-dependent validation slice.
That keeps the framework honest: AQAMI validates exactly the state values it has been given, and it does not pretend to own a serialization contract that has not been designed yet.

## Impacted Areas

- `crates/aqami-spec/src/validate.rs`
- `crates/aqami-runtime/src/descriptors.rs`
- `crates/aqami-runtime/src/lib.rs`
- `crates/aqami-runtime/src/validate.rs`
- `crates/aqami-codegen/src/rust_program.rs`
- `crates/aqami-codegen/testdata/rust_program/escrow/*`
- `README.md`
- `docs/roadmap.md`
- `docs/specification.md`

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`
- generate and compile a temporary spec that uses `account_field` PDA seeds

## Follow-Ups

- explicit raw-byte account layout helpers once AQAMI defines a stable serialization model
- non-pubkey `account_field` seed support where the seed encoding is explicit in the spec/runtime contract
- fuller generated instruction-boundary helpers that derive validation context closer to execution flow
