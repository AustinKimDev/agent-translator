# Bootstrap Agent Translator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first testable Rust CLI/library slice for scanning AI agent setting files, extracting linked Markdown documents, reporting feature differences, and producing migration dry-run text.

**Architecture:** Put reusable behavior in `src/lib.rs` and keep `src/main.rs` as a thin CLI adapter. The library exposes small value types for agent tools, discovered setting files, Markdown links, scan results, and migration plans.

**Tech Stack:** Rust 2024, Cargo, Rust standard library only.

---

## File Structure

- Create: `docs/PRD.md` for product scope.
- Create: `docs/superpowers/plans/2026-06-16-bootstrap-agent-translator.md` for this implementation plan.
- Create: `src/lib.rs` for core domain model, scanner, Markdown link extraction, and migration dry-run planner.
- Modify: `src/main.rs` for CLI argument parsing and output.
- Test: unit tests inside `src/lib.rs`.

### Task 1: Core Agent Tool Model

**Files:**
- Create: `src/lib.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn detects_known_agent_tool_from_file_names() {
    assert_eq!(AgentTool::from_file_name("CLAUDE.md"), AgentTool::Claude);
    assert_eq!(AgentTool::from_file_name("AGENTS.md"), AgentTool::Codex);
    assert_eq!(AgentTool::from_file_name(".cursorrules"), AgentTool::Cursor);
    assert_eq!(AgentTool::from_file_name("antigravity.md"), AgentTool::Antigravity);
    assert_eq!(AgentTool::from_file_name("notes.md"), AgentTool::Unknown);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test detects_known_agent_tool_from_file_names`

Expected: compile failure because `AgentTool` does not exist.

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTool {
    Claude,
    Codex,
    Antigravity,
    Cursor,
    Unknown,
}
```

Add `AgentTool::from_file_name(file_name: &str) -> Self`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test detects_known_agent_tool_from_file_names`

Expected: PASS.

