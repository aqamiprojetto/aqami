# Foundation Infrastructure

## Why

AQAMI is starting from a blank repository, but it aims to become a public Solana framework with an AI-first operating model.
That means the repository needs durable documentation, architectural boundaries, a specification story, and clear agent guidance before framework code starts to accumulate.

## Scope

In scope:

- public repository README
- AI agent working contract
- architecture document
- specification strategy
- MCP positioning
- testing strategy
- machine-readable starter schema
- example spec fixture
- task-spec template

Out of scope:

- runtime crate implementation
- CLI implementation
- generator implementation
- MCP server implementation

## Constraints

- the framework must remain explicit and agent-friendly
- Solana-specific correctness must not be hidden behind convenience magic
- MCP should not become the primary source of truth
- the initial foundation should be coherent enough to guide future crate boundaries

## Options Considered

### 1. Start coding the runtime immediately

Pros:

- faster visible code output

Cons:

- risks drifting into undocumented conventions
- makes later spec and generator design reactive instead of intentional

### 2. Build MCP first

Pros:

- strong grant demo narrative

Cons:

- likely to produce unstable tool contracts
- encourages wrapping undefined internal behavior

### 3. Build documentation and spec foundation first

Pros:

- aligns source of truth early
- improves future agent effectiveness
- creates a cleaner platform for runtime, generator, CLI, and MCP work

Cons:

- slower to show executable framework behavior

## Chosen Approach

Option 3.

AQAMI should establish its documentation, specification model, and architecture before code generation or runtime work begins.
That produces a stronger long-term framework surface and a more credible story for future MCP integration.

## Impacted Areas

- `README.md`
- `AGENTS.md`
- `docs/architecture.md`
- `docs/specification.md`
- `docs/mcp-strategy.md`
- `docs/testing-strategy.md`
- `docs/task-specs/TEMPLATE.md`
- `schemas/aqami.project.schema.json`
- `examples/specs/escrow.aqami.yaml`

## Verification Plan

- review all new docs for internal consistency
- parse the JSON schema successfully
- parse the example YAML successfully
- inspect repository status and resulting file layout

## Follow-Ups

- implement `aqami-spec`
- implement CLI `validate` and `inspect`
- formalize spec normalization rules
- add golden-test strategy once codegen exists
