# Normalization And Initial Codegen

## Why

AQAMI now has spec loading and validation, but it still lacks the next crucial layer: a normalized project model and a deterministic generator that can turn specs into a meaningful Rust skeleton.

This phase also formalizes performance as a real project value with the right scope.

## Scope

In scope:

- document AQAMI performance priorities
- cache immutable schema validation artifacts
- add normalization and richer generator-facing diagnostics
- add `aqami-codegen`
- add CLI generation support for a minimal Rust skeleton

Out of scope:

- Solana runtime implementation
- real on-chain execution helpers
- MCP implementation
- client SDK generation

## Constraints

- keep the architecture spec-first
- keep generator output deterministic
- do not overfit the generator to a fake Solana runtime yet
- treat runtime-facing performance as a higher bar than general tooling performance

## Options Considered

### 1. Build runtime crates before generation

Pros:

- starts Solana-facing code earlier

Cons:

- delays a deterministic feedback loop from spec to generated output

### 2. Build code generation from raw spec models without normalization

Pros:

- fewer moving parts in the short term

Cons:

- creates repeated identifier/type logic
- weakens long-term consistency across CLI, codegen, and future MCP

### 3. Add normalization first, then initial codegen

Pros:

- cleaner architecture
- stronger generator invariants
- easier future reuse

Cons:

- slightly more up-front design work

## Chosen Approach

Option 3.

AQAMI should normalize validated specs into a deterministic generator-facing model before code generation.

## Impacted Areas

- `README.md`
- `AGENTS.md`
- `docs/architecture.md`
- `docs/performance.md`
- `docs/specification.md`
- `docs/testing-strategy.md`
- `Cargo.toml`
- `crates/aqami-spec/*`
- `crates/aqami-codegen/*`
- `crates/aqami-cli/*`

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- richer normalization and naming rules
- file ownership/overwrite strategy for generated code
- golden fixtures for generated output
- runtime-aware code generation