### Task 2: Markdown Link Extraction

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn extracts_local_markdown_links_from_inline_and_reference_links() {
    let text = "Read [rules](rules/worktree.md) and [ports][ports].\n\n[ports]: ./rules/ports.md\n";

    assert_eq!(
        extract_markdown_links(text),
        vec!["rules/worktree.md".to_string(), "./rules/ports.md".to_string()]
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test extracts_local_markdown_links_from_inline_and_reference_links`

Expected: compile failure because `extract_markdown_links` does not exist.

- [ ] **Step 3: Write minimal implementation**

Implement `extract_markdown_links(text: &str) -> Vec<String>` that collects local `.md` targets from inline links and reference definitions.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test extracts_local_markdown_links_from_inline_and_reference_links`

Expected: PASS.

### Task 3: Directory Scan

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn scan_directory_includes_settings_and_linked_docs() {
    let root = unique_test_dir("scan_directory_includes_settings_and_linked_docs");
    std::fs::create_dir_all(root.join("rules")).unwrap();
    std::fs::write(root.join("CLAUDE.md"), "Read [worktree](rules/worktree.md)").unwrap();
    std::fs::write(root.join("rules/worktree.md"), "worktree rule").unwrap();

    let scan = scan_directory(&root).unwrap();

    assert_eq!(scan.settings.len(), 1);
    assert_eq!(scan.settings[0].tool, AgentTool::Claude);
    assert_eq!(scan.linked_documents.len(), 1);
    assert!(scan.linked_documents[0].ends_with("rules/worktree.md"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test scan_directory_includes_settings_and_linked_docs`

Expected: compile failure because `scan_directory` and result types do not exist.

- [ ] **Step 3: Write minimal implementation**

Add `SettingFile`, `ScanResult`, and `scan_directory(root: &Path) -> io::Result<ScanResult>`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test scan_directory_includes_settings_and_linked_docs`

Expected: PASS.

### Task 4: CLI Scan and Migration Dry Run

**Files:**
- Modify: `src/main.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn migration_plan_describes_source_target_and_steps() {
    let plan = migration_plan(AgentTool::Claude, AgentTool::Codex, std::path::Path::new("."));

    assert!(plan.contains("source: Claude"));
    assert!(plan.contains("target: Codex"));
    assert!(plan.contains("1. scan source settings"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test migration_plan_describes_source_target_and_steps`

Expected: compile failure because `migration_plan` does not exist.

- [ ] **Step 3: Write minimal implementation**

Add `migration_plan(source: AgentTool, target: AgentTool, root: &Path) -> String` and a small CLI dispatcher in `main`.

- [ ] **Step 4: Run verification**

Run: `cargo test`

Expected: all tests pass.

Run: `cargo run -- scan .`

Expected: exits successfully and prints scan summary.

### Task 5: Hook Feature Detection and Diff CLI

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing hook feature test**

```rust
#[test]
fn scan_directory_marks_hook_features() {
    let root = unique_test_dir("scan_directory_marks_hook_features");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("CLAUDE.md"), "Use hooks for format checks").unwrap();

    let scan = scan_directory(&root).unwrap();

    assert_eq!(scan.settings[0].features, vec![FeatureFlag::Hooks]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test scan_directory_marks_hook_features`

Expected: compile failure because `FeatureFlag` and `SettingFile::features` do not exist.

- [ ] **Step 3: Add minimal hook feature model**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureFlag {
    Hooks,
}
```

Add `detect_feature_flags(content: &str) -> Vec<FeatureFlag>` and store the result on each `SettingFile`.

- [ ] **Step 4: Write the failing diff CLI test**

```rust
#[test]
fn cli_diff_reports_tool_and_feature_differences() {
    let root = unique_test_dir("cli_diff_reports_tool_and_feature_differences");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("CLAUDE.md"), "Use hooks for format checks").unwrap();
    std::fs::write(root.join("AGENTS.md"), "Use the same style guide").unwrap();
    let args = vec![
        "agent-translator".to_string(),
        "diff".to_string(),
        root.display().to_string(),
    ];

    let output = run_cli(&args).unwrap();

    assert!(output.contains("settings: 2"));
    assert!(output.contains("Claude"));
    assert!(output.contains("Codex"));
    assert!(output.contains("feature mismatch: Hooks"));
}
```

- [ ] **Step 5: Run test to verify it fails**

Run: `cargo test cli_diff_reports_tool_and_feature_differences`

Expected: FAIL with `unknown command: diff`.

- [ ] **Step 6: Add minimal diff implementation**

Add `diff [path]` to `run_cli`, format the scan result, and print `feature mismatch: Hooks` when at least one setting has `Hooks` and another setting does not.

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test cli_diff_reports_tool_and_feature_differences`

Expected: PASS.

### Task 6: Review Fixes, Normalization, and JSON Output

**Files:**
- Modify: `src/lib.rs`
- Modify: `docs/PRD.md`
- Modify: `docs/superpowers/plans/2026-06-16-bootstrap-agent-translator.md`

- [ ] **Step 1: Write failing tests for review issues**

```rust
#[test]
fn hook_feature_detection_ignores_substrings() {
    assert_eq!(detect_feature_flags("Configure webhook delivery"), vec![]);
    assert_eq!(detect_feature_flags("Use hooks for format checks"), vec![FeatureFlag::Hooks]);
    assert_eq!(detect_feature_flags("hooks: run cargo fmt"), vec![FeatureFlag::Hooks]);
}

#[cfg(unix)]
#[test]
fn scan_directory_does_not_follow_symlinked_directories() {
    let root = unique_test_dir("scan_directory_does_not_follow_symlinked_directories");
    let outside = unique_test_dir("scan_directory_does_not_follow_symlinked_directories_outside");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("CLAUDE.md"), "outside settings").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("linked")).unwrap();

    let scan = scan_directory(&root).unwrap();

    assert!(scan.settings.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
- `cargo test hook_feature_detection_ignores_substrings`
- `cargo test scan_directory_does_not_follow_symlinked_directories`

Expected: both fail against the initial substring detection and symlink-following directory traversal.

- [ ] **Step 3: Implement minimal fixes**

Use `DirEntry::file_type()` for traversal and detect `hook`/`hooks` as tokens, not arbitrary substrings.

- [ ] **Step 4: Write failing normalization test**

```rust
#[test]
fn normalizes_scan_result_for_migration_and_gui_consumers() {
    let root = unique_test_dir("normalizes_scan_result_for_migration_and_gui_consumers");
    std::fs::create_dir_all(root.join("rules")).unwrap();
    std::fs::write(
        root.join("CLAUDE.md"),
        "Use hooks for format checks\nRead [worktree](rules/worktree.md)",
    )
    .unwrap();
    std::fs::write(root.join("rules/worktree.md"), "worktree rule").unwrap();

    let scan = scan_directory(&root).unwrap();
    let project = normalize_scan_result(&scan);

    assert_eq!(project.root, root);
    assert_eq!(project.agents.len(), 1);
    assert_eq!(project.agents[0].tool, AgentTool::Claude);
    assert!(project.agents[0].source_path.ends_with("CLAUDE.md"));
    assert_eq!(project.agents[0].linked_markdown_paths.len(), 1);
    assert_eq!(project.agents[0].features, vec![FeatureFlag::Hooks]);
}
```

- [ ] **Step 5: Implement minimal normalization model**

Add `NormalizedProject`, `NormalizedAgentConfig`, and `normalize_scan_result`.

- [ ] **Step 6: Write failing JSON output tests**

Add tests for:
- `scan --json <path>`
- `diff --json <path>`
- `migrate <source> <target> --json <path>`

- [ ] **Step 7: Implement JSON output without dependencies**

Add `--json` parsing, manual JSON escaping, and JSON formatters for scan, diff, and migrate.
