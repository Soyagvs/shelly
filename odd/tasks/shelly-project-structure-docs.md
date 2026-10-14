# Shelly project structure docs

Feature: document recommended Shelly project layouts for small, medium, and large scripts so users can keep automation modular and readable.

## Scope

- Add a dedicated docs page for project structure.
- Explain when to use `main.shelly`, `consts/`, `functions/`, `utils/`, `setup/`, and `scripts/`.
- Include small and advanced layouts with examples.
- Link the guide from the language guide and README documentation tables.

## Tasks

- [x] Write the project structure guide.
- [x] Link the guide from public documentation indexes.
- [x] Verify documentation references and markdown structure.

## Evidence

- Added `docs/project-structure.md` with example-first layouts for tiny, small modular, medium, and advanced Shelly projects.
- Documented roles for `main.shelly`, `consts/`, `functions/`, `utils/`, `setup/`, `scripts/`, and optional `modules/` or domain folders.
- Explained `import` versus `include` as reusable exported modules versus intentional setup/executed files.
- Linked the guide from `docs/language.md` and `README.md` documentation tables.
- Verification PASS: `docs/project-structure.md` exists; links found in `docs/language.md` and `README.md`; no stale `example-01.shelly` references in changed docs; snippets use documented forms (`import`, `include`, `export`, `call`, `if`, `exists`, `shellout`).

