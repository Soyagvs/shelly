# VS Code VSIX Versioning

## Objective
Ensure every Shelly VS Code extension source update produces a new versioned VSIX artifact rather than overwriting an existing version's package.

## Delivered policy
- A VS Code extension source change must select and apply the appropriate SemVer bump before package generation.
- Version-coupled metadata and package-verification inputs must be updated with the bump.
- Packaging must create and retain a new version-named VSIX without overwriting a prior versioned artifact, and report its exact path.
- Publishing, uploads, commits, and deletion of package artifacts remain user decisions.

## Tasks
- [x] VVV-1 — Bump the VS Code extension to 0.1.2 and package a new VSIX.
  - Outcome: updated package metadata, lockfile root versions, release badge, and version-coupled package verifier; generated the new artifact.
  - Evidence: `npm test` passed (14 tests); `npm run validate`, `npm run build`, `npm run package`, and `npm run verify-package` passed.
  - Artifact: `/home/soyagvs/proyectos/personal/shelly-vscode/shelly-language-oficial-0.1.2.vsix` (849,723 bytes).
- [x] VVV-2 — Require semantic versioning before VSIX packaging in the integration skill.
  - Outcome: updated `skills/shelly-language-integration/SKILL.md` and refreshed its registry entry.
  - Evidence: independent structural verification passed; the registry contains the exact skill path.

## Verification evidence
- Independent VS Code verification: metadata, badge, verification script, artifact, `npm test`, and `npm run verify-package` all passed.
- Independent skill verification: required headings, SemVer/version-coupling rule, non-overwrite/retention rule, sibling authorization, and registry entry all passed.
- Parent spot check: `npm --prefix /home/soyagvs/proyectos/personal/shelly-vscode run verify-package` passed; SemVer skill assertion passed.
- A first parent text assertion used the wrong exact phrase and exited non-zero after the package verification itself passed; it was corrected and rerun successfully. This was a verification-command typo, not a product failure.
- Native review preflight and assessment were unavailable because the verified package-local Gentle AI binary is missing; independent verification was run instead.

## Next step
Use the language-integration skill for every future Shelly language change; release/publish the retained VSIX only with explicit user authorization.

## Delivery evidence
- Work-unit commit: `937acedce631d18d120e8154df0a4f0e8007fd85` — `feat: support process watch commands` in `shelly-vscode`.
- Companion integration commit: `7b0ffd8cabda495d24e8789cc1cf0bf73dd28ce2` — `feat: highlight process watch commands` in `shelly-nvim`.
