# Documentation style — a portable maintainer contract

Use this contract to keep Shelly documentation scannable, technically accurate, and portable across GitHub Markdown renderers. It defines presentation conventions; it does not change language or runtime behavior.

## Title and lead

- Use one outcome-oriented H1: name the reader's goal, not an internal file label.
- Follow it with a short lead that states who the page helps and what they can accomplish.
- Put the happy path before background, edge cases, or exhaustive reference material.

## Heading order and navigation

- Keep heading levels sequential: H1, then H2, then H3; do not skip levels for visual size.
- Use short, recognizable H2 headings. Semantic icons may support recognition, but the words must carry the meaning.
- Link related pages with descriptive text. Do not use color, position, or an icon as the only navigation cue.

## Icons and callouts

- Use a small, consistent set of Unicode semantic icons in headings or labels, such as `🚀` for starting, `📖` for learning, and `🛠️` for maintenance.
- Icons are supplemental and must not be the only way to identify a section.
- Use portable Markdown blockquotes for callouts:

  > **Note:** State the useful constraint or next action in text.

  Do not use inline CSS, custom HTML callout components, or color-only warnings.

## Code examples

- Preserve verified commands, flags, paths, and links exactly unless a replacement is verified locally.
- Use fenced code blocks with a language identifier (`sh`, `powershell`, `shelly`, or the most accurate available label).
- Keep examples minimal and runnable. Show expected outcomes in prose when useful; do not fake terminal output.
- Distinguish platform-specific commands with a heading or label.

## Tables and lists

- Use tables for compact comparisons with a clear header row; keep cells short and avoid paragraphs inside cells.
- Use ordered lists for sequences and unordered lists for options or features.
- Prefer short lists and progressive disclosure over dense walls of prose.

## Local assets and brand presentation

- Use repository-local artwork, such as `assets/icons/shelly.svg`, with a meaningful alt description.
- Do not add new external image services, inline CSS, or ANSI escape sequences to documentation.
- Purple is Shelly's brand presentation. Never use purple, or any color, as the sole signal for status, emphasis, or meaning.

## Accessibility and portability

- Use descriptive link text, meaningful alt text, and sufficient text context around icons.
- Ensure the page remains understandable when colors, images, or emoji are unavailable.
- Prefer GitHub-compatible Markdown over renderer-specific extensions.
- Keep heading structure, code fences, and links valid for screen readers and keyboard navigation.

## Terminal output versus documentation presentation

Terminal ANSI output is runtime behavior: it describes escape sequences and colors emitted by Shelly or a user's terminal. Documentation presentation is the visual structure of Markdown pages on GitHub. Keep these concerns separate: never paste ANSI escape sequences into Markdown to create a purple visual system, and never imply that a documentation color changes terminal behavior.
