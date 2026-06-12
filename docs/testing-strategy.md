# Testing Strategy

## Goal

AQAMI should be trusted for security-sensitive development.
That means tests must cover not only handwritten runtime code, but also the framework surfaces that future users and agents will rely on.

## Testing Layers

### 1. Schema Validation Tests

Validate that:

- valid specs pass
- invalid shapes fail clearly
- required fields remain enforced
- version fields behave as expected

### 2. Semantic Validation Tests

Validate domain rules that live beyond raw schema shape:

- duplicate names
- invalid PDA references
- missing instruction account dependencies
- incompatible field definitions
- invalid signer or mutability combinations

### 3. Codegen Golden Tests

Generators should use snapshot or golden-file tests for:

- Rust program skeletons
- generated clients
- generated tests
- generated manifests or metadata

Deterministic output is essential for agent trust.

### 4. Runtime Unit Tests

Test explicit runtime helpers and primitives:

- validation helpers
- serialization boundaries
- instruction argument handling
- error mapping

### 5. Property Tests

Use property-style tests where they provide strong value, especially for:

- normalization
- deterministic naming
- seed handling
- PDA derivation helpers
- reversible formatting or parsing behavior

### 6. Integration Tests

Use `solana-program-test` for behavior that needs realistic execution boundaries:

- account initialization flows
- signer and owner validation
- instruction success and failure cases
- event or log expectations

### 7. End-To-End Generation Tests

At least a small set of example projects should be generated and verified end to end.

Ideal path:

```text
spec fixture
  -> validate
  -> generate
  -> compile
  -> run selected tests
```

## Testing Policy By Change Type

- spec changes: schema, semantic validation, docs, and example fixtures should update together
- generator changes: add or update golden tests and at least one end-to-end example
- runtime changes: unit tests plus integration coverage for behaviorally meaningful changes
- MCP changes: tool contract tests where possible, plus backing CLI or library verification

## Security-Sensitive Expectations

High-risk behavior should have focused tests around:

- PDA derivation
- signer rules
- mutability constraints
- ownership checks
- serialization boundaries
- code generation around account and instruction constraints

## Definition Of Done

A meaningful AQAMI change is not done when it merely compiles.

It is done when:

- the source of truth is clear
- docs and examples are aligned
- the relevant test layer is updated
- the behavior is difficult for future agents to accidentally break
