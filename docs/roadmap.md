# Roadmap

## Purpose

This document answers three questions:

1. where AQAMI started
2. where AQAMI is now
3. where AQAMI is going next

It is the high-level project map.
The task specs under `docs/task-specs/` remain the more detailed step-by-step execution history.

## Long-Term Vision

AQAMI aims to become an AI-first, spec-first Solana framework in Rust.

The desired end state is a system where:

- a developer or agent writes an explicit project spec
- AQAMI validates and normalizes that spec
- AQAMI generates safe, reviewable program and client scaffolding
- AQAMI runtime primitives enforce explicit Solana semantics
- AQAMI CLI and MCP expose those capabilities through stable tooling

## Where We Started

### Phase 0: Foundation

AQAMI started from an almost empty repository.

The first goal was not runtime code.
It was to establish the operating model:

- project vision and principles
- architecture direction
- agent guidance
- specification strategy
- testing strategy
- MCP positioning

That work lives mainly in:

- [`README.md`](../README.md)
- [`AGENTS.md`](../AGENTS.md)
- [`docs/architecture.md`](./architecture.md)
- [`docs/specification.md`](./specification.md)
- [`docs/mcp-strategy.md`](./mcp-strategy.md)
- [`docs/testing-strategy.md`](./testing-strategy.md)

## Where We Are Now

AQAMI is no longer just a concept.
It has a real executable foundation, but it is not yet a production-usable Solana framework.

### Current State

Today AQAMI includes:

- a machine-readable project schema
- a typed Rust spec model
- YAML/JSON spec loading
- schema validation
- semantic validation
- normalized generator-facing models
- a CLI for `validate`, `inspect`, and `generate`
- deterministic Rust skeleton generation
- explicit account ownership and instruction lifecycle semantics in the spec
- explicit program-owned account space semantics in the spec
- explicit `hasOne` relationship semantics in the spec
- explicit PDA bump semantics in the spec
- explicit close-target semantics in the spec

The main implementation crates are:

- `aqami-spec`
- `aqami-runtime`
- `aqami-codegen`
- `aqami-cli`

### What AQAMI Can Do Today

Right now, AQAMI can:

- validate an AQAMI project spec
- inspect the parsed project shape
- normalize the spec into generator-ready structures
- generate a minimal Rust program skeleton
- emit structured instruction-account descriptors into generated code
- share those descriptors through the first `aqami-runtime` crate
- validate generated instruction account descriptors against initial runtime rules

### What AQAMI Cannot Do Yet

Right now, AQAMI does not yet provide:

- generated execution wiring beyond descriptor-level validation
- integration with `solana-program-test`
- real CPI helpers
- client SDK generation
- compatibility layers with existing Solana frameworks
- MCP server implementation

## Is It Usable Yet?

### Short Answer

Yes for framework development and experimentation.
No for real framework users building serious Solana apps on top of it yet.

### More Precise Answer

AQAMI is usable today as:

- a design and architecture foundation
- a spec experiment platform
- a generator and tooling prototype
- a serious internal framework project already worth continuing

AQAMI is not yet usable today as:

- a production-ready replacement for an existing Solana framework
- a stable public framework for external users
- a complete end-to-end developer experience

So the right mental model is:

- we have crossed from "idea" into "real project"
- we have not yet crossed from "foundation project" into "usable public framework"

## Build History So Far

The implementation history has followed this sequence:

1. documentation and agent foundation
2. schema and example spec
3. `aqami-spec` plus CLI validation/inspection
4. normalization and deterministic code generation
5. explicit instruction-account typing
6. typed CLI error handling
7. explicit account ownership and instruction lifecycle semantics
8. explicit program-owned account space metadata
9. first shared `aqami-runtime` descriptor and validation crate

This has been deliberate.
Each phase was chosen to strengthen AQAMI's source of truth before adding more runtime complexity.

## Next Phases

### Phase 1: Stronger Solana Spec Semantics

Next, AQAMI should deepen the spec with additional Solana-relevant constraints such as:

- account relationship constraints like `has_one`
- richer PDA and bump semantics
- richer account sizing and layout metadata
- close semantics and lifecycle relationships

### Phase 2: `aqami-runtime`

AQAMI now has the first real runtime crate.
The next job is to deepen it carefully.

It should continue growing into:

- reusable account validation primitives
- instruction boundary helpers
- explicit error and event foundations
- runtime data structures that generated code can target

### Phase 3: Runtime-Aware Generation

This phase has started.
Next, AQAMI code generation should move from:

- minimal skeletons

to:

- generated code that uses runtime descriptors and helper APIs more deeply
- generated execution boundaries that call runtime validation in real instruction flow

### Phase 4: Testing Depth

After runtime-aware generation:

- add golden generation fixtures
- add integration coverage
- add `solana-program-test` examples
- validate generated examples end to end

### Phase 5: Client And MCP Surfaces

Only after spec, runtime, and codegen are stable enough:

- generate client SDKs
- add MCP on top of the stable CLI/spec/runtime capabilities

## Current Recommended Priorities

In practical order:

1. deepen the AQAMI spec
2. deepen `aqami-runtime` validation and helper surfaces
3. upgrade generation from descriptor emission to real runtime-aware execution boundaries
4. add stronger integration tests
5. add SDK generation
6. add MCP

## How To Read Progress

If you want to know progress quickly:

- `README.md` explains the vision and current shape
- `docs/roadmap.md` explains the long-term arc
- `docs/architecture.md` explains the target system structure
- `docs/task-specs/` explains what we have done in implementation order

## Success Checkpoint

AQAMI will become meaningfully "usable as a framework" when all of these are true:

- specs are expressive enough to model realistic Solana programs
- generated code targets a real AQAMI runtime crate
- generated projects compile and pass integration tests
- the framework has at least one convincing end-to-end example beyond a toy skeleton

We are not there yet.
But we are on the right path, and the path is now concrete rather than hypothetical.
