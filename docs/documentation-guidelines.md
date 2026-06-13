# Documentation Guidelines

## Purpose

AQAMI should be heavily documented, but not noisily commented.

For a framework, the goal is not "comment every line."
The goal is:

- public surfaces are easy to understand
- invariants are explicit
- security-relevant behavior is hard to misread
- future contributors and agents do not need to reverse-engineer intent

## What Should Be Documented

AQAMI should document these aggressively:

- public crate APIs
- spec shapes and semantic rules
- runtime invariants
- generator assumptions that affect review or safety
- lifecycle behavior such as `init`, `closeTo`, PDA rules, signer rules, and ownership rules
- typed error surfaces and what each failure means

If a concept affects correctness, security, generation behavior, or agent interpretation, it deserves explicit documentation.

## What Should Be Commented In Code

Comments in code should explain:

- why a rule exists
- what invariant is being protected
- what tradeoff was chosen
- what would be easy to misunderstand from reading the code alone

Good candidates:

- Solana-specific safety rules
- normalization assumptions
- descriptor-generation constraints
- performance-sensitive ownership or allocation choices

Poor candidates:

- restating the code in English
- trivial one-line transformations
- comments that become stale faster than the code around them

## Rustdoc Expectations

As AQAMI's public APIs stabilize, prefer Rustdoc comments on:

- public modules
- public structs and enums
- public functions and methods
- error enums that users or generated code are expected to handle

Rustdoc should explain framework meaning, not just type shape.

For example:

- what this descriptor represents
- when this helper should be used
- what assumptions a validator enforces

## Examples Over Commentary

For workflows and authoring patterns, examples are often better than long comments.

Prefer:

- example specs
- generated examples
- focused documentation snippets

over large narrative comments embedded deep in implementation files.

## Generated Code

Generated code should include concise, high-signal comments only.

It is useful for generated code to expose:

- account metadata summaries
- important lifecycle markers
- descriptor names that help review

It should not drown users in repetitive boilerplate comments.

## Tests As Documentation

Test names and fixtures are part of AQAMI's documentation surface.

Prefer tests that read like executable rules:

- `initialized_instruction_account_requires_payer`
- `arg_bump_requires_known_instruction_argument`
- `rejects_init_and_close_conflict`

When a behavior is subtle, a good test is often more durable than a large inline comment.

## Rule Of Thumb

If a future contributor or agent asks:

"Could I misuse this without realizing it?"

then AQAMI should probably add documentation, a comment, a test, or all three.
