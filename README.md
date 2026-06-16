# Agent Translator

![status](https://img.shields.io/badge/status-early%20MVP-yellow)
![language](https://img.shields.io/badge/language-Rust-orange)
![interface](https://img.shields.io/badge/interface-CLI-blue)

Inspect, compare, and prepare migrations between AI agent configuration files.

Agent Translator looks for files such as `CLAUDE.md`, `AGENTS.md`, `.cursorrules`, and `antigravity.md`, follows their local Markdown links, and reports what differs between agent setups. It is built in Rust and currently ships as a small CLI plus reusable library core.

**Status:** early MVP. It scans and reports. It does not rewrite your files yet.

## Contents

- [Highlights](#highlights)
- [Quick Start](#quick-start)
- [Examples](#examples)
- [Supported Inputs](#supported-inputs)
- [JSON Output](#json-output)
- [Why Use It?](#why-use-it)
- [What It Does Not Do Yet](#what-it-does-not-do-yet)
- [Development](#development)
- [Roadmap](#roadmap)
- [License](#license)

## Highlights

- Finds known AI agent setting files in a project tree.
- Detects Claude, Codex, Cursor, and Antigravity settings.
- Extracts local linked `.md` documents from agent instructions.
- Includes linked documents in scan results.
- Detects hook-related configuration as a feature flag.
- Compares feature mismatches across agent settings.
- Emits human-readable text or machine-readable JSON.
- Produces a migration dry-run plan.
- Avoids `.git`, `.worktrees`, and `target`.
- Skips symlinked directories while scanning.
- Uses the Rust standard library only.

## Quick Start

```bash
git clone https://github.com/AustinKimDev/agent-translator.git
cd agent-translator
cargo run -- scan .
```

Install from a local checkout:

```bash
cargo install --path .
agent-translator scan .
```

## Examples

### Scan a project

```bash
agent-translator scan /path/to/project
```

Example:

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

### Compare settings

```bash
agent-translator diff /path/to/project
```

Example:

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

### Preview a migration

```bash
agent-translator migrate claude codex /path/to/project
```

Example:

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

## Supported Inputs

| File name | Tool |
| --- | --- |
| `CLAUDE.md` | Claude |
| `AGENTS.md` | Codex |
| `.cursorrules` | Cursor |
| `cursor.md` | Cursor |
| `antigravity.md` | Antigravity |

Agent Translator currently extracts local Markdown links from:

```md
[shared rules](rules/shared.md)

[shared]: rules/shared.md
```

Remote URLs, anchors-only links, and non-Markdown files are ignored.

## JSON Output

Every command that reports project state can emit JSON.

```bash
agent-translator scan --json /path/to/project
agent-translator diff --json /path/to/project
agent-translator migrate claude codex --json /path/to/project
```

`scan --json` returns a normalized project model:

```json
{
  "command": "scan",
  "root": "/path/to/project",
  "agents": [
    {
      "tool": "Claude",
      "source_path": "/path/to/project/CLAUDE.md",
      "linked_markdown_paths": ["/path/to/project/rules/shared.md"],
      "features": ["Hooks"]
    }
  ],
  "linked_documents": ["/path/to/project/rules/shared.md"]
}
```

The normalized model is intentionally small:

- project root
- agent tool
- source setting path
- linked Markdown document paths
- detected feature flags

Current feature flags:

- `Hooks`

## Why Use It?

Use Agent Translator when a project has more than one agent setup and you want to answer questions like:

- Which agent settings exist in this repository?
- Which linked instruction files are part of the effective setup?
- Does one tool rely on a feature another tool does not support?
- What would a migration workflow need to inspect before writing a target config?

It is meant to be a boring inspection layer first. Migration and GUI work should build on top of a dependable scanner.

## What It Does Not Do Yet

- It does not write migrated config files.
- It does not call an AI model or `codex exec`.
- It does not implement a GUI.
- It does not fully parse CommonMark.
- It does not crawl network links.
- It does not infer the full semantics of every agent ecosystem.
- It does not overwrite project files.

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

## Development

Requirements:

- Rust 2024 edition compatible toolchain
- Cargo

Run the checks:

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Run the CLI locally:

```bash
cargo run -- scan .
cargo run -- scan --json .
cargo run -- diff .
cargo run -- migrate claude codex .
```

## Roadmap

- Add fixtures from real-world agent configurations.
- Expand the normalized schema for tool-specific settings.
- Model more feature flags beyond `Hooks`.
- Add richer parsers for Claude, Codex, Cursor, and Antigravity.
- Generate migration draft files.
- Add Codex-powered migration assistance.
- Build a GUI on the same Rust core.

## Contributing

Issues and pull requests are welcome.

Good first areas:

- Add scan fixtures for real projects.
- Improve local Markdown link extraction.
- Add tests for edge cases in JSON output.
- Document agent-specific setting conventions.
- Propose the migration output format.

Before opening a pull request:

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

## License

No license has been selected yet. Until a license is added, this repository is public source code but not formally open-source licensed.
