# Repository Agent Guide

## Mission

- Preserve AQAMI as a spec-first, AI-first Solana framework.
- Prefer durable architecture over quick local hacks.
- Make agent behavior safer by reducing ambiguity, not by relying on better prompting alone.
- Keep public framework surfaces explicit, documented, versioned, and easy to inspect.

## Primary Priorities

In descending order:

1. correctness and security
2. architectural clarity
3. performance on runtime-facing and generated hot paths
4. stable machine-readable surfaces
5. maintainability and testability
6. developer ergonomics
7. speed of implementation

If a faster path weakens the future framework surface, call it out and recommend the stronger path.

## AQAMI Philosophy

AQAMI is not just a Rust library.
It is a development surface composed of:

- specs
- schemas
- code generation
- runtime primitives
- CLI workflows
- MCP tools

Agents should reason about all of those as first-class framework components.

## Performance Expectations

- Treat future runtime crates and generated program code as performance-sensitive by default.
- Avoid hidden allocations, unnecessary clones, and expensive abstraction layers in runtime-facing code.
- Keep CLI, validation, and codegen efficient and deterministic, but do not turn them into unreadable micro-optimized systems without evidence.
- Prefer structural wins over cleverness: better data flow, clearer ownership, fewer redundant passes, and cached immutable artifacts.
- When optimizing, explain whether the change helps runtime code, generator throughput, or repository ergonomics. Those are different goals.

## Source Of Truth

Use this hierarchy unless a document explicitly says otherwise:

1. files under `schemas/` and future typed spec definitions in code
2. project specs under `examples/specs/` and future real project spec files
3. generation logic and generated outputs
4. runtime crates and tests
5. prose documentation

Do not let generated code become a shadow source of truth.

## Default Workflow

For meaningful work:

1. read [`README.md`](./README.md)
2. read [`docs/architecture.md`](./docs/architecture.md)
3. read [`docs/specification.md`](./docs/specification.md)
4. inspect the relevant schema or example spec
5. use `aqami-cli` to validate or inspect the spec when the CLI already covers the area you are changing
6. create or update a task spec for larger or riskier changes
7. implement the smallest coherent change
8. verify proportionally
9. self-review before concluding

## Spec-First Guardrails

- Do not add framework behavior that cannot be represented cleanly in the spec model.
- Do not introduce hidden conventions that only exist in generator code.
- Prefer explicit names, fields, and constraints over inferred behavior.
- Keep specs versioned.
- When changing spec shape, update schemas, docs, examples, tests, and generator assumptions together.

## Solana-Specific Guardrails

- PDA derivation rules must be deterministic and testable.
- Account ownership, signer requirements, mutability, and seed constraints must remain explicit.
- Generated instruction/account surfaces should be easy to review by humans.
- Avoid "magic" abstractions that obscure lamport movement, signer rules, serialization boundaries, rent implications, or CPI behavior.
- Security-relevant behavior must not live only in templates without tests.

## MCP Positioning

Treat MCP as an official interface for AI agents, not as the first source of truth.

Preferred layering:

```text
spec model
  -> CLI capabilities
  -> generator capabilities
  -> MCP tool surface
```

If the CLI or spec cannot express a capability cleanly, the MCP layer should not invent it ad hoc.

## Documentation Expectations

When behavior changes materially, update the relevant docs in the same change.

Important categories:

- spec shape
- generation behavior
- runtime invariants
- CLI commands
- MCP capabilities
- testing expectations

If a concept is central to future contributors or agents, document it before it becomes tribal knowledge.

## Tests And Verification

Prefer a layered approach:

- schema validation tests
- spec parsing and normalization tests
- codegen golden tests
- runtime unit tests
- `solana-program-test` integration tests
- end-to-end example generation checks

For risky or reusable behavior, do not stop at compile success.

## When To Pause And Confirm

Pause for confirmation before committing to:

- breaking spec version changes
- public crate boundary changes
- dependency choices with long-term ecosystem impact
- architectural moves that reduce explicitness in favor of convenience
- major compatibility decisions involving Anchor or other Solana frameworks

## Task Spec Workflow

For medium tasks, a short written plan is usually enough.

For large, cross-cutting, risky, or architecture-shaping work, create a task spec under `docs/task-specs/` using the template in [`docs/task-specs/TEMPLATE.md`](./docs/task-specs/TEMPLATE.md).

Good candidates include:

- spec model redesign
- generator architecture
- runtime account model
- CLI command system
- MCP surface design
- compatibility layers
- security-critical execution logic

## Editing Guidance

- Keep naming explicit and stable.
- Prefer ASCII unless the file format requires otherwise.
- Avoid speculative abstractions with no near-term use.
- Preserve a clean dependency direction between crates.
- Add concise comments only where invariants or tradeoffs are not obvious.
- Never hand-edit generated outputs if the repository later establishes them as generated artifacts.

## Review Mindset

Before you finish substantial work, ask:

- is this explicit enough for a future agent to understand safely?
- is the source of truth clear?
- does the change create a repeatable precedent we actually want?
- are the docs, examples, schema, and tests still aligned?

If the answer is no, keep polishing.
