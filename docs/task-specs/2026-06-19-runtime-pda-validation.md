# Runtime PDA Validation

## Why

AQAMI already validates account-count, signer, mutability, owner, and system-program semantics in a real `solana-program-test` harness.

The next most valuable runtime invariant is PDA derivation correctness.
PDA mistakes are one of the easiest ways for both humans and agents to produce invalid Solana code, and AQAMI's spec model already carries enough structured data to validate part of that surface explicitly.

## Scope

In scope:

- add a typed runtime helper for validating PDA-backed instruction accounts
- support the PDA seed forms the current runtime can resolve safely from existing execution context:
  - `const`
  - `account_key`
- support canonical bump validation
- keep unsupported seed and bump forms explicit rather than guessed
- add focused `solana-program-test` coverage for canonical PDA success and failure
- update high-level docs

Out of scope:

- runtime resolution of `arg` seeds
- runtime resolution of `account_field` seeds
- runtime validation of `arg` bump semantics
- account-data parsing for `hasOne`
- generated entrypoints or full execution wiring

## Constraints

- keep runtime-facing validation explicit and typed
- do not invent hidden conventions for unresolved PDA seed sources
- avoid broad API churn while generated code still targets descriptor-level helpers
- keep the helper efficient and deterministic

## Options Considered

### 1. Delay PDA validation until generated instruction boundaries exist

Pros:

- fewer intermediate helpers

Cons:

- leaves one of the highest-value Solana invariants unchecked
- delays pressure-testing the current descriptor model

### 2. Add a narrow runtime helper now for currently resolvable PDA cases

Pros:

- improves correctness immediately
- keeps unsupported cases explicit
- gives future generated code a real validation target

Cons:

- partial coverage rather than full PDA semantics

## Chosen Approach

Option 2.

AQAMI should validate the PDA cases it can derive safely from the current descriptor model and runtime account metadata, and it should fail explicitly for seed or bump forms that need more execution context than AQAMI exposes today.

## Impacted Areas

- `crates/aqami-runtime/src/lib.rs`
- `crates/aqami-runtime/src/validate.rs`
- `crates/aqami-runtime/tests/solana_runtime.rs`
- `README.md`
- `docs/roadmap.md`

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- introduce runtime inputs for `arg`-backed seed and bump validation
- add layout-aware account-data helpers for `account_field` seeds and `hasOne`
- wire generated instruction boundaries directly into runtime PDA validation helpers
