# PDA Bump Semantics

## Why

PDA derivation is one of the highest-risk surfaces for both humans and AI agents.

AQAMI already models PDA seeds, but it still leaves bump handling underspecified.
That creates ambiguity in generation, review, and future runtime helpers.

## Scope

In scope:

- add explicit PDA bump metadata to the spec
- validate bump references for PDA-backed instruction accounts
- propagate bump semantics through normalization and generation
- emit shared PDA descriptors through `aqami-runtime`

Out of scope:

- runtime PDA derivation implementation
- bump persistence or serialization strategy inside account state
- full `solana-program-test` execution coverage

## Chosen Shape

For this phase, PDA bumps support:

- `canonical`
- `arg`

Example:

```yaml
pdas:
  - name: "vault_pda"
    seeds:
      - kind: "const"
        value: "vault"
      - kind: "account_key"
        value: "authority"
    bump:
      kind: "canonical"
```

Or:

```yaml
bump:
  kind: "arg"
  value: "vault_bump"
```

`account_field` bump sourcing can be added later once the surrounding runtime and account-layout model is stronger.

## Validation Rules

- PDA bump metadata is optional for now, but explicit bump declarations are preferred
- `kind: canonical` must not carry `value`
- `kind: arg` must carry `value`
- `kind: arg` values must reference a known instruction argument in every instruction that uses the PDA

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- runtime PDA derivation helpers
- richer bump sources such as account-field backed bumps
- instruction-level PDA derivation tests with `solana-program-test`
