# Generated Execution Boundaries

## Why

AQAMI-generated instruction modules now expose real runtime validation entrypoints.

That is useful, but the generated execution surface still has a structural mismatch:

- generated `...Accounts` structs currently mix runtime account bindings with typed state values
- generated `execute(...)` functions still sit behind `todo!()` stubs with no explicit preparation layer
- generated code does not yet expose a clean boundary that turns `AccountInfo` plus typed AQAMI inputs into reviewable execution inputs

The next coherent step is to generate explicit execution-preparation helpers.
That gives AQAMI a real runtime-aware instruction boundary without pretending that AQAMI already owns a generic instruction-data or raw-account serialization contract.

## Scope

In scope:

- change generated instruction account views so they represent resolved instruction account keys rather than pretending state is already deserialized
- generate explicit readable state-input structs for non-`init` instruction accounts with declared `accountType`
- generate explicit execution-preparation helpers that:
  - validate runtime accounts
  - collect named account keys from `AccountInfo`
  - clone typed instruction args
  - carry explicit typed state inputs when supplied
- generate an execution wrapper that composes preparation with the existing `execute(...)` hook
- add stronger generated-program end-to-end tests using `solana-program-test`
- update docs and golden fixtures

Out of scope:

- generic instruction-data decoding or dispatch
- raw account-byte decoding in `aqami-runtime`
- automatic mutation or serialization of account state back into Solana accounts
- client SDK generation
- MCP server work

## Constraints

- do not introduce hidden serialization or account-layout conventions
- keep generated boundaries explicit and reviewable
- preserve deterministic code generation
- keep runtime-facing generated code lightweight and allocation-conscious
- do not silently imply that business logic is implemented when it is still scaffolded

## Options Considered

### 1. Keep current generated `execute(...)` stubs and only add more runtime validation helpers

Pros:

- smaller change

Cons:

- leaves the generated execution boundary structurally muddled
- keeps generated account structs misleading for initialized versus existing state accounts
- does not give generated-program tests a stronger runtime-aware target

### 2. Generate explicit execution preparation and key/state views now

Pros:

- makes the runtime boundary honest and reviewable
- separates account keys from typed state snapshots cleanly
- gives generated tests a meaningful boundary before full dispatch or serialization exists

Cons:

- expands the generated API surface
- requires golden fixture and test updates

## Chosen Approach

Option 2.

AQAMI should generate explicit execution-preparation helpers now.
That is the strongest next step toward real instruction flow while still respecting the repo's rule against hidden serialization assumptions.

## Impacted Areas

- `crates/aqami-codegen/src/rust_program.rs`
- `crates/aqami-codegen/testdata/rust_program/escrow/*`
- `README.md`
- `docs/roadmap.md`
- `docs/architecture.md`
- `docs/testing-strategy.md`

## Verification Plan

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`
- generated-program end-to-end `solana-program-test` coverage for the new preparation boundary

## Follow-Ups

- explicit instruction-data decoding and dispatch once AQAMI defines a stable instruction serialization contract
- richer typed state-input generation beyond the first readable-account slice
- generated client and MCP surfaces on top of the stabilized execution boundary
