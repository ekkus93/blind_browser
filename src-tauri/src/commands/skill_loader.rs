use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::Path;

use super::skill_parser::{parse_bundled_skills, parse_skill_document};
use super::*;

pub(crate) const BUNDLED_SKILLS_MARKDOWN: &str = include_str!("../../../docs/SKILLS.md");

pub(crate) fn discover_skills(
    project_root: Option<&Path>,
    user_skill_root: Option<&Path>,
    available_tools: &[AvailableTool],
) -> DiscoveredSkills {
    let available_tool_names = available_tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();
    let mut discovered = HashMap::<String, LoadedSkill>::new();
    let mut diagnostics = SkillDiscoveryDiagnostics::default();

    if let Some(project_root) = project_root {
        load_skills_from_directory(
            &project_root.join(".pi").join("skills"),
            SkillSource::Project,
            &available_tool_names,
            &mut discovered,
            &mut diagnostics,
        );
    }

    if let Some(user_skill_root) = user_skill_root {
        load_skills_from_directory(
            user_skill_root,
            SkillSource::User,
            &available_tool_names,
            &mut discovered,
            &mut diagnostics,
        );
    }

    // Bundled skills are a compile-time asset (include_str! of docs/SKILLS.md); a
    // parse defect (e.g. a malformed requires_confirmation safety flag) is a build
    // error that must fail loudly rather than ship a skill with a silently-wrong
    // confirmation policy. A regression test parses the same constant, so this
    // expect cannot fire in a shipped build that passed CI.
    let bundled_skills = parse_bundled_skills(BUNDLED_SKILLS_MARKDOWN, &available_tool_names)
        .expect("bundled skills (docs/SKILLS.md) must parse; fix the malformed bundled skill");
    for skill in bundled_skills {
        discovered
            .entry(skill.summary.name.clone())
            .or_insert(skill);
    }

    DiscoveredSkills {
        skills: discovered.into_values().collect(),
        diagnostics,
    }
}

fn skill_source_label(source: SkillSource) -> &'static str {
    match source {
        SkillSource::Project => "project",
        SkillSource::User => "user",
        SkillSource::Bundled => "bundled",
    }
}

fn skill_directory_label(path: &Path) -> &str {
    path.file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown-skill")
}

#[derive(Debug, Default, PartialEq, Eq)]
struct SkillEntryWarningSummary {
    skipped_entries: usize,
    error_categories: BTreeMap<&'static str, usize>,
}

fn skill_entry_error_category(kind: io::ErrorKind) -> &'static str {
    match kind {
        io::ErrorKind::PermissionDenied => "permission_denied",
        io::ErrorKind::InvalidData => "invalid_data",
        io::ErrorKind::NotFound => "not_found",
        io::ErrorKind::Interrupted => "interrupted",
        _ => "other_io",
    }
}

fn collect_readable_entries<T, I>(entries: I) -> (Vec<T>, SkillEntryWarningSummary)
where
    I: IntoIterator<Item = io::Result<T>>,
{
    let mut readable = Vec::new();
    let mut warnings = SkillEntryWarningSummary::default();
    for entry in entries {
        match entry {
            Ok(entry) => readable.push(entry),
            Err(error) => {
                warnings.skipped_entries += 1;
                *warnings
                    .error_categories
                    .entry(skill_entry_error_category(error.kind()))
                    .or_insert(0) += 1;
            }
        }
    }
    (readable, warnings)
}

