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
- `Generate responsibly`: generate the repetitive parts, keep the critical parts reviewable
- `Agent-operable`: everything important should be discoverable through docs, schemas, examples, CLI output, or MCP tools

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

This is intentionally the first code milestone because it gives future codegen, runtime, and MCP work a stable project model to build on.

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
- `crates/aqami-cli`: CLI entrypoint for validation and inspection
- [`docs/architecture.md`](./docs/architecture.md): planned system boundaries and workspace layout
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

The framework should be something a serious team can trust, not a demo that happens to compile.

## Status

As of June 12, 2026, this repository has its first executable foundation:

- documentation and agent guidance
- starter schema and example spec
- a Rust workspace
- typed spec loading and validation
- CLI `validate` and `inspect` commands

The next most valuable step is to deepen `aqami-spec` and add early code generation, not to jump straight into MCP or runtime complexity.
