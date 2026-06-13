# Solana Program Test Foundation

## Why

AQAMI's spec, normalization, and generator layers are now strong enough that the next most valuable step is to prove behavior in a realistic Solana execution harness.

Adding `solana-program-test` only makes sense if AQAMI also gains a runtime helper that validates actual account metas from `AccountInfo`, not just static descriptors.

## Scope

In scope:

- add the minimal Solana crate dependencies needed for a real integration test
- add runtime validation against actual `AccountInfo` metadata
- validate account-count, signer, and mutability expectations against runtime inputs
- add a focused `solana-program-test` integration test around one small mock instruction boundary

Out of scope:

- generated on-chain entrypoints
- CPI helpers
- full account owner or data-layout enforcement
- broad Solana framework compatibility work

## Constraints

- keep the first harness narrow and meaningful
- prefer typed runtime errors over ad hoc assertions
- avoid depending on unstable Solana APIs beyond what `solana-program-test` already requires
- do not pretend AQAMI has a full runtime execution model yet

## Options Considered

### 1. Add `solana-program-test` now without new runtime helpers

Pros:

- fast to wire up

Cons:

- low signal
- would mostly test scaffolding, not AQAMI behavior

### 2. Add a small actual runtime helper first, then test it through `solana-program-test`

Pros:

- creates a real reusable framework primitive
- proves AQAMI behavior under transaction execution
- gives future generated code something meaningful to target

Cons:

- slightly more implementation work now

## Chosen Approach

Option 2.

AQAMI should enter `solana-program-test` with at least one real runtime validation surface.

## Impacted Areas

- `Cargo.toml`
- `crates/aqami-runtime/Cargo.toml`
- `crates/aqami-runtime/src/*`
- `crates/aqami-runtime/tests/*`
- relevant docs

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- owner checks against actual runtime accounts
- PDA derivation helpers exercised in execution tests
- generated example programs tested with `solana-program-test`