fn load_skills_from_directory(
    skill_root: &Path,
    source: SkillSource,
    available_tool_names: &[ToolName],
    discovered: &mut HashMap<String, LoadedSkill>,
    diagnostics: &mut SkillDiscoveryDiagnostics,
) {
    let source_label = skill_source_label(source);
    let entries = match fs::read_dir(skill_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            diagnostics.push(source_label, "directory_unreadable", 1, None);
            tracing::warn!(
                source = source_label,
                error_kind = ?error.kind(),
                "failed to read skill directory"
            );
            return;
        }
    };

    let (entries, entry_warnings) = collect_readable_entries(entries);
    if entry_warnings.skipped_entries > 0 {
        for (category, count) in &entry_warnings.error_categories {
            diagnostics.push(source_label, category, *count, None);
        }
        tracing::warn!(
            source = source_label,
            skipped_entries = entry_warnings.skipped_entries,
            error_categories = ?entry_warnings.error_categories,
            "skipped unreadable skill directory entries"
        );
    }

    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let directory_name = skill_directory_label(&path).to_string();
        let skill_file_path = path.join("SKILL.md");
        let content = match fs::read_to_string(&skill_file_path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                diagnostics.push(
                    source_label,
                    "manifest_unreadable",
                    1,
                    Some(directory_name.clone()),
                );
                tracing::warn!(
                    source = source_label,
                    skill = %directory_name,
                    error_kind = ?error.kind(),
                    "failed to read SKILL.md"
                );
                continue;
            }
        };

        match parse_skill_document(&content, source, available_tool_names) {
            Ok(skill) => {
                if directory_name != skill.summary.name {
                    diagnostics.push(
                        source_label,
                        "name_mismatch",
                        1,
                        Some(directory_name.clone()),
                    );
                    tracing::warn!(
                        source = source_label,
                        expected = %directory_name,
                        actual = %skill.summary.name,
                        "skipping skill because directory name does not match frontmatter name"
                    );
                    continue;
                }
                discovered
                    .entry(skill.summary.name.clone())
                    .or_insert(skill);
            }
            Err(_error) => {
                diagnostics.push(
                    source_label,
                    "invalid_manifest",
                    1,
                    Some(directory_name.clone()),
                );
                tracing::warn!(
                    source = source_label,
                    skill = %directory_name,
                    error_category = "invalid_manifest",
                    "skipping invalid skill document"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_readable_entries, skill_directory_label, skill_source_label,
        SkillDiscoveryDiagnostics, SkillSource,
    };
    use std::io;
    use std::path::Path;

    #[test]
    fn skill_diagnostics_use_source_and_leaf_name_only() {
        let path = Path::new("/home/private-user/secret-project/.pi/skills/navigation");

        assert_eq!(skill_source_label(SkillSource::Project), "project");
        assert_eq!(skill_source_label(SkillSource::User), "user");
        assert_eq!(skill_source_label(SkillSource::Bundled), "bundled");
        assert_eq!(skill_directory_label(path), "navigation");
        assert!(!skill_directory_label(path).contains("private-user"));
        assert!(!skill_directory_label(path).contains("secret-project"));
    }

    #[test]
    fn typed_skill_diagnostics_merge_without_private_paths() {
        let mut diagnostics = SkillDiscoveryDiagnostics::default();
        diagnostics.push("project", "permission_denied", 1, None);
        diagnostics.push("project", "permission_denied", 2, None);
        diagnostics.push(
            "user",
            "invalid_manifest",
            1,
            Some(String::from("navigation")),
        );
        assert_eq!(diagnostics.warnings.len(), 2);
        assert_eq!(diagnostics.warnings[0].count, 3);
        let encoded = serde_json::to_string(&diagnostics).expect("diagnostics serialize");
        assert!(!encoded.contains("/home/"));
        assert!(!encoded.contains("private-user"));
    }

    #[test]
    fn unreadable_entries_are_aggregated_without_dropping_valid_neighbors() {
        let entries = vec![
            Ok("valid-one"),
            Err(io::Error::from(io::ErrorKind::PermissionDenied)),
            Ok("valid-two"),
            Err(io::Error::from(io::ErrorKind::InvalidData)),
        ];
        let (readable, warnings) = collect_readable_entries(entries);
        assert_eq!(readable, vec!["valid-one", "valid-two"]);
        assert_eq!(warnings.skipped_entries, 2);
        assert_eq!(warnings.error_categories.get("permission_denied"), Some(&1));
        assert_eq!(warnings.error_categories.get("invalid_data"), Some(&1));
        let diagnostic = format!("{warnings:?}");
        assert!(!diagnostic.contains("/home/"));
        assert!(!diagnostic.contains("secret-project"));
    }
}
