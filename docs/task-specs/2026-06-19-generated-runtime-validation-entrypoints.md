# Generated Runtime Validation Entrypoints

## Why

AQAMI now has real runtime validation helpers for account metadata, owner semantics, and canonical PDAs.

But generated instruction modules still only expose static descriptor validation.
That leaves a gap between AQAMI's runtime layer and the code it generates.

The next highest-value step is to have generated instruction modules expose real runtime-validation entrypoints that call `aqami-runtime` directly.

## Scope

In scope:

- expose the minimal Solana runtime types that generated code needs through `aqami-runtime`
- update Rust code generation so each instruction module emits a runtime validation entrypoint
- route instructions without PDAs through `validate_program_account_infos`
- route instructions with PDAs through `validate_program_account_infos_with_pdas`
- emit per-instruction PDA descriptor slices from the program-level PDA descriptors
- update codegen tests and high-level docs

Out of scope:

- generated full on-chain entrypoints
- generated argument-to-seed runtime wiring for `arg` seeds or bumps
- account-data parsing for `hasOne`
- CPI helpers

## Constraints

- generated code should not need to depend on Solana crates directly just to call AQAMI runtime validation
- keep the generated surface explicit and reviewable
- do not pretend that generated `execute(...)` already has full Solana runtime wiring
- preserve deterministic generation

## Options Considered

### 1. Keep runtime validation only inside `aqami-runtime` tests for now

Pros:

- no generator changes

Cons:

- generated code stays disconnected from AQAMI's real execution model
- delays the runtime-aware generation phase unnecessarily

### 2. Generate explicit runtime validation entrypoints now

Pros:

- gives generated code a real runtime integration surface
- keeps AQAMI's source of truth aligned from spec to runtime
- prepares the way for fuller generated instruction boundaries later

Cons:

- slightly expands the generated API surface

## Chosen Approach

Option 2.

AQAMI should generate explicit runtime validation entrypoints now, using the runtime helpers it already has.

## Impacted Areas

- `crates/aqami-runtime/src/lib.rs`
- `crates/aqami-codegen/src/rust_program.rs`
- `README.md`
- `docs/roadmap.md`

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- generated instruction wrappers that decode args and call runtime validators directly
- generated support for arg-backed PDA seeds and bumps once AQAMI exposes that runtime context
- generated end-to-end `solana-program-test` example programs
