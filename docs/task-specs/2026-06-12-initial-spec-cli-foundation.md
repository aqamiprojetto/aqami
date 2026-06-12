# Initial Spec And CLI Foundation

## Why

AQAMI needs a real executable core behind the documentation.
The most leverage-rich first step is a typed specification layer plus a small CLI that can validate and inspect specs before any code generation or runtime logic exists.

## Scope

In scope:

- create a Cargo workspace
- add `aqami-spec`
- add `aqami-cli`
- support loading AQAMI specs from YAML or JSON
- validate specs against the bundled JSON schema
- run initial semantic validation
- provide `validate` and `inspect` CLI commands

Out of scope:

- code generation
- runtime primitives
- MCP server
- project scaffolding templates beyond the existing example spec

## Constraints

- keep the source of truth spec-first
- avoid deprecated YAML parsing choices if a maintained alternative exists
- keep the CLI thin so future MCP can wrap the same logic
- prefer explicit validation over permissive magic

## Options Considered

### 1. Start with code generation immediately

Pros:

- more visually impressive output

Cons:

- would force generation decisions before validation and spec loading are stable

### 2. Start with runtime primitives first

Pros:

- begins core Solana-facing code earlier

Cons:

- weakens the spec-first architecture by leaving the project model implicit

### 3. Start with typed spec loading and a thin CLI

Pros:

- creates a stable backbone for codegen and MCP
- keeps the architecture honest
- gives immediate executable value

Cons:

- less flashy than scaffolding or runtime code

## Chosen Approach

Option 3.

AQAMI should first become capable of loading, validating, and inspecting its own machine-readable project model.

## Impacted Areas

- `Cargo.toml`
- `crates/aqami-spec/*`
- `crates/aqami-cli/*`
- `docs/task-specs/2026-06-12-initial-spec-cli-foundation.md`
- selected documentation updates if command behavior becomes user-facing

## Verification Plan

- `cargo fmt --all`
- `cargo test`
- `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- `cargo run -p aqami-cli -- inspect examples/specs/escrow.aqami.yaml --format json`

## Follow-Ups

- refine the spec domain model
- add normalization rules
- add richer diagnostics
- add generated fixture tests
- decide when to split project specs across multiple files
