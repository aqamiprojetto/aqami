# MCP Strategy

## Position

MCP can be very valuable for AQAMI, but it should not be the first foundational artifact.

AQAMI should treat MCP as the official agent interface built on top of a stable internal system, not as the primary system itself.

## The Wrong Order

This is the risky order:

1. invent MCP tools first
2. backfill CLI behavior later
3. discover the real model through tool drift

That usually leads to:

- inconsistent semantics
- duplicate business logic
- unstable tool contracts
- generators becoming the accidental source of truth

## The Right Order

Recommended order:

1. define the domain model
2. define the spec format
3. validate it through schema and semantic checks
4. expose stable CLI capabilities
5. wrap those capabilities with MCP

This gives agents a structured interface without creating a second architecture.

## Why MCP Still Matters

Once the underlying surfaces are stable, MCP becomes high leverage because agents can ask structured questions instead of inferring everything from prose and code.

Examples of strong future MCP capabilities:

- `list_templates()`
- `inspect_project()`
- `inspect_program(program_name)`
- `validate_spec(path)`
- `generate_program(template, output_dir)`
- `derive_pda(program, account, inputs)`
- `generate_client_sdk(language)`
- `explain_instruction(program, instruction)`

## Backing Principle

Every MCP capability should correspond to a real framework capability that also exists outside chat.

That usually means:

- a CLI command
- a typed library API
- or both

If a tool only exists for chat convenience and is not grounded in the real system, it will be harder to test, version, and maintain.

## What MCP Should Help With

MCP is especially valuable for:

- project introspection
- template discovery
- validation diagnostics
- safe scaffolding
- code generation entrypoints
- PDA derivation utilities
- SDK generation
- documentation lookup against structured data

## What MCP Should Not Replace

MCP should not replace:

- the machine-readable spec
- CLI automation
- tests
- runtime correctness
- clear repository documentation

If AQAMI depends on MCP to explain behavior that is otherwise hidden, that is a design smell.

## Phase Recommendation

### Before MCP

Build:

- spec model
- schema
- validator
- CLI inspection and generation
- example projects

### With Initial MCP

Expose:

- read-only project inspection
- validation
- template listing
- deterministic generation entrypoints

### Later MCP

Add:

- richer project refactoring assistance
- migration helpers
- compatibility inspection
- agent-safe modification workflows

## Grant Narrative

For a grant application, the strongest technical story is usually not "we built an MCP server."

It is:

- we designed a framework surface that is machine-readable
- we made Solana application structure explicit
- we exposed that structure through stable tooling
- MCP became the natural interface for AI agents because the underlying system deserved it

That is a stronger and more defensible architecture story.
