# Shelly VS Code extension repository

Feature: move Shelly VS Code language support into its own public repository.

## Decisions

- Use a separate public repository named `shelly-vscode` so users can clone and test the extension independently from the Shelly compiler/runtime repository.
- Keep the extension optional and separate from the Shelly installers.
- The main Shelly repository links to the extension repository and does not contain the extension implementation.
- Start with syntax highlighting and language configuration; diagnostics should come later from a safe `shelly check` command that parses without executing scripts.

## Tasks

- [x] Create separate public GitHub repository.
- [x] Move/copy VS Code extension files into the extension repository.
- [x] Remove the extension implementation from the main Shelly repository.
- [x] Update main Shelly docs to link to the extension repository.
- [x] Verify JSON structure and local extension instructions.
- [x] Commit and push the extension repository.
- [ ] Commit and push the main Shelly repo cleanup/link.

## Evidence

- Created public repository: `https://github.com/Soyagvs/shelly-vscode`.
- Extension repo initial commit: `0b3dd90 feat: add Shelly VS Code language support`.
- Created sibling Git repository at `/home/soyagvs/proyectos/personal/shelly-vscode`.
- Extension files copied into repo root:
  - `package.json`
  - `language-configuration.json`
  - `syntaxes/shelly.tmLanguage.json`
  - `README.md`
  - `.gitignore`
  - `LICENSE`
- Updated extension README to use the real clone workflow:
  - `git clone https://github.com/Soyagvs/shelly-vscode.git`
  - `cd shelly-vscode`
  - `code .`
  - `F5`
- Updated Shelly root README to link to `https://github.com/Soyagvs/shelly-vscode`.
- Verification passed:
  - `node -e "for (const f of ['package.json','language-configuration.json','syntaxes/shelly.tmLanguage.json']) JSON.parse(require('fs').readFileSync(f,'utf8'))"`
  - README clone workflow checked for `git clone https://github.com/Soyagvs/shelly-vscode.git`, `cd shelly-vscode`, and `code .`.
  - Main README link checked at `README.md:127`.
  - `test ! -d editors/vscode` passed.
