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

## Source Formats

For now, AQAMI source specs should remain human- and agent-friendly text formats:

- YAML for authoring convenience
- JSON for machine interchange and tooling compatibility

Binary schema-driven formats such as Protobuf may become useful later for:

- CLI or MCP transport
- cached compiled-spec artifacts
- remote service boundaries

But they are not the best primary authoring format yet.
At this stage, easy review, editing, prompting, and diffing matter more than binary compactness.

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
- owner expectation
- space requirement for program-owned state
- field layout
- ownership expectations
- seed relationships when used as a PDA
- identity relationships used by instructions, such as authority or beneficiary links
- versioning or migration notes if relevant later

Future tooling should be able to inspect an account definition and answer:

- what fields exist
- who owns it
- how it is derived
- which instructions read or write it

For declared program account types, the owner should be explicit in the spec instead of being implied by framework folklore.
For declared program account types, `space` should also be explicit so generated code and validators do not have to guess allocation size.

## Instruction Modeling Rules

Instruction definitions should be explicit about:

- required accounts
- account roles
- declared account types when an instruction account maps to a program-owned state account
- lifecycle constraints such as initialization, payer, close target, and rent-exempt expectations
- space expectations inherited from declared account types
- account relationship constraints such as `hasOne`
- signer requirements
- mutability
- PDA expectations
- argument types
- emitted events
- expected errors

The long-term goal is that an agent can generate or review an instruction implementation without guessing the execution surface.

When an instruction account represents a concrete program account type, the spec should declare that explicitly rather than relying on naming conventions alone.
When an instruction account is initialized or closed in a specific instruction, that lifecycle behavior should be declared on the instruction account binding rather than hidden in generator templates.
When an initialized instruction account targets a program-owned account type, the declared account type should carry explicit `space` so both generation and runtime validation remain deterministic.
When an instruction depends on identity links inside account data, those links should be declared explicitly through structured constraints such as `hasOne` instead of being implied by field names or handwritten comments.
When an instruction closes an account, the close target should be explicit through `closeTo`, and both the closing account and close target should be writable in the modeled surface.

Recommended `hasOne` shape:

```yaml
constraints:
  hasOne:
    - field: "authority"
      account: "authority"
```

AQAMI should prefer this explicit object form over shorthand strings because agents and generators should not have to infer whether the compared field and instruction account name are meant to be the same thing.

## PDA Modeling Rules

PDA rules should be represented as data, not folklore.

That means a spec should allow tools to reason about:

- seed list
- bump handling
- seed sources
- whether the PDA is derived from args, accounts, constants, or framework metadata

This is one of the highest-value areas for agent support because LLMs frequently make PDA mistakes when left to infer patterns from prose alone.

For PDA bumps, AQAMI should also prefer explicit metadata over convention.
The current spec supports:

- `canonical`
- `arg`
- `account_field`

Recommended shape:

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

When `bump.kind` is `arg`, the referenced instruction argument should use AQAMI type `u8`.
For arg-backed seeds and bumps, AQAMI runtime validation should consume typed instruction arguments directly rather than rely on hidden Borsh, JSON, or string-based packing conventions.
For `account_field` seeds, the referenced instruction account should declare `accountType`, and the referenced field should use AQAMI type `pubkey`.
Runtime validation for those seeds should consume explicit typed pubkey-field context supplied by generated instruction boundaries rather than decode raw account bytes through hidden layout assumptions.

That keeps future generation, diagnostics, and MCP tooling aligned around a machine-readable derivation model instead of handwritten assumptions.

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
