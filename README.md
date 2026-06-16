# Agent Translator

Agent Translator is a Rust CLI for inspecting AI agent configuration files across tools such as Claude, Codex, Cursor, and Antigravity.

Modern projects often accumulate several agent-specific setup files: `CLAUDE.md`, `AGENTS.md`, Cursor rules, hook notes, linked Markdown documents, and tool-specific workflows. Agent Translator provides a small common scanner so you can see what exists, compare differences, and prepare for migration between agent ecosystems.

This repository is an early MVP. The current implementation is a CLI and reusable Rust library core. GUI support and real migration writing are planned, but not implemented yet.

## What It Does Today

- Detects known agent setting files:
  - `CLAUDE.md` -> Claude
  - `AGENTS.md` -> Codex
  - `.cursorrules` or `cursor.md` -> Cursor
  - `antigravity.md` -> Antigravity
- Extracts local Markdown links from agent setting files.
- Includes linked `.md` documents in scan results.
- Detects hook-related feature usage using token-based matching.
- Compares detected feature differences across agent settings.
- Provides text and JSON output for automation.
- Produces a migration dry-run plan without writing files.
- Skips `.git`, `.worktrees`, and `target` directories.
- Does not follow symlinked directories while scanning.

## Non-Goals For The Current MVP

- No actual file migration is performed.
- No AI or `codex exec` call is made.
- No GUI is implemented yet.
- No full CommonMark parser is used.
- No remote URL crawling is performed.
- No files are overwritten by the CLI.

## Installation

From source:

```bash
git clone https://github.com/AustinKimDev/agent-translator.git
cd agent-translator
cargo build
```

Run without installing:

```bash
cargo run -- scan .
```

Install locally from the checkout:

```bash
cargo install --path .
agent-translator scan .
```

## Usage

### Scan a project

```bash
agent-translator scan /path/to/project
```

Example output:

```text
scan
root: /path/to/project
settings: 2
- Codex: /path/to/project/AGENTS.md
  linked: /path/to/project/rules/codex-workflow.md
  linked: /path/to/project/rules/shared.md
- Claude: /path/to/project/CLAUDE.md
  feature: Hooks
  linked: /path/to/project/rules/claude-workflow.md
  linked: /path/to/project/rules/shared.md
linked documents: 3
- /path/to/project/rules/claude-workflow.md
- /path/to/project/rules/codex-workflow.md
- /path/to/project/rules/shared.md
```

### Scan as JSON

```bash
agent-translator scan --json /path/to/project
```

The JSON output is intended for scripts, future GUI integration, and migration tooling.

### Compare agent settings

```bash
agent-translator diff /path/to/project
```

Example output:

```text
diff
root: /path/to/project
settings: 2
- Codex: /path/to/project/AGENTS.md
- Claude: /path/to/project/CLAUDE.md
feature mismatch: Hooks
present: Claude
missing: Codex
```

JSON output is also available:

```bash
agent-translator diff --json /path/to/project
```

### Produce a migration dry-run plan

```bash
agent-translator migrate claude codex /path/to/project
```

Example output:

```text
migration dry-run
source: Claude
target: Codex
root: /path/to/project

1. scan source settings
2. collect linked Markdown documents
3. classify unsupported tool-specific features
4. write target draft
```

JSON output:

```bash
agent-translator migrate claude codex --json /path/to/project
```

## CLI Reference

```text
agent-translator

usage:
  agent-translator scan [--json] [path]
  agent-translator diff [--json] [path]
  agent-translator migrate <source> <target> [--json] [path]

tools: claude, codex, antigravity, cursor
```

If `path` is omitted, the current directory is used.

## Output Model

The scanner normalizes discovered settings into:

- project root
- agent tool
- source setting path
- linked Markdown document paths
- detected feature flags

The current normalized feature set contains:

- `Hooks`

This model is intentionally small. It is the shared base for future migration and GUI work.

## Development

Requirements:

- Rust 2024 edition compatible toolchain
- Cargo

Common commands:

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Run the CLI during development:

```bash
cargo run -- scan .
cargo run -- scan --json .
cargo run -- diff .
cargo run -- migrate claude codex .
```

## Project Status

Agent Translator is in early MVP stage.

Implemented:

- Rust CLI and library core
- basic file detection
- local Markdown link extraction
- hook feature detection
- text and JSON output
- dry-run migration plan

Planned:

- richer parsers for each agent ecosystem
- stronger normalized configuration schema
- actual migration draft generation
- Codex-powered migration assistance
- GUI built on top of the same core model

## Contributing

Issues and pull requests are welcome. For now, the highest-value areas are:

- adding tests for real-world agent setting layouts
- improving Markdown parsing coverage
- expanding tool-specific models for Claude, Codex, Cursor, and Antigravity
- designing the migration output format
- adding examples under a dedicated fixtures directory

Before opening a pull request, run:

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

## License

No license has been selected yet. Until a license is added, this repository is public source code but not formally open-source licensed.
