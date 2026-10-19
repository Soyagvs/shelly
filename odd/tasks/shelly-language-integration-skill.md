# Shelly Language Integration Skill

## Objective
Create a project skill that makes every Shelly language addition update the primary documentation, LSP, Neovim integration, and VS Code integration; when VS Code changes, it must build and retain a VSIX artifact.

## Delivered scope
- Created `skills/shelly-language-integration/SKILL.md` as a reusable project runtime contract.
- Requires documentation and LSP alignment for language additions.
- Requires inspecting and updating affected Neovim and VS Code integrations; modifying either independent sibling repository requires explicit user authorization.
- Requires `npm run package` and retention/reporting of the generated VSIX when VS Code extension sources change.
- Refreshed `.atl/skill-registry.md` and its cache using `gentle-ai skill-registry refresh`.

## Constraints
- The skill does not require semantic tokens or unrelated editor features.
- It does not authorize commits, publishing, uploads, or deletion of package artifacts.

## Tasks
- [x] SLI-1 — Create the project language-integration skill.
  - Evidence: valid trigger-first frontmatter and all required sections independently verified.
- [x] SLI-2 — Refresh and verify skill discovery.
  - Evidence: `gentle-ai skill-registry refresh` succeeded; registry contains the exact skill path; structural verification and parent spot check passed.

## Verification evidence
- Independent structural review found no defects.
- `git diff --check -- skills/shelly-language-integration/SKILL.md .atl/skill-registry.md .atl/.skill-registry.cache.json` passed.
- Registry path presence check passed.
- Native review preflight and assessment were unavailable because the verified package-local Gentle AI binary is missing; independent verification was run instead.

## Next step
Future Shelly language changes must load `skills/shelly-language-integration/SKILL.md` before implementation.

## Delivery evidence
- Work-unit commit: `096e598bd415be34a5f64639c038f933e707d311` — `chore: add language integration skill`.
