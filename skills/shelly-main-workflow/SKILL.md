---
name: shelly-main-workflow
description: "Trigger: Shelly work, Shelly repository changes, commits, pushes, releases, or documentation. Enforce direct-main workflow and scoped Shelly delivery rules."
license: "MIT"
metadata:
  author: "Gentle AI"
  version: "1.0.0"
---

## Activation Contract

Load for any change in the Shelly repository, including runtime, documentation, tests, workflows, release automation, or sibling integration coordination.

## Hard Rules

- Work directly on `main`; do not create, switch to, push, merge, or propose a branch unless the user explicitly requests a branch.
- Treat an existing non-`main` branch as an incident: stop and ask before changing branch state.
- Require explicit user authorization for commits, pushes, tags, releases, asset publication, or branch deletion.
- Do not create a release for documentation, workflow, or automation-only changes unless the user explicitly requests it.
- Preserve focused tests and documentation with user-visible behavior changes.

## Decision Gates

| Situation | Action |
| --- | --- |
| User explicitly requests a branch | Confirm its purpose, then use that branch only. |
| No branch request | Verify `main` and work there. |
| Release or publish requested | Load `shelly-coordinated-release` and require SemVer rationale. |
| Runtime syntax or command changes | Load `shelly-language-integration`. |
| Sibling repository could be affected | Inspect first; obtain explicit authorization before editing it. |

## Execution Steps

1. Check the current branch and working tree before mutation.
2. Keep the change scoped; use tests first when deterministic behavior is affected.
3. Run proportionate focused verification.
4. Commit and push only when explicitly authorized, always on `main` unless an explicit branch request overrides this rule.
5. Report changed paths, verification, delivery state, and any skipped checks.

## Output Contract

Report the active branch, changed paths, checks, commit/push/tag/release state, and any blocked authorization. State explicitly when no release was created.

## References

- `skills/shelly-language-integration/SKILL.md`
- `skills/shelly-coordinated-release/SKILL.md`
