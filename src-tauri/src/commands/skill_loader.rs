use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::skill_parser::{parse_bundled_skills, parse_skill_document};
use super::*;

pub(crate) const BUNDLED_SKILLS_MARKDOWN: &str = include_str!("../../../docs/SKILLS.md");

pub(crate) fn discover_skills(
    project_root: Option<&Path>,
    user_skill_root: Option<&Path>,
    available_tools: &[AvailableTool],
) -> Vec<LoadedSkill> {
    let available_tool_names = available_tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();
    let mut discovered = HashMap::<String, LoadedSkill>::new();

    if let Some(project_root) = project_root {
        load_skills_from_directory(
            &project_root.join(".pi").join("skills"),
            SkillSource::Project,
            &available_tool_names,
            &mut discovered,
        );
    }

    if let Some(user_skill_root) = user_skill_root {
        load_skills_from_directory(
            user_skill_root,
            SkillSource::User,
            &available_tool_names,
            &mut discovered,
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

    discovered.into_values().collect()
}

fn load_skills_from_directory(
    skill_root: &Path,
    source: SkillSource,
    available_tool_names: &[ToolName],
    discovered: &mut HashMap<String, LoadedSkill>,
) {
    let entries = match fs::read_dir(skill_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            tracing::warn!(
                path = %skill_root.display(),
                error = %error,
                "failed to read skill directory"
            );
            return;
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_file_path = path.join("SKILL.md");
        let content = match fs::read_to_string(&skill_file_path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                tracing::warn!(
                    path = %skill_file_path.display(),
                    error = %error,
                    "failed to read SKILL.md"
                );
                continue;
            }
        };

        match parse_skill_document(&content, source, available_tool_names) {
            Ok(skill) => {
                let directory_name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                if directory_name != skill.summary.name {
                    tracing::warn!(
                        path = %skill_file_path.display(),
                        expected = directory_name,
                        actual = %skill.summary.name,
                        "skipping skill because directory name does not match frontmatter name"
                    );
                    continue;
                }
                discovered
                    .entry(skill.summary.name.clone())
                    .or_insert(skill);
            }
            Err(error) => {
                tracing::warn!(
                    path = %skill_file_path.display(),
                    error = %error,
                    "skipping invalid skill document"
                );
            }
        }
    }
}
