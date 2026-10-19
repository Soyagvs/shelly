---
name: shelly-language-integration
description: "Trigger: Shelly language, runtime syntax, or command additions. Keep documentation, LSP, Neovim, and VS Code integrations aligned."
license: "MIT"
metadata:
  author: "Gentle AI"
  version: "1.0.0"
---

## Activation Contract

Load for any Shelly language or runtime syntax/command addition, including parser, diagnostics, documentation, or editor integration work.

## Hard Rules

- Update canonical Shelly documentation and the LSP parser, diagnostics, and tests where applicable.
- Inspect sibling `shelly-nvim` and `shelly-vscode`; if affected, update their syntax/docs or grammar/indentation/tests/docs.
- Obtain explicit user authorization before modifying either independent sibling repository.
- Do not claim editor support until every affected integration is updated or the user explicitly defers it.
- If VS Code extension source changes, select and apply the appropriate SemVer bump before package generation; update all version-coupled metadata and package-verification inputs, generate a new version-named `.vsix` without overwriting any previously versioned release artifact, retain it, report its exact path, and never publish or delete artifacts without authorization.
- Run relevant tests/build checks and report skipped or unavailable checks honestly.

## Decision Gates

| Condition | Action |
| --- | --- |
| Canonical syntax/runtime behavior changes | Update docs, LSP implementation, and applicable tests. |
| Neovim or VS Code is affected | Inspect it; request explicit authorization before edits. |
| VS Code source changes | Select and apply the appropriate SemVer bump first; update version-coupled metadata and verification inputs, then generate and retain a new version-named VSIX without overwriting prior versioned artifacts. |
| Authorization or a check is unavailable | Stop that branch or report it as deferred/skipped; do not imply coverage. |

## Execution Steps

1. Identify the canonical Shelly behavior, docs, parser, diagnostics, and tests.
2. Inspect both sibling repositories and determine impact before requesting changes.
3. Obtain authorization for each affected sibling, then make only scoped updates.
4. If VS Code source changed, choose the appropriate SemVer bump, update every version-coupled metadata and verification input, and apply the bump before package generation.
5. Generate a new version-named VSIX without overwriting a previously versioned release artifact; retain it and record its exact path.
6. Run focused tests, LSP checks, editor tests, and VS Code packaging as applicable.
7. Verify support claims against each updated or explicitly deferred integration.

## Output Contract

Report changed paths, authorization/deferred decisions, checks and exact results, skipped/unavailable checks, and (when applicable) the retained VSIX exact path. State any unsupported editor integration explicitly.

## References

- `odd/tasks/shelly-language-integration-skill.md`
- `/home/soyagvs/.config/opencode/skills/skill-creator/references/skill-style-guide.md`
