# Architecture

## Purpose

AQAMI should become a framework that is easy to understand and extend by both humans and AI agents.
That requires architecture that is explicit not only in code, but also in the artifacts around the code.

This document defines the intended system boundaries and explains how the current implementation fits into that target shape.

## Architectural Thesis

The best long-term shape for AQAMI is:

- `spec-first`
- `generator-assisted`
- `runtime-explicit`
- `CLI-operable`
- `MCP-exposed`

In other words:

1. a project is described through a machine-readable spec
2. the spec is validated and normalized
3. codegen produces deterministic scaffolding and support code
4. runtime crates provide the explicit primitives used by generated and handwritten code
5. CLI commands expose those capabilities for humans and automation
6. MCP wraps stable CLI and spec capabilities for agent-native workflows

## Workspace Layout

The repository already uses a Cargo workspace with the core foundation crates in place.
The target long-term shape remains:

```text
aqami/
  crates/
    aqami-spec/
    aqami-runtime/
    aqami-codegen/
    aqami-cli/
    aqami-mcp/
  docs/
  examples/
  schemas/
```

Current and intended responsibilities:

- `aqami-spec`
  - typed spec model
  - parsing
  - normalization
  - semantic validation
  - version migration helpers

- `aqami-runtime`
  - account traits and helpers
  - instruction and error primitives
  - shared descriptor types consumed by generated code
  - shared PDA descriptor types consumed by generated code
  - serialization boundaries
  - Solana-specific validation helpers
  - reusable runtime utilities

- `aqami-codegen`
  - Rust program skeleton generation
  - TypeScript/Rust client generation
  - test fixture generation
  - file emission and update strategy

- `aqami-cli`
  - `init`
  - `validate`
  - `inspect`
  - `generate`
  - `format-spec`
  - `doctor`

- `aqami-mcp`
  - project scanning
  - spec inspection
  - template listing
  - safe generation entrypoints
  - validation and diagnostics

## Dependency Direction

The architecture should keep a clean one-way dependency flow:

```text
aqami-spec
  -> aqami-codegen
  -> aqami-cli
  -> aqami-mcp

aqami-runtime
  <- generated code depends on it
```

Performance implication:

- optimize `aqami-runtime` and generated program surfaces as hot-path code
- keep `aqami-spec`, `aqami-cli`, and `aqami-mcp` efficient, but prioritize correctness and determinism first

Important implication:

- `aqami-mcp` should depend on stable spec and CLI capabilities
- `aqami-runtime` should not depend on the MCP or CLI layer
- codegen should consume normalized spec models instead of parsing ad hoc

## Current Implementation Status

Today the architecture is partially implemented:

- `aqami-spec` exists and handles typed loading, schema validation, semantic validation, and normalization
- `aqami-codegen` exists and generates deterministic Rust program skeletons
- `aqami-runtime` exists and provides shared descriptors plus runtime validation helpers
- `aqami-cli` exists and exposes `validate`, `inspect`, and `generate`
- `aqami-mcp` is still intentionally deferred until the spec, runtime, and CLI surfaces stabilize further

That means AQAMI is no longer only an architecture sketch.
It is an implemented foundation with a still-incomplete framework surface.

## Core Flows

### Spec To Code Flow

```text
author spec
  -> schema validation
  -> semantic validation
  -> normalized internal model
  -> generator
  -> Rust program skeletons, clients, tests, manifests
```

### Agent To Project Flow

```text
agent
  -> MCP tool call
  -> aqami-mcp
  -> aqami-cli or aqami-spec services
  -> validated spec/model data
  -> generation or diagnostics
```

### Runtime Execution Flow

```text
transaction input
  -> generated instruction boundary
  -> runtime account/instruction validation helpers
  -> explicit program logic
  -> explicit error/event surfaces
```

## Boundary Rules

### Spec Layer

The spec layer defines what a project is allowed to express.
It should contain:

- programs
- accounts
- instruction arguments
- required accounts
- PDA definitions
- events
- errors
- generation hints that are stable enough to version

It should not become a dumping ground for editor preferences or unstable generator internals.

### Generator Layer

The generator layer translates normalized specs into files.
It should be:

- deterministic
- testable via golden fixtures
- explicit about which regions are generated versus handwritten
- conservative about overwriting user code
- efficient enough to support repeated local and agent-driven use without redundant parsing or unstable output

### Runtime Layer

The runtime layer provides explicit primitives and helpers.
It should avoid surprising behavior and should make Solana constraints easier to express correctly.

## Error Handling Position

AQAMI should treat error surfaces as part of the framework contract.

That means:

- schema, semantic, and normalization failures should produce direct diagnostics tied to the violated invariant
- stable crate and generated-code boundaries should prefer typed error enums
- dynamic or opaque error handling should not become the long-term public framework surface

Internal implementation details may still use contextual error plumbing where appropriate, but AQAMI should not ask future users or agents to reverse-engineer what a failure actually means.

### CLI Layer

The CLI should be the operational entrypoint for humans, CI, and later MCP.
It should expose stable commands before the MCP server wraps them.

### MCP Layer

The MCP layer should make AQAMI agent-friendly, but not become an alternate architecture.
It should expose structured tool calls backed by the same core spec and CLI capabilities used elsewhere.

## Spec-First Before MCP

MCP is valuable, but it should come after the spec model and CLI foundation.

Why:

- a weak spec produces weak tools
- a weak CLI produces thin wrappers around unstable logic
- agents need deterministic surfaces more than they need chat-oriented wrappers

This means the first milestone is not "ship MCP."
It is "define the framework surface cleanly enough that MCP is obvious."

## Compatibility Strategy

AQAMI should learn from frameworks like Anchor without copying their opaque or assumption-heavy patterns blindly.

Potential compatibility goals:

- import or interoperate with selected existing Solana workflows where valuable
- make generated code easy to compare with handwritten code
- keep the AQAMI surface explicit even when compatibility helpers exist

Compatibility should not force the framework into hidden behavior or harder-to-audit code.

## Versioning Strategy

The spec model should be versioned from the beginning.

Recommended approach:

- explicit `specVersion` in every project spec
- semver for framework crates
- migration helpers for spec upgrades once the model stabilizes

## Architectural Risks To Avoid

- making the generator the real source of truth
- inventing MCP tools before the underlying model is stable
- letting templates encode security assumptions silently
- coupling runtime behavior too tightly to scaffolding details
- hiding Solana invariants behind convenience macros without strong visibility

## Milestones So Far

The architecture has already crossed these checkpoints:

- repository docs, schemas, example specs, and task-spec workflow
- `aqami-spec` with parser, schema validation, semantic validation, and normalization
- CLI `validate`, `inspect`, and `generate`
- deterministic Rust program skeleton generation
- first shared `aqami-runtime` descriptor and validation surface
- generated instruction-level runtime validation entrypoints

## Next Architectural Milestones

The next architecture-bearing steps are:

- deepen generated execution boundaries beyond validation-only stubs
- expand runtime helpers without introducing hidden serialization or decoding conventions
- broaden end-to-end generated-program testing
- add client and MCP surfaces only on top of those stabilized foundations
- snapshot tests
- generated example projects

### Phase 3

- runtime primitives
- generated code targets `aqami-runtime`
- integration testing with `solana-program-test`

### Phase 4

- official MCP server
- agent-oriented project introspection and safe code generation

## Success Criteria

AQAMI architecture is succeeding when:

- an experienced Rust engineer can understand the system quickly
- an AI agent can inspect the repo and avoid guessing core invariants
- the spec, generator, runtime, CLI, and MCP layers reinforce each other instead of drifting apart
