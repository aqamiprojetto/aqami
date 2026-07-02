# AQAMI

AQAMI is an AI-first framework for building Solana applications in Rust.

The core idea is simple:

- humans and AI agents should work against the same explicit system model
- framework behavior should be predictable, inspectable, and code-generatable
- Solana-specific complexity should be surfaced clearly instead of hidden behind magic

AQAMI is not intended to be "yet another abstraction layer."
It is intended to become a durable development surface for:

- writing Solana programs
- generating boilerplate safely
- validating domain and account models
- generating client SDKs
- powering AI agents through a stable CLI and, later, an official MCP server

## Why AQAMI

Many existing frameworks are optimized primarily for experienced human maintainers.
That works, but it creates friction for AI-assisted development:

- implicit conventions are easy to miss
- PDA derivation is easy to get wrong
- account constraints become scattered
- generated code and handwritten code drift
- project-specific context is not machine-readable

AQAMI aims to make high-quality Solana development easier for both experienced engineers and AI coding systems by making the framework surface explicit from day one.

## Design Principles

- `Spec-first`: machine-readable project and program specs are the source of truth for structure
- `Explicit over implicit`: no hidden runtime behavior that agents must guess
- `Stable conventions`: repository layout, naming, and crate boundaries should be predictable
- `Rust-native`: use idiomatic Rust where it improves clarity and safety
- `Explicit errors`: public and generated surfaces should prefer typed, inspectable errors over opaque failure paths
- `Generate responsibly`: generate the repetitive parts, keep the critical parts reviewable
- `Agent-operable`: everything important should be discoverable through docs, schemas, examples, CLI output, or MCP tools
- `Performance-aware`: optimize runtime-facing and generated hot paths aggressively, while keeping tooling layers explicit and maintainable
- `Zero-debt mindset`: do not normalize technical debt; fix drift, stale assumptions, and temporary seams as soon as they are understood well enough to remove safely

## What "AI-First" Means Here

AQAMI should be understandable to an agent without relying on vague prose alone.
That means the repository must provide:

- machine-readable specs
- JSON Schema or equivalent validation contracts
- examples that demonstrate the intended shapes
- an official agent guide
- architectural boundaries that are easy to follow
- deterministic code generation inputs and outputs

The goal is not to let agents guess better.
The goal is to reduce guessing.

## Planned System

The framework is expected to grow into a Rust workspace with clear component boundaries:

- `aqami-spec`: typed spec model and validation
- `aqami-runtime`: runtime primitives for accounts, instructions, errors, events, and helpers
- `aqami-codegen`: Rust, TypeScript, and test generation from specs
- `aqami-cli`: project scaffolding, validation, generation, inspection, and dev workflows
- `aqami-mcp`: official MCP server for AI agent interaction
- `examples/*`: reference projects and spec fixtures

Once AQAMI is mature enough for real external users, those framework crates should also become stable published Rust crates with intentional semver and crates.io distribution rather than remaining only workspace-local development artifacts.

The intended dependency direction is:

```text
spec files
  -> aqami-spec
  -> aqami-codegen
  -> generated program/client/test assets
  -> aqami-runtime

aqami-cli
  -> aqami-spec
  -> aqami-codegen
  -> aqami-runtime

aqami-mcp
  -> aqami-cli and aqami-spec
```

## Current Workspace

The repository now includes the first executable AQAMI layer:

- `crates/aqami-spec`: loads AQAMI YAML or JSON specs, validates them against the bundled schema, and applies initial semantic validation
- `crates/aqami-cli`: exposes `validate` and `inspect` commands on top of `aqami-spec`
- `crates/aqami-codegen`: generates deterministic Rust skeletons from normalized AQAMI specs
- `crates/aqami-runtime`: shared descriptor and validation primitives targeted by generated code

The current spec layer now also carries:

- explicit declared account ownership on account types
- explicit account `space` metadata for program-owned state
- explicit instruction-account type references
- explicit instruction lifecycle constraints such as `init`, `payer`, and `rentExempt`
- explicit close semantics through `closeTo`
- explicit account relationship constraints such as `hasOne`
- explicit PDA bump semantics

Generated Rust skeletons now depend on `aqami-runtime` instead of re-declaring their own local descriptor surface.
That gives AQAMI its first shared runtime contract between spec normalization, code generation, and generated projects.
AQAMI now also has its first `solana-program-test` integration harness proving runtime account-meta and owner validation under actual transaction execution.
AQAMI can now also validate canonical PDAs derived from explicit `const` and `account_key` seed metadata in the same execution harness.
AQAMI can now also validate arg-backed PDA seeds and bumps through explicit typed runtime argument context instead of opaque byte-packing conventions.
AQAMI can now also validate `account_field` PDA seeds and `hasOne` account relationships through explicit typed pubkey-field context supplied by generated instruction entrypoints.
Generated instruction modules now also expose runtime validation entrypoints that call AQAMI runtime helpers directly, including typed arg wiring for arg-backed PDAs and explicit typed account-data wiring for pubkey field checks.
Generated instruction modules now also expose execution-preparation helpers that validate runtime inputs, collect explicit account-key views, and carry owned typed readable state inputs into program logic without hidden decoding conventions.
Generated `instructions/mod.rs` surfaces now also expose a typed dispatch-preparation contract that routes instruction variants into the correct prepared execution boundary without inventing a raw byte wire format yet.
Generated `instructions/mod.rs` surfaces now also expose an explicit per-program instruction-data codec, typed instruction/context binding, and resolver-driven byte-dispatch preparation while still keeping the wire contract generator-defined and reviewable instead of pretending AQAMI has a stable ecosystem-wide serialization format already.

