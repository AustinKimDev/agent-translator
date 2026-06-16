use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTool {
    Claude,
    Codex,
    Antigravity,
    Cursor,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureFlag {
    Hooks,
}

impl AgentTool {
    pub fn from_file_name(file_name: &str) -> Self {
        match file_name.to_ascii_lowercase().as_str() {
            "claude.md" => Self::Claude,
            "agents.md" => Self::Codex,
            ".cursorrules" | "cursor.md" => Self::Cursor,
            "antigravity.md" => Self::Antigravity,
            _ => Self::Unknown,
        }
    }

    pub fn from_cli_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            "antigravity" => Some(Self::Antigravity),
            "cursor" => Some(Self::Cursor),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::Codex => "Codex",
            Self::Antigravity => "Antigravity",
            Self::Cursor => "Cursor",
            Self::Unknown => "Unknown",
        }
    }
}

impl FeatureFlag {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hooks => "Hooks",
        }
    }
}

pub fn extract_markdown_links(text: &str) -> Vec<String> {
    let mut links = Vec::new();

    for line in text.lines() {
        collect_inline_markdown_links(line, &mut links);

        if let Some(target) = reference_markdown_target(line) {
            links.push(target);
        }
    }

    links
}

fn collect_inline_markdown_links(line: &str, links: &mut Vec<String>) {
    let mut remaining = line;

    while let Some(label_end) = remaining.find("](") {
        let target_start = label_end + 2;
        let after_start = &remaining[target_start..];
        let Some(target_end) = after_start.find(')') else {
            break;
        };

        if let Some(target) = local_markdown_target(&after_start[..target_end]) {
            links.push(target);
        }

        remaining = &after_start[target_end + 1..];
    }
}

fn reference_markdown_target(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if !trimmed.starts_with('[') {
        return None;
    }

    let label_end = trimmed.find("]:")?;
    local_markdown_target(&trimmed[label_end + 2..])
}

