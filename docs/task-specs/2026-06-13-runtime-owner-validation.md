# Runtime Owner Validation

## Why

AQAMI now validates runtime signer, mutability, and account-count expectations in a real `solana-program-test` harness.

The next highest-value step is to validate ownership and unambiguous role semantics at the same instruction boundary.
That improves security and correctness without pretending AQAMI already has a full execution model.

## Scope

In scope:

- add a program-context runtime helper that can validate actual account owners
- enforce program-owned account expectations against the executing `program_id`
- enforce `system_program` role identity where the expected address is unambiguous
- add explicit typed runtime errors for ownership and role mismatches
- add focused `solana-program-test` coverage for the new rules
- update the high-level docs to reflect the stronger runtime validation surface

Out of scope:

- full PDA derivation enforcement at runtime
- account data-layout parsing for `hasOne`
- token-program role compatibility decisions beyond explicit documentation
- generated instruction entrypoints

## Constraints

- keep the runtime surface explicit and typed
- avoid silent partial validation for ambiguous Solana concepts
- do not collapse classic SPL Token and Token-2022 into one hidden runtime assumption
- preserve fast validation on runtime-facing hot paths

## Options Considered

### 1. Add owner validation directly into the existing metadata helper

Pros:

- fewer public functions

Cons:

- would require `program_id` for every call
- would blur metadata-only checks with execution-context checks

### 2. Add a second helper for program-context checks

Pros:

- keeps the current metadata helper simple
- makes execution-context requirements explicit
- gives generated code a clearer upgrade path later

Cons:

- slightly larger public surface

## Chosen Approach

Option 2.

AQAMI should keep `validate_account_infos` as the metadata-level helper and add a second helper for checks that require the active `program_id`.

For `token_program` ownership semantics, AQAMI should fail explicitly instead of inventing a hidden assumption while the spec still treats token ownership as a single coarse concept.

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

- refine token-program ownership semantics in the spec model
- add PDA/runtime derivation checks
- add runtime account-data helpers for `hasOne` and layout-aware validation