## Performance Position

AQAMI should care deeply about performance, but with the right layering.

- generated program code and future runtime crates should be treated as hot-path code
- Solana-facing abstractions should avoid hidden allocations, unnecessary copies, and expensive convenience layers
- validation, CLI, and codegen should still avoid obvious waste, but they should optimize for deterministic behavior and maintainability before chasing micro-optimizations

## Current Foundation

This repository is starting with infrastructure before framework code.
That is intentional.

The initial goal is to establish:

- architectural direction
- agent guidance
- specification format
- MCP strategy
- testing philosophy
- examples and schemas that future code must respect
- an executable spec and CLI foundation

## Source Of Truth Hierarchy

When the repository grows, treat sources in this order:

1. machine-readable specs and schemas
2. generated artifacts derived from those specs
3. runtime crate APIs and tests
4. human-facing documentation and examples

If those layers disagree, the mismatch should be resolved immediately instead of tolerated.

## Repository Map

- [`AGENTS.md`](./AGENTS.md): repository contract for AI coding agents
- [`Cargo.toml`](./Cargo.toml): workspace root
- `crates/aqami-spec`: spec loading, schema validation, and semantic validation
- `crates/aqami-cli`: CLI entrypoint for validation, inspection, and generation
- `crates/aqami-codegen`: deterministic Rust skeleton generation from normalized specs
- `crates/aqami-runtime`: shared account and instruction descriptor types plus runtime validation helpers
- [`docs/architecture.md`](./docs/architecture.md): planned system boundaries and workspace layout
- [`docs/documentation-guidelines.md`](./docs/documentation-guidelines.md): how AQAMI should document public surfaces and use comments
- [`docs/roadmap.md`](./docs/roadmap.md): long-term project arc from foundation to usable framework
- [`docs/performance.md`](./docs/performance.md): performance values and optimization priorities
- [`docs/specification.md`](./docs/specification.md): spec-first model, domain model, and authoring rules
- [`docs/mcp-strategy.md`](./docs/mcp-strategy.md): when MCP helps and how it should fit the system
- [`docs/testing-strategy.md`](./docs/testing-strategy.md): quality and verification expectations
- [`docs/task-specs/TEMPLATE.md`](./docs/task-specs/TEMPLATE.md): template for substantial or risky changes
- [`schemas/aqami.project.schema.json`](./schemas/aqami.project.schema.json): initial machine-readable schema
- [`examples/specs/escrow.aqami.yaml`](./examples/specs/escrow.aqami.yaml): example project spec

## Quick Start

- run `cargo test`
- run `cargo run -p aqami-cli -- validate examples/specs/escrow.aqami.yaml`
- run `cargo run -p aqami-cli -- inspect examples/specs/escrow.aqami.yaml`
- run `cargo run -p aqami-cli -- inspect examples/specs/escrow.aqami.yaml --format json`
- run `cargo run -p aqami-cli -- generate rust-program --spec examples/specs/escrow.aqami.yaml --output-dir /tmp/aqami-out`

## CI And Release Builds

GitHub Actions CI now validates the repository through the same explicit surfaces AQAMI expects future users and agents to trust:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`
- `aqami-cli validate` and `inspect` against the reference escrow spec
- spec-to-code round-tripping by generating the reference Rust program and compiling it

Release builds are tuned for maximum host-side optimization at the workspace level:

- `opt-level = 3`
- `lto = "fat"`
- `codegen-units = 1`
- `panic = "abort"`
- `strip = "symbols"`

That profile is intentionally optimized for shipped release artifacts rather than fast local iteration.
CI keeps its main verification path on the default test and dev profiles so day-to-day feedback stays quick.

## Development Priorities

Recommended build order:

1. stabilize the spec model
2. validate specs through schema plus Rust validation
3. scaffold CLI inspection and validation
4. generate minimal Rust program skeletons and test fixtures
5. add runtime primitives
6. add MCP once the spec and CLI surfaces are stable

This order matters because a weak spec layer produces a weak MCP surface.

## Quality Bar

AQAMI should optimize for:

- security-sensitive correctness
- clarity for future maintainers
- explicit Solana invariants
- high-value documentation
- strong tests around generated and security-relevant behavior
- a zero-technical-debt culture where known debt is treated as a defect to retire, not a permanent layer to build on

The framework should be something a serious team can trust, not a demo that happens to compile.

## Status

As of July 2, 2026, this repository has a stronger executable foundation:

- documentation and agent guidance
- starter schema and example spec
- a Rust workspace
- typed spec loading and validation
- normalized spec modeling
- deterministic Rust skeleton generation
- a first shared runtime crate for account and instruction descriptors
- a first `solana-program-test` integration test path
- runtime owner, system-program, canonical PDA, arg-backed PDA, `account_field` PDA, and `hasOne` validation helpers
- generated instruction-level runtime validation entrypoints with typed arg wiring for arg-backed PDAs and typed account-data wiring for pubkey field checks
- generated execution-preparation boundaries that turn `AccountInfo` plus typed AQAMI inputs into explicit execution values with owned readable state inputs
- generated-program `solana-program-test` coverage for the execution-preparation boundary
- generated per-program instruction-data encode/decode helpers, typed instruction/context binding, and resolver-driven byte-dispatch preparation
- CLI `validate`, `inspect`, and `generate` commands

The next most valuable step is to broaden end-to-end generated execution flow around that explicit instruction-data contract, keep raw account-state decoding assumptions out of the runtime surface, and then build client and MCP surfaces on top of that stabilized execution model.
