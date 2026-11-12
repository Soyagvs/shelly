# Style Context Guide

## Goal

Document that styled values and ordinary blank `shellout ""` calls work in Shelly executable control-flow contexts, with concise runnable examples.

## Scope

Extend the terminal styling guide with examples for functions, conditional/case blocks, loops, error handling, and nesting. Preserve the existing color-policy documentation and link behavior to the executable regression.

## Tasks

- [x] Add a compact control-flow guide with styled and blank output examples.
- [x] Check documentation structure and verify the existing runtime regression.

## Evidence

`docs/terminal-styling.md` now includes one runnable, compact control-flow example covering conditionals, `case`, `for`/`while`/`repeat`, a function, `try`/`catch`/`retry`, and nesting with both `style(...)` and `shellout ""`.

Independent verification confirmed that every documented context is covered by `tests/cli_smoke.rs::style_and_blank_shellout_cover_all_control_flow_contexts`; `cargo test --test cli_smoke style_and_blank_shellout_cover_all_control_flow_contexts -- --exact` passed (1 test).
