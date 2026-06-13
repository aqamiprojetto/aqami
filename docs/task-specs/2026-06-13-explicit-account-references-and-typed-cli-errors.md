# Explicit Account References And Typed CLI Errors

## Why

AQAMI's generator is still relying too much on naming conventions for instruction accounts, and the CLI still uses `anyhow` at the boundary.

The next step is to make instruction-account typing explicit in the spec and keep the CLI on typed error paths so the public framework surface stays more intentional.

## Scope

In scope:

- add explicit instruction-account type references to the schema and model
- validate those references semantically
- use them during normalization and generation
- replace `anyhow` in `aqami-cli` with typed errors
- document source-format and error-handling direction

Out of scope:

- Protobuf or binary-spec adoption
- runtime crate implementation
- MCP transport redesign

## Constraints

- preserve YAML/JSON as the source authoring format for now
- avoid breaking the existing example-driven workflow
- keep generation deterministic
- prefer typed errors on library and CLI surfaces

## Options Considered

### 1. Keep account typing implicit through naming conventions

Pros:

- less schema work

Cons:

- generator ambiguity
- more agent guesswork
- weaker long-term source of truth

### 2. Add explicit account references now

Pros:

- stronger generator inputs
- easier future runtime generation
- safer agent interpretation

Cons:

- slightly more verbose specs

### 3. Keep `anyhow` in the CLI indefinitely

Pros:

- ergonomic top-level code

Cons:

- weakens the typed-framework story
- less deliberate boundary design

## Chosen Approach

Option 2 for spec typing and a typed-error CLI boundary instead of long-term `anyhow` use.

## Impacted Areas

- `schemas/aqami.project.schema.json`
- `examples/specs/escrow.aqami.yaml`
- `crates/aqami-spec/*`
- `crates/aqami-codegen/*`
- `crates/aqami-cli/*`
- `docs/specification.md`

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- richer account-role typing
- generated runtime validation helpers
- optional compiled-spec or transport format evaluation once profiling justifies it
