# Arg-Backed Runtime PDA Validation

## Why

AQAMI's spec model already allows PDA seeds and bumps to reference instruction arguments.

But the runtime layer and generated validation entrypoints still stop at descriptor-only PDA cases unless every seed can be resolved from account metadata alone.
That leaves one of AQAMI's explicit spec features disconnected from the actual runtime validation path.

The next coherent step is to expose typed instruction-argument runtime context so AQAMI can validate arg-backed PDA seeds and bumps without inventing hidden serialization conventions.

## Scope

In scope:

- add explicit typed runtime argument descriptors to `aqami-runtime`
- add a PDA validation helper that can resolve `arg` seeds and `arg` bumps from that runtime argument context
- keep `account_field` seed handling out of scope and explicitly unsupported
- teach generated Rust instruction modules to pass typed AQAMI args into runtime validation when an instruction references arg-backed PDA semantics
- add validation that arg-backed PDA bumps reference `u8` instruction arguments
- add runtime and codegen tests plus high-level docs

Out of scope:

- full generated on-chain entrypoint wiring
- account-data parsing for `account_field` seeds or `hasOne`
- persistence or deserialization strategy for PDA bump fields inside account data
- non-AQAMI argument encodings or opaque byte-packing conventions

## Constraints

- the runtime API must stay explicit about argument names and types
- generated code should not need to depend on Solana crates directly for PDA arg validation
- do not silently encode args through Borsh, JSON, or stringification just to obtain seed bytes
- preserve backward-compatible helpers for descriptor-only and account-metadata-only PDA validation
- keep generation deterministic and reviewable

## Options Considered

### 1. Delay arg-backed runtime PDA validation until full generated instruction entrypoints exist

Pros:

- fewer intermediate APIs

Cons:

- leaves an explicit spec feature effectively unverifiable at runtime
- keeps generated validation entrypoints incomplete longer than necessary

### 2. Add a narrow typed runtime argument surface now and wire only PDA validation through it

Pros:

- validates a real AQAMI spec feature end to end
- keeps the API explicit instead of smuggling bytes through opaque serialization
- prepares generated instruction boundaries for fuller runtime wiring later

Cons:

- expands the runtime surface with a new typed argument model

## Chosen Approach

Option 2.

AQAMI should introduce a narrow, explicit runtime argument model now and use it only for arg-backed PDA validation.
That keeps the spec, runtime, and generated-code layers aligned without pretending AQAMI already has full instruction decoding or account-data inspection.

## Impacted Areas

- `crates/aqami-spec/src/validate.rs`
- `crates/aqami-runtime/src/descriptors.rs`
- `crates/aqami-runtime/src/lib.rs`
- `crates/aqami-runtime/src/validate.rs`
- `crates/aqami-codegen/src/rust_program.rs`
- `README.md`
- `docs/roadmap.md`
- `docs/specification.md`

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- runtime support for `account_field` seeds once AQAMI has explicit account-data layout helpers
- generated instruction wrappers that decode Solana instruction bytes into AQAMI args before validation
- generated end-to-end `solana-program-test` examples that exercise arg-backed PDA validation in program flow
