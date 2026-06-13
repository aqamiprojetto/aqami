# Runtime Crate And Account Space

## Why

AQAMI now has explicit account ownership and instruction lifecycle semantics, but generated code still needs a shared runtime target and stronger Solana-relevant allocation semantics.

The next step is to add account `space` metadata and begin the first real `aqami-runtime` crate so generated projects start depending on shared runtime definitions instead of local stand-ins.

## Scope

In scope:

- add account `space` to the spec model
- validate allocation requirements for initialized program-owned accounts
- create `aqami-runtime`
- move generated descriptor types into `aqami-runtime`
- generate projects that depend on `aqami-runtime`

Out of scope:

- full Solana execution logic
- CPI support
- public release packaging

## Constraints

- keep the source of truth spec-first
- keep the runtime crate explicit and lightweight
- avoid fake completeness; shared descriptors and validators are enough for this phase
- generated projects should still compile locally

## Options Considered

### 1. Keep generated projects self-contained for longer

Pros:

- simpler generation

Cons:

- delays real framework shape
- duplicates runtime-adjacent types in generated projects

### 2. Start `aqami-runtime` now and generate against it

Pros:

- creates a real shared framework surface
- reduces drift between generated projects
- sets up future runtime-aware codegen correctly

Cons:

- requires local path-dependency handling during generation

## Chosen Approach

Option 2.

AQAMI should begin `aqami-runtime` now, even if the first version is intentionally small.

## Impacted Areas

- `Cargo.toml`
- `schemas/aqami.project.schema.json`
- `examples/specs/escrow.aqami.yaml`
- `crates/aqami-spec/*`
- `crates/aqami-runtime/*`
- `crates/aqami-codegen/*`
- relevant docs

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- richer runtime validation helpers
- account relationship validation
- generated runtime integration tests
