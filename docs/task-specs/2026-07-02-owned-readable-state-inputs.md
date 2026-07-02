# Owned Readable State Inputs And Resolver Dispatch

## Why

AQAMI-generated instruction modules already expose:

- explicit runtime validation
- explicit execution-preparation helpers
- typed dispatch contracts
- explicit per-program instruction-data codecs

But one structural seam is still weaker than the docs and architecture now imply:

- generated `...AccountData<'a>` types borrow full state accounts instead of carrying owned readable inputs
- generated prepared-execution values borrow that account data too
- byte-driven dispatch still requires callers to build account-data context before decoding/binding can be composed naturally

That keeps the generated runtime surface more lifetime-heavy than necessary and makes future entrypoint wiring awkward even when AQAMI is intentionally avoiding hidden raw-byte account decoding.

The next coherent step is to make readable state inputs owned and then add a resolver-driven byte dispatch helper on top of the existing explicit codec.

## Scope

In scope:

- generate owned instruction account-data structs for readable state inputs
- generate owned prepared-execution values that carry those readable state inputs by value
- simplify generated program-level instruction, account-data, and prepared-instruction enums by removing unnecessary lifetimes
- generate a resolver-based byte-dispatch helper that decodes instruction bytes, asks the caller for explicit account-data context, then binds and prepares execution
- update escrow golden fixtures, generated-program runtime tests, and high-level docs

Out of scope:

- hidden raw-byte account deserialization in `aqami-runtime`
- full generated Solana entrypoints
- automatic state write-back or mutation helpers
- client SDK generation

## Constraints

- preserve AQAMI's explicitness around account-state inputs
- do not invent hidden account decoding conventions
- keep runtime-facing generated code reviewable and deterministic
- avoid weakening the generated execution surface just to reduce signatures
- keep docs aligned with the implemented source of truth

## Options Considered

### 1. Keep borrowed account-data structs and only add more wrapper helpers

Pros:

- smaller immediate patch

Cons:

- preserves an awkward lifetime-heavy execution seam
- keeps future entrypoint wiring needlessly cumbersome
- leaves docs overstating what the generated execution boundary really carries

### 2. Move readable state inputs to owned values and build resolver dispatch on top

Pros:

- matches the intended “readable state inputs” model more honestly
- simplifies generated program-level dispatch surfaces
- gives AQAMI an explicit path toward future entrypoint wiring without hidden decoding
- makes generated tests able to cover multiple instruction variants through one byte-driven flow

Cons:

- expands the codegen diff
- clones readable state inputs into owned values

## Chosen Approach

Option 2.

AQAMI should carry owned readable state inputs through generated execution boundaries now.
That makes the generated surface more explicit, easier to compose, and closer to the future on-chain model AQAMI is working toward, while still refusing to guess at raw account-byte decoding rules.

## Impacted Areas

- `crates/aqami-codegen/src/rust_program.rs`
- `crates/aqami-codegen/testdata/rust_program/escrow/*`
- `README.md`
- `docs/architecture.md`
- `docs/roadmap.md`

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- inspect the generated escrow fixture for owned account-data surfaces and resolver-based byte dispatch

## Follow-Ups

- generated entrypoint wiring on top of the explicit resolver-based execution seam
- stronger end-to-end generated-program execution coverage for richer specs
- future explicit account decoding strategies once AQAMI defines a stable serialization model
