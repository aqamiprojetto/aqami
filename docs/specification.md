# Specification

## Goal

AQAMI should be built around a machine-readable specification layer that describes a Solana application in a stable, agent-friendly format.

The spec is not "extra documentation."
It is part of the framework itself.

## Why Spec-First

A strong specification layer gives AQAMI several benefits at once:

- better code generation
- safer AI assistance
- inspectable project structure
- more reliable documentation
- cleaner CLI and MCP integration

Without a strong spec, every tool ends up reverse-engineering the codebase.
That is fragile for both humans and agents.

## Spec Scope

An AQAMI project spec should eventually describe:

- package metadata
- target cluster or deployment profile
- program definitions
- accounts and field layouts
- instruction definitions
- PDA rules
- events
- error catalog
- generation hints
- test vectors or example scenarios

## Domain Model

The AQAMI domain model is the conceptual core represented by the spec.
This is the "DM" layer in the broadest sense.

At minimum, the domain model includes:

- `Program`
- `Account`
- `Instruction`
- `InstructionAccount`
- `InstructionArg`
- `Pda`
- `Event`
- `FrameworkError`

The domain model should remain small, explicit, and versionable.
If a feature cannot be represented in the domain model cleanly, that is a signal to redesign the feature instead of papering over it with docs.

## Authoring Rules

Specs should follow these rules:

- prefer explicit names over inferred conventions
- prefer flat, readable structures over deeply nested magic
- include docs for security-relevant fields and instructions
- keep generation hints separate from semantic truth when possible
- avoid ambiguous optionality around signer, mutability, ownership, or seed rules

## Source Of Truth Relationship

The intended relationship is:

```text
YAML or JSON spec
  -> JSON Schema validation
  -> Rust semantic validation
  -> normalized model
  -> code generation and tooling
```

Schema validation catches structural issues.
Rust semantic validation catches domain rules that are awkward to encode purely in JSON Schema.
Normalization then derives deterministic generator-facing identifiers and supported type mappings from the validated model.

## Proposed File Shapes

Initial direction:

- root project spec:
  - `aqami.yaml`
- optional split layout for larger projects:
  - `programs/*.aqami.yaml`
  - `accounts/*.aqami.yaml`
  - `instructions/*.aqami.yaml`

For the earliest version, a single-file project spec is the simplest starting point.

## Required Top-Level Concepts

The initial project spec should likely require:

- `specVersion`
- `package`
- `programs`

And each program should likely contain:

- `name`
- `accounts`
- `instructions`
- optional `pdas`
- optional `events`
- optional `errors`

## Account Modeling Rules

Account definitions should aim to describe:

- account name
- semantic purpose
- field layout
- ownership expectations
- seed relationships when used as a PDA
- versioning or migration notes if relevant later

Future tooling should be able to inspect an account definition and answer:

- what fields exist
- who owns it
- how it is derived
- which instructions read or write it

## Instruction Modeling Rules

Instruction definitions should be explicit about:

- required accounts
- account roles
- signer requirements
- mutability
- PDA expectations
- argument types
- emitted events
- expected errors

The long-term goal is that an agent can generate or review an instruction implementation without guessing the execution surface.

## PDA Modeling Rules

PDA rules should be represented as data, not folklore.

That means a spec should allow tools to reason about:

- seed list
- bump handling
- seed sources
- whether the PDA is derived from args, accounts, constants, or framework metadata

This is one of the highest-value areas for agent support because LLMs frequently make PDA mistakes when left to infer patterns from prose alone.

## Generation Hints

Generation hints can be useful, but they are not the semantic core.

Examples:

- file layout preferences
- whether to generate tests
- whether to emit client SDKs
- whether to generate example CPI helpers

Keep those hints explicit and versioned, but do not blur them with security or business semantics.

## Example Lifecycle

Recommended flow:

1. author or update spec
2. validate against schema
3. run semantic validation
4. inspect normalized model
5. generate code
6. run tests against generated and handwritten code

## What The Spec Should Unlock

If the spec is successful, AQAMI should be able to support:

- project scaffolding from templates
- account and instruction skeleton generation
- client SDK generation
- project introspection for AI agents
- safer MCP tools
- documentation generation

## Initial Deliverables In This Repository

This repository starts with:

- [`schemas/aqami.project.schema.json`](../schemas/aqami.project.schema.json)
- [`examples/specs/escrow.aqami.yaml`](../examples/specs/escrow.aqami.yaml)

These are early foundation artifacts, not final design commitments.
They exist to force clarity early.
