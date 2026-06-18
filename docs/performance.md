# Performance

## Position

Performance should be a real AQAMI value.
But AQAMI is a layered system, so the performance bar must be applied with precision rather than as a blanket excuse for complexity everywhere.

## Where Performance Matters Most

Highest priority:

- future runtime crates
- generated on-chain program code
- account validation helpers
- serialization and instruction-boundary logic
- PDA and account constraint helpers that may sit on critical paths

Medium priority:

- CLI repeated operations
- spec loading and validation
- code generation throughput

Lower priority:

- purely descriptive or documentation-only tooling paths

## AQAMI Rule Of Thumb

If a code path is likely to influence:

- on-chain compute behavior
- runtime allocations
- repeated developer or agent generation loops

then performance should be considered early.

If a code path is mostly:

- repository metadata
- infrequent setup logic
- prose rendering

then clarity should usually win unless there is evidence of waste worth removing.

## What To Optimize First

- redundant parsing or compilation of immutable artifacts
- unnecessary `clone()` calls on hot paths
- repeated string allocations where stable identifiers can be derived once
- generator instability that causes unnecessary rewrites or diff churn
- hidden abstraction layers that make runtime costs hard to reason about

## What To Avoid

- premature low-signal micro-optimizations in non-runtime tooling
- performance claims without identifying the actual hot path
- convenience abstractions that obscure runtime cost
- optimizing generated or runtime code in ways that make audits substantially harder without strong payoff

## Current Foundation Implications

At the current repository stage, the practical performance guidance is:

- cache immutable validation artifacts such as compiled schemas
- keep normalized spec data explicit and reusable
- generate deterministic outputs so agents and humans do not waste work on unstable diffs
- delay runtime-specific micro-optimization until runtime crates and generated Solana code exist

The current repository policy now reflects that split:

- CI should use dependency and target caching aggressively, but keep regular verification on debug and test profiles for faster feedback
- shipped host-side release artifacts should use a maximally optimized Cargo release profile with `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, and stripped symbols
- future Solana SBF or other target-specific release pipelines should be tuned explicitly instead of assuming host release settings are automatically sufficient
