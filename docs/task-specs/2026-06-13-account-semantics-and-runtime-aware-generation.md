# Account Semantics And Runtime-Aware Generation

## Why

AQAMI now has explicit account-type references, but it still needs a clearer Solana execution model around account ownership and instruction-level lifecycle constraints.

Without that, generated code remains too generic for a real Solana framework.

## Scope

In scope:

- add account ownership to declared account types
- add instruction account lifecycle constraints such as `init`, `payer`, `closeTo`, and `rentExempt`
- validate those rules
- carry them through normalization and generation
- emit richer generated metadata that reflects Solana account semantics

Out of scope:

- real runtime execution logic
- CPI helpers
- binary-spec transport

## Constraints

- keep the spec readable for humans and agents
- avoid pretending runtime behavior already exists when it does not
- keep generation deterministic
- prefer explicit semantics over naming conventions

## Options Considered

### 1. Keep the current generic account model

Pros:

- less immediate schema work

Cons:

- weak Solana semantics
- harder future runtime generation

### 2. Add owner and lifecycle semantics now

Pros:

- better source of truth
- stronger generated metadata
- easier path to runtime-aware code generation

Cons:

- slightly more spec verbosity

## Chosen Approach

Option 2.

Ownership belongs on account type declarations.
Lifecycle behavior belongs on instruction-account bindings.

## Impacted Areas

- `schemas/aqami.project.schema.json`
- `examples/specs/escrow.aqami.yaml`
- `crates/aqami-spec/*`
- `crates/aqami-codegen/*`
- relevant docs

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- owner constraints for external program IDs
- account relationship constraints such as `has_one`
- runtime validation helpers generated from descriptors
