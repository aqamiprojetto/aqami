# Architecture

## Purpose

AQAMI should become a framework that is easy to understand and extend by both humans and AI agents.
That requires architecture that is explicit not only in code, but also in the artifacts around the code.

This document defines the intended system boundaries before implementation begins.

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

## Proposed Workspace Layout

The repository will likely evolve into a Cargo workspace similar to:

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

Suggested responsibilities:

- `aqami-spec`
  - typed spec model
  - parsing
  - normalization
  - semantic validation
  - version migration helpers

- `aqami-runtime`
  - account traits and helpers
  - instruction and error primitives
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

Important implication:

- `aqami-mcp` should depend on stable spec and CLI capabilities
- `aqami-runtime` should not depend on the MCP or CLI layer
- codegen should consume normalized spec models instead of parsing ad hoc

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

### Runtime Layer

The runtime layer provides explicit primitives and helpers.
It should avoid surprising behavior and should make Solana constraints easier to express correctly.

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

## Initial Milestones

### Phase 0

- repository docs
- schemas
- example specs
- task-spec workflow

### Phase 1

- `aqami-spec`
- parser and validator
- CLI `validate` and `inspect`

### Phase 2

- minimal Rust program generation
- snapshot tests
- generated example projects

### Phase 3

- runtime primitives
- integration testing with `solana-program-test`

### Phase 4

- official MCP server
- agent-oriented project introspection and safe code generation

## Success Criteria

AQAMI architecture is succeeding when:

- an experienced Rust engineer can understand the system quickly
- an AI agent can inspect the repo and avoid guessing core invariants
- the spec, generator, runtime, CLI, and MCP layers reinforce each other instead of drifting apart
