# Has One Constraints And Explicit Error Surface

## Why

AQAMI now has ownership, initialization, and allocation semantics, but it still lacks explicit account relationship constraints.

One of the highest-value next constraints is `hasOne` because it makes account identity relationships visible in the spec instead of leaving them buried in handwritten logic or generator folklore.

At the same time, AQAMI should be explicit about error handling as a framework value:

- public and generated surfaces should prefer typed errors
- diagnostics should name the violated invariant directly
- framework layers should avoid opaque error boundaries where stable semantics matter

## Scope

In scope:

- add explicit `hasOne` relationship constraints to instruction account bindings
- validate `hasOne` references against declared account types and instruction accounts
- surface `hasOne` through normalization, runtime descriptors, and generated code
- document the framework's explicit error-handling position

Out of scope:

- full runtime data-level `hasOne` enforcement against deserialized account state
- complete Solana execution logic
- compatibility helpers with external frameworks

## Constraint Shape

Chosen shape:

```yaml
constraints:
  hasOne:
    - field: "authority"
      account: "authority"
```

This is intentionally more explicit than a shorthand string list because AQAMI should avoid hidden coupling between field names and instruction account names.

## Validation Rules

- `constraints.hasOne` is only valid on instruction accounts with role `account`
- `constraints.hasOne` requires `accountType`
- each `field` must exist on the declared account type
- each `field` used by `hasOne` must currently be a `pubkey`
- each referenced `account` must exist in the instruction account list

## Error Handling Position

For this phase, "robust error handling" means:

- schema and semantic violations produce precise diagnostics
- normalization refuses ambiguous or under-specified constraints
- runtime helpers use typed error enums
- generated code exposes stable, inspectable validation entrypoints

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir <temp-dir>`
- `cargo check --manifest-path <generated-project>/Cargo.toml`

## Follow-Ups

- runtime state-level `hasOne` enforcement once account deserialization helpers exist
- richer relationship constraints such as `has_many` or authority role conventions
- explicit error-code mapping strategy for generated runtime validation failures