fn local_markdown_target(raw_target: &str) -> Option<String> {
    let target = raw_target
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches('<')
        .trim_matches('>');
    let path_without_fragment = target.split('#').next().unwrap_or(target);
    let lower = path_without_fragment.to_ascii_lowercase();

    if path_without_fragment.is_empty()
        || lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with('#')
        || !lower.ends_with(".md")
    {
        return None;
    }

    Some(path_without_fragment.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingFile {
    pub tool: AgentTool,
    pub path: PathBuf,
    pub linked_documents: Vec<PathBuf>,
    pub features: Vec<FeatureFlag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    pub root: PathBuf,
    pub settings: Vec<SettingFile>,
    pub linked_documents: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedProject {
    pub root: PathBuf,
    pub agents: Vec<NormalizedAgentConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedAgentConfig {
    pub tool: AgentTool,
    pub source_path: PathBuf,
    pub linked_markdown_paths: Vec<PathBuf>,
    pub features: Vec<FeatureFlag>,
}

pub fn scan_directory(root: &Path) -> io::Result<ScanResult> {
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    files.sort();

    let mut settings = Vec::new();
    let mut linked_documents = BTreeSet::new();

    for path in files {
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let tool = AgentTool::from_file_name(file_name);

        if tool == AgentTool::Unknown {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let base = path.parent().unwrap_or(root);
        let mut setting_links = Vec::new();

        for link in extract_markdown_links(&content) {
            let linked_path = base.join(link);

            if linked_path.exists() {
                linked_documents.insert(linked_path.clone());
                setting_links.push(linked_path);
            }
        }

        setting_links.sort();
        let features = detect_feature_flags(&content);
        settings.push(SettingFile {
            tool,
            path,
            linked_documents: setting_links,
            features,
        });
    }

    Ok(ScanResult {
        root: root.to_path_buf(),
        settings,
        linked_documents: linked_documents.into_iter().collect(),
    })
}

pub fn normalize_scan_result(scan: &ScanResult) -> NormalizedProject {
    NormalizedProject {
        root: scan.root.clone(),
        agents: scan
            .settings
            .iter()
            .map(|setting| NormalizedAgentConfig {
                tool: setting.tool,
                source_path: setting.path.clone(),
                linked_markdown_paths: setting.linked_documents.clone(),
                features: setting.features.clone(),
            })
            .collect(),
    }
}

pub fn detect_feature_flags(content: &str) -> Vec<FeatureFlag> {
    let lower = content.to_ascii_lowercase();
    let mut features = BTreeSet::new();

    if has_hook_token(&lower) {
        features.insert(FeatureFlag::Hooks);
    }

    features.into_iter().collect()
}

fn has_hook_token(content: &str) -> bool {
    content
        .split(|value: char| !value.is_ascii_alphanumeric())
        .any(|token| matches!(token, "hook" | "hooks"))
}

pub fn migration_plan(source: AgentTool, target: AgentTool, root: &Path) -> String {
    let mut output = format!(
        "migration dry-run\nsource: {}\ntarget: {}\nroot: {}\n\n",
        source.as_str(),
        target.as_str(),
        root.display()
    );

    for (index, step) in MIGRATION_STEPS.iter().enumerate() {
        output.push_str(&format!("{}. {step}\n", index + 1));
    }

    output
}

const MIGRATION_STEPS: [&str; 4] = [
    "scan source settings",
    "collect linked Markdown documents",
    "classify unsupported tool-specific features",
    "write target draft",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

struct CommandArgs<'a> {
    format: OutputFormat,
    values: Vec<&'a str>,
}

fn parse_command_args(args: &[String], start: usize) -> CommandArgs<'_> {
    let mut format = OutputFormat::Text;
    let mut values = Vec::new();

    for arg in args.iter().skip(start) {
        if arg == "--json" {
            format = OutputFormat::Json;
        } else {
            values.push(arg.as_str());
        }
    }

    CommandArgs { format, values }
}

pub fn run_cli(args: &[String]) -> Result<String, String> {
    match args.get(1).map(String::as_str) {
        Some("scan") => {
            let command_args = parse_command_args(args, 2);
            let root = command_args
                .values
                .first()
                .map(|value| Path::new(*value))
                .unwrap_or_else(|| Path::new("."));
            let scan = scan_directory(root).map_err(|error| error.to_string())?;

            match command_args.format {
                OutputFormat::Text => Ok(format_scan_result(&scan)),
                OutputFormat::Json => Ok(format_scan_json(&scan)),
            }
        }
        Some("diff") => {
            let command_args = parse_command_args(args, 2);
            let root = command_args
                .values
                .first()
                .map(|value| Path::new(*value))
                .unwrap_or_else(|| Path::new("."));
            let scan = scan_directory(root).map_err(|error| error.to_string())?;

            match command_args.format {
                OutputFormat::Text => Ok(format_diff_result(&scan)),
                OutputFormat::Json => Ok(format_diff_json(&scan)),
            }
        }
        Some("migrate") => {
            let command_args = parse_command_args(args, 2);
            let source = parse_tool_arg(command_args.values.first().copied(), "source")?;
            let target = parse_tool_arg(command_args.values.get(1).copied(), "target")?;
            let root = command_args
                .values
                .get(2)
                .map(|value| Path::new(*value))
                .unwrap_or_else(|| Path::new("."));

            match command_args.format {
                OutputFormat::Text => Ok(migration_plan(source, target, root)),
                OutputFormat::Json => Ok(format_migration_json(source, target, root)),
            }
        }
        Some("-h" | "--help" | "help") | None => Ok(usage()),
        Some(command) => Err(format!("unknown command: {command}\n\n{}", usage())),
    }
}

fn parse_tool_arg(value: Option<&str>, name: &str) -> Result<AgentTool, String> {
    let Some(value) = value else {
        return Err(format!("missing {name} tool\n\n{}", usage()));
    };

    AgentTool::from_cli_name(value).ok_or_else(|| {
        format!("unknown {name} tool: {value}\nexpected one of: claude, codex, antigravity, cursor")
    })
}

fn format_migration_json(source: AgentTool, target: AgentTool, root: &Path) -> String {
    format!(
        "{{\"command\":\"migrate\",\"source\":{},\"target\":{},\"root\":{},\"steps\":[{}]}}\n",
        json_string(source.as_str()),
        json_string(target.as_str()),
        json_path(root),
        json_string_array(MIGRATION_STEPS)
    )
}

fn format_scan_result(scan: &ScanResult) -> String {
    let mut output = format!(
        "scan\nroot: {}\nsettings: {}\n",
        scan.root.display(),
        scan.settings.len()
    );

    for setting in &scan.settings {
        output.push_str(&format!(
            "- {:?}: {}\n",
            setting.tool,
            setting.path.display()
        ));

        for feature in &setting.features {
            output.push_str(&format!("  feature: {feature:?}\n"));
        }

        for linked_document in &setting.linked_documents {
            output.push_str(&format!("  linked: {}\n", linked_document.display()));
        }
    }

    output.push_str(&format!(
        "linked documents: {}\n",
        scan.linked_documents.len()
    ));

    for linked_document in &scan.linked_documents {
        output.push_str(&format!("- {}\n", linked_document.display()));
    }

    output
}

fn format_scan_json(scan: &ScanResult) -> String {
    let project = normalize_scan_result(scan);
    let agents = project
        .agents
        .iter()
        .map(format_normalized_agent_json)
        .collect::<Vec<_>>()
        .join(",");
    let linked_documents = scan
        .linked_documents
        .iter()
        .map(|path| json_path(path))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"command\":\"scan\",\"root\":{},\"agents\":[{agents}],\"linked_documents\":[{linked_documents}]}}\n",
        json_path(&project.root)
    )
}

fn format_normalized_agent_json(agent: &NormalizedAgentConfig) -> String {
    let linked_markdown_paths = agent
        .linked_markdown_paths
        .iter()
        .map(|path| json_path(path))
        .collect::<Vec<_>>()
        .join(",");
    let features = agent
        .features
        .iter()
        .map(|feature| json_string(feature.as_str()))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"tool\":{},\"source_path\":{},\"linked_markdown_paths\":[{linked_markdown_paths}],\"features\":[{features}]}}",
        json_string(agent.tool.as_str()),
        json_path(&agent.source_path)
    )
}

fn format_diff_result(scan: &ScanResult) -> String {
    let mut output = format!(
        "diff\nroot: {}\nsettings: {}\n",
        scan.root.display(),
        scan.settings.len()
    );

    for setting in &scan.settings {
        output.push_str(&format!(
            "- {:?}: {}\n",
            setting.tool,
            setting.path.display()
        ));
    }

    for mismatch in feature_mismatches(scan) {
        output.push_str(&format!(
            "feature mismatch: {}\npresent: {}\nmissing: {}\n",
            mismatch.feature.as_str(),
            mismatch.present.join(", "),
            mismatch.missing.join(", ")
        ));
    }

    output
}

fn format_diff_json(scan: &ScanResult) -> String {
    let feature_mismatches = feature_mismatches(scan)
        .iter()
        .map(format_feature_mismatch_json)
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"command\":\"diff\",\"root\":{},\"settings_count\":{},\"feature_mismatches\":[{feature_mismatches}]}}\n",
        json_path(&scan.root),
        scan.settings.len()
    )
}

struct FeatureMismatch {
    feature: FeatureFlag,
    present: Vec<String>,
    missing: Vec<String>,
}

fn feature_mismatches(scan: &ScanResult) -> Vec<FeatureMismatch> {
    let mut features = BTreeSet::new();
    for setting in &scan.settings {
        for feature in &setting.features {
            features.insert(*feature);
        }
    }

    features
        .into_iter()
        .filter_map(|feature| {
            let present: Vec<String> = scan
                .settings
                .iter()
                .filter(|setting| setting.features.contains(&feature))
                .map(|setting| setting.tool.as_str().to_string())
                .collect();
            let missing: Vec<String> = scan
                .settings
                .iter()
                .filter(|setting| !setting.features.contains(&feature))
                .map(|setting| setting.tool.as_str().to_string())
                .collect();

            if present.is_empty() || missing.is_empty() {
                None
            } else {
                Some(FeatureMismatch {
                    feature,
                    present,
                    missing,
                })
            }
        })
        .collect()
}

fn format_feature_mismatch_json(mismatch: &FeatureMismatch) -> String {
    format!(
        "{{\"feature\":{},\"present\":[{}],\"missing\":[{}]}}",
        json_string(mismatch.feature.as_str()),
        json_string_array(mismatch.present.iter().map(String::as_str)),
        json_string_array(mismatch.missing.iter().map(String::as_str))
    )
}

fn usage() -> String {
    "agent-translator\n\nusage:\n  agent-translator scan [--json] [path]\n  agent-translator diff [--json] [path]\n  agent-translator migrate <source> <target> [--json] [path]\n\ntools: claude, codex, antigravity, cursor\n".to_string()
}

fn json_path(path: &Path) -> String {
    json_string(&path.to_string_lossy())
}

fn json_string_array<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    values
        .into_iter()
        .map(json_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");

    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            value if value.is_control() => {
                output.push_str(&format!("\\u{:04x}", value as u32));
            }
            value => output.push(value),
        }
    }

    output.push('"');
    output
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }

            collect_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }

    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|value| value.to_str()),
        Some(".git" | ".worktrees" | "target")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_known_agent_tool_from_file_names() {
        assert_eq!(AgentTool::from_file_name("CLAUDE.md"), AgentTool::Claude);
        assert_eq!(AgentTool::from_file_name("AGENTS.md"), AgentTool::Codex);
        assert_eq!(AgentTool::from_file_name(".cursorrules"), AgentTool::Cursor);
        assert_eq!(
            AgentTool::from_file_name("antigravity.md"),
            AgentTool::Antigravity
        );
        assert_eq!(AgentTool::from_file_name("notes.md"), AgentTool::Unknown);
    }

    #[test]
    fn extracts_local_markdown_links_from_inline_and_reference_links() {
        let text =
            "Read [rules](rules/worktree.md) and [ports][ports].\n\n[ports]: ./rules/ports.md\n";

        assert_eq!(
            extract_markdown_links(text),
            vec![
                "rules/worktree.md".to_string(),
                "./rules/ports.md".to_string()
            ]
        );
    }

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

    #[test]
    fn scan_directory_marks_hook_features() {
        let root = unique_test_dir("scan_directory_marks_hook_features");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("CLAUDE.md"), "Use hooks for format checks").unwrap();

        let scan = scan_directory(&root).unwrap();

        assert_eq!(scan.settings[0].features, vec![FeatureFlag::Hooks]);
    }

    #[test]
    fn hook_feature_detection_ignores_substrings() {
        assert_eq!(detect_feature_flags("Configure webhook delivery"), vec![]);
        assert_eq!(
            detect_feature_flags("Use hooks for format checks"),
            vec![FeatureFlag::Hooks]
        );
        assert_eq!(
            detect_feature_flags("hooks: run cargo fmt"),
            vec![FeatureFlag::Hooks]
        );
    }

    #[cfg(unix)]
    #[test]
    fn scan_directory_does_not_follow_symlinked_directories() {
        let root = unique_test_dir("scan_directory_does_not_follow_symlinked_directories");
        let outside =
            unique_test_dir("scan_directory_does_not_follow_symlinked_directories_outside");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("CLAUDE.md"), "outside settings").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("linked")).unwrap();

        let scan = scan_directory(&root).unwrap();

        assert!(scan.settings.is_empty());
    }

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

    #[test]
    fn migration_plan_describes_source_target_and_steps() {
        let plan = migration_plan(
            AgentTool::Claude,
            AgentTool::Codex,
            std::path::Path::new("."),
        );

        assert!(plan.contains("source: Claude"));
        assert!(plan.contains("target: Codex"));
        assert!(plan.contains("1. scan source settings"));
        assert!(plan.contains("4. write target draft"));
    }

    #[test]
    fn cli_scan_prints_detected_settings() {
        let root = unique_test_dir("cli_scan_prints_detected_settings");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("AGENTS.md"), "Codex instructions").unwrap();
        let args = vec![
            "agent-translator".to_string(),
            "scan".to_string(),
            root.display().to_string(),
        ];

        let output = run_cli(&args).unwrap();

        assert!(output.contains("settings: 1"));
        assert!(output.contains("Codex"));
        assert!(output.contains("AGENTS.md"));
    }

    #[test]
    fn cli_migrate_prints_dry_run_plan() {
        let args = vec![
            "agent-translator".to_string(),
            "migrate".to_string(),
            "claude".to_string(),
            "codex".to_string(),
            ".".to_string(),
        ];

        let output = run_cli(&args).unwrap();

        assert!(output.contains("migration dry-run"));
        assert!(output.contains("source: Claude"));
        assert!(output.contains("target: Codex"));
    }

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

    #[test]
    fn cli_scan_json_prints_normalized_project() {
        let root = unique_test_dir("cli_scan_json_prints_normalized_project");
        std::fs::create_dir_all(root.join("rules")).unwrap();
        std::fs::write(
            root.join("CLAUDE.md"),
            "Use hooks\nRead [worktree](rules/worktree.md)",
        )
        .unwrap();
        std::fs::write(root.join("rules/worktree.md"), "worktree rule").unwrap();
        let args = vec![
            "agent-translator".to_string(),
            "scan".to_string(),
            "--json".to_string(),
            root.display().to_string(),
        ];

        let output = run_cli(&args).unwrap();

        assert!(output.starts_with("{"));
        assert!(output.contains("\"command\":\"scan\""));
        assert!(output.contains("\"agents\""));
        assert!(output.contains("\"tool\":\"Claude\""));
        assert!(output.contains("\"features\":[\"Hooks\"]"));
        assert!(output.contains("\"linked_markdown_paths\""));
    }

    #[test]
    fn cli_diff_json_prints_feature_mismatches() {
        let root = unique_test_dir("cli_diff_json_prints_feature_mismatches");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("CLAUDE.md"), "Use hooks for format checks").unwrap();
        std::fs::write(root.join("AGENTS.md"), "Use the same style guide").unwrap();
        let args = vec![
            "agent-translator".to_string(),
            "diff".to_string(),
            "--json".to_string(),
            root.display().to_string(),
        ];

        let output = run_cli(&args).unwrap();

        assert!(output.starts_with("{"));
        assert!(output.contains("\"command\":\"diff\""));
        assert!(output.contains("\"feature_mismatches\""));
        assert!(output.contains("\"feature\":\"Hooks\""));
        assert!(output.contains("\"present\":[\"Claude\"]"));
        assert!(output.contains("\"missing\":[\"Codex\"]"));
    }

    #[test]
    fn cli_migrate_json_prints_dry_run_plan() {
        let args = vec![
            "agent-translator".to_string(),
            "migrate".to_string(),
            "claude".to_string(),
            "codex".to_string(),
            "--json".to_string(),
            ".".to_string(),
        ];

        let output = run_cli(&args).unwrap();

        assert!(output.starts_with("{"));
        assert!(output.contains("\"command\":\"migrate\""));
        assert!(output.contains("\"source\":\"Claude\""));
        assert!(output.contains("\"target\":\"Codex\""));
        assert!(output.contains("\"steps\""));
    }

    fn unique_test_dir(name: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("agent-translator-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }
}
