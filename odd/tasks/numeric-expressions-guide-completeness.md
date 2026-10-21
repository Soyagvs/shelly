# Numeric expressions guide completeness

## Goal
Make `docs/numeric-expressions.md` a complete, approachable guide to Shelly arithmetic: syntax, evaluation order, numeric values, errors, limits, and practical contexts.

## Scope
- `docs/numeric-expressions.md`
- This task record

## Non-goals
- Change numeric parser, runtime, formatting, or context behavior.
- Duplicate the full variables, constants, loops, lists, or exit references.
- Commit, publish, or create a pull request without explicit user approval.

## Tasks

- [x] **NEG-1 — Audit numeric semantics.** Map arithmetic behavior and corroborate proposed claims against source/tests.
- [x] **NEG-2 — Complete arithmetic guidance.** Documented literals, operators, order, unary behavior, references, and numeric representation.
- [x] **NEG-3 — Explain contexts, errors, and limits.** Added practical contexts, rejected forms, division/formatting/precision guidance, and reference links.
- [x] **NEG-4 — Verify the guide.** Ran Markdown, link, and focused executable/source checks for all new behavioral examples.
  - Checks: `git diff --check -- docs/numeric-expressions.md`; fence/local-link validation; targeted numeric, exit-code, argument-index, and list-index tests; source corroboration for malformed forms and formatting.

## Evidence
- Verification passed: all documented supported contexts and rejection classes are corroborated. Literal malformed examples are supported by parser/source and targeted malformed-expression tests.
- The user requires complete documentation of supported and rejected forms, limits, and practical behavior.
- No commit requested; working changes remain uncommitted unless the user explicitly asks.
